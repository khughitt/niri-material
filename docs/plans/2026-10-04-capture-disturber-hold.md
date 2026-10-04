# Capture disturber hold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `capture-meta preflight` holds the host's disturbers (user timers, declared services, desktop idle and monitors) for the length of a capture, a guard restores them if the fixture dies, and `release` restores them, scans the journal over the run, and fails a disturbed run.

**Architecture:** A new module, `tools/capture_hold.py`, owns the host side: discovering what to hold, the write-ahead host hold file, applying and undoing each item, the journal scan and the guard's watch loop. `tools/capture-meta` keeps the lifecycle and the record: it wires the hold into `preflight` and `release`, adds the `guard` and `restore` commands, and writes the `hold` and `hold_end` sections of `capture.json`. Every host command goes through one `Host` object, which tests replace with `tools/fake_capture_host.py`; no test reaches the real machine.

**Tech Stack:** Python 3 standard library (`unittest`), systemd (`systemctl`, `systemd-run`, `journalctl`, systemd 262), niri IPC (`niri msg`), Noctalia (`noctalia msg`), sysfs DRM connectors, bash fixtures.

**Spec:** [docs/specs/2026-10-04-capture-disturber-hold-design.md](../specs/2026-10-04-capture-disturber-hold-design.md) (approved, review round 8).

## Global Constraints

- Hold file: `$XDG_RUNTIME_DIR/capture-meta.hold.json`, beside the lock; items written to it *before* their action (write-ahead).
- A hold file names a run only when both `run_id` and the resolved `run_dir` match.
- Hold-file restore and removal happen under the lock's `guarded()` directory; `guarded()` is taken only after `acquire_lock` has returned, never nested in one process (flock is per open file).
- Declared services file: `${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold`, one unit per line, `#` comments and blank lines allowed; a listed unit that does not exist is CannotRun.
- Desktop socket: `NIRI_SOCKET` when set and nonempty, else the one live `niri.*.sock` directly in `$XDG_RUNTIME_DIR`; more than one live socket is CannotRun.
- Power-off check: every connected and `enabled` connector's `dpms` must read `Off` within 5 s, else CannotRun after restoring.
- Journal message ids: job start `7d4958e842da4a758f6c1cdc7b36dcc5`, unit started/finished `39f53479d3a045ac8e11786248231fbf`, failed start `be02cf6855d2428ba40df7e9d022f03d`, scheduled restart `5eb03494b6584870a536b337290809b3`.
- Invocation id: `USER_INVOCATION_ID` on user-manager entries (unit in `USER_UNIT`), `INVOCATION_ID` on system entries (unit in `UNIT`); an entry with neither is an activation of its own.
- `hold_end.scan` is written once, bounded by `until_us` fixed at the first restore; `hold_end.restore` is `{state: complete|failed, attempts: [{at, by, failures[, notes]}]}`, `by` one of `release`, `preflight`, `guard`, `hand`, `next-preflight`.
- Exit codes: `restore.state` `failed` → 2; else `scan.verdict` `clean` → 0, `disturbed` → 1, `unscanned` → 2, `not-run` → the preflight failure's own code.
- `--hold-settle` defaults to 10 seconds.
- Python tooling gate: `just --set one_cmd 'python3 -m unittest' test-one tools.<module>`; all tooling tests: `just --set fast_cmd 'python3 -m unittest discover -s tools 2>&1' test-fast`. Never call a test runner directly.
- Commits: conventional commits, no AI attribution trailers; `tasks done <id>` for each step in the same commit as its code; `tasks check` before every commit.

## Review Focus

1. **A test that reaches the real host.** Any in-process test that forgets to pass a fake would stop the developer's real timers. Expected: such a test fails loudly instead. Pinned in Task 4 (`cm.Host` replaced by a refusing stand-in for the whole test module) and Task 5 (the end-to-end test runs with fake `systemctl`, `journalctl`, `systemd-run` and `niri` first on `PATH`).
2. **The fixture shell is killed while the `preflight` Python child keeps applying the hold.** The guard restores and removes the hold file; preflight's next write-ahead step must stop rather than recreate it. Pinned in Task 2 (`test_apply_stops_when_the_hold_file_vanishes`).
3. **A sysfs read fails inside the guard** (connector unplugged mid-run). Expected: the guard keeps guarding and the run is flagged, not a dead guard. Pinned in Task 3 (`monitor-unwatched` kind) and Task 5 (`test_guard_survives_an_unreadable_connector`).
4. **The user manager is unreachable** (`systemctl --user` fails in an odd TTY environment). Expected: CannotRun before anything changes. Pinned in Task 1 (`test_unreachable_user_manager_changes_nothing`).
5. **A stale `niri.*.sock` from a dead desktop.** Expected: treated as no desktop, not CannotRun and not a hold on a dead compositor. Pinned in Task 1 (`test_dead_socket_file_is_no_desktop`).

---

## File Structure

- Create `tools/capture_hold.py` — host side of the hold (exceptions and `guarded()` move here so both files share them).
- Create `tools/fake_capture_host.py` — scripted host for tests (not a test module: no `test_` prefix).
- Create `tools/test_capture_hold.py` — unit tests for `capture_hold`.
- Modify `tools/capture-meta` — import from `capture_hold`; lifecycle (`restore_run`, `update_hold_end`, `exit_for`, preflight, release, `guard`, `restore`), record validation, `show`.
- Modify `tools/test_capture_meta.py` — lifecycle tests; existing preflight tests pass a `FakeHost`; end-to-end test gets fake host commands.
- Modify `docs/materials/scripts/glass-optic-smoke-lib.sh`, `ring-motion-clips.sh`, `drag-lag-clips.sh`, `focus-swap-clips.sh`, `optic-settling-smoke.sh` and their tests in `tools/test_glass_optic_smoke.py`, `tools/test_optic_settling.py`.
- Modify `docs/materials/capture-host-setup.md` — the hold, its config file, recovery.

### Task 1: Host adapter and hold discovery

**Files:**
- Create: `tools/capture_hold.py`
- Create: `tools/fake_capture_host.py`
- Create: `tools/test_capture_hold.py`
- Modify: `tools/capture-meta:1-75` (exceptions and `guarded()` now imported)

**Interfaces:**
- Produces: `capture_hold.Refused`, `capture_hold.CannotRun`, `capture_hold.guarded(path)`, `capture_hold.Host(env=None)` with `run(*command, extra_env=None)`, `has(name)`, `pid_alive(pid)`, `now_us()`, `sleep(seconds)`, attributes `env`, `runtime`, `config`, `sysfs`; `checked(host, *command, extra_env=None) -> str`; `list_timers(host, manager, all_) -> dict[str, str]`; `unit_props(host, manager, unit, *props) -> dict[str, str]`; `declared_services(host) -> list[str] | None`; `desktop_socket(host) -> str | None`; `lit_connectors(host) -> list[str]`; `dpms(host, connector) -> str`; `plan_hold(host) -> dict` with keys `items`, `not_held`, `socket`, `config`, `desktop`, `timer_map`, `invocations`, `active_at_hold`, `system_timers`; `rfc3339(us) -> str`.
- Produces (tests): `fake_capture_host.FakeHost(root, *, timers=(), system_timers=(), units=None, journal=None, sockets=(), live=(), noctalia=True, locked=False, connectors=None, env=None, dpms_follows=True)`, `FakeHost.Killed`, `FakeHost.snapshot()`, `FakeHost.wake(connector)`, `FakeHost.add_timer(name, manager="user", service=None, active=True, transient="no")`.

- [ ] **Step 1: Write the fake host**

`tools/fake_capture_host.py`:

```python
"""A scripted host for capture-meta's hold tests: no command reaches the real machine."""
import json
import pathlib


class Killed(BaseException):
    """Stands in for SIGKILL arriving right after the n-th change to the host."""


class Result:
    def __init__(self, returncode=0, stdout="", stderr=""):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


class FakeHost:
    Killed = Killed

    def __init__(self, root, *, timers=(), system_timers=(), units=None, journal=None, sockets=(), live=(),
                 noctalia=True, locked=False, connectors=None, env=None, dpms_follows=True):
        root = pathlib.Path(root)
        self.runtime = root / "rt"; self.runtime.mkdir(parents=True, exist_ok=True)
        self.config = root / "cfg" / "niri-material" / "capture-hold"
        self.sysfs = root / "drm"; self.sysfs.mkdir(parents=True, exist_ok=True)
        self.env = {"XDG_RUNTIME_DIR": str(self.runtime), "PATH": "/usr/bin", **(env or {})}
        self.units = {}
        for timer in timers:
            self.add_timer(timer, "user")
        for timer in system_timers:
            self.add_timer(timer, "system")
        for name, props in (units or {}).items():
            self.units[("user", name)] = {"LoadState": "loaded", "ActiveState": "active", "Transient": "no",
                                          "InvocationID": "", **props}
        self.journal = {"user": [], "system": []}
        for manager, entries in (journal or {}).items():
            self.journal[manager] = list(entries)
        for name in sockets:
            (self.runtime / name).touch()
        self.live = set(live)
        self.noctalia, self.locked, self.caffeine = noctalia, locked, False
        self.dpms_follows = dpms_follows
        self.connectors = {name: dict(c) for name, c in (connectors or {}).items()}
        self.sync_sysfs()
        self.guards, self.calls, self.fail, self.alive = set(), [], {}, {}
        self.clock = 1_791_180_000_000_000
        self.kill_after, self.changes = None, 0

    def add_timer(self, name, manager="user", service=None, active=True, transient="no"):
        service = service or name[: -len(".timer")] + ".service"
        self.units[(manager, name)] = {"LoadState": "loaded", "ActiveState": "active" if active else "inactive",
                                       "Transient": transient, "Activates": service}
        self.units.setdefault((manager, service), {"LoadState": "loaded", "ActiveState": "inactive",
                                                    "Transient": "no", "InvocationID": ""})

    def sync_sysfs(self):
        for name, c in self.connectors.items():
            path = self.sysfs / name; path.mkdir(exist_ok=True)
            for key in ("status", "enabled", "dpms"):
                (path / key).write_text(c[key] + "\n")

    def wake(self, connector):
        self.connectors[connector]["dpms"] = "On"; self.sync_sysfs()

    def change(self):
        self.changes += 1
        if self.kill_after is not None and self.changes == self.kill_after:
            raise Killed()

    def snapshot(self):
        return ({key: p["ActiveState"] for key, p in self.units.items()}, self.caffeine,
                {name: c["dpms"] for name, c in self.connectors.items()})

    # --- the Host interface -------------------------------------------------
    def has(self, name):
        return name != "noctalia" or self.noctalia

    def pid_alive(self, pid):
        return self.alive.get(pid, True)

    def now_us(self):
        return self.clock

    def sleep(self, seconds):
        self.clock += int(seconds * 1_000_000)

    def run(self, *command, extra_env=None):
        self.calls.append(command)
        if command in self.fail:
            return Result(1, "", self.fail[command])
        head = command[0]
        if head == "systemctl":
            return self.systemctl(list(command[1:]))
        if head == "systemd-run":
            self.guards.add(next(a for a in command if a.startswith("--unit="))[len("--unit="):])
            return Result()
        if head == "journalctl":
            manager = "user" if "--user" in command else "system"
            return Result(0, "".join(json.dumps(e) + "\n" for e in self.journal[manager]))
        if head == "niri":
            socket = (extra_env or {}).get("NIRI_SOCKET") or self.env.get("NIRI_SOCKET", "")
            return self.niri(tuple(command[2:]), socket)
        if head == "noctalia":
            return self.noctalia_msg(command[2])
        raise AssertionError(f"unexpected command {command}")

    def systemctl(self, args):
        manager = "system"
        if args[:1] == ["--user"]:
            manager, args = "user", args[1:]
        verb = args[0]
        if verb == "list-timers":
            rows = [{"unit": n, "activates": p["Activates"]} for (m, n), p in sorted(self.units.items())
                    if m == manager and n.endswith(".timer") and ("--all" in args or p["ActiveState"] == "active")]
            return Result(0, json.dumps(rows))
        if verb == "show":
            unit, props = args[1], [a.split("=", 1)[1] for a in args[2:]]
            p = self.units.get((manager, unit), {"LoadState": "not-found", "ActiveState": "inactive"})
            return Result(0, "".join(f"{k}={p.get(k, '')}\n" for k in props))
        if verb == "is-active":
            up = args[1] in self.guards
            return Result(0 if up else 3, "active\n" if up else "inactive\n")
        if verb in ("stop", "start"):
            unit = args[1]
            if verb == "stop" and unit in self.guards:
                self.guards.discard(unit); return Result()
            p = self.units.get((manager, unit))
            if p is None or p["LoadState"] == "not-found":
                return Result(5, "", f"Unit {unit} not found.")
            p["ActiveState"] = "active" if verb == "start" else "inactive"
            self.change()
            return Result()
        raise AssertionError(f"unexpected systemctl {args}")

    def niri(self, args, socket):
        if pathlib.Path(socket).name not in self.live:
            return Result(1, "", "Error connecting to the niri socket")
        if args == ("version",):
            return Result(0, "niri 26.04\n")
        if args in (("action", "power-off-monitors"), ("action", "power-on-monitors")):
            on = args[1] == "power-on-monitors"
            if on or self.dpms_follows:
                for c in self.connectors.values():
                    if c["status"] == "connected" and c["enabled"] == "enabled":
                        c["dpms"] = "On" if on else "Off"
                self.sync_sysfs()
            self.change()
            return Result()
        raise AssertionError(f"unexpected niri {args}")

    def noctalia_msg(self, verb):
        if verb == "status":
            return Result(0, json.dumps({"locked": self.locked}))
        if verb in ("caffeine-enable", "caffeine-disable"):
            self.caffeine = verb == "caffeine-enable"
            self.change()
            return Result()
        raise AssertionError(f"unexpected noctalia {verb}")
```

