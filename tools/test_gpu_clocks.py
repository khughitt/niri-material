"""The capture owns its clock sampler, and wall-time joins retain uncertainty."""
import csv
import datetime
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

    def test_trace_failure_is_reported_even_when_the_sampler_also_exited(self):
        rc, stderr, _, _ = self.capture(mode='early', trace_rc=7)
        self.assertEqual(rc, 1, stderr)
        self.assertIn('tracy-capture exited 7', stderr)
        self.assertIn('GPU clock sampler had also exited', stderr)


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


def clock_row(tenths, graphics, pstate, power):
    """One nvidia-smi row, `tenths` of a second after 2026-10-11 10:00:00 UTC (the fixture anchor)."""
    stamp = datetime.datetime(2026, 10, 11, 10) + datetime.timedelta(seconds=tenths / 10)
    return (f'{stamp:%Y/%m/%d %H:%M:%S}.{stamp.microsecond // 1000:03d}, {graphics} MHz, '
            f'{405 if pstate == "P8" else 7001} MHz, {pstate}, {power} W\n')


CLOCK_HEADER = 'timestamp, clocks.current.graphics [MHz], clocks.current.memory [MHz], pstate, power.draw [W]\n'
REPORT_SOURCE = COST.read_text().split("<<'PYREPORT' >> \"$OUT/metrics.txt\"\n", 1)[1].split('\nPYREPORT', 1)[0]


