"""Hold host disturbers for the length of a quiet capture.

Design: docs/specs/2026-10-04-capture-disturber-hold-design.md. capture-meta
drives the lifecycle and owns capture.json; this module owns the host side:
what is held, the host hold file, applying and undoing each item, the
journal scan, and the guard's watch loop. Every command goes through a Host,
which tests replace with fake_capture_host.FakeHost.
"""
import contextlib
import datetime
import fcntl
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import time

HOLD_NAME = "capture-meta.hold.json"
JOB_START = "7d4958e842da4a758f6c1cdc7b36dcc5"
UNIT_STARTED = "39f53479d3a045ac8e11786248231fbf"
START_FAILED = "be02cf6855d2428ba40df7e9d022f03d"
RESTART = "5eb03494b6584870a536b337290809b3"
ACTIVATIONS = (JOB_START, UNIT_STARTED, START_FAILED)
POWER_OFF_WAIT_S = 5
GUARD_RETRY_S = 30
GUARD_RETRIES = 20
COMMAND_TIMEOUT_S = 30
JOURNAL_TIMEOUT_S = 120


class Refused(Exception):
    """Exit 1: the run must not proceed."""


class CannotRun(Exception):
    """Exit 2: the helper could not do its job."""


@contextlib.contextmanager
def guarded(path):
    path = pathlib.Path(path)
    guard = path.with_name(path.name + ".d")
    try:
        guard.mkdir(exist_ok=True)
        fd = os.open(guard, os.O_RDONLY)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX)
            yield
        finally:
            os.close(fd)
    except OSError as error:
        raise CannotRun(f"lock guard {guard}: {error}") from error


def rfc3339(us):
    return datetime.datetime.fromtimestamp(us / 1_000_000).astimezone().replace(microsecond=0).isoformat()


class Host:
    """The real machine: commands, sysfs, clock and pids the hold touches."""

    def __init__(self, env=None):
        self.env = dict(os.environ if env is None else env)
        runtime = self.env.get("XDG_RUNTIME_DIR")
        if not runtime:
            raise CannotRun("XDG_RUNTIME_DIR is unset; the hold file has nowhere to live")
        self.runtime = pathlib.Path(runtime)
        home = self.env.get("XDG_CONFIG_HOME") or str(pathlib.Path(self.env.get("HOME", "~")).expanduser() / ".config")
        self.config = pathlib.Path(home) / "niri-material" / "capture-hold"
        self.sysfs = pathlib.Path(self.env.get("CAPTURE_META_SYSFS", "/sys/class/drm"))

    def run(self, *command, extra_env=None, timeout=COMMAND_TIMEOUT_S):
        try:
            return subprocess.run(command, capture_output=True, text=True, check=False, timeout=timeout,
                                  env={**self.env, **(extra_env or {})})
        except subprocess.TimeoutExpired as error:
            raise CannotRun(f"{' '.join(command)}: no answer in {timeout} s") from error
        except OSError as error:
            raise CannotRun(f"{' '.join(command)}: {error}") from error

    def has(self, name):
        return shutil.which(name, path=self.env.get("PATH")) is not None

    def pid_alive(self, pid):
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return False
        except PermissionError:
            return True
        return True

    def now_us(self):
        return time.time_ns() // 1000

    def sleep(self, seconds):
        time.sleep(seconds)


def checked(host, *command, extra_env=None):
    result = host.run(*command, extra_env=extra_env)
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).strip() or f"exit {result.returncode}"
        raise CannotRun(f"{' '.join(command)}: {detail}")
    return result.stdout


def systemctl(manager):
    return ("systemctl", "--user") if manager == "user" else ("systemctl",)


def list_timers(host, manager, all_):
    command = (*systemctl(manager), "list-timers", *(("--all",) if all_ else ()), "--output=json", "--no-pager")
    try:
        rows = json.loads(checked(host, *command))
        return {row["unit"]: row["activates"] for row in rows}
    except (json.JSONDecodeError, TypeError, KeyError) as error:
        raise CannotRun(f"{' '.join(command)}: unexpected output: {error}") from error


