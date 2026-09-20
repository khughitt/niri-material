#!/usr/bin/env bash
# focus-ring-light.sh: measured captures for the ring-of-light focus response
# (docs/plans/2026-09-05-ring-light-focus-response.md). It nests niri under a
# headless Weston host, opens two transparent kitty windows over the dark
# split glass from the focus-glass spike, and measures the filament in
# linear light.
#
# Every measured case renders the same scene twice, once with the filament
# and once with it disabled (`response "default" { focus "none"; accent
# "none"; }`), so refraction, attenuation, Fresnel, and jelly are identical in
# both and only the filament differs. Rest scenes are deterministic, so the
# disabled render is taken at rest in its own nested instance, and every case
# pins `ring-beam-speed 0` so the beam never enters a comparison.
#
# Cases:
#   baseline          the spike's stills and bursts (static gradient ring)
#   rest-confinement  at rest the filament stays within its derived reach,
#                     and the band is present and untinted; writes the shared `off`
#                     reference frame the later cases compare against
#   accent-midfade    the presence crossfade carries straight color
#   resize-flex       under jelly the filament breathes; motion evidence is recorded
#   selectors         accent and focus select independently
#   tiny              recorded as not verified by render (no client is small
#                     enough to drive the shader's zero-chamfer gate)
#
# Env: NIRI (default the release build of this checkout), CASES
# (space-separated, default all; the measured cases need rest-confinement's
# `off` frame, so run it first or keep the default order),
# NIRI_MATERIAL_WORK_ROOT (captures land under it when set, else ./work),
# SPIKE_WALLPAPER (backdrop image; default a generated checkerboard),
# BURST (frames per burst, 12).
# Requires: weston, kitty, swaybg, jq, ImageMagick, bc, and a Prism prism.kdl
# in $XDG_STATE_HOME/prism/generated for the "terminal-glass" block.
set -eu
# Measurement helpers run inside `$(...)`, where a plain `exit` would only end
# the subshell and let the caller carry on with an empty value, so failures go
# through `die`, which signals the top-level shell.
TOP_PID=$$
trap 'exit 1' TERM
die() { echo "FAIL: $*" >&2; kill -s TERM "$TOP_PID" 2>/dev/null || true; exit 1; }
HERE=$(dirname "$(readlink -f "$0")")
REPO=$(cd "$HERE/../../.." && pwd)
NIRI=${NIRI:-$(cd "$REPO" && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/niri}
CONTENT=$HERE/focus-glass-content.sh
SHA=$(sha256sum "$NIRI" | cut -c1-8)
WORK=${NIRI_MATERIAL_WORK_ROOT:-$PWD/work}/focus-ring-light-$SHA; mkdir -p "$WORK"
RT=$XDG_RUNTIME_DIR/frl-rt; rm -rf "$RT"; mkdir -p "$RT"
PRISM_KDL=${XDG_STATE_HOME:-$HOME/.local/state}/prism/generated/prism.kdl
BURST=${BURST:-12}
LOG=$WORK/harness.log
WALL=${SPIKE_WALLPAPER:-}
if [ -z "$WALL" ]; then WALL=$WORK/checker.png; magick -size 160x90 pattern:checkerboard -scale 800% "$WALL"; fi

# --- pinned glass and sampling geometry ------------------------------------
# The prism block is regenerated day to day and its geometry sets where the
# filament lands, so the three fields the sampling constants depend on are
# pinned here and the written config is checked for them.
PIN_IOR=1.02; PIN_THICKNESS=41.7; PIN_BEVEL=11
PIN_OFFSET=1                              # the prism block's offset-x/offset-y
# `ring-gap` is measured from the face edge (the slab minus its chamfer); the
# retired inset key this replaced was measured from the slab's outer edge.
# The sampling row and the reach bound below still carry the slab-edge
# derivation and need re-deriving before the measured cases are next graded.
RING_GAP=5
RING_WIDTH=2.6
# The layout below (1280x720 host, gaps 54, two columns at proportion 0.5)
# puts the focused right window at this rect; `wait_geometry` refuses to
# measure if it ever moves.
WIN_X=667; WIN_Y=54; WIN_W=559; WIN_H=612
FACE_INSET=40                             # the face region clears the bevel and the corner radius
FACE_CROP=$((WIN_W - 2 * FACE_INSET))x$((WIN_H - 2 * FACE_INSET))+$((WIN_X + FACE_INSET))+$((WIN_Y + FACE_INSET))
# `material_frame`: the slab is the window inflated by `bevel - max(|offset|)`
# and slid by the offset, so its top edge sits this far above the window.
SLAB_TOP=$((WIN_Y - (PIN_BEVEL - PIN_OFFSET) + PIN_OFFSET))
FIL_X=$((WIN_X + WIN_W / 2))              # top edge of the right window, mid-span
FIL_Y=$((SLAB_TOP + RING_GAP))            # the band's Gaussian core
# The strip spanning the left column's right edge and the gap to the right
# column: where a difference in layout position between two hosts shows up.
GAP_CROP=(440 300 200 120)                # x y w h

declare -a I_UNIT=() I_PID=() I_SOCK=()
stop_nested() {   # $1 = slot
    local slot=$1
    [ -n "${I_PID[$slot]:-}" ] && { kill "${I_PID[$slot]}" 2>/dev/null || true; wait "${I_PID[$slot]}" 2>/dev/null || true; }
    [ -n "${I_UNIT[$slot]:-}" ] && { systemctl --user stop "${I_UNIT[$slot]}" 2>/dev/null || true; }
    I_PID[$slot]=; I_UNIT[$slot]=; I_SOCK[$slot]=
    rm -rf "${RT:?}/$slot"
    sleep 0.5
}
cleanup() {
    local slot
    for slot in 1 2; do stop_nested "$slot" || true; done
    rm -rf "$RT"
}
trap cleanup EXIT
msg() { "$NIRI" msg "$@"; }
use_slot() { NIRI_SOCKET=${I_SOCK[$1]}; export NIRI_SOCKET; }
kitty_ids() { msg -j windows | jq -r '.[] | select(.app_id=="kitty") | .id'; }
wait_kitty() { for _ in $(seq 100); do [ "$(kitty_ids | wc -l)" -ge "$1" ] && return; sleep 0.1; done; echo "FAIL: kitty $1" >&2; exit 1; }
# The rightmost kitty column: the window every measurement samples.
right_id() { msg -j windows | jq -r '[.[] | select(.app_id=="kitty")] | sort_by(.layout.pos_in_scrolling_layout[0]) | last | .id'; }

ACTIVE=$(sed -n '/^material "terminal-glass"/,/^}/p' "$PRISM_KDL")
[ -n "$ACTIVE" ] || { echo "FAIL: no terminal-glass in $PRISM_KDL" >&2; exit 1; }
DARK_ACTIVE=$(printf '%s\n' "$ACTIVE" | sed \
    -e 's/"terminal-glass"/"dark"/' \
    -e 's/attenuation-color .*/attenuation-color "#222436"/' \
    -e 's/attenuation-distance .*/attenuation-distance 30/' \
    -e "s/^\\( *\\)ior .*/\\1ior $PIN_IOR/" \
    -e "s/^\\( *\\)thickness .*/\\1thickness $PIN_THICKNESS/" \
    -e "s/^\\( *\\)bevel .*/\\1bevel $PIN_BEVEL/" \
    -e "s/^\\( *\\)offset-x .*/\\1offset-x $PIN_OFFSET/" \
    -e "s/^\\( *\\)offset-y .*/\\1offset-y $PIN_OFFSET/")
DARK_INACTIVE=$(printf '%s\n' "$DARK_ACTIVE" | sed \
    -e 's/"dark"/"dark-inactive"/' \
    -e 's/roughness .*/roughness 0.5/' \
    -e 's/chromatic-aberration .*/chromatic-aberration 0.08/' \
    -e 's/distortion .*/distortion 0.10 scale=0.03/' \
    -e 's/attenuation-distance .*/attenuation-distance 70/')
# The measurement flavour of the same glass: a neutral attenuation color so
# the exponent scales all channels equally and the filament's hue survives,
# no chromatic aberration, and no roughness so the derived rest bound is exact.
measurable() { printf '%s\n' "$1" | sed -e 's/attenuation-color .*/attenuation-color "#888888"/' -e 's/chromatic-aberration .*/chromatic-aberration 0/' -e 's/roughness .*/roughness 0/'; }
MEAS_ACTIVE=$(measurable "$DARK_ACTIVE")
MEAS_INACTIVE=$(measurable "$DARK_INACTIVE")

# Replace Prism's inherited default response, then append the pinned one.
with_response() {   # $1 = material block, $2 = response body
    printf '%s\n' "$1" | sed '/^    response "default" {$/,/^    }$/d' | sed '$d'
    printf '    response "default" {\n        ring-color "#ccccff"\n        ring-gap %s\n        ring-width %s\n        %s\n    }\n}\n' \
        "$RING_GAP" "$RING_WIDTH" "$2"
}
assert_pinned_geometry() {   # $1 = written config; both materials carry the calibrated slab
    local k n
    for k in "ior $PIN_IOR" "thickness $PIN_THICKNESS" "bevel $PIN_BEVEL" \
             "offset-x $PIN_OFFSET" "offset-y $PIN_OFFSET"; do
        n=$(sed 's/^ *//' "$1" | grep -cxF "$k" || true)
        [ "$n" -eq 2 ] || { echo "FAIL: $1: expected 2 lines of '$k', found $n" >&2; exit 1; }
    done
}
assert_rest_glass() { # $1 = written config; measurement scenes are unscattered
    assert_pinned_geometry "$1"
    local n
    n=$(sed 's/^ *//' "$1" | grep -cxF 'roughness 0' || true)
    [ "$n" -eq 2 ] || { echo "FAIL: $1: expected 2 lines of 'roughness 0', found $n" >&2; exit 1; }
}
assert_pinned_response() { # $1 = written config, $2 = expected beam speed
    local k n
    for k in 'ring-color "#ccccff"' "ring-gap $RING_GAP" "ring-width $RING_WIDTH" "ring-beam-speed $2;"; do
        n=$(grep -cF "$k" "$1" || true)
        [ "$n" -eq 2 ] || { echo "FAIL: $1: expected 2 response bodies containing '$k', found $n" >&2; exit 1; }
    done
}
validate() { "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; exit 1; }; }

layout_block() {   # $1 = focus-ring block
    cat <<KDL
layout {
    gaps 54
    default-column-width { proportion 0.5; }
    $1
    border { off; }
    shadow { on; softness 5; spread 1; offset x=-5 y=5; color "#00000070"; }
}
hotkey-overlay { skip-at-startup; }
window-rule { match app-id="^kitty\$"; geometry-corner-radius 12; clip-to-geometry true; }
window-rule { match app-id="^kitty\$" is-active=true; material "dark"; }
window-rule { match app-id="^kitty\$" is-active=false; material "dark-inactive"; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
KDL
}
# The retained spike config: the static gradient focus ring over the dark glass.
write_baseline_config() {   # $1 = path
    {
        layout_block 'focus-ring { width 4; active-gradient from="#ffffff00" to="#ccccff11" angle=45 relative-to="workspace-view"; }'
        echo 'animations { slowdown 6.0; }'
        printf '%s\n%s\n' "$DARK_ACTIVE" "$DARK_INACTIVE"
    } > "$1"
    assert_pinned_geometry "$1"; validate "$1"
}
# The measurement config: no gradient ring, the measurable glass, and the
# case's response on both materials.
write_capture_config() {   # $1 = path, $2 = response body, $3 = animations body
    local speed=0
    case $2 in
        *'ring-beam-speed 0;'*) ;;
        *'ring-beam-speed 3000;'*) speed=3000 ;;
        *) echo "FAIL: response must set ring-beam-speed to 0 or 3000" >&2; exit 1 ;;
    esac
    {
        layout_block 'focus-ring { off; }'
        printf 'animations { %s }\n' "$3"
        with_response "$MEAS_ACTIVE" "$2"
        with_response "$MEAS_INACTIVE" "$2"
    } > "$1"
    assert_rest_glass "$1"; assert_pinned_response "$1" "$speed"; validate "$1"
}
RESP_ON='ring-beam-speed 0;'
RESP_OFF='focus "none"; accent "none"; ring-beam-speed 0;'

