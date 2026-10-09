"""Unit tests for tools/capture-meta."""
import contextlib
import importlib.machinery
import importlib.util
import json
import io
import itertools
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

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fake_capture_host import FakeHost  # noqa: E402


class _NoRealHost:
    def __init__(self, *args, **kwargs):
        raise AssertionError("a test reached the real host: pass a FakeHost")


cm.Host = _NoRealHost



class LockTests(unittest.TestCase):
    def test_acquire_release_and_ownership_check(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            info = cm.acquire_lock(path, owner_pid=111, run_id="run-a", alive=lambda pid: True, run_dir=path.parent)
            self.assertEqual(info, {"path": str(path), "owner_pid": 111, "run_id": "run-a", "run_dir": str(path.parent), "reclaimed": False})
            self.assertEqual(sorted(p.name for p in path.parent.iterdir()), ["capture-meta.lock", "capture-meta.lock.d"])
            self.assertEqual(cm.read_lock(path)["run_id"], "run-a")
            self.assertFalse(cm.release_lock(path, owner_pid=222, run_id="run-b", run_dir=path.parent))
            self.assertTrue(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a", run_dir=path.parent))
            self.assertFalse(path.exists())
            self.assertTrue(cm.release_lock(path, owner_pid=111, run_id="run-a", run_dir=path.parent))

    def test_half_written_or_corrupt_lock_is_never_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            path.write_text("")
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False, run_dir=path.parent)
            self.assertEqual(path.read_text(), "")
            path.write_text("{not json")
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False, run_dir=path.parent)
            self.assertFalse(cm.release_lock(path, 222, "run-b", run_dir=path.parent))

    def test_binary_corrupt_lock_is_never_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            corrupt = b"\xff"
            path.write_bytes(corrupt)
            with self.assertRaises(cm.CannotRun):
                cm.read_lock(path)
            with self.assertRaises(cm.CannotRun):
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: False, run_dir=path.parent)
            self.assertFalse(cm.release_lock(path, 222, "run-b", run_dir=path.parent))
            self.assertEqual(path.read_bytes(), corrupt)

    def test_guard_serializes_acquire_against_a_concurrent_holder(self):
        import fcntl
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            guard = path.with_name(path.name + ".d"); guard.mkdir()
            fd = os.open(guard, os.O_RDONLY); fcntl.flock(fd, fcntl.LOCK_EX)
            done = threading.Event(); result = {}
            def worker():
                result["info"] = cm.acquire_lock(path, 333, "run-c", alive=lambda pid: True, run_dir=path.parent); done.set()
            threading.Thread(target=worker, daemon=True).start()
            self.assertFalse(done.wait(0.3))
            fcntl.flock(fd, fcntl.LOCK_UN); os.close(fd)
            self.assertTrue(done.wait(3.0))
            self.assertEqual(result["info"]["run_id"], "run-c")

    def test_held_by_live_pid_refuses_stale_is_reclaimed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            cm.acquire_lock(path, 111, "run-a", alive=lambda pid: True, run_dir=path.parent)
            with self.assertRaises(cm.Refused) as ctx:
                cm.acquire_lock(path, 222, "run-b", alive=lambda pid: True, run_dir=path.parent)
            self.assertIn("run-a", str(ctx.exception))
            info = cm.acquire_lock(path, 222, "run-b", alive=lambda pid: pid != 111, run_dir=path.parent)
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
                    "try:\n    cm.acquire_lock(pathlib.Path(sys.argv[2]), int(sys.argv[3]), sys.argv[3], alive=lambda p: True, run_dir=pathlib.Path(sys.argv[2]).parent)\n"
                    "except cm.Refused:\n    sys.exit(1)\n")
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            procs = [subprocess.Popen([sys.executable, "-c", code, tool, str(path), str(1000 + i)]) for i in range(8)]
            codes = sorted(p.wait() for p in procs)
            self.assertEqual(codes, [0] + [1] * 7)

    def test_public_read_waits_for_guard(self):
        import fcntl
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "capture-meta.lock"
            path.write_text(json.dumps({"owner_pid": 111, "run_id": "run-a", "run_dir": str(path.parent)}))
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
                    cm.acquire_lock(path, pid, run_id, run_dir=path.parent)
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
            "show host_load": ({"schema": 1, "preflight": {"verdict": "refused", "host_load": {"load": {}}}}, ["show"]),
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
                  "sub_runs": [{"name": "gpu-plain-1", "verdict": "settled", "started": "2026-09-11T04:35:00-04:00",
                                "finished": "2026-09-11T04:36:00-04:00", "duration_s": 60},
                               {"name": "gpu-plain-2", "verdict": "settled", "started": "2026-09-11T04:37:00-04:00"},
                               {"name": "gpu-aurora-1", "verdict": "refused", "reason": "gpu_power_w 24.1 exceeds baseline"}]}
        text = cm.render(record)
        record["run"].update(finished="2026-09-11T04:40:00-04:00", duration_s=327, finished_by="guard")
        record["preflight"]["at"] = "2026-09-11T04:34:50-04:00"
        finished = cm.render(record)
        self.assertIn("finished 2026-09-11T04:40:00-04:00 (327 s, by guard)", finished)
        self.assertIn("preflight quiet at 2026-09-11T04:34:50-04:00", finished)
        for needle in ("trace-1", "RTX", "610", "abc", "niri 00", "not finished",
                       "gpu-plain-1: settled  2026-09-11T04:35:00-04:00 to 2026-09-11T04:36:00-04:00 (60 s)",
                       "gpu-plain-2: settled  2026-09-11T04:37:00-04:00, never finished",
                       "gpu-aurora-1: refused (gpu_power_w 24.1 exceeds baseline)", "preset=aurora"):
            self.assertIn(needle, text)

    def test_main_show_exit_codes(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory)
            self.assertEqual(cm.main(["show", directory]), 2)
            cm.write_section(run, "run", {"id": "x", "task": "t", "fixture": "f", "lane": "headless",
                                          "started": "now", "host": "h"})
            self.assertEqual(cm.main(["show", directory]), 0)

    def test_show_marks_unchecked_measured_launches(self):
        record = {"schema": 1, "run": {"id": "drm", "lane": "dedicated", "started": STARTED},
                  "sub_runs": [{"name": "drm-a", "verdict": "settled", "started": STARTED},
                               {"name": "drm-b", "verdict": "settled", "started": STARTED,
                                "renderer": {"verdict": "verified"}},
                               {"name": "drm-c", "verdict": "refused", "reason": "gpu busy"}]}
        lines = cm.render(record).splitlines()
        line = lambda name: next(l for l in lines if l.strip().startswith(f"{name}:"))
        self.assertTrue(line("drm-a").endswith("renderer unchecked"), line("drm-a"))
        self.assertTrue(line("drm-b").endswith("renderer verified"), line("drm-b"))
        self.assertNotIn("renderer", line("drm-c"))
        pixel = {"schema": 1, "run": {"id": "px", "lane": "pixels", "started": STARTED},
                 "sub_runs": [{"name": "cell-1", "verdict": "begun", "started": STARTED}]}
        self.assertNotIn("renderer", cm.render(pixel))

    def test_show_renders_old_and_pixel_records(self):
        old = {"schema": 1, "run": {"id": "old", "lane": "headless", "started": STARTED},
               "preflight": {"verdict": "quiet"},
               "sub_runs": [{"name": "A", "verdict": "settled", "started": STARTED}]}
        self.assertIn("A: settled", cm.render(old))
        pixel = {"schema": 1, "run": {"id": "px", "lane": "pixels", "started": STARTED},
                 "environment": {"kernel": "k", "cpu": "c", "cpu_threads": 1, "memory_total_kib": 1,
                                 "session": {"type": "wayland", "display": "w"}, "tools": {}},
                 "preflight": {"verdict": "unsampled", "at": STARTED, "host_condition": "desktop"},
                 "sub_runs": [{"name": "cell-1", "verdict": "begun", "started": STARTED}]}
        cm.validate_record(pixel)
        text = cm.render(pixel)
        for needle in ("GPU not sampled: pixels lane", "preflight unsampled", "host desktop",
                       "cell-1: begun", "never finished"):
            self.assertIn(needle, text)


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


