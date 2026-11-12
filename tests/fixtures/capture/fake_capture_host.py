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

    def run(self, *command, extra_env=None, timeout=None):
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
