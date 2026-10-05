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


def report_source():
    return SCRIPT.read_text().split("<<'PYREPORT' >> \"$OUT/metrics.txt\"\n", 1)[1].split("\nPYREPORT", 1)[0]


class NoisePlacementCostTests(unittest.TestCase):
    def test_failure_cleanup_reaps_owned_wallpaper_and_preserves_exit_status(self):
        source = SCRIPT.read_text()
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

    def test_damage_report_uses_reload_window_and_counts_every_buffer_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cpu = [
                ("State::reload_config", 1000, 5), ("State::reload_config", 2000, 5),
                ("EffectBuffer::prepare_grain", 0, 1),
                ("EffectBuffer::prepare_grain", 1100, 1),
                ("EffectBuffer::prepare_grain", 1500, 1),
            ]
            gpu = [
                ("Grain::render", 0, 10000), ("Grain::render", 1100, 100),
                ("Grain::render", 1500, 300), ("Blur::render", 1200, 400),
                ("Prefilter::downsample", 1300, 500), ("MaterialRenderElement::draw", 1400, 200),
            ]
            for suffix, header, rows in [
                ("csv", ["name", "ns_since_start", "exec_time_ns"], cpu),
                ("gpu.csv", ["name", "Time from start of program", "GPU execution time"], gpu),
            ]:
                with (root / ("damage-backdrop." + suffix)).open("w", newline="") as f:
                    writer = csv.writer(f)
                    writer.writerow(header)
                    writer.writerows(rows)
            result = subprocess.run(
                ["python3", "-c", report_source(), str(root), "damage-backdrop", "damage", "backdrop", "2"],
                capture_output=True, text=True, timeout=5,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            metrics = dict(line.split("=", 1) for line in result.stdout.splitlines())
            self.assertEqual(metrics["damage-backdrop_EffectBuffer::prepare_grain_count"], "2")
            self.assertEqual(metrics["damage-backdrop_Grain::render_median_ms"], "0.000200")
            self.assertEqual(metrics["damage-backdrop_mean_total_per_change_ms"], "0.000750")