- [ ] **Step 2: Write the failing discovery tests**

`tools/test_capture_hold.py`:

```python
"""Unit tests for tools/capture_hold.py, against fake_capture_host."""
import json
import pathlib
import tempfile
import unittest

import capture_hold as ch
from fake_capture_host import FakeHost

LIT = {"card1-DP-1": {"status": "connected", "enabled": "enabled", "dpms": "On"},
       "card1-DP-2": {"status": "disconnected", "enabled": "disabled", "dpms": "On"}}


class TempHost(unittest.TestCase):
    def host(self, **kw):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        return FakeHost(temp.name, **kw)


class PlanTests(TempHost):
    def test_holds_active_non_transient_user_timers_and_maps_both_managers(self):
        host = self.host(timers=["wali-rotate.timer", "familiar-reap.timer"], system_timers=["man-db.timer"])
        host.add_timer("run-r1.timer", transient="yes")
        host.add_timer("idle.timer", active=False)
        plan = ch.plan_hold(host)
        self.assertEqual([i["unit"] for i in plan["items"]], ["familiar-reap.timer", "wali-rotate.timer"])
        self.assertEqual(plan["items"][0]["restore"], "systemctl --user start familiar-reap.timer")
        self.assertEqual(plan["not_held"], [{"kind": "timer", "unit": "run-r1.timer", "reason": "transient"}])
        self.assertEqual(plan["timer_map"]["system"], {"man-db.timer": "man-db.service"})
        self.assertIn("idle.timer", plan["timer_map"]["user"])
        self.assertEqual(plan["system_timers"], ["man-db.timer"])
        self.assertEqual(plan["desktop"], "absent")
        self.assertEqual(plan["config"], "absent")

    def test_records_invocations_and_services_running_at_hold(self):
        host = self.host(timers=["familiar-reap.timer"])
        host.units[("user", "familiar-reap.service")].update(ActiveState="active", InvocationID="abc")
        plan = ch.plan_hold(host)
        self.assertEqual(plan["invocations"], {"user": {"familiar-reap.service": "abc"}})
        self.assertEqual(plan["active_at_hold"], [{"manager": "user", "unit": "familiar-reap.service"}])

    def test_declared_services_running_are_held_and_others_recorded(self):
        host = self.host(units={"dropbox.service": {"ActiveState": "activating"},
                                "syncthing.service": {"ActiveState": "inactive"}})
        host.config.parent.mkdir(parents=True)
        host.config.write_text("# crash-loops without a compositor\ndropbox.service\n\nsyncthing.service  # idle\n")
        plan = ch.plan_hold(host)
        self.assertEqual([(i["kind"], i["unit"]) for i in plan["items"]], [("service", "dropbox.service")])
        self.assertEqual(plan["not_held"], [{"kind": "service", "unit": "syncthing.service", "reason": "not_running"}])
        self.assertEqual(plan["config"], str(host.config))

    def test_declared_unit_that_does_not_exist_cannot_run(self):
        host = self.host()
        host.config.parent.mkdir(parents=True); host.config.write_text("dropbx.service\n")
        with self.assertRaisesRegex(ch.CannotRun, "dropbx.service, which does not exist"):
            ch.plan_hold(host)

    def test_desktop_adds_idle_then_monitors_and_records_lock_state(self):
        host = self.host(sockets=["niri.wayland-1.5.sock"], live=["niri.wayland-1.5.sock"], locked=True,
                         connectors=LIT)
        plan = ch.plan_hold(host)
        socket = str(host.runtime / "niri.wayland-1.5.sock")
        self.assertEqual(plan["desktop"], "present")
        self.assertEqual(plan["socket"], socket)
        idle, monitors = plan["items"]
        self.assertEqual(idle, {"kind": "idle", "action": "caffeine-enabled", "prior": "unknown", "locked": True,
                                "socket": socket, "restore": "noctalia msg caffeine-disable"})
        self.assertEqual(monitors["connectors"], ["card1-DP-1"])
        self.assertEqual(monitors["restore"], f"NIRI_SOCKET={socket} niri msg action power-on-monitors")

    def test_empty_niri_socket_falls_back_to_the_runtime_glob(self):
        host = self.host(sockets=["niri.wayland-1.5.sock"], live=["niri.wayland-1.5.sock"], env={"NIRI_SOCKET": ""})
        self.assertEqual(ch.desktop_socket(host), str(host.runtime / "niri.wayland-1.5.sock"))

    def test_two_live_sockets_cannot_run(self):
        host = self.host(sockets=["niri.a.1.sock", "niri.b.2.sock"], live=["niri.a.1.sock", "niri.b.2.sock"])
        with self.assertRaisesRegex(ch.CannotRun, "more than one live niri socket"):
            ch.desktop_socket(host)

    def test_dead_socket_file_is_no_desktop(self):
        host = self.host(sockets=["niri.wayland-1.9.sock"])
        self.assertIsNone(ch.desktop_socket(host))

    def test_desktop_without_noctalia_holds_monitors_and_records_the_gap(self):
        host = self.host(sockets=["niri.w.1.sock"], live=["niri.w.1.sock"], noctalia=False, connectors=LIT)
        plan = ch.plan_hold(host)
        self.assertEqual([i["kind"] for i in plan["items"]], ["monitors"])
        self.assertEqual(plan["not_held"], [{"kind": "idle", "reason": "noctalia not on PATH"}])

    def test_unreachable_user_manager_changes_nothing(self):
        host = self.host(timers=["wali-rotate.timer"])
        host.fail[("systemctl", "--user", "list-timers", "--output=json", "--no-pager")] = "Failed to connect to bus"
        before = host.snapshot()
        with self.assertRaisesRegex(ch.CannotRun, "Failed to connect to bus"):
            ch.plan_hold(host)
        self.assertEqual(host.snapshot(), before)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: FAIL — `ModuleNotFoundError: No module named 'capture_hold'`.

- [ ] **Step 4: Write `capture_hold.py` with the host adapter and discovery**

`tools/capture_hold.py`:

```python
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

    def run(self, *command, extra_env=None):
        try:
            return subprocess.run(command, capture_output=True, text=True, check=False,
                                  env={**self.env, **(extra_env or {})})
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
    names = []
    for path in sorted(host.sysfs.glob("card*-*")):
        try:
            lit = ((path / "status").read_text().strip() == "connected"
                   and (path / "enabled").read_text().strip() == "enabled")
        except OSError:
            continue
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
```

In `tools/capture-meta`, delete the `Refused`, `CannotRun` and `guarded` definitions (lines 28-33 and 62-75) and import them instead, right after the standard-library imports:

```python
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import capture_hold as ch  # noqa: E402
from capture_hold import CannotRun, Refused, guarded  # noqa: E402
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: PASS (10 tests).
Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS (the moved names still resolve as `cm.CannotRun`, `cm.Refused`).

- [ ] **Step 6: Commit**

```bash
tasks done <task-1-id> "capture_hold: Host adapter, fake host, hold discovery"
tasks check
git add tools/capture_hold.py tools/fake_capture_host.py tools/test_capture_hold.py tools/capture-meta tasks/
git commit -m "feat(capture): discover host disturbers to hold (material-188aaa)"
```

### Task 2: Hold file, applying and restoring

**Files:**
- Modify: `tools/capture_hold.py` (append)
- Modify: `tools/test_capture_hold.py` (append)

