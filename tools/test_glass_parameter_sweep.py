"""Offline stimulus routing and retained provenance for the glass sweep."""
import hashlib
import json
import os
from pathlib import Path
import signal
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'docs/materials/scripts/glass-parameter-sweep.sh'


class GlassParameterSweepTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        # Stop at the first compositor boundary; generation and metadata are real.
        for name, body in (
            ('niri', 'echo offline-test'),
            ('weston', 'echo compositor-boundary >&2; exit 1'),
        ):
            path = self.bin / name
            path.write_text('#!/bin/sh\n' + body + '\n')
            path.chmod(0o755)
        # Keep the actual generator CLI and noise algorithm; a full field takes
        # ~90 s. Reduced dimensions keep this routing check in the tooling budget.
        python = self.bin / 'python3'
        python.write_text(f'#!{sys.executable}\n' + '''import os, runpy, sys
if sys.argv[1].endswith('/warp-calibration.py'):
    module = runpy.run_path(sys.argv[1])
    module['backdrop'].__defaults__ = (8, 4)
    sys.exit(module['main'](sys.argv[2:]))
os.execv(sys.executable, [sys.executable, *sys.argv[1:]])
''')
        python.chmod(0o755)

    def run_sweep(self, out, **overrides):
        env = {k: v for k, v in os.environ.items() if k not in (
            'BACKDROP', 'TABLE_ONLY', 'ROI_BEVEL', 'ROI_FACE')}
        env.update(OUT=str(out), BLOCK='glass', KEY='thickness', VALUES='0 20',
                   NIRI=str(self.bin / 'niri'), XDG_RUNTIME_DIR=str(self.root),
                   PATH=str(self.bin) + os.pathsep + os.environ['PATH'])
        env.update(overrides)
        with subprocess.Popen(['bash', str(SCRIPT)], env=env, text=True,
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              start_new_session=True) as process:
            try:
                stdout, stderr = process.communicate(timeout=20)
            except BaseException:
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate()
                raise
            return subprocess.CompletedProcess(process.args, process.returncode, stdout, stderr)

    def capture_backdrop(self, name, **env):
        out = self.root / name
        result = self.run_sweep(out, **env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('compositor-boundary', (out / 'weston.log').read_text())
        self.assertTrue((out / 'backdrop.json').is_file(), 'capture must retain backdrop identity')
        return out

    def test_default_and_explicit_grid_keep_the_colored_stimulus(self):
        default = self.capture_backdrop('default')
        explicit = self.capture_backdrop('explicit', BACKDROP='grid20')
        for out in (default, explicit):
            record = json.loads((out / 'backdrop.json').read_text())
            self.assertEqual(record['name'], 'grid20')
            self.assertIsNone(record['seed'])
            self.assertEqual(record['generator'], 'docs/materials/scripts/glass-parameter-sweep.sh')
            self.assertEqual(record['generator_sha256'], hashlib.sha256(SCRIPT.read_bytes()).hexdigest())
            pixels = subprocess.check_output([
                'magick', str(out / 'backdrop.png'), '-depth', '8', 'rgb:-'])
            # Interior samples in the original four quadrant colors, off grid lines.
            for x, y, rgb in ((10, 10, (150, 60, 52)), (650, 10, (120, 96, 40)),
                              (10, 370, (38, 44, 62)), (650, 370, (48, 120, 80))):
                offset = (y * 1280 + x) * 3
                self.assertEqual(tuple(pixels[offset:offset + 3]), rgb)

    def test_aperiodic_routes_to_the_calibrated_deterministic_gray_field(self):
        images = []
        for name in ('first', 'repeat'):
            out = self.capture_backdrop(name, BACKDROP='aperiodic-s20260929')
            data = (out / 'backdrop.png').read_bytes()
            record = json.loads((out / 'backdrop.json').read_text())
            self.assertEqual(record['name'], 'aperiodic-s20260929')
            self.assertEqual(record['seed'], 20260929)
            generator = 'docs/materials/scripts/warp-calibration.py'
            self.assertEqual(record['generator'], generator)
            self.assertEqual(record['generator_sha256'],
                             hashlib.sha256((ROOT / generator).read_bytes()).hexdigest())
            self.assertEqual(record['png_sha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual(struct.unpack('>II', data[16:24]), (8, 4))
            self.assertIn(b'backdrop aperiodic-s20260929', data)
            chunks = []
            pos = 8
            while pos < len(data):
                size = int.from_bytes(data[pos:pos + 4], 'big')
                if data[pos + 4:pos + 8] == b'IDAT':
                    chunks.append(data[pos + 8:pos + 8 + size])
                pos += size + 12
            raw = zlib.decompress(b''.join(chunks))
            for y in range(4):
                row = raw[y * 25:(y + 1) * 25]
                self.assertEqual(row[0], 0)
                self.assertEqual(row[1::3], row[2::3])
                self.assertEqual(row[1::3], row[3::3])
            images.append(data)
        self.assertEqual(*images)

    def test_invalid_name_and_gray_saturation_refuse_before_capture(self):
        for name, env in (
            ('unknown', dict(BACKDROP='unknown')),
            ('glass', dict(BACKDROP='aperiodic-s20260929', KEY='saturation')),
            ('blur', dict(BACKDROP='aperiodic-s20260929', KEY='saturation', BLOCK='blur')),
        ):
            with self.subTest(name=name):
                out = self.root / name
                result = self.run_sweep(out, **env)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('BACKDROP', result.stderr)
                self.assertFalse((out / 'niri.version').exists())
                self.assertFalse((out / 'weston.log').exists())

    def prepared_captures(self):
        out = self.root / 'retained'
        out.mkdir()
        (out / 'values.txt').write_text('0\n20\n')
        (out / 'roi.txt').write_text('bevel\t2x2+0+0\nface\t2x2+2+2\n')
        for i in range(2):
            subprocess.run(['magick', '-size', '4x4', 'xc:gray',
                            str(out / f'cap-{i:03d}.png')], check=True)
        return out

    def test_table_only_uses_retained_identity_and_preserves_columns(self):
        out = self.prepared_captures()
        record = dict(name='aperiodic-s20260929', seed=20260929,
                      generator='retained-generator', generator_sha256='a' * 64,
                      png_sha256='b' * 64)
        path = out / 'backdrop.json'
        path.write_text(json.dumps(record))
        before = path.read_bytes()
        result = self.run_sweep(out, TABLE_ONLY='1', BACKDROP='invalid-current-env')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('aperiodic-s20260929', result.stdout)
        self.assertIn('retained-generator', result.stdout)
        self.assertNotIn('invalid-current-env', result.stdout)
        self.assertEqual(path.read_bytes(), before)
        self.assertFalse((out / 'backdrop.png').exists())
        self.assertEqual((out / 'sweep.tsv').read_text().splitlines(), [
            'key\tvalue\tstep\tbevel_neighbor\tbevel_per_step\tbevel_cumulative'
            '\tbevel_normalized\tface_neighbor\tface_per_step\tface_cumulative\tface_normalized',
            'thickness\t0\t-\t-\t-\t0\t-\t-\t-\t0\t-',
            'thickness\t20\t20\t0\t0\t0\t0.0000\t0\t0\t0\t0.0000',
        ])

    def test_older_table_reports_unrecorded_without_inventing_metadata(self):
        out = self.prepared_captures()
        result = self.run_sweep(out, TABLE_ONLY='1', BACKDROP='aperiodic-s20260929')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('Backdrop: unrecorded', result.stdout)
        self.assertFalse((out / 'backdrop.json').exists())
        self.assertFalse((out / 'backdrop.png').exists())

    def test_capture_refuses_to_relabel_retained_captures(self):
        out = self.prepared_captures()
        (out / 'backdrop.json').write_text('{"name":"aperiodic-s20260929"}')
        (out / 'backdrop.png').write_bytes(b'retained backdrop')
        before = {path.name: path.read_bytes() for path in out.iterdir()}
        result = self.run_sweep(out, BACKDROP='grid20')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('already contains captures', result.stderr)
        self.assertEqual({path.name: path.read_bytes() for path in out.iterdir()}, before)


if __name__ == '__main__':
    unittest.main()
