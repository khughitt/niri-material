#!/usr/bin/env bash
# glass-optic-smoke-lib.sh: the helpers the per-optic smokes share
# (glass-iridescence-smoke.sh, glass-aurora-smoke.sh). Sourced, never run.
#
# A smoke nests niri under a headless Weston host, opens one transparent
# kitty (app-id gos-probe) over a pinned glass material on a flat warm
# backdrop, and compares decoded pixels of screen captures. Redraw counts
# and GPU cost come from Tracy: the `profile-with-tracy` build is captured
# for 30 s, `Niri::redraw` zones are counted in the final 20 s, and the
# `MaterialRenderElement::draw` GPU zone's median between 20 s and 28 s is
# the frame cost, following material-signals-smoke.sh.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT (holds the retained Tracy
# 0.13.1 tools under material-roughness-b220152d/tools).
# Every run records provenance and machine quietness through tools/capture-meta
# (docs/specs/2026-09-11-material-capture-protocol-design.md). The entry script
# calls `capture_preflight headless` right after sourcing this lib and
# `capture_identity` after build_binaries; start_nested settles before every
# launch. This lib never preflights on its own: idle-budget sources it in a
# mode that must stay offline.
# Requires: weston, kitty, swaybg, jq, rg, flock, ss, ImageMagick 7 with
# Oklab, cargo.
set -eu
OUT=${OUT:?artifact directory}
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?evidence root with the retained Tracy tools}
# A run owns its directory: traces are exported unconditionally and the
# GPU medians files accumulate one line per round, so a rerun into a used
# directory would mix measurements. Refuse anything but a fresh one.
if [ -e "$OUT" ] && [ -n "$(ls -A "$OUT")" ]; then
    echo "FAIL: OUT must be a fresh directory, $OUT is not empty" >&2; exit 1
