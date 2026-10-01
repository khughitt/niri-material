"""Offline checks for the sustained optic capture verdict."""

import csv
import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.optic_settling import FAMILIES, analyze_run, check_window, config_identity, parse_edges


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
        (self.case / 'case.kdl').write_text('valid config')
        self.manifest = {
            'schema': 2, 'mode': 'pilot', 'lane': 'headless', 'source_commit': 'a' * 40,
            'binary_sha256': hashlib.sha256(b'identified binary').hexdigest(),
            'cases': [{'name': 'active-idle-resume', 'family': 'active-idle-resume',
                       'lane': 'headless', 'config_sha256': config_identity(self.case),
                       'repetitions': 1, 'hold_ns': 5 * S, 'required': True,
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
        # Trace time is monotonic time minus 1000 s.
        self.observation = {
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
        self.assertEqual(result['verdict'], 'passed')
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
