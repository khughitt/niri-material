"""Offline checks for the sustained optic capture verdict."""

import csv
import hashlib
import json
import io
import os
import signal
import subprocess
import tempfile
import time
import unittest
from contextlib import redirect_stdout
from pathlib import Path

from tools.optic_settling import FAMILIES, analyze_run, check_window, config_identity, main, parse_edges


class EdgeTests(unittest.TestCase):
    def test_pause_resume_preserve_logical_time(self):
        rows = [
            {'MessageName': 'OpticTimeline active=0 real_ns=100 logical_ns=110', 'total_ns': '10'},
            {'MessageName': 'OpticTimeline active=1 real_ns=900 logical_ns=110', 'total_ns': '810'},
        ]
        edges = parse_edges(rows)
        self.assertEqual([e['logical_ns'] for e in edges], [110, 110])
        self.assertEqual([e['trace_ns'] for e in edges], [10, 810])
        for bad in (rows[:1] + rows[:1], rows[1:] + rows[:1],
                    [{'MessageName': 'OpticTimeline active=2 real_ns=100 logical_ns=110', 'total_ns': '10'}],
                    [{'MessageName': rows[0]['MessageName'], 'total_ns': '-1'}]):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                parse_edges(bad)

    def test_window_counts_half_open_intervals(self):
        check_window([], [], 20, 800, 0, 0)
        check_window([19, 800], [19, 800], 20, 800, 0, 0)
        with self.assertRaises(ValueError):
            check_window([30], [], 20, 800, 0, 0)
        with self.assertRaises(ValueError):
            check_window([], [30], 20, 800, 0, 0)
        with self.assertRaises(ValueError):
            check_window([], [], 800, 20, 0, 0)


S = 1_000_000_000


