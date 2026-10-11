"""The capture owns its clock sampler, and wall-time joins retain uncertainty."""
import csv
import json
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT / 'docs/materials/scripts/glass-optic-smoke-lib.sh'
COST = ROOT / 'docs/materials/scripts/noise-layers-cost.sh'


class ClockSamplerTests(unittest.TestCase):
    def capture(self, *, enabled=True, mode='normal', trace_rc=0, abort=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'runtime').mkdir()
            (root / 'bin').mkdir()
            smi = root / 'bin/nvidia-smi'
            smi.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys, time
root = pathlib.Path(os.environ['TEST_ROOT'])
(root / 'sampler.pid').write_text(str(os.getpid()))
(root / 'args.json').write_text(json.dumps(sys.argv[1:]))
if os.environ['SAMPLER_MODE'] == 'failed':
    sys.exit(4)
print('timestamp, clocks.current.graphics [MHz], clocks.current.memory [MHz], pstate, power.draw [W]', flush=True)
if os.environ['SAMPLER_MODE'] == 'empty':
    sys.exit(0)
if os.environ['SAMPLER_MODE'] == 'early':
    print('2026/10/11 10:00:00.000, 210 MHz, 405 MHz, P8, 14 W', flush=True)
    time.sleep(0.2)
    sys.exit(0)
while True:
    print('2026/10/11 10:00:00.000, 210 MHz, 405 MHz, P8, 14 W', flush=True)
    time.sleep(0.1)
''')
            smi.chmod(0o755)
            tracy = root / 'bin/tracy-capture'
            tracy.write_text('''#!/usr/bin/env python3