fi
mkdir -p "$OUT"
ROOT=$(git rev-parse --show-toplevel)
# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/gos.XXXXXX")
RUN=$(basename "$RT")
HOST=$RUN-host; WESTON_PID=; NIRI_PID=; CAP_PID=
fail() { echo "FAIL: $*" >&2; exit 1; }
remove_runtime_dir() {
    python3 - "$RT" <<'PY'
import shutil
import sys

shutil.rmtree(sys.argv[1])
PY
}
stop_weston() {
    if [ -n "$WESTON_PID" ]; then
        kill "$WESTON_PID" 2>/dev/null || true
        wait "$WESTON_PID" 2>/dev/null || true
        WESTON_PID=
    fi
    for _ in $(seq 50); do [ ! -e "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ ! -e "$XDG_RUNTIME_DIR/$HOST" ] || fail "Weston socket $HOST still present"
}
cleanup() {
    local rc=$?
    if [ -n "$CAP_PID" ]; then kill "$CAP_PID" 2>/dev/null || true; wait "$CAP_PID" 2>/dev/null || true; fi
    if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; fi
    stop_weston || rc=1
    remove_runtime_dir || rc=1
    capture_meta release "$OUT" || true
    exit "$rc"
}
trap cleanup EXIT

# --- capture record ---------------------------------------------------------
# CAPTURE_META lets a test substitute a recording stub; unset, it is the tool.
capture_meta() { ${CAPTURE_META:-python3 "$ROOT/tools/capture-meta"} "$@"; }
# Entry scripts call this first thing after sourcing. The fixture name is the
# calling script; the task id comes from CAPTURE_TASK, which every smoke sets.
capture_preflight() {
    capture_meta preflight "$OUT" --lane "$1" --task "${CAPTURE_TASK:?task id authorizing this run}" \
        --fixture "$(basename "$0")" --owner-pid $$ --tool weston --tool kitty --tool "tracy=0.13.1" \
        || fail "preflight refused; see $OUT/capture.json"
}
# After build_binaries: both binaries, this lib, and the calling script are the
# static inputs. Extra --input/--config arguments pass through.
capture_identity() {   # NIRI_TRACY is unset for fixtures that measure the installed binary only
    capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" ${NIRI_TRACY:+--binary "$NIRI_TRACY"} \
        --input "$ROOT/docs/materials/scripts/glass-optic-smoke-lib.sh" --input "$0" "$@" \
        || fail "identity refused"
}
# The previous launch's GPU work can hold P5 for a moment after its niri exits;
# settling straight away lands that tail in the window and refuses the run (the
# idle GPU holds P8: material-3db428, material-124f1f). Wait for 3 s of
# consecutive P8 samples, at most 120 polls (60 s), before the settle. The
# settle gate itself is unchanged. Waits are logged to $OUT/cooldown.txt.
gpu_cooldown() {   # $1 sub-run name
    local polls=0 run=0
    while [ "$run" -lt 6 ]; do
        [ "$polls" -lt 120 ] || fail "GPU not back at P8 within 120 polls before $1"
        if [ "$(nvidia-smi --query-gpu=pstate --format=csv,noheader)" = P8 ]; then run=$((run + 1)); else run=0; fi
        polls=$((polls + 1))
        sleep 0.5
    done
    echo "$1 $polls" >> "$OUT/cooldown.txt"
}
# Before every nested launch: the sub-run is named after its config unless the
# caller's observation is not its config (idle-budget reuses six configs).
settle_before_launch() {
    local cfg=$1 name=${2:-}
    [ -n "$name" ] || name=$(basename "$cfg" .kdl)
    gpu_cooldown "$name"
    capture_meta settle "$OUT" --sub-run "$name" --input "$cfg" || fail "settle refused before $name; see $OUT/capture.json"
}

# --- binaries ---------------------------------------------------------------
# Two builds: the release build for captures and the Tracy build for counts
# and cost, snapshotted under $OUT so a concurrent build cannot replace the
# executable mid-run. The target dir is shared across worktrees; toggling the
# feature rebuilds the niri crate.
build_binaries() {
    local target
    target=$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
    (cd "$ROOT" && cargo build --release)
    cp "$target/release/niri" "$OUT/niri"
    (cd "$ROOT" && cargo build --release --features profile-with-tracy)
    cp "$target/release/niri" "$OUT/niri-tracy"
    NIRI=$OUT/niri; NIRI_TRACY=$OUT/niri-tracy
}

# --- configs ----------------------------------------------------------------
WALL=$OUT/warm-mid.png
magick -size 1280x720 xc:'rgb(140,115,90)' "$WALL"
GEOM_COLOR='#ff00ff'
GLASS_EXTRA=; RESPONSE_EXTRA=; FOCUS_RESPONSE=none; TOP_EXTRA=
IDLE='printf "\033[?25l"; exec sleep 600'
TICK='printf "\033[?25l"; while :; do date +%s%N; sleep 0.1; done'
# The material is pinned rather than taken from a generated prism.kdl, which
# drifts. `animations { off }` is deliberately absent: it pins every optic
# clock through OpticFrame, which the aurora smoke measures. The response
# lights no filament and no accent so the probe is signal-free; two windows
# at proportion 0.4 both fit in view, so a second window can take focus
# without scrolling the probe.
#
# GLASS_EXTRA is newline-separated glass lines. A line whose node name
# matches a baseline line replaces it, as the sweep's emit_block does: KDL
# rejects a duplicate single node (`ior 1.5` then `ior 1.7` fails
# validation), so a preset body cannot simply be appended.
GLASS_BASELINE=(
    "ior 1.5"
    "thickness 20"
    "attenuation-color \"#dfe8ff\""
    "attenuation-distance 60"
    "chromatic-aberration 0"
    "distortion 0 scale=0.5"
    "anisotropic-blur 0"
    "roughness 0"
    "backdrop-blur false"
    "jelly-flex 0"
    "jelly-ripple 0"
    "bevel 12"
    "offset-x 6"
    "offset-y 6"
)
emit_glass() {
    local line key extra_keys=" "
    while IFS= read -r line; do
        line=${line#"${line%%[! ]*}"}
        [ -n "$line" ] && extra_keys+="${line%% *} "
    done <<< "$GLASS_EXTRA"
    for line in "${GLASS_BASELINE[@]}"; do
        key=${line%% *}
        case $extra_keys in *" $key "*) continue ;; esac
        printf '        %s\n' "$line"
    done
    while IFS= read -r line; do
        line=${line#"${line%%[! ]*}"}
        [ -n "$line" ] && printf '        %s\n' "$line"
    done <<< "$GLASS_EXTRA"
    return 0
}
write_config() {
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
$TOP_EXTRA
material "gos-probe" {
    glass {
$glass
    }
    response "default" {
        focus "$FOCUS_RESPONSE"
        accent "none"
        ring-beam-speed 0
        $RESPONSE_EXTRA
    }
}
window-rule {
    match app-id="^gos-probe$"
    material "gos-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
KDL
}
write_geometry_config() {
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
window-rule {
    match app-id="^gos-probe$"
    geometry-corner-radius 0
}
KDL
}

# --- nested host ------------------------------------------------------------
start_nested() {   # $1 niri, $2 config, $3 sub-run name (defaults to the config's basename)
    settle_before_launch "$2" "${3-}"
    weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
        --width=1280 --height=720 --socket="$HOST" >> "$OUT/weston.log" 2>&1 &
    WESTON_PID=$!
    for _ in $(seq 100); do
        [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break
        kill -0 "$WESTON_PID" 2>/dev/null || break
        sleep 0.1
    done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || fail "no Weston socket (see $OUT/weston.log)"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    "$1" validate -c "$2" || fail "config $2 does not validate with $1"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$1" -c "$2" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1) || fail "no niri socket"
    export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    stop_weston
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
msg() { "$1" msg "${@:2}"; }
windows_with() { msg "$1" -j windows | jq -r --arg id "$2" '[.[] | select(.app_id==$id)] | length'; }
spawn_probe() {
    msg "$1" action spawn -- kitty --config NONE --class gos-probe -o background_opacity=0 \
        -o cursor_blink_interval=0 sh -c "$2"
    for _ in $(seq 100); do [ "$(windows_with "$1" gos-probe)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$1" gos-probe)" -eq 1 ] || fail "expected exactly one probe window"
}
spawn_geometry_probe() {
    msg "$1" action spawn -- kitty --config NONE --class gos-probe \
        -o background="$GEOM_COLOR" -o background_opacity=1 -o cursor_blink_interval=0 sh -c "$IDLE"
    for _ in $(seq 100); do [ "$(windows_with "$1" gos-probe)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$1" gos-probe)" -eq 1 ] || fail "expected exactly one geometry probe window"
}
steal_focus() {
    msg "$1" action spawn -- kitty --config NONE --class gos-other -o cursor_blink_interval=0 \
        sh -c "$IDLE"
    for _ in $(seq 100); do [ "$(windows_with "$1" gos-other)" -ge 1 ] && break; sleep 0.1; done
    sleep 0.5
    [ "$(msg "$1" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .is_focused')" = false ] \
        || fail "the probe is still focused"
}
measure_rect() {
    local bbox
    bbox=$(magick "$1" -fuzz 12% +transparent "$GEOM_COLOR" -format '%@' info:) \
        || fail "bounding-box measurement failed on $1"
    [[ $bbox =~ ^([0-9]+)x([0-9]+)\+([0-9]+)\+([0-9]+)$ ]] \
        || fail "could not locate the geometry probe in $1 (got '$bbox')"
    printf '%s %s %s %s\n' "${BASH_REMATCH[3]}" "${BASH_REMATCH[4]}" \
        "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
}
# Measure each layout in screenshot pixel space. The headless output is
# transformed and IPC reports tile_pos_in_workspace_view as null, so logical
# positions cannot safely drive screenshot crops. The caller calibrates the
# one- or two-window layout before capturing material sessions.
calibrate_probe_rect() {
    local other=$2 name=geometry-$2
    [ "$other" = 0 ] || [ "$other" = 1 ] || fail "geometry focus thief must be 0 or 1"
    write_geometry_config "$OUT/$name.kdl"
    start_nested "$1" "$OUT/$name.kdl"
    spawn_geometry_probe "$1"
    [ "$other" = 1 ] && steal_focus "$1"
    sleep 2
    shot "$1" "$name"
    measure_rect "$OUT/$name.png" > "$OUT/probe-rect-$other.txt"
    stop_nested
}
probe_rect() {
    local other j iw ih
    other=$(windows_with "$1" gos-other)
    [ "$other" = 0 ] || [ "$other" = 1 ] || fail "expected zero or one focus thief, got $other"
    [ -s "$OUT/probe-rect-$other.txt" ] || fail "probe geometry for $other focus thieves was not calibrated"
    read -r PX PY PW PH < "$OUT/probe-rect-$other.txt"
    j=$(msg "$1" -j windows | jq -c '.[] | select(.app_id=="gos-probe") | .layout.window_size')
    iw=$(jq -r '.[0] | floor' <<< "$j"); ih=$(jq -r '.[1] | floor' <<< "$j")
    [ "$PW" -eq "$iw" ] && [ "$PH" -eq "$ih" ] \
        || fail "measured probe ${PW}x${PH}, current IPC size is ${iw}x${ih}"
    [ "$PW" -gt 200 ] && [ "$PH" -gt 400 ] || fail "probe rect too small: ${PW}x${PH}+$PX+$PY"
}
shot() {
    local png=$OUT/$2.png
    msg "$1" action screenshot-screen --write-to-disk true --show-pointer false --path "$png"
    for _ in $(seq 50); do [ -s "$png" ] && break; sleep 0.1; done
    [ -s "$png" ] || fail "capture $2 not written"
}
roi() {
    magick "$OUT/$1.png" -crop "$2" +repage -define png:color-type=2 "$OUT/$1-$3.png" \
        || fail "crop $1 $3 failed"
}
face_roi() { echo "200x400+$((PX + 60))+$((PY + 120))"; }
# This headless capture presents the visible chamfer at the measured window's
# left edge; the output transform makes the brief's right-band crop nearly flat.
chamfer_roi() { echo "20x400+$((PX - 4))+$((PY + 120))"; }

# --- comparison -------------------------------------------------------------
is_number() { [[ $1 =~ ^-?[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }
compare_metric() {
    local out status
    set +e
    out=$(magick compare -metric "$1" "$2" "$3" null: 2>&1 >/dev/null)
    status=$?
    set -e
    case $status in 0|1) ;; *) fail "magick compare $1 $2 $3 exited $status: $out" ;; esac
    out=${out%% *}
    is_number "$out" || fail "magick compare $1 $2 $3 returned a non-numeric metric: $out"
    METRIC=$out
}
ae() { compare_metric AE "$1" "$2"; }
rmse() { compare_metric RMSE "$1" "$2"; }
sd() {
    local out
    out=$(magick "$1" -colorspace Gray -format '%[fx:standard_deviation]' info:) || fail "magick info on $1 failed"
    is_number "$out" || fail "standard deviation of $1 is not numeric: $out"
    METRIC=$out
}
mean() {
    local out
    out=$(magick "$1" -colorspace Gray -format '%[fx:mean]' info:) || fail "magick mean on $1 failed"
    is_number "$out" || fail "mean of $1 is not numeric: $out"
    METRIC=$out
}
oklab_ab() {
    magick "$1" -colorspace Oklab -channel R -evaluate set 50% +channel -set colorspace sRGB -depth 16 "$2" \
        || fail "oklab ab $2 failed"
}
one_code() { magick xc: -format '%[fx:quantumrange/255]' info: || fail "magick quantum range failed"; }
assert_zero() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v == 0) }' || fail "$1 expected 0, got $2"
}
assert_positive() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v > 0) }' || fail "$1 expected > 0, got $2"
}
assert_greater() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a > b) }' || fail "$1 expected $2 > $3"
}
assert_less() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a < b) }' || fail "$1 expected $2 < $3"
}
assert_about() {
    is_number "$2" && is_number "$3" && is_number "$4" || fail "$1 is not numeric: $2 vs $3 (tol $4)"
    awk -v g="$2" -v w="$3" -v t="$4" 'BEGIN { exit !(g >= w*(1-t) && g <= w*(1+t)) }' \
        || fail "$1 expected $2 within $(awk -v t="$4" 'BEGIN { printf "%d%%", t*100 }') of $3"
}

