#!/usr/bin/env python3
"""Reduce bounded optic settling traces to an evidence verdict."""

import csv
import hashlib
import json
import re
import sys
from pathlib import Path


EDGE = re.compile(r'^OpticTimeline active=([01]) real_ns=(\d+) logical_ns=(\d+)$')
BEAT = 'Niri::refresh_idle_inhibit'
REDRAW = 'Niri::redraw'
MATERIAL = 'MaterialRenderElement::draw'
GAP_NS = 1_500_000_000
FAMILIES = (
    'active-idle-resume', 'combined-motion', 'gate-policy',
    'client-damage', 'finite-attention', 'visibility',
    'outputs-dpms', 'session-activation', 'reload-clock',
)


def integer(value, label):
    if isinstance(value, bool):
        raise ValueError(f'invalid {label}')
    try:
        number = int(value)
    except (TypeError, ValueError) as error:
        raise ValueError(f'invalid {label}') from error
    if number < 0 or str(value) != str(number):
        raise ValueError(f'invalid {label}')
    return number


def parse_edges(rows):
    """Parse transition messages in trace order; unrelated messages are ignored."""
    edges = []
    for row in rows:
        message = row.get('MessageName')
        if not isinstance(message, str) or not message.startswith('OpticTimeline'):
            continue
        match = EDGE.fullmatch(message)
        if not match:
            raise ValueError(f'malformed optic transition: {message}')
        active, real_ns, logical_ns = map(int, match.groups())
        trace_ns = integer(row.get('total_ns'), 'message trace time')
        if edges and (trace_ns <= edges[-1]['trace_ns'] or real_ns <= edges[-1]['real_ns']
                      or active == edges[-1]['active']):
            raise ValueError('duplicate, unordered, or invalid optic transition')
        if edges and active == 1 and logical_ns != edges[-1]['logical_ns']:
            raise ValueError('optic time changed across the held interval')
        edges.append(dict(active=active, real_ns=real_ns,
                          logical_ns=logical_ns, trace_ns=trace_ns))
    return edges


def check_window(cpu_ns, gpu_ns, start_ns, end_ns, max_redraws, max_material_draws):
    start_ns = integer(start_ns, 'window start')
    end_ns = integer(end_ns, 'window end')
    if end_ns <= start_ns:
        raise ValueError('invalid observation window')
    for name, rows, limit in (('redraw', cpu_ns, max_redraws),
                              ('material draw', gpu_ns, max_material_draws)):
        if sum(start_ns <= integer(time, name + ' time') < end_ns for time in rows) > limit:
            raise ValueError(f'too many {name}s in observation window')


def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f'missing or invalid {path.name}') from error


def sha256(path):
    try:
        with path.open('rb') as stream:
            return hashlib.file_digest(stream, 'sha256').hexdigest()
    except OSError as error:
        raise ValueError(f'missing {path}') from error


def csv_rows(path, required):
    try:
        with path.open(newline='') as stream:
            reader = csv.DictReader(stream)
            if not set(required) <= set(reader.fieldnames or []):
                raise ValueError(f'missing CSV headers in {path.name}')
            return list(reader)
    except (OSError, UnicodeError, csv.Error) as error:
        raise ValueError(f'missing or invalid CSV: {path.name}') from error


def zones(path, time_column, duration_column):
    rows = csv_rows(path, ('name', time_column, duration_column))
    zones_by_name = {}
    for row in rows:
        start = integer(row[time_column], f'{path.name} time')
        duration = integer(row[duration_column], f'{path.name} duration')
        zones_by_name.setdefault(row['name'], []).append((start, duration))
    for events in zones_by_name.values():
        events.sort()
    return zones_by_name


def coverage(beats, start, end):
    times = sorted(t for t, _ in beats if start <= t <= end)
    if not times or times[0] - start > GAP_NS or end - times[-1] > GAP_NS:
        raise ValueError('incomplete heartbeat coverage')
    if any(b - a > GAP_NS for a, b in zip(times, times[1:])):
        raise ValueError('heartbeat gap exceeds 1.5 seconds')


def interval(value, label):
    if not isinstance(value, list) or len(value) != 2:
        raise ValueError(f'invalid {label} interval')
    start, end = (integer(item, label) for item in value)
    if start >= end:
        raise ValueError(f'invalid {label} interval')
    return start, end