def unit_props(host, manager, unit, *props):
    out = checked(host, *systemctl(manager), "show", unit, *(f"--property={p}" for p in props))
    values = dict(line.split("=", 1) for line in out.splitlines() if "=" in line)
    return {p: values.get(p, "") for p in props}


def declared_services(host):
    try:
        text = host.config.read_text()
    except FileNotFoundError:
        return None
    except (OSError, UnicodeError) as error:
        raise CannotRun(f"{host.config}: {error}") from error
    return [line for line in (raw.split("#", 1)[0].strip() for raw in text.splitlines()) if line]


def desktop_socket(host):
    inherited = host.env.get("NIRI_SOCKET")
    candidates = [inherited] if inherited else sorted(str(p) for p in host.runtime.glob("niri.*.sock"))
    live = [s for s in candidates
            if host.run("niri", "msg", "version", extra_env={"NIRI_SOCKET": s}).returncode == 0]
    if len(live) > 1:
        raise CannotRun(f"more than one live niri socket in {host.runtime}: {', '.join(live)}; "
                        "set NIRI_SOCKET to the desktop's")
    return live[0] if live else None


def lit_connectors(host):
    """Connected, enabled connectors. A connector whose state cannot be read refuses the hold:
    it might be a lit monitor that would then go unwatched."""
    names = []
    for path in sorted(host.sysfs.glob("card*-*")):
        try:
            lit = ((path / "status").read_text().strip() == "connected"
                   and (path / "enabled").read_text().strip() == "enabled")
        except OSError as error:
            raise CannotRun(f"cannot tell whether {path.name} is a lit monitor: {error}") from error
        if lit:
            names.append(path.name)
    return names


def dpms(host, connector):
    try:
        return (host.sysfs / connector / "dpms").read_text().strip()
    except OSError as error:
        raise CannotRun(f"cannot read {connector} dpms: {error}") from error


def unit_item(kind, unit):
    return {"kind": kind, "unit": unit, "action": "stopped", "restore": f"systemctl --user start {unit}"}


def plan_hold(host):
    """Discover what to hold. Changes nothing on the host."""
    active = list_timers(host, "user", all_=False)
    timer_map = {"user": list_timers(host, "user", all_=True), "system": list_timers(host, "system", all_=True)}
    items, not_held, active_at_hold, invocations = [], [], [], {}
    for timer in sorted(active):
        if unit_props(host, "user", timer, "Transient")["Transient"] == "yes":
            not_held.append({"kind": "timer", "unit": timer, "reason": "transient"})
        else:
            items.append(unit_item("timer", timer))
    for manager in ("user", "system"):
        for service in sorted(set(timer_map[manager].values())):
            props = unit_props(host, manager, service, "InvocationID", "ActiveState")
            if props["InvocationID"]:
                invocations.setdefault(manager, {})[service] = props["InvocationID"]
            if props["ActiveState"] in ("active", "activating", "deactivating"):
                active_at_hold.append({"manager": manager, "unit": service})
    declared = declared_services(host)
    for unit in declared or []:
        props = unit_props(host, "user", unit, "LoadState", "ActiveState")
        if props["LoadState"] == "not-found":
            raise CannotRun(f"{host.config} names {unit}, which does not exist")
        if props["ActiveState"] in ("active", "activating"):
            items.append(unit_item("service", unit))
        else:
            not_held.append({"kind": "service", "unit": unit, "reason": "not_running"})
    socket = desktop_socket(host)
    if socket:
        if host.has("noctalia"):
            try:
                locked = json.loads(checked(host, "noctalia", "msg", "status")).get("locked")
            except (json.JSONDecodeError, AttributeError) as error:
                raise CannotRun(f"noctalia msg status: unexpected output: {error}") from error
            items.append({"kind": "idle", "action": "caffeine-enabled", "prior": "unknown", "locked": locked,
                          "socket": socket, "restore": "noctalia msg caffeine-disable"})
        else:
            not_held.append({"kind": "idle", "reason": "noctalia not on PATH"})
        items.append({"kind": "monitors", "action": "powered-off", "socket": socket,
                      "connectors": lit_connectors(host),
                      "restore": f"NIRI_SOCKET={socket} niri msg action power-on-monitors"})
    return {"items": items, "not_held": not_held, "socket": socket,
            "config": str(host.config) if declared is not None else "absent",
            "desktop": "present" if socket else "absent", "timer_map": timer_map,
            "invocations": invocations, "active_at_hold": active_at_hold,
            "system_timers": sorted(timer_map["system"])}