class RunTests(unittest.TestCase):
    """A synthetic 4 Hz Aurora case: active to 10 s, settled to 20 s, active to 24 s."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.run = Path(self.tmp.name)
        self.case = self.run / 'active-idle-resume'
        self.case.mkdir()
        (self.run / 'binary').write_bytes(b'identified binary')
        (self.run / 'niri.log').write_text('INFO niri: listening on Wayland socket\n')
        (self.case / 'case.kdl').write_text('valid config')
        self.manifest = {
            'schema': 2, 'mode': 'pilot', 'lane': 'headless', 'source_commit': 'a' * 40,
            'binary_sha256': hashlib.sha256(b'identified binary').hexdigest(),
            'cases': [{'name': 'active-idle-resume', 'family': 'active-idle-resume',
                       'lane': 'headless', 'config_sha256': config_identity(self.case),
                       'repetitions': 1, 'hold_ns': 5 * S, 'required': True, 'capture_s': 24,
                       'edges': [0, 1],
                       'segments': [{'state': 'active', 'min_hz': 4, 'max_hz': 4},
                                    {'state': 'settled'},
                                    {'state': 'active', 'min_hz': 4, 'max_hz': 4}],
                       'stimuli': [{'label': 'damage', 'min_redraws': 1, 'min_draws': 1}],
                       'pixels': [{'before': 'before.rgb', 'after': 'after.rgb', 'expect': 'equal'}]}],
        }
        self.manifest['cases'] += [
            {'name': family, 'family': family, 'lane': 'dedicated', 'required': False}
            for family in FAMILIES if family != 'active-idle-resume'
        ]
        # Trace time is monotonic time minus 1000 s; the capture connected at
        # trace time 0 for 24 s.
        self.observation = {
            'capture_start_mono_ns': 1000 * S,
            'journal': [{'label': 'damage', 'start_mono_ns': 1000 * S + 14 * S,
                         'end_mono_ns': 1000 * S + 15 * S}],
            'topology': ['headless-1'],
        }
        self.save()
        (self.case / 'export.json').write_text(json.dumps({'messages': True, 'cpu': True, 'gpu': True}))
        (self.case / 'before.rgb').write_bytes(b'pixels')
        (self.case / 'after.rgb').write_bytes(b'pixels')
        self.messages = [
            ['OpticTimeline active=0 real_ns=1010000000000 logical_ns=10000000000', 10 * S],
            ['OpticTimeline active=1 real_ns=1020000000000 logical_ns=10000000000', 20 * S],
        ]
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], self.messages)
        ticks = [t * S // 4 for t in range(0, 40)] + [t * S // 4 for t in range(80, 96)]
        self.cpu = [['Niri::redraw', t, 1000] for t in ticks]
        self.cpu += [['Niri::redraw', 14 * S + S // 2, 1000]]          # the stimulus
        self.cpu += [['Niri::refresh_idle_inhibit', t * S, 1000] for t in range(0, 25)]
        self.cpu += [['Niri::notify_activity', 4 * S, 1000], ['Niri::notify_activity', 20 * S, 1000]]
        self.gpu = [['MaterialRenderElement::draw', t, 1000] for t in ticks]
        self.gpu += [['MaterialRenderElement::draw', 14 * S + S // 2, 1000]]
        self.write_tables()

    def write_csv(self, name, header, rows):
        with (self.case / name).open('w', newline='') as stream:
            writer = csv.writer(stream)
            writer.writerow(header)
            writer.writerows(rows)

    def write_tables(self):
        self.write_csv('cpu.csv', ['name', 'ns_since_start', 'exec_time_ns'], self.cpu)
        self.write_csv('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'], self.gpu)

    def save(self):
        (self.run / 'manifest.json').write_text(json.dumps(self.manifest))
        (self.case / 'observation.json').write_text(json.dumps(self.observation))

    def rejects(self, message=None):
        with self.assertRaises(ValueError) as caught:
            analyze_run(self.run)
        if message:
            self.assertIn(message, str(caught.exception))

    def test_complete_pilot_passes(self):
        result = analyze_run(self.run)
        self.assertEqual(result['verdict'], 'lane-passed')
        case = result['cases'][0]
        self.assertEqual([s['state'] for s in case['segments']], ['active', 'settled', 'active'])
        self.assertEqual(case['segments'][1]['redraws'], 1)
        self.assertEqual({r['verdict'] for r in result['cases'][1:]}, {'unverified'})

    def test_rejects_absent_edges_headers_export_and_controls(self):
        for name, mutate in (
            ('edges', lambda: (self.case / 'messages.csv').write_text('MessageName,total_ns\n')),
            ('headers', lambda: (self.case / 'gpu.csv').write_text('name,bad,time\n')),
            ('export', lambda: (self.case / 'export.json').write_text('{"messages": false, "cpu": true, "gpu": true}')),
            ('cpu control', lambda: (self.case / 'cpu.csv').write_text(
                'name,ns_since_start,exec_time_ns\n' + ''.join(f'Niri::refresh_idle_inhibit,{t * S},1\n' for t in range(25)))),
            ('gpu control', lambda: (self.case / 'gpu.csv').write_text(
                'name,Time from start of program,GPU execution time\nrender,1,1\n')),
        ):
            with self.subTest(name=name):
                saved = {p: p.read_bytes() for p in self.case.iterdir() if p.is_file()}
                mutate()
                self.rejects()
                for p, data in saved.items():
                    p.write_bytes(data)

    def test_rejects_missing_duplicate_short_and_false_hardware_cases(self):
        for name, mutate in (
            ('missing', lambda: self.manifest['cases'].pop(0)),
            ('duplicate', lambda: self.manifest['cases'].append(dict(self.manifest['cases'][0]))),
            ('short', lambda: self.manifest['cases'][0].update(hold_ns=20 * S)),
            ('tty', lambda: self.manifest['cases'][0].update(lane='dedicated')),
            ('edge plan', lambda: self.manifest['cases'][0].update(edges=[0])),
            ('pixels', lambda: self.manifest['cases'][0]['pixels'][0].update(expect='different')),
            ('cadence', lambda: self.manifest['cases'][0]['segments'][0].update(min_hz=8, max_hz=8)),
            ('stimulus effect', lambda: self.manifest['cases'][0]['stimuli'][0].update(min_redraws=2)),
            ('journal', lambda: self.observation['journal'].clear()),
        ):
            with self.subTest(name=name):
                old_manifest = json.loads(json.dumps(self.manifest))
                old_observation = json.loads(json.dumps(self.observation))
                mutate(); self.save()
                self.rejects()
                self.manifest = old_manifest; self.observation = old_observation; self.save()

    def test_bounds_the_edge_flush_and_settled_redraws(self):
        self.cpu += [['Niri::redraw', 10 * S + t * S // 10, 1000] for t in (1, 2)]
        self.write_tables()
        analyze_run(self.run)                       # two flush redraws are allowed
        self.cpu += [['Niri::redraw', 10 * S + 3 * S // 10, 1000]]
        self.write_tables()
        self.rejects('edge flush')                  # a third is not
        self.cpu.pop()
        self.cpu += [['Niri::redraw', 17 * S, 1000]]
        self.write_tables()
        self.rejects('while settled')               # nor one after the flush

    def test_stimulus_exempts_only_its_interval(self):
        self.cpu += [['Niri::redraw', 15 * S + S // 10, 1000]]
        self.write_tables()
        self.rejects('while settled')

    def test_rejects_heartbeat_gap(self):
        self.cpu = [row for row in self.cpu if not (row[0] == 'Niri::refresh_idle_inhibit' and 15 * S < row[1] < 19 * S)]
        self.write_tables()
        self.rejects('heartbeat')

    def test_journal_needs_consistent_alignment(self):
        self.messages[1][0] = 'OpticTimeline active=1 real_ns=1020100000000 logical_ns=10000000000'
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], self.messages)
        self.rejects('monotonic offset')


    def test_setup_segment_is_unchecked_only_before_a_pause(self):
        case = self.manifest['cases'][0]
        case['segments'][0] = {'state': 'setup'}
        self.cpu += [['Niri::redraw', 5 * S + k, 1000] for k in range(1, 40)]   # launch activity
        self.write_tables(); self.save()
        analyze_run(self.run)
        case['segments'][2] = {'state': 'setup'}
        self.save()
        self.rejects('setup')

    def test_development_subset_never_passes_as_a_pilot(self):
        self.manifest['development'] = True
        self.manifest['cases'].append({'name': 'extra', 'family': 'gate-policy', 'lane': 'headless',
                                       'selected': False, 'required': False})
        self.save()
        result = analyze_run(self.run)
        self.assertEqual(result['verdict'], 'development-passed')
        self.assertIn('not_run', {r['verdict'] for r in result['cases']})
        self.manifest['cases'][-1]['required'] = True
        self.save()
        self.rejects('not run')

    def test_trace_without_messages(self):
        case = self.manifest['cases'][0]
        case.update(edges=[], segments=[{'state': 'active', 'min_hz': 4, 'max_hz': 4}], stimuli=[])
        self.observation['journal'] = []
        self.save()
        (self.case / 'messages.csv').write_bytes(b'There are currently no messages!\n')
        self.cpu = [row for row in self.cpu if not (row[0] == 'Niri::redraw' and 10 * S <= row[1] < 20 * S)]
        self.cpu += [['Niri::redraw', t * S // 4, 1000] for t in range(40, 80)]
        self.gpu = [['MaterialRenderElement::draw', row[1], 1000] for row in self.cpu if row[0] == 'Niri::redraw']
        self.write_tables()
        analyze_run(self.run)
        (self.case / 'messages.csv').write_bytes(b'garbage\n')
        self.rejects()

    def test_unverified_lanes_carry_their_reason(self):
        self.manifest['cases'][1]['why'] = 'needs a real TTY'
        self.save()
        result = analyze_run(self.run)
        self.assertEqual(result['cases'][1], {'name': self.manifest['cases'][1]['name'],
                                              'family': self.manifest['cases'][1]['family'],
                                              'verdict': 'unverified', 'lane': 'dedicated',
                                              'why': 'needs a real TTY'})
        self.assertFalse(result['complete'])


    def settled_tail(self, trace_s, collect_redraws=1):
        """The review's I-1 case: active to 10 s, then settled to a declared 40 s
        capture end, whose final collect frame is journaled at 39 s."""
        case = self.manifest['cases'][0]
        case.update(edges=[0], segments=[{'state': 'active', 'min_hz': 4, 'max_hz': 4}, {'state': 'settled'}],
                    capture_s=40, stimuli=[{'label': 'collect', 'min_redraws': collect_redraws}])
        self.observation['journal'] = [{'label': 'collect', 'start_mono_ns': 1039 * S,
                                        'end_mono_ns': 1039 * S + 6 * S // 10}]
        self.save()
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], self.messages[:1])
        # Without the fix, 12-17 s alone is a 5 s quiet span that meets the hold.
        self.cpu = [row for row in self.cpu if row[0] != 'Niri::refresh_idle_inhibit' and row[1] < 14 * S]
        self.cpu += [['Niri::refresh_idle_inhibit', t * S, 1000] for t in range(0, trace_s + 1)]
        if trace_s >= 40:
            self.cpu += [['Niri::redraw', 39 * S + S // 5, 1000]]      # the collect frame
        self.gpu = [row for row in self.gpu if row[1] < 14 * S]
        self.write_tables()

    def test_rejects_a_trace_that_stops_before_the_declared_end(self):
        self.settled_tail(40)
        self.assertEqual(analyze_run(self.run)['cases'][0]['verdict'], 'passed')
        # Even where the collect frame demands no redraw (as the review's
        # probe did), the trace must reach the declared end.
        self.settled_tail(17, collect_redraws=0)
        self.rejects()
        self.settled_tail(17)
        with redirect_stdout(io.StringIO()):
            self.assertEqual(main([str(self.run)]), 1)
        self.assertEqual(json.loads((self.run / 'analysis.json').read_text())['verdict'], 'invalid')

    def test_collect_frame_must_redraw(self):
        self.settled_tail(40)
        self.cpu = [row for row in self.cpu if row[1] != 39 * S + S // 5]
        self.write_tables()
        self.rejects('collect produced too few redraws')

    def test_rejects_stimulus_after_the_trace(self):
        self.manifest['cases'][0]['stimuli'].append({'label': 'late'})
        self.observation['journal'].append({'label': 'late', 'start_mono_ns': 1030 * S,
                                            'end_mono_ns': 1031 * S})
        self.save()
        self.rejects('journaled after the trace ends')

    def test_rejects_short_trace_against_declared_capture(self):
        self.manifest['cases'][0]['capture_s'] = 25          # within the heartbeat bound
        self.save()
        analyze_run(self.run)
        self.manifest['cases'][0]['capture_s'] = 28
        self.save()
        self.rejects('before the declared capture end')
        self.manifest['cases'][0]['capture_s'] = 24
        self.observation.pop('capture_start_mono_ns')
        self.save()
        self.rejects('capture start')

    def test_rejects_a_panic_in_the_compositor_log(self):
        with (self.run / 'niri.log').open('a') as log:
            log.write("thread 'main' panicked at src/niri.rs:1:1:\n")
        self.rejects('panic')
        (self.run / 'niri.log').unlink()
        self.rejects('niri.log')

    def test_lane_verdict_exits_zero_and_lists_unverified_cases(self):
        self.manifest['mode'] = 'matrix'
        self.save()
        with redirect_stdout(io.StringIO()):
            self.assertEqual(main([str(self.run)]), 0)
        analysis = json.loads((self.run / 'analysis.json').read_text())
        self.assertEqual(analysis['verdict'], 'lane-passed')
        self.assertFalse(analysis['complete'])
        self.assertEqual(analysis['unverified'], [f for f in FAMILIES if f != 'active-idle-resume'])
        # A case of this lane that was not run is not a declared other lane.
        self.manifest['cases'].append({'name': 'extra', 'family': 'gate-policy', 'lane': 'headless',
                                       'selected': False, 'required': False})
        self.save()
        self.assertEqual(analyze_run(self.run)['verdict'], 'incomplete')
        with redirect_stdout(io.StringIO()):
            self.assertEqual(main([str(self.run)]), 1)

    def test_complete_lane_passes(self):
        template = self.manifest['cases'][0]
        self.manifest['mode'] = 'matrix'
        self.manifest['cases'] = [template]
        for family in FAMILIES[1:]:
            (self.run / family).symlink_to(self.case, target_is_directory=True)
            self.manifest['cases'].append(dict(json.loads(json.dumps(template)), name=family, family=family))
        self.save()
        result = analyze_run(self.run)
        self.assertEqual((result['verdict'], result['complete'], result['unverified']), ('passed', True, []))


class IdentityTests(unittest.TestCase):
    def test_run_identity_ignores_the_run_directory_only(self):
        from tools.optic_settling import run_config_identity
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            for run in ('a', 'b'):
                case = base / run / 'case'
                case.mkdir(parents=True)
                (case / 'case.kdl').write_text(f'spawn-at-startup "swaybg" "-i" "{base / run}/warm.png"\n')
                (case / 'live.kdl').write_text(f'anything {run}')
            self.assertNotEqual(config_identity(base / 'a/case'), config_identity(base / 'b/case'))
            self.assertEqual(run_config_identity(base / 'a/case'), run_config_identity(base / 'b/case'))
            (base / 'b/case/case.kdl').write_text('gaps 41\n')
            self.assertNotEqual(run_config_identity(base / 'a/case'), run_config_identity(base / 'b/case'))


class PrepareTests(unittest.TestCase):
    def test_prepare_validates_inventory_and_cleans_runtime(self):
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            bin_dir = base / 'bin'; bin_dir.mkdir()
            niri = bin_dir / 'niri'
            niri.write_text('#!/bin/sh\n[ "$1" = validate ]\n')
            magick = bin_dir / 'magick'
            magick.write_text('#!/bin/sh\nfor arg do :; done\nprintf image > "$arg"\n')
            niri.chmod(0o755); magick.chmod(0o755)
            source = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
            identity = dict(source_commit=source, features=['profile-with-tracy'],
                            binary_sha256=hashlib.sha256(niri.read_bytes()).hexdigest())
            Path(str(niri) + '.identity.json').write_text(json.dumps(identity))
            env = dict(os.environ, OUT=str(base / 'out'), NIRI_BIN=str(niri),
                       NIRI_MATERIAL_WORK_ROOT=str(base), XDG_RUNTIME_DIR=str(base),
                       CAPTURE_META='/bin/true', PATH=f'{bin_dir}:{os.environ["PATH"]}')
            script = root / 'docs/materials/scripts/optic-settling-smoke.sh'
            run = subprocess.run(['bash', str(script), 'prepare'], env=env, capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            manifest = json.loads((base / 'out/manifest.json').read_text())
            self.assertEqual({case['family'] for case in manifest['cases']}, set(FAMILIES))
            self.assertEqual(list(base.glob('gos.*')), [])
            again = subprocess.run(['bash', str(script), 'prepare'], env=env, capture_output=True, text=True)
            self.assertNotEqual(again.returncode, 0)
            self.assertIn('fresh directory', again.stderr)
            # A journal aligns through optic edges: an edge-free case journals nothing.
            for case in manifest['cases']:
                if case['lane'] == 'headless' and not case['edges']:
                    self.assertEqual(case['stimuli'], [], case['name'])

            # The collect frame proves the trace reached its declared end.
            for case in manifest['cases']:
                if case['lane'] == 'headless' and case['edges']:
                    self.assertIn({'label': 'collect', 'min_redraws': 1}, case['stimuli'], case['name'])
            # grim is a screencopy; a screencast needs a consumer this lane lacks.
            by_name = {case['name']: case for case in manifest['cases']}
            self.assertEqual(by_name['screencopy']['lane'], 'headless')
            self.assertEqual((by_name['screencast']['lane'], by_name['screencast']['why']),
                             ('screencast-consumer', 'no screencast consumer in the headless lane'))
            self.assertFalse(by_name['screencast']['required'])


# Stubs for every program the driver launches. Each long-lived one records
# "name pid" so a test can prove cleanup reaped it.
SERVE = """
import os, signal, socket, sys, time
os.chdir(os.environ['XDG_RUNTIME_DIR'])        # relative binds keep socket paths short
if sys.argv[1] == 'weston':
    names = [arg.split('=', 1)[1] for arg in sys.argv[2:] if arg.startswith('--socket=')]
