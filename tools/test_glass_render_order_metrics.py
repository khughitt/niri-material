"""Catch absolute-grain statistics and noise-dependent additive light."""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


class RenderOrderMetricsTest(unittest.TestCase):
    def test_signed_grain_and_quantized_additive_light(self):
        path = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts'
                / 'glass-render-order-metrics.py')
        spec = importlib.util.spec_from_file_location('render_order_metrics', path)
        metrics = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(metrics)

        self.assertAlmostEqual(metrics.grain_sd([(90,) * 3, (110,) * 3],
                                               [(100,) * 3] * 2), 10 / 255)
        self.assertEqual(metrics.grain_sd([(100,) * 3] * 2, [(100,) * 3] * 2), 0)
        for on, off in (([], []), ([(100,) * 3], [])):
            with self.assertRaises(ValueError):
                metrics.grain_sd(on, off)

        lo, hi = metrics.additive_interval([160, 128, 160, 128])
        self.assertLessEqual(lo, 0)
        self.assertGreaterEqual(hi, 0)
        lo, hi = metrics.additive_interval([190, 128, 160, 128])
        self.assertGreater(lo, 0)

        def frames(codes):
            return [[(c,) * 3] * 400 for c in codes]

        good = metrics.additive_report(frames([160, 128, 160, 128]))
        self.assertEqual(good['failures'], 0)
        self.assertEqual(good['valid_channels'], 1200)
        self.assertEqual(good['informative_channels'], 1200)
        bad = metrics.additive_report(frames([190, 128, 160, 128]))
        self.assertEqual(bad['failures'], 1200)
        with self.assertRaisesRegex(ValueError, 'uninformative'):
            metrics.additive_report(frames([128, 128, 128, 128]))
        with self.assertRaisesRegex(ValueError, 'uninformative'):
            metrics.additive_report(frames([255, 128, 160, 128]))
        with self.assertRaises(ValueError):
            metrics.additive_report([[(100,) * 3]] * 3)
        with self.assertRaises(ValueError):
            metrics.additive_report([[(100,) * 3]] * 3 + [[]])

        self.assertEqual(metrics.check_rect((3, 4, 10, 20), 20, 30), '10x20+3+4')
        for rect in ((-1, 0, 1, 1), (0, 0, 0, 1), (19, 0, 2, 1)):
            with self.assertRaises(ValueError):
                metrics.check_rect(rect, 20, 30)

    @unittest.skipUnless(shutil.which('magick'), 'ImageMagick is required for capture decoding')
    def test_cli_distinguishes_failed_gate_from_invalid_capture(self):
        script = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts'
                  / 'glass-render-order-metrics.py')
        with tempfile.TemporaryDirectory() as directory:
            paths = {}
            for code in (128, 160, 190, 255):
                path = Path(directory) / f'{code}.ppm'
                path.write_bytes(b'P6\n20 20\n255\n' + bytes([code]) * 1200)
                paths[code] = str(path)

            def run(command, codes, rect=(0, 0, 20, 20)):
                return subprocess.run([sys.executable, str(script), command,
                                       *(paths[c] for c in codes), '--rect', *map(str, rect)],
                                      capture_output=True, text=True)

            grain = run('grain', [160, 128])
            self.assertEqual(grain.returncode, 0, grain.stderr)
            self.assertEqual(json.loads(grain.stdout)['sd'], 0)
            good = run('additive', [160, 128, 160, 128])
            self.assertEqual(good.returncode, 0, good.stderr)
            bad = run('additive', [190, 128, 160, 128])
            self.assertEqual(bad.returncode, 1, bad.stderr)
            self.assertEqual(json.loads(bad.stdout)['failures'], 1200)
            clipped = run('additive', [255, 128, 160, 128])
            self.assertEqual(clipped.returncode, 2)
            self.assertIn('uninformative', clipped.stderr)
            outside = run('grain', [160, 128], rect=(0, 0, 21, 20))
            self.assertEqual(outside.returncode, 2)
            Path(paths[160]).write_bytes(b'invalid image')
            corrupt = run('grain', [160, 128])
            self.assertEqual(corrupt.returncode, 2)