def hold_file(lock_file):
    return pathlib.Path(lock_file).with_name(HOLD_NAME)


def wake_file(lock_file, hold):
    return pathlib.Path(lock_file).with_name(f"capture-meta.wakes.{hold['key']}.jsonl")


def run_key(run_dir):
    return hashlib.sha256(str(pathlib.Path(run_dir).resolve()).encode()).hexdigest()[:12]


def read_hold(path):
    try:
        text = pathlib.Path(path).read_text()
    except FileNotFoundError:
        return None
    except (OSError, UnicodeError) as error:
        raise CannotRun(f"hold file {path}: {error}") from error
    try:
        hold = json.loads(text)
    except json.JSONDecodeError as error:
        raise CannotRun(f"hold file {path} is unreadable ({error}); restore its items by hand from the "
                        "run's capture.json, then remove it") from error
    if (not isinstance(hold, dict) or not isinstance(hold.get("items"), list)
            or not isinstance(hold.get("run_id"), str) or not isinstance(hold.get("run_dir"), str)):
        raise CannotRun(f"hold file {path} has no run_id/run_dir/items; restore its items by hand "
                        "from the run's capture.json, then remove it")
    return hold


def write_hold(path, hold):
    path = pathlib.Path(path)
    tmp = path.with_name(f"{path.name}.tmp.{os.getpid()}")
    try:
        tmp.write_text(json.dumps(hold, indent=2) + "\n")
        tmp.replace(path)
    except OSError as error:
        raise CannotRun(f"cannot write hold file {path}: {error}") from error


def names_run(hold, run_id, run_dir):
    return (hold is not None and hold["run_id"] == run_id
            and hold["run_dir"] == str(pathlib.Path(run_dir).resolve()))


def hold_names_run(lock_file, run_id, run_dir):
    with guarded(lock_file):
        return names_run(read_hold(hold_file(lock_file)), run_id, run_dir)


def create_hold(lock_file, run_id, run_dir, owner_pid, plan, now_us):
    key = run_key(run_dir)
    hold = {"run_id": run_id, "run_dir": str(pathlib.Path(run_dir).resolve()), "owner_pid": owner_pid,
            "key": key, "guard": f"capture-meta-guard-{key}.service", "started_us": now_us,
            "socket": plan["socket"], "timer_map": plan["timer_map"], "invocations": plan["invocations"],
            "items": [], "connectors": []}
    path = hold_file(lock_file)
    with guarded(lock_file):
        if read_hold(path) is not None:
            raise CannotRun(f"{path} exists: a hold is already recorded; run `capture-meta restore`")
        write_hold(path, hold)
    return hold


def act(host, item):
    if item["kind"] in ("timer", "service"):
        checked(host, "systemctl", "--user", "stop", item["unit"])
    elif item["kind"] == "idle":
        checked(host, "noctalia", "msg", "caffeine-enable")
    elif item["kind"] == "monitors":
        checked(host, "niri", "msg", "action", "power-off-monitors", extra_env={"NIRI_SOCKET": item["socket"]})
    else:
        raise CannotRun(f"unknown hold item kind {item['kind']!r}")