start_nested() {   # $1 = slot, $2 = config
    local slot=$1 cfg=$2 host unit
    host=frl-host-$$-$slot; unit=$host-weston
    systemd-run --user --unit="$unit" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$host" >/dev/null 2>&1
    I_UNIT[$slot]=$unit
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$host" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$host" ] || { echo "FAIL: no Weston socket for slot $slot" >&2; exit 1; }
    mkdir -p "$RT/$slot"
    ln -sf "$XDG_RUNTIME_DIR/$host" "$RT/$slot/$host"
    XDG_RUNTIME_DIR=$RT/$slot WAYLAND_DISPLAY=$host "$NIRI" -c "$cfg" >> "$LOG" 2>&1 &
    I_PID[$slot]=$!
    for _ in $(seq 100); do ls "$RT/$slot"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    ls "$RT/$slot"/niri.*.sock >/dev/null 2>&1 || { echo "FAIL: no niri socket for slot $slot" >&2; exit 1; }
    I_SOCK[$slot]=$(ls -t "$RT/$slot"/niri.*.sock | head -1)
    use_slot "$slot"
    sleep 1
}
spawn_kitty() {   # $1 label
    msg action spawn -- kitty --config NONE -o background_opacity=0 -o background=#222436 -o foreground=#c8d3f5 \
        -o cursor_blink_interval=0 -o hide_window_decorations=yes -o font_size=11 -o window_padding_width=8 --hold "$CONTENT" "$1"
}
px() { printf '%s/%s.png' "$WORK" "$1"; }
shot_request() { rm -f "$1"; msg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"; }
# The write is async: wait for a complete, decodable PNG, not just a non-empty file.
shot_wait() { for _ in $(seq 200); do [ -s "$1" ] && magick identify "$1" >/dev/null 2>&1 && return; sleep 0.05; done; echo "FAIL: shot $1" >&2; exit 1; }
shot() { shot_request "$1"; shot_wait "$1"; }
# A rest frame is one the compositor no longer changes: two shots 0.6 s apart
# that are byte-identical. Slowed-down open and resize animations make a fixed
# sleep a guess, and every rest comparison here depends on being at rest.
settle() {
    local a=$WORK/.settle-a.png b=$WORK/.settle-b.png
    for _ in $(seq 40); do
        shot "$a"; sleep 0.6; shot "$b"
        cmp -s "$a" "$b" && { rm -f "$a" "$b"; return; }
    done
    echo "FAIL: scene never settled" >&2; exit 1
}
# The sampling constants above assume this exact layout. This host reports a
# null `tile_pos_in_workspace_view`, so the rect is pinned from what the IPC
# does report: with a 1280x720 output at scale 1, gaps 54, and two
# proportional columns, the second column's tile fixes WIN_X and WIN_Y.
win_layout() {   # $1 = window id
    msg -j windows | jq -r --argjson id "$1" '.[] | select(.id==$id) | .layout
        | "\(.pos_in_scrolling_layout[0]) \(.pos_in_scrolling_layout[1]) \(.window_size[0]) \(.window_size[1]) \(.window_offset_in_tile[0] | floor) \(.window_offset_in_tile[1] | floor)"'
}
window_layout_json() { # $1 = window id; target layout, not animated render geometry
    msg -j windows | jq -c --argjson id "$1" '.[] | select(.id==$id) | .layout'
}
wait_geometry() {   # $1 = window id: the layout is final within ten seconds or the run stops
    local want="2 1 $WIN_W $WIN_H 0 0" got out
    for _ in $(seq 100); do got=$(win_layout "$1"); [ "$got" = "$want" ] && break; sleep 0.1; done
    [ "$got" = "$want" ] || { echo "FAIL: right window layout is '$got', the sampling constants assume '$want'" >&2; exit 1; }
    out=$(msg -j outputs | jq -r '.[] | "\(.logical.x) \(.logical.y) \(.logical.width) \(.logical.height) \(.logical.scale | floor)"')
    [ "$out" = "0 0 1280 720 1" ] || { echo "FAIL: output is '$out', the sampling constants assume '0 0 1280 720 1'" >&2; exit 1; }
}
# Two windows over the split glass with the right one focused: the scene every
# measured case renders. Prints the focused window's id. Rest is a separate
# step: the baseline case runs the default focus beam before coming to rest.
open_scene() {
    spawn_kitty left;  wait_kitty 1
    spawn_kitty right; wait_kitty 2
    local right; right=$(right_id)
    msg action focus-window --id "$right"
    wait_geometry "$right"
    echo "$right"
}

# --- measurement -----------------------------------------------------------
plin() {   # $1 image, $2 x, $3 y: linear-light r g b in 0..1 (PNG is sRGB; -colorspace RGB decodes it)
    magick "$1" -colorspace RGB -format '%[fx:p{'"$2"','"$3"'}.r] %[fx:p{'"$2"','"$3"'}.g] %[fx:p{'"$2"','"$3"'}.b]' info:
}
# Exactly three numbers, or the reading is not a reading. A failing `magick
# -fx` prints nothing, and `paste` pads that into a short or one-sided triple
# that `red_fraction` reports as 0, which would let a "<= 0.005" check pass
# vacuously; every triple is therefore asserted before any check sees it.
assert_triple() {   # $1 = value, $2 = where it came from
    awk -v e="$1" 'BEGIN {
        n = split(e, c, " ")
        if (n != 3) exit 1
        for (i = 1; i <= 3; i++)
            if (c[i] !~ /^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$/) exit 1
    }' || die "$2: expected three numbers, got '$1' (magick failed?)"
}
# Emissive contribution at a pixel: linear(with filament) - linear(same scene, filament disabled).
emissive() {
    local a b e
    a=$(plin "$1" "$3" "$4"); assert_triple "$a" "$1 at ($3,$4)"
    b=$(plin "$2" "$3" "$4"); assert_triple "$b" "$2 at ($3,$4)"
    e=$(paste -d' ' <(printf '%s\n' "$a" | tr ' ' '\n') <(printf '%s\n' "$b" | tr ' ' '\n') | awk '{ printf "%f ", $1 - $2 }')
    assert_triple "$e" "emissive at ($3,$4)"
    printf '%s' "$e"
}
red_fraction() { awk -v e="$1" 'BEGIN { split(e, c, " "); s = c[1] + c[2] + c[3]; if (s <= 0) { print 0; exit }; printf "%.3f", c[1] / s }'; }
emissive_lum() { awk -v e="$1" 'BEGIN { split(e, c, " "); printf "%.4f", 0.2126 * c[1] + 0.7152 * c[2] + 0.0722 * c[3] }'; }
face_ae() {   # $1 a, $2 b: count of differing pixels over the face region
    # `compare` reports the metric on stderr as "count (normalised)".
    magick compare -metric AE \( "$1" -crop "$FACE_CROP" +repage \) \( "$2" -crop "$FACE_CROP" +repage \) null: 2>&1 | awk '{ print $1 }'
}
reach() { # $1 label, $2 on, $3 off
    local rc=0
    python3 "$HERE/glass-render-order-metrics.py" reach "$2" "$3" \
        --window "$WIN_X" "$WIN_Y" "$WIN_W" "$WIN_H" --bevel "$PIN_BEVEL" \
        --thickness "$PIN_THICKNESS" --inset "$RING_GAP" --width "$RING_WIDTH" \
        --scatter 0 --offset-x "$PIN_OFFSET" --offset-y "$PIN_OFFSET" --corner-radius 12 \
        > "$WORK/$1-reach.json" || rc=$?
    cat "$WORK/$1-reach.json" >> "$WORK/checks.txt"
    [ "$rc" -eq 0 ] || die "$1 exceeded its derived rest reach bound"
}
motion_record() { # label, on, off, on slot/id, off slot/id; records, never measures reach
    local label=$1 on=$2 off=$3 on_slot=$4 on_id=$5 off_slot=$6 off_id=$7
    local on_geometry off_geometry changed
    use_slot "$on_slot"; on_geometry=$(window_layout_json "$on_id")
    use_slot "$off_slot"; off_geometry=$(window_layout_json "$off_id")
    changed=$(magick compare -metric AE "$on" "$off" null: 2>&1 || true)
    info "$label on target layout" "$on_geometry" "raw IPC target layout adjacent to screenshot request"
    info "$label off target layout" "$off_geometry" "raw IPC target layout adjacent to screenshot request"
    info "$label animated slab geometry" "unavailable" "IPC target layout is not the renderer's animated window geometry; no interior reach is measured"
    info "$label whole-frame AE" "$changed" "whole-frame paired-host delta only; not an interior metric or gate"
}
motion_pair_burst() { # label, on slot/id, off slot/id
    local label=$1 on_slot=$2 on_id=$3 off_slot=$4 off_id=$5 d i t0 t1 on off
    d=$WORK/$label; mkdir -p "$d"
    t0=$(date +%s.%N)
    for i in $(seq 1 "$BURST"); do
        on=$d/on-f$(printf %02d "$i").png; off=$d/off-f$(printf %02d "$i").png
        use_slot "$on_slot"; info "$label-f$(printf %02d "$i") on screenshot request" "$(date --iso-8601=seconds)" "paired hosts are not lockstep"; shot_request "$on"
        use_slot "$off_slot"; info "$label-f$(printf %02d "$i") off screenshot request" "$(date --iso-8601=seconds)" "paired hosts are not lockstep"; shot_request "$off"
        shot_wait "$on"; shot_wait "$off"
        motion_record "$label-f$(printf %02d "$i")" "$on" "$off" \
            "$on_slot" "$on_id" "$off_slot" "$off_id"
    done
    t1=$(date +%s.%N)
    echo "$label: $BURST paired frames in $(echo "$t1 - $t0" | bc) s" >> "$WORK/timing.txt"
}
face_max() {   # $1 a, $2 b: max per-channel difference over the face region, 0..255
    magick \( "$1" -crop "$FACE_CROP" +repage \) \( "$2" -crop "$FACE_CROP" +repage \) -compose difference -composite -format '%[fx:int(255*maxima)]' info:
}
# How far apart two hosts are in a running animation: the horizontal shift, in
# whole pixels, that best aligns the second frame's strip onto the first's.
# Hosts in lockstep align at 0 with a zero residual.
align_skew() {   # $1 a, $2 b, $3 x, $4 y, $5 w, $6 h
    local k d best= best_k=0
    for k in $(seq -8 8); do
        d=$(magick \( "$1" -crop "${5}x${6}+${3}+${4}" +repage \) \
                   \( "$2" -crop "${5}x${6}+$(($3 + k))+${4}" +repage \) \
                   -compose difference -composite -format '%[fx:mean]' info:)
        if [ -z "$best" ] || awk -v a="$d" -v b="$best" 'BEGIN { exit !(a < b) }'; then best=$d; best_k=$k; fi
    done
    echo "$best_k"
}
record() { printf '%s\n' "$1" | tee -a "$WORK/checks.txt"; }
info() { record "info: $1 = $2 ($3)"; }   # recorded, never gated
check() {   # $1 label, $2 value, $3 awk condition on v, $4 human bound
    awk -v v="$2" "BEGIN { exit !($3) }" || { record "FAIL: $1 = $2, want $4"; exit 1; }
    record "ok: $1 = $2 (want $4)"
}
require_off() {
    [ -s "$(px off)" ] || { echo "FAIL: $(px off) is missing; run the rest-confinement case first" >&2; exit 1; }
}
rest_lum() {
    [ -s "$WORK/rest-lum.txt" ] || { echo "FAIL: $WORK/rest-lum.txt is missing; run the rest-confinement case first" >&2; exit 1; }
    cat "$WORK/rest-lum.txt"
}

