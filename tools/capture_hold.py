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
