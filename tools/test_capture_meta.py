"""Unit tests for tools/capture-meta."""
import importlib.machinery
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import threading
import unittest
from unittest import mock

for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

spec = importlib.util.spec_from_loader(
    "capture_meta",
    importlib.machinery.SourceFileLoader(
        "capture_meta", str(pathlib.Path(__file__).with_name("capture-meta"))))
cm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cm)


class LockTests(unittest.TestCase):
    def test_acquire_release_and_ownership_check(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            info = cm.acquire_lock(path, owner_pid=111, run_id="run-a", alive=lambda pid: True)
            self.assertEqual(info, {"path": str(path), "owner_pid": 111, "run_id": "run-a", "reclaimed": False})
            self.assertEqual(sorted(p.name for p in path.parent.iterdir()), ["capture-meta.lock", "capture-meta.lock.d"])
            self.assertEqual(cm.read_lock(path)["run_id"], "run-a")
            self.assertFalse(cm.release_lock(path, owner_pid=222, run_id="run-b"))
            self.assertTrue(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a"))
            self.assertFalse(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a"))

    def test_half_written_or_corrupt_lock_is_never_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            path.write_text("")
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False)
            self.assertEqual(path.read_text(), "")
            path.write_text("{not json")
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False)
            self.assertFalse(cm.release_lock(path, 222, "run-b"))

    def test_binary_corrupt_lock_is_never_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            corrupt = b"\xff"
            path.write_bytes(corrupt)
            with self.assertRaises(cm.CannotRun):
                cm.read_lock(path)
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False)
            self.assertFalse(cm.release_lock(path, 222, "run-b"))
            self.assertEqual(path.read_bytes(), corrupt)

    def test_guard_serializes_acquire_against_a_concurrent_holder(self):
        import fcntl
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            guard = path.with_name(path.name + ".d"); guard.mkdir()
            fd = os.open(guard, os.O_RDONLY); fcntl.flock(fd, fcntl.LOCK_EX)
            done = threading.Event(); result = {}
            def worker():
                result["info"] = cm.acquire_lock(path, 333, "run-c", alive=lambda pid: True); done.set()
            threading.Thread(target=worker, daemon=True).start()
            self.assertFalse(done.wait(0.3))
            fcntl.flock(fd, fcntl.LOCK_UN); os.close(fd)
            self.assertTrue(done.wait(3.0))
            self.assertEqual(result["info"]["run_id"], "run-c")

    def test_held_by_live_pid_refuses_stale_is_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            cm.acquire_lock(path, 111, "run-a", alive=lambda pid: True)
            with self.assertRaises(cm.Refused) as ctx:
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: True)
            self.assertIn("run-a", str(ctx.exception))
            info = cm.acquire_lock(path, 222, "run-b", alive=lambda pid: pid != 111)
            self.assertEqual(info["reclaimed"], True)
            self.assertEqual(info["previous_owner_pid"], 111)
            self.assertEqual(cm.read_lock(path)["owner_pid"], 222)

    def test_two_processes_racing_admit_exactly_one(self):
        import subprocess
        import sys
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            code = ("import sys, importlib.util, importlib.machinery, pathlib\n"
                    "spec = importlib.util.spec_from_loader('cm', importlib.machinery.SourceFileLoader('cm', sys.argv[1]))\n"
                    "cm = importlib.util.module_from_spec(spec); spec.loader.exec_module(cm)\n"
                    "try:\n    cm.acquire_lock(pathlib.Path(sys.argv[2]), int(sys.argv[3]), sys.argv[3], alive=lambda p: True)\n"
                    "except cm.Refused:\n    sys.exit(1)\n")
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            procs = [subprocess.Popen([sys.executable, "-c", code, tool, str(path), str(1000 + i)]) for i in range(8)]
            codes = sorted(p.wait() for p in procs)
            self.assertEqual(codes, [0] + [1] * 7)

    def test_public_read_waits_for_guard(self):
        import fcntl
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            path.write_text('{"owner_pid": 111, "run_id": "run-a"}')
            guard = path.with_name(path.name + ".d"); guard.mkdir()
            fd = os.open(guard, os.O_RDONLY); fcntl.flock(fd, fcntl.LOCK_EX)
            done = threading.Event()
            threading.Thread(target=lambda: (cm.read_lock(path), done.set()), daemon=True).start()
            self.assertFalse(done.wait(0.3))
            fcntl.flock(fd, fcntl.LOCK_UN); os.close(fd)
            self.assertTrue(done.wait(3.0))

    def test_rejects_invalid_owner_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            for pid, run_id in ((0, "run"), (-1, "run"), (True, "run"), (1, ""), (1, None)):
                with self.subTest(pid=pid, run_id=run_id), self.assertRaises(cm.CannotRun):
                    cm.acquire_lock(path, pid, run_id)
            for holder in ({"owner_pid": 0, "run_id": "run"}, {"owner_pid": True, "run_id": "run"},
                           {"owner_pid": 1, "run_id": ""}, {"owner_pid": 1, "run_id": 2}):
                path.write_text(json.dumps(holder))
                with self.subTest(holder=holder), self.assertRaises(cm.CannotRun):
                    cm.read_lock(path)


