"""Offline checks for the sustained optic capture verdict."""

import csv
import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.optic_settling import FAMILIES, analyze_run, check_window, parse_edges


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


class RunTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.run = Path(self.tmp.name)
        self.case = self.run / 'active-idle-resume'
        self.case.mkdir()
        (self.run / 'binary').write_bytes(b'identified binary')
        (self.case / 'case.kdl').write_text('valid config')
        self.manifest = {
            'schema': 1, 'mode': 'pilot', 'lane': 'headless', 'source_commit': 'a' * 40,
            'binary_sha256': __import__('hashlib').sha256(b'identified binary').hexdigest(),
            'cases': [{'name': 'active-idle-resume', 'family': 'active-idle-resume',
                       'lane': 'headless', 'config_sha256': __import__('hashlib').sha256(b'valid config').hexdigest(),
                       'stimuli': ['pointer'], 'repetitions': 1, 'hold_ns': 5_000_000_000,
                       'required': True}],
        }
        self.manifest['cases'] += [
            {'name': family, 'family': family, 'lane': 'dedicated',
             'config_sha256': '0' * 64, 'stimuli': [], 'repetitions': 1,
             'hold_ns': 5_000_000_000, 'required': False}
            for family in FAMILIES if family != 'active-idle-resume'
        ]
        self.observation = {
            'trace_start_ns': 0, 'trace_end_ns': 11_000_000_000,
            'active_before': [0, 2_000_000_000], 'hold': [3_000_000_000, 8_000_000_000],
            'active_after': [9_000_000_000, 11_000_000_000],
            'expected_pixels': 'equal', 'before_rgb': 'before.rgb', 'after_rgb': 'after.rgb',
            'topology': ['headless-1'], 'stimuli_intervals': [],
        }
        (self.run / 'manifest.json').write_text(json.dumps(self.manifest))
        (self.case / 'observation.json').write_text(json.dumps(self.observation))
        (self.case / 'export.json').write_text(json.dumps({'messages': True, 'cpu': True, 'gpu': True}))
        (self.case / 'before.rgb').write_bytes(b'pixels')
        (self.case / 'after.rgb').write_bytes(b'pixels')
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], [
            ['OpticTimeline active=0 real_ns=3000000000 logical_ns=3000000000', 3_000_000_000],
            ['OpticTimeline active=1 real_ns=8000000000 logical_ns=3000000000', 8_000_000_000],
        ])
        cpu = [['Niri::redraw', 1_000_000_000, 1000], ['Niri::redraw', 9_500_000_000, 1000]]
        cpu += [['Niri::refresh_idle_inhibit', t * 1_000_000_000, 1000] for t in range(0, 12)]
        self.write_csv('cpu.csv', ['name', 'ns_since_start', 'exec_time_ns'], cpu)
        self.write_csv('gpu.csv', ['name', 'Time from start of program', 'GPU execution time'], [
            ['MaterialRenderElement::draw', 1_000_000_000, 1000],
            ['MaterialRenderElement::draw', 9_500_000_000, 1000],
        ])

    def write_csv(self, name, header, rows):
        with (self.case / name).open('w', newline='') as stream:
            writer = csv.writer(stream)
            writer.writerow(header)
            writer.writerows(rows)

    def save(self):
        (self.run / 'manifest.json').write_text(json.dumps(self.manifest))
        (self.case / 'observation.json').write_text(json.dumps(self.observation))

    def test_complete_pilot_passes(self):
        self.assertEqual(analyze_run(self.run)['verdict'], 'passed')

    def test_rejects_absent_edges_headers_export_and_controls(self):
        for name, mutate in (
            ('edges', lambda: (self.case / 'messages.csv').write_text('MessageName,total_ns\n')),
            ('headers', lambda: (self.case / 'gpu.csv').write_text('name,bad,time\n')),
            ('export', lambda: (self.case / 'export.json').write_text('{"messages": false, "cpu": true, "gpu": true}')),
            ('cpu control', lambda: (self.case / 'cpu.csv').write_text('name,ns_since_start,exec_time_ns\n')),
            ('gpu control', lambda: (self.case / 'gpu.csv').write_text('name,Time from start of program,GPU execution time\n')),
        ):
            with self.subTest(name=name):
                saved = {p: p.read_bytes() for p in self.case.iterdir() if p.is_file()}
                mutate()
                with self.assertRaises(ValueError):
                    analyze_run(self.run)
                for p, data in saved.items():
                    p.write_bytes(data)

    def test_rejects_missing_duplicate_short_and_false_hardware_cases(self):
        for name, mutate in (
            ('missing', lambda: self.manifest['cases'].append({**self.manifest['cases'][0], 'name': 'absent'})),
            ('duplicate', lambda: self.manifest['cases'].append(dict(self.manifest['cases'][0]))),
            ('short', lambda: self.observation['hold'].__setitem__(1, 4_000_000_000)),
            ('tty', lambda: self.manifest['cases'][0].update(lane='dedicated')),
        ):
            with self.subTest(name=name):
                old_manifest = json.loads(json.dumps(self.manifest))
                old_observation = json.loads(json.dumps(self.observation))
                mutate(); self.save()
                with self.assertRaises(ValueError):
                    analyze_run(self.run)
                self.manifest = old_manifest; self.observation = old_observation; self.save()

    def test_rejects_third_flush_and_heartbeat_gap(self):
        cpu = (self.case / 'cpu.csv').read_text()
        (self.case / 'cpu.csv').write_text(cpu + 'Niri::redraw,4000000000,1000\n'
                                            'Niri::redraw,5000000000,1000\n'
                                            'Niri::redraw,6000000000,1000\n')
        with self.assertRaises(ValueError):
            analyze_run(self.run)
        (self.case / 'cpu.csv').write_text(cpu.replace('Niri::refresh_idle_inhibit,5000000000,1000\n', ''))
        with self.assertRaises(ValueError):
            analyze_run(self.run)


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