def _held(host, lock_file, run_id, run_dir):
    """The hold file, if this run may still add to it. Called under guarded(), so a restore that
    has started (it marks the file under the same lock) is always seen."""
    path = hold_file(lock_file)
    hold = read_hold(path)
    if not names_run(hold, run_id, run_dir):
        raise CannotRun(f"hold file {path} is gone or names another run: the guard restored the host "
                        "after the owner died; stopping")
    if hold.get("restoring"):
        raise CannotRun(f"hold for run {run_id} is being restored; holding nothing more")
    if not host.pid_alive(hold["owner_pid"]):
        raise CannotRun(f"owner pid {hold['owner_pid']} of run {run_id} is dead; holding nothing more")
    return path, hold


def wait_powered_off(host, connectors):
    deadline = host.now_us() + POWER_OFF_WAIT_S * 1_000_000
    while True:
        states = {c: dpms(host, c) for c in connectors}
        on = [c for c, state in states.items() if state != "Off"]
        if not on:
            return
        if host.now_us() >= deadline:
            raise CannotRun(f"{', '.join(f'{c} still reports dpms {states[c]}' for c in on)} "
                            f"{POWER_OFF_WAIT_S} s after power-off-monitors; a monitor wake could not be "
                            "seen, so the run cannot be held")
        host.sleep(0.25)


def apply_hold(host, lock_file, run_id, run_dir, plan):
    """Act on each planned item, appending it to the hold file first (write-ahead)."""
    for item in plan["items"]:
        with guarded(lock_file):
            path, hold = _held(host, lock_file, run_id, run_dir)
            hold["items"].append(item)
            write_hold(path, hold)
            act(host, item)
    monitors = next((i for i in plan["items"] if i["kind"] == "monitors"), None)
    if monitors:
        wait_powered_off(host, monitors["connectors"])
    with guarded(lock_file):
        path, hold = _held(host, lock_file, run_id, run_dir)
        hold["connectors"] = monitors["connectors"] if monitors else []
        hold["started_us"] = host.now_us()
        write_hold(path, hold)
    return hold


def undo(host, item):
    """Restore one item. Returns a note or None; raises CannotRun when it fails."""
    kind = item["kind"]
    if kind in ("timer", "service"):
        result = host.run("systemctl", "--user", "start", item["unit"])
        if result.returncode == 0:
            return None
        if unit_props(host, "user", item["unit"], "LoadState")["LoadState"] == "not-found":
            return f"{item['unit']} no longer exists"
        raise CannotRun(f"systemctl --user start {item['unit']}: {(result.stderr or result.stdout).strip()}")
    if kind in ("idle", "monitors") and not pathlib.Path(item["socket"]).exists():
        return "desktop gone; " + ("caffeine left as it is" if kind == "idle" else "monitors left as they are")
    if kind == "idle":
        checked(host, "noctalia", "msg", "caffeine-disable")
        return None
    if kind == "monitors":
        checked(host, "niri", "msg", "action", "power-on-monitors", extra_env={"NIRI_SOCKET": item["socket"]})
        return None
    raise CannotRun(f"unknown hold item kind {kind!r}")


def read_wakes(lock_file, hold):
    try:
        lines = wake_file(lock_file, hold).read_text().splitlines()
    except FileNotFoundError:
        return []
    except (OSError, UnicodeError) as error:
        raise CannotRun(f"wake log: {error}") from error
    wakes = []
    for line in lines:
        try:
            wakes.append(json.loads(line))
        except json.JSONDecodeError:
            wakes.append({"at_us": hold["started_us"], "connector": "?", "error": f"unreadable wake entry {line!r}"})
    return wakes


def still_held(hold):
    """Items not yet restored. `items` never shrinks: it is the scan's input until the scan is durable."""
    return hold.get("held", hold["items"])


