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
    def test_a_timers_and_services_plan_never_probes_idle_or_monitors(self):
        # A desktop whose noctalia status fails and whose connector cannot be read:
        # the full hold refuses on either; a pixel hold must not query them.
        def host():
            h = self.host(timers=["wali-rotate.timer"], sockets=["niri.w.1.sock"], live=["niri.w.1.sock"],
                          connectors=LIT)
            h.fail[("noctalia", "msg", "status")] = "noctalia: no shell running"
            (h.sysfs / "card1-DP-1" / "enabled").unlink()
            return h
        full = host()
        with self.assertRaises(ch.CannotRun):
            ch.plan_hold(full, ch.desktop_socket(full), ch.HOLD_KINDS)
        pixel = host()
        plan = ch.plan_hold(pixel, ch.desktop_socket(pixel), ("timer", "service"))
        self.assertEqual([i["kind"] for i in plan["items"]], ["timer"])
        self.assertEqual((plan["desktop"], plan["not_held"]), ("present", []))
        self.assertNotIn(("noctalia", "msg", "status"), pixel.calls)

    def test_holds_active_non_transient_user_timers_and_maps_both_managers(self):
        host = self.host(timers=["wali-rotate.timer", "familiar-reap.timer"], system_timers=["man-db.timer"])
        host.add_timer("run-r1.timer", transient="yes")
        host.add_timer("idle.timer", active=False)
        plan = ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
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
        plan = ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
        self.assertEqual(plan["invocations"], {"user": {"familiar-reap.service": "abc"}})
        self.assertEqual(plan["active_at_hold"], [{"manager": "user", "unit": "familiar-reap.service"}])

    def test_declared_services_running_are_held_and_others_recorded(self):
        host = self.host(units={"dropbox.service": {"ActiveState": "activating"},
                                "syncthing.service": {"ActiveState": "inactive"}})
        host.config.parent.mkdir(parents=True)
        host.config.write_text("# crash-loops without a compositor\ndropbox.service\n\nsyncthing.service  # idle\n")
        plan = ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
        self.assertEqual([(i["kind"], i["unit"]) for i in plan["items"]], [("service", "dropbox.service")])
        self.assertEqual(plan["not_held"], [{"kind": "service", "unit": "syncthing.service", "reason": "not_running"}])
        self.assertEqual(plan["config"], str(host.config))

    def test_declared_unit_that_does_not_exist_cannot_run(self):
        host = self.host()
        host.config.parent.mkdir(parents=True); host.config.write_text("dropbx.service\n")
        with self.assertRaisesRegex(ch.CannotRun, "dropbx.service, which does not exist"):
            ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)

    def test_desktop_adds_idle_then_monitors_and_records_lock_state(self):
        host = self.host(sockets=["niri.wayland-1.5.sock"], live=["niri.wayland-1.5.sock"], locked=True,
                         connectors=LIT)
        plan = ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
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
        plan = ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
        self.assertEqual([i["kind"] for i in plan["items"]], ["monitors"])
        self.assertEqual(plan["not_held"], [{"kind": "idle", "reason": "noctalia not on PATH"}])

    def test_unreadable_connector_state_refuses_the_desktop_hold(self):
        host = self.host(sockets=["niri.w.1.sock"], live=["niri.w.1.sock"], connectors=LIT)
        (host.sysfs / "card1-DP-1" / "enabled").unlink()
        with self.assertRaisesRegex(ch.CannotRun, "cannot tell whether card1-DP-1 is a lit monitor"):
            ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)

    def test_unreachable_user_manager_changes_nothing(self):
        host = self.host(timers=["wali-rotate.timer"])
        host.fail[("systemctl", "--user", "list-timers", "--output=json", "--no-pager")] = "Failed to connect to bus"
        before = host.snapshot()
        with self.assertRaisesRegex(ch.CannotRun, "Failed to connect to bus"):
            ch.plan_hold(host, ch.desktop_socket(host), ch.HOLD_KINDS)
        self.assertEqual(host.snapshot(), before)


