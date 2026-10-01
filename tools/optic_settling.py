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


# tracy-csvexport --messages on a trace without messages prints this, not a header.
NO_MESSAGES = b'There are currently no messages!\n'


def csv_rows(path, required, allow_empty=False):
    """Rows of an export; the no-messages notice is no rows only where the export may be empty."""
    try:
        if allow_empty and path.read_bytes() == NO_MESSAGES:
            return []
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


# A settled segment may hold the edge flush (the queued redraw and an already
# dispatched one-shot) in its first FLUSH_NS; afterwards every redraw and
# material draw must fall inside a journaled stimulus or pointer interval.
FLUSH_NS = 2_000_000_000
FLUSH_MAX = 2
POINTER = 'Niri::notify_activity'
POINTER_NS = 300_000_000
STIMULUS_MAX_NS = 6_000_000_000
WINDOW_MARGIN_NS = 250_000_000
ACTIVE_MIN_NS = 1_000_000_000
OFFSET_SLACK_NS = 5_000_000
SEGMENT_STATES = ('active', 'static', 'settled', 'setup')


def config_identity(directory):
    """One hash over every config a case runs, by name; the live copy is excluded."""
    digest = hashlib.sha256()
    for path in sorted(Path(directory).glob('*.kdl')):
        if path.name == 'live.kdl':
            continue
        digest.update(path.name.encode() + b'\0' + path.read_bytes() + b'\0')
    return digest.hexdigest()


def run_config_identity(directory):
    """config_identity with the run's own directory written as $OUT: configs name
    files inside their run (the backdrop), so only this form compares across runs."""
    directory = Path(directory)
    run = str(directory.resolve().parent).encode()
    digest = hashlib.sha256()
    for path in sorted(directory.glob('*.kdl')):
        if path.name == 'live.kdl':
            continue
        digest.update(path.name.encode() + b'\0' + path.read_bytes().replace(run, b'$OUT') + b'\0')
    return digest.hexdigest()


def merge(intervals):
    merged = []
    for start, end in sorted(intervals):
        if merged and start <= merged[-1][1]:
            merged[-1][1] = max(merged[-1][1], end)
        else:
            merged.append([start, end])
    return [tuple(item) for item in merged]


def free_spans(start, end, exempt):
    """The stimulus-free spans of [start, end), longest first."""
    spans, cursor = [], start
    for a, b in merge(exempt):
        if b <= cursor or a >= end:
            continue
        if a > cursor:
            spans.append((cursor, a))
        cursor = max(cursor, b)
    if cursor < end:
        spans.append((cursor, end))
    return sorted(spans, key=lambda span: span[0] - span[1])


def count(times, start, end):
    return sum(start <= t < end for t in times)


def cadence_bounds(min_hz, max_hz, duration_ns):
    """Event-count bounds for a cadence window: 25 % either way plus one coalesced frame."""
    seconds = duration_ns / 1e9
    low = int(min_hz * seconds * 0.75)
    high = int(max_hz * seconds * 1.25) + 1
    return max(low, 1 if min_hz > 0 else 0), high


def journal_intervals(observation, edges, declared):
    """Journaled stimuli in trace time, aligned through the edges' monotonic stamps."""
    journal = observation.get('journal')
    if not isinstance(journal, list):
        raise ValueError('missing stimulus journal')
    labels = [entry.get('label') for entry in journal if isinstance(entry, dict)]
    if labels != [stimulus['label'] for stimulus in declared]:
        raise ValueError(f'journaled stimuli {labels} differ from the declared ones')
    if not journal:
        return []
    if not edges:
        raise ValueError('journaled stimuli need an optic edge to align them')
    offsets = [edge['real_ns'] - edge['trace_ns'] for edge in edges]
    if max(offsets) - min(offsets) > OFFSET_SLACK_NS:
        raise ValueError('optic edges disagree on the monotonic offset')
    offset = offsets[0]
    intervals = []
    for entry in journal:
        start = integer(entry.get('start_mono_ns'), 'stimulus start') - offset
        end = integer(entry.get('end_mono_ns'), 'stimulus end') - offset
        if not 0 < end - start <= STIMULUS_MAX_NS:
            raise ValueError(f"stimulus {entry['label']} has an invalid or too long interval")
        intervals.append((start, end))
    return intervals