**Interfaces:**
- Consumes: Task 1's `plan_hold`, `checked`, `unit_props`, `dpms`, `guarded`, `rfc3339`.
- Produces: `hold_file(lock_file) -> Path`; `wake_file(lock_file, hold) -> Path`; `run_key(run_dir) -> str`; `read_hold(path) -> dict | None`; `write_hold(path, hold)`; `names_run(hold, run_id, run_dir) -> bool`; `hold_names_run(lock_file, run_id, run_dir) -> bool`; `create_hold(lock_file, run_id, run_dir, owner_pid, plan, now_us) -> dict`; `apply_hold(host, lock_file, run_id, run_dir, plan) -> dict` (the hold file after); `mark_restoring(lock_file, run_id, run_dir, now_us) -> (hold, wakes) | None`; `restore(host, lock_file, run_id, run_dir, by) -> attempt dict | None`. The hold file has keys `run_id, run_dir, owner_pid, key, guard, started_us, socket, timer_map, invocations, items, connectors` and, once restoring, `restoring, until_us`.

- [ ] **Step 1: Write the failing tests**

Append to `tools/test_capture_hold.py` (before the `__main__` block):

```python
class HoldFileTests(TempHost):
    def setUp(self):
        self.host_ = self.host(timers=["wali-rotate.timer", "familiar-reap.timer"],
                               units={"dropbox.service": {"ActiveState": "active"}},
                               sockets=["niri.w.1.sock"], live=["niri.w.1.sock"], connectors=LIT)
        self.host_.config.parent.mkdir(parents=True); self.host_.config.write_text("dropbox.service\n")
        self.lock = self.host_.runtime / "capture-meta.lock"
        self.run_dir = self.host_.runtime.parent / "runs" / "pilot-1"; self.run_dir.mkdir(parents=True)

    def hold(self):
        plan = ch.plan_hold(self.host_)
        ch.create_hold(self.lock, "pilot-1", self.run_dir, 4242, plan, self.host_.now_us())
        return plan

    def test_create_writes_an_empty_hold_naming_the_run(self):
        self.hold()
        hold = ch.read_hold(ch.hold_file(self.lock))
        self.assertEqual(hold["items"], [])
        self.assertEqual(hold["run_dir"], str(self.run_dir.resolve()))
        self.assertTrue(hold["guard"].startswith("capture-meta-guard-") and hold["guard"].endswith(".service"))
        self.assertTrue(ch.names_run(hold, "pilot-1", self.run_dir))
        self.assertFalse(ch.names_run(hold, "pilot-1", self.run_dir.parent / "other" / "pilot-1"))

    def test_create_refuses_over_an_existing_hold(self):
        self.hold()
        with self.assertRaisesRegex(ch.CannotRun, "already recorded"):
            self.hold()

    def test_apply_holds_everything_and_records_connectors_after_power_off(self):
        plan = self.hold()
        hold = ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        self.assertEqual([i.get("unit", i["kind"]) for i in hold["items"]],
                         ["familiar-reap.timer", "wali-rotate.timer", "dropbox.service", "idle", "monitors"])
        self.assertEqual(hold["connectors"], ["card1-DP-1"])
        self.assertTrue(self.host_.caffeine)
        self.assertEqual(self.host_.connectors["card1-DP-1"]["dpms"], "Off")
        self.assertEqual(self.host_.units[("user", "wali-rotate.timer")]["ActiveState"], "inactive")
        self.assertGreater(hold["started_us"], 0)

    def test_power_off_the_kernel_does_not_report_cannot_run(self):
        self.host_.dpms_follows = False
        plan = self.hold()
        with self.assertRaisesRegex(ch.CannotRun, "card1-DP-1 still reports dpms On 5 s after power-off"):
            ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)

    def test_a_kill_after_any_change_leaves_a_hold_file_that_restores_the_host(self):
        for n in range(1, 6):
            with self.subTest(kill_after=n):
                self.setUp()
                before = self.host_.snapshot()
                plan = self.hold()
                self.host_.kill_after = n
                with self.assertRaises(self.host_.Killed):
                    ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
                self.host_.kill_after = None
                ch.mark_restoring(self.lock, "pilot-1", self.run_dir, self.host_.now_us())
                attempt = ch.restore(self.host_, self.lock, "pilot-1", self.run_dir, "guard")
                self.assertEqual(attempt["failures"], [])
                self.assertEqual(self.host_.snapshot()[:2], before[:2])
                self.assertIsNone(ch.read_hold(ch.hold_file(self.lock)))

    def test_apply_stops_when_the_hold_file_vanishes(self):
        plan = self.hold()
        ch.hold_file(self.lock).unlink()
        with self.assertRaisesRegex(ch.CannotRun, "hold file .* is gone"):
            ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        self.assertFalse(ch.hold_file(self.lock).exists())

    def test_restore_reverses_order_keeps_failures_and_notes_vanished_units(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        del self.host_.units[("user", "familiar-reap.timer")]
        self.host_.fail[("systemctl", "--user", "start", "dropbox.service")] = "Job failed"
        hold, wakes = ch.mark_restoring(self.lock, "pilot-1", self.run_dir, self.host_.now_us())
        attempt = ch.restore(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        starts = [c[3] for c in self.host_.calls if c[:3] == ("systemctl", "--user", "start")]
        self.assertEqual(starts, ["dropbox.service", "wali-rotate.timer", "familiar-reap.timer"])
        self.assertEqual(attempt["by"], "release")
        self.assertEqual(attempt["notes"], ["familiar-reap.timer no longer exists"])
        self.assertEqual([f["unit"] for f in attempt["failures"]], ["dropbox.service"])
        self.assertEqual(attempt["failures"][0]["restore"], "systemctl --user start dropbox.service")
        left = ch.read_hold(ch.hold_file(self.lock))
        self.assertEqual([i.get("unit") for i in left["items"]], ["dropbox.service"])
        self.assertTrue(left["restoring"])

    def test_mark_restoring_fixes_until_once(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        first, _ = ch.mark_restoring(self.lock, "pilot-1", self.run_dir, 100)
        second, _ = ch.mark_restoring(self.lock, "pilot-1", self.run_dir, 999)
        self.assertEqual((first["until_us"], second["until_us"]), (100, 100))

    def test_other_runs_hold_is_left_alone(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        other = self.run_dir.parent / "elsewhere" / "pilot-1"
        self.assertIsNone(ch.mark_restoring(self.lock, "pilot-1", other, 1))
        self.assertIsNone(ch.restore(self.host_, self.lock, "pilot-1", other, "release"))
        self.assertTrue(ch.hold_file(self.lock).exists())

    def test_restore_of_a_gone_desktop_notes_it(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        (self.host_.runtime / "niri.w.1.sock").unlink()
        ch.mark_restoring(self.lock, "pilot-1", self.run_dir, 1)
        attempt = ch.restore(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        self.assertEqual(attempt["failures"], [])
        self.assertIn("desktop gone; monitors left as they are", attempt["notes"])
        self.assertIn("desktop gone; caffeine left as it is", attempt["notes"])

    def test_unreadable_hold_file_cannot_run(self):
        ch.hold_file(self.lock).write_text("{")
        with self.assertRaisesRegex(ch.CannotRun, "restore its items by hand"):
            ch.read_hold(ch.hold_file(self.lock))
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: FAIL — `AttributeError: module 'capture_hold' has no attribute 'create_hold'` (and siblings).

- [ ] **Step 3: Implement the hold file, apply and restore**

Append to `tools/capture_hold.py`:

```python
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


def _held(lock_file, run_id, run_dir):
    path = hold_file(lock_file)
    hold = read_hold(path)
    if not names_run(hold, run_id, run_dir):
        raise CannotRun(f"hold file {path} is gone or names another run: the guard restored the host "
                        "after the owner died; stopping")
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
            path, hold = _held(lock_file, run_id, run_dir)
            hold["items"].append(item)
            write_hold(path, hold)
            act(host, item)
    monitors = next((i for i in plan["items"] if i["kind"] == "monitors"), None)
    if monitors:
        wait_powered_off(host, monitors["connectors"])
    with guarded(lock_file):
        path, hold = _held(lock_file, run_id, run_dir)
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


def mark_restoring(lock_file, run_id, run_dir, now_us):
    """Stop the wake watch and fix the scan's end. Returns (hold, wakes) or None if no hold names the run."""
    path = hold_file(lock_file)
    with guarded(lock_file):
        hold = read_hold(path)
        if not names_run(hold, run_id, run_dir):
            return None
        hold["restoring"] = True
        hold.setdefault("until_us", now_us)
        write_hold(path, hold)
        wakes = []
        try:
            lines = wake_file(lock_file, hold).read_text().splitlines()
        except FileNotFoundError:
            lines = []
        except OSError as error:
            raise CannotRun(f"wake log: {error}") from error
        for line in lines:
            try:
                wakes.append(json.loads(line))
            except json.JSONDecodeError:
                wakes.append({"at_us": hold["until_us"], "connector": "?", "error": f"unreadable wake entry {line!r}"})
        return hold, wakes


def restore(host, lock_file, run_id, run_dir, by):
    """Undo the hold in reverse order, best-effort per item.

    The hold file keeps only the items that failed; with none left, it and the
    wake log are removed. Returns the attempt, or None if no hold names the run.
    """
    path = hold_file(lock_file)
    with guarded(lock_file):
        hold = read_hold(path)
        if not names_run(hold, run_id, run_dir):
            return None
        failures, notes, remaining = [], [], []
        for item in reversed(hold["items"]):
            try:
                note = undo(host, item)
            except CannotRun as error:
                failures.append({**{k: item[k] for k in ("kind", "unit") if k in item},
                                 "error": str(error), "restore": item["restore"]})
                remaining.insert(0, item)
                continue
            if note:
                notes.append(note)
        attempt = {"at": rfc3339(host.now_us()), "by": by, "failures": failures}
        if notes:
            attempt["notes"] = notes
        if remaining:
            hold["items"] = remaining
            hold["restoring"] = True
            write_hold(path, hold)
        else:
            try:
                wake_file(lock_file, hold).unlink(missing_ok=True)
                path.unlink()
            except OSError as error:
                raise CannotRun(f"cannot remove hold file {path}: {error}") from error
        return attempt
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: PASS (21 tests).

- [ ] **Step 5: Commit**

```bash
tasks done <task-2-id> "capture_hold: write-ahead hold file, apply, restore"
tasks check
git add tools/capture_hold.py tools/test_capture_hold.py tasks/
git commit -m "feat(capture): write-ahead hold file, apply and restore (material-188aaa)"
```

### Task 3: Journal scan and the guard's watch loop

**Files:**
- Modify: `tools/capture_hold.py` (append)
- Modify: `tools/test_capture_hold.py` (append)

**Interfaces:**
- Consumes: Task 2's hold file shape (`timer_map`, `invocations`, `items`, `started_us`, `until_us`, `connectors`, `restoring`, `owner_pid`, `key`), `read_hold`, `hold_file`, `wake_file`, `names_run`, `dpms`, `list_timers`.
- Produces: `journal(host, manager, since_us) -> list[dict]`; `scan(host, hold, wakes) -> (verdict, disturbances)` where verdict is `clean` or `disturbed` and each disturbance is `{"at", "at_us", "manager", "unit", "kind"}` with kind in `timer-fired`, `timer-added`, `restart`, `held-started`, `monitor-woke`, `monitor-unwatched` (raises CannotRun when the journal cannot be read); `guard_loop(host, lock_file, run_id, run_dir, on_owner_dead, iterations=None) -> str` returning `"released"` or `"restored"`.

