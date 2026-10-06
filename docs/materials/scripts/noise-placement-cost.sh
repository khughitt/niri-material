#!/usr/bin/env bash
# Noise placement Tracy costs (design §7.3): cached static work, backdrop
# damage and 10 Hz parameter changes, with both sites at equal workload.
# NOISE_COST_PILOT=1 limits wallpaper changes to five; all six case blocks
# and their assertions still run before committing to the full run.
set -eu
PILOT=${NOISE_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
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
    local sharp
    sharp=$(GLASS_EXTRA=$'ior 1.5\nattenuation-color "#ffffff"\nsaturation 1\nbackdrop-blur false\nroughness 0.5\nnoise '"$amount"' type="fine" site="'"$site"'"' emit_glass)
    cat >> "$path" <<KDL
material "gos-sharp" {
    glass {
$sharp
    }
    response "default" { focus "none"; accent "none"; ring-beam-speed 0; }
}
window-rule {
    match app-id="^gos-sharp$"
    material "gos-sharp"
    geometry-corner-radius 0
    background-effect { blur false; noise 0; saturation 1; }
}
KDL
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
        spawn_probe "$NIRI" "$IDLE"
        msg "$NIRI" action spawn -- kitty --config NONE --class gos-sharp \
            -o background_opacity=0 -o cursor_blink_interval=0 sh -c "$IDLE"
        for _ in $(seq 100); do
            [ "$(windows_with "$NIRI" gos-sharp)" -eq 1 ] && break
            sleep 0.05
        done
        [ "$(windows_with "$NIRI" gos-sharp)" -eq 1 ] || fail "no sharp roughness consumer"
        sleep 1
        steps=0
        if [ "$case_name" = drag ]; then reload_marker "$cfg"; sleep 0.2; fi
        if [ "$case_name" = damage ]; then
            steps=$DAMAGE_STEPS
            start=$(now_ns)
            for i in $(seq 1 "$steps"); do
                reload_marker "$cfg"
                magick -size 1280x720 xc:"rgb($((i*10)),100,120)" "$OUT/$name-wall-$i.png"
                stop_walls; wait_for_wall_removal; start_wall "$OUT/$name-wall-$i.png"
                sleep_until "$((start + i*1000000000))" 500000000
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
                sleep_until "$((start + (i+1)*100000000))" 50000000
            done
            cfg=$next_cfg
        fi
        if [ "$case_name" != static ]; then sleep 0.3; reload_marker "$cfg"; fi
        capture_wait
        for pid in "${WALL_PIDS[@]}"; do
            kill -0 "$pid" 2>/dev/null || fail "wallpaper exited during $name"
        done
        [ "$(wall_count)" -eq 1 ] || fail "wallpaper layer disappeared during $name"
        stop_walls; stop_nested
        # Also proves the trace has a complete, unstalled final 20 seconds.
        count_last20 "$name" > "$OUT/$name-redraws.txt"
        csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
        python3 - "$OUT" "$name" "$case_name" "$site" "$steps" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, math, pathlib, statistics, sys
root, name, case, site, steps = pathlib.Path(sys.argv[1]), *sys.argv[2:5], int(sys.argv[5])
def zones(path, time_column, duration_column):
    with path.open() as f:
        reader = csv.DictReader(f)
        assert time_column in reader.fieldnames and duration_column in reader.fieldnames, path
        result = [(next(iter(r.values())), float(r[time_column]), float(r[duration_column])) for r in reader]
        assert all(math.isfinite(t) and math.isfinite(d) and t >= 0 and d >= 0 for _, t, d in result), path
        return result
cpu = zones(root / (name + '.csv'), 'ns_since_start', 'exec_time_ns')
gpu = zones(root / (name + '.gpu.csv'), 'Time from start of program', 'GPU execution time')
end = max(t for _, t, _ in cpu)
if case == 'static':
    begin = end - 20e9
else:
    markers = sorted((t, d) for n, t, d in cpu if n == 'State::reload_config')
    expected = steps + 1 if case == 'damage' else steps + 2
    assert len(markers) == expected, f'{name}: {len(markers)} reload markers, expected {expected}'
    begin = markers[0][0]
    end = markers[-1][0] + markers[-1][1]
    intervals = list(zip(markers[:-1] if case == 'damage' else markers[1:-1], markers[1:] if case == 'damage' else markers[2:]))
    for index, ((lo, _), (hi, _)) in enumerate(intervals, 1):
        events = [n for n, t, _ in cpu if lo <= t < hi]
        if case == 'damage':
            assert 'Layer::mapped' in events, f'{name}: missing wallpaper mapping at step {index}'
            assert 'EffectBuffer::sharp_damage' in events, f'{name}: missing backdrop damage at step {index}'
        if case == 'damage' or site == 'backdrop':
            for span in ['EffectBuffer::prepare_sharp_prefilter', 'EffectBuffer::prepare_blurred_prefilter']:
                assert span in events, f'{name}: missing {span} at step {index}'
    print(f'{name}_observed_stimulus_count={len(intervals)}')
print(f'{name}_window_begin_ns={begin:.0f}\n{name}_window_end_ns={end:.0f}')
for span in ['EffectBuffer::prepare_grain', 'Blur::render', 'EffectBuffer::prepare_prefilter', 'EffectBuffer::prepare_sharp_prefilter', 'EffectBuffer::prepare_blurred_prefilter']:
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
    mapped = min((t for n, t, _ in cpu if n == 'Layer::mapped'), default=None)
    assert mapped is not None, f'{name}: initial wallpaper never mapped'
    first = next((d for n, t, d in gpu if n == 'Grain::render' and t >= mapped), None)
    assert first is not None, f'{name}: no initial Grain::render GPU sample'
    print(f'{name}_first_grain_ms={first / 1e6:.6f}')
if case != 'static':
    print(f'{name}_stimulus_count={steps}')
    print(f'{name}_median_sum_per_call_ms={median_sum / 1e6:.6f}')
    # There may be two effect buffers per output. Count every measured pass
    # instead of pretending one per-call median is a whole damage cascade.
    print(f'{name}_mean_total_per_change_ms={total / len(intervals) / 1e6:.6f}')
PYREPORT
    done
done
finish
printf 'PASS: noise placement cost (pilot=%s)\n' "$PILOT"