class RecordTests(unittest.TestCase):
    def test_write_section_creates_record_and_refuses_rewrite(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            self.assertEqual(cm.load_record(run), {})
            cm.write_section(run, "run", {"id": "trace-1"})
            record = json.loads((run / "capture.json").read_text())
            self.assertEqual(record["schema"], cm.SCHEMA)
            self.assertEqual(record["run"], {"id": "trace-1"})
            with self.assertRaises(cm.CannotRun):
                cm.write_section(run, "run", {"id": "other"})
            self.assertEqual(cm.load_record(run)["run"], {"id": "trace-1"})

    def test_append_sub_run_accumulates_in_order(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            cm.append_sub_run(run, {"name": "a", "verdict": "settled"})
            cm.append_sub_run(run, {"name": "b", "verdict": "refused"})
            self.assertEqual([e["name"] for e in cm.load_record(run)["sub_runs"]], ["a", "b"])

    def test_load_record_rejects_malformed_and_unknown_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            (run / "capture.json").write_text("{not json")
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)
            (run / "capture.json").write_text(json.dumps({"schema": 99}))
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)

    def test_load_record_rejects_non_object_json(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            (run / "capture.json").write_text("[]")
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)

    def test_append_sub_run_rejects_non_list(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            cm.write_section(run, "sub_runs", {})
            with self.assertRaises(cm.CannotRun):
                cm.append_sub_run(run, {"name": "a"})

    def test_expected_read_failures_are_cannot_run(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            (run / "capture.json").mkdir()
            with self.assertRaises(cm.CannotRun):
                cm.load_record(run)

    def test_malformed_sections_make_cli_consumers_exit_two_without_mutation(self):
        cases = {
            "show": ({"schema": 1, "provenance": {"binaries": [{}]}}, ["show"]),
            "settle": ({"schema": 1, "run": {"id": "settle", "lane": "headless"},
                        "baseline": {}, "preflight": {"verdict": "quiet", "thresholds": {},
                                                       "lock": {"owner_pid": 1}}},
                       ["settle", "--sub-run", "A", "--seconds", "1"]),
            "release": ({"schema": 1, "run": {"task": "t"}}, ["release"]),
        }
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            runtime = root / "runtime"; runtime.mkdir()
            lock = runtime / cm.LOCK_NAME
            lock.write_text('{"owner_pid": 1, "run_id": "foreign"}')
            fake = root / "nvidia-smi"; fake.write_text("#!/bin/sh\nexit 1\n"); fake.chmod(0o755)
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            for name, (record, args) in cases.items():
                run = root / name; run.mkdir()
                path = run / cm.RECORD; path.write_text(json.dumps(record))
                before_record = path.read_bytes(); before_lock = lock.read_bytes()
                result = subprocess.run(
                    [sys.executable, tool, args[0], str(run), *args[1:]],
                    env={**os.environ, "XDG_RUNTIME_DIR": str(runtime),
                         "PATH": f"{root}:{os.environ['PATH']}"},
                    capture_output=True, text=True)
                with self.subTest(command=name):
                    self.assertEqual(result.returncode, 2, result.stderr)
                    self.assertNotIn("Traceback", result.stderr)
                    self.assertEqual(path.read_bytes(), before_record)
                    self.assertEqual(lock.read_bytes(), before_lock)


class IdentityTests(unittest.TestCase):
    def repo(self, root):
        import subprocess
        subprocess.run(["git", "-C", str(root), "init", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q",
                       "--allow-empty", "-m", "init"], check=True)

    def test_hashes_match_sha256sum_and_records_source(self):
        import hashlib
        with tempfile.TemporaryDirectory() as directory:
            # The run directory sits beside the checkout, not inside it: files written into the
            # source tree would be untracked and correctly make the tree dirty.
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            (run / "niri").write_bytes(b"binary"); (run / "A.kdl").write_text("glass")
            cm.identity(run, source=root, binaries=[run / "niri"], inputs=[run / "A.kdl"],
                        config=["preset=aurora", "glass.ior=1.24"])
            prov = cm.load_record(run)["provenance"]
            self.assertEqual(prov["binaries"], [{"name": "niri", "sha256": hashlib.sha256(b"binary").hexdigest()}])
            self.assertEqual(prov["inputs"][0]["name"], "A.kdl")
            self.assertEqual(len(prov["source"]["commit"]), 40)
            self.assertEqual(prov["source"]["branch"], "main")
            self.assertFalse(prov["source"]["dirty"])
            self.assertEqual(prov["config"], {"preset": "aurora", "glass.ior": "1.24"})

    def test_dirty_tree_records_diff_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            (root / "tracked.txt").write_text("v1\n")
            import subprocess
            subprocess.run(["git", "-C", str(root), "add", "tracked.txt"], check=True)
            subprocess.run(["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "t"], check=True)
            (root / "tracked.txt").write_text("v2\n")
            cm.identity(run, source=root, binaries=[], inputs=[], config=[])
            source = cm.load_record(run)["provenance"]["source"]
            self.assertTrue(source["dirty"])
            self.assertEqual(len(source["diff_sha256"]), 64)

    def test_untracked_source_file_makes_the_tree_dirty_and_enters_the_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); self.repo(root)
            (root / "new_optic.rs").write_text("fn a() {}\n")
            first = cm.source_facts(root)
            self.assertTrue(first["dirty"])
            self.assertEqual(first["untracked"], ["new_optic.rs"])
            (root / "new_optic.rs").write_text("fn b() {}\n")
            self.assertNotEqual(cm.source_facts(root)["diff_sha256"], first["diff_sha256"])
            (root / ".gitignore").write_text("new_optic.rs\n")
            still = cm.source_facts(root)
            self.assertEqual(still["untracked"], [".gitignore"])

    def test_untracked_source_handles_unusual_filenames(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); self.repo(root)
            names = ["café.rs", "tab\toptic.rs", "line\noptic.rs"]
            for name in names:
                (root / name).write_text(name)
            facts = cm.source_facts(root)
            self.assertEqual(facts["untracked"], sorted(names))
            self.assertEqual(len(facts["diff_sha256"]), 64)

    def test_untracked_symlink_hashes_its_target_text(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); self.repo(root)
            (root / "a.rs").write_text("same\n"); (root / "b.rs").write_text("same\n")
            link = root / "optic.rs"; link.symlink_to("a.rs")
            first = cm.source_facts(root)["diff_sha256"]
            link.unlink(); link.symlink_to("b.rs")
            second = cm.source_facts(root)["diff_sha256"]
            link.unlink(); link.symlink_to("missing.rs")
            broken = cm.source_facts(root)["diff_sha256"]
            self.assertEqual(len({first, second, broken}), 3)

    def test_missing_path_refuses_and_writes_nothing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir(); self.repo(root)
            with self.assertRaises(cm.Refused):
                cm.identity(run, source=root, binaries=[run / "absent"], inputs=[], config=[])
            self.assertEqual(cm.load_record(run), {})

    def test_missing_input_cli_refuses_without_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "absent-source"; run = pathlib.Path(directory) / "run"; run.mkdir()
            self.assertEqual(cm.main(["identity", str(run), "--source", str(root),
                                      "--input", str(run / "absent")]), 1)
            self.assertEqual(cm.load_record(run), {})

    def test_git_launch_failure_is_cannot_run_and_writes_nothing(self):
        from unittest import mock
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "src"; root.mkdir(); run = pathlib.Path(directory) / "run"; run.mkdir()
            with mock.patch.object(cm.subprocess, "run", side_effect=OSError("git unavailable")):
                self.assertEqual(cm.main(["identity", str(run), "--source", str(root)]), 2)
            self.assertEqual(cm.load_record(run), {})


class ShowTests(unittest.TestCase):
    def test_render_lists_environment_provenance_and_verdicts(self):
        record = {"schema": 1,
                  "run": {"id": "trace-1", "task": "material-x", "fixture": "f.sh", "lane": "headless",
                          "started": "2026-09-11T04:34:33-04:00", "host": "box"},
                  "environment": {"kernel": "7.2.2", "cpu": "Threadripper", "cpu_threads": 32,
                                  "memory_total_kib": 1000,
                                  "gpu": {"name": "RTX", "uuid": "GPU-1", "driver": "610", "device": "/dev/dri/renderD128"},
                                  "session": {"type": "wayland", "display": "wayland-1"}, "tools": {"weston": "15"}},
                  "baseline": {"cpu_busy_pct": 1.0, "gpu_power_w": 18.0},
                  "preflight": {"verdict": "quiet", "thresholds": {}},
                  "provenance": {"source": {"commit": "abc", "branch": "main", "dirty": False},
                                 "binaries": [{"name": "niri", "sha256": "00"}], "inputs": [], "config": {"preset": "aurora"}},
                  "sub_runs": [{"name": "gpu-plain-1", "verdict": "settled"},
                               {"name": "gpu-aurora-1", "verdict": "refused", "reason": "gpu_power_w 24.1 exceeds baseline"}]}
        text = cm.render(record)
        for needle in ("trace-1", "RTX", "610", "abc", "niri 00", "gpu-plain-1: settled",
                       "gpu-aurora-1: refused (gpu_power_w 24.1 exceeds baseline)", "preset=aurora"):
            self.assertIn(needle, text)

    def test_main_show_exit_codes(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            self.assertEqual(cm.main(["show", directory]), 2)
            cm.write_section(run, "run", {"id": "x", "task": "t", "fixture": "f", "lane": "headless",
                                          "started": "now", "host": "h"})
            self.assertEqual(cm.main(["show", directory]), 0)


class FakeProc:
    def __init__(self, ticks, load1=0.5, available_kib=80_000, total_kib=100_000):
        self.ticks = list(ticks); self.i = 0
        self._load1 = load1; self.available_kib = available_kib; self.total_kib = total_kib
    def cpu_ticks(self):
        busy, total = self.ticks[min(self.i, len(self.ticks) - 1)]; self.i += 1
        return busy, total
    def load1(self): return self._load1
    def meminfo(self): return self.available_kib, self.total_kib


class FakeGpu:
    def __init__(self, queries, clients=None, static=None):
        self.queries = list(queries); self.i = 0
        self._clients = clients or {"compute": [], "graphics": ["niri"]}
        self._static = static or {"name": "NVIDIA test", "uuid": "GPU-1", "driver": "610"}
    def query(self):
        value = self.queries[min(self.i, len(self.queries) - 1)]; self.i += 1
        return dict(value)
    def clients(self): return {k: list(v) for k, v in self._clients.items()}
    def static(self): return dict(self._static)


QUIET = {"util_pct": 0.0, "power_w": 18.4, "clock_mhz": 360, "pstate": "P8"}


def quiet_samples(n=5, **overrides):
    proc = FakeProc([(i * 2, i * 100) for i in range(n + 1)])
    gpu = FakeGpu([dict(QUIET, **overrides)] * n)
    return cm.sample_stream(proc, gpu, n, sleep=lambda s: None)


class SamplingTests(unittest.TestCase):
    def test_sample_stream_and_summary(self):
        samples = quiet_samples(3)
        self.assertEqual([round(s.cpu_busy_pct, 1) for s in samples], [2.0] * 3)
        self.assertEqual(cm.summarize(samples)["mem_total_kib"], 100_000)

    def test_proc_reader_does_not_double_count_guest_ticks(self):
        with tempfile.TemporaryDirectory() as directory:
            proc_root = pathlib.Path(directory)
            stat = proc_root / "stat"
            stat.write_text("cpu 0 0 0 0 0 0 0 0 0 0\n")
            (proc_root / "loadavg").write_text("0.5 0.4 0.3 1/1 1\n")
            (proc_root / "meminfo").write_text(
                "MemTotal: 100000 kB\nMemAvailable: 80000 kB\n")

            def advance(_):
                stat.write_text("cpu 60 20 0 920 0 0 0 0 60 20\n")

            sample = cm.sample_stream(cm.ProcReader(proc_root), FakeGpu([QUIET]), 1,
                                      sleep=advance)[0]
            self.assertEqual(sample.cpu_busy_pct, 8.0)

    def test_summary_medians_iqr_and_clients(self):
        proc = FakeProc([(0, 0), (2, 100), (4, 200), (6, 300)])
        gpu = FakeGpu([dict(QUIET, power_w=18.0), dict(QUIET, power_w=19.0), dict(QUIET, power_w=25.0, pstate="P0")])
        summary = cm.summarize(cm.sample_stream(proc, gpu, 3, sleep=lambda s: None))
        self.assertEqual(summary["gpu_power_w"], 19.0)
        self.assertEqual(summary["gpu_power_iqr_w"], 3.5)
        self.assertEqual(summary["gpu_pstates"], ["P0", "P8"])
        self.assertEqual(summary["gpu_clients"], {"compute": [], "graphics": ["niri"]})
        self.assertEqual(summary["mem_available_pct"], 80.0)
        self.assertEqual(summary["samples"], 3)


class GpuEvidenceTests(unittest.TestCase):
    def test_failed_command_reports_stdout_when_stderr_is_empty(self):
        result = subprocess.CompletedProcess([], 18,
            stdout="Failed to initialize NVML: Driver/library version mismatch\n", stderr="")
        with mock.patch.object(cm.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(cm.CannotRun, "Driver/library version mismatch"):
                object.__new__(cm.GpuReader)._run("nvidia-smi", "--query-gpu=name")

    XML = '<nvidia_smi_log><gpu><processes>{}</processes></gpu></nvidia_smi_log>'
    ENTRY = '<process_info><type>{}</type><process_name>/usr/bin/{}</process_name></process_info>'
    def test_clients_and_invalid_telemetry(self):
        xml = self.XML.format(self.ENTRY.format("G", "niri") + self.ENTRY.format("C+G", "blender") + self.ENTRY.format("C", "python3"))
        self.assertEqual(cm.parse_clients(xml), {"compute": ["blender", "python3"], "graphics": ["niri"]})
        with self.assertRaises(cm.CannotRun): cm.parse_clients("<broken")
        for text in ("nan", "inf", "N/A", ""):
            with self.assertRaises(cm.CannotRun): cm.parse_number(text, "utilization.gpu")
        good = {"util_pct": 0.0, "power_w": 18.4, "clock_mhz": 360.0, "pstate": "P8"}
        for bad in ({"util_pct": 101.0}, {"power_w": 0.0}, {"clock_mhz": -1.0}, {"pstate": "X8"}):
            with self.assertRaises(cm.CannotRun): cm.validate_telemetry({**good, **bad})

    def test_missing_or_unsupported_inventory_cannot_run(self):
        for body in ('<nvidia_smi_log><gpu></gpu></nvidia_smi_log>', self.XML.format("N/A"),
                     '<nvidia_smi_log><gpu><processes/></gpu><gpu><processes/></gpu></nvidia_smi_log>',
                     self.XML.format('<process_info><type>X</type><process_name>a</process_name></process_info>')):
            with self.subTest(body=body), self.assertRaises(cm.CannotRun): cm.parse_clients(body)
        self.assertEqual(cm.parse_clients(self.XML.format("")), {"compute": [], "graphics": []})

    def test_malformed_gpu_sample_and_cpu_memory_cannot_run(self):
        malformed = [{"util_pct": 0.0}, {**QUIET, "extra": 1}]
        for query in malformed:
            with self.assertRaises(cm.CannotRun):
                cm.sample_stream(FakeProc([(0, 0), (1, 100)]), FakeGpu([query]), 1, sleep=lambda s: None)
        for clients in ({"compute": [], "graphics": None}, {"compute": [], "graphics": [None]}):
            with self.assertRaises(cm.CannotRun):
                cm.sample_stream(FakeProc([(0, 0), (1, 100)]), FakeGpu([QUIET], clients=clients), 1, sleep=lambda s: None)
        for ticks in ([(1, 100), (0, 90)], [(0, 100), (100, 150)]):
            with self.assertRaises(cm.CannotRun):
                cm.sample_stream(FakeProc(ticks), FakeGpu([QUIET]), 1, sleep=lambda s: None)
        with self.assertRaises(cm.CannotRun):
            cm.sample_stream(FakeProc([(0, 0), (1, 100)], total_kib=-1), FakeGpu([QUIET]), 1, sleep=lambda s: None)


class JudgementTests(unittest.TestCase):
    def test_quiet_and_settle(self):
        samples = quiet_samples()
        self.assertEqual(cm.judge_quiet(cm.summarize(samples), samples, cm.DEFAULT_THRESHOLDS, "headless"), [])
        samples = quiet_samples(3, power_w=24.1)
        reasons = cm.judge_settled(cm.summarize(samples), samples, cm.summarize(quiet_samples()), cm.DEFAULT_THRESHOLDS, "headless")
        self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", " ".join(reasons))
        dropped = quiet_samples(3)
        for sample in dropped: sample.mem_available_kib = 70_000
        self.assertIn("mem_available_pct", " ".join(cm.judge_settled(cm.summarize(dropped), dropped, cm.summarize(quiet_samples()), cm.DEFAULT_THRESHOLDS, "headless")))

    def test_each_quiet_threshold_and_lane_client_refuses(self):
        for key, value, needle in (("util_pct", 35.0, "gpu_util_pct"), ("pstate", "P0", "gpu_pstate")):
            self.assertIn(needle, " ".join(cm.judge_quiet(cm.summarize(quiet_samples(5, **{key: value})), quiet_samples(5, **{key: value}), cm.DEFAULT_THRESHOLDS, "headless")))
        for kwargs, needle in (({"load1": 4.5}, "load1"), ({"available_kib": 10_000}, "mem_available_pct")):
            samples = quiet_samples(5)
            for sample in samples: setattr(sample, "load1" if "load1" in kwargs else "mem_available_kib", kwargs.get("load1", kwargs.get("available_kib")))
            self.assertIn(needle, " ".join(cm.judge_quiet(cm.summarize(samples), samples, cm.DEFAULT_THRESHOLDS, "headless")))
        proc = FakeProc([(i * 2, i * 100) for i in range(6)])
        gpu = FakeGpu([dict(QUIET, power_w=value) for value in (18.0, 19.0, 30.0, 30.0, 19.0)])
        samples = cm.sample_stream(proc, gpu, 5, sleep=lambda s: None)
        self.assertIn("gpu_power_iqr_w", " ".join(cm.judge_quiet(cm.summarize(samples), samples, cm.DEFAULT_THRESHOLDS, "headless")))
        compute = quiet_samples(5)
        for sample in compute: sample.gpu_clients = {"compute": ["python3"], "graphics": ["niri"]}
        self.assertIn("compute", " ".join(cm.judge_quiet(cm.summarize(compute), compute, cm.DEFAULT_THRESHOLDS, "headless")))
        self.assertEqual(cm.judge_quiet(cm.summarize(quiet_samples()), quiet_samples(), cm.DEFAULT_THRESHOLDS, "headless"), [])
        self.assertIn("gpu_clients", " ".join(cm.judge_quiet(cm.summarize(quiet_samples()), quiet_samples(), cm.DEFAULT_THRESHOLDS, "dedicated")))
    def test_threshold_validation(self):
        self.assertEqual(cm.parse_thresholds(["cpu_busy_pct=20", "gpu_pstate=P5"])["cpu_busy_pct"], 20.0)
        self.assertEqual(cm.parse_thresholds(["cpu_busy_pct=101", "gpu_power_iqr_w=2000"])["cpu_busy_pct"], 101.0)
        for pair in ("nonsense=1", "load1=nan", "load1=-1", "gpu_pstate=X8"):
            with self.assertRaises(cm.CannotRun): cm.parse_thresholds([pair])


class PreflightTests(unittest.TestCase):
    def run_preflight(self, run, lane="headless", gpu=None, proc=None, env=None, lock=None, **kw):
        proc = proc or FakeProc([(i * 2, i * 100) for i in range(30)])
        gpu = gpu or FakeGpu([QUIET] * 30)
        saved = dict(os.environ)
        os.environ.update(env or {})
        try:
            return cm.preflight(run, lane=lane, task="material-x", fixture="f.sh", seconds=3, owner_pid=os.getpid(),
                                thresholds=cm.DEFAULT_THRESHOLDS, tools=kw.get("tools", ["tracy=0.13.1"]),
                                proc=proc, gpu=gpu, sleep=lambda s: None, lock=lock or (lambda: run / "lock"))
        finally:
            os.environ.clear(); os.environ.update(saved)

    def test_quiet_headless_writes_run_environment_baseline_preflight(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "trace-20260911T000000"; run.mkdir()
            self.run_preflight(run, env={"XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "wayland-1"})
            record = cm.load_record(run)
            self.assertEqual(record["run"]["id"], "trace-20260911T000000")
            self.assertEqual(record["run"]["lane"], "headless")
            self.assertEqual(record["run"]["task"], "material-x")
            self.assertRegex(record["run"]["started"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$")
            self.assertNotIn("/", record["run"]["host"])
            self.assertEqual(record["environment"]["gpu"]["name"], "NVIDIA test")
            self.assertEqual(record["environment"]["tools"]["tracy"], "0.13.1")
            self.assertEqual(record["environment"]["session"], {"type": "wayland", "display": "wayland-1"})
            self.assertEqual(record["baseline"]["samples"], 3)
            self.assertEqual(record["preflight"]["verdict"], "quiet")
            self.assertEqual(record["preflight"]["thresholds"], cm.DEFAULT_THRESHOLDS)
            self.assertEqual(record["preflight"]["lock"]["owner_pid"], os.getpid())
            self.assertTrue((run / "lock").exists())

    def test_busy_machine_refuses_and_records_reasons_and_clients(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            gpu = FakeGpu([dict(QUIET, util_pct=35.0)] * 5, clients={"compute": [], "graphics": ["niri", "BitwigStudio"]})
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, gpu=gpu)
            record = cm.load_record(run)
            self.assertEqual(record["preflight"]["verdict"], "refused")
            self.assertIn("gpu_util_pct 35.0 exceeds 5.0", record["preflight"]["reasons"])
            self.assertIn("BitwigStudio", record["baseline"]["gpu_clients"]["graphics"])
            self.assertFalse((run / "lock").exists())

    def test_dedicated_requires_tty_and_no_clients(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, lane="dedicated", env={"XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "w"})
            self.assertIn("XDG_SESSION_TYPE", " ".join(cm.load_record(run)["preflight"]["reasons"]))
            run2 = pathlib.Path(directory) / "r2"; run2.mkdir()
            with self.assertRaises(cm.Refused):
                self.run_preflight(run2, lane="dedicated", env={"XDG_SESSION_TYPE": "tty"},
                                   gpu=FakeGpu([QUIET] * 5, clients={"compute": [], "graphics": ["niri"]}))
            run3 = pathlib.Path(directory) / "r3"; run3.mkdir()
            saved = {k: os.environ.pop(k, None) for k in ("DISPLAY", "WAYLAND_DISPLAY")}
            try:
                self.run_preflight(run3, lane="dedicated", env={"XDG_SESSION_TYPE": "tty"},
                                   gpu=FakeGpu([QUIET] * 5, clients={"compute": [], "graphics": []}))
            finally:
                os.environ.update({k: v for k, v in saved.items() if v is not None})
            self.assertEqual(cm.load_record(run3)["preflight"]["verdict"], "quiet")

    def test_held_lock_refuses_without_sampling(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            lock = run / "lock"
            cm.acquire_lock(lock, os.getpid(), "other-run")
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, lock=lambda: lock)
            record = cm.load_record(run)
            self.assertEqual(record["preflight"]["verdict"], "refused")
            self.assertIn("other-run", " ".join(record["preflight"]["reasons"]))
            self.assertNotIn("baseline", record)
            self.assertEqual(cm.read_lock(lock)["run_id"], "other-run")

    def test_invalid_seconds_or_pid_does_not_mutate_run(self):
        with tempfile.TemporaryDirectory() as directory:
            for seconds, owner_pid in ((0, 1), (-1, 1), (1, 0), (1, -1), (1, True)):
                run = pathlib.Path(directory) / f"r-{seconds}-{owner_pid}"; run.mkdir()
                with self.subTest(seconds=seconds, owner_pid=owner_pid), self.assertRaises(cm.CannotRun):
                    cm.preflight(run, "headless", "t", "f", seconds, owner_pid, cm.DEFAULT_THRESHOLDS,
                                 [], FakeProc([(0, 0), (1, 100)]), FakeGpu([QUIET]), sleep=lambda _: None,
                                 lock=lambda: run / "lock")
                self.assertEqual(cm.load_record(run), {})

    def test_missing_tool_cannot_run_and_releases_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory); missing = "capture-meta-tool-that-does-not-exist"
            with self.assertRaises(cm.CannotRun):
                self.run_preflight(run, tools=[missing])
            self.assertFalse((run / "lock").exists())

    def test_cli_validation_returns_two_without_mutating_run(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "run"
            code = cm.main(["preflight", str(run), "--lane", "headless", "--task", "t", "--fixture", "f",
                            "--seconds", "0", "--owner-pid", "1"])
            self.assertEqual(code, 2)
            self.assertFalse(run.exists())


class SettleTests(unittest.TestCase):
    def prepared(self, directory):
        run = pathlib.Path(directory) / "r"; run.mkdir()
        lock = run / "lock"
        PreflightTests("test_quiet_headless_writes_run_environment_baseline_preflight").run_preflight(
            run, lock=lambda: lock)
        (run / "A.kdl").write_text("glass")
        return run, lock

    def test_settled_entry_carries_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            cm.settle(run, "A-move-1", [run / "A.kdl"], 3,
                      FakeProc([(i * 2, i * 100) for i in range(9)]),
                      FakeGpu([dict(QUIET, power_w=19.0)] * 3),
                      sleep=lambda s: None, lock=lambda: lock)
            entry = cm.load_record(run)["sub_runs"][0]
            self.assertEqual(entry["name"], "A-move-1")
            self.assertEqual(entry["verdict"], "settled")
            self.assertEqual(entry["inputs"][0]["name"], "A.kdl")
            self.assertEqual(entry["gpu_power_w"], 19.0)
            self.assertRegex(entry["settled_at"], r"^\d{4}-")

    def test_off_baseline_appends_refused_and_raises(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            with self.assertRaises(cm.Refused):
                cm.settle(run, "B-move-1", [run / "A.kdl"], 3,
                          FakeProc([(i * 2, i * 100) for i in range(9)]),
                          FakeGpu([dict(QUIET, power_w=24.1)] * 3),
                          sleep=lambda s: None, lock=lambda: lock)
            entry = cm.load_record(run)["sub_runs"][-1]
            self.assertEqual(entry["verdict"], "refused")
            self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", entry["reason"])

    def test_missing_input_and_foreign_lock_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            args = (3, FakeProc([(i * 2, i * 100) for i in range(9)]), FakeGpu([QUIET] * 3))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "x", [run / "missing.kdl"], *args, sleep=lambda s: None, lock=lambda: lock)
            lock.write_text(json.dumps({"owner_pid": 1, "run_id": "someone-else"}))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "y", [run / "A.kdl"], *args, sleep=lambda s: None, lock=lambda: lock)

    def test_same_run_id_with_wrong_owner_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            lock.write_text(json.dumps({"owner_pid": os.getpid() + 1, "run_id": run.name}))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "x", [run / "A.kdl"], 1, FakeProc([(0, 0), (1, 100)]),
                          FakeGpu([QUIET]), sleep=lambda s: None, lock=lambda: lock)

    def test_invalid_seconds_does_not_mutate_or_sample(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            before = (run / cm.RECORD).read_text()
            for seconds in (0, -1, True):
                with self.subTest(seconds=seconds), self.assertRaises(cm.CannotRun):
                    cm.settle(run, "x", [run / "A.kdl"], seconds, None, None,
                              sleep=lambda s: None, lock=lambda: lock)
                self.assertEqual((run / cm.RECORD).read_text(), before)

    def test_release_command_is_ownership_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = str(run)
            try:
                lock.rename(run / cm.LOCK_NAME)
                self.assertEqual(cm.main(["release", str(run)]), 0)
                self.assertFalse((run / cm.LOCK_NAME).exists())
                cm.acquire_lock(run / cm.LOCK_NAME, 1, "someone-else")
                self.assertEqual(cm.main(["release", str(run)]), 1)
                self.assertTrue((run / cm.LOCK_NAME).exists())
            finally:
                if saved is None: os.environ.pop("XDG_RUNTIME_DIR", None)
                else: os.environ["XDG_RUNTIME_DIR"] = saved

    def test_refused_before_acquire_does_not_release_foreign_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            cm.write_section(run, "run", {"id": run.name})
            cm.write_section(run, "preflight", {"verdict": "refused", "reasons": ["held"]})
            saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = directory
            try:
                foreign = pathlib.Path(directory) / cm.LOCK_NAME
                cm.acquire_lock(foreign, 1, "someone-else")
                self.assertEqual(cm.main(["release", str(run)]), 1)
                self.assertEqual(cm.read_lock(foreign)["run_id"], "someone-else")
            finally:
                if saved is None: os.environ.pop("XDG_RUNTIME_DIR", None)
                else: os.environ["XDG_RUNTIME_DIR"] = saved


class EndToEndTest(unittest.TestCase):
    def test_real_binary_against_fake_nvidia_smi(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory); bins = root / "bin"; bins.mkdir()
            fake = bins / "nvidia-smi"
            fake.write_text(textwrap.dedent("""\
                #!/bin/sh
                case "$*" in
                  *utilization.gpu*) echo "0, 18.40, 360, P8" ;;
                  *name,uuid*) echo "NVIDIA test, GPU-1, 610.57.04" ;;
                  *-q*-x*) echo '<nvidia_smi_log><gpu><processes><process_info><type>G</type><process_name>/usr/bin/niri</process_name></process_info></processes></gpu></nvidia_smi_log>' ;;
                  *) exit 9 ;;
                esac
                """)); fake.chmod(0o755)
            run = root / "material-x" / "headless-20260911T000000"; run.mkdir(parents=True)
            runtime = root / "rt"; runtime.mkdir()
            proc = root / "proc"; proc.mkdir()
            (proc / "stat").write_text("cpu  100 0 50 9000 10 0 0 0 0 0\n")
            (proc / "loadavg").write_text("0.42 0.40 0.39 1/900 1\n")
            (proc / "meminfo").write_text("MemTotal:       100000 kB\nMemFree:         50000 kB\nMemAvailable:    80000 kB\n")
            src = root / "src"; src.mkdir()
            (root / "A.kdl").write_text("glass")
            subprocess.run(["git", "-C", str(src), "init", "-q", "-b", "main"], check=True)
            subprocess.run(["git", "-C", str(src), "-c", "user.name=t", "-c", "user.email=t@t",
                            "commit", "-q", "--allow-empty", "-m", "i"], check=True)
            env = {**os.environ, "PATH": f"{bins}:{os.environ['PATH']}", "XDG_RUNTIME_DIR": str(runtime),
                   "CAPTURE_META_PROC": str(proc), "XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "wayland-1"}
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            ticks = 0
            def cm_run(*args, sampling=False):
                nonlocal ticks
                process = subprocess.Popen([sys.executable, tool, *args], env=env,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                if sampling:
                    ticks += 100
                    threading.Timer(.2, lambda: (proc / "stat").write_text(
                        f"cpu  100 0 50 {9010 + ticks} 0 0 0 0 0 0\n")).start()
                stdout, stderr = process.communicate()
                return process.returncode, stdout, stderr
            self.assertEqual(cm_run("preflight", str(run), "--lane", "headless", "--task", "material-x",
                                    "--fixture", "t.sh", "--seconds", "1", "--tool", "tracy=0.13.1",
                                    sampling=True)[0], 0)
            self.assertEqual(cm_run("identity", str(run), "--source", str(src), "--input", str(root / "A.kdl"))[0], 0)
            self.assertEqual(cm_run("settle", str(run), "--sub-run", "A-1", "--input", str(root / "A.kdl"),
                                    "--seconds", "1", sampling=True)[0], 0)
            self.assertEqual(cm_run("release", str(run))[0], 0)
            code, stdout, stderr = cm_run("show", str(run))
            self.assertEqual(code, 0, stderr)
            self.assertIn("A-1: settled", stdout)
            record = json.loads((run / "capture.json").read_text())
            self.assertFalse(record["provenance"]["source"]["dirty"])
            self.assertEqual(record["environment"]["gpu"]["driver"], "610.57.04")
            self.assertEqual(record["baseline"]["gpu_clients"]["graphics"], ["niri"])
            self.assertEqual(record["baseline"]["load1"], 0.42)
            self.assertEqual(record["baseline"]["mem_available_pct"], 80.0)
            self.assertEqual(record["baseline"]["cpu_busy_pct"], 0.0)
            self.assertFalse((runtime / cm.LOCK_NAME).exists())


if __name__ == "__main__":
    unittest.main()