def analyze_case(run, case, lane):
    name = case['name']
    if integer(case.get('repetitions'), 'repetitions') != 1:
        raise ValueError(f'{name}: repeated captures are not recorded')
    directory = run / name
    if not directory.is_dir():
        raise ValueError(f'missing case {name}')
    if sha256(directory / 'case.kdl') != case['config_sha256']:
        raise ValueError(f'{name}: config identity mismatch')
    export = read_json(directory / 'export.json')
    if not all(export.get(kind) is True for kind in ('messages', 'cpu', 'gpu')):
        raise ValueError(f'{name}: failed trace export')
    edges = parse_edges(csv_rows(directory / 'messages.csv', ('MessageName', 'total_ns')))
    if len(edges) != 2 or [edge['active'] for edge in edges] != [0, 1]:
        raise ValueError(f'{name}: missing pause/resume edge')
    cpu = zones(directory / 'cpu.csv', 'ns_since_start', 'exec_time_ns')
    gpu = zones(directory / 'gpu.csv', 'Time from start of program', 'GPU execution time')
    observation = read_json(directory / 'observation.json')
    if observation.get('stimuli_intervals') != []:
        raise ValueError(f'{name}: finite stimuli are not accounted for')
    before = interval(observation.get('active_before'), 'active before')
    hold = interval(observation.get('hold'), 'hold')
    after = interval(observation.get('active_after'), 'active after')
    if not (before[1] <= edges[0]['trace_ns'] <= hold[0]
            < hold[1] <= edges[1]['trace_ns'] <= after[0]):
        raise ValueError(f'{name}: observation endpoints do not bracket edges')
    if hold[1] - hold[0] < integer(case['hold_ns'], 'declared hold'):
        raise ValueError(f'{name}: short observation')
    trace_end = integer(observation.get('trace_end_ns'), 'trace end')
    if integer(observation.get('trace_start_ns'), 'trace start') > before[0] or trace_end < after[1]:
        raise ValueError(f'{name}: trace endpoints do not cover observation')
    if max((t + d for events in cpu.values() for t, d in events), default=0) < after[1]:
        raise ValueError(f'{name}: truncated CPU trace')
    coverage(cpu.get(BEAT, []), hold[0], hold[1])
    redraws = [t for t, _ in cpu.get(REDRAW, [])]
    draws = [t for t, _ in gpu.get(MATERIAL, [])]
    for label, window in (('before', before), ('after', after)):
        if not any(window[0] <= t < window[1] for t in redraws):
            raise ValueError(f'{name}: missing active CPU {label} control')
        if not any(window[0] <= t < window[1] for t in draws):
            raise ValueError(f'{name}: missing active GPU {label} control')
    if edges[0]['trace_ns'] < hold[0]:
        check_window(redraws, draws, edges[0]['trace_ns'], hold[0], 2, 2)
    check_window(redraws, draws, hold[0], hold[1], 0, 0)
    # The whole settled span is bounded too: a third edge flush cannot hide
    # between the declared quiet window and the resume marker.
    check_window(redraws, draws, edges[0]['trace_ns'], edges[1]['trace_ns'], 2, 2)
    before_rgb = directory / observation['before_rgb']
    after_rgb = directory / observation['after_rgb']
    if not before_rgb.is_file() or not after_rgb.is_file() or not before_rgb.stat().st_size or not after_rgb.stat().st_size:
        raise ValueError(f'{name}: empty decoded pixels')
    same_pixels = sha256(before_rgb) == sha256(after_rgb)
    expected_pixels = observation.get('expected_pixels')
    if expected_pixels == 'equal' and not same_pixels:
        raise ValueError(f'{name}: held pixels changed')
    if expected_pixels == 'different' and same_pixels:
        raise ValueError(f'{name}: damage did not change pixels')
    if expected_pixels not in ('equal', 'different'):
        raise ValueError(f'{name}: missing pixel expectation')
    topology = observation.get('topology')
    if not isinstance(topology, list) or len(topology) != 1 or not topology[0]:
        raise ValueError(f'{name}: output topology unverified')
    if lane == 'dedicated' and topology[0].startswith('headless'):
        raise ValueError(f'{name}: dedicated lane uses headless output')
    return {'name': name, 'family': case['family'], 'verdict': 'passed',
            'redraws_held': sum(edges[0]['trace_ns'] <= t < edges[1]['trace_ns'] for t in redraws),
            'material_draws_held': sum(edges[0]['trace_ns'] <= t < edges[1]['trace_ns'] for t in draws),
            'hold_ns': hold[1] - hold[0], 'topology': topology}


def analyze_run(run: Path) -> dict:
    run = Path(run)
    manifest = read_json(run / 'manifest.json')
    if manifest.get('schema') != 1 or manifest.get('mode') not in ('pilot', 'matrix'):
        raise ValueError('invalid manifest schema or mode')
    lane = manifest.get('lane')
    if lane not in ('headless', 'dedicated'):
        raise ValueError('invalid capture lane')
    if not re.fullmatch('[0-9a-f]{40}', manifest.get('source_commit', '')):
        raise ValueError('invalid source commit')
    if sha256(run / 'binary') != manifest.get('binary_sha256'):
        raise ValueError('binary identity mismatch')
    cases = manifest.get('cases')
    if not isinstance(cases, list) or not cases:
        raise ValueError('missing cases')
    names = [case.get('name') for case in cases]
    if len(names) != len(set(names)):
        raise ValueError('duplicate cases')
    if {case.get('family') for case in cases} != set(FAMILIES):
        raise ValueError('missing or unknown check family')
    results = []
    for case in cases:
        if case.get('lane') != lane:
            if case.get('required'):
                raise ValueError(f"{case['name']}: required hardware lane falsely verified")
            results.append({'name': case['name'], 'family': case['family'], 'verdict': 'unverified'})
            continue
        results.append(analyze_case(run, case, lane))
    complete = all(result['verdict'] == 'passed' for result in results)
    required_passed = all(result['verdict'] == 'passed' for case, result in zip(cases, results)
                          if case.get('required'))
    return {'verdict': 'passed' if required_passed and (complete or manifest['mode'] == 'pilot')
            else 'incomplete', 'complete': complete,
            'cases': results}


def main():
    if len(sys.argv) != 2:
        print('usage: optic_settling.py RUN_DIR', file=sys.stderr)
        return 2
    run = Path(sys.argv[1])
    try:
        result = analyze_run(run)
    except (IndexError, KeyError, OSError, ValueError) as error:
        result = {'verdict': 'invalid', 'error': str(error)}
    (run / 'analysis.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return 0 if result['verdict'] == 'passed' else 1


if __name__ == '__main__':
    sys.exit(main())