class FakeHostLoad:
    def __init__(self, top=None):
        self.calls = 0
        self._top = top if top is not None else [
            {"pid": 4242, "comm": "bun", "cmd": "bun test", "cpu_pct": 97.0, "age_s": 3600,
             "tty": False, "unit": "kitty-1-0.scope", "scope_dead": True},
            {"pid": 1700, "comm": "niri", "cmd": "niri --session", "cpu_pct": 4.0, "age_s": 7200,
             "tty": False, "unit": "niri.service", "scope_dead": False}]
    def report(self):
        self.calls += 1
        return {"host": "h", "at": "2026-09-27T12:00:00+00:00",
                "load": {"load1": 6.65, "cpus": 32, "interval_s": 1.0, "top": [dict(p) for p in self._top]}}


# Every run section preflight writes carries its start; a release stamps the finish against it.
STARTED = "2026-10-06T10:00:00-04:00"
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


class HostLoadReaderTests(unittest.TestCase):
    def reader(self, script):
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        fake = pathlib.Path(directory.name) / "host-load"
        fake.write_text("#!/bin/sh\n" + script); fake.chmod(0o755)
        with mock.patch.dict(os.environ, {"PATH": directory.name}):
            reader = cm.HostLoadReader()
        return reader, directory.name

    def test_absent_tool_cannot_run_with_install_hint(self):
        with tempfile.TemporaryDirectory() as empty, mock.patch.dict(os.environ, {"PATH": empty}):
            with self.assertRaises(cm.CannotRun) as failed:
                cm.HostLoadReader()
        self.assertIn("host-load not found on PATH", str(failed.exception))
        self.assertIn("just install", str(failed.exception))

    def test_report_runs_the_load_section_and_drops_its_own_process(self):
        reader, bindir = self.reader(textwrap.dedent("""\
            [ "$*" = "--json --section load -n 5" ] || exit 7
            printf '{"host":"h","at":"t","load":{"load1":6.6,"top":[{"pid":%s,"comm":"python3"},{"pid":1,"comm":"bun"}]}}' $$
            """))
        with mock.patch.dict(os.environ, {"PATH": bindir}):
            report = reader.report()
        self.assertEqual([p["comm"] for p in report["load"]["top"]], ["bun"])

    def test_failed_or_malformed_report_cannot_run(self):
        for script, expected in (("echo boom >&2; exit 3\n", "boom"), ("echo not json\n", "not JSON"),
                                 ("echo '{\"load\": {}}'\n", "no load.top list")):
            reader, bindir = self.reader(script)
            with self.subTest(expected=expected), mock.patch.dict(os.environ, {"PATH": bindir}):
                with self.assertRaises(cm.CannotRun) as failed:
                    reader.report()
                self.assertIn(expected, str(failed.exception))


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
            self.assertRegex(record["preflight"]["at"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$")
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

    def test_gpu_only_refusal_does_not_ask_who_loads_the_cpu(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            host_load = FakeHostLoad()
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, gpu=FakeGpu([dict(QUIET, util_pct=35.0)] * 5), host_load=host_load)
            self.assertEqual(host_load.calls, 0)
            self.assertNotIn("host_load", cm.load_record(run)["preflight"])

    def test_cpu_or_load_refusal_records_and_names_the_load(self):
        cases = {"load1": {"proc": FakeProc([(i * 2, i * 100) for i in range(30)], load1=6.65)},
                 "cpu_busy_pct": {"proc": FakeProc([(i * 50, i * 100) for i in range(30)])}}
        for key, kw in cases.items():
            with tempfile.TemporaryDirectory() as directory, self.subTest(refused_on=key):
                run = pathlib.Path(directory) / "r"; run.mkdir()
                host_load = FakeHostLoad()
                with self.assertRaises(cm.Refused) as refused:
                    self.run_preflight(run, host_load=host_load, **kw)
                self.assertEqual(host_load.calls, 1)
                message = str(refused.exception)
                self.assertIn(f"{key} ", message)
                self.assertIn("bun pid 4242 97.0% cpu, 3600 s old, no tty, scope dead", message)
                self.assertIn("niri pid 1700", message)
                preflight = cm.load_record(run)["preflight"]
                self.assertEqual(preflight["verdict"], "refused")
                self.assertEqual(preflight["host_load"]["load"]["top"][0]["comm"], "bun")
                self.assertIn("bun pid 4242", cm.render(cm.load_record(run)))
                self.assertFalse((run / "lock").exists())

    def test_quiet_preflight_does_not_run_host_load(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            host_load = FakeHostLoad()
            self.run_preflight(run, host_load=host_load)
            self.assertEqual(host_load.calls, 0)
            self.assertNotIn("host_load", cm.load_record(run)["preflight"])

    def test_host_load_failure_cannot_run_names_the_refusal_and_releases_lock(self):
        class Broken:
            def report(self): raise cm.CannotRun("host-load --json: exit 3")
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            with self.assertRaises(cm.CannotRun) as failed:
                self.run_preflight(run, proc=FakeProc([(i * 2, i * 100) for i in range(30)], load1=6.65),
                                   host_load=Broken())
            self.assertIn("load1 6.65 exceeds 2.0", str(failed.exception))
            self.assertIn("exit 3", str(failed.exception))
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
            cm.acquire_lock(lock, os.getpid(), "other-run", run_dir=run)
            with self.assertRaises(cm.Refused):
                self.run_preflight(run, lock=lambda: lock)
            record = cm.load_record(run)
            self.assertEqual(record["preflight"]["verdict"], "refused")
            self.assertRegex(record["preflight"]["at"], r"^\d{4}-")
            self.assertIn("other-run", " ".join(record["preflight"]["reasons"]))
            self.assertNotIn("baseline", record)
            self.assertEqual(cm.read_lock(lock)["run_id"], "other-run")

    def test_invalid_seconds_or_pid_does_not_mutate_run(self):
        with tempfile.TemporaryDirectory() as directory:
            for seconds, owner_pid in ((0, 1), (-1, 1), (1, 0), (1, -1), (1, True)):
                run = pathlib.Path(directory) / f"r-{seconds}-{owner_pid}"; run.mkdir()
                with self.subTest(seconds=seconds, owner_pid=owner_pid), self.assertRaises(cm.CannotRun):
                    cm.preflight(run, "headless", "t", "f", seconds, owner_pid, cm.DEFAULT_THRESHOLDS,
                                 [], FakeProc([(0, 0), (1, 100)]), FakeGpu([QUIET]), FakeHostLoad(), host=FakeHost(pathlib.Path(directory) / "host"),
                                 sleep=lambda _: None,
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
            with mock.patch.object(cm, "Host", return_value=FakeHost(pathlib.Path(directory) / "host")):
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
            self.assertLessEqual(entry["started"], entry["settled_at"])
            self.assertNotIn("finished", entry)

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
            lock.write_text(json.dumps({"owner_pid": 1, "run_id": "someone-else", "run_dir": str(run.resolve())}))
            with self.assertRaises(cm.Refused):
                cm.settle(run, "y", [run / "A.kdl"], *args, sleep=lambda s: None, lock=lambda: lock)

    def test_same_run_id_with_wrong_owner_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            lock.write_text(json.dumps({"owner_pid": os.getpid() + 1, "run_id": run.name, "run_dir": str(run.resolve())}))
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

    def settled(self, run, lock, name, power_w=19.0, clock=("2026-10-06T10:00:00-04:00",
                                                            "2026-10-06T10:00:10-04:00")):
        with mock.patch.object(cm, "now_rfc3339", side_effect=clock):
            cm.settle(run, name, [run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                      FakeGpu([dict(QUIET, power_w=power_w)] * 3), sleep=lambda s: None, lock=lambda: lock)

    def test_finish_stamps_the_sub_runs_end_and_duration(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            self.settled(run, lock, "A")
            with mock.patch.object(cm, "now_rfc3339", return_value="2026-10-06T10:02:05-04:00"):
                self.assertEqual(cm.main(["finish", str(run), "--sub-run", "A"]), 0)
            entry = cm.load_record(run)["sub_runs"][0]
            self.assertEqual((entry["started"], entry["settled_at"], entry["finished"], entry["duration_s"]),
                             ("2026-10-06T10:00:00-04:00", "2026-10-06T10:00:10-04:00",
                              "2026-10-06T10:02:05-04:00", 125))

    def test_finish_takes_the_latest_settle_of_a_repeated_name(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            self.settled(run, lock, "A")
            cm.finish_sub_run(run, "A")
            self.settled(run, lock, "A", clock=("2026-10-06T11:00:00-04:00", "2026-10-06T11:00:10-04:00"))
            with mock.patch.object(cm, "now_rfc3339", return_value="2026-10-06T11:00:30-04:00"):
                cm.finish_sub_run(run, "A")
            first, second = cm.load_record(run)["sub_runs"]
            self.assertEqual(second["duration_s"], 30)
            self.assertNotEqual(first["finished"], second["finished"])

    def test_finish_refuses_an_unknown_refused_or_finished_sub_run_without_writing(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            self.settled(run, lock, "A")
            cm.finish_sub_run(run, "A")
            with self.assertRaises(cm.Refused):
                self.settled(run, lock, "B", power_w=24.1)
            before = (run / cm.RECORD).read_text()
            for name, message in (("C", "no sub-run 'C'"), ("B", "was refused"), ("A", "already finished")):
                with self.subTest(name=name), self.assertRaisesRegex(cm.CannotRun, message):
                    cm.finish_sub_run(run, name)
                self.assertEqual((run / cm.RECORD).read_text(), before)
            self.assertEqual(cm.main(["finish", str(run), "--sub-run", "C"]), 2)

    def test_release_command_is_ownership_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            run, lock = self.prepared(directory)
            saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = str(run)
            try:
                host = FakeHost(pathlib.Path(directory) / "host")
                lock.rename(run / cm.LOCK_NAME)
                with mock.patch.object(cm, "Host", return_value=host):
                    self.assertEqual(cm.main(["release", str(run)]), 0)
                self.assertFalse((run / cm.LOCK_NAME).exists())
                cm.acquire_lock(run / cm.LOCK_NAME, 1, "someone-else", run_dir=run)
                with mock.patch.object(cm, "Host", return_value=host):
                    self.assertEqual(cm.main(["release", str(run)]), 0)  # repeats the finished verdict
                self.assertTrue((run / cm.LOCK_NAME).exists())
            finally:
                if saved is None: os.environ.pop("XDG_RUNTIME_DIR", None)
                else: os.environ["XDG_RUNTIME_DIR"] = saved

    def test_refused_before_acquire_does_not_release_foreign_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            run = pathlib.Path(directory) / "r"; run.mkdir()
            cm.write_section(run, "run", {"id": run.name, "started": STARTED})
            cm.write_section(run, "preflight", {"verdict": "refused", "reasons": ["held"]})
            saved = os.environ.get("XDG_RUNTIME_DIR"); os.environ["XDG_RUNTIME_DIR"] = directory
            try:
                foreign = pathlib.Path(directory) / cm.LOCK_NAME
                cm.acquire_lock(foreign, 1, "someone-else", run_dir=run)
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
            host_load = bins / "host-load"
            host_load.write_text('#!/bin/sh\necho \'{"host":"h","at":"t","load":{"load1":0.4,"top":[]}}\'\n')
            host_load.chmod(0o755)
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
                   "XDG_CONFIG_HOME": str(root / "cfg"), "CAPTURE_META_SYSFS": str(root / "drm"), "NIRI_SOCKET": "",
                   "CAPTURE_META_PROC": str(proc), "XDG_SESSION_TYPE": "wayland", "WAYLAND_DISPLAY": "wayland-1"}
            tool = str(pathlib.Path(__file__).with_name("capture-meta"))
            stat = proc / "stat"
            def serve_idle_ticks():
                # A one-second sample reads stat twice. The first read blocks on a FIFO; the advanced
                # counters replace it before that read is released, so the second read sees them
                # however late the tool starts. No wall-clock timing is involved.
                stat.unlink(); os.mkfifo(stat)
                def serve():
                    with open(stat, "w") as first:
                        staged = proc / "stat.next"
                        staged.write_text("cpu  100 0 50 9100 10 0 0 0 0 0\n"); staged.replace(stat)
                        first.write("cpu  100 0 50 9000 10 0 0 0 0 0\n")
                server = threading.Thread(target=serve); server.start()
                return server
            def cm_run(*args, sampling=False):
                server = serve_idle_ticks() if sampling else None
                process = subprocess.Popen([sys.executable, tool, *args], env=env,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                stdout, stderr = process.communicate()
                if server is not None and server.is_alive():
                    # The tool exited without reading stat: open the reading end so the server finishes.
                    reader = os.open(stat, os.O_RDONLY | os.O_NONBLOCK)
                    server.join(); os.close(reader)
                return process.returncode, stdout, stderr
            code, _, stderr = cm_run("preflight", str(run), "--lane", "headless", "--task", "material-x",
                                     "--fixture", "t.sh", "--seconds", "1", "--hold-settle", "0", "--tool", "tracy=0.13.1",
                                     sampling=True)
            self.assertEqual(code, 0, stderr)
            code, _, stderr = cm_run("identity", str(run), "--source", str(src), "--input", str(root / "A.kdl"))
            self.assertEqual(code, 0, stderr)
            code, _, stderr = cm_run("settle", str(run), "--sub-run", "A-1", "--input", str(root / "A.kdl"),
                                     "--seconds", "1", sampling=True)
            self.assertEqual(code, 0, stderr)
            gpu_name = "NVIDIA test"   # the fake nvidia-smi's --query-gpu=name answer
            (root / "niri.slice").write_text(f'INFO smithay::backend::renderer::gles: GL Renderer: "{gpu_name}/PCIe/SSE2"\n')
            (root / "weston.slice").write_text(f"[00:00:00.000] GL renderer: {gpu_name}/PCIe/SSE2\n")
            code, _, stderr = cm_run("renderer", str(run), "--sub-run", "A-1", "--niri-log", str(root / "niri.slice"),
                                     "--weston-log", str(root / "weston.slice"))
            self.assertEqual(code, 0, stderr)
            code, _, stderr = cm_run("finish", str(run), "--sub-run", "A-1")
            self.assertEqual(code, 0, stderr)
            code, _, stderr = cm_run("release", str(run))
            self.assertEqual(code, 0, stderr)
            record = json.loads((run / "capture.json").read_text())
            self.assertGreaterEqual(record["run"]["duration_s"], 1)
            self.assertLessEqual(record["preflight"]["at"], record["run"]["finished"])
            self.assertGreaterEqual(record["sub_runs"][0]["duration_s"], 1)
            self.assertEqual(record["hold"]["desktop"], "absent")
            self.assertEqual(record["hold_end"]["scan"]["verdict"], "clean")
            self.assertFalse((runtime / "capture-meta.hold.json").exists())
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
            host_load.unlink()
            bare = root / "material-x" / "no-host-load"
            result = subprocess.run([sys.executable, tool, "preflight", str(bare), "--lane", "headless",
                                     "--task", "material-x", "--fixture", "t.sh", "--seconds", "1"],
                                    env={**env, "PATH": str(bins)}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("host-load not found on PATH", result.stderr)
            self.assertFalse(bare.exists())


DATA = pathlib.Path(__file__).with_name("testdata") / "renderer"


class RendererTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        self.root = pathlib.Path(temp.name)

    def measured(self, lane="headless", gpu_name="NVIDIA GeForce RTX 3070", name=None):
        run = self.root / (name or f"{lane}-run"); run.mkdir()
        cm.write_section(run, "run", {"id": run.name, "lane": lane, "started": STARTED})
        cm.write_section(run, "environment", {"gpu": {"name": gpu_name}, "session": {}, "tools": {}})
        cm.append_sub_run(run, {"name": "A", "verdict": "settled", "started": STARTED})
        return run

    def weston(self, text=None):
        path = self.root / "weston.slice"
        path.write_text(text if text is not None else (DATA / "weston-nvidia.log").read_text())
        return path

    def test_both_compositors_on_the_sampled_gpu_verify(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        r = cm.load_record(run)["sub_runs"][-1]["renderer"]
        self.assertEqual((r["verdict"], r["niri"]["verdict"], r["weston"]["verdict"]),
                         ("verified", "verified", "verified"))
        self.assertEqual(r["niri"]["lines"], ["NVIDIA GeForce RTX 3070/PCIe/SSE2"])
        self.assertEqual(r["expected"], "NVIDIA GeForce RTX 3070")

    def test_default_filter_llvmpipe_and_other_gpu_are_recorded_then_refused(self):
        for i, (log, gpu, niri_verdict) in enumerate(((DATA / "niri-default.log", "NVIDIA GeForce RTX 3070", "missing"),
                                                      (DATA / "niri-llvmpipe.log", "NVIDIA GeForce RTX 3070", "mismatch"),
                                                      (DATA / "niri-nvidia.log", "NVIDIA GeForce RTX 4090", "mismatch"))):
            with self.subTest(log=log.name, gpu=gpu):
                run = self.measured(gpu_name=gpu, name=f"case-{i}")
                with self.assertRaises(cm.Refused):
                    cm.renderer(run, "A", log, self.weston())
                entry = cm.load_record(run)["sub_runs"][-1]
                self.assertNotIn("finished", entry)
                self.assertEqual(entry["renderer"]["niri"]["verdict"], niri_verdict)
                self.assertNotEqual(entry["renderer"]["verdict"], "verified")

    def test_weston_spelled_like_niri_is_missing(self):
        run = self.measured()
        with self.assertRaises(cm.Refused):
            cm.renderer(run, "A", DATA / "niri-nvidia.log",
                        self.weston("[21:51:39.332] GL Renderer: NVIDIA GeForce RTX 3070/PCIe/SSE2\n"))
        self.assertEqual(cm.load_record(run)["sub_runs"][-1]["renderer"]["weston"]["verdict"], "missing")

    def test_weston_log_is_required_nested_and_refused_on_drm(self):
        with self.assertRaisesRegex(cm.CannotRun, "--weston-log"):
            cm.renderer(self.measured(), "A", DATA / "niri-nvidia.log", None)
        with self.assertRaisesRegex(cm.CannotRun, "no Weston"):
            cm.renderer(self.measured("dedicated"), "A", DATA / "niri-nvidia.log", self.weston())
        drm = self.measured("dedicated", name="drm-ok")
        cm.renderer(drm, "A", DATA / "niri-nvidia.log", None)
        self.assertNotIn("weston", cm.load_record(drm)["sub_runs"][-1]["renderer"])

    def test_second_call_and_no_open_entry_cannot_run(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        with self.assertRaisesRegex(cm.CannotRun, "already"):
            cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        with self.assertRaisesRegex(cm.CannotRun, "no open sub-run"):
            cm.renderer(run, "B", DATA / "niri-nvidia.log", self.weston())

    def test_renderer_attaches_to_the_latest_open_entry(self):
        run = self.measured()
        cm.renderer(run, "A", DATA / "niri-nvidia.log", self.weston())
        cm.finish_sub_run(run, "A")
        cm.append_sub_run(run, {"name": "A", "verdict": "settled", "started": STARTED})
        with self.assertRaises(cm.Refused):
            cm.renderer(run, "A", DATA / "niri-llvmpipe.log", self.weston())
        first, second = cm.load_record(run)["sub_runs"]
        self.assertEqual((first["renderer"]["verdict"], second["renderer"]["niri"]["verdict"]),
                         ("verified", "mismatch"))

    def test_pixels_records_without_requiring(self):
        run = self.root / "px"; run.mkdir()
        cm.write_section(run, "run", {"id": "px", "lane": "pixels", "started": STARTED})
        cm.write_section(run, "environment", {"session": {}, "tools": {}})
        cm.append_sub_run(run, {"name": "A", "verdict": "begun", "started": STARTED})
        cm.renderer(run, "A", DATA / "niri-default.log", self.weston())
        r = cm.load_record(run)["sub_runs"][-1]["renderer"]
        self.assertEqual((r["verdict"], r["niri"]["verdict"], r["weston"]["verdict"]),
                         ("recorded", "missing", "recorded"))
        self.assertNotIn("expected", r)

    def test_unverified_launches_counts_settled_entries_finished_or_open(self):
        record = {"run": {"lane": "dedicated"}, "sub_runs": [
            {"name": "open", "verdict": "settled"},
            {"name": "done", "verdict": "settled", "finished": STARTED},
            {"name": "ok", "verdict": "settled", "renderer": {"verdict": "verified"}},
            {"name": "refused", "verdict": "refused"}]}
        self.assertEqual(cm.unverified_launches(record), ["open", "done"])
        record["run"]["lane"] = "pixels"
        self.assertEqual(cm.unverified_launches(record), [])


class HoldRecordTests(unittest.TestCase):
    def run_dir(self):
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        run = pathlib.Path(temp.name) / "r"; run.mkdir()
        cm.write_section(run, "run", {"id": "r", "started": STARTED})
        return run

    def test_scan_is_written_once_and_attempts_accumulate(self):
        run = self.run_dir()
        failed = {"at": "t1", "by": "release", "failures": [{"unit": "a.timer", "error": "x", "restore": "systemctl --user start a.timer"}]}
        end = cm.update_hold_end(run, {"until_us": 5, "verdict": "clean", "disturbances": []}, failed,
                                 {"guard": "capture-meta-guard-x.service", "owner_pid": 7})
        self.assertEqual(end["restore"]["state"], "failed")
        end = cm.update_hold_end(run, attempt={"at": "t1b", "by": "release", "failures": failed["failures"]},
                                 cleanup={"guard": "other", "owner_pid": 8})
        self.assertEqual(end["cleanup"], {"guard": "capture-meta-guard-x.service", "owner_pid": 7})
        with self.assertRaisesRegex(cm.CannotRun, "written once"):
            cm.update_hold_end(run, {"until_us": 9, "verdict": "clean", "disturbances": []})
        end = cm.update_hold_end(run, attempt={"at": "t2", "by": "guard", "failures": []})
        self.assertEqual(end["restore"]["state"], "complete")
        self.assertEqual([a["by"] for a in end["restore"]["attempts"]], ["release", "release", "guard"])
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

    def test_record_writes_wait_for_a_restore_writing_hold_end(self):
        run = self.run_dir()
        entered, go, done = threading.Event(), threading.Event(), threading.Event()
        def restoring():
            with cm.record_lock(run):
                entered.set(); go.wait(5)
                cm._update_hold_end(run, {"until_us": 1, "verdict": "clean", "disturbances": []},
                                    {"at": "t", "by": "guard", "undone": 1, "failures": []})
        worker = threading.Thread(target=restoring); worker.start()
        entered.wait(5)
        threading.Thread(target=lambda: (cm.write_section(run, "baseline", {"seconds": 3}), done.set())).start()
        self.assertFalse(done.wait(0.3))          # an orphaned preflight's write waits
        go.set(); worker.join(5)
        self.assertTrue(done.wait(5))
        record = cm.load_record(run)
        self.assertEqual(record["hold_end"]["restore"]["state"], "complete")
        self.assertEqual(record["baseline"], {"seconds": 3})

    def test_save_record_leaves_no_temporary_file(self):
        run = self.run_dir()
        cm.update_hold_end(run, attempt={"at": "t", "by": "release", "failures": []})
        self.assertEqual(sorted(p.name for p in run.iterdir()), [cm.RECORD])

    def test_validation_rejects_malformed_hold_sections(self):
        for bad in ({"hold": {"items": {}}},
                    {"hold_end": {"restore": {"state": "maybe", "attempts": []}}},
                    {"hold_end": {"scan": {"verdict": "fine", "disturbances": []}}},
                    {"hold_end": {"cleanup": {"guard": "g", "owner_pid": True}}}):
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

    def lane_preflight(self, lane, gpu="quiet", env=None):   # gpu=None: no sampler (pixels lane)
        saved = {k: os.environ.get(k) for k in ("XDG_SESSION_TYPE", "DISPLAY", "WAYLAND_DISPLAY")}
        for key in ("DISPLAY", "WAYLAND_DISPLAY"):
            os.environ.pop(key, None)
        os.environ.update(env or {"XDG_SESSION_TYPE": "tty"})
        try:
            return cm.preflight(self.run, lane=lane, task="material-18c2a1", fixture="t.sh", seconds=3,
                                owner_pid=4242, thresholds=cm.DEFAULT_THRESHOLDS, tools=["tracy=0.13.1"],
                                proc=FakeProc([(i * 2, i * 100) for i in range(30)]),
                                gpu=FakeGpu([QUIET] * 30) if gpu == "quiet" else gpu,
                                host_load=FakeHostLoad(), host=self.host, sleep=lambda s: None,
                                lock=lambda: self.lock, hold_settle=0)
        finally:
            for key, value in saved.items():
                if value is None: os.environ.pop(key, None)
                else: os.environ[key] = value

    def cli_preflight_without_nvidia_smi(self, lane):
        # Everything a measured preflight needs except nvidia-smi: the sampler alone must refuse.
        bins = self.root / "no-nvidia-smi"; bins.mkdir(exist_ok=True)
        (bins / "host-load").write_text("#!/bin/sh\nexit 0\n"); (bins / "host-load").chmod(0o755)
        saved = {k: os.environ.get(k) for k in ("PATH", "XDG_SESSION_TYPE")}
        os.environ.update({"PATH": str(bins), "XDG_SESSION_TYPE": "wayland"})
        stderr = io.StringIO()
        try:
            with contextlib.redirect_stderr(stderr):
                code = cm.main(["preflight", str(self.run), "--lane", lane, "--task", "material-18c2a1",
                                "--fixture", "t.sh", "--owner-pid", "4242"])
            return code, stderr.getvalue()
        finally:
            for key, value in saved.items():
                if value is None: os.environ.pop(key, None)
                else: os.environ[key] = value

    def test_measured_lanes_without_nvidia_smi_refuse_before_touching_the_host(self):
        for lane in ("headless", "dedicated"):
            with self.subTest(lane=lane):
                code, stderr = self.cli_preflight_without_nvidia_smi(lane)
                self.assertEqual(code, 2, stderr)
                self.assertIn("nvidia-smi not found", stderr)
                self.assertFalse((self.run / cm.RECORD).exists())   # refused before the run section and the lock
                self.assertEqual(self.host.calls, [])
                self.assertIsNone(cm.read_lock(self.lock))
                self.assertFalse((self.lock.with_name(cm.ch.HOLD_NAME)).exists())

    def test_pixels_without_nvidia_smi_holds_and_records_no_gpu(self):
        code, stderr = self.cli_preflight_without_nvidia_smi("pixels")
        self.assertEqual(code, 0, stderr)
        record = cm.load_record(self.run)
        self.assertEqual(record["preflight"]["verdict"], "unsampled")
        self.assertNotIn("gpu", record["environment"])
        self.assertEqual([i.get("unit", i["kind"]) for i in record["hold"]["items"]],
                         ["wali-rotate.timer", "dropbox.service"])
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(self.host.snapshot(), self.before)

    def test_dedicated_refuses_a_live_desktop_before_holding(self):
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated")
        preflight = cm.load_record(self.run)["preflight"]
        self.assertEqual(preflight["verdict"], "refused")
        self.assertIn("a desktop niri is running", " ".join(preflight["reasons"]))
        self.assertEqual(preflight["host_condition"], "desktop")
        self.assertEqual(hold_actions(self.host), [])
        self.assertIsNone(cm.read_lock(self.lock))
        self.assertNotIn("baseline", cm.load_record(self.run))

    def test_dedicated_refuses_a_graphics_client_before_holding(self):
        self.host.live.clear()
        gpu = FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": ["niri"]})
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated", gpu=gpu)
        self.assertIn("gpu_clients present on the dedicated lane: niri",
                      cm.load_record(self.run)["preflight"]["reasons"])
        self.assertEqual(hold_actions(self.host), [])

    def test_dedicated_refuses_display_variables_before_holding(self):
        self.host.live.clear()
        with self.assertRaises(cm.Refused):
            self.lane_preflight("dedicated", env={"XDG_SESSION_TYPE": "tty", "WAYLAND_DISPLAY": "wayland-1"})
        self.assertEqual(hold_actions(self.host), [])

    def test_headless_refuses_a_compute_client_before_holding(self):
        gpu = FakeGpu([QUIET] * 30, clients={"compute": ["python3"], "graphics": []})
        with self.assertRaises(cm.Refused):
            self.lane_preflight("headless", gpu=gpu, env={"XDG_SESSION_TYPE": "wayland"})
        self.assertIn("compute clients present: python3", cm.load_record(self.run)["preflight"]["reasons"])
        self.assertEqual(hold_actions(self.host), [])

    def test_dedicated_ignores_a_dead_desktop_socket(self):
        self.host.live.clear()   # the socket file stays; niri msg version fails against it
        self.lane_preflight("dedicated", gpu=FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": []}))
        preflight = cm.load_record(self.run)["preflight"]
        self.assertEqual((preflight["verdict"], preflight["host_condition"]), ("quiet", "tty"))
        self.assertEqual(cm.main(["release", str(self.run)]), 0)

    def test_host_condition_is_desktop_whenever_a_desktop_is_live(self):
        self.lane_preflight("headless", env={"XDG_SESSION_TYPE": "tty"})
        self.assertEqual(cm.load_record(self.run)["preflight"]["host_condition"], "desktop")

    def test_pixel_preflight_holds_timers_and_services_only_and_samples_nothing(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        record = cm.load_record(self.run)
        self.assertEqual([i.get("unit", i["kind"]) for i in record["hold"]["items"]],
                         ["wali-rotate.timer", "dropbox.service"])
        self.assertNotIn(("noctalia", "msg", "caffeine-enable"), [tuple(c) for c in self.host.calls])
        self.assertFalse([c for c in self.host.calls if "power-off-monitors" in c])
        self.assertNotIn("baseline", record)
        self.assertNotIn("gpu", record["environment"])
        self.assertEqual(record["preflight"]["verdict"], "unsampled")
        self.assertEqual(record["preflight"]["host_condition"], "desktop")
        self.assertNotIn("thresholds", record["preflight"])
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(self.host.snapshot(), self.before)

    def test_pixel_preflight_never_probes_idle_or_monitors(self):
        # Either probe refuses a measured hold; a pixel run must not reach them.
        self.host.fail[("noctalia", "msg", "status")] = "noctalia: no shell running"
        (self.host.sysfs / "card1-DP-1" / "enabled").unlink()
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        self.assertEqual(cm.load_record(self.run)["preflight"]["verdict"], "unsampled")
        self.assertNotIn(("noctalia", "msg", "status"), [tuple(c) for c in self.host.calls])
        self.assertEqual(cm.main(["release", str(self.run)]), 0)

    def test_pixel_preflight_records_the_gpu_when_one_is_present(self):
        self.lane_preflight("pixels", gpu=FakeGpu([]), env={"XDG_SESSION_TYPE": "wayland"})
        self.assertEqual(cm.load_record(self.run)["environment"]["gpu"]["name"], "NVIDIA test")

    def test_begin_opens_a_pixel_sub_run_and_finish_closes_it(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        (self.run / "A.kdl").write_text("glass")
        cm.begin(self.run, "A", [self.run / "A.kdl"], lock=lambda: self.lock)
        entry = cm.load_record(self.run)["sub_runs"][-1]
        self.assertEqual((entry["name"], entry["verdict"]), ("A", "begun"))
        self.assertEqual(entry["inputs"][0]["name"], "A.kdl")
        cm.finish_sub_run(self.run, "A")
        self.assertIn("finished", cm.load_record(self.run)["sub_runs"][-1])

    def test_settle_refuses_a_pixel_run_and_begin_refuses_a_measured_run(self):
        self.lane_preflight("pixels", gpu=None, env={"XDG_SESSION_TYPE": "wayland"})
        with self.assertRaisesRegex(cm.CannotRun, "pixels run opens sub-runs with begin"):
            cm.settle(self.run, "A", [], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                      FakeGpu([QUIET] * 3), sleep=lambda s: None, lock=lambda: self.lock)
        other = self.root / "runs" / "measured"; other.mkdir()
        PreflightTests("test_quiet_headless_writes_run_environment_baseline_preflight").run_preflight(
            other, lock=lambda: other / "lock")
        with self.assertRaisesRegex(cm.CannotRun, "begin is for pixels runs"):
            cm.begin(other, "A", [], lock=lambda: other / "lock")

    def settle_one(self, name, clients=None):
        (self.run / "A.kdl").write_text("glass")
        cm.settle(self.run, name, [self.run / "A.kdl"], 3, FakeProc([(i * 2, i * 100) for i in range(9)]),
                  FakeGpu([QUIET] * 3, clients=clients), sleep=lambda s: None, lock=lambda: self.lock)

    def test_release_refuses_an_unverified_finished_launch_after_restoring(self):
        self.preflight()
        self.settle_one("A"); cm.finish_sub_run(self.run, "A")
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        self.assertEqual(self.host.snapshot(), self.before)
        self.assertIsNone(cm.read_lock(self.lock))
        self.assertEqual(cm.main(["release", str(self.run)]), 1)   # repeats

    def test_release_refuses_an_open_unverified_dedicated_launch(self):
        self.host.live.clear()
        self.lane_preflight("dedicated", gpu=FakeGpu([QUIET] * 30, clients={"compute": [], "graphics": []}))
        # FakeGpu reports a graphics client by default, which the dedicated lane refuses.
        self.settle_one("drm-aurora", clients={"compute": [], "graphics": []})   # never finished, as idle-budget power entries are today
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        self.assertIsNone(cm.read_lock(self.lock))

    def test_release_passes_a_verified_launch_and_ignores_a_refused_settle(self):
        self.preflight()
        self.settle_one("A")
        weston = self.root / "w.log"; weston.write_text((DATA / "weston-nvidia.log").read_text())
        # FakeGpu's static name is "NVIDIA test"; the fixture names a real GPU, so pin the expected name.
        record = cm.load_record(self.run); record["environment"]["gpu"]["name"] = "NVIDIA GeForce RTX 3070"
        cm.save_record(self.run, record)
        cm.renderer(self.run, "A", DATA / "niri-nvidia.log", weston)
        cm.finish_sub_run(self.run, "A")
        cm.append_sub_run(self.run, {"name": "B", "verdict": "refused", "reason": "gpu busy"})
        self.assertEqual(cm.main(["release", str(self.run)]), 0)

    def test_quiet_preflight_holds_then_release_restores_and_is_clean(self):
        self.preflight()
        record = cm.load_record(self.run)
        self.assertEqual([i.get("unit", i["kind"]) for i in record["hold"]["items"]],
                         ["wali-rotate.timer", "dropbox.service", "idle", "monitors"])
        self.assertTrue(self.host.guards)
        guard_call = next(c for c in self.host.calls if c[0] == "systemd-run")
        self.assertLess(self.host.calls.index(guard_call),
                        self.host.calls.index(("systemctl", "--user", "stop", "wali-rotate.timer")))
        started = cm.load_record(self.run)["run"]["started"]
        finished = (cm.datetime.datetime.fromisoformat(started) + cm.datetime.timedelta(seconds=754)).isoformat()
        with mock.patch.object(cm, "now_rfc3339", return_value=finished):
            self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(self.host.snapshot(), self.before)
        run = cm.load_record(self.run)["run"]
        self.assertEqual((run["finished"], run["duration_s"], run["finished_by"]), (finished, 754, "release"))
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
        self.host.add_timer("man-db.timer", "system")
        self.before = self.host.snapshot()
        self.preflight()
        self.host.journal["system"].append({"__REALTIME_TIMESTAMP": str(self.host.now_us() + 5), "UNIT": "man-db.service",
                                            "MESSAGE_ID": "7d4958e842da4a758f6c1cdc7b36dcc5", "INVOCATION_ID": "m"})
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
        self.assertNotIn("finished", cm.load_record(self.run)["run"])
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        record = cm.load_record(self.run)
        self.assertEqual(record["preflight"]["verdict"], "refused")
        self.assertIn("finished", record["run"])
        self.assertGreaterEqual(record["run"]["duration_s"], 0)

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
        self.assertEqual(cm.load_record(self.run)["run"]["finished_by"], "release")

    def test_guard_after_a_kill_mid_run_writes_scan_and_restore(self):
        self.preflight()
        self.host.alive[4242] = False
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["scan"]["verdict"], end["restore"]["attempts"][-1]["by"]), ("clean", "guard"))
        self.assertEqual(self.host.snapshot(), self.before)
        run = cm.load_record(self.run)["run"]
        self.assertEqual(run["finished_by"], "guard")
        self.assertGreaterEqual(run["duration_s"], 0)

    def test_guard_stamps_the_end_after_a_kill_left_only_the_record(self):
        self.preflight()
        self.host.alive[4242] = False
        real = cm.ch.remove_hold
        def killed(lock_file, hold):
            raise FakeHost.Killed()
        cm.ch.remove_hold = killed
        try:
            with self.assertRaises(FakeHost.Killed):
                cm.guard(self.host, self.lock, self.run)
        finally:
            cm.ch.remove_hold = real
        stamped = cm.load_record(self.run)["run"]
        self.assertEqual(stamped["finished_by"], "guard")
        self.assertTrue((self.host.runtime / ch_hold_name()).exists())
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        self.assertEqual(cm.load_record(self.run)["run"], stamped)

    def test_a_kill_before_the_stamp_is_stamped_by_the_next_recovery(self):
        self.preflight()
        self.host.alive[4242] = False
        real = cm.stamp_finish
        def killed(record, by):
            raise FakeHost.Killed()
        cm.stamp_finish = killed
        try:
            with self.assertRaises(FakeHost.Killed):
                cm.guard(self.host, self.lock, self.run)
        finally:
            cm.stamp_finish = real
        record = cm.load_record(self.run)
        self.assertEqual(record["hold_end"]["restore"]["state"], "complete")
        self.assertNotIn("finished", record["run"])
        self.assertEqual(cm.main(["restore"]), 0)
        self.assertEqual(cm.load_record(self.run)["run"]["finished_by"], "hand")

    def test_recovered_end_refreshes_the_manifest(self):
        import hashlib
        self.preflight()
        record = self.run / cm.RECORD
        manifest = self.run / "SHA256SUMS"
        manifest.write_text(hashlib.sha256(record.read_bytes()).hexdigest() + "  ./capture.json\n")
        self.host.alive[4242] = False
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        self.assertIn("finished", cm.load_record(self.run)["run"])
        self.assertEqual(manifest.read_text(), hashlib.sha256(record.read_bytes()).hexdigest() + "  ./capture.json\n")

    def test_guard_survives_an_unreadable_connector(self):
        self.preflight()
        self.host.clock += 1_000_000                  # into the run window
        (self.host.sysfs / "card1-DP-1" / "dpms").unlink()
        self.assertIsNone(cm.guard(self.host, self.lock, self.run, iterations=2))
        self.host.sync_sysfs()
        self.host.clock += 1_000_000
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        kinds = [d["kind"] for d in cm.load_record(self.run)["hold_end"]["scan"]["disturbances"]]
        self.assertEqual(kinds, ["monitor-unwatched"])

    def test_kill_between_restore_and_record_keeps_the_disturbance(self):
        self.preflight()
        self.host.journal["user"].append({"__REALTIME_TIMESTAMP": str(self.host.now_us() + 5), "USER_UNIT": "dropbox.service",
                                          "MESSAGE_ID": "39f53479d3a045ac8e11786248231fbf", "USER_INVOCATION_ID": "d1"})
        self.host.clock += 60_000_000
        real = cm._update_hold_end
        def killed(*a, **k):
            raise FakeHost.Killed()
        cm._update_hold_end = killed
        try:
            with self.assertRaises(FakeHost.Killed):
                cm.main(["release", str(self.run)])
        finally:
            cm._update_hold_end = real
        self.assertTrue((self.host.runtime / ch_hold_name()).exists())
        self.assertNotIn("hold_end", cm.load_record(self.run))
        self.assertEqual(cm.main(["release", str(self.run)]), 1)
        end = cm.load_record(self.run)["hold_end"]
        self.assertEqual((end["restore"]["state"], end["scan"]["verdict"]), ("complete", "disturbed"))
        self.assertEqual([d["kind"] for d in end["scan"]["disturbances"]], ["held-started"])

    def test_a_kill_after_the_record_before_each_cleanup_step_is_finished_by_release(self):
        def kill_remove(lock_file, hold):
            raise FakeHost.Killed()
        real_run = FakeHost.run
        def kill_stop(host, *command, **kw):
            if command[:3] == ("systemctl", "--user", "stop") and command[3].startswith("capture-meta-guard-"):
                raise FakeHost.Killed()
            return real_run(host, *command, **kw)
        def kill_release(*a, **k):
            raise FakeHost.Killed()
        steps = (("refresh manifest", cm, "refresh_record_sum", kill_release),
                 ("remove hold", cm.ch, "remove_hold", kill_remove),
                 ("stop guard", FakeHost, "run", kill_stop),
                 ("release lock", cm, "release_lock", kill_release))
        starts = (("preflighted", lambda: self.preflight()),
                  ("only run", self.hold_without_preflight))
        for (name, target, attribute, replacement), (start, begin) in itertools.product(steps, starts):
            with self.subTest(step=name, start=start):
                self.setUp()
                begin()
                original = getattr(target, attribute)
                setattr(target, attribute, replacement)
                try:
                    with self.assertRaises(FakeHost.Killed):
                        cm.main(["release", str(self.run)])
                finally:
                    setattr(target, attribute, original)
                saved = (self.run / cm.RECORD).read_text()
                self.assertEqual(cm.load_record(self.run)["hold_end"]["restore"]["state"], "complete")
                self.assertEqual(cm.main(["release", str(self.run)]), 0)
                self.assertEqual((self.run / cm.RECORD).read_text(), saved)
                self.assertFalse((self.host.runtime / ch_hold_name()).exists())
                self.assertFalse(self.host.guards)
                self.assertIsNone(cm.read_lock(self.lock))

    def hold_without_preflight(self):
        """The state preflight leaves when killed right after holding: a record with only
        `run`, a hold file, a guard and the lock."""
        cm.write_section(self.run, "run", {"id": self.run.name, "started": STARTED})
        cm.acquire_lock(self.lock, 4242, self.run.name, run_dir=self.run)
        plan = cm.ch.plan_hold(self.host, cm.ch.desktop_socket(self.host), cm.ch.HOLD_KINDS)
        hold = cm.ch.create_hold(self.lock, self.run.name, self.run, 4242, plan, self.host.now_us())
        cm.start_guard(self.host, hold)
        cm.ch.apply_hold(self.host, self.lock, self.run.name, self.run, plan)
        self.assertEqual(set(cm.load_record(self.run)), {"schema", "run"})

    def test_restore_by_hand_refuses_a_live_owner_and_records_hand(self):
        self.preflight()
        self.assertEqual(cm.main(["restore"]), 1)
        self.host.alive[4242] = False
        self.assertEqual(cm.main(["restore"]), 0)
        self.assertEqual(cm.load_record(self.run)["hold_end"]["restore"]["attempts"][-1]["by"], "hand")
        self.assertEqual(cm.load_record(self.run)["run"]["finished_by"], "hand")
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
        self.assertEqual(cm.load_record(first)["run"]["finished_by"], "next-preflight")
        self.assertNotIn("finished", cm.load_record(self.run)["run"])

    def test_hold_file_naming_another_run_dir_is_not_released(self):
        self.preflight()
        twin = self.root / "elsewhere" / "pilot-1"; twin.mkdir(parents=True)
        cm.write_section(twin, "run", {"id": "pilot-1", "started": STARTED})
        cm.main(["release", str(twin)])
        self.assertTrue((self.host.runtime / ch_hold_name()).exists())


    def test_completed_release_keeps_a_colliding_live_runs_lock(self):
        self.preflight()
        old = self.run
        self.assertEqual(cm.main(["release", str(old)]), 0)
        self.run = self.root / "other" / old.name
        self.run.mkdir(parents=True)
        self.preflight()  # same owner PID and basename, different directory
        lock = self.lock.read_bytes()
        hold = cm.ch.hold_file(self.lock).read_bytes()
        self.assertEqual(cm.main(["release", str(old)]), 0)
        self.assertTrue(self.lock.exists())
        self.assertEqual(self.lock.read_bytes(), lock)
        self.assertEqual(cm.ch.hold_file(self.lock).read_bytes(), hold)

    def test_release_after_the_manifest_rehashes_the_stamped_record(self):
        import hashlib
        self.preflight()
        record = self.run / cm.RECORD
        manifest = self.run / "SHA256SUMS"
        manifest.write_text(hashlib.sha256(record.read_bytes()).hexdigest() + "  ./capture.json\n")
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertIn("finished", cm.load_record(self.run)["run"])
        self.assertEqual(manifest.read_text(), hashlib.sha256(record.read_bytes()).hexdigest() + "  ./capture.json\n")

    def test_completed_release_does_not_rehash_an_altered_record(self):
        import hashlib
        self.preflight()
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        record = self.run / cm.RECORD
        manifest = self.run / "SHA256SUMS"
        manifest.write_text(hashlib.sha256(record.read_bytes()).hexdigest() + "  ./capture.json\n")
        original = manifest.read_bytes()
        data = json.loads(record.read_text())
        data["run"]["task"] = "altered"
        record.write_text(json.dumps(data))
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual(manifest.read_bytes(), original)

    def test_unreadable_wake_log_restores_the_host_and_records_unscanned(self):
        for cause in (PermissionError("wake evidence denied"), UnicodeError("invalid wake encoding")):
            with self.subTest(cause=cause):
                self.setUp()
                self.preflight()
                self.host.alive[4242] = False
                wake = cm.ch.wake_file(self.lock, cm.ch.read_hold(cm.ch.hold_file(self.lock)))
                read_text = pathlib.Path.read_text
                def unreadable(path, *args, **kwargs):
                    if path == wake:
                        raise cause
                    return read_text(path, *args, **kwargs)
                with mock.patch.object(pathlib.Path, "read_text", unreadable):
                    self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
                self.assertEqual(self.host.snapshot(), self.before)
                end = cm.load_record(self.run)["hold_end"]
                self.assertEqual(end["restore"]["state"], "complete")
                self.assertEqual(end["scan"]["verdict"], "unscanned")
                self.assertIn(str(cause), end["scan"]["error"])

    def test_guard_recovery_refreshes_the_finished_capture_checksum(self):
        self.preflight()
        self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")] = "busy"
        self.assertEqual(cm.main(["release", str(self.run)]), 2)
        # The failure persists through the fixture's final cleanup release.
        self.assertEqual(cm.main(["release", str(self.run)]), 2)
        (self.run / "artifact").write_text("evidence")
        (self.run / "SHA256SUMS").write_text(
            f"{cm.sha256_file(self.run / cm.RECORD)}  ./capture.json\n"
            f"{cm.sha256_file(self.run / 'artifact')}  ./artifact\n")
        del self.host.fail[("systemctl", "--user", "start", "wali-rotate.timer")]
        self.host.alive[4242] = False
        self.assertEqual(cm.guard(self.host, self.lock, self.run), "restored")
        check = subprocess.run(["sha256sum", "-c", "--quiet", "SHA256SUMS"], cwd=self.run,
                               capture_output=True, text=True)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)
        self.assertEqual(cm.load_record(self.run)["hold_end"]["restore"]["state"], "complete")
        saved = (self.run / "SHA256SUMS").read_bytes()
        self.assertEqual(cm.main(["release", str(self.run)]), 0)
        self.assertEqual((self.run / "SHA256SUMS").read_bytes(), saved)


def ch_hold_name():
    return cm.ch.HOLD_NAME


HOLD_ACTIONS = (("systemd-run",), ("systemctl", "--user", "stop"), ("noctalia", "msg", "caffeine-enable"),
                ("niri", "msg", "action", "power-off-monitors"))


def hold_actions(host):
    return [c for c in host.calls if any(tuple(c[:len(a)]) == a for a in HOLD_ACTIONS)]


if __name__ == "__main__":
    unittest.main()
