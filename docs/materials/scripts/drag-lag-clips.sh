#!/usr/bin/env bash
# drag-lag-clips.sh: review clips for the drag follow-lag stimulus
# (docs/specs/2026-10-01-drag-follow-lag-design.md §7). Each sequence runs a nested
# headless niri with one kitty and one vdrag client (docs/materials/scripts/vdrag.py).
# vdrag drags its own window through xdg_toplevel.move, driven by a virtual pointer,
# while a burst of screenshots records it, under the capture protocol
# (tools/capture-meta: preflight, identity, settle before every launch, release).
# The clips are for the owner's judgment against the native column move; this script
# records, it does not grade. It does check that the timed segment ran as an
# interactive move (IPC reports no workspace for a window being moved), not a view pan.
#
# Sequences (SEQUENCES env, space-separated, default all):
#   scroll-fast  tiled: drag vdrag left 20 px per 8 ms event (2500 px/s), 24 events,
#                hold 600 ms, release
#   scroll-slow  tiled: 5 px per event (625 px/s), 60 events
#   float-fast   floating vdrag over kitty: right at 20 px per event, 24 events
#   float-slow   floating: 5 px per event, 60 events
#   native       tiled, no drag: move-column-left on vdrag (the control)
#
# Env: NIRI_MATERIAL_WORK_ROOT (artifacts land under it), CAPTURE_TASK (the task id
# authorizing the run), NIRI (default: this checkout's release build, built here).
# Requires: weston, kitty, swaybg, jq, ImageMagick, python3.
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root}
TASK=${CAPTURE_TASK:?task id authorizing this run}
RUN=drag-lag-$$-$(date +%s)
OUT=$EVIDENCE/drag-lag-clips-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # short: nested niri panics on long socket paths
SEQUENCES=${SEQUENCES:-"scroll-fast scroll-slow float-fast float-slow native"}
RUN_S=4            # lift 0.7 s, slow segment 0.5 s, hold 0.6 s, release settling under 1 s
GO_DELAY=0.2       # burst start to the go file
LIFT_S=0.71        # 26 lift events at 125 Hz, then --lift-ms 500
VDRAG=$ROOT/docs/materials/scripts/vdrag.py
UNIT=; HOST_SOCKET=; NIRI_PID=; NIRI_SOCKET=; VDRAG_PID=; BURST_PID=; HOST_SEQ=0
mkdir -p "$OUT" "$RT"

capture_meta() { python3 "$ROOT/tools/capture-meta" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
# After a sequence's host is down; a sequence cut short is never finished.
finish_sub_run() { capture_meta finish "$OUT" --sub-run "$1" || fail "finish refused for $1; see $OUT/capture.json"; }
pid_running() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }
msg() { "$NIRI" msg "$@"; }
app_ids() { msg -j windows | jq -r '.[].app_id'; }
wait_app() { for _ in $(seq 100); do app_ids | grep -qx "$1" && return; sleep 0.1; done; fail "no $1 window"; }
app_window() { msg -j windows | jq -r --arg a "$1" '.[] | select(.app_id == $a) | .id'; }