# --- Tracy ------------------------------------------------------------------
reserve_tracy_port() {
    local port listeners
    for port in $(seq 20000 39999); do
        exec {TRACY_LOCK_FD}>"$XDG_RUNTIME_DIR/niri-material-tracy-$port.lock"
        if flock -n "$TRACY_LOCK_FD" \
            && listeners=$(ss -H -ltn "sport = :$port") && [ -z "$listeners" ]; then
            TRACY_PORT=$port; export TRACY_PORT
            return
        fi
        exec {TRACY_LOCK_FD}>&-
    done
    fail "no free Tracy port in 20000-39999"
}
tools_ready() {
    local retained=$EVIDENCE/material-roughness-b220152d/tools
    TOOLS=$retained
    sha256sum -c --quiet - <<EOF 2>/dev/null || fail "retained Tracy 0.13.1 tools missing or altered under $retained; rebuild them as material-signals-smoke.sh tools_ready does"
957db02d917aaf217021ff04c59fbe196415ddb5e8b9615d9e0f91ff25d75ca6  $retained/tracy-capture
588642902b24282d831cf4f4088ba7c3f28e59f0c506da3fa8f02288ec3efc30  $retained/tracy-csvexport
EOF
}
capture_bg() {
    : > "$OUT/$1.capture.log"
    timeout 90 "$TOOLS/tracy-capture" -o "$OUT/$1.tracy" -a 127.0.0.1 -p "$TRACY_PORT" -s 30 \
        > "$OUT/$1.capture.log" 2>&1 & CAP_PID=$!
}
capture_ready() {
    local _; for _ in $(seq 300); do
        [ -n "$(ss -H -tn state established "( sport = :$TRACY_PORT or dport = :$TRACY_PORT )")" ] && return
        kill -0 "$CAP_PID" 2>/dev/null || break
        sleep 0.1
    done
    fail "tracy-capture never reported a connection (see $OUT/$1.capture.log)"
}
capture_wait() {
    local rc=0; wait "$CAP_PID" || rc=$?; CAP_PID=
    [ "$rc" -eq 0 ] || fail "tracy-capture exited $rc (no client connected, or timed out)"
}
col() {
    local idx; idx=$(head -1 "$1" | tr ',' '\n' | grep -nx "$2" | cut -d: -f1)
    [ -n "$idx" ] || fail "column '$2' not in $(head -1 "$1")"
    echo "$idx"
}
# Some traces send tracy-csvexport into an endless loop (view-tilt-smoke
# gpu-deep-3, 2026-09-24: reproducible; siblings export in under a second),
# so every export is bounded and a hang fails the run instead of stalling it.
csvexport() {   # flag, tracy file, csv out
    timeout 120 "$TOOLS/tracy-csvexport" "$1" "$2" > "$3" \
        || fail "tracy-csvexport $1 ${2##*/} failed or ran past 120 s"
}
export_cpu() { csvexport --unwrap "$OUT/$1.tracy" "$OUT/$1.csv"; }
trace_end() {
    local c; c=$(col "$OUT/$1.csv" ns_since_start)
    awk -F, -v c="$c" 'NR>1 { t=$c+0; if (t>end) end=t } END { printf "%d", end }' "$OUT/$1.csv"
}
count_last20() {
    export_cpu "$1"
    local end c; end=$(trace_end "$1"); c=$(col "$OUT/$1.csv" ns_since_start)
    # Idle-inhibit refresh is a one-second heartbeat even without damage.
    # An empty/truncated/stalled trace must not masquerade as zero redraws.
    awk -F, -v c="$c" -v end="$end" '
        NR>1 && $1=="Niri::refresh_idle_inhibit" {
            t=$c+0
            if (t>=end-20e9) {
                if (last && t-last>1.5e9) bad=1
                if (!first) first=t
                last=t; beats++
            }
        }
        END { exit !(end>=30e9 && beats>=19 && last-first>=18e9 && !bad) }
    ' "$OUT/$1.csv" || fail "$1: incomplete or stalled final 20 s trace"
    awk -F, -v c="$c" -v end="$end" \
        'NR>1 && $1=="Niri::redraw" { t=$c+0; if (t>=end-20e9) n++ } END { printf "%d", n+0 }' "$OUT/$1.csv"
}
gpu_median_ns() {
    local csv=${1%.tracy}.gpu.csv
    csvexport --gpu "$1" "$csv"
    local ct ce; ct=$(col "$csv" "Time from start of program"); ce=$(col "$csv" "GPU execution time")
    awk -F, -v ct="$ct" -v ce="$ce" 'NR>1 && $1=="MaterialRenderElement::draw" && $ct+0>=20e9 && $ct+0<28e9 { print $ce+0 }' "$csv" \
        | head -14 | sort -n | awk '{ a[NR]=$1 } END { if (NR!=14) { print "FAIL: " NR " MaterialRenderElement::draw samples in 20-28 s, need 14" > "/dev/stderr"; exit 1 }
              printf "%d\n", (a[7]+a[8])/2 }'
}
median3() {
    awk 'NF != 1 || $1 !~ /^[0-9]+$/ || $1 <= 0 { bad=1 } END { exit (bad || NR != 3) }' "$1" \
        || fail "$1 must contain exactly three positive integer medians"
    sort -n "$1" | sed -n 2p
}
trace_run() {
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI_TRACY" "$OUT/$1.kdl"
    spawn_probe "$NIRI_TRACY" "$2"
    [ "$3" = 1 ] && steal_focus "$NIRI_TRACY"
    sleep 2
    capture_bg "$1"; capture_ready "$1"; capture_wait
    stop_nested
}
ns_to_ms() { awk -v n="$1" 'BEGIN { printf "%.3f", n/1e6 }'; }
pct_delta() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%+.1f", (b-a)/a*100 }'; }

finish() {
    (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > SHA256SUMS)
    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
        fail "material error, fallback or panic in niri.log"
    fi
    echo "PASS: artifacts in $OUT"
}
