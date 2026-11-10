#!/usr/bin/env bash
# Rigid tilt GPU cost (design 2026-10-09-transient-rigid-tilt §8.5): a same-session A/B of
# this tree's niri-tracy (a) against a kept baseline built from the branch's base (b),
# alternating a, b, a, b so a shift between sessions cannot pass for one between binaries.
# Rest cases, each with the live look and the beam off: `view-tilt 0`, `view-tilt 10` with no
# swing running, and one fine noise layer. Each replaces the wallpaper DAMAGE_STEPS times at
# 1 Hz, so every step redraws resting glass. Bound: a's median material draw within 1 % of
# b's or 0.002 ms, whichever is larger; a/a or b/b repeats further apart than that make the
# case inconclusive. The swing case (a only) repeats focus gains at 10°, κ 6 and records the
# swing frames' draws beside the rest median, and the extra GPU time per swing: no bound.
# Cost: ~8 min cold build (release plus Tracy), then await_load waits up to 5 min for load1 < 1.0.
# Needs OUT (a fresh dir), CAPTURE_TASK and NIRI_MATERIAL_WORK_ROOT in the environment.
# VIEW_TILT_COST_AB=<baseline niri-tracy> is required; VIEW_TILT_COST_PILOT=1 runs three
# damage steps per case and two gains.
set -eu
PILOT=${VIEW_TILT_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "VIEW_TILT_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
AB=${VIEW_TILT_COST_AB:-}
[ -n "$AB" ] && [ -x "$AB" ] || { echo "VIEW_TILT_COST_AB must name an executable" >&2; exit 2; }
HERE=$(cd "$(dirname "$0")" && pwd)
source "$HERE/glass-optic-smoke-lib.sh"
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
build_binaries
await_load
capture_preflight headless
cp "$AB" "$OUT/niri-tracy-b"
capture_identity --binary "$OUT/niri-tracy-b" --input "$HERE/view_tilt_cost_zones.py" --config "pilot=$PILOT" --config ab=1
reserve_tracy_port
tools_ready
DAMAGE_STEPS=20; GAINS=6
[ "$PILOT" = 0 ] || { DAMAGE_STEPS=3; GAINS=2; }
LIVE=$'ior 1.28\nthickness 31.2\nattenuation-color "#0D1D1E"\nattenuation-distance 11\nchromatic-aberration 0.36\nroughness 0.24\nbevel 12\nbevel-profile 2\noffset-x -6\noffset-y -5'
declare -A GLASS=([tilt0]="$LIVE" [tilt10]="$LIVE" [fine]="$LIVE"$'\nnoise 0.3 type="fine"')
declare -A VIEW=([tilt0]='view-tilt 0' [tilt10]=$'view-tilt 10\n        view-distance 6' [fine]='view-tilt 0')
ORDER=()
declare -A BIN=() BASE=() SIDE=()
for base in tilt0 tilt10 fine; do for round in 1 2; do for side in a b; do
    name=$base-$side$round; ORDER+=("$name"); BASE[$name]=$base; SIDE[$name]=$side
    BIN[$name]=$NIRI_TRACY; [ "$side" = a ] || BIN[$name]=$OUT/niri-tracy-b
done; done; done
write_cost_config() {   # $1 path, $2 case, $3 side
    local view=${VIEW[$2]}
    # The baseline predates the keys, so its configs carry none: b is today's rest path.
    [ "$3" = a ] || view=
    GLASS_EXTRA=${GLASS[$2]} RESPONSE_EXTRA=$view FOCUS_RESPONSE=none \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$1.tmp"
    # This script owns and reaps every wallpaper PID.
    sed '/^spawn-at-startup "swaybg" /d' "$1.tmp" > "$1"
    rm "$1.tmp"
}
for name in "${ORDER[@]}"; do
    cfg=$OUT/$name.kdl
    write_cost_config "$cfg" "${BASE[$name]}" "${SIDE[$name]}"
    start_nested "${BIN[$name]}" "$cfg"
    capture_bg "$name"; capture_ready "$name"
    start_wall "$WALL"
    sleep 0.5
    spawn_probe "$NIRI" "$IDLE"
    # The probe's first focus starts a swing under view-tilt 10; let it end before measuring.
    sleep 1.5
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
    csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
    export_cpu "$name"
    PYTHONPATH=$HERE python3 - "$OUT" "$name" "$DAMAGE_STEPS" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, statistics, sys, pathlib
root, name, steps = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
from view_tilt_cost_zones import zones
cpu = zones(root / f'{name}.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / f'{name}.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted((t, d) for n, t, d in cpu if n == 'State::reload_config')
assert len(markers) == steps + 1, f'{name}: {len(markers)} reload markers, expected {steps + 1}'
begin, end = markers[0][0], markers[-1][0] + markers[-1][1]
values = [d for n, t, d in gpu if n == 'MaterialRenderElement::draw' and begin <= t <= end]
assert len(values) >= steps, f'{name}: {len(values)} material draws for {steps} damages'
print(f'{name}_MaterialRenderElement::draw_median_ms={statistics.median(values) / 1e6:.6f}')
PYREPORT
done
python3 - "$OUT/metrics.txt" <<'PYVERDICT' >> "$OUT/metrics.txt"
import sys
m = dict(line.strip().split('=', 1) for line in open(sys.argv[1]) if '=' in line)
get = lambda n: float(m[f'{n}_MaterialRenderElement::draw_median_ms'])
for base in ['tilt0', 'tilt10', 'fine']:
    a1, a2, b1, b2 = (get(f'{base}-{s}') for s in ['a1', 'a2', 'b1', 'b2'])
    b = (b1 + b2) / 2
    a = (a1 + a2) / 2
    bound = max(0.01 * b, 0.002)
    if abs(a1 - a2) > bound or abs(b1 - b2) > bound:
        verdict = 'INCONCLUSIVE'
    else:
        verdict = 'PASS' if a - b <= bound else 'FAIL'
    print(f'{base}_a_ms={a:.6f}')
    print(f'{base}_b_ms={b:.6f}')
    print(f'{base}_bound_ms={bound:.6f}')
    print(f'{base}_verdict={verdict}')
PYVERDICT

# The swing case, on a only: two windows, focus gains on the probe every 3 s.
cfg=$OUT/swing.kdl
write_cost_config "$cfg" tilt10 a
start_nested "$NIRI_TRACY" "$cfg"
capture_bg swing; capture_ready swing
start_wall "$WALL"
sleep 0.5
spawn_probe "$NIRI" "$IDLE"
steal_focus "$NIRI"
probe=$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .id')
other=$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-other") | .id')
sleep 1.5
reload_marker "$cfg"
# Six gains at 3 s end near 23 s of the 30 s trace; a gain that overruns fails here.
start=$(now_ns)
for i in $(seq 1 "$GAINS"); do
    t0=$((start + (i - 1) * 3000000000))
    reload_marker "$cfg"
    msg "$NIRI" action focus-window --id "$probe"
    sleep_until "$((t0 + 1500000000))" 500000000
    msg "$NIRI" action focus-window --id "$other"
    sleep_until "$((t0 + 3000000000))" 500000000
done
capture_wait
stop_walls; stop_nested
csvexport --gpu "$OUT/swing.tracy" "$OUT/swing.gpu.csv"
export_cpu swing
PYTHONPATH=$HERE python3 - "$OUT" "$GAINS" <<'PYSWING' >> "$OUT/metrics.txt"
import csv, statistics, sys, pathlib
root, gains = pathlib.Path(sys.argv[1]), int(sys.argv[2])
from view_tilt_cost_zones import zones
cpu = zones(root / 'swing.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / 'swing.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted(t for n, t, _ in cpu if n == 'State::reload_config')[1:]
assert len(markers) == gains, f'{len(markers)} gain markers, expected {gains}'
m = dict(line.strip().split('=', 1) for line in open(root / 'metrics.txt') if '=' in line)
rest = float(m['tilt10_a_ms']) * 1e6
frames = [d for t0 in markers for n, t, d in gpu if n == 'MaterialRenderElement::draw' and t0 <= t <= t0 + 0.9e9]
assert len(frames) >= 6 * gains, f'{len(frames)} swing frames for {gains} gains'
print(f'swing_frames={len(frames)}')
print(f'swing_frame_median_ms={statistics.median(frames) / 1e6:.6f}')
print(f'swing_frame_max_ms={max(frames) / 1e6:.6f}')
print(f'swing_extra_ms_per_gain={sum(f - rest for f in frames) / gains / 1e6:.6f}')
PYSWING
finish
printf 'PASS: view tilt cost run complete (pilot=%s); verdicts in %s/metrics.txt\n' "$PILOT" "$OUT"