def analyze_case(run, case, lane):
    name = case['name']
    if integer(case.get('repetitions'), 'repetitions') != 1:
        raise ValueError(f'{name}: repeated captures are recorded as separate cases')
    directory = run / name
    if not directory.is_dir():
        raise ValueError(f'missing case {name}')
    if config_identity(directory) != case['config_sha256']:
        raise ValueError(f'{name}: config identity mismatch')
    export = read_json(directory / 'export.json')
    if not all(export.get(kind) is True for kind in ('messages', 'cpu', 'gpu')):
        raise ValueError(f'{name}: failed trace export')
    # A trace without any message exports a notice instead of a header.
    edges = parse_edges(csv_rows(directory / 'messages.csv', ('MessageName', 'total_ns'), allow_empty=True))
    expected_edges = case.get('edges')
    if [edge['active'] for edge in edges] != expected_edges:
        raise ValueError(f'{name}: optic edges {[e["active"] for e in edges]}, expected {expected_edges}')
    segments = case.get('segments')
    if not isinstance(segments, list) or len(segments) != len(edges) + 1:
        raise ValueError(f'{name}: one segment state per edge interval required')
    cpu = zones(directory / 'cpu.csv', 'ns_since_start', 'exec_time_ns')
    gpu = zones(directory / 'gpu.csv', 'Time from start of program', 'GPU execution time')
    observation = read_json(directory / 'observation.json')
    declared = case.get('stimuli')
    if not isinstance(declared, list):
        raise ValueError(f'{name}: missing declared stimuli')
    stimuli = journal_intervals(observation, edges, declared)
    beats = cpu.get(BEAT, [])
    if not beats:
        raise ValueError(f'{name}: capture contains no heartbeat')
    trace_end = max(t + d for t, d in beats)
    if any(edge['trace_ns'] >= trace_end for edge in edges):
        raise ValueError(f'{name}: truncated CPU trace')
    redraws = [t for t, _ in cpu.get(REDRAW, [])]
    draws = [t for t, _ in gpu.get(MATERIAL, [])]
    gpu_live = min((t for events in gpu.values() for t, _ in events), default=None)
    if gpu_live is None:
        raise ValueError(f'{name}: the trace holds no GPU zone')
    pointers = [(t, t + POINTER_NS) for t, _ in cpu.get(POINTER, [])]
    exempt = merge(stimuli + pointers)

    for stimulus, (start, end) in zip(declared, stimuli):
        for label, times, key in (('redraws', redraws, 'min_redraws'), ('draws', draws, 'min_draws')):
            if count(times, start, end) < integer(stimulus.get(key, 0), key):
                raise ValueError(f"{name}: stimulus {stimulus['label']} produced too few {label}")

    bounds = [0] + [edge['trace_ns'] for edge in edges] + [trace_end]
    report = []
    held = 0
    for index, segment in enumerate(segments):
        state = segment.get('state') if isinstance(segment, dict) else None
        if state not in SEGMENT_STATES:
            raise ValueError(f'{name}: invalid segment state {state}')
        start, end = bounds[index], bounds[index + 1]
        if state == 'setup':
            # Launch and client setup before the capture connects: only the
            # leading segment of a case that sends no input may go unchecked.
            if index != 0 or not expected_edges or expected_edges[0] != 0:
                raise ValueError(f'{name}: only a leading segment before a pause may be setup')
            report.append({'state': state, 'end_ns': end})
            continue
        if state == 'settled':
            if index == 0 or expected_edges[index - 1] != 0:
                raise ValueError(f'{name}: a settled segment must follow a pause edge')
            flush_end = min(start + FLUSH_NS, end)
            for label, times in (('redraw', redraws), ('material draw', draws)):
                loose = [t for t in times if start <= t < end
                         and not any(a <= t < b for a, b in exempt)]
                if sum(t < flush_end for t in loose) > FLUSH_MAX:
                    raise ValueError(f'{name}: more than {FLUSH_MAX} {label}s in the edge flush')
                late = [t for t in loose if t >= flush_end]
                if late:
                    raise ValueError(f'{name}: {len(late)} {label}(s) while settled, outside any stimulus '
                                     f'(first at {late[0] / 1e9:.3f} s)')
            quiet = free_spans(flush_end, end, exempt)
            if not quiet or quiet[0][1] - quiet[0][0] < integer(case.get('hold_ns'), 'declared hold'):
                raise ValueError(f'{name}: short observation')
            for span in quiet:
                if span[1] - span[0] >= GAP_NS:
                    coverage(beats, *span)
            held += count(redraws, start, end)
            report.append({'state': state, 'start_ns': start, 'end_ns': end,
                           'quiet_ns': quiet[0][1] - quiet[0][0],
                           'redraws': count(redraws, start, end),
                           'material_draws': count(draws, start, end)})
            continue
        # The last stimulus-free second-plus before the segment ends: after
        # setup and the keepalive motions, nearest the edge it controls. A
        # cadence window also starts after GPU zones are recorded; a static
        # one needs no GPU zone, since no redraw means no draw.
        margin_start = start + WINDOW_MARGIN_NS
        if state == 'active':
            margin_start = max(margin_start, gpu_live)
        spans = [span for span in free_spans(margin_start, end - WINDOW_MARGIN_NS, exempt)
                 if span[1] - span[0] >= ACTIVE_MIN_NS]
        if not spans:
            raise ValueError(f'{name}: no stimulus-free {state} window in segment {index}')
        window = max(spans)
        coverage(beats, *window)
        n_redraws, n_draws = count(redraws, *window), count(draws, *window)
        if state == 'static':
            low, high = 0, 0
        else:
            low, high = cadence_bounds(float(segment['min_hz']), float(segment['max_hz']), window[1] - window[0])
        for label, value in (('redraws', n_redraws), ('material draws', n_draws)):
            if not low <= value <= high:
                raise ValueError(f'{name}: {value} {label} in the {state} window of segment {index}, '
                                 f'expected {low}..{high}')
        report.append({'state': state, 'window_ns': list(window),
                       'redraws': n_redraws, 'material_draws': n_draws, 'expected': [low, high]})

    pixels = case.get('pixels')
    if not isinstance(pixels, list):
        raise ValueError(f'{name}: missing pixel expectations')
    for pair in pixels:
        before, after = directory / pair['before'], directory / pair['after']
        if not before.is_file() or not after.is_file() or not before.stat().st_size or not after.stat().st_size:
            raise ValueError(f'{name}: empty decoded pixels')
        same = sha256(before) == sha256(after)
        if pair.get('expect') == 'equal' and not same:
            raise ValueError(f"{name}: {pair['before']} and {pair['after']} differ")
        if pair.get('expect') == 'different' and same:
            raise ValueError(f"{name}: {pair['before']} and {pair['after']} did not change")
        if pair.get('expect') not in ('equal', 'different'):
            raise ValueError(f'{name}: missing pixel expectation')
    topology = observation.get('topology')
    if not isinstance(topology, list) or len(topology) != 1 or not topology[0]:
        raise ValueError(f'{name}: output topology unverified')
    if lane == 'dedicated' and topology[0].startswith('headless'):
        raise ValueError(f'{name}: dedicated lane uses headless output')
    return {'name': name, 'family': case['family'], 'verdict': 'passed',
            'edges': [{'active': e['active'], 'trace_ns': e['trace_ns'], 'logical_ns': e['logical_ns']}
                      for e in edges],
            'segments': report, 'redraws_settled': held, 'topology': topology}