# --- cases -----------------------------------------------------------------
still() {   # $1 label, $2 side (left|right): full frame plus 3x top corner crop of that side
    local f=$WORK/$1.png; shot "$f"
    if [ "$2" = right ]; then
        magick "$f" -crop 260x160+640+30 +repage -scale 300% "$WORK/$1-corner.png"
    else
        magick "$f" -crop 260x160+30+30 +repage -scale 300% "$WORK/$1-corner.png"
    fi
    sha256sum "$f" >> "$WORK/SHA256SUMS"
}
burst() {   # $1 label, $2 side: BURST frames as fast as the async screenshot allows
    local d=$WORK/$1; mkdir -p "$d"
    local geo; [ "$2" = right ] && geo=600x660+640+30 || geo=600x660+30+30
    local t0; t0=$(date +%s.%N)
    for i in $(seq 1 "$BURST"); do shot "$d/f$(printf %02d "$i").png"; done
    local t1; t1=$(date +%s.%N)
    echo "$1: $BURST frames in $(echo "$t1 - $t0" | bc) s" >> "$WORK/timing.txt"
    for f in "$d"/f*.png; do magick "$f" -crop "$geo" +repage "${f%.png}-crop.png"; done
    magick montage "$d"/f*-crop.png -tile 6x -geometry 300x330+2+2 -background '#111' "$WORK/$1-sheet.png"
    magick -delay 12 -loop 0 "$d"/f*-crop.png "$WORK/$1.gif"
    magick "$d/f01-crop.png" -crop 300x200+0+0 +repage -scale 300% "$WORK/$1-f01-corner.png"
    magick "$d/f$(printf %02d "$BURST")-crop.png" -crop 300x200+0+0 +repage -scale 300% "$WORK/$1-fN-corner.png"
}
# The spike's visual case, kept for the geometry check and the corner crops.
case_baseline() {
    write_baseline_config "$WORK/baseline.kdl"
    start_nested 1 "$WORK/baseline.kdl"
    open_scene >/dev/null
    still baseline-right-focused right
    burst baseline-idle right
    msg action focus-column-left; sleep 1.5
    still baseline-left-focused left
    # Jelly: push the focused (left) column right so it slides and the ring
    # sees residuals; capture from the moment the move starts.
    msg action move-column-right
    burst baseline-move right
    stop_nested 1
}
# At rest the filament stays within its derived reach and its color is the
# explicitly pinned ring color. Writes the shared `off` reference frame.
case_rest_confinement() {
    write_capture_config "$WORK/rest-on.kdl"  "$RESP_ON"  'slowdown 6.0;'
    write_capture_config "$WORK/rest-off.kdl" "$RESP_OFF" 'slowdown 6.0;'
    start_nested 1 "$WORK/rest-on.kdl";  open_scene >/dev/null; settle; shot "$(px on)";  stop_nested 1
    start_nested 1 "$WORK/rest-off.kdl"; open_scene >/dev/null; settle; shot "$(px off)"; stop_nested 1
    local e f l
    e=$(emissive "$(px on)" "$(px off)" "$FIL_X" "$FIL_Y")
    f=$(red_fraction "$e"); l=$(emissive_lum "$e")
    reach rest-confinement "$(px on)" "$(px off)"
    check "rest-confinement red share"        "$f" 'v >= 0.24 && v <= 0.31'                           "0.24 to 0.31"
    check "rest-confinement emissive luminance" "$l" 'v > 0.01'                                       "> 0.01"
    printf '%s\n' "$l" > "$WORK/rest-lum.txt"
    record "rest-confinement emissive at ($FIL_X,$FIL_Y) = $e"
}
# The presence crossfade carries straight color: at half fade the band's red
# share is the straight mix, not the doubly faded one.
case_accent_midfade() {
    require_off
    write_capture_config "$WORK/midfade.kdl" "$RESP_ON" 'slowdown 20; material-signal { duration-ms 400; curve "linear"; };'
    start_nested 1 "$WORK/midfade.kdl"
    local right; right=$(open_scene); settle
    msg set-window-signal --id "$right" --source demo --accent '#ff0000'
    sleep 4.0; shot "$(px mid)"          # presence 0.46 to 0.54 given screenshot latency
    sleep 6;   shot "$(px settled)"
    stop_nested 1
    local mid settled
    mid=$(red_fraction "$(emissive "$(px mid)" "$(px off)" "$FIL_X" "$FIL_Y")")
    settled=$(red_fraction "$(emissive "$(px settled)" "$(px off)" "$FIL_X" "$FIL_Y")")
    # Straight color at presence 0.46 to 0.54 gives 0.476 to 0.526; the
    # double-faded regression (RGB scaled by presence, then mixed by it)
    # gives at most 0.435.
    check "accent-midfade red share at half fade" "$mid"     'v >= 0.46 && v <= 0.54' "0.46 to 0.54"
    check "accent-midfade red share settled"      "$settled" 'v >= 0.90'              ">= 0.90"
}
# Under a slowed resize the jelly breath should brighten the filament while the
# face comparison is recorded, not gated. Only the rest comparison is gated. The two
# hosts run on independent clocks -- one `niri msg` process spawn separates the
# two resizes, and another the two screenshots -- so mid-resize they sit at
# different points in the animation: the layout is about a pixel apart and each
# client has re-rendered its text at a slightly different width. That swamps
# anything the filament could add to the face, so the mid-resize numbers are
# recorded with the measured skew beside them rather than gated. See
# docs/materials/2026-09-05-ring-light-focus-smoke.md, "resize-flex".
case_resize_flex() {
    local lum_rest; lum_rest=$(rest_lum)
    write_capture_config "$WORK/flex-on.kdl"  "$RESP_ON"  'slowdown 50;'
    write_capture_config "$WORK/flex-off.kdl" "$RESP_OFF" 'slowdown 50;'
    local right_on right_off
    start_nested 1 "$WORK/flex-on.kdl";  right_on=$(open_scene); settle
    start_nested 2 "$WORK/flex-off.kdl"; right_off=$(open_scene); settle
    use_slot 1; shot "$(px flex-rest-on)"
    use_slot 2; shot "$(px flex-rest-off)"
    reach resize-flex-rest "$(px flex-rest-on)" "$(px flex-rest-off)"
    use_slot 1; msg action set-column-width +200
    use_slot 2; msg action set-column-width +200
    sleep 3                              # jelly at high flex; the 20 s resize is far from settled
    use_slot 1; shot_request "$(px flex-on)"
    use_slot 2; shot_request "$(px flex-off)"
    shot_wait "$(px flex-on)"; shot_wait "$(px flex-off)"
    motion_pair_burst resize-flex-motion 1 "$right_on" 2 "$right_off"
    stop_nested 1; stop_nested 2
    local lum ratio
    lum=$(emissive_lum "$(emissive "$(px flex-on)" "$(px flex-off)" "$FIL_X" "$FIL_Y")")
    ratio=$(awk -v a="$lum" -v b="$lum_rest" 'BEGIN { printf "%.3f", a / b }')
    info "resize-flex host layout skew at rest" "$(align_skew "$(px flex-rest-on)" "$(px flex-rest-off)" "${GAP_CROP[@]}") px" "the hosts agree exactly before the resize"
    info "resize-flex host layout skew in flex" "$(align_skew "$(px flex-on)" "$(px flex-off)" "${GAP_CROP[@]}") px" "0 would be lockstep; the resize travels 200 px in about 20 s"
    info "resize-flex face pixels differing"     "$(face_ae "$(px flex-on)" "$(px flex-off)")" "not gated: the two clients re-render their text at different points in the resize"
    info "resize-flex face max channel delta"    "$(face_max "$(px flex-on)" "$(px flex-off)")" "not gated: same cause"
    info "resize-flex emissive luminance vs rest ($lum_rest)" "${ratio}x" "not gated: sampled across the skew"
}
# Focus toggles for the moving bursts: the beam runs (P + L) / speed, about a
# second on this pane at 3000 px/s, so motion throughout a burst comes from
# re-gaining focus every half second on both hosts (the off host toggles too,
# so the paired scenes' active materials stay in step). Runs in the
# background for the burst; the caller stops it before the next layout
# command, which acts on the focused column.
TOGGLES_PID=
toggles_start() {   # $1 on slot, $2 on right id, $3 off slot, $4 off right id
    (
        local on_left off_left
        use_slot "$1"; on_left=$(kitty_ids | grep -vx "$2" | head -1)
        use_slot "$3"; off_left=$(kitty_ids | grep -vx "$4" | head -1)
        while :; do
            use_slot "$1"; msg action focus-window --id "$on_left"
            use_slot "$3"; msg action focus-window --id "$off_left"
            sleep 0.25
            use_slot "$1"; msg action focus-window --id "$2"
            use_slot "$3"; msg action focus-window --id "$4"
            sleep 0.5
        done
    ) &
    TOGGLES_PID=$!
}
toggles_stop() {   # $1 on slot, $2 on right id, $3 off slot, $4 off right id
    kill "$TOGGLES_PID" 2>/dev/null || true; wait "$TOGGLES_PID" 2>/dev/null || true; TOGGLES_PID=
    use_slot "$1"; msg action focus-window --id "$2"
    use_slot "$3"; msg action focus-window --id "$4"
}
case_ring_motion() {
    require_off
    local right_on right_off label on off
    for label in pinned moving; do
        if [ "$label" = pinned ]; then
            on=$RESP_ON; off=$RESP_OFF
        else
            # The fastest beam the config allows short of its cap, re-triggered
            # by focus gains through each burst (`toggles_start`): the beam
            # runs on the unadjusted clock, so `slowdown` does not stretch it.
            on='ring-beam-speed 3000;'; off='focus "none"; accent "none"; ring-beam-speed 3000;'
        fi
        write_capture_config "$WORK/ring-motion-$label-on.kdl" "$on" 'slowdown 6.0;'
        write_capture_config "$WORK/ring-motion-$label-off.kdl" "$off" 'slowdown 6.0;'
        start_nested 1 "$WORK/ring-motion-$label-on.kdl"; right_on=$(open_scene); sleep 1
        start_nested 2 "$WORK/ring-motion-$label-off.kdl"; right_off=$(open_scene); sleep 1
        use_slot 1; msg action move-column-left
        use_slot 2; msg action move-column-left
        [ "$label" = moving ] && toggles_start 1 "$right_on" 2 "$right_off"
        motion_pair_burst "ring-motion-$label-move" 1 "$right_on" 2 "$right_off"
        [ "$label" = moving ] && toggles_stop 1 "$right_on" 2 "$right_off"
        use_slot 1; msg action set-column-width +200
        use_slot 2; msg action set-column-width +200
        [ "$label" = moving ] && toggles_start 1 "$right_on" 2 "$right_off"
        motion_pair_burst "ring-motion-$label-resize" 1 "$right_on" 2 "$right_off"
        [ "$label" = moving ] && toggles_stop 1 "$right_on" 2 "$right_off"
        stop_nested 1; stop_nested 2
    done
}
# `accent` and `focus` select independently: the accent tints the band only
# where it is selected, the focus draws it only where it is selected.
case_selectors() {
    require_off
    local name resp right e
    while read -r name resp; do
        write_capture_config "$WORK/sel-$name.kdl" "$resp ring-beam-speed 0;" 'slowdown 6.0;'
        start_nested 1 "$WORK/sel-$name.kdl"
        right=$(open_scene); settle
        msg set-window-signal --id "$right" --source demo --accent '#ff0000'
        sleep 2; settle
        shot "$(px "sel-$name")"
        stop_nested 1
        reach "selectors-$name" "$(px "sel-$name")" "$(px off)"
        e=$(emissive "$(px "sel-$name")" "$(px off)" "$FIL_X" "$FIL_Y")
        record "selectors $name emissive at ($FIL_X,$FIL_Y) = $e"
        case $name in
            ring-light|ring-none)
                check "selectors $name red share" "$(red_fraction "$e")" 'v >= 0.90' ">= 0.90" ;;
            none-light)
                check "selectors $name red share" "$(red_fraction "$e")" 'v >= 0.24 && v <= 0.31' "0.24 to 0.31" ;;
            none-none)
                check "selectors $name emissive luminance" "$(emissive_lum "$e")" 'v <= 0.005' "<= 0.005" ;;
        esac
    done <<'SEL'