class HoldFileTests(TempHost):
    def setUp(self):
        self.host_ = self.host(timers=["wali-rotate.timer", "familiar-reap.timer"],
                               units={"dropbox.service": {"ActiveState": "active"}},
                               sockets=["niri.w.1.sock"], live=["niri.w.1.sock"], connectors=LIT)
        self.host_.config.parent.mkdir(parents=True); self.host_.config.write_text("dropbox.service\n")
        self.lock = self.host_.runtime / "capture-meta.lock"
        self.run_dir = self.host_.runtime.parent / "runs" / "pilot-1"; self.run_dir.mkdir(parents=True)

    def hold(self):
        plan = ch.plan_hold(self.host_, ch.desktop_socket(self.host_), ch.HOLD_KINDS)
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
                _, attempt = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "guard")
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
        _, attempt = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        starts = [c[3] for c in self.host_.calls if c[:3] == ("systemctl", "--user", "start")]
        self.assertEqual(starts, ["dropbox.service", "wali-rotate.timer", "familiar-reap.timer"])
        self.assertEqual(attempt["by"], "release")
        self.assertEqual(attempt["notes"], ["familiar-reap.timer no longer exists"])
        self.assertEqual([f["unit"] for f in attempt["failures"]], ["dropbox.service"])
        self.assertEqual(attempt["failures"][0]["restore"], "systemctl --user start dropbox.service")
        left = ch.read_hold(ch.hold_file(self.lock))
        self.assertEqual([i.get("unit") for i in left["held"]], ["dropbox.service"])
        self.assertEqual(len(left["items"]), 5)          # the scan's input never shrinks
        self.assertTrue(left["restoring"])

    def test_until_is_fixed_by_the_first_attempt(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        self.host_.fail[("systemctl", "--user", "start", "dropbox.service")] = "Job failed"
        first, _ = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        self.host_.clock += 5_000_000
        second, _ = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        self.assertEqual(second["until_us"], first["until_us"])

    def test_record_runs_before_the_hold_file_goes(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        seen = []
        def record(hold, wakes, attempt):
            seen.append((ch.hold_file(self.lock).exists(), len(hold["items"]), attempt["failures"]))
        ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release", record)
        self.assertEqual(seen, [(True, 5, [])])          # all five items still there for the scan
        self.assertFalse(ch.hold_file(self.lock).exists())

    def test_a_kill_inside_record_leaves_the_hold_for_the_next_attempt(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        def killed(hold, wakes, attempt):
            raise self.host_.Killed()
        with self.assertRaises(self.host_.Killed):
            ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release", killed)
        self.assertTrue(ch.hold_file(self.lock).exists())
        seen = []
        _, attempt = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "guard",
                                            lambda hold, wakes, a: seen.append(len(hold["items"])))
        self.assertEqual((attempt["failures"], attempt["undone"], seen), ([], 0, [5]))
        self.assertFalse(ch.hold_file(self.lock).exists())

    def test_finalize_removes_only_a_fully_restored_hold(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        self.assertIsNone(ch.finalize(self.lock, "pilot-1", self.run_dir))     # still held
        path = ch.hold_file(self.lock); hold = ch.read_hold(path)
        hold["held"] = []; ch.write_hold(path, hold)
        self.assertEqual(ch.finalize(self.lock, "pilot-1", self.run_dir)["run_id"], "pilot-1")
        self.assertFalse(path.exists())

    def test_apply_stops_once_a_restore_has_started(self):
        plan = self.hold()
        first, rest = {**plan, "items": plan["items"][:1]}, plan["items"][1:]
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, first)
        self.host_.fail[("systemctl", "--user", "start", "familiar-reap.timer")] = "busy"
        ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "guard")
        stops = len([c for c in self.host_.calls if c[:3] == ("systemctl", "--user", "stop")])
        with self.assertRaisesRegex(ch.CannotRun, "being restored; holding nothing more"):
            ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, {**plan, "items": rest})
        self.assertEqual(len([c for c in self.host_.calls if c[:3] == ("systemctl", "--user", "stop")]), stops)

    def test_apply_stops_when_the_owner_is_dead(self):
        plan = self.hold()
        self.host_.alive[4242] = False
        with self.assertRaisesRegex(ch.CannotRun, "owner pid 4242 of run pilot-1 is dead"):
            ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        self.assertFalse([c for c in self.host_.calls if c[:3] == ("systemctl", "--user", "stop")])

    def test_other_runs_hold_is_left_alone(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        other = self.run_dir.parent / "elsewhere" / "pilot-1"
        self.assertIsNone(ch.restore_transaction(self.host_, self.lock, "pilot-1", other, "release"))
        self.assertTrue(ch.hold_file(self.lock).exists())

    def test_restore_of_a_gone_desktop_notes_it(self):
        plan = self.hold()
        ch.apply_hold(self.host_, self.lock, "pilot-1", self.run_dir, plan)
        (self.host_.runtime / "niri.w.1.sock").unlink()
        _, attempt = ch.restore_transaction(self.host_, self.lock, "pilot-1", self.run_dir, "release")
        self.assertEqual(attempt["failures"], [])
        self.assertIn("desktop gone; monitors left as they are", attempt["notes"])
        self.assertIn("desktop gone; caffeine left as it is", attempt["notes"])

    def test_unreadable_hold_file_cannot_run(self):
        ch.hold_file(self.lock).write_text("{")
        with self.assertRaisesRegex(ch.CannotRun, "restore its items by hand"):
            ch.read_hold(ch.hold_file(self.lock))


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

    def test_wakes_outside_the_window_are_ignored(self):
        result = self.scan(wakes=[{"at_us": T0 - 1, "connector": "card1-DP-1"},
                                  {"at_us": T0 + 600_000_000, "connector": "card1-DP-1"}])
        self.assertEqual(result, ("clean", []))

    def test_wakes_and_unwatched_connectors(self):
        result = self.scan(wakes=[{"at_us": T0 + 9, "connector": "card1-DP-1"},
                                  {"at_us": T0 + 10, "connector": "card1-DP-1", "error": "gone"}])
        self.assertEqual(self.kinds(result), [("monitor-woke", "card1-DP-1"), ("monitor-unwatched", "card1-DP-1")])

    def test_unreadable_journal_cannot_run(self):
        host = self.host()
        host.fail[("journalctl", "--user", "-o", "json", "--no-pager", f"--since=@{T0 // 1_000_000 - 1}")] = "denied"  # noqa: E501
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
        path = ch.hold_file(self.lock); hold = ch.read_hold(path)
        hold["restoring"] = True; ch.write_hold(path, hold)
        self.h.wake("card1-DP-1")
        ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir, lambda: True, iterations=3)
        self.assertEqual(self.wakes(), [])

    def test_a_round_waits_for_a_restore_in_progress_and_then_logs_nothing(self):
        import threading
        entered, release = threading.Event(), threading.Event()
        def restoring():
            with ch.guarded(self.lock):
                path = ch.hold_file(self.lock); hold = ch.read_hold(path)
                hold["restoring"] = True; ch.write_hold(path, hold)
                entered.set(); release.wait(5)
                self.h.wake("card1-DP-1")        # the restore's own power-on
        worker = threading.Thread(target=restoring); worker.start()
        entered.wait(5)
        done = threading.Event()
        threading.Thread(target=lambda: (ch.guard_loop(self.h, self.lock, "pilot-1", self.run_dir,
                                                       lambda: True, iterations=1), done.set())).start()
        self.assertFalse(done.wait(0.3))           # blocked behind the restore's lock
        release.set(); worker.join(5)
        self.assertTrue(done.wait(5))
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


if __name__ == "__main__":
    unittest.main()