def restore_transaction(host, lock_file, run_id, run_dir, by, record=None):
    """Restore this run's hold as one transaction under guarded() (spec §4.3).

    Marks the hold `restoring` and fixes `until_us` (once, ever), undoes the
    still-held items in reverse order best-effort, records what remains in
    `held` (`items` keeps every item ever held, so a retried scan sees the
    same input), calls record(hold, wakes, attempt), and only after that
    returns removes the hold file and wake log, if nothing is still held.
    A kill anywhere leaves the hold file for the next attempt; a competing
    attempt waits on the lock. Returns (hold, attempt), or None if no hold
    names the run. `attempt["undone"]` counts the items this attempt tried.
    """
    path = hold_file(lock_file)
    with guarded(lock_file):
        hold = read_hold(path)
        if not names_run(hold, run_id, run_dir):
            return None
        hold["restoring"] = True
        hold.setdefault("until_us", host.now_us())
        pending = still_held(hold)
        hold["held"] = pending
        write_hold(path, hold)
        try:
            wakes = read_wakes(lock_file, hold)
        except CannotRun as error:
            hold["wake_error"] = str(error)
            wakes = []
        failures, notes, remaining = [], [], []
        for item in reversed(pending):
            try:
                note = undo(host, item)
            except CannotRun as error:
                failures.append({**{k: item[k] for k in ("kind", "unit") if k in item},
                                 "error": str(error), "restore": item["restore"]})
                remaining.insert(0, item)
                continue
            if note:
                notes.append(note)
        attempt = {"at": rfc3339(host.now_us()), "by": by, "undone": len(pending), "failures": failures}
        if notes:
            attempt["notes"] = notes
        hold["held"] = remaining
        write_hold(path, hold)
        if record is not None:
            record(hold, wakes, attempt)
        if not remaining:
            remove_hold(lock_file, hold)
        return hold, attempt


def remove_hold(lock_file, hold):
    """Delete the hold file and wake log. Caller holds guarded()."""
    path = hold_file(lock_file)
    try:
        wake_file(lock_file, hold).unlink(missing_ok=True)
        path.unlink(missing_ok=True)
    except OSError as error:
        raise CannotRun(f"cannot remove hold file {path}: {error}") from error


def finalize(lock_file, run_id, run_dir):
    """Remove a fully restored hold left behind by a kill after its record was saved.
    Returns the hold it removed (for its guard and owner), or None."""
    with guarded(lock_file):
        hold = read_hold(hold_file(lock_file))
        if not names_run(hold, run_id, run_dir) or still_held(hold):
            return None
        remove_hold(lock_file, hold)
        return hold


def journal(host, manager, since_us):
    flag = "--user" if manager == "user" else "--system"
    command = ("journalctl", flag, "-o", "json", "--no-pager", f"--since=@{since_us // 1_000_000 - 1}")
    result = host.run(*command, timeout=JOURNAL_TIMEOUT_S)
    if result.returncode != 0:
        raise CannotRun(f"{' '.join(command)}: {(result.stderr or result.stdout).strip() or f'exit {result.returncode}'}")
    out = result.stdout
    entries = []
    for line in out.splitlines():
        try:
            entries.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise CannotRun(f"journalctl {flag}: malformed entry: {error}") from error
    return entries


def _disturbance(at_us, manager, unit, kind):
    return {"at": rfc3339(at_us), "at_us": at_us, "manager": manager, "unit": unit, "kind": kind}


