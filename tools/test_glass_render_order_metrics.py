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
    @staticmethod
    def metrics():
        path = (Path(__file__).resolve().parents[1] / 'docs/materials/scripts'
                / 'glass-render-order-metrics.py')
        spec = importlib.util.spec_from_file_location('render_order_metrics', path)
        metrics = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(metrics)
        return metrics

    def test_reach_uses_scatter_bound_and_rounded_slab_geometry(self):
        metrics = self.metrics()
        self.assertEqual(metrics.ring_bound(20, 5, 2.6), 37)
        self.assertEqual(metrics.ring_bound(80, 5, 2.6), 61)
        self.assertEqual(metrics.ring_bound(20, 5, 20), 61)
        self.assertEqual(metrics.ring_bound(20, 5, 2.6, 0.5), 56)
        self.assertEqual(metrics.ring_bound(20, 20, 2.6), 59)

        off = [[(0, 0, 0)] * 120 for _ in range(120)]
        on = [[(0, 0, 0)] * 120 for _ in range(120)]
        on[50][50] = (2, 0, 0)  # interior reach, not an outside-bound failure
        on[36][36] = (2, 0, 0)  # outside the rounded slab, ignored
        report = metrics.reach_report(on, off, (40, 40, 100, 100), 4, 20, 5, 2.6)
        self.assertEqual(report['outside_bound_pixels'], 0)
        self.assertGreaterEqual(report['visible_interior_reach'], 0)
        self.assertEqual(report['outside_slab_pixels'], 1)

        on[90][90] = (1, 0, 0)  # one-code noise passes even outside the bound
        report = metrics.reach_report(on, off, (40, 40, 100, 100), 4, 20, 5, 2.6)
        self.assertEqual(report['outside_bound_pixels'], 0)
        on[90][90] = (2, 0, 0)
        report = metrics.reach_report(on, off, (40, 40, 100, 100), 4, 20, 5, 2.6)
        self.assertEqual(report['outside_bound_pixels'], 1)
        self.assertEqual(report['max_channel_delta'], 2)
        self.assertEqual(report['max_interior_channel_delta'], 2)

        on[50][50] = (0, 0, 0)
        on[90][90] = (0, 0, 0)
        report = metrics.reach_report(on, off, (40, 40, 100, 100), 4, 20, 5, 2.6)
        self.assertEqual(report['max_channel_delta'], 2)
        self.assertEqual(report['max_interior_channel_delta'], 0)

    def test_reach_rejects_invalid_or_uncovered_slab_even_without_a_delta(self):
        metrics = self.metrics()
        frame = [[(0, 0, 0)] * 4 for _ in range(4)]
        with self.assertRaisesRegex(ValueError, 'covered slab'):
            metrics.reach_report(frame, frame, (20, 20, 4, 4), 2, 20, 5, 2.6)
        with self.assertRaisesRegex(ValueError, 'invalid reach geometry'):
            metrics.reach_report(frame, frame, (0, 0, 4, 4), 2, 20, 5, 0)

    def test_signed_grain_and_quantized_additive_light(self):
        metrics = self.metrics()

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

    def test_profile_and_attenuation_use_decoded_quantization_intervals(self):
        metrics = self.metrics()
        pinned = [(0, 0, 0), (4, 4, 4), (8, 8, 8), (4, 4, 4), (0, 0, 0)]
        rough = [(0, 0, 0), (3, 3, 3), (4, 4, 4), (3, 3, 3), (0, 0, 0)]
        self.assertEqual(metrics.profile_report(pinned, [(0, 0, 0)] * 5),
                         {'profile_peak_delta': 8, 'profile_fwhm': 3})
        self.assertEqual(metrics.profile_report(rough, [(0, 0, 0)] * 5),
                         {'profile_peak_delta': 4, 'profile_fwhm': 3})
        with self.assertRaisesRegex(ValueError, 'uninformative'):
            metrics.profile_report([(1, 1, 1)], [(0, 0, 0)])

        # Linear deltas for dense and white are both quantized; the expected
        # ratio belongs in their quotient interval, rather than matching codes.
        frames = [[(160, 160, 160)] * 100,
                  [(128, 128, 128)] * 100,
                  [(190, 190, 190)] * 100,
                  [(128, 128, 128)] * 100]
        expected = (metrics.decode(160 / 255) - metrics.decode(128 / 255)) / \
                   (metrics.decode(190 / 255) - metrics.decode(128 / 255))
        report = metrics.attenuation_report(frames, [expected] * 3)
        self.assertEqual(report['failures'], 0)
        self.assertEqual(report['valid_channels'], 300)
        report = metrics.attenuation_report(frames, [expected * 2] * 3)
        self.assertEqual(report['failures'], 300)
        with self.assertRaisesRegex(ValueError, 'uninformative'):
            metrics.attenuation_report([[((128,) * 3)] * 100] * 4, [1, 1, 1])

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
            expected = (self.metrics().decode(160 / 255) - self.metrics().decode(128 / 255)) / \
                       (self.metrics().decode(190 / 255) - self.metrics().decode(128 / 255))
            reach = subprocess.run([sys.executable, str(script), 'reach', paths[190], paths[128],
                                    '--window', '0', '0', '20', '20', '--bevel', '0',
                                    '--thickness', '20', '--inset', '5', '--width', '2.6',
                                    '--profile', '0', '0', '20', '1', '--attenuation-images',
                                    paths[160], paths[128], paths[190], paths[128],
                                    '--attenuation-rect', '0', '0', '20', '20',
                                    '--attenuation-ratio', *[str(expected)] * 3],
                                   capture_output=True, text=True)
            self.assertEqual(reach.returncode, 0, reach.stderr)
            report = json.loads(reach.stdout)
            self.assertEqual(report['failures'], 0)
            self.assertEqual(report['profile_fwhm'], 20)
            wrong_size = Path(directory) / 'wrong-size.ppm'
            wrong_size.write_bytes(b'P6\n10 10\n255\n' + bytes([160]) * 300)
            mismatched = subprocess.run([sys.executable, str(script), 'reach', paths[190], paths[128],
                                         '--window', '0', '0', '20', '20', '--bevel', '0',
                                         '--thickness', '20', '--inset', '5', '--width', '2.6',
                                         '--attenuation-images', str(wrong_size), paths[128],
                                         paths[190], paths[128], '--attenuation-rect', '0', '0', '10', '10',
                                         '--attenuation-ratio', *[str(expected)] * 3],
                                        capture_output=True, text=True)
            self.assertEqual(mismatched.returncode, 2)
            self.assertIn('dimensions differ', mismatched.stderr)
            outside = run('grain', [160, 128], rect=(0, 0, 21, 20))
            self.assertEqual(outside.returncode, 2)
            Path(paths[160]).write_bytes(b'invalid image')
            corrupt = run('grain', [160, 128])
            self.assertEqual(corrupt.returncode, 2)
