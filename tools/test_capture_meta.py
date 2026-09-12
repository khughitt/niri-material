"""Unit tests for tools/capture-meta."""
import importlib.machinery
import importlib.util
import json
import os
import pathlib
import tempfile
import unittest

for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

spec = importlib.util.spec_from_loader(
    "capture_meta",
    importlib.machinery.SourceFileLoader(
        "capture_meta", str(pathlib.Path(__file__).with_name("capture-meta"))))
cm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cm)


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

    def test_summary_medians_iqr_and_clients(self):
        proc = FakeProc([(0, 0), (2, 100), (4, 200), (6, 300)])
        gpu = FakeGpu([dict(QUIET, power_w=18.0), dict(QUIET, power_w=19.0), dict(QUIET, power_w=25.0, pstate="P0")])
        summary = cm.summarize(cm.sample_stream(proc, gpu, 3, sleep=lambda s: None))
        self.assertEqual(summary["gpu_power_w"], 19.0)
        self.assertEqual(summary["gpu_power_iqr_w"], 3.5)
        self.assertEqual(summary["gpu_pstates"], ["P0", "P8"])


class GpuEvidenceTests(unittest.TestCase):
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


class JudgementTests(unittest.TestCase):
    def test_quiet_and_settle(self):
        samples = quiet_samples()
        self.assertEqual(cm.judge_quiet(cm.summarize(samples), samples, cm.DEFAULT_THRESHOLDS, "headless"), [])
        samples = quiet_samples(3, power_w=24.1)
        reasons = cm.judge_settled(cm.summarize(samples), samples, cm.summarize(quiet_samples()), cm.DEFAULT_THRESHOLDS, "headless")
        self.assertIn("gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5", " ".join(reasons))
    def test_threshold_validation(self):
        self.assertEqual(cm.parse_thresholds(["cpu_busy_pct=20", "gpu_pstate=P5"])["cpu_busy_pct"], 20.0)
        for pair in ("nonsense=1", "load1=nan", "load1=-1", "gpu_pstate=X8"):
            with self.assertRaises(cm.CannotRun): cm.parse_thresholds([pair])


if __name__ == "__main__":
    unittest.main()
