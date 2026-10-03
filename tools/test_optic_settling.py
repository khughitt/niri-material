"""Offline checks for the sustained optic capture verdict."""

import csv
import hashlib
import json
import io
import os
import shutil
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

    def test_edge_must_fall_inside_its_named_stimulus(self):
        # A second stimulus around the resume edge (trace 20 s); the damage
        # stimulus and its draws at 14.5 s stay where they are. It ends the
        # settled quiet span at 19.5 s (4.5 s after damage), so the declared
        # hold drops to 4 s for this case.
        self.manifest['cases'][0]['hold_ns'] = 4 * S
        self.manifest['cases'][0]['stimuli'].append({'label': 'resume'})
        self.observation['journal'].append({'label': 'resume', 'start_mono_ns': 1019 * S + S // 2,
                                            'end_mono_ns': 1020 * S + S // 2})
        self.manifest['cases'][0]['edge_in'] = [[1, 'damage']]
        self.save()
        self.rejects('edge 1 is not inside stimulus damage')
        self.manifest['cases'][0]['edge_in'] = [[1, 'resume']]
        self.save()
        analyze_run(self.run)
        self.manifest['cases'][0]['edge_in'] = [[2, 'resume']]   # no such edge
        self.save()
        self.rejects('edge 2 is not inside stimulus resume')

    def test_dedicated_topology_pins_output_mode_and_scale(self):
        self.manifest['lane'] = 'dedicated'
        self.manifest['drm'] = {'output': 'DP-1', 'mode': '3440x1440@59.999'}
        for index, case in enumerate(self.manifest['cases']):
            case['lane'] = 'dedicated' if index == 0 else 'headless'
        good = {'name': 'DP-1', 'mode': {'width': 3440, 'height': 1440, 'refresh_hz': 59.999}, 'scale': 1.0}
        self.observation['topology'] = [good]
        self.save()
        analyze_run(self.run)
        self.observation['topology'] = [dict(good, mode=dict(good['mode'], refresh_hz=59.995))]
        self.save()
        analyze_run(self.run)                                   # refresh within 0.01 Hz
        for output, message in (
            ('headless-1', 'must record name, mode and scale'),
            (dict(good, name='HDMI-A-1'), "output 'HDMI-A-1' is not the pinned DP-1"),
            (dict(good, scale=2.0), 'output scale 2.0, expected 1'),
            (dict(good, mode=dict(good['mode'], width=2560)), 'output mode 2560x1440, expected 3440x1440'),
            (dict(good, mode=dict(good['mode'], refresh_hz=59.97)), 'output refresh 59.97 Hz'),
            (dict(good, mode=None), 'output DP-1 has no current mode'),
        ):
            with self.subTest(output=output):
                self.observation['topology'] = [output]
                self.save()
                self.rejects(message)
        self.observation['topology'] = [good]
        for drm, message in ((None, 'names no drm output'),
                             ({'output': 'DP-1', 'mode': 'bogus'}, 'is not WIDTHxHEIGHT[@REFRESH]')):
            with self.subTest(drm=drm):
                self.manifest['drm'] = drm
                self.save()
                self.rejects(message)
        self.manifest['drm'] = {'output': 'DP-1', 'mode': '3440x1440'}   # no refresh pinned
        self.observation['topology'] = [dict(good, mode=dict(good['mode'], refresh_hz=75.0))]
        self.save()
        analyze_run(self.run)

    def cast(self, frames, stopped=True, samples=()):
        (self.case / 'cast-frames.tsv').write_text(''.join(f'{t}\n' for t in frames))
        (self.case / 'cast-summary.json').write_text(json.dumps({'stopped_by_signal': stopped}))
        for label, request, frame in samples:
            (self.case / f'{label}.raw.json').write_text(json.dumps(
                {'request_mono_ns': request, 'frame_mono_ns': frame, 'width': 1, 'height': 1}))

    def test_consumer_frames_and_samples(self):
        self.manifest['cases'][0]['consumer'] = {'frames_in': ['damage'], 'samples': ['damage']}
        self.save()
        inside = 1014 * S + S // 2
        self.cast([1013 * S], samples=[('damage', 1014 * S + S // 4, inside)])
        self.rejects('no cast frame inside stimulus damage')
        self.cast([inside], samples=[('damage', 1014 * S + S // 4, inside)])
        analyze_run(self.run)
        self.cast([inside], stopped=False, samples=[('damage', 1014 * S + S // 4, inside)])
        self.rejects('did not stop on a signal')

    def test_rejects_stale_missing_and_late_samples(self):
        self.manifest['cases'][0]['consumer'] = {'frames_in': [], 'samples': ['damage']}
        self.save()
        inside = 1014 * S + S // 2
        for label, request, frame in (
            ('stale', inside, inside),                         # the frame did not follow the request
            ('early request', 1013 * S, inside),               # armed before the stimulus began
            ('late frame', 1014 * S + S // 4, 1016 * S),       # answered after the window
        ):
            with self.subTest(label):
                self.cast([inside], samples=[('damage', request, frame)])
                self.rejects('sample damage is stale or outside its window')
        (self.case / 'damage.raw.json').unlink()
        self.rejects('damage.raw.json')

    def test_an_empty_consumer_is_still_checked(self):
        # A declared consumer, even one naming no stimulus, needs its signal stop.
        self.manifest['cases'][0]['consumer'] = {}
        self.save()
        self.rejects('cast-frames.tsv')
        self.cast([], stopped=False)
        self.rejects('did not stop on a signal')
        self.cast([])
        analyze_run(self.run)
        for consumer, message in (
            ([], 'consumer must be a mapping'),
            ({'frames_in': 'damage'}, 'consumer frames_in must be a list'),
            ({'samples': None}, 'consumer samples must be a list'),
        ):
            with self.subTest(consumer=consumer):
                self.manifest['cases'][0]['consumer'] = consumer
                self.save()
                self.rejects(message)

    def test_rejects_malformed_edge_in_entries(self):
        for entry in ([-1, 'damage'], [1], [1, 'damage', 2], 'damage',
                      [True, 'damage'], [1, 2], [1.5, 'damage']):
            with self.subTest(entry=entry):
                self.manifest['cases'][0]['edge_in'] = [entry]
                self.save()
                self.rejects('active-idle-resume: invalid edge_in entry')
        self.manifest['cases'][0]['edge_in'] = 'damage'
        self.save()
        self.rejects('active-idle-resume: edge_in must be a list')

    def test_a_sample_must_be_the_first_cast_frame_after_its_request(self):
        self.manifest['cases'][0]['consumer'] = {'frames_in': [], 'samples': ['damage']}
        self.save()
        request, frame = 1014 * S + S // 4, 1014 * S + S // 2
        self.cast([1014 * S + S // 3], samples=[('damage', request, frame)])
        self.rejects('active-idle-resume: sample damage is not a recorded cast frame')
        between = 1014 * S + S // 3
        self.cast([between, frame], samples=[('damage', request, frame)])
        self.rejects('active-idle-resume: sample damage is not the first cast frame after its request')
        self.cast([request - 1, frame, frame + 1], samples=[('damage', request, frame)])
        analyze_run(self.run)

    def test_a_malformed_cast_frame_names_its_case(self):
        self.manifest['cases'][0]['consumer'] = {}
        self.save()
        for line in ('x', '-5', '1.5'):
            with self.subTest(line=line):
                self.cast([line])
                self.rejects(f"active-idle-resume: invalid cast frame {line!r} in cast-frames.tsv")

    def test_tty_resume_needs_the_resume_edge_inside_vt_return(self):
        # The driver's tty-resume declaration on this fixture's resume edge
        # (trace 20 s): paused on the spare VT niri cannot redraw, so only
        # vt-return needs a redraw. No client damage in this case.
        self.cpu = [row for row in self.cpu if row[1] != 14 * S + S // 2]
        self.gpu = [row for row in self.gpu if row[1] != 14 * S + S // 2]
        self.write_tables()
        case = self.manifest['cases'][0]
        case['stimuli'] = [{'label': 'vt-out'}, {'label': 'vt-return', 'min_redraws': 1}]
        case['edge_in'] = [[1, 'vt-return']]

        def windows(out, back):   # trace seconds: vt-out [out, back - gap), vt-return [back, back + 1.5)
            (out_start, out_end), back_start = out, back
            self.observation['journal'] = [
                {'label': 'vt-out', 'start_mono_ns': int((1000 + out_start) * S),
                 'end_mono_ns': int((1000 + out_end) * S)},
                {'label': 'vt-return', 'start_mono_ns': int((1000 + back_start) * S),
                 'end_mono_ns': int((1000 + back_start + 1.5) * S)}]
            self.save()

        windows((17, 19.5), 19.5)              # the resume falls in the return
        analyze_run(self.run)
        windows((17.5, 20.5), 20.5)            # the resume falls in the switch out
        self.rejects('active-idle-resume: edge 1 is not inside stimulus vt-return')
        windows((17, 19.5), 20.5)              # the resume falls between the two
        self.rejects('active-idle-resume: edge 1 is not inside stimulus vt-return')

    def test_a_redraw_between_samples_breaks_the_quiet_interval(self):
        # The settled-segment check is what keeps the gap before sample-3 quiet.
        self.cpu += [['Niri::redraw', 17 * S, 1000]]
        self.write_tables()
        self.rejects('while settled')

    def test_complete_pilot_passes(self):
        result = analyze_run(self.run)
        self.assertEqual(result['verdict'], 'lane-passed')
        case = result['cases'][0]
        self.assertEqual([s['state'] for s in case['segments']], ['active', 'settled', 'active'])
        self.assertEqual(case['segments'][1]['redraws'], 1)
        self.assertEqual({r['verdict'] for r in result['cases'][1:]}, {'unverified'})

    def test_zone_names_with_unquoted_commas(self):
        # tracy-csvexport writes generic zone names unquoted, so a name like
        # MultiRenderer<'_, '_, '_> spans several fields; the DRM renderer's
        # zones put text where the time column would be.
        lines = ['name,src_file,src_line,ns_since_start,exec_time_ns,thread,value']
        lines += [f'{name},src/niri.rs,1,{t},{d},1,' for name, t, d in self.cpu]
        lines += [f"<smithay::backend::renderer::multigpu::MultiRenderer<'_, '_, '_> as "
                  f"smithay::backend::renderer::Renderer>::cleanup_texture_cache,"
                  f"/smithay/src/backend/renderer/multigpu/mod.rs,1032,{15 * S + 1},20,1,",
                  f"smithay::backend::renderer::gles::GlesFrame<'_, '_>::finish_internal,"
                  f"/smithay/src/backend/renderer/gles/mod.rs,2478,{16 * S},20,1,"]
        (self.case / 'cpu.csv').write_text('\n'.join(lines) + '\n')
        result = analyze_run(self.run)
        self.assertEqual(result['verdict'], 'lane-passed')
        self.assertEqual(result['cases'][0]['segments'][1]['redraws'], 1)

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

    def test_stimulus_requires_its_messages_inside_its_window(self):
        self.manifest['cases'][0]['stimuli'][0]['messages'] = ['IdleInhibit inhibited=1']
        self.save()
        self.rejects("stimulus damage never traced 'IdleInhibit inhibited=1'")
        late = ['IdleInhibit inhibited=1', 16 * S]          # after the 14-15 s window
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], self.messages + [late])
        self.rejects("stimulus damage never traced 'IdleInhibit inhibited=1'")
        inside = ['IdleInhibit inhibited=1', 14 * S + S // 2]
        self.write_csv('messages.csv', ['MessageName', 'total_ns'], self.messages + [inside])
        analyze_run(self.run)

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
            self.assertEqual((by_name['screencast']['lane'], by_name['screencast']['required']),
                             ('dedicated', False))
            # The idle inhibitor runs in this lane: a real client, traced taking hold.
            inhibitor = by_name['idle-inhibitor']
            self.assertEqual((inhibitor['lane'], inhibitor['edges']), ('headless', [0]))
            self.assertEqual(
                [(stimulus['label'], stimulus.get('messages')) for stimulus in inhibitor['stimuli']],
                [('inhibit', ['IdleInhibit inhibited=1']), ('client', None),
                 ('release', ['IdleInhibit inhibited=0']), ('collect', None)])

    def test_prepare_the_dedicated_lane(self):
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
                       CAPTURE_META='/bin/true', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                       PATH=f'{bin_dir}:{os.environ["PATH"]}')
            script = root / 'docs/materials/scripts/optic-settling-smoke.sh'
            run = subprocess.run(['bash', str(script), 'prepare', '--lane', 'dedicated'], env=env,
                                 capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            manifest = json.loads((base / 'out/manifest.json').read_text())
            by_name = {case['name']: case for case in manifest['cases']}
            dedicated = sorted(name for name, case in by_name.items() if case['lane'] == 'dedicated')
            self.assertEqual(dedicated, ['drm-aurora', 'screencast', 'tty-resume', 'unlock'])
            self.assertEqual(by_name['tty-resume']['edge_in'], [[1, 'vt-return']])
            # Paused on the spare VT, niri cannot redraw: only the return must.
            self.assertEqual(by_name['tty-resume']['stimuli'][:2],
                             [{'label': 'vt-out'}, {'label': 'vt-return', 'min_redraws': 1}])
            self.assertEqual(by_name['unlock']['edge_in'], [[1, 'unlock']])
            self.assertEqual(by_name['screencast']['consumer'],
                             {'frames_in': ['sample-1', 'sample-2', 'sample-3'],
                              'samples': ['sample-1', 'sample-2', 'sample-3']})
            for name in dedicated:
                self.assertIn({'label': 'collect', 'min_redraws': 1}, by_name[name]['stimuli'], name)
                kdl = (base / 'out' / name / 'case.kdl').read_text()
                self.assertIn('output "DP-1" { mode "3440x1440@59.999"; scale 1; }', kdl)
                self.assertIn('dbus-interfaces-in-non-session-instances', kdl)
            self.assertFalse((base / 'out/aurora-full').exists())      # headless configs not written
            missing = subprocess.run(['bash', str(script), 'prepare', '--lane', 'dedicated'],
                                     env=dict(env, OUT=str(base / 'out2'), DRM_MODE=''),
                                     capture_output=True, text=True)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn('DRM_MODE', missing.stderr)


# Stubs for every program the driver launches. Each long-lived one records
# "name pid" so a test can prove cleanup reaped it.
SERVE = """
import os, signal, socket, sys, time
os.chdir(os.environ['XDG_RUNTIME_DIR'])        # relative binds keep socket paths short
if sys.argv[1] == 'weston':
    names = [arg.split('=', 1)[1] for arg in sys.argv[2:] if arg.startswith('--socket=')]
elif sys.argv[1] == 'dbus-daemon':
    names = [arg.split('path=', 1)[1] for arg in sys.argv[2:] if arg.startswith('--address=')]
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
if sys.argv[1] == 'niri' and os.environ.get('STUB_NIRI_IGNORES_TERM'):
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
while True:
    time.sleep(60)
"""
CONSUMER = """
import json, os, pathlib, signal, sys, time
stub = pathlib.Path(os.environ['STUB_DIR'])
with open(stub / 'pids', 'a') as pids:
    pids.write(f'consumer {os.getpid()}\\n')
def stop(*_):
    pathlib.Path(sys.argv[3]).write_text(json.dumps({'stopped_by_signal': True}))
    sys.exit(0)
def sample(*_):
    # Answer an armed request at once with a frame of STUB_SAMPLE_SIZE.
    target = pathlib.Path(pathlib.Path(sys.argv[4]).read_text().strip())
    pathlib.Path(f'{target}.armed').touch()
    width, height = map(int, os.environ.get('STUB_SAMPLE_SIZE', '3440x1440').split('x'))
    target.write_bytes(bytes(3))
    pathlib.Path(f'{target}.json').write_text(json.dumps({'width': width, 'height': height}))
signal.signal(signal.SIGTERM, stop)
signal.signal(signal.SIGUSR1, sample)
print('ready 1', flush=True)
(stub / 'cast-ready').touch()
if os.environ.get('STUB_CONSUMER_EXITS'):
    sys.exit(0)                                     # dies after ready, before any sample
while True:
    time.sleep(0.1)
"""
STUBS = {
    'bin/weston': 'echo "weston $$" >> "$STUB_DIR/pids"\nexec python3 "$STUB_DIR/serve.py" weston "$@"\n',
    'bin/niri': (
        'case $1 in\n'
        '    validate) exit 0 ;;\n'
        '    msg) case "$2 $3" in\n'
        '        "-j outputs") echo \'{"DP-1": {"name": "DP-1", "current_mode": 0, "logical": {"scale": 1.0},'
        ' "modes": [{"width": 3440, "height": 1440, "refresh_rate": 59999, "is_preferred": true}]}}\' ;;\n'
        '        -j*) echo "[{\\"id\\": 1, \\"app_id\\": \\"gos-probe\\", \\"is_focused\\": false,'
        ' \\"layout\\": {\\"window_size\\": [${STUB_PROBE_SIZE:-1000, 1200}]}},'
        ' {\\"id\\": 2, \\"app_id\\": \\"gos-other\\", \\"is_focused\\": true}]" ;;\n'
        '        esac\n'
        '        for a do case $prev in --path) printf png > "$a" ;; esac; prev=$a; done; exit 0 ;;\n'
        '    -c) echo "niri $$" >> "$STUB_DIR/pids"; exec python3 "$STUB_DIR/serve.py" niri ;;\n'
        'esac\nexit 2\n'),
    'bin/kitty': 'echo "kitty $$" >> "$STUB_DIR/pids"\nwhile [ "$1" != sh ]; do shift; done\nexec "$@"\n',
    'bin/magick': ('case "$*" in\n'
                   # info: output ends without a newline, as real magick's does.
                   '    *%@*) printf %s "${STUB_RECT:-1000x1200+100+100}"; exit 0 ;;\n'
                   '    *"%w %h"*) printf "3440 1440"; exit 0 ;;\n'
                   'esac\nfor arg do :; done\n'
                   'case $arg in *:-) printf image ;; *) printf image > "$arg" ;; esac\n'),
    'bin/dbus-daemon': 'echo "dbus-daemon $$" >> "$STUB_DIR/pids"\nexec python3 "$STUB_DIR/serve.py" dbus-daemon "$@"\n',
    'bin/sudo': 'exit 0\n',
    'bin/pgrep': 'exit 1\n',
    'bin/gst-inspect-1.0': 'exit 0\n',
    'vt/chvt': ('printf "tty%s\\n" "$1" > "$VT_ACTIVE_FILE"\necho "chvt $1" >> "$STUB_DIR/meta.log"\n'
                '[ "$1" = 1 ] || : > "$STUB_DIR/away"\n'),
    'vt/loginctl': 'exit 0\n',
    'tools/lock': ('echo "lock $$" >> "$STUB_DIR/pids"\ntrap \'exit 0\' USR1\necho locked\n'
                   ': > "$STUB_DIR/locked"\nwhile :; do sleep 0.1; done\n'),
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
        (self.stubs / 'tools/consumer.py').write_text(CONSUMER)
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
        self.drm = self.sysfs({'card1-DP-1': 'connected', 'card1-HDMI-A-1': 'disconnected'})

    def sysfs(self, connectors):
        root = self.base / 'drm'
        shutil.rmtree(root, ignore_errors=True)
        for connector, status in connectors.items():
            (root / connector).mkdir(parents=True)
            (root / connector / 'status').write_text(status + '\n')
        (root / 'card1').mkdir(parents=True)                   # the card itself has no status
        return root

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
        lane = env.pop('LANE_ARGS', '').split()
        run_env = dict(self.env, CASES=case, **env)
        if lane:
            # The dedicated lane refuses a Wayland session; this suite may run inside one.
            run_env.pop('WAYLAND_DISPLAY', None)
            # Nor may it read this host's outputs.
            run_env.setdefault('OPTIC_SETTLING_STUB_DRM_SYSFS', str(self.drm))
        return subprocess.Popen(['bash', str(script), 'pilot', *lane], cwd=self.root, start_new_session=True,
                                env=run_env,
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

    def dedicated(self, case, marker):
        """TERM a stub dedicated run once `marker` appears; everything is reaped."""
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start(case, STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05',
                            OPTIC_SETTLING_STUB_CONSUMER=str(self.stubs / 'tools/consumer.py'),
                            OPTIC_SETTLING_STUB_LOCK=str(self.stubs / 'tools/lock'),
                            LANE_ARGS='--lane dedicated')
        self.await_marker(driver, marker, 60)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assert_cleaned_up(case)
        return active

    def started(self, name):
        return [line for line in (self.stubs / 'pids').read_text().splitlines() if line.startswith(name)]

    def test_term_while_casting_reaps_the_consumer_and_the_bus(self):
        self.dedicated('screencast', 'cast-ready')
        self.assertTrue(self.started('consumer'))
        self.assertTrue(self.started('dbus-daemon'))

    def test_term_with_the_session_locked_reaps_the_lock_client(self):
        self.dedicated('unlock', 'locked')
        self.assertTrue(self.started('lock'))

    def test_term_while_switched_away_restores_the_vt_before_release(self):
        active = self.dedicated('tty-resume', 'away')
        self.assertEqual(active.read_text(), 'tty1\n')
        record = json.loads((self.out / 'vt-restore.json').read_text())
        self.assertEqual((record['outcome'], record['home'], record['from']), ('restored', 1, 2))
        meta = (self.stubs / 'meta.log').read_text().splitlines()
        self.assertLess(meta.index('chvt 1'), max(i for i, line in enumerate(meta) if line.startswith('release ')))

    def test_stub_overrides_need_stub_tools(self):
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        for name, value in (('OPTIC_SETTLING_STUB_TIMESCALE', '0.05'),
                            ('OPTIC_SETTLING_STUB_DRM_SYSFS', str(self.base / 'drm'))):
            with self.subTest(name=name):
                env = dict(self.env, OUT=str(self.base / f'out-{name}'), **{name: value})
                env.pop('OPTIC_SETTLING_STUB_TOOLS')
                run = subprocess.run(['bash', str(script), 'pilot'], cwd=self.root, env=env,
                                     capture_output=True, text=True, timeout=60)
                self.assertNotEqual(run.returncode, 0)
                self.assertIn('need OPTIC_SETTLING_STUB_TOOLS', run.stderr)

    def test_vt_overrides_need_stub_tools(self):
        script = self.root / 'docs/materials/scripts/optic-settling-smoke.sh'
        for name, value in (('VT_CHVT', '/bin/true'), ('VT_LOGINCTL', '/bin/true'),
                            ('VT_ACTIVE_FILE', str(self.base / 'active'))):
            with self.subTest(name=name):
                env = dict(self.env, OUT=str(self.base / f'out-{name}'), **{name: value})
                env.pop('OPTIC_SETTLING_STUB_TOOLS')
                run = subprocess.run(['bash', str(script), 'pilot'], cwd=self.root, env=env,
                                     capture_output=True, text=True, timeout=60)
                self.assertNotEqual(run.returncode, 0)
                self.assertIn(f'{name} needs OPTIC_SETTLING_STUB_TOOLS', run.stderr)

    def test_a_refused_return_fails_the_run_after_release(self):
        # The return to VT 1 never lands: restoration fails, the run exits 1,
        # and the capture lock is still released.
        (self.stubs / 'vt/chvt').write_text(
            '#!/bin/sh\necho "chvt $1" >> "$STUB_DIR/meta.log"\n[ "$1" = 1 ] && exit 0\n'
            'printf "tty%s\\n" "$1" > "$VT_ACTIVE_FILE"\n: > "$STUB_DIR/away"\n')
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start('tty-resume', STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05', LANE_ARGS='--lane dedicated')
        self.await_marker(driver, 'away', 60)
        _, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 1, stderr)
        self.assertIn('VT restoration failed', stderr)
        record = json.loads((self.out / 'vt-restore.json').read_text())
        self.assertEqual((record['outcome'], record['home'], record['observed']), ('failed', 1, 2))
        self.assertEqual(active.read_text(), 'tty2\n')
        meta = (self.stubs / 'meta.log').read_text().splitlines()
        self.assertEqual(meta.count('chvt 1'), 3)                 # three verified attempts
        self.assertTrue(meta[-1].startswith('release '), meta)
        self.assertEqual([pid for pid in self.pids() if alive(pid)], [])

    def test_a_drm_niri_that_ignores_term_is_killed_at_case_end(self):
        # A DRM niri that ignores TERM holds the device and the VT; the case's
        # stop must not wait on it forever, or restoration never runs.
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start('drm-aurora', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'), STUB_NIRI_IGNORES_TERM='1',
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05', LANE_ARGS='--lane dedicated')
        try:
            _, stderr = driver.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            driver.kill()
            self.fail('the driver hung on a niri that ignores TERM')
        observation = json.loads((self.out / 'drm-aurora/observation.json').read_text())   # the case ran to its stop
        self.assertEqual(observation['topology'],
                         [{'name': 'DP-1', 'scale': 1.0,
                           'mode': {'width': 3440, 'height': 1440, 'refresh_hz': 59.999}}])
        self.assertEqual(json.loads((self.out / 'manifest.json').read_text())['drm'],
                         {'output': 'DP-1', 'mode': '3440x1440@59.999'})
        self.assertTrue(self.started('niri'))
        self.assertEqual([pid for pid in self.pids() if alive(pid)], [])
        self.assertEqual(json.loads((self.out / 'vt-restore.json').read_text())['outcome'], 'not-needed')
        self.assertTrue((self.stubs / 'meta.log').read_text().splitlines()[-1].startswith('release '))

    def run_dedicated(self, case, timeout_s=90, **env):
        """A stub dedicated run left to finish on its own; returns its stderr."""
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start(case, DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05',
                            OPTIC_SETTLING_STUB_CONSUMER=str(self.stubs / 'tools/consumer.py'),
                            OPTIC_SETTLING_STUB_LOCK=str(self.stubs / 'tools/lock'),
                            LANE_ARGS='--lane dedicated', **env)
        try:
            _, stderr = driver.communicate(timeout=timeout_s)
        except subprocess.TimeoutExpired:
            driver.kill()
            self.fail(f'the {case} run hung')
        self.returncode = driver.returncode
        self.assertEqual([pid for pid in self.pids() if alive(pid)], [])
        self.assertTrue((self.stubs / 'meta.log').read_text().splitlines()[-1].startswith('release '))
        return stderr

    def test_a_lock_client_that_ignores_unlock_fails_within_the_bound(self):
        (self.stubs / 'tools/lock').write_text(
            '#!/bin/sh\necho "lock $$" >> "$STUB_DIR/pids"\ntrap \'\' USR1\necho locked\n'
            'while :; do sleep 0.1; done\n')
        started = time.monotonic()
        stderr = self.run_dedicated('unlock')
        self.assertEqual(self.returncode, 1, stderr)
        self.assertIn('unlock: the lock client did not exit within 10 s', stderr)
        self.assertLess(time.monotonic() - started, 45, stderr)
        self.assertTrue(self.started('lock'))
        self.assertEqual(json.loads((self.out / 'vt-restore.json').read_text())['outcome'], 'not-needed')

    def test_screencast_refuses_unchecked_crops_and_a_dead_consumer(self):
        for env, message in (
            (dict(STUB_PROBE_SIZE='900, 1200'), "calibrated probe 1000x1200, the case's probe is 900x1200 over IPC"),
            (dict(STUB_RECT='400x1200+100+100', STUB_PROBE_SIZE='400, 1200'),
             'probe 400x1200 is too small for the screencast crops'),
            (dict(STUB_SAMPLE_SIZE='1280x720'), 'sample-1 is 1280x720, the calibrated screen is 3440x1440'),
            (dict(STUB_CONSUMER_EXITS='1'), 'the screencast consumer exited (see cast.log)'),
        ):
            with self.subTest(message=message):
                shutil.rmtree(self.out, ignore_errors=True)
                (self.stubs / 'meta.log').unlink(missing_ok=True)
                stderr = self.run_dedicated('screencast', **env)
                self.assertEqual(self.returncode, 1, stderr)
                self.assertIn(f'screencast: {message}', stderr)

    def test_a_screencast_run_reaches_its_crops(self):
        # The whole drive with matching sizes: the run gets through every
        # sample and crop (the stub traces then fail analysis, never the driver).
        stderr = self.run_dedicated('screencast')
        for k in (1, 2, 3):
            for crop in ('client', 'aurora'):
                self.assertTrue((self.out / f'screencast/{crop}-{k}.rgb').is_file(), stderr)
        self.assertEqual((self.out / 'screen-drm.txt').read_text(), '3440 1440\n')
        self.assertTrue((self.out / 'screencast/observation.json').is_file(), stderr)
        self.assertTrue((self.out / 'screencast/cast-summary.json').is_file(), stderr)

    def test_a_second_term_during_cleanup_still_restores_and_releases(self):
        (self.stubs / 'vt/chvt').write_text(
            '#!/bin/sh\necho "chvt $1" >> "$STUB_DIR/meta.log"\n'
            'if [ "$1" = 1 ]; then : > "$STUB_DIR/restoring"; sleep 1; fi\n'
            'printf "tty%s\\n" "$1" > "$VT_ACTIVE_FILE"\n[ "$1" = 1 ] || : > "$STUB_DIR/away"\n')
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start('tty-resume', STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(active), VT_CHVT=str(self.stubs / 'vt/chvt'),
                            VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            OPTIC_SETTLING_STUB_TIMESCALE='0.05', LANE_ARGS='--lane dedicated')
        self.await_marker(driver, 'away', 60)
        driver.send_signal(signal.SIGTERM)
        self.await_marker(driver, 'restoring', 30)
        driver.send_signal(signal.SIGTERM)                    # lands while restoration runs
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertEqual(active.read_text(), 'tty1\n')
        self.assertEqual(json.loads((self.out / 'vt-restore.json').read_text())['outcome'], 'restored')
        self.assert_cleaned_up('tty-resume')

    def test_term_during_a_dedicated_capture_reaps_the_bus_and_records_the_vt(self):
        active = self.base / 'active'
        active.write_text('tty1\n')
        driver = self.start('drm-aurora', STUB_CAPTURE_S='600', DRM_OUTPUT='DP-1',
                            DRM_MODE='3440x1440@59.999', VT_ACTIVE_FILE=str(active),
                            VT_CHVT=str(self.stubs / 'vt/chvt'), VT_LOGINCTL=str(self.stubs / 'vt/loginctl'),
                            LANE_ARGS='--lane dedicated')
        self.await_marker(driver, 'capture-started', 60)
        time.sleep(0.5)
        elapsed, stderr = self.terminate(driver)
        self.assertEqual(driver.returncode, 143, stderr)
        self.assertLess(elapsed, self.BOUND_S, stderr)
        self.assertIn('dbus-daemon', (self.stubs / 'pids').read_text())
        self.assert_cleaned_up('drm-aurora')
        self.assertEqual(json.loads((self.out / 'vt-restore.json').read_text())['outcome'], 'not-needed')
        self.assertEqual(json.loads((self.out / 'vt.json').read_text()), {'home': 1, 'spare': 2})

    def test_dedicated_prerequisites_name_the_missing_item(self):
        (self.stubs / 'bin/sudo').write_text('#!/bin/sh\nexit 1\n')
        driver = self.start('drm-aurora', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(self.base / 'active'), LANE_ARGS='--lane dedicated')
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 1)
        self.assertIn('no NOPASSWD rule for /usr/bin/chvt', stderr)
        self.assertEqual(self.pids(), [])                     # nothing was launched
        self.assertEqual(list(self.runtime.glob('gos.*')), [])

    def refused(self, **env):
        driver = self.start('drm-aurora', DRM_OUTPUT='DP-1', DRM_MODE='3440x1440@59.999',
                            VT_ACTIVE_FILE=str(self.base / 'active'), LANE_ARGS='--lane dedicated', **env)
        _, stderr = driver.communicate(timeout=60)
        self.assertEqual(driver.returncode, 1, stderr)
        self.assertEqual(self.pids(), [])                     # nothing was launched
        return stderr

    def test_a_second_connected_output_is_refused_before_launch(self):
        drm = self.sysfs({'card1-DP-1': 'connected', 'card1-HDMI-A-1': 'connected'})
        stderr = self.refused(OPTIC_SETTLING_STUB_DRM_SYSFS=str(drm))
        self.assertIn('HDMI-A-1 is connected besides DRM_OUTPUT DP-1', stderr)

    def test_an_unconnected_drm_output_is_refused_before_launch(self):
        for index, connectors in enumerate((
                {'card1-DP-1': 'disconnected', 'card1-HDMI-A-1': 'disconnected'},  # none connected
                {})):                                                                # no connector
            with self.subTest(connectors=connectors):
                drm = self.sysfs(connectors)
                stderr = self.refused(OPTIC_SETTLING_STUB_DRM_SYSFS=str(drm), OUT=str(self.base / f'out-{index}'))
                self.assertIn('DRM_OUTPUT DP-1 is not connected', stderr)

    def test_a_leftover_niri_from_an_earlier_run_is_refused(self):
        # pgrep -x niri misses it: the run's snapshot copy is named binary.
        old = self.base / 'optic-settling/pilot-old'
        old.mkdir(parents=True)
        shutil.copy(shutil.which('sleep'), old / 'binary')
        leftover = subprocess.Popen([str(old / 'binary'), '60'])
        self.addCleanup(leftover.wait)
        self.addCleanup(leftover.kill)
        stderr = self.refused()
        self.assertIn(f'pid {leftover.pid} runs {(old / "binary").resolve()}', stderr)

    def test_a_tty_resume_run_journals_the_switch_out_and_the_return_apart(self):
        self.run_dedicated('tty-resume')
        labels = [line.split('\t')[0] for line in
                  (self.out / 'tty-resume/journal.tsv').read_text().splitlines()]
        self.assertEqual(labels[:2], ['vt-out', 'vt-return'])
