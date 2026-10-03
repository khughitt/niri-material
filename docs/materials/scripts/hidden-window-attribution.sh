#!/usr/bin/env bash
# hidden-window-attribution.sh: does a material window nobody can see still
# cost compositor or client work? (material-d09741)
#
#   CAPTURE_TASK=material-d09741 NIRI_MATERIAL_WORK_ROOT=<evidence root> \
#   OUT=<fresh dir> docs/materials/scripts/hidden-window-attribution.sh pilot|matrix
#
# One workload per case: weston-simple-egl (app-id
# org.freedesktop.weston.simple-egl) under a pinned glass material with
# roughness, so the shared prefilter path runs. It paces on frame callbacks
# and prints its own frame rate every 5 s, which separates client work from
# compositor work. A static kitty (gos-other) holds focus. Each case is a
# fresh nested niri under a headless Weston host, traced by Tracy for 30 s;
# zones are counted in the final 20 s.
#
# pilot: visible (positive control), inactive-workspace, empty (no probe).
# matrix: the pilot plus hidden-tab, offscreen-column, covered-opaque,
# covered-alpha and overview (positive control). Only the controls and the
# culled inactive-workspace case gate the run; the others are findings.
#
# HWA_REHEARSAL=1 stubs capture-meta and skips the GPU rest wait, so the case
# setups and the analysis can be exercised on a busy desktop. Its counts are
# not evidence: the run carries no preflight, settle or provenance record.
#
# A second lit output cannot be nested (winit makes one output); that
# control rests on the per-output render path, not on this capture.
set -eu
MODE=${1:?pilot or matrix}
case $MODE in
    pilot) CASES=(visible inactive-workspace empty) ;;
    matrix) CASES=(visible inactive-workspace empty hidden-tab offscreen-column covered-opaque covered-alpha overview) ;;
    *) echo "FAIL: mode must be pilot or matrix, got $MODE" >&2; exit 1 ;;
esac
cd "$(git rev-parse --show-toplevel)"
if [ "${HWA_REHEARSAL:-0}" = 1 ]; then CAPTURE_META=true; echo "REHEARSAL: no capture record; counts are not evidence" >&2; fi
. docs/materials/scripts/glass-optic-smoke-lib.sh
capture_preflight headless

PROBE=org.freedesktop.weston.simple-egl
IDLE='printf "\033[?25l"; exec sleep 1800'
GLASS_EXTRA='roughness 0.3'
export RUST_LOG=niri=debug,smithay::backend::renderer::gles=info

build_tracy() {
    local target
    target=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
    cargo build --release --features profile-with-tracy
    cp "$target/release/niri" "$OUT/niri-tracy"
    NIRI=$OUT/niri-tracy; NIRI_TRACY=
    await_load   # a build's tail refuses the first settle
}
await_load() {
    [ "${HWA_REHEARSAL:-0}" != 1 ] || return 0
    for _ in $(seq 60); do
        awk '{ exit !($1 < 1.0) }' /proc/loadavg && return 0
        sleep 5
    done
    fail 'load1 did not fall below 1.0 within 5 min of the build'
}

# The lib's write_config matches gos-probe; this scene needs the probe rule on
# simple-egl and two floating covers at the output's top-left, full size.
write_scene() {
    local glass; glass=$(emit_glass)
    cat > "$1" <<KDL
prefer-no-csd
layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.4; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
material "hwa-probe" {
    glass {
$glass
    }
    response "default" {
        focus "none"
        accent "none"
        ring-beam-speed 0
    }
}
window-rule {
    match app-id=r"^${PROBE//./\\.}$"
    material "hwa-probe"
    geometry-corner-radius 0
}
window-rule {
    match app-id="^gos-cover-"
    open-floating true
    default-column-width { fixed 1280; }
    default-window-height { fixed 720; }
    default-floating-position x=0 y=0 relative-to="top-left"
}
KDL
}