import os, pathlib, sys, time
root = pathlib.Path(os.environ['TEST_ROOT'])
(root / 'trace.pid').write_text(str(os.getpid()))
pathlib.Path(sys.argv[sys.argv.index('-o') + 1]).write_text('trace')
time.sleep(0.4)
sys.exit(int(os.environ['TRACE_RC']))
''')
            tracy.chmod(0o755)
            script = ('magick() { :; }; source "$1"\n'
                      'TOOLS="$TEST_ROOT/bin"; TRACY_PORT=20000\n'
                      'capture_bg case\n' + ('kill -TERM $$\n' if abort == 'term' else
                                               'exit 23\n' if abort else 'capture_wait\n'))
            child = subprocess.Popen(
                ['bash', '-eu', '-c', script, 'clock-test', str(LIB)], cwd=ROOT,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
                env={**os.environ, 'PATH': f'{root}/bin:{os.environ["PATH"]}',
                     'OUT': str(root / 'out'), 'XDG_RUNTIME_DIR': str(root / 'runtime'),
                     'NIRI_MATERIAL_WORK_ROOT': str(root), 'CAPTURE_META': ':',
                     'CAPTURE_GPU_CLOCKS': '1' if enabled else '0', 'TEST_ROOT': str(root),
                     'SAMPLER_MODE': mode, 'TRACE_RC': str(trace_rc)},
            )
            try:
                stdout, stderr = child.communicate(timeout=5)
                for filename in ('sampler.pid', 'trace.pid'):
                    if (root / filename).exists():
                        with self.assertRaises(ProcessLookupError, msg=f'{filename} survived capture'):
                            os.kill(int((root / filename).read_text()), 0)
                samples = root / 'out/case.gpu-clocks.csv'
                args = root / 'args.json'
                return child.returncode, stderr, samples.read_text() if samples.exists() else '', json.loads(args.read_text()) if args.exists() else None
            finally:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait()

    def test_capture_records_100_ms_samples_and_reaps_sampler(self):
        rc, stderr, samples, args = self.capture()
        self.assertEqual(rc, 0, stderr)
        self.assertIn('210 MHz, 405 MHz, P8', samples)
        self.assertEqual(args, ['--query-gpu=timestamp,clocks.gr,clocks.mem,pstate,power.draw', '--format=csv', '-lms', '100'])

    def test_unsampled_capture_never_launches_sampler(self):
        rc, stderr, samples, args = self.capture(enabled=False)
        self.assertEqual(rc, 0, stderr)
        self.assertEqual(samples, '')
        self.assertIsNone(args)

    def test_failed_or_empty_sampler_refuses_capture(self):
        for mode in ('failed', 'empty', 'early'):
            with self.subTest(mode=mode):
                rc, stderr, _, _ = self.capture(mode=mode)
                self.assertNotEqual(rc, 0)
                self.assertIn('GPU clock sampler', stderr)

    def test_trace_failure_and_early_exit_reap_sampler(self):
        for trace_rc, abort, want in ((7, False, 1), (0, True, 23), (0, 'term', 143)):
            with self.subTest(trace_rc=trace_rc, abort=abort):
                rc, stderr, _, _ = self.capture(trace_rc=trace_rc, abort=abort)
                self.assertEqual(rc, want, stderr)


class ClockAnchorTests(unittest.TestCase):
    def test_anchor_waits_for_config_loaded_after_async_action_reply(self):
        source = COST.read_text()
        match = re.search(r'^reload_clock_anchor\(\) \{.*?^}', source, re.S | re.M)
        self.assertIsNotNone(match, 'missing reload completion anchor')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            server = socket.socket(socket.AF_UNIX)
            server.bind(str(root / 'niri.sock'))
            server.listen()
            observed = []

            def serve():
                with server, server.accept()[0] as events:
                    stream = events.makefile('r')
                    observed.append(json.loads(stream.readline()))
                    events.sendall(b'{"Ok":"Handled"}\n{"WorkspacesChanged":{"workspaces":[]}}\n'
                                   b'{"WindowsChanged":{"windows":[]}}\n{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":["English"],"current_idx":0}}}\n'
                                   b'{"OverviewOpenedOrClosed":{"is_open":false}}\n{"ConfigLoaded":{"failed":true}}\n{"CastsChanged":{"casts":[]}}\n')
                    with server.accept()[0] as action:
                        observed.append(json.loads(action.makefile('r').readline()))
                        action.sendall(b'{"Ok":"Handled"}\n')
                        time.sleep(0.05)
                        observed.append(time.time_ns())
                        events.sendall(b'{"ConfigLoaded":{"failed":false}}\n')

            thread = threading.Thread(target=serve, daemon=True)
            thread.start()
            try:
                result = subprocess.run(['bash', '-eu', '-c', match.group() +
                                         '\nOUT=$1; NIRI_SOCKET=$2; reload_clock_anchor "$OUT/case.kdl" case',
                                         'anchor-test', directory, str(root / 'niri.sock')],
                                        text=True, capture_output=True, timeout=5)
                self.assertEqual(result.returncode, 0, result.stderr)
                thread.join(timeout=1)
                self.assertFalse(thread.is_alive())
                lo, hi = map(int, (root / 'case.clock-anchor.txt').read_text().split())
                self.assertEqual(observed[:2], ['EventStream', {'Action': {'LoadConfigFile': {'path': str(root / 'case.kdl')}}}])
                self.assertLessEqual(lo, observed[2])
                self.assertGreaterEqual(hi, observed[2])
            finally:
                server.close()


class ClockAlignmentTests(unittest.TestCase):
    def report(self, *, sampled=True, empty=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for suffix, header, rows in (
                ('csv', ['name', 'ns_since_start', 'exec_time_ns'], [
                    ('State::reload_config', 2_000_000_000, 5),
                    ('EffectBuffer::sharp_damage', 2_050_000_000, 1),
                    ('State::reload_config', 3_000_000_000, 5)]),
                ('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'], [
                    ('MaterialRenderElement::draw', 1_950_000_000, 30_000),
                    ('MaterialRenderElement::draw', 2_050_000_000, 230_000)]),
            ):
                with (root / f'none.{suffix}').open('w') as file:
                    writer = csv.writer(file)
                    writer.writerow(header)
                    writer.writerows(rows)
            if sampled:
                (root / 'none.clock-anchor.txt').write_text('1791712800000000000 1791712800020000000\n')
                (root / 'none.gpu-clocks.csv').write_text(
                    'timestamp, clocks.current.graphics [MHz], clocks.current.memory [MHz], pstate, power.draw [W]\n' +
                    ('' if empty else '2026/10/11 09:59:59.900, 1680 MHz, 7001 MHz, P0, 40 W\n'
                     '2026/10/11 10:00:00.000, 210 MHz, 405 MHz, P8, 14 W\n'
                     '2026/10/11 10:00:00.100, 210 MHz, 405 MHz, P8, 14 W\n'))
            source = COST.read_text().split("<<'PYREPORT' >> \"$OUT/metrics.txt\"\n", 1)[1].split('\nPYREPORT', 1)[0]
            result = subprocess.run(['python3', '-c', source, directory, 'none', '1', '100000'],
                                    text=True, capture_output=True, timeout=5, env={**os.environ, 'TZ': 'UTC'})
            joined = root / 'none.draw-clocks.csv'
            rows = []
            if joined.exists():
                with joined.open() as file:
                    rows = list(csv.DictReader(file))
            return result, rows

    def test_first_draw_wall_time_and_clock_join_retain_anchor_width(self):
        result, rows = self.report()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]['wall_earliest_ns'], '1791712799950000000')
        self.assertEqual(rows[0]['wall_latest_ns'], '1791712799970000000')
        self.assertEqual(rows[0]['graphics_mhz'], '1680')
        self.assertEqual(rows[1]['graphics_mhz'], '210')
        self.assertEqual(rows[0]['gpu_ms'], '0.030000')
        self.assertEqual(rows[1]['gpu_ms'], '0.230000')

    def test_report_without_clock_sampling_keeps_original_metrics(self):
        result, rows = self.report(sampled=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('none_MaterialRenderElement::draw_median_ms=0.230000', result.stdout)
        self.assertEqual(rows, [])

    def test_empty_clock_trace_cannot_produce_a_cost_verdict(self):
        result, _ = self.report(empty=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('clock', result.stderr.lower())


class NoiseCostPilotTests(unittest.TestCase):
    def test_one_case_reaches_its_verdict_without_running_other_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'runtime').mkdir()
            (root / 'noise-layers-cost.sh').write_text(COST.read_text())
            cpu = [('State::reload_config', t * 10**9, 5) for t in (2, 3, 4, 5)]
            cpu += [('EffectBuffer::sharp_damage', t * 10**9 + 100, 1) for t in (2, 3, 4)]
            gpu = [('MaterialRenderElement::draw', t * 10**9 + 200, 230_000) for t in (2, 3, 4)]
            for name, header, rows in [('cpu.csv', ['name', 'ns_since_start', 'exec_time_ns'], cpu),
                                       ('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'], gpu)]:
                with (root / name).open('w') as file:
                    writer = csv.writer(file)
                    writer.writerow(header)
                    writer.writerows(rows)
            # Replace the expensive host/build boundary; run the real config and report code.
            (root / 'glass-optic-smoke-lib.sh').write_text('''