class ClockAlignmentTests(unittest.TestCase):
    # Trace time 2 s is wall 10:00:00.000-.020 (the anchor). The window is one
    # step, 2-3 s. Readings are P0 before the window, P8 at 225 MHz for its
    # first half second (a lagged reading), P8 at the 210 MHz idle clock after,
    # with one 270 MHz reading at 10:00:01.300.
    SAMPLES = ([clock_row(-1, 1815, 'P0', 43.1)] + [clock_row(t, 225, 'P8', 14.4) for t in range(0, 5)] +
               [clock_row(t, 270 if t == 13 else 210, 'P8', 22.5 if t == 13 else 10.5) for t in range(5, 22)])
    DRAWS = [(1_950_000_000, 30_000), (2_050_000_000, 270_000), (2_250_000_000, 290_000),
             (2_550_000_000, 330_000), (2_850_000_000, 310_000), (3_500_000_000, 320_000)]

    def report(self, *, sampled=True, samples=None, minimum_idle=0):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for suffix, header, rows in (
                ('csv', ['name', 'ns_since_start', 'exec_time_ns'], [
                    ('State::reload_config', 2_000_000_000, 5),
                    ('EffectBuffer::sharp_damage', 2_050_000_000, 1),
                    ('State::reload_config', 3_000_000_000, 5)]),
                ('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'],
                 [('MaterialRenderElement::draw', t, d) for t, d in self.DRAWS]),
            ):
                with (root / f'none.{suffix}').open('w') as file:
                    writer = csv.writer(file)
                    writer.writerow(header)
                    writer.writerows(rows)
            if samples is not None:
                (root / 'none.clock-anchor.txt').write_text('1791712800000000000 1791712800020000000\n')
                (root / 'none.gpu-clocks.csv').write_text(CLOCK_HEADER + ''.join(samples))
            result = subprocess.run(['python3', '-c', REPORT_SOURCE, directory, 'none', '1', '100000',
                                     '1' if sampled else '0', str(minimum_idle)],
                                    text=True, capture_output=True, timeout=5, env={**os.environ, 'TZ': 'UTC'})
            joined = root / 'none.draw-clocks.csv'
            rows = []
            if joined.exists():
                with joined.open() as file:
                    rows = list(csv.DictReader(file))
            metrics = dict(line.split('=', 1) for line in result.stdout.splitlines())
            return result, rows, metrics

    def test_first_draw_wall_time_and_clock_join_retain_anchor_width(self):
        result, rows, metrics = self.report(samples=self.SAMPLES)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(rows), len(self.DRAWS))
        self.assertEqual(rows[0]['wall_earliest_ns'], '1791712799950000000')
        self.assertEqual(rows[0]['wall_latest_ns'], '1791712799970000000')
        self.assertEqual([r['graphics_mhz'] for r in rows], ['1815', '225', '225', '210', '210', '210'])
        self.assertEqual(rows[0]['gpu_ms'], '0.030000')
        self.assertEqual(rows[1]['gpu_ms'], '0.270000')
        self.assertEqual(metrics['none_first_material_draw_wall_earliest_ns'], '1791712799950000000')

    def test_rows_mark_the_stimulus_window(self):
        _, rows, _ = self.report(samples=self.SAMPLES)
        self.assertEqual([r['in_window'] for r in rows], ['0', '1', '1', '1', '1', '0'])

    def test_idle_clock_draws_hold_the_idle_clock_through_the_reading_lag(self):
        # Idle: P8 at the trace's lowest P8 clock from the joined reading through
        # the first reading 0.5 s after the draw. The draw at 2.25 s joins a
        # lagged 225 MHz reading; 2.85 s has the 270 MHz reading inside its lag.
        _, rows, metrics = self.report(samples=self.SAMPLES)
        self.assertEqual([r['idle_clock'] for r in rows], ['0', '0', '0', '1', '0', '1'])
        self.assertEqual(metrics['none_idle_graphics_mhz'], '210')
        self.assertEqual(metrics['none_MaterialRenderElement::draw_idle_clock_count'], '1')
        self.assertEqual(metrics['none_MaterialRenderElement::draw_idle_clock_median_ms'], '0.330000')
        self.assertEqual(metrics['none_idle_clock_power_median_w'], '10.50')
        # All in-window draws, as before: the key earlier comparisons pinned.
        self.assertEqual(metrics['none_MaterialRenderElement::draw_median_ms'], '0.300000')

    def test_too_few_idle_clock_draws_refuse_the_report(self):
        result, _, _ = self.report(samples=self.SAMPLES, minimum_idle=2)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('1 idle-clock material draws', result.stderr)

    def test_a_pilot_without_idle_clock_draws_reports_none(self):
        samples = [clock_row(t, 1815, 'P0', 43.1) for t in range(-1, 15)] + [clock_row(t, 210, 'P8', 10.5) for t in range(15, 22)]
        result, _, metrics = self.report(samples=samples)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(metrics['none_MaterialRenderElement::draw_idle_clock_count'], '0')
        self.assertNotIn('none_MaterialRenderElement::draw_idle_clock_median_ms', metrics)
        self.assertNotIn('none_idle_clock_power_median_w', metrics)

    def test_a_trace_ending_inside_a_draws_reading_lag_refuses(self):
        result, _, _ = self.report(samples=self.SAMPLES[:-1])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('GPU clock trace does not cover', result.stderr)

    def test_report_without_clock_sampling_keeps_original_metrics(self):
        result, rows, metrics = self.report(sampled=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(metrics['none_MaterialRenderElement::draw_median_ms'], '0.300000')
        self.assertNotIn('none_MaterialRenderElement::draw_idle_clock_count', metrics)
        self.assertEqual(rows, [])

    def test_sampled_report_requires_the_clock_trace(self):
        result, _, _ = self.report(samples=None)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('none.gpu-clocks.csv', result.stderr)

    def test_empty_clock_trace_cannot_produce_a_cost_verdict(self):
        result, _, _ = self.report(samples=[])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('clock', result.stderr.lower())


class NoiseCostPilotTests(unittest.TestCase):
    # Replace the expensive host/build boundary; run the real config, report and A/B code.
    STUB_LIB = '''
magick() { :; }
source "$REAL_LIB"
build_binaries() { touch "$TEST_ROOT/built"; NIRI=/bin/true; NIRI_TRACY=/bin/true; }
await_load() { :; }
capture_preflight() { :; }
capture_identity() { printf '%s\\n' "$@" > "$TEST_ROOT/identity-args"; }
reserve_tracy_port() { :; }
tools_ready() { :; }
calibrate_probe_rect() { :; }
start_nested() { :; }
capture_bg() {   # a case's own clocks-<case>.csv, else the shared trace
    local f=$TEST_ROOT/clocks-$1.csv; [ -e "$f" ] || f=$TEST_ROOT/clocks.csv
    [ "$CAPTURE_GPU_CLOCKS" = 0 ] || cp "$f" "$OUT/$1.gpu-clocks.csv"
}
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
csvexport() { local f; f=$TEST_ROOT/gpu-$(basename "$2" .tracy).csv; [ -e "$f" ] || f=$TEST_ROOT/gpu.csv; cp "$f" "$3"; }
sleep() { :; }
'''

    def run_cost(self, root, *args, pilot=True, ab='', clocks=None, prepare=None):
        steps = 3 if pilot else 20
        (root / 'runtime').mkdir()
        # The anchor connects to a live niri; the stub writes the bracket the fixture times assume.
        source = re.sub(r'^reload_clock_anchor\(\) \{.*?^}', 'reload_clock_anchor() { '
                        'echo "1791712800000000000 1791712800005000000" > "$OUT/$2.clock-anchor.txt"; }',
                        COST.read_text(), count=1, flags=re.S | re.M)
        (root / 'noise-layers-cost.sh').write_text(source)
        (root / 'glass-optic-smoke-lib.sh').write_text(self.STUB_LIB)
        cpu = [('State::reload_config', t * 10**9, 5) for t in range(2, steps + 3)]
        cpu += [('EffectBuffer::sharp_damage', t * 10**9 + 100, 1) for t in range(2, steps + 2)]
        gpu = [('MaterialRenderElement::draw', t * 10**9 + 200_000_000, 230_000) for t in range(2, steps + 2)]
        gpu += [('Grain::render', t * 10**9 + 250_000_000, 50_000) for t in range(2, steps + 2)]
        for name, header, rows in [('cpu.csv', ['name', 'ns_since_start', 'exec_time_ns'], cpu),
                                   ('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'], gpu)]:
            with (root / name).open('w') as file:
                writer = csv.writer(file)
                writer.writerow(header)
                writer.writerows(rows)
        (root / 'clocks.csv').write_text(CLOCK_HEADER + ''.join(clock_row(t, 210, 'P8', 10.5) for t in range(-1, steps * 10 + 10)))
        if prepare:
            prepare(root)
        env = {k: v for k, v in os.environ.items() if k != 'CAPTURE_GPU_CLOCKS'}
        env.update({'REAL_LIB': str(LIB), 'TEST_ROOT': str(root), 'TZ': 'UTC',
                    'OUT': str(root / 'out'), 'XDG_RUNTIME_DIR': str(root / 'runtime'),
                    'NIRI_MATERIAL_WORK_ROOT': str(root), 'CAPTURE_META': ':',
                    'NOISE_LAYERS_COST_PILOT': '1' if pilot else '0', 'NOISE_LAYERS_COST_AB': ab})
        if clocks is not None:
            env['CAPTURE_GPU_CLOCKS'] = clocks
        return subprocess.run(['bash', str(root / 'noise-layers-cost.sh'), *args], cwd=ROOT,
                              env=env, text=True, capture_output=True, timeout=20)

    def test_one_case_reaches_its_verdict_without_running_other_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, 'none', clocks='0')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('PASS: noise layers cost (pilot=1)', result.stdout)
            self.assertIn('none_MaterialRenderElement::draw_gpu_count=3', (root / 'out/metrics.txt').read_text())
            self.assertEqual(sorted(p.name for p in (root / 'out').glob('*.kdl')), ['none.kdl'])
            self.assertIn('noise 0', (root / 'out/none.kdl').read_text())
            check = subprocess.run(['sha256sum', '-c', '--quiet', 'SHA256SUMS'], cwd=root / 'out', capture_output=True)
            self.assertEqual(check.returncode, 0, check.stderr)

    def test_identity_records_the_selected_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, 'none', clocks='0')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('cases=none', (root / 'identity-args').read_text().splitlines())

    def test_cost_runs_sample_clocks_by_default(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, 'none')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / 'out/none.draw-clocks.csv').exists())
            self.assertIn('none_MaterialRenderElement::draw_idle_clock_count=3', (root / 'out/metrics.txt').read_text())

    def test_case_arguments_are_refused_before_the_build(self):
        for args, ab, message in ((('bogus',), '', 'unknown noise cost case: bogus'),
                                  (('none', 'one-fine-1'), '', 'expected at most one noise cost case'),
                                  (('none',), '/bin/true', 'cannot be combined with NOISE_LAYERS_COST_AB')):
            with self.subTest(args=args), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                result = self.run_cost(root, *args, ab=ab, clocks='0')
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)
                self.assertFalse((root / 'built').exists())

    def test_sampled_ab_run_joins_clocks_for_every_sub_run(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, pilot=False, ab='/bin/true')
            self.assertEqual(result.returncode, 0, result.stderr)
            metrics = (root / 'out/metrics.txt').read_text()
            subs = [f'{case}-{side}-{n}' for case in ('none', 'one-fine-1') for n in (1, 2) for side in ('a', 'b1')]
            for name in subs:
                with self.subTest(sub_run=name):
                    self.assertTrue((root / f'out/{name}.draw-clocks.csv').exists())
                    self.assertIn(f'{name}_MaterialRenderElement::draw_idle_clock_count=20', metrics)
                    self.assertIn(f'{name}_MaterialRenderElement::draw_idle_clock_median_ms=0.230000', metrics)
            self.assertEqual(len(list((root / 'out').glob('*.draw-clocks.csv'))), len(subs))

    @staticmethod
    def burst_case(root):
        # one-fine-1: two draws per step; the twenty draws in the first ten steps
        # run at P0 and cost 0.03 ms, the twenty after at the idle clock, 0.33 ms.
        rows = [('MaterialRenderElement::draw', t * 10**9 + offset, 30_000 if t < 12 else 330_000)
                for t in range(2, 22) for offset in (200_000_000, 600_000_000)]
        with (root / 'gpu-one-fine-1.csv').open('w') as file:
            writer = csv.writer(file)
            writer.writerow(['name', 'Time from start of program', 'GPU execution time'])
            writer.writerows(rows)
        (root / 'clocks-one-fine-1.csv').write_text(CLOCK_HEADER + ''.join(
            clock_row(t, 1815, 'P0', 43.1) if t <= 100 else clock_row(t, 210, 'P8', 10.5) for t in range(-1, 210)))

    def test_full_run_reports_idle_clock_deltas_over_none(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, pilot=False, prepare=self.burst_case)
            self.assertEqual(result.returncode, 0, result.stderr)
            metrics = (root / 'out/metrics.txt').read_text()
            # Over none's 0.23 ms on 240000 px: all draws (median 0.18 ms), idle-clock draws (0.33 ms).
            self.assertIn('one-fine-1_MaterialRenderElement::draw_idle_clock_count=20', metrics)
            self.assertIn('one-fine-1_material_ns_per_px_over_none=-0.2083', metrics)
            self.assertIn('one-fine-1_material_idle_clock_ns_per_px_over_none=0.4167', metrics)

    def test_full_run_refuses_cases_idling_at_different_clocks(self):
        def prepare(root):
            (root / 'clocks-backdrop-fine-8.csv').write_text(CLOCK_HEADER + ''.join(
                clock_row(t, 225, 'P8', 14.4) for t in range(-1, 210)))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, pilot=False, prepare=prepare)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("cases idled at different graphics clocks: ['210', '225'] MHz", result.stderr)

    def test_an_empty_case_name_is_refused_by_name(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_cost(root, '', clocks='0')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('unknown noise cost case: ', result.stderr)
            self.assertNotIn('bad array subscript', result.stderr)
            self.assertFalse((root / 'built').exists())