- [ ] **Step 1: Write the failing tests**

Append to `tools/test_capture_hold.py`:

```python
def entry(manager, at, mid, unit, inv=None):
    e = {"__REALTIME_TIMESTAMP": str(at), "MESSAGE_ID": mid, ("USER_UNIT" if manager == "user" else "UNIT"): unit}
    if inv:
        e["USER_INVOCATION_ID" if manager == "user" else "INVOCATION_ID"] = inv
    return e


T0 = 1_791_180_000_000_000


class ScanTests(TempHost):
    def hold(self, **extra):
        hold = {"run_id": "r", "run_dir": "/r", "owner_pid": 1, "key": "k", "guard": "g.service",
                "started_us": T0, "until_us": T0 + 600_000_000, "socket": None,
                "timer_map": {"user": {"wali-rotate.timer": "wali-rotate.service",
                                       "familiar-reap.timer": "familiar-reap.service"},
                              "system": {"plocate-updatedb.timer": "plocate-updatedb.service"}},
                "invocations": {"user": {"familiar-reap.service": "old"}},
                "items": [ch.unit_item("timer", "wali-rotate.timer"), ch.unit_item("service", "dropbox.service")],
                "connectors": []}
        hold.update(extra)
        return hold

    def scan(self, user=(), system=(), wakes=(), **extra):
        host = self.host(journal={"user": list(user), "system": list(system)})
        return ch.scan(host, self.hold(**extra), list(wakes))

    def kinds(self, result):
        return [(d["kind"], d["unit"]) for d in result[1]]

    def test_quiet_window_is_clean(self):
        self.assertEqual(self.scan(), ("clean", []))

    def test_system_timer_service_with_job_start_fires(self):
        result = self.scan(system=[entry("system", T0 + 5, ch.JOB_START, "plocate-updatedb.service", "i1"),
                                   entry("system", T0 + 9, ch.UNIT_STARTED, "plocate-updatedb.service", "i1")])
        self.assertEqual(result[0], "disturbed")
        self.assertEqual(self.kinds(result), [("timer-fired", "plocate-updatedb.service")])

    def test_user_entries_are_keyed_on_user_invocation_id(self):
        result = self.scan(user=[entry("user", T0 + 5, ch.JOB_START, "wali-rotate.service", "u1"),
                                 entry("user", T0 + 7, ch.UNIT_STARTED, "wali-rotate.service", "u1")])
        self.assertEqual(self.kinds(result), [("timer-fired", "wali-rotate.service")])

    def test_started_only_service_counts(self):
        result = self.scan(user=[entry("user", T0 + 5, ch.UNIT_STARTED, "dropbox.service", "d1")])
        self.assertEqual(self.kinds(result), [("held-started", "dropbox.service")])

    def test_failed_start_counts_once(self):
        result = self.scan(user=[entry("user", T0 + 1, ch.JOB_START, "wali-rotate.service", "f1"),
                                 entry("user", T0 + 2, ch.START_FAILED, "wali-rotate.service", "f1")])
        self.assertEqual(self.kinds(result), [("timer-fired", "wali-rotate.service")])

    def test_service_running_at_hold_that_finishes_inside_is_not_flagged(self):
        result = self.scan(user=[entry("user", T0 + 3, ch.UNIT_STARTED, "familiar-reap.service", "old")])
        self.assertEqual(result, ("clean", []))

    def test_entry_without_invocation_id_is_its_own_activation(self):
        result = self.scan(user=[entry("user", T0 + 3, ch.UNIT_STARTED, "wali-rotate.service"),
                                 entry("user", T0 + 4, ch.UNIT_STARTED, "wali-rotate.service")])
        self.assertEqual(len(result[1]), 2)

    def test_added_transient_timer_and_its_same_named_service(self):
        result = self.scan(user=[entry("user", T0 + 1, ch.UNIT_STARTED, "run-r9.timer", "t9"),
                                 entry("user", T0 + 30, ch.UNIT_STARTED, "run-r9.service", "s9")])
        self.assertEqual(self.kinds(result), [("timer-added", "run-r9.timer"), ("timer-fired", "run-r9.service")])

    def test_held_timer_started_again_is_held_started(self):
        result = self.scan(user=[entry("user", T0 + 1, ch.UNIT_STARTED, "wali-rotate.timer", "t1")])
        self.assertEqual(self.kinds(result), [("held-started", "wali-rotate.timer")])

    def test_restart_of_any_unit(self):
        result = self.scan(user=[entry("user", T0 + 1, ch.RESTART, "mystery.service", "m1")])
        self.assertEqual(self.kinds(result), [("restart", "mystery.service")])

    def test_window_bounds_are_in_microseconds(self):
        result = self.scan(system=[entry("system", T0 - 1, ch.JOB_START, "plocate-updatedb.service", "a"),
                                   entry("system", T0 + 600_000_000, ch.JOB_START, "plocate-updatedb.service", "b")])
        self.assertEqual(result, ("clean", []))

    def test_fixture_transient_services_are_not_flagged(self):
        result = self.scan(user=[entry("user", T0 + 1, ch.UNIT_STARTED, "gos-weston-1234.service", "w1")])
        self.assertEqual(result, ("clean", []))

    def test_wakes_and_unwatched_connectors(self):
        result = self.scan(wakes=[{"at_us": T0 + 9, "connector": "card1-DP-1"},
                                  {"at_us": T0 + 10, "connector": "card1-DP-1", "error": "gone"}])
        self.assertEqual(self.kinds(result), [("monitor-woke", "card1-DP-1"), ("monitor-unwatched", "card1-DP-1")])

    def test_unreadable_journal_cannot_run(self):
        host = self.host()
        host.fail[("journalctl", "--user", "-o", "json", "--no-pager", f"--since=@{T0 // 1_000_000 - 1}")] = "denied"
        with self.assertRaisesRegex(ch.CannotRun, "denied"):
            ch.scan(host, self.hold(), [])


class GuardLoopTests(TempHost):
    def setUp(self):
        self.h = self.host(connectors=LIT)
        self.lock = self.h.runtime / "capture-meta.lock"
        self.run_dir = self.h.runtime.parent / "pilot-1"; self.run_dir.mkdir()
        plan = {"socket": None, "timer_map": {"user": {}, "system": {}}, "invocations": {}}
        ch.create_hold(self.lock, "pilot-1", self.run_dir, 77, plan, 0)
        path = ch.hold_file(self.lock); hold = ch.read_hold(path)
        hold["connectors"] = ["card1-DP-1"]; ch.write_hold(path, hold)
        self.h.connectors["card1-DP-1"]["dpms"] = "Off"; self.h.sync_sysfs()

    def wakes(self):
        hold = ch.read_hold(ch.hold_file(self.lock))
        path = ch.wake_file(self.lock, hold)
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_logs_each_wake_transition_once(self):
        loop = lambda n: ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: True, iterations=n)
        loop(2)
        self.h.wake("card1-DP-1"); loop(3)
        self.assertEqual([w["connector"] for w in self.wakes()], ["card1-DP-1"])

    def test_no_wakes_logged_once_restoring(self):
        ch.mark_restoring(self.lock, "pilot-1", self.run_dir, 5)
        self.h.wake("card1-DP-1")
        ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: True, iterations=3)
        self.assertEqual(self.wakes(), [])

    def test_unreadable_connector_is_logged_not_fatal(self):
        (self.h.sysfs / "card1-DP-1" / "dpms").unlink()
        self.assertIsNone(ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: True, iterations=2))
        self.assertEqual([("error" in w) for w in self.wakes()], [True])

    def test_owner_death_restores_and_retries_until_complete(self):
        outcomes = iter([False, False, True])
        self.h.alive[77] = False
        self.assertEqual(ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: next(outcomes)), "restored")

    def test_gives_up_after_bounded_retries(self):
        self.h.alive[77] = False
        with self.assertRaisesRegex(ch.CannotRun, "still not restored after 20 attempts"):
            ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: False)

    def test_released_hold_ends_the_guard(self):
        ch.hold_file(self.lock).unlink()
        self.assertEqual(ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: True), "released")
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: FAIL — `AttributeError: module 'capture_hold' has no attribute 'scan'` / `'guard_loop'`.

- [ ] **Step 3: Implement the scan and the guard loop**

Append to `tools/capture_hold.py`:

```python
def journal(host, manager, since_us):
    flag = "--user" if manager == "user" else "--system"
    out = checked(host, "journalctl", flag, "-o", "json", "--no-pager", f"--since=@{since_us // 1_000_000 - 1}")
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
        kind = "monitor-unwatched" if "error" in wake else "monitor-woke"
        found.append(_disturbance(wake["at_us"], "sysfs", wake["connector"], kind))
    found.sort(key=lambda d: d["at_us"])
    return ("disturbed" if found else "clean"), found


def _log_wake(lock_file, hold, record):
    try:
        with open(wake_file(lock_file, hold), "a") as stream:
            stream.write(json.dumps(record) + "\n")
    except OSError as error:
        raise CannotRun(f"wake log: {error}") from error


def guard_loop(host, lock_file, run_id, run_dir, on_owner_dead, iterations=None):
    """Watch the owner and the held monitors once a second (spec §4.4).

    Returns "released" when no hold names the run any more, "restored" once
    on_owner_dead() reports a complete restore, or None after `iterations`
    rounds (tests only). Gives up with CannotRun after GUARD_RETRIES failed
    restores, leaving the hold file for `capture-meta restore`.
    """
    last, rounds, failed = {}, 0, 0
    while iterations is None or rounds < iterations:
        rounds += 1
        hold = read_hold(hold_file(lock_file))
        if not names_run(hold, run_id, run_dir):
            return "released"
        if not host.pid_alive(hold["owner_pid"]):
            if on_owner_dead():
                return "restored"
            failed += 1
            if failed >= GUARD_RETRIES:
                raise CannotRun(f"hold for run {run_id} still not restored after {GUARD_RETRIES} attempts; "
                                "run `capture-meta restore` by hand")
            host.sleep(GUARD_RETRY_S)
            continue
        if not hold.get("restoring"):
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
        host.sleep(1)
    return None
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: PASS (41 tests).

- [ ] **Step 5: Commit**

```bash
tasks done <task-3-id> "capture_hold: journal scan and guard watch loop"
tasks check
git add tools/capture_hold.py tools/test_capture_hold.py tasks/
git commit -m "feat(capture): journal disturbance scan and guard watch loop (material-188aaa)"
```

### Task 4: The hold record in capture.json

**Files:**
- Modify: `tools/capture-meta` (`validate_record`, new `update_hold_end`, `exit_for`, `describe_disturbance`, `render`)
- Modify: `tools/test_capture_meta.py`