magick() { :; }
source "$REAL_LIB"
build_binaries() { NIRI=/bin/true; NIRI_TRACY=/bin/true; }
await_load() { :; }
capture_preflight() { :; }
capture_identity() { :; }
reserve_tracy_port() { :; }
tools_ready() { :; }
calibrate_probe_rect() { :; }
start_nested() { :; }
capture_bg() { :; }
capture_ready() { :; }
start_wall() { :; }
spawn_probe() { :; }
probe_rect() { PW=400; PH=600; }
reload_marker() { :; }
stop_walls() { :; }
wait_for_wall_removal() { :; }
sleep_until() { :; }
capture_wait() { :; }
wall_count() { echo 1; }
stop_nested() { :; }
count_last20() { cp "$TEST_ROOT/cpu.csv" "$OUT/$1.csv"; echo 20; }
csvexport() { cp "$TEST_ROOT/gpu.csv" "$3"; }
sleep() { :; }
''')
            env = {**os.environ, 'REAL_LIB': str(LIB), 'TEST_ROOT': directory,
                   'OUT': str(root / 'out'), 'XDG_RUNTIME_DIR': str(root / 'runtime'),
                   'NIRI_MATERIAL_WORK_ROOT': directory, 'CAPTURE_META': ':',
                   'NOISE_LAYERS_COST_PILOT': '1', 'NOISE_LAYERS_COST_AB': '', 'CAPTURE_GPU_CLOCKS': '0'}
            result = subprocess.run(['bash', str(root / 'noise-layers-cost.sh'), 'none'], cwd=ROOT,
                                    env=env, text=True, capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('PASS: noise layers cost (pilot=1)', result.stdout)
            self.assertIn('none_MaterialRenderElement::draw_gpu_count=3', (root / 'out/metrics.txt').read_text())
            self.assertEqual(sorted(p.name for p in (root / 'out').glob('*.kdl')), ['none.kdl'])
            self.assertIn('noise 0', (root / 'out/none.kdl').read_text())
            check = subprocess.run(['sha256sum', '-c', '--quiet', 'SHA256SUMS'], cwd=root / 'out', capture_output=True)
            self.assertEqual(check.returncode, 0, check.stderr)
