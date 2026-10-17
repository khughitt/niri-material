#!/usr/bin/env bash
# Noise layers Tracy costs (design 2026-10-06-noise-layers-design.md §7.3):
# the material program's GPU span per damaged frame for stacks of fine
# layers at scale 1 and 8, and the backdrop grain pass at scale 1 and 8.
# Each case replaces the wallpaper DAMAGE_STEPS times at 1 Hz; every
# replacement damages the backdrop, which re-renders the glass and reruns the
# grain pass. NOISE_LAYERS_COST_PILOT=1 runs three steps per case.
# NOISE_LAYERS_COST_AB=<niri-tracy binary>[:<binary>...] instead compares
# those binaries (b1, b2, ...) with this tree's (a) in one session: each case
# in NOISE_LAYERS_COST_AB_CASES (default `none one-fine-1`) runs a, b1, b2,
# ..., then again, so a shift between sessions cannot pass for a shift
# between binaries. metrics.txt maps each side to its binary (ab_side_<s>=).
# Clock samples (CAPTURE_GPU_CLOCKS, on by default here) are joined to the
# material draws in <case>.draw-clocks.csv, and metrics.txt adds the median of
# the in-window draws that ran at the idle clock: the start-up burst runs at P0
# and costs about an eighth as much (2026-10-06-noise-layers-evidence.md,
# Hardware verdict). The `draw_median_ms` keys keep every in-window draw, as
# earlier runs reported; CAPTURE_GPU_CLOCKS=0 keeps their unsampled condition.
# One optional positional case name picks one case for any run; it is refused
# with NOISE_LAYERS_COST_AB.
set -eu
CAPTURE_GPU_CLOCKS=${CAPTURE_GPU_CLOCKS:-1}
PILOT=${NOISE_LAYERS_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_LAYERS_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
AB=${NOISE_LAYERS_COST_AB:-}
AB_BINS=()
[ -z "$AB" ] || IFS=: read -r -a AB_BINS <<< "$AB"
for bin in "${AB_BINS[@]}"; do
    [ -x "$bin" ] || { echo "NOISE_LAYERS_COST_AB must name executables: $bin" >&2; exit 2; }
done
read -r -a AB_CASES <<< "${NOISE_LAYERS_COST_AB_CASES:-none one-fine-1}"
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
FINE=$'noise 0.15 type="fine"'
declare -A CASES=(
    [none]='noise 0'
    [one-fine-1]='noise 0.3 type="fine"'
    [four-fine-1]="$FINE"$'\n'"$FINE"$'\n'"$FINE"$'\n'"$FINE"
    [four-fine-8]=$(printf '%s scale=8\n' "$FINE" "$FINE" "$FINE" "$FINE")
    [backdrop-fine-1]='noise 0.3 type="fine" site="backdrop"'
    [backdrop-fine-8]='noise 0.3 type="fine" site="backdrop" scale=8'
)
ORDER=(none one-fine-1 four-fine-1 four-fine-8 backdrop-fine-1 backdrop-fine-8)
if [ "$#" -gt 0 ]; then
    [ "$#" = 1 ] || fail 'expected at most one noise cost case'
    [ -z "$AB" ] || fail 'case arguments cannot be combined with NOISE_LAYERS_COST_AB'
    [ -n "${CASES[$1]+set}" ] || fail "unknown noise cost case: $1"
    ORDER=("$1")
fi
build_binaries
await_load
capture_preflight headless
if [ -n "$AB" ]; then
    AB_IDENTITY=()
    for i in "${!AB_BINS[@]}"; do
        cp "${AB_BINS[$i]}" "$OUT/niri-tracy-b$((i + 1))"
        AB_IDENTITY+=(--binary "$OUT/niri-tracy-b$((i + 1))")
    done
    capture_identity "${AB_IDENTITY[@]}" --config "pilot=$PILOT" --config ab=1 \
        --config "ab_cases=${AB_CASES[*]}"
else
    capture_identity --config "pilot=$PILOT" --config "cases=${ORDER[*]}"
fi
reserve_tracy_port
tools_ready
DAMAGE_STEPS=20
[ "$PILOT" = 0 ] || DAMAGE_STEPS=3
# A sampled full run's window outlasts the start-up burst by most of its steps,
# so it must yield an idle-clock draw per step; a pilot's three steps end inside it.
IDLE_DRAWS=0
[ "$PILOT" = 1 ] || [ "$CAPTURE_GPU_CLOCKS" = 0 ] || IDLE_DRAWS=$DAMAGE_STEPS
declare -A BIN=()
for name in "${ORDER[@]}"; do BIN[$name]=$NIRI_TRACY; done
if [ -n "$AB" ]; then
    SIDES=(a)
    echo "ab_side_a=$NIRI_TRACY" >> "$OUT/metrics.txt"
    for i in "${!AB_BINS[@]}"; do
        SIDES+=("b$((i + 1))")
        echo "ab_side_b$((i + 1))=${AB_BINS[$i]}" >> "$OUT/metrics.txt"
    done
    ORDER=()
    for base in "${AB_CASES[@]}"; do
        [ -n "${CASES[$base]+set}" ] || fail "NOISE_LAYERS_COST_AB_CASES: unknown case $base"
        for round in 1 2; do for side in "${SIDES[@]}"; do
            CASES[$base-$side-$round]=${CASES[$base]}
            BIN[$base-$side-$round]=$NIRI_TRACY
            [ "$side" = a ] || BIN[$base-$side-$round]=$OUT/niri-tracy-$side
            ORDER+=("$base-$side-$round")
        done; done
    done
fi
write_cost_config() {   # $1 path, $2 glass noise lines
    GLASS_EXTRA=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1\n'"$2" \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$1.tmp"
    # This script owns and reaps every wallpaper PID.
    sed '/^spawn-at-startup "swaybg" /d' "$1.tmp" > "$1"
    rm "$1.tmp"
}
reload_clock_anchor() {   # $1 config, $2 case name
    python3 - "$NIRI_SOCKET" "$1" "$OUT/$2.clock-anchor.txt" <<'PYANCHOR'
import json, pathlib, socket, sys, time
def connect():
    sock = socket.socket(socket.AF_UNIX)
    sock.settimeout(5)
    sock.connect(sys.argv[1])
    return sock
with connect() as events:
    events.sendall(b'"EventStream"\n')
    stream = events.makefile('r')
    assert json.loads(stream.readline()) == {'Ok': 'Handled'}, 'event stream refused'
    # ConfigLoaded is also in the initial state; consume it before requesting a reload.
    for line in stream:
        if 'ConfigLoaded' in json.loads(line):
            break
    else:
        raise RuntimeError('event stream closed before initial config state')
    before = time.time_ns()
    with connect() as action:
        request = {'Action': {'LoadConfigFile': {'path': sys.argv[2]}}}
        action.sendall(json.dumps(request).encode() + b'\n')
        assert json.loads(action.makefile('r').readline()) == {'Ok': 'Handled'}, 'reload action refused'
    for line in stream:
        event = json.loads(line)
        if 'ConfigLoaded' in event:
            assert event['ConfigLoaded']['failed'] is False, 'config reload failed'
            pathlib.Path(sys.argv[3]).write_text(f'{before} {time.time_ns()}\n')
            break
    else:
        raise RuntimeError('event stream closed before ConfigLoaded')
PYANCHOR
}
calibrate_probe_rect "$NIRI" 0
for name in "${ORDER[@]}"; do
    cfg=$OUT/$name.kdl
    write_cost_config "$cfg" "${CASES[$name]}"
    start_nested "${BIN[$name]}" "$cfg"
    capture_bg "$name"; capture_ready "$name"
    start_wall "$WALL"
    sleep 0.5
    spawn_probe "$NIRI" "$IDLE"
    probe_rect "$NIRI"
    sleep 1
    start=$(now_ns)
    if [ "$CAPTURE_GPU_CLOCKS" = 1 ]; then reload_clock_anchor "$cfg" "$name"; else reload_marker "$cfg"; fi
    for i in $(seq 1 "$DAMAGE_STEPS"); do
        magick -size 1280x720 xc:"rgb($((i * 10)),100,120)" "$OUT/$name-wall-$i.png"
        stop_walls; wait_for_wall_removal; start_wall "$OUT/$name-wall-$i.png"
        sleep_until "$((start + i * 1000000000))" 500000000
        reload_marker "$cfg"
    done
    capture_wait
    [ "$(wall_count)" -eq 1 ] || fail "wallpaper layer disappeared during $name"
    stop_walls; stop_nested
    count_last20 "$name" > "$OUT/$name-redraws.txt"
    csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
    python3 - "$OUT" "$name" "$DAMAGE_STEPS" "$((PW * PH))" "$CAPTURE_GPU_CLOCKS" "$IDLE_DRAWS" <<'PYREPORT' >> "$OUT/metrics.txt"
import bisect, csv, datetime, math, pathlib, statistics, sys
root, name, steps, area = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
sampled, idle_draws = sys.argv[5] == '1', int(sys.argv[6])
def zones(path, time_column, duration_column):
    with path.open() as f:
        reader = csv.DictReader(f)
        assert time_column in reader.fieldnames and duration_column in reader.fieldnames, path
        result = [(next(iter(r.values())), float(r[time_column]), float(r[duration_column])) for r in reader]
        assert all(math.isfinite(t) and math.isfinite(d) and t >= 0 and d >= 0 for _, t, d in result), path
        return result
cpu = zones(root / f'{name}.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / f'{name}.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted((t, d) for n, t, d in cpu if n == 'State::reload_config')
assert len(markers) == steps + 1, f'{name}: {len(markers)} reload markers, expected {steps + 1}'
begin, end = markers[0][0], markers[-1][0] + markers[-1][1]
for index, ((lo, _), (hi, _)) in enumerate(zip(markers, markers[1:]), 1):
    events = [n for n, t, _ in cpu if lo <= t < hi]
    assert 'EffectBuffer::sharp_damage' in events, f'{name}: no backdrop damage at step {index}'
for span in ['MaterialRenderElement::draw', 'Grain::render']:
    values = [d for n, t, d in gpu if n == span and begin <= t <= end]
    if span == 'MaterialRenderElement::draw':
        assert len(values) >= steps, f'{name}: {len(values)} material draws for {steps} damages'
    elif name.startswith('backdrop'):
        assert values, f'{name}: no grain pass in the stimulus window'
    print(f'{name}_{span}_gpu_count={len(values)}')
    print(f'{name}_{span}_median_ms={(statistics.median(values) if values else 0) / 1e6:.6f}')
print(f'{name}_glass_area_px={area}')
clocks_path = root / f'{name}.gpu-clocks.csv'
if sampled:
    assert clocks_path.exists(), f'{clocks_path} missing: CAPTURE_GPU_CLOCKS=1 records it'
    wall_lo, wall_hi = map(int, (root / f'{name}.clock-anchor.txt').read_text().split())
    assert 0 < wall_lo <= wall_hi and wall_hi - wall_lo <= 100_000_000, 'clock anchor exceeds one sample period'
    clocks = []
    with clocks_path.open() as file:
        for row in csv.DictReader(file, skipinitialspace=True):
            stamp = datetime.datetime.strptime(row['timestamp'], '%Y/%m/%d %H:%M:%S.%f').replace(tzinfo=datetime.timezone.utc)
            ns = int(stamp.timestamp()) * 10**9 + stamp.microsecond * 1000
            gr = int(row['clocks.current.graphics [MHz]'].split()[0])
            mem = int(row['clocks.current.memory [MHz]'].split()[0])
            power = float(row['power.draw [W]'].split()[0])
            assert gr > 0 and mem > 0 and math.isfinite(power) and power >= 0, 'invalid GPU clock sample'
            assert row['pstate'].startswith('P') and row['pstate'][1:].isdigit(), 'invalid GPU pstate'
            clocks.append((ns, gr, mem, row['pstate'], power))
    assert len(clocks) >= 2, 'GPU clock trace needs at least two samples'
    times = [sample[0] for sample in clocks]
    gaps = [b - a for a, b in zip(times, times[1:])]
    assert all(0 < gap <= 250_000_000 for gap in gaps), 'GPU clock samples must increase with gaps below 250 ms (one GPU)'
    # The later draws run at the idle clock: P8 at the lowest P8 graphics clock.
    idle_mhz = min((gr for _, gr, _, pstate, _ in clocks if pstate == 'P8'), default=None)
    assert idle_mhz is not None, f'{name}: the GPU clock trace never reaches P8'
    # The driver refreshes its readings every 0.5 s, so a draw's own state can
    # first show up to that long after it: a draw ran at the idle clock only if
    # every reading from its joined one through the first one past that lag does.
    lag = 500_000_000
    draws = sorted((int(t), d) for n, t, d in gpu if n == 'MaterialRenderElement::draw')
    assert draws, 'no material draws to align'
    selected = []
    with (root / f'{name}.draw-clocks.csv').open('w') as file:
        writer = csv.writer(file)
        writer.writerow(['trace_ns', 'gpu_ms', 'wall_earliest_ns', 'wall_latest_ns', 'sample_wall_ns', 'graphics_mhz', 'memory_mhz', 'pstate', 'power_w', 'in_window', 'idle_clock'])
        for t, duration in draws:
            lo, hi = wall_lo + t - int(begin), wall_hi + t - int(begin)
            index = bisect.bisect_right(times, lo) - 1
            after = bisect.bisect_left(times, hi + lag)
            assert index >= 0 and after < len(times) and hi - times[index] <= 250_000_000, 'GPU clock trace does not cover material draw and its reading lag'
            in_window = begin <= t <= end
            idle = all(clocks[k][3] == 'P8' and clocks[k][1] == idle_mhz for k in range(index, after + 1))
            if in_window and idle:
                selected.append((duration, clocks[index][4]))
            writer.writerow([t, f'{duration / 1e6:.6f}', lo, hi, *clocks[index], int(in_window), int(idle)])
    assert len(selected) >= idle_draws, f'{name}: {len(selected)} idle-clock material draws, expected at least {idle_draws}'
    print(f'{name}_idle_graphics_mhz={idle_mhz}')
    print(f'{name}_MaterialRenderElement::draw_idle_clock_count={len(selected)}')
    if selected:
        print(f'{name}_MaterialRenderElement::draw_idle_clock_median_ms={statistics.median(d for d, _ in selected) / 1e6:.6f}')
        # Which idle power floor (about 10.4 or 17 W) the case ran on.
        print(f'{name}_idle_clock_power_median_w={statistics.median(w for _, w in selected):.2f}')
    print(f'{name}_clock_anchor_width_ms={(wall_hi - wall_lo) / 1e6:.3f}')
    print(f'{name}_clock_max_gap_ms={max(gaps) / 1e6:.3f}')
    print(f'{name}_first_material_draw_wall_earliest_ns={wall_lo + draws[0][0] - int(begin)}')
    print(f'{name}_first_material_draw_wall_latest_ns={wall_hi + draws[0][0] - int(begin)}')
PYREPORT
done
if [ -z "$AB" ] && [ "$#" = 0 ]; then python3 - "$OUT/metrics.txt" "$IDLE_DRAWS" <<'PYDELTA' >> "$OUT/metrics.txt"
import sys
m = dict(line.strip().split('=', 1) for line in open(sys.argv[1]) if '=' in line)
names = ['one-fine-1', 'four-fine-1', 'four-fine-8', 'backdrop-fine-1', 'backdrop-fine-8']
def deltas(key, label):
    none = float(m[f'none_MaterialRenderElement::{key}'])
    for name in names:
        median = float(m[f'{name}_MaterialRenderElement::{key}'])
        area = int(m[f'{name}_glass_area_px'])
        print(f'{name}_{label}_ns_per_px_over_none={(median - none) * 1e6 / area:.4f}')
deltas('draw_median_ms', 'material')
if sys.argv[2] != '0':   # every case has an idle-clock median: sampled, not a pilot
    idle = {m[f'{name}_idle_graphics_mhz'] for name in ['none', *names]}
    assert len(idle) == 1, f'cases idled at different graphics clocks: {sorted(idle)} MHz'
    deltas('draw_idle_clock_median_ms', 'material_idle_clock')
PYDELTA
fi
finish
printf 'PASS: noise layers cost (pilot=%s%s)\n' "$PILOT" "${AB:+, a/b}"