def scan(host, hold, wakes):
    """Disturbances in [started_us, until_us): see the spec, §5."""
    start, end = hold["started_us"], hold["until_us"]
    timer_map = {m: {**hold["timer_map"].get(m, {}), **list_timers(host, m, all_=True)} for m in ("user", "system")}
    held = {i["unit"] for i in hold["items"] if i["kind"] in ("timer", "service")}
    events = []
    for manager in ("user", "system"):
        unit_key, inv_key = ("USER_UNIT", "USER_INVOCATION_ID") if manager == "user" else ("UNIT", "INVOCATION_ID")
        for e in journal(host, manager, start):
            try:
                at = int(e["__REALTIME_TIMESTAMP"])
            except (KeyError, ValueError) as error:
                raise CannotRun(f"journal entry without a usable __REALTIME_TIMESTAMP: {error}") from error
            if start <= at < end and e.get(unit_key):
                events.append((at, manager, e.get("MESSAGE_ID"), e[unit_key], e.get(inv_key)))
    events.sort(key=lambda event: event[0])
    found = []
    for at, manager, mid, unit, _ in events:
        if unit.endswith(".timer") and mid == UNIT_STARTED:
            held_timer = manager == "user" and unit in held
            found.append(_disturbance(at, manager, unit, "held-started" if held_timer else "timer-added"))
            timer_map[manager][unit] = unit[: -len(".timer")] + ".service"
        elif mid == RESTART:
            found.append(_disturbance(at, manager, unit, "restart"))
    fired = {m: set(timer_map[m].values()) for m in ("user", "system")}
    seen = set()
    for n, (at, manager, mid, unit, inv) in enumerate(events):
        if mid not in ACTIVATIONS or unit.endswith(".timer"):
            continue
        if inv:
            if (manager, unit, inv) in seen or hold["invocations"].get(manager, {}).get(unit) == inv:
                continue
            seen.add((manager, unit, inv))
        if unit in fired[manager]:
            found.append(_disturbance(at, manager, unit, "timer-fired"))
        if manager == "user" and unit in held:
            found.append(_disturbance(at, manager, unit, "held-started"))
    for wake in wakes:
        if not start <= wake["at_us"] < end:
            continue
        kind = "monitor-unwatched" if "error" in wake else "monitor-woke"
        found.append(_disturbance(wake["at_us"], "sysfs", wake["connector"], kind))
    found.sort(key=lambda d: d["at_us"])
    return ("disturbed" if found else "clean"), found


def _log_wake(lock_file, hold, record):
    try:
        with open(wake_file(lock_file, hold), "a") as stream:
            stream.write(json.dumps(record) + "\n")
    except (OSError, UnicodeError) as error:
        raise CannotRun(f"wake log: {error}") from error


def guard_loop(host, lock_file, run_id, run_dir, on_owner_dead, iterations=None):
    """Watch the owner and the held monitors once a second (spec §4.4).

    Each round reads the hold and polls the monitors under guarded(), the
    lock restore_transaction marks `restoring` under, so a wake is either
    logged before the restore's cutoff or not at all. Returns "released"
    when no hold names the run any more, "restored" once on_owner_dead()
    reports a complete restore, or None after `iterations` rounds (tests
    only). After GUARD_RETRIES failed restores, GUARD_RETRY_S apart, it gives
    up with CannotRun and leaves the hold file for `capture-meta restore`.
    """
    last, rounds, failed = {}, 0, 0
    while iterations is None or rounds < iterations:
        rounds += 1
        with guarded(lock_file):
            hold = read_hold(hold_file(lock_file))
            if not names_run(hold, run_id, run_dir):
                return "released"
            owner_dead = not host.pid_alive(hold["owner_pid"])
            if not owner_dead and not hold.get("restoring"):
                for connector in hold.get("connectors", []):
                    try:
                        state = dpms(host, connector)
                    except CannotRun as error:
                        state = "unreadable"
                        if last.get(connector) != state:
                            _log_wake(lock_file, hold, {"at_us": host.now_us(), "connector": connector,
                                                        "error": str(error)})
                    else:
                        if state == "On" and last.get(connector) != "On":
                            _log_wake(lock_file, hold, {"at_us": host.now_us(), "connector": connector})
                    last[connector] = state
        if owner_dead:
            if on_owner_dead():
                return "restored"
            failed += 1
            if failed >= GUARD_RETRIES:
                raise CannotRun(f"hold for run {run_id} still not restored after {GUARD_RETRIES} attempts; "
                                "run `capture-meta restore` by hand")
            host.sleep(GUARD_RETRY_S)
            continue
        host.sleep(1)
    return None
