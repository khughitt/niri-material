#!/usr/bin/env bash
# Noise layers Tracy costs (design 2026-10-06-noise-layers-design.md §7.3):
# the material program's GPU span per damaged frame for stacks of fine
# layers at scale 1 and 8, and the backdrop grain pass at scale 1 and 8.
# Each case replaces the wallpaper DAMAGE_STEPS times at 1 Hz; every
# replacement damages the backdrop, which re-renders the glass and reruns the
# grain pass. NOISE_LAYERS_COST_PILOT=1 runs three steps per case.
# NOISE_LAYERS_COST_AB=<niri-tracy binary> instead compares that binary (b)
# with this tree's (a) in one session: `none` and `one-fine-1` alternate
# a, b, a, b, so a shift between sessions cannot pass for a shift between
# binaries.
set -eu
PILOT=${NOISE_LAYERS_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_LAYERS_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
AB=${NOISE_LAYERS_COST_AB:-}
[ -z "$AB" ] || [ -x "$AB" ] || { echo "NOISE_LAYERS_COST_AB must name an executable" >&2; exit 2; }
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
capture_preflight headless
build_binaries
if [ -n "$AB" ]; then
    cp "$AB" "$OUT/niri-tracy-b"
    capture_identity --binary "$OUT/niri-tracy-b" --config "pilot=$PILOT" --config ab=1
else
    capture_identity --config "pilot=$PILOT"
fi
reserve_tracy_port
tools_ready
DAMAGE_STEPS=20
[ "$PILOT" = 0 ] || DAMAGE_STEPS=3
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
declare -A BIN=()
for name in "${ORDER[@]}"; do BIN[$name]=$NIRI_TRACY; done
if [ -n "$AB" ]; then
    ORDER=()
    for base in none one-fine-1; do for round in 1 2; do for side in a b; do
        CASES[$base-$side$round]=${CASES[$base]}
        BIN[$base-$side$round]=$NIRI_TRACY
        [ "$side" = a ] || BIN[$base-$side$round]=$OUT/niri-tracy-b
        ORDER+=("$base-$side$round")
    done; done; done
fi
write_cost_config() {   # $1 path, $2 glass noise lines
    GLASS_EXTRA=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1\n'"$2" \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$1.tmp"
    # This script owns and reaps every wallpaper PID.
    sed '/^spawn-at-startup "swaybg" /d' "$1.tmp" > "$1"
    rm "$1.tmp"
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
    reload_marker "$cfg"
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
    python3 - "$OUT" "$name" "$DAMAGE_STEPS" "$((PW * PH))" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, math, pathlib, statistics, sys
root, name, steps, area = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
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
PYREPORT
done
[ -n "$AB" ] || python3 - "$OUT/metrics.txt" <<'PYDELTA' >> "$OUT/metrics.txt"
import sys
m = dict(line.strip().split('=', 1) for line in open(sys.argv[1]) if '=' in line)
none = float(m['none_MaterialRenderElement::draw_median_ms'])
for name in ['one-fine-1', 'four-fine-1', 'four-fine-8', 'backdrop-fine-1', 'backdrop-fine-8']:
    median = float(m[f'{name}_MaterialRenderElement::draw_median_ms'])
    area = int(m[f'{name}_glass_area_px'])
    print(f'{name}_material_ns_per_px_over_none={(median - none) * 1e6 / area:.4f}')
PYDELTA
finish
printf 'PASS: noise layers cost (pilot=%s%s)\n' "$PILOT" "${AB:+, a/b}"