else:
    names = ['stub-1', f'niri.stub-1.{os.getpid()}.sock']
sockets = []
for name in names:
    sockets.append(socket.socket(socket.AF_UNIX))
    sockets[-1].bind(name)
def stop(*_):
    for name in names:
        try:
            os.unlink(name)
        except OSError:
            pass
    sys.exit(0)
signal.signal(signal.SIGTERM, stop)
while True:
    time.sleep(60)
"""
STUBS = {
    'bin/weston': 'echo "weston $$" >> "$STUB_DIR/pids"\nexec python3 "$STUB_DIR/serve.py" weston "$@"\n',
    'bin/niri': (
        'case $1 in\n'
        '    validate) exit 0 ;;\n'
        '    msg) [ "$2" != -j ] || echo \'[{"id": 1, "app_id": "gos-probe", "is_focused": false},'
        ' {"id": 2, "app_id": "gos-other", "is_focused": true}]\'; exit 0 ;;\n'
        '    -c) echo "niri $$" >> "$STUB_DIR/pids"; exec python3 "$STUB_DIR/serve.py" niri ;;\n'
        'esac\nexit 2\n'),
    'bin/kitty': 'echo "kitty $$" >> "$STUB_DIR/pids"\nwhile [ "$1" != sh ]; do shift; done\nexec "$@"\n',
    'bin/magick': 'for arg do :; done\nprintf image > "$arg"\n',
    'bin/nvidia-smi': 'echo P8\n',
    'bin/ss': 'case "$*" in *established*) echo "ESTAB 0 0 127.0.0.1:1 127.0.0.1:2" ;; esac\n',
    'tools/tracy-capture': (
        'echo "tracy-capture $$" >> "$STUB_DIR/pids"\n'
        'while [ "$1" != -o ]; do shift; done\n: > "$2"\n'
        ': > "$STUB_DIR/capture-started"\nexec sleep "${STUB_CAPTURE_S:-0}"\n'),
    'tools/tracy-csvexport': (
        'echo "tracy-csvexport $$" >> "$STUB_DIR/pids"\n'
        ': > "$STUB_DIR/export-started"\nexec sleep "${STUB_EXPORT_S:-0}"\n'),
    'capture-meta': (
        'echo "$*" >> "$STUB_DIR/meta.log"\n'
        'if [ "$1" = preflight ] && [ -n "${STUB_REFUSE:-}" ]; then\n'
        '    echo \'{"refused": "load"}\' > "$2/capture.json"; exit 1\nfi\n'),
}


def alive(pid):
    try:
        return Path(f'/proc/{pid}/stat').read_text().split(') ')[-1][0] != 'Z'
    except OSError:
        return False


class DriverCleanupTests(unittest.TestCase):
    """The driver on stubs: a signal or a refusal must stop every child promptly,
    keep the partial evidence and release the capture lock."""

    BOUND_S = 5          # the old foreground waits deferred TERM for up to 609 s

    def setUp(self):
        self.root = Path(__file__).resolve().parents[1]
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name)
        self.stubs = self.base / 'stubs'
        for name, body in STUBS.items():
            path = self.stubs / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('#!/bin/sh\n' + body)
            path.chmod(0o755)
        (self.stubs / 'serve.py').write_text(SERVE)
        (self.stubs / 'pids').touch()
        self.runtime = self.base / 'rt'
        self.runtime.mkdir()
        niri = self.stubs / 'bin/niri'
        source = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        Path(str(niri) + '.identity.json').write_text(json.dumps(dict(
            source_commit=source, features=['profile-with-tracy'],
            binary_sha256=hashlib.sha256(niri.read_bytes()).hexdigest())))
        self.out = self.base / 'out'
        self.env = dict(os.environ, OUT=str(self.out), NIRI_BIN=str(niri), CAPTURE_TASK='material-test',
                        NIRI_MATERIAL_WORK_ROOT=str(self.base), XDG_RUNTIME_DIR=str(self.runtime),
                        CAPTURE_META=str(self.stubs / 'capture-meta'), STUB_DIR=str(self.stubs),
                        OPTIC_SETTLING_STUB_TOOLS=str(self.stubs / 'tools'),
                        PATH=f'{self.stubs / "bin"}:{os.environ["PATH"]}')
        self.addCleanup(self.kill_leftovers)

    def pids(self):
        return [int(line.split()[1]) for line in (self.stubs / 'pids').read_text().splitlines()]

    def kill_leftovers(self):
        for pid in self.pids():
            try:
                os.kill(pid, signal.SIGKILL)
            except OSError:
                pass

    def start(self, case, **env):
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        return subprocess.Popen(['bash', str(script), 'pilot'], cwd=self.root, start_new_session=True,
                                env=dict(self.env, CASES=case, **env),
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

    def await_marker(self, driver, marker, timeout_s):
        deadline = time.monotonic() + timeout_s
        while not (self.stubs / marker).exists():
            if driver.poll() is not None or time.monotonic() > deadline:
                driver.kill()
                self.fail(f'{marker} never appeared: {driver.communicate()[1]}')
            time.sleep(0.05)

    def terminate(self, driver):
        """TERM the driver alone (not its process group) and time its exit."""
        sent = time.monotonic()
        driver.send_signal(signal.SIGTERM)
        try:
            _, stderr = driver.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            driver.kill()
            raise
        return time.monotonic() - sent, stderr

    def assert_cleaned_up(self, case):
        self.assertTrue(self.pids())
        self.assertEqual([pid for pid in self.pids() if alive(pid)], [])
        self.assertEqual(list(self.runtime.glob('gos.*')), [])
        meta = (self.stubs / 'meta.log').read_text().splitlines()
        self.assertTrue(meta[-1].startswith('release '), meta)
        for kept in ('manifest.json', f'{case}/case.kdl', f'{case}/live.kdl', f'{case}/journal.tsv'):
            self.assertTrue((self.out / kept).is_file(), kept)
        sums = (self.out / 'SHA256SUMS').read_text()
        self.assertIn(' ./manifest.json\n', sums)
        self.assertIn(f' ./{case}/journal.tsv\n', sums)
        check = subprocess.run(['sha256sum', '-c', '--quiet', 'SHA256SUMS'], cwd=self.out,
                               capture_output=True, text=True)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

    def test_term_during_capture_stops_the_schedule_wait(self):
        # startup-no-input waits for its 10 s stimulus straight after the
        # capture connects; TERM must not wait that out.
        driver = self.start('startup-no-input', STUB_CAPTURE_S='600')
        self.await_marker(driver, 'capture-started', 60)
        time.sleep(0.5)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assertFalse((self.out / 'startup-no-input/export.json').exists())
        self.assert_cleaned_up('startup-no-input')

    def test_term_during_export(self):
        # gate-off drives only its keepalive (12 s), then exports.
        driver = self.start('gate-off', STUB_EXPORT_S='600')
        self.await_marker(driver, 'export-started', 60)
        time.sleep(0.2)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assertTrue((self.out / 'gate-off/capture.tracy').is_file())
        self.assert_cleaned_up('gate-off')

    def test_failed_preflight_releases_and_keeps_its_record(self):
        driver = self.start('gate-off', STUB_REFUSE='1')
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 1)
        self.assertIn('preflight refused', stderr)
        self.assertEqual(self.pids(), [])                     # nothing was launched
        self.assertEqual(list(self.runtime.glob('gos.*')), [])
        meta = (self.stubs / 'meta.log').read_text().splitlines()
        self.assertEqual([line.split()[0] for line in meta], ['preflight', 'release'])
        self.assertEqual(json.loads((self.out / 'capture.json').read_text()), {'refused': 'load'})
        self.assertTrue((self.out / 'SHA256SUMS').is_file())

    def test_stub_tools_need_a_stub_capture_record(self):
        env = dict(self.env)
        env.pop('CAPTURE_META')
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        run = subprocess.run(['bash', str(script), 'pilot'], cwd=self.root, env=env,
                             capture_output=True, text=True, timeout=60)
        self.assertNotEqual(run.returncode, 0)
        self.assertIn('OPTIC_SETTLING_STUB_TOOLS needs a stub CAPTURE_META', run.stderr)