def analyze_run(run: Path) -> dict:
    run = Path(run)
    manifest = read_json(run / 'manifest.json')
    if manifest.get('schema') != 2 or manifest.get('mode') not in ('pilot', 'matrix'):
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
            results.append({'name': case['name'], 'family': case['family'], 'verdict': 'unverified',
                            'lane': case.get('lane'), 'why': case.get('why')})
            continue
        if not case.get('selected', True):
            if case.get('required'):
                raise ValueError(f"{case['name']}: a required case was not run")
            results.append({'name': case['name'], 'family': case['family'], 'verdict': 'not_run'})
            continue
        results.append(analyze_case(run, case, lane))
    complete = all(result['verdict'] == 'passed' for result in results)
    required_passed = all(result['verdict'] == 'passed' for case, result in zip(cases, results)
                          if case.get('required'))
    if manifest.get('development'):
        # A CASES subset never stands for a pilot or matrix verdict.
        verdict = 'development-passed' if required_passed else 'incomplete'
    else:
        verdict = 'passed' if required_passed and (complete or manifest['mode'] == 'pilot') else 'incomplete'
    return {'verdict': verdict, 'complete': complete, 'cases': results}


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
    return 0 if result['verdict'] in ('passed', 'development-passed') else 1


if __name__ == '__main__':
    sys.exit(main())