win() { msg "$NIRI" -j windows | jq -r --argjson id "$1" ".[] | select(.id==\$id) | $2"; }
wid_of() { msg "$NIRI" -j windows | jq -r --arg a "$1" '[.[] | select(.app_id==$a)] | if length==1 then .[0].id else error("expected one \($a), got \(length)") end'; }
assert_eq() { [ "$1" = "$2" ] || fail "$3: expected '$2', got '$1'"; }
spawn_kitty() {   # class, opacity
    msg "$NIRI" action spawn -- kitty --config NONE --class "$1" -o background_opacity="$2" \
        -o cursor_blink_interval=0 -o mouse_hide_wait=0 sh -c "$IDLE"
    for _ in $(seq 100); do [ "$(windows_with "$NIRI" "$1")" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" "$1")" -eq 1 ] || fail "expected one $1 window"
}
spawn_probe() {
    msg "$NIRI" action spawn -- sh -c "exec weston-simple-egl > '$OUT/$1.egl.log' 2>&1"
    for _ in $(seq 100); do [ "$(windows_with "$NIRI" "$PROBE")" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$NIRI" "$PROBE")" -eq 1 ] || fail "expected one probe window"
}

# --- cases: each leaves the probe in its state and gos-other focused -------
setup_visible() { msg "$NIRI" action focus-window --id "$OTHER"; }
setup_empty() { :; }
setup_inactive_workspace() {
    msg "$NIRI" action focus-window --id "$WID"
    msg "$NIRI" action move-window-to-workspace-down --focus false
    [ "$(win "$WID" .workspace_id)" != "$(win "$OTHER" .workspace_id)" ] || fail "probe still on the active workspace"
    assert_eq "$(win "$OTHER" .is_focused)" true "focus stayed on gos-other"
}
setup_hidden_tab() {
    msg "$NIRI" action focus-window --id "$OTHER"
    msg "$NIRI" action consume-or-expel-window-left
    msg "$NIRI" action toggle-column-tabbed-display
    msg "$NIRI" action focus-window --id "$OTHER"
    assert_eq "$(win "$WID" '.layout.pos_in_scrolling_layout[0]')" "$(win "$OTHER" '.layout.pos_in_scrolling_layout[0]')" "same column"
}
setup_offscreen_column() {
    local k workspace view width
    for k in 2 3 4; do spawn_kitty "gos-col$k" 1; done
    msg "$NIRI" action focus-column-last
    assert_eq "$(win "$WID" '.layout.pos_in_scrolling_layout[0]')" 1 "probe leftmost"
    sleep 2
    workspace=$(win "$WID" .workspace_id)
    view=$(msg "$NIRI" -j workspaces | jq -r --argjson id "$workspace" '.[] | select(.id==$id) | .scrolling_view_pos')
    width=$(win "$WID" '.layout.tile_size[0]')
    awk -v view="$view" -v width="$width" 'BEGIN { exit !(width-view <= 0) }' \
        || fail "probe still in view (view=$view width=$width)"
}
setup_covered_opaque() { cover gos-cover-opaque 1; }
setup_covered_alpha() { cover gos-cover-alpha 0.5; }
cover() {
    spawn_kitty "$1" "$2"
    local c; c=$(wid_of "$1")
    sleep 1
    assert_eq "$(win "$c" .is_floating)" true "cover floats"
    assert_eq "$(win "$c" '.layout.window_size | map(floor) | join("x")')" 1280x720 "cover fills the output"
}
setup_overview() { msg "$NIRI" action focus-window --id "$OTHER"; msg "$NIRI" action open-overview; }

check_renderer() {
    tail -c "+$((LOG_OFFSET + 1))" "$OUT/niri.log" > "$OUT/$1.renderer.log"
    rg -q 'GL Renderer:.*NVIDIA' "$OUT/$1.renderer.log" || fail "$1: missing NVIDIA compositor renderer identity"
    if rg -qi 'llvmpipe|software rasterizer|error compiling material shader|material.*fallback|panic' "$OUT/$1.renderer.log"; then
        fail "$1: renderer failure"
    fi
}