**Interfaces:**
- Consumes: Task 3's disturbance shape.
- Produces: `update_hold_end(run_dir, scan=None, attempt=None) -> dict` (writes `scan` once; appends `attempt` and sets `restore.state` from its failures); `exit_for(hold_end)` (returns for exit 0, raises Refused for 1, CannotRun for 2); `SCAN_VERDICTS = ("clean", "disturbed", "unscanned", "not-run")`.

- [ ] **Step 1: Write the failing tests**

At the top of `tools/test_capture_meta.py`, right after `spec.loader.exec_module(cm)`, add the real-host fence (Review Focus 1):

```python
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fake_capture_host import FakeHost  # noqa: E402


class _NoRealHost:
    def __init__(self, *args, **kwargs):
        raise AssertionError("a test reached the real host: pass a FakeHost")


cm.Host = _NoRealHost
```

Append a test class:

```python
class HoldRecordTests(unittest.TestCase):
    def run_dir(self):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        run = pathlib.Path(temp.name) / "r"; run.mkdir()
        cm.write_section(run, "run", {"id": "r"})
        return run

    def test_scan_is_written_once_and_attempts_accumulate(self):
        run = self.run_dir()
        failed = {"at": "t1", "by": "release", "failures": [{"unit": "a.timer", "error": "x", "restore": "systemctl --user start a.timer"}]}
        end = cm.update_hold_end(run, {"until_us": 5, "verdict": "clean", "disturbances": []}, failed)
        self.assertEqual(end["restore"]["state"], "failed")
        with self.assertRaisesRegex(cm.CannotRun, "written once"):
            cm.update_hold_end(run, {"until_us": 9, "verdict": "clean", "disturbances": []})
        end = cm.update_hold_end(run, attempt={"at": "t2", "by": "guard", "failures": []})
        self.assertEqual(end["restore"]["state"], "complete")
        self.assertEqual([a["by"] for a in end["restore"]["attempts"]], ["release", "guard"])
        self.assertEqual(cm.load_record(run)["hold_end"]["scan"]["until_us"], 5)

    def test_exit_codes_follow_the_record(self):
        complete = {"state": "complete", "attempts": []}
        failed = {"state": "failed", "attempts": [{"by": "release", "failures": [
            {"unit": "a.timer", "error": "x", "restore": "systemctl --user start a.timer"}]}]}
        cm.exit_for({"restore": complete, "scan": {"verdict": "clean", "disturbances": []}})
        with self.assertRaisesRegex(cm.Refused, "disturbed: timer-fired man-db.service"):
            cm.exit_for({"restore": complete, "scan": {"verdict": "disturbed", "disturbances": [
                {"kind": "timer-fired", "unit": "man-db.service", "manager": "system", "at": "t"}]}})
        with self.assertRaisesRegex(cm.CannotRun, "systemctl --user start a.timer"):
            cm.exit_for({"restore": failed, "scan": {"verdict": "clean", "disturbances": []}})
        with self.assertRaisesRegex(cm.CannotRun, "not evidence until rescanned"):
            cm.exit_for({"restore": complete, "scan": {"verdict": "unscanned", "error": "e", "disturbances": []}})
        with self.assertRaises(cm.Refused):
            cm.exit_for({"restore": complete, "scan": {"verdict": "not-run", "preflight_exit": 1, "disturbances": []}})
        with self.assertRaises(cm.CannotRun):
            cm.exit_for({"restore": complete, "scan": {"verdict": "not-run", "preflight_exit": 2, "disturbances": []}})

    def test_validation_rejects_malformed_hold_sections(self):
        for bad in ({"hold": {"items": {}}},
                    {"hold_end": {"restore": {"state": "maybe", "attempts": []}}},
                    {"hold_end": {"scan": {"verdict": "fine", "disturbances": []}}}):
            with self.subTest(bad=bad), self.assertRaises(cm.CannotRun):
                cm.validate_record({"schema": cm.SCHEMA, **bad})

    def test_show_prints_hold_and_verdict(self):
        record = {"run": {"id": "r"}, "hold": {"desktop": "absent", "items": [
                      {"kind": "timer", "unit": "a.timer"}, {"kind": "service", "unit": "dropbox.service"}],
                      "not_held": [{"kind": "idle", "reason": "noctalia not on PATH"}]},
                  "hold_end": {"restore": {"state": "complete", "attempts": [{"by": "release", "failures": []}]},
                               "scan": {"verdict": "disturbed", "disturbances": [
                                   {"kind": "restart", "unit": "x.service", "manager": "user", "at": "t"}]}}}
        text = cm.render(record)
        self.assertIn("hold  desktop absent  2 items: a.timer, dropbox.service", text)
        self.assertIn("  not held: idle (noctalia not on PATH)", text)
        self.assertIn("hold end  restore complete (release)  scan disturbed", text)
        self.assertIn("  restart x.service (user) at t", text)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_meta`
Expected: FAIL — `AttributeError: module 'capture_meta' has no attribute 'update_hold_end'`.

- [ ] **Step 3: Implement the record functions**

In `tools/capture-meta`, add to `validate_record` before the `sub_runs` check:

```python
    hold = section("hold")
    if hold is not None and not isinstance(hold.get("items"), list):
        raise CannotRun(f"{path}: hold.items must be a list")
    end = section("hold_end")
    if end is not None:
        restore = end.get("restore")
        if restore is not None and (not isinstance(restore, dict) or restore.get("state") not in ("complete", "failed")
                                    or not isinstance(restore.get("attempts"), list)):
            raise CannotRun(f"{path}: hold_end.restore needs state complete|failed and an attempts list")
        scan = end.get("scan")
        if scan is not None and (not isinstance(scan, dict) or scan.get("verdict") not in SCAN_VERDICTS
                                 or not isinstance(scan.get("disturbances"), list)):
            raise CannotRun(f"{path}: hold_end.scan needs a verdict and a disturbances list")
```

Add after `append_sub_run`:

```python
SCAN_VERDICTS = ("clean", "disturbed", "unscanned", "not-run")


def update_hold_end(run_dir, scan=None, attempt=None):
    """hold_end.scan is written once; hold_end.restore takes every attempt (spec §4.3)."""
    record = load_record(run_dir)
    end = record.get("hold_end", {})
    if scan is not None:
        if "scan" in end:
            raise CannotRun(f"{RECORD}: hold_end.scan is written once")
        end["scan"] = scan
    if attempt is not None:
        restore = end.setdefault("restore", {"state": "failed", "attempts": []})
        restore["attempts"].append(attempt)
        restore["state"] = "failed" if attempt["failures"] else "complete"
    record["hold_end"] = end
    save_record(run_dir, record)
    return end


def describe_disturbance(d):
    return f"{d['kind']} {d['unit']} ({d['manager']}) at {d['at']}"


def failure_list(attempt):
    return "; ".join(f"{f.get('unit') or f['kind']}: {f['restore']}" for f in attempt["failures"])


def exit_for(end):
    restore, scan = end.get("restore", {}), end.get("scan", {})
    if restore.get("state") == "failed":
        raise CannotRun(f"hold not fully restored; restore by hand: {failure_list(restore['attempts'][-1])}")
    verdict = scan.get("verdict")
    if verdict == "clean":
        return
    if verdict == "disturbed":
        raise Refused("disturbed: " + "; ".join(describe_disturbance(d) for d in scan["disturbances"]))
    if verdict == "unscanned":
        raise CannotRun(f"journal scan failed ({scan.get('error')}); the run is not evidence until rescanned")
    if verdict == "not-run":
        message = "preflight failed; its hold was rolled back"
        raise Refused(message) if scan.get("preflight_exit") == 1 else CannotRun(message)
    raise CannotRun(f"{RECORD}: hold_end has no scan verdict")
```

In `render`, before the `sub_runs` block:

```python
    hold = record.get("hold")
    if hold:
        names = [item.get("unit") or item["kind"] for item in hold.get("items", [])]
        lines += ["", f"hold  desktop {hold.get('desktop')}  {len(names)} items: {', '.join(names)}"]
        lines += [f"  not held: {n.get('unit') or n['kind']} ({n['reason']})" for n in hold.get("not_held", [])]
    end = record.get("hold_end")
    if end:
        restore, scan = end.get("restore", {}), end.get("scan", {})
        last = (restore.get("attempts") or [{}])[-1].get("by")
        lines.append(f"hold end  restore {restore.get('state')} ({last})  scan {scan.get('verdict')}")
        lines += [f"  {describe_disturbance(d)}" for d in scan.get("disturbances", [])]
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS, except existing `PreflightTests` and `EndToEndTest`, which still pass because preflight does not yet construct a Host. If any test fails with "a test reached the real host", that test is fixed in Task 5, not here; nothing in Task 4 constructs `Host`.

- [ ] **Step 5: Commit**

```bash
tasks done <task-4-id> "capture-meta: hold and hold_end record, exit codes, show"
tasks check
git add tools/capture-meta tools/test_capture_meta.py tasks/
git commit -m "feat(capture): record the hold and its end in capture.json (material-188aaa)"
```

### Task 5: Lifecycle — preflight, release, guard and restore

**Files:**
- Modify: `tools/capture-meta` (`preflight`, `cmd_preflight`, `cmd_release`, new `restore_run`, `recover_stale`, `hold_section`, `start_guard`, `guard`, `cmd_guard`, `cmd_restore`, parser)
- Modify: `tools/test_capture_meta.py` (`PreflightTests.run_preflight` passes a host; new `LifecycleTests`; `EndToEndTest` fakes)

**Interfaces:**
- Consumes: Tasks 1–4.
- Produces: `preflight(run_dir, lane, task, fixture, seconds, owner_pid, thresholds, tools, proc, gpu, host_load, host, sleep=time.sleep, lock=lock_path, hold_settle=10, launch_guard=None)`; `restore_run(host, lock_file, run_dir, run_id, by, preflight_exit=None) -> {"attempt", "hold_end"} | None`; `guard(host, lock_file, run_dir, iterations=None)`; CLI `capture-meta guard RUN_DIR`, `capture-meta restore`, `preflight --hold-settle N`.

- [ ] **Step 1: Write the failing tests**

Change `PreflightTests.run_preflight` so every existing preflight test runs against a fake host with nothing to hold:

```python
    def run_preflight(self, run, lane="headless", gpu=None, proc=None, env=None, lock=None, host_load=None, host=None, **kw):
        proc = proc or FakeProc([(i * 2, i * 100) for i in range(30)])
        gpu = gpu or FakeGpu([QUIET] * 30)
        host_load = host_load or FakeHostLoad()
        host = host or FakeHost(run.parent / "host")
        saved = dict(os.environ)
        os.environ.update(env or {})
        try:
            return cm.preflight(run, lane=lane, task="material-x", fixture="f.sh", seconds=3, owner_pid=os.getpid(),
                                thresholds=cm.DEFAULT_THRESHOLDS, tools=kw.get("tools", ["tracy=0.13.1"]),
                                proc=proc, gpu=gpu, host_load=host_load, host=host, sleep=lambda s: None,
                                lock=lock or (lambda: run / "lock"), hold_settle=0)
        finally:
            os.environ.clear(); os.environ.update(saved)
