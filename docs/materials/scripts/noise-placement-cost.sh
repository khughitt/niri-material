#!/usr/bin/env bash
# Noise placement Tracy costs (design §7.3): cached static work, backdrop
# damage and 10 Hz parameter changes, with both sites at equal workload.
# NOISE_COST_PILOT=1 limits wallpaper changes to five; all six case blocks
# and their assertions still run before committing to the full run.
set -eu
PILOT=${NOISE_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
WALL_PIDS=()
start_wall() {
    local display
    display=$(cd "$RT" && ls -t wayland-* 2>/dev/null | grep -v '\.lock$' | head -1)
    [ -n "$display" ] || fail "no nested wayland display in $RT"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$display swaybg -m fill -i "$1" >> "$OUT/swaybg.log" 2>&1 &
    WALL_PIDS+=($!)
}
stop_walls() {
    local pid
    for pid in "${WALL_PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    done
    WALL_PIDS=()
}
cleanup_cost() {
    local rc=$?
    stop_walls
    # The conditional preserves rc for cleanup without errexit aborting it.
    if (exit "$rc"); then cleanup; else cleanup; fi
}
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
capture_preflight headless
build_binaries
capture_identity --config "pilot=$PILOT"
reserve_tracy_port
tools_ready
# The shared writer generates the pinned material; remove its startup
# wallpaper line so this script owns and reaps every wallpaper PID.
write_cost_config() {
    local path=$1 site=$2 amount=$3
    GLASS_EXTRA=$'ior 1.5\nattenuation-color "#ffffff"\nsaturation 1\nbackdrop-blur true\nroughness 0.5\nnoise '"$amount"' type="fine" site="'"$site"'"' \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$path.tmp"
    sed '/^spawn-at-startup "swaybg" /d' "$path.tmp" > "$path"
    rm "$path.tmp"
}
reload_marker() { msg "$NIRI_TRACY" action load-config-file --path "$1"; }
# Sleep to an absolute monotonic deadline: image work and IPC consume part
# of the specified one-second / 100 ms period, rather than extending it.
now_ns() { python3 -c 'import time; print(time.monotonic_ns())'; }
sleep_until() {
    python3 - "$1" <<'PYTIME'
import sys, time
time.sleep(max(0, (int(sys.argv[1]) - time.monotonic_ns()) / 1e9))
PYTIME
}
DAMAGE_STEPS=20
[ "$PILOT" = 0 ] || DAMAGE_STEPS=5
for site in backdrop glass; do
    for case_name in static damage drag; do
        name=$case_name-$site
        cfg=$OUT/$name.kdl
        write_cost_config "$cfg" "$site" 0.3
        start_nested "$NIRI_TRACY" "$cfg"
        capture_bg "$name"; capture_ready "$name"
        start_wall "$WALL"
        sleep 0.5
        spawn_probe "$NIRI_TRACY" "$IDLE"
        sleep 1
        steps=0
        if [ "$case_name" != static ]; then reload_marker "$cfg"; sleep 0.2; fi
        if [ "$case_name" = damage ]; then
            steps=$DAMAGE_STEPS
            start=$(now_ns)
            for i in $(seq 1 "$steps"); do
                magick -size 1280x720 xc:"rgb($((i*10)),100,120)" "$OUT/$name-wall-$i.png"
                stop_walls; start_wall "$OUT/$name-wall-$i.png"
                sleep_until "$((start + i*1000000000))"
            done
        elif [ "$case_name" = drag ]; then
            steps=50
            start=$(now_ns)
            for i in $(seq 0 49); do
                amount=$(awk -v i="$i" 'BEGIN { printf "%.2f", 0.10+i/100 }')
                next_cfg=$OUT/$name-step-$i.kdl
                write_cost_config "$next_cfg" "$site" "$amount"
                # Completed files, unique paths: the watcher cannot duplicate
                # the explicit reload while a file is still being written.
                reload_marker "$next_cfg"
                sleep_until "$((start + (i+1)*100000000))"
            done
            cfg=$next_cfg
        fi
        if [ "$case_name" != static ]; then sleep 0.3; reload_marker "$cfg"; fi
        capture_wait
        stop_walls; stop_nested
        # Also proves the trace has a complete, unstalled final 20 seconds.
        count_last20 "$name" > "$OUT/$name-redraws.txt"
        csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
        python3 - "$OUT" "$name" "$case_name" "$site" "$steps" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, pathlib, statistics, sys
root, name, case, site, steps = pathlib.Path(sys.argv[1]), *sys.argv[2:5], int(sys.argv[5])
def zones(path, time_column, duration_column):
    with path.open() as f:
        reader = csv.DictReader(f)
        assert time_column in reader.fieldnames and duration_column in reader.fieldnames, path
        return [(next(iter(r.values())), float(r[time_column]), float(r[duration_column])) for r in reader]
cpu = zones(root / (name + '.csv'), 'ns_since_start', 'exec_time_ns')
gpu = zones(root / (name + '.gpu.csv'), 'Time from start of program', 'GPU execution time')
end = max(t for _, t, _ in cpu)
if case == 'static':
    begin = end - 20e9
else:
    markers = [(t, d) for n, t, d in cpu if n == 'State::reload_config']
    assert len(markers) >= 2, f'{name}: no stimulus markers'
    begin = min(t for t, _ in markers)
    end = max(t + d for t, d in markers)
print(f'{name}_window_begin_ns={begin:.0f}\n{name}_window_end_ns={end:.0f}')
for span in ['EffectBuffer::prepare_grain', 'Blur::render', 'EffectBuffer::prepare_prefilter']:
    count = sum(n == span and begin <= t <= end for n, t, _ in cpu)
    print(f'{name}_{span}_count={count}')
    if case == 'static' or (case == 'drag' and site == 'glass'):
        assert count == 0, f'{name}: {span} ran {count} times'
    elif case == 'drag':
        assert count > 40, f'{name}: {span} ran only {count} times'
spans = ['Grain::render', 'Blur::render', 'Prefilter::downsample', 'MaterialRenderElement::draw']
median_sum = total = 0
for span in spans:
    values = [d for n, t, d in gpu if n == span and begin <= t <= end]
    if case != 'static':
        if not (site == 'glass' and (span == 'Grain::render' or case == 'drag' and span != spans[-1])):
            assert values, f'{name}: no {span} GPU samples in stimulus window'
        median = statistics.median(values) if values else 0
        print(f'{name}_{span}_gpu_count={len(values)}')
        print(f'{name}_{span}_median_ms={median / 1e6:.6f}')
        median_sum += median
        total += sum(values)
if case == 'static' and site == 'backdrop':
    first = next((d for n, _, d in gpu if n == 'Grain::render'), None)
    assert first is not None, f'{name}: no initial Grain::render GPU sample'
    print(f'{name}_first_grain_ms={first / 1e6:.6f}')
if case != 'static':
    print(f'{name}_stimulus_count={steps}')
    print(f'{name}_median_sum_per_call_ms={median_sum / 1e6:.6f}')
    # There may be two effect buffers per output. Count every measured pass
    # instead of pretending one per-call median is a whole damage cascade.
    print(f'{name}_mean_total_per_change_ms={total / steps / 1e6:.6f}')
PYREPORT
    done
done
finish
printf 'PASS: noise placement cost (pilot=%s)\n' "$PILOT"