ring-light accent "ring"; focus "ring-light";
ring-none accent "ring"; focus "none";
none-light accent "none"; focus "ring-light";
none-none accent "none"; focus "none";
SEL
}
# Zero chamfer: not verified by render. The shader's gate
#   if ((showAccent || showFocus) && slabChamfer > 0.0)
# is only reached with slabChamfer == 0 when the slab is at most 2 px on one
# axis, because `max_chamfer = max(min(half_ext) - 1, 0)` is otherwise
# positive. No client on
# this host produces such a window: kitty and foot have a one-cell minimum
# and weston-simple-egl is fixed at 250 px. `niri-visual-tests` renders with
# `xray: None`, so materials do not draw there either.
case_tiny() {
    record "tiny (zero chamfer): NOT VERIFIED BY RENDER; no client on this host is small enough to drive slabChamfer == 0"
}

sha256sum "$NIRI" > "$WORK/SHA256SUMS"
"$NIRI" --version >> "$WORK/SHA256SUMS"
git -C "$REPO" rev-parse HEAD >> "$WORK/SHA256SUMS"
: > "$LOG"
for c in ${CASES:-baseline rest-confinement accent-midfade resize-flex ring-motion selectors tiny}; do
    record "== $c"
    case $c in
        baseline)         case_baseline ;;
        rest-confinement) case_rest_confinement ;;
        accent-midfade)   case_accent_midfade ;;
        resize-flex)      case_resize_flex ;;
        ring-motion)      case_ring_motion ;;
        selectors)        case_selectors ;;
        tiny)             case_tiny ;;
        *) echo "FAIL: unknown case $c" >&2; exit 1 ;;
    esac
done
echo "captures: $WORK"