```

Append `LifecycleTests`:

```python
class LifecycleTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.root = pathlib.Path(temp.name)
        self.host = FakeHost(self.root / "host", timers=["wali-rotate.timer"],
                             units={"dropbox.service": {"ActiveState": "active"}},
                             sockets=["niri.w.1.sock"], live=["niri.w.1.sock"],
                             connectors={"card1-DP-1": {"status": "connected", "enabled": "enabled", "dpms": "On"}})
        self.host.config.parent.mkdir(parents=True); self.host.config.write_text("dropbox.service\n")
        self.lock = self.host.runtime / cm.LOCK_NAME
        self.run = self.root / "runs" / "pilot-1"; self.run.mkdir(parents=True)
        self.before = self.host.snapshot()
        saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = str(self.host.runtime)
        self.addCleanup(lambda: os.environ.__setitem__("XDG_RUNTIME_DIR", saved) if saved else os.environ.pop("XDG_RUNTIME_DIR", None))
        cm.Host = lambda *a, **k: self.host
        self.addCleanup(setattr, cm, "Host", _NoRealHost)

    def preflight(self, gpu=None, owner=4242, launch_guard=None):
        return cm.preflight(self.run, lane="headless", task="material-188aaa", fixture="t.sh", seconds=3,
                            owner_pid=owner, thresholds=cm.DEFAULT_THRESHOLDS, tools=["tracy=0.13.1"],
                            proc=FakeProc([(i * 2, i * 100) for i in range(30)]), gpu=gpu or FakeGpu([QUIET] * 30),
                            host_load=FakeHostLoad(), host=self.host, sleep=lambda s: None,
                            lock=lambda: self.lock, hold_settle=10, launch_guard=launch_guard)

    def test_quiet_preflight_holds_then_release_restores_and_is_clean(self):
        self.preflight()
        record = cm.load_record(self.run)
        self.assertEqual([i.get("unit", i["kind"]) for i in record["hold"]["items"]],
                         ["wali-rotate.timer", "dropbox.service", "idle", "monitors"])
        self.assertTrue(self.host.guards)
        guard_call = next(c for c in self.host.calls if c[0] == "systemd-run")
        self.assertLess(self.host.calls.index(guard_call),
                        self.host.calls.index(("systemctl", "--user", "stop", "wali-rotate.timer")))
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(self.host.snapshot(), self.before)
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["restore"]["state"], end["scan"]["verdict"]), ("complete", "clean"))
        self.assertFalse(self.host.guards)
        self.assertFalse((self.host.runtime / ch_hold_name()).exists())
        self.assertIsNone(cm.read_lock(self.lock))

    def test_second_release_changes_nothing_and_repeats_the_code(self):
        self.preflight()
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        text = (self.run / cm.RECORD).read_text()
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual((self.run / cm.RECORD).read_text(), text)

    def test_disturbed_run_exits_one_after_restoring(self):
        self.preflight()
        self.host.journal["system"].append({"__REALTIME_TIMESTAMP": str(self.host.now_us() + 5), "UNIT": "man-db.service",
                                            "MESSAGE_ID": "7d4958e842da4a758f6c1cdc7b36dcc5", "INVOCATION_ID": "m"})
        self.host.add_timer("man-db.timer", "system")
        self.host.clock += 60_000_000
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        self.assertEqual(self.host.snapshot()[:2], self.before[:2])
        self.assertEqual(cm.load_record(self.run)["hold_end"]["scan"]["disturbances"][0]["kind"], "timer-fired")

    def test_refused_preflight_rolls_back_and_release_repeats_its_code(self):
        busy = dict(QUIET, util_pct=90.0)
        with self.assertRaises(cm.Refused):
            self.preflight(gpu=FakeGpu([busy] * 30))
        self.assertEqual(self.host.snapshot(), self.before)
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["scan"]["verdict"], end["restore"]["attempts"][-1]["by"]), ("not-run", "preflight"))
        self.assertIsNone(cm.read_lock(self.lock))
        self.assertEqual(cm.main(["release", str(self.run)]), 1)

    def test_guard_that_does_not_start_holds_nothing(self):
        def broken(host, hold):
            raise cm.CannotRun("guard is failed, not active")
        with self.assertRaisesRegex(cm.CannotRun, "not active"):
            self.preflight(launch_guard=broken)
        self.assertEqual(self.host.snapshot(), self.before)
        self.assertIsNone(cm.read_lock(self.lock))

    def test_failed_rollback_keeps_hold_guard_and_lock_and_release_retries(self):
        self.host.fail[("systemctl", "--user", "start", "dropbox.service")] = "Job failed"
        busy = dict(QUIET, util_pct=90.0)
        with self.assertRaisesRegex(cm.CannotRun, "could not be rolled back: dropbox.service"):
            self.preflight(gpu=FakeGpu([busy] * 30))
        self.assertTrue((self.host.runtime / ch_hold_name()).exists())
        self.assertTrue(self.host.guards)
        self.assertIsNotNone(cm.read_lock(self.lock))
        self.assertEqual(cm.main(["release", str(self.run)]), 2)
        del self.host.fail[("systemctl", "--user", "start", "dropbox.service")]
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual(end["restore"]["state"], "complete")
        self.assertEqual(end["scan"]["verdict"], "not-run")
        self.assertIsNone(cm.read_lock(self.lock))

    def test_retry_never_rescans(self):
        self.preflight()
        self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")] = "busy"
        self.assertEqual(cm.main(["release", str(self.run)]), 2)
        until = cm.load_record(self.run)["hold_end"]["scan"]["until_us"]
        self.host.journal["user"].append({"__REALTIME_TIMESTAMP": str(self.host.now_us() + 10), "USER_UNIT": "wali-rotate.timer",
                                          "MESSAGE_ID": "39f53479d3a045ac8e11786248231fbf", "USER_INVOCATION_ID": "late"})
        del self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")]
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["scan"]["until_us"], end["scan"]["verdict"]), (until, "clean"))

    def test_guard_restores_when_the_owner_dies_and_completes_a_failed_record(self):
        self.preflight()
        self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")] = "busy"
        self.assertEqual(cm.main(["release", str(self.run)]), 2)
        del self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")]
        self.host.alive[4242] = False
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["restore"]["state"], end["restore"]["attempts"][-1]["by"]), ("complete", "guard"))
        self.assertEqual(self.host.snapshot()[:2], self.before[:2])

    def test_guard_after_a_kill_mid_run_writes_scan_and_restore(self):
        self.preflight()
        self.host.alive[4242] = False
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["scan"]["verdict"], end["restore"]["attempts"][-1]["by"]), ("clean", "guard"))
        self.assertEqual(self.host.snapshot(), self.before)

    def test_guard_survives_an_unreadable_connector(self):
        self.preflight()
        (self.host.sysfs / "card1-DP-1" / "dpms").unlink()
        self.assertIsNone(cm.guard(self.host, self.lock, self.run, iterations=2))
        self.host.sync_sysfs()
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        kinds = [d["kind"] for d in cm.load_record(self.run)["hold_end"]["scan"]["disturbances"]]
        self.assertEqual(kinds, ["monitor-unwatched"])

    def test_restore_by_hand_refuses_a_live_owner_and_records_hand(self):
        self.preflight()
        self.assertEqual(cm.main(["restore"]), 1)
        self.host.alive[4242] = False
        self.assertEqual(cm.main(["restore"]), 0)
        self.assertEqual(cm.load_record(self.run)["hold_end"]["restore"]["attempts"][-1]["by"], "hand")
        self.assertEqual(cm.main(["restore"]), 0)

    def test_next_preflight_recovers_a_stale_hold_and_updates_its_run(self):
        # acquire_lock checks the old holder with the real pid_alive: use a pid known to be dead.
        dead = subprocess.Popen(["true"]); dead.wait()
        self.preflight(owner=dead.pid)
        self.host.alive[dead.pid] = False
        first = self.run
        self.run = self.root / "runs" / "pilot-2"; self.run.mkdir()
        self.preflight(owner=4343)
        recovered = cm.load_record(self.run)["hold"]["recovered"]
        self.assertEqual(recovered["run_id"], "pilot-1")
        self.assertEqual(cm.load_record(first)["hold_end"]["restore"]["attempts"][-1]["by"], "next-preflight")

    def test_hold_file_naming_another_run_dir_is_not_released(self):
        self.preflight()
        twin = self.root / "elsewhere" / "pilot-1"; twin.mkdir(parents=True)
        cm.write_section(twin, "run", {"id": "pilot-1"})
        cm.main(["release", str(twin)])
        self.assertTrue((self.host.runtime / ch_hold_name()).exists())


def ch_hold_name():
    return cm.ch.HOLD_NAME
```

In `EndToEndTest.test_real_binary_against_fake_nvidia_smi`, after the `host_load` fake is written, add fakes so the subprocess never reaches the real host, and isolate config and sysfs:

```python
            for name, body in {
                "systemctl": textwrap.dedent("""\
                    #!/bin/sh
                    case "$*" in
                      *list-timers*) echo '[]' ;;
                      *is-active*) echo active ;;
                      *show*) for p in "$@"; do case "$p" in --property=*) echo "${p#--property=}=" ;; esac; done ;;
                      *) : ;;
                    esac
                    """),
                "journalctl": "#!/bin/sh\n:\n",
                "systemd-run": "#!/bin/sh\n:\n",
                "niri": "#!/bin/sh\nexit 1\n",
            }.items():
                (bins / name).write_text(body); (bins / name).chmod(0o755)
```

and extend `env` with `"XDG_CONFIG_HOME": str(root / "cfg")`, `"CAPTURE_META_SYSFS": str(root / "drm")`, `"NIRI_SOCKET": ""`, and pass `"--hold-settle", "0"` to the preflight call. After the release assertions add:

```python
            record = json.loads((run / "capture.json").read_text())
            self.assertEqual(record["hold"]["desktop"], "absent")
            self.assertEqual(record["hold_end"]["scan"]["verdict"], "clean")
            self.assertFalse((runtime / "capture-meta.hold.json").exists())
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_meta`
Expected: FAIL — `TypeError: preflight() got an unexpected keyword argument 'host'`.

- [ ] **Step 3: Implement the lifecycle**

In `tools/capture-meta`, add `TOOL = str(pathlib.Path(__file__).resolve())` and `Host = ch.Host` after the imports (tests replace `cm.Host`; code must call `Host()` through the module global, never `ch.Host()` directly). Add before `preflight`:

```python
def hold_section(hold, plan, recovered):
    return {"started": ch.rfc3339(hold["started_us"]), "started_us": hold["started_us"], "items": hold["items"],
            "not_held": plan["not_held"], "socket": plan["socket"], "config": plan["config"],
            "desktop": plan["desktop"], "timer_map": plan["timer_map"], "active_at_hold": plan["active_at_hold"],
            "system_timers": plan["system_timers"], "connectors": hold["connectors"], "guard": hold["guard"],
            "recovered": recovered}