run_case() {
    local name=$1 fn=setup_${1//-/_}
    touch "$OUT/niri.log"; LOG_OFFSET=$(wc -c < "$OUT/niri.log")
    start_nested "$NIRI" "$OUT/scene.kdl" "$name"
    # The probe opens first, so it is the leftmost column in every case.
    if [ "$name" != empty ]; then spawn_probe "$name"; WID=$(wid_of "$PROBE"); fi
    spawn_kitty gos-other 1; OTHER=$(wid_of gos-other)
    "$fn"
    sleep 3
    msg "$NIRI" -j windows > "$OUT/$name.windows.json"
    shot "$NIRI" "$name"
    capture_bg "$name"; capture_ready "$name"; capture_wait
    check_renderer "$name"
    stop_nested
    await_gpu_rest
}

await_gpu_rest() {
    [ "${HWA_REHEARSAL:-0}" != 1 ] || return 0
    local streak=0
    for _ in $(seq 60); do
        if [ "$(nvidia-smi --query-gpu=pstate --format=csv,noheader)" = P8 ]; then
            streak=$((streak + 1)); [ "$streak" -lt 6 ] || return 0
        else
            streak=0
        fi
        sleep .5
    done
    fail 'GPU did not return to P8 within 30 s of the last compositor exit'
}

# --- analysis ----------------------------------------------------------------
CPU_ZONES=(Niri::redraw Tile::render OffscreenBuffer::render CompositorHandler::commit
    Niri::send_frame_callbacks Niri::send_frame_callbacks_on_fallback_timer)
GPU_ZONES=(MaterialRenderElement::draw Prefilter::downsample)
count_zones() {   # name -> one tab-separated row of counts in the final 20 s
    local name=$1 end c csv=$OUT/$1.csv gpu=$OUT/$1.gpu.csv ct row=() z
    count_last20 "$name" > /dev/null   # exports and checks the heartbeat
    end=$(trace_end "$name"); c=$(col "$csv" ns_since_start)
    for z in "${CPU_ZONES[@]}"; do
        row+=("$(awk -F, -v c="$c" -v end="$end" -v z="$z" \
            'NR>1 && $1==z && $c+0>=end-20e9 { n++ } END { printf "%d", n+0 }' "$csv")")
    done
    csvexport --gpu "$OUT/$name.tracy" "$gpu"
    ct=$(col "$gpu" "Time from start of program")
    for z in "${GPU_ZONES[@]}"; do
        row+=("$(awk -F, -v c="$ct" -v end="$end" -v z="$z" \
            'NR>1 && $1==z && $c+0>=end-20e9 { n++ } END { printf "%d", n+0 }' "$gpu")")
    done
    row+=("$(client_fps "$name")")
    local IFS=$'\t'; echo "${row[*]}"
}
client_fps() {   # median of the last three 5 s reports, or - for no probe
    [ -s "$OUT/$1.egl.log" ] || { echo -; return; }
    rg -o '[0-9.]+ fps' "$OUT/$1.egl.log" | tail -3 | awk '{ print $1 }' | sort -n | sed -n 2p | grep . || echo 0
}
col_of() { local i; for i in "${!HEADER[@]}"; do [ "${HEADER[$i]}" = "$1" ] && echo $((i + 2)) && return; done; fail "no column $1"; }
verdict() {   # case -> assert the gated expectations on its row
    local draw off fps
    draw=$(awk -F'\t' -v c="$1" -v k="$(col_of MaterialRenderElement::draw)" '$1==c { print $k }' "$OUT/results.tsv")
    off=$(awk -F'\t' -v c="$1" -v k="$(col_of OffscreenBuffer::render)" '$1==c { print $k }' "$OUT/results.tsv")
    fps=$(awk -F'\t' -v c="$1" -v k="$(col_of client_fps)" '$1==c { print $k }' "$OUT/results.tsv")
    case $1 in
        visible|overview) assert_positive "$1 material draws" "$draw"; assert_greater "$1 client fps" "$fps" 20 ;;
        inactive-workspace) assert_zero "$1 material draws" "$draw"; assert_zero "$1 offscreen renders" "$off" ;;
        empty) assert_zero "$1 material draws" "$draw" ;;
    esac
}

build_tracy
capture_identity
tools_ready; reserve_tracy_port
write_scene "$OUT/scene.kdl"
"$NIRI" validate -c "$OUT/scene.kdl" || fail "scene does not validate"
for name in "${CASES[@]}"; do run_case "$name"; done
HEADER=("${CPU_ZONES[@]}" "${GPU_ZONES[@]}" client_fps)
{ local_ifs=$IFS; IFS=$'\t'; echo "case	${HEADER[*]}"; IFS=$local_ifs
  for name in "${CASES[@]}"; do printf '%s\t%s\n' "$name" "$(count_zones "$name")"; done; } > "$OUT/results.tsv"
column -t -s $'\t' "$OUT/results.tsv"
for name in "${CASES[@]}"; do verdict "$name"; done
finish