stop_nested() {
    # The screenshot worker first: it talks to the niri being stopped.
    if [ -n "$BURST_PID" ]; then
        kill "$BURST_PID" 2>/dev/null || true; wait "$BURST_PID" 2>/dev/null || true; BURST_PID=
    fi
    if [ -n "$VDRAG_PID" ]; then
        kill "$VDRAG_PID" 2>/dev/null || true; wait "$VDRAG_PID" 2>/dev/null || true; VDRAG_PID=
    fi
    if [ -n "$NIRI_PID" ]; then
        if [ -S "$NIRI_SOCKET" ]; then msg action quit --skip-confirmation >/dev/null 2>&1 || true; fi
        for _ in $(seq 50); do pid_running "$NIRI_PID" || break; sleep 0.1; done
        if pid_running "$NIRI_PID"; then kill "$NIRI_PID" 2>/dev/null || true; fi
        wait "$NIRI_PID" 2>/dev/null || true
        NIRI_PID=; NIRI_SOCKET=
    fi
    if [ -n "$UNIT" ]; then
        timeout 10 systemctl --user stop "$UNIT" >/dev/null 2>&1 || true
        for _ in $(seq 50); do [ -e "$HOST_SOCKET" ] || break; sleep 0.1; done
        UNIT=; HOST_SOCKET=
        sleep 4   # weston's GL renderer steps the GPU down over about 2 s
    fi
    rm -rf "${RT:?}"/*
}
cleanup() {
    local rc=$?
    trap - EXIT INT TERM
    stop_nested
    rm -rf "$RT"
    capture_meta release "$OUT" || rc=1
    exit "$rc"
}
trap cleanup EXIT INT TERM

# --- capture protocol -------------------------------------------------------
capture_meta preflight "$OUT" --lane headless --task "$TASK" --fixture "$(basename "$0")" \
    --owner-pid $$ --tool weston --tool kitty || fail "preflight refused; see $OUT/capture.json"

TARGET=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
if [ -z "${NIRI:-}" ]; then
    cargo build --release
    NIRI=$TARGET/release/niri
fi
[ -x "$NIRI" ] || fail "niri binary not found at $NIRI"
cp "$NIRI" "$OUT/niri"; NIRI=$OUT/niri
sha256sum "$NIRI" "$VDRAG" | tee -a "$OUT/SHA256SUMS"

# --- configs -----------------------------------------------------------------
CHECKER=$OUT/checker.png
magick -size 160x90 pattern:checkerboard -scale 800% "$CHECKER"
# The glass is pinned (bevel 12, thickness 20, the live jelly-flex 0.0066, ripple
# off so flex is the only motion cue). The idle gate is off: the nested instance
# sees only the virtual pointer.
write_config() {   # $1 = path; FLOAT=1 opens vdrag floating
    cat > "$1" <<KDL
material "tg" { glass { bevel 12; thickness 20; jelly-flex 0.0066; jelly-ripple 0; }; }
window-rule { match app-id="^(kitty|vdrag)$"; material "tg"; }
window-rule { match app-id="^vdrag$"; open-floating ${FLOAT:-false}; }
layout { focus-ring { off; }; gaps 24; }
hotkey-overlay { skip-at-startup; }
signal { idle-after-ms 0; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" "-o" "cursor_blink_interval=0" "-o" "background_opacity=0.6" "--hold" "true"
KDL
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; fail "$1 does not validate"; }
}
write_config "$OUT/tiled.kdl"
FLOAT=true write_config "$OUT/floating.kdl"

capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" --input "$0" --input "$VDRAG" \
    --input "$OUT/tiled.kdl" --input "$OUT/floating.kdl" || fail "identity refused"

# --- nested instance ---------------------------------------------------------
NESTED_WAYLAND=
start_nested() {   # $1 = config, $2 = sub-run name; sets NIRI_SOCKET and NESTED_WAYLAND
    capture_meta settle "$OUT" --sub-run "$2" --input "$1" || fail "settle refused before $2; see $OUT/capture.json"
    local niri_from weston_from
    niri_from=$( [ -e "$OUT/niri.log" ] && wc -c < "$OUT/niri.log" || echo 0 )
    weston_from=$( [ -e "$OUT/weston.log" ] && wc -c < "$OUT/weston.log" || echo 0 )
    HOST_SEQ=$((HOST_SEQ + 1))
    local host=$RUN-h$HOST_SEQ
    UNIT=$host-weston; HOST_SOCKET=$XDG_RUNTIME_DIR/$host
    systemd-run --user --unit="$UNIT" --collect \
        --property=StandardOutput=append:"$OUT/weston.log" --property=StandardError=append:"$OUT/weston.log" \
        weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$host" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$HOST_SOCKET" ] && break; sleep 0.1; done
    [ -S "$HOST_SOCKET" ] || fail "Weston socket never appeared at $HOST_SOCKET"
    ln -s "$HOST_SOCKET" "$RT/$host"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$host RUST_LOG=niri=debug,smithay::backend::renderer::gles=info \
        "$NIRI" -c "$1" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=
    for f in "$RT"/niri.*.sock; do [ -S "$f" ] && NIRI_SOCKET=$f; done
    [ -S "$NIRI_SOCKET" ] || fail "nested IPC socket not found under $RT"
    export NIRI_SOCKET
    # The nested instance's own Wayland socket, by absolute path: the driver must not
    # resolve it against the outer XDG_RUNTIME_DIR.
    NESTED_WAYLAND=
    for f in "$RT"/wayland-*; do [ -S "$f" ] && NESTED_WAYLAND=$f; done
    [ -S "$NESTED_WAYLAND" ] || fail "nested Wayland socket not found under $RT"
    wait_app kitty
    # This launch's renderer lines only: both logs are appended across launches.
    tail -c "+$((niri_from + 1))" "$OUT/niri.log" > "$OUT/$2.niri.renderer.log"
    tail -c "+$((weston_from + 1))" "$OUT/weston.log" > "$OUT/$2.weston.renderer.log"
    capture_meta renderer "$OUT" --sub-run "$2" --niri-log "$OUT/$2.niri.renderer.log" \
        --weston-log "$OUT/$2.weston.renderer.log" || fail "$2: renderer check refused; see $OUT/capture.json"
}
shot_request() { rm -f "$1"; msg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"; }
shot_wait() { for _ in $(seq 200); do [ -s "$1" ] && magick identify "$1" >/dev/null 2>&1 && return; sleep 0.05; done; fail "shot $1"; }
shot() { shot_request "$1"; shot_wait "$1"; }
settle() {   # two shots 0.6 s apart agree
    local a=$OUT/settle-a.png b=$OUT/settle-b.png n
    for _ in $(seq 20); do
        shot "$a"; sleep 0.6; shot "$b"
        n=$(magick compare -metric AE "$a" "$b" null: 2>&1 | awk '{print $1}' || true)
        [ "${n%.*}" = 0 ] && return
    done
    fail "scene never settled"
}

# --- bursts ------------------------------------------------------------------
BURST_T0=
burst_start() {   # $1 = label, $2 = seconds
    local d=$OUT/$1; mkdir -p "$d"
    BURST_T0=$(date +%s.%N)
    (
        local now i=0 f
        while :; do
            now=$(date +%s.%N)
            awk -v a="$now" -v b="$BURST_T0" -v s="$2" 'BEGIN { exit !(a - b < s) }' || break
            i=$((i + 1)); f=$d/f$(printf %03d "$i").png
            printf '%s %s\n' "$(basename "$f")" "$(awk -v a="$now" -v b="$BURST_T0" 'BEGIN { printf "%.3f", a - b }')" >> "$d/frames.txt"
            shot "$f"
        done
        echo "$1: $i frames in $2 s" >> "$OUT/timing.txt"
    ) &
    BURST_PID=$!
}
burst_wait() {
    wait "$BURST_PID"; BURST_PID=
    local d=$OUT/$1 frames
    frames=("$d"/f*.png)
    [ -f "${frames[0]}" ] || fail "$1: no frames"
    magick -delay 8 -loop 0 "$d"/f*.png -scale 50% "$OUT/$1.gif"
    magick montage "$d"/f*.png -tile 8x -geometry 320x180+2+2 -background '#111' "$OUT/$1-sheet.png"
    echo "$1: $d (${#frames[@]} frames); gif $OUT/$1.gif (approximate playback: fixed 80 ms/frame; use frames.txt for request intervals, not GIF duration)" | tee -a "$OUT/clips.txt"
}

# --- sequences ---------------------------------------------------------------
drag_sequence() {   # $1 = label, $2 = config, $3 = dx per event, $4 = events
    local ready=$RT/$1.ready go=$RT/$1.go done=$RT/$1.done drag_s moving
    start_nested "$2" "$1"
    python3 "$VDRAG" --socket "$NESTED_WAYLAND" --extent 1280,720 --dx "$3" --frames "$4" \
        --hz 125 --hold-ms 600 --ready "$ready" --go "$go" --done "$done" >> "$OUT/vdrag.log" 2>&1 &
    VDRAG_PID=$!
    for _ in $(seq 150); do [ -e "$ready" ] && break; pid_running "$VDRAG_PID" || fail "$1: vdrag exited; see $OUT/vdrag.log"; sleep 0.1; done
    [ -e "$ready" ] || fail "$1: vdrag never found its window"
    settle
    shot "$OUT/$1-rest.png"
    burst_start "$1" "$RUN_S"
    sleep "$GO_DELAY"
    touch "$go"
    # During the timed segment the window must be under interactive move: niri's IPC
    # reports `workspace_id: null` only for the moving window (Layout::with_windows).
    # A view pan keeps it on its workspace, so this rejects the pan a pixel check
    # would accept.
    drag_s=$(awk -v n="$4" 'BEGIN { print n / 125 }')
    sleep "$LIFT_S"
    moving=0
    for _ in $(seq 20); do
        if [ "$(msg -j windows | jq -r '.[] | select(.app_id == "vdrag") | .workspace_id')" = null ]; then
            moving=1; break
        fi
        sleep "$(awk -v d="$drag_s" 'BEGIN { print d / 20 }')"
    done
    burst_wait "$1"
    [ -e "$done" ] || fail "$1: vdrag did not finish the drag; see $OUT/vdrag.log"
    [ "$moving" = 1 ] || fail "$1: the timed segment never ran as an interactive move (a view pan, or the lift failed)"
    echo "$1: interactive move confirmed during the timed segment" | tee -a "$OUT/clips.txt"
    stop_nested
    finish_sub_run "$1"
}
seq_native() {
    start_nested "$OUT/tiled.kdl" native
    python3 "$VDRAG" --socket "$NESTED_WAYLAND" --extent 1280,720 --dx 0 --frames 0 \
        --ready "$RT/native.ready" --go "$RT/native.never" --done "$RT/native.done" >> "$OUT/vdrag.log" 2>&1 &
    VDRAG_PID=$!
    for _ in $(seq 150); do [ -e "$RT/native.ready" ] && break; sleep 0.1; done
    msg action focus-window --id "$(app_window vdrag)"; settle
    shot "$OUT/native-rest.png"
    burst_start native "$RUN_S"; sleep "$GO_DELAY"
    msg action move-column-left
    burst_wait native
    stop_nested
    finish_sub_run native
}

for s in $SEQUENCES; do
    case $s in
        scroll-fast) drag_sequence scroll-fast "$OUT/tiled.kdl" -20 24 ;;
        scroll-slow) drag_sequence scroll-slow "$OUT/tiled.kdl" -5 60 ;;
        float-fast)  drag_sequence float-fast "$OUT/floating.kdl" 20 24 ;;
        float-slow)  drag_sequence float-slow "$OUT/floating.kdl" 5 60 ;;
        native)      seq_native ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