def start_guard(host, hold):
    env = {key: host.env[key] for key in ("XDG_RUNTIME_DIR", "PATH", "WAYLAND_DISPLAY", "XDG_CONFIG_HOME",
                                          "CAPTURE_META_SYSFS") if host.env.get(key)}
    if hold["socket"]:
        env["NIRI_SOCKET"] = hold["socket"]
    ch.checked(host, "systemd-run", "--user", f"--unit={hold['guard']}", "--collect", "--quiet",
               "--property=Type=exec", *(f"--setenv={k}={v}" for k, v in env.items()),
               sys.executable, TOOL, "guard", hold["run_dir"])
    state = host.run("systemctl", "--user", "is-active", hold["guard"]).stdout.strip()
    if state != "active":
        raise CannotRun(f"guard {hold['guard']} is {state or 'unknown'}, not active; nothing was held")


def restore_run(host, lock_file, run_dir, run_id, by, preflight_exit=None):
    """Restore this run's hold, scan it once, record both (spec §4.3).

    Returns {"attempt", "hold_end"}, or None when no hold file names the run.
    The guard and the lock go only once every item is back.
    """
    marked = ch.mark_restoring(lock_file, run_id, run_dir, host.now_us())
    if marked is None:
        return None
    hold, wakes = marked
    attempt = ch.restore(host, lock_file, run_id, run_dir, by)
    if attempt is None:
        return None
    end = None
    run_dir = pathlib.Path(run_dir)
    if (run_dir / RECORD).exists():
        scan = None
        if "scan" not in load_record(run_dir).get("hold_end", {}):
            base = {"until_us": hold["until_us"], "disturbances": []}
            if preflight_exit is not None:
                scan = {**base, "verdict": "not-run", "preflight_exit": preflight_exit}
            else:
                try:
                    verdict, disturbances = ch.scan(host, hold, wakes)
                    scan = {**base, "verdict": verdict, "disturbances": disturbances}
                except CannotRun as error:
                    scan = {**base, "verdict": "unscanned", "error": str(error)}
        end = update_hold_end(run_dir, scan, attempt)
    if not attempt["failures"]:
        if by != "guard":
            host.run("systemctl", "--user", "stop", hold["guard"])
        release_lock(lock_file, hold["owner_pid"], run_id)
    return {"attempt": attempt, "hold_end": end}


def recover_stale(host, lock_file):
    with guarded(lock_file):
        stale = ch.read_hold(ch.hold_file(lock_file))
    if stale is None:
        return None
    result = restore_run(host, lock_file, stale["run_dir"], stale["run_id"], "next-preflight")
    if result is None:
        return None
    if result["attempt"]["failures"]:
        raise Refused(f"a stale hold from run {stale['run_id']} could not be restored: "
                      f"{failure_list(result['attempt'])}")
    return {"run_id": stale["run_id"], "items": [i.get("unit") or i["kind"] for i in stale["items"]], "failures": []}
```

Replace `preflight` with:

```python
def preflight(run_dir, lane, task, fixture, seconds, owner_pid, thresholds, tools, proc, gpu, host_load,
              host, sleep=time.sleep, lock=lock_path, hold_settle=10, launch_guard=None):
    if isinstance(seconds, bool) or not isinstance(seconds, int) or seconds <= 0:
        raise CannotRun("seconds must be a positive integer")
    if isinstance(hold_settle, bool) or not isinstance(hold_settle, int) or hold_settle < 0:
        raise CannotRun("hold_settle must be a nonnegative integer")
    _validate_owner(owner_pid, "preflight")
    run_dir = pathlib.Path(run_dir)
    run = run_section(run_dir, lane, task, fixture)
    write_section(run_dir, "run", run)
    lock_file = lock()
    try:
        lock_info = acquire_lock(lock_file, owner_pid, run["id"])
    except Refused as error:
        write_section(run_dir, "preflight", {"verdict": "refused", "reasons": [str(error)], "thresholds": thresholds})
        raise
    held = False
    try:
        # guarded() is per open file: every hold step takes it after acquire_lock has returned, never inside it.
        recovered = recover_stale(host, lock_file)
        plan = ch.plan_hold(host)
        hold = ch.create_hold(lock_file, run["id"], run_dir, owner_pid, plan, host.now_us())
        held = True
        (launch_guard or start_guard)(host, hold)
        hold = ch.apply_hold(host, lock_file, run["id"], run_dir, plan)
        write_section(run_dir, "hold", hold_section(hold, plan, recovered))
        sleep(hold_settle)
        write_section(run_dir, "environment", environment(gpu, tools, proc))
        reasons = session_reasons(lane)
        samples = sample_stream(proc, gpu, seconds, sleep)
        summary = summarize(samples)
        write_section(run_dir, "baseline", {"seconds": seconds, **summary})
        reasons += judge_quiet(summary, samples, thresholds, lane)
        section = {"verdict": "quiet" if not reasons else "refused",
                   "reasons": reasons, "thresholds": thresholds, "lock": lock_info}
        message = "; ".join(reasons)
        if any(summary[key] > thresholds[key] for key in HOST_LOAD_KEYS):
            try:
                section["host_load"] = host_load.report()
            except CannotRun as error:
                raise CannotRun(f"preflight refused ({message}) and could not name the load: {error}") from error
            message += f"; load carried by: {describe_load(section['host_load']['load']['top'])}"
        write_section(run_dir, "preflight", section)
        if reasons:
            raise Refused(message)
    except BaseException as error:
        if not held:
            release_lock(lock_file, owner_pid, run["id"])
            raise
        code = 1 if isinstance(error, Refused) else 2
        result = restore_run(host, lock_file, run_dir, run["id"], "preflight", preflight_exit=code)
        if result is None:
            release_lock(lock_file, owner_pid, run["id"])
        elif result["attempt"]["failures"]:
            raise CannotRun(f"{error}; the hold could not be rolled back: {failure_list(result['attempt'])}") from error
        raise
```

Update `cmd_preflight`:

```python
def cmd_preflight(args):
    preflight(args.run_dir, args.lane, args.task, args.fixture, args.seconds,
              args.owner_pid if args.owner_pid is not None else os.getppid(),
              parse_thresholds(args.threshold), args.tool, ProcReader(), GpuReader(), HostLoadReader(),
              Host(), hold_settle=args.hold_settle)
```

Replace `cmd_release` and add the guard and restore commands:

```python
def cmd_release(args):
    run_dir = pathlib.Path(args.run_dir)
    record = load_record(run_dir)
    run = record.get("run")
    if not run:
        return
    lock_file = lock_path()
    end = record.get("hold_end")
    if (end is None or end.get("restore", {}).get("state") == "failed") and ch.hold_names_run(lock_file, run["id"], run_dir):
        result = restore_run(Host(), lock_file, run_dir, run["id"], "release")
        if result is not None:
            end = result["hold_end"]
    if end is not None:
        exit_for(end)
        return
    owner = record.get("preflight", {}).get("lock", {}).get("owner_pid")
    if owner is None:
        if read_lock(lock_file) is not None:
            raise Refused(f"lock is not this run's ({run.get('id')}); left in place")
        return
    if not release_lock(lock_file, owner, run["id"]):
        raise Refused(f"lock is not this run's ({run['id']}); left in place")


def guard(host, lock_file, run_dir, iterations=None):
    run_dir = pathlib.Path(run_dir).resolve()
    def owner_dead():
        result = restore_run(host, lock_file, run_dir, run_dir.name, "guard")
        return result is None or not result["attempt"]["failures"]
    return ch.guard_loop(host, lock_file, run_dir.name, run_dir, owner_dead, iterations)


def cmd_guard(args):
    guard(Host(), lock_path(), args.run_dir)


def cmd_restore(args):
    lock_file = lock_path()
    with guarded(lock_file):
        hold = ch.read_hold(ch.hold_file(lock_file))
    if hold is None:
        return
    host = Host()
    if host.pid_alive(hold["owner_pid"]):
        raise Refused(f"run {hold['run_id']}'s owner pid {hold['owner_pid']} is alive; release that run instead")
    result = restore_run(host, lock_file, hold["run_dir"], hold["run_id"], "hand")
    if result is not None and result["attempt"]["failures"]:
        raise CannotRun(f"still held: {failure_list(result['attempt'])}")
```

In `build_parser`, add `pre.add_argument("--hold-settle", type=int, default=10)` and:

```python
    grd = subs.add_parser("guard")
    grd.add_argument("run_dir")
    grd.set_defaults(func=cmd_guard)
    rst = subs.add_parser("restore")
    rst.set_defaults(func=cmd_restore)
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_meta`
Expected: PASS (all classes, including the end-to-end test against fake host commands).
Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_capture_hold`
Expected: PASS.

- [ ] **Step 5: Confirm no other test can reach the real host**

Run: `rg -n "capture-meta|capture_meta" tools/test_*.py | rg -v "test_capture_meta|test_capture_hold"`
Expected: only the stubbed uses in `tools/test_optic_settling.py` (a stub `capture-meta` under its stub dir) and the text checks in `tools/test_glass_optic_smoke.py`; none runs `tools/capture-meta` itself.

- [ ] **Step 6: Commit**

```bash
tasks done <task-5-id> "capture-meta: hold in preflight, release, guard and restore"
tasks check
git add tools/capture-meta tools/test_capture_meta.py tasks/
git commit -m "feat(capture): hold disturbers through preflight and release (material-188aaa)"
```

### Task 6: Fixtures fail a disturbed run and hash the finished record

**Files:**
- Modify: `docs/materials/scripts/glass-optic-smoke-lib.sh:57-66,491-497`
- Modify: `docs/materials/scripts/ring-motion-clips.sh:85`, `drag-lag-clips.sh:76`, `focus-swap-clips.sh:84`
- Modify: `docs/materials/scripts/optic-settling-smoke.sh:31-33,116-123`
- Test: `tools/test_glass_optic_smoke.py`, `tools/test_optic_settling.py`

**Interfaces:**
- Consumes: Task 5's release exit codes and idempotent second release.

- [ ] **Step 1: Write the failing tests**

In `tools/test_glass_optic_smoke.py`, replace `test_cleanup_ignores_capture_release_refusal` and `test_finish_hashes_every_file_but_the_manifest` with:

