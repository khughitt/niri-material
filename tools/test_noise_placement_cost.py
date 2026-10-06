"""Offline checks of the real cost script's failure cleanup and CSV report."""
import csv
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "docs/materials/scripts/noise-placement-cost.sh"
LIB = ROOT / "docs/materials/scripts/glass-optic-smoke-lib.sh"


def report_source():
    return SCRIPT.read_text().split("<<'PYREPORT' >> \"$OUT/metrics.txt\"\n", 1)[1].split("\nPYREPORT", 1)[0]


class NoisePlacementCostTests(unittest.TestCase):
    def test_failure_cleanup_reaps_owned_wallpaper_and_preserves_exit_status(self):
        source = LIB.read_text()
        self.assertIn("trap cleanup_cost EXIT", SCRIPT.read_text())
        stop = source.split("stop_walls() {", 1)[1].split("\n}\n", 1)[0]
        cleanup = source.split("cleanup_cost() {", 1)[1].split("\n}\n", 1)[0]
        for code in (0, 23):
            with self.subTest(code=code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                harness = (
                    "set -eu\nSTATUS_FILE=$2\nWALL_PIDS=()\nstop_walls() {" + stop + "\n}\n"
                    "cleanup_cost() {" + cleanup + "\n}\n"
                    'cleanup() { local rc=$?; printf "%s" "$rc" > "$STATUS_FILE"; exit "$rc"; }\n'
                    'trap cleanup_cost EXIT\nsleep 30 >/dev/null 2>&1 &\n'
                    'WALL_PIDS+=($!)\nsleep 0.1\nprintf "%s" "$!" > "$1"\nexit "$3"\n'
                )
                pid_file, status_file = root / "pid", root / "status"
                child = subprocess.Popen(
                    ["bash", "-c", harness, "cleanup-test", str(pid_file), str(status_file), str(code)],
                    start_new_session=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                )
                try:
                    _, stderr = child.communicate(timeout=5)
                    self.assertEqual(child.returncode, code, stderr.decode())
                    self.assertTrue(status_file.exists(), "library cleanup was skipped")
                    self.assertEqual(status_file.read_text(), str(code))
                    with self.assertRaises(ProcessLookupError):
                        os.kill(int(pid_file.read_text()), 0)
                finally:
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    child.wait()
                    if child.stderr is not None:
                        child.stderr.close()

    def damage_report(self, *, missing_map=False, missing_sharp=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cpu = [
                ("State::reload_config", t, 5) for t in (1000, 1500, 2000)
            ] + [("EffectBuffer::prepare_grain", 0, 1)]
            for t in (1100, 1600):
                if not (missing_map and t == 1600):
                    cpu.append(("Layer::mapped", t, 1))
                cpu += [("EffectBuffer::sharp_damage", t + 1, 1),
                        ("EffectBuffer::prepare_grain", t + 2, 1),
                        ("EffectBuffer::prepare_blurred_prefilter", t + 3, 1)]
                if not missing_sharp:
                    cpu.append(("EffectBuffer::prepare_sharp_prefilter", t + 4, 1))
            gpu = [("Grain::render", 0, 10000)]
            for t, grain in [(1100, 100), (1600, 300)]:
                gpu += [("Grain::render", t, grain), ("Blur::render", t + 100, 400),
                        ("Prefilter::downsample", t + 200, 500),
                        ("MaterialRenderElement::draw", t + 300, 200)]
            for suffix, header, rows in [
                ("csv", ["name", "ns_since_start", "exec_time_ns"], cpu),
                ("gpu.csv", ["name", "Time from start of program", "GPU execution time"], gpu),
            ]:
                with (root / ("damage-backdrop." + suffix)).open("w", newline="") as f:
                    writer = csv.writer(f)
                    writer.writerow(header)
                    writer.writerows(rows)
            return subprocess.run(
                ["python3", "-c", report_source(), str(root), "damage-backdrop", "damage", "backdrop", "2"],
                capture_output=True, text=True, timeout=5,
            )

    def test_damage_report_uses_reload_window_and_counts_every_buffer_pass(self):
        result = self.damage_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        metrics = dict(line.split("=", 1) for line in result.stdout.splitlines())
        self.assertEqual(metrics["damage-backdrop_EffectBuffer::prepare_grain_count"], "2")
        self.assertEqual(metrics["damage-backdrop_Grain::render_median_ms"], "0.000200")
        self.assertEqual(metrics["damage-backdrop_mean_total_per_change_ms"], "0.001300")

    def test_damage_report_refuses_a_partial_wallpaper_stimulus(self):
        result = self.damage_report(missing_map=True)
        self.assertNotEqual(result.returncode, 0, "old samples must not validate a missing wallpaper update")

    def test_damage_report_requires_both_pyramid_consumers(self):
        result = self.damage_report(missing_sharp=True)
        self.assertNotEqual(result.returncode, 0, "the blurred pyramid alone is not the full cascade")
