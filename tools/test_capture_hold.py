"""Unit tests for tools/capture_hold.py, against fake_capture_host."""
import json
import pathlib
import sys
import tempfile
import unittest

# capture-meta imports capture_hold by its bare name; import it the same way so both see one
# module (and one CannotRun), whether run as tools.test_capture_hold or by discover.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import capture_hold as ch  # noqa: E402
from fake_capture_host import FakeHost  # noqa: E402

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

    def test_unreadable_connector_state_refuses_the_desktop_hold(self):
        host = self.host(sockets=["niri.w.1.sock"], live=["niri.w.1.sock"], connectors=LIT)
        (host.sysfs / "card1-DP-1" / "enabled").unlink()
        with self.assertRaisesRegex(ch.CannotRun, "cannot tell whether card1-DP-1 is a lit monitor"):
            ch.plan_hold(host)

    def test_unreachable_user_manager_changes_nothing(self):
        host = self.host(timers=["wali-rotate.timer"])
        host.fail[("systemctl", "--user", "list-timers", "--output=json", "--no-pager")] = "Failed to connect to bus"
        before = host.snapshot()
        with self.assertRaisesRegex(ch.CannotRun, "Failed to connect to bus"):
            ch.plan_hold(host)
        self.assertEqual(host.snapshot(), before)


if __name__ == "__main__":
    unittest.main()