```python
    def test_cleanup_fails_on_capture_release_failure(self):
        with tempfile.TemporaryDirectory() as out:
            script = ('capture_meta() { printf "%s" "$*" > "$OUT/release-call"; return 1; }\n'
                      'stop_weston() { :; }\nremove_runtime_dir() { :; }\n' + self.function('cleanup') +
                      '\nOUT=$1; CAP_PID=; NIRI_PID=; cleanup')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertEqual((Path(out) / 'release-call').read_text(), f'release {out}')

    def test_finish_releases_before_hashing_every_file_but_the_manifest(self):
        with tempfile.TemporaryDirectory() as out:
            for name in ('a.png', 'b.kdl', 'c.tracy', 'niri.log'):
                (Path(out) / name).write_text(name)
            (Path(out) / 'sub').mkdir(); (Path(out) / 'sub' / 'd.csv').write_text('d')
            script = ('rg() { return 1; }\n'
                      'capture_meta() { echo "{\\"hold_end\\": \\"$*\\"}" > "$OUT/capture.json"; }\n'
                      + self.function('finish') +
                      '\nOUT=$1; finish >/dev/null; cut -d" " -f3- "$OUT/SHA256SUMS" | LC_ALL=C sort')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.split(), ['./a.png', './b.kdl', './c.tracy', './capture.json', './niri.log', './sub/d.csv'])
            check = subprocess.run(['sha256sum', '-c', '--quiet', 'SHA256SUMS'], cwd=out, capture_output=True, text=True)
            self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

    def test_finish_writes_sums_then_fails_a_disturbed_run(self):
        with tempfile.TemporaryDirectory() as out:
            (Path(out) / 'a.png').write_text('a')
            script = ('rg() { return 1; }\nfail() { echo "FAIL: $*" >&2; exit 1; }\n'
                      'capture_meta() { return 1; }\n' + self.function('finish') + '\nOUT=$1; finish')
            result = self.run_bash(script, out)
            self.assertEqual(result.returncode, 1)
            self.assertIn('capture release reported 1', result.stderr)
            self.assertNotIn('PASS', result.stdout)
            self.assertTrue((Path(out) / 'SHA256SUMS').is_file())

    def test_clip_fixtures_fail_on_capture_release_failure(self):
        scripts = Path(__file__).resolve().parents[1] / 'docs/materials/scripts'
        for name in ('ring-motion-clips.sh', 'drag-lag-clips.sh', 'focus-swap-clips.sh'):
            text = (scripts / name).read_text()
            self.assertIn('capture_meta release "$OUT" || rc=1', text, name)
            self.assertNotIn('capture_meta release "$OUT" || true', text, name)
```

(`subprocess` is already imported in that file; add `import subprocess` at the top if not.)

In `tools/test_optic_settling.py`, in `assert_cleaned_up`, after the `sha256sum -c` check add:

```python
        if (self.out / 'capture.json').exists():
            self.assertIn(' ./capture.json\n', sums)
```

and add to the same test class:

```python
    def test_exit_path_releases_before_writing_sums(self):
        text = (self.root / 'docs/materials/scripts/optic-settling-smoke.sh').read_text()
        body = text[text.index('on_exit() {'):text.index('write_sums() {')]
        self.assertLess(body.index('capture_meta release "$OUT" || rc=1'), body.index('write_sums || rc=1'))
        self.assertNotIn("! -name capture.json", text)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_glass_optic_smoke`
Expected: FAIL — cleanup exits 0; finish does not call `capture_meta`.
Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling`
Expected: FAIL in `test_exit_path_releases_before_writing_sums`.

- [ ] **Step 3: Change the fixtures**

`docs/materials/scripts/glass-optic-smoke-lib.sh`, in `cleanup`:

```bash
    capture_meta release "$OUT" || rc=1
```

and `finish`:

```bash
finish() {
    # Release first: it completes capture.json (the hold's end), which the manifest then covers.
    # A second release from cleanup changes nothing.
    local released=0
    capture_meta release "$OUT" || released=$?
    (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > SHA256SUMS)
    [ "$released" = 0 ] || fail "capture release reported $released (a disturbed run or an unrestored hold); see $OUT/capture.json"
    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
        fail "material error, fallback or panic in niri.log"
    fi
    echo "PASS: artifacts in $OUT"
}
```

`ring-motion-clips.sh:85`, `drag-lag-clips.sh:76`, `focus-swap-clips.sh:84`: `capture_meta release "$OUT" || true` → `capture_meta release "$OUT" || rc=1`.

`docs/materials/scripts/optic-settling-smoke.sh`: header lines 31-33 become

```bash
# invalid case, a compositor panic, a refused/interrupted run, or a run the
# capture hold found disturbed. Every exit keeps the partial evidence,
# releases the capture lock (which completes capture.json) and then writes
# SHA256SUMS over everything, capture.json included.
```

in `on_exit`, swap the two lines so release comes first:

```bash
    capture_meta release "$OUT" || rc=1
    write_sums || rc=1
```

and `write_sums` drops the exclusion:

```bash
write_sums() {
    (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z \
        | xargs -0 -r sha256sum > SHA256SUMS)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_glass_optic_smoke`
Expected: PASS.
Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling`
Expected: PASS. If the stub `capture-meta` in `test_optic_settling.py` writes `capture.json` on `release`, the new `assertIn(' ./capture.json\n', sums)` now covers it; if `test_failed_preflight_releases_and_keeps_its_record` relied on the old order, its expectations (`['preflight', 'release']` then `SHA256SUMS` present) still hold.

- [ ] **Step 5: Commit**

```bash
tasks done <task-6-id> "fixtures fail disturbed runs and hash the released record"
tasks check
git add docs/materials/scripts tools/test_glass_optic_smoke.py tools/test_optic_settling.py tasks/
git commit -m "fix(capture): fixtures fail a disturbed run and hash the finished record (material-188aaa)"
```

### Task 7: Host setup, full gate, and tonight's quiet run staged

**Files:**
- Modify: `docs/materials/capture-host-setup.md`
- Host (outside the repository, owner-visible): `~/.config/niri-material/capture-hold`

**Interfaces:**
- Consumes: everything above.

- [ ] **Step 1: Document the hold**

Append to `docs/materials/capture-host-setup.md`, before "Running the dedicated lane":

````markdown
## Disturber hold

`capture-meta preflight` holds the host's disturbers for the whole run
([design](../specs/2026-10-04-capture-disturber-hold-design.md)): every
active user timer, the services named in
`${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold`, and on the
desktop Noctalia's caffeine and monitor power. `release` restores them and
scans the journal; a disturbed run fails its fixture.

The reference host's file:

```text
# Crash-loops every ~14 s while the desktop is stopped (DISPLAY=:0 override).
dropbox.service
```

After a killed run the guard unit (`capture-meta-guard-*.service`) restores
within seconds. If it could not, `tools/capture-meta restore` restores by
hand; `$XDG_RUNTIME_DIR/capture-meta.hold.json` lists what is still held,
each item with its restore command.
````

- [ ] **Step 2: Write the reference host's config file**

```bash
mkdir -p ~/.config/niri-material
printf '%s\n' '# Crash-loops every ~14 s while the desktop is stopped (DISPLAY=:0 override).' 'dropbox.service' \
  > ~/.config/niri-material/capture-hold
cat ~/.config/niri-material/capture-hold
```

Expected: the two lines. Note it on the task: `tasks note material-188aaa "host: wrote ~/.config/niri-material/capture-hold (dropbox.service); remove to stop holding services"`.

- [ ] **Step 3: Run the full tooling gate and the check gate**

Run: `just --set fast_cmd 'python3 -m unittest discover -s tools 2>&1' test-fast`
Expected: PASS.
Run: `just check`
Expected: PASS (hygiene, rustfmt, clippy, tooling tests, task checks).

- [ ] **Step 4: Find the identified Tracy snapshot for the evidence run**

```bash
ls -d "$NIRI_MATERIAL_WORK_ROOT"/*/ 2>/dev/null; find "$NIRI_MATERIAL_WORK_ROOT" -name 'niri-tracy.identity.json' -newer docs/specs/2026-10-02-real-tty-settling-lane-design.md 2>/dev/null | head
```

Expected: a snapshot directory with `niri-tracy` and its `.identity.json`. If none exists, the runbook's step 4 builds one first (`optic-settling-smoke.sh prepare`, per the dedicated-lane section of capture host setup) and the estimate grows by its build minutes.

- [ ] **Step 5: Commit, then park the task for tonight's quiet run with the runbook**

```bash
tasks check
git add docs/materials/capture-host-setup.md tasks/
git commit -m "docs(capture): record the disturber hold in capture host setup (material-188aaa)"
tasks park material-188aaa "TTY quiet run, desktop stopped, in .worktrees/disturber-hold with T=tools/capture-meta and W=\$NIRI_MATERIAL_WORK_ROOT/hold-pilot: (1) round trip 2 min: \$T preflight \$W/rt-1 --lane dedicated --task material-188aaa --fixture hold-pilot --owner-pid \$\$; systemctl --user list-timers (none active); \$T release \$W/rt-1 → exit 0, show: clean; (2) positive control 2 min: same with W/pc-1 and 'systemd-run --user --on-active=30s true; sleep 45' between → exit 1, timer-added + timer-fired; (3) kill recovery 1 min: bash -c '\$T preflight \$W/kill-1 ... --owner-pid \$\$ && sleep 600' & then kill -9 it after 30 s; within 5 s timers active again, show: restore by guard; (4) evidence: optic-settling-smoke.sh pilot --lane dedicated with CASES='drm-aurora tty-resume screencast' and the snapshot from step 4, OUT under \$W. Earlier hazard: dropbox crash-loop is now held by the config file." --reason quiet --waiting-on user --needs headless --minutes 25
```

## Self-Review Notes

- **Spec coverage:** §3.1 timers → Task 1; §3.2 services → Task 1; §3.3 desktop, socket rules, power-off check → Tasks 1–2; §3.4 system timers recorded → Task 1, flagged → Task 3; §4.1 hold file, write-ahead, run matching, guarded → Task 2; §4.2 order (guard before first change), settle wait, rollback policy → Task 5; §4.3 release, scan once, retries, exit codes → Tasks 4–5; §4.4 guard (watch, `restoring`, retries), `restore`, next-preflight recovery → Tasks 3 and 5; §5 scan kinds, invocation ids, window → Task 3; fixture changes and checksums (§2) → Task 6; §6 unit tests → Tasks 1–6, live runs → Task 7's park note.
- **Deviation recorded:** the spec names the wake log `capture-meta.wakes.<run id>.jsonl`; the plan keys it on `run_key(run_dir)` (the same key as the guard unit), because run ids can collide (spec §4.1). Hold-time invocation ids live in the hold file's `invocations` rather than inside `timer_map`.
- **Added beyond the spec:** the `monitor-unwatched` kind (Review Focus 3) and the guard's bounded retries (`GUARD_RETRIES` × `GUARD_RETRY_S`, 10 minutes) so a permanently failing restore ends in a CannotRun pointing at `capture-meta restore` rather than a guard that never exits.
