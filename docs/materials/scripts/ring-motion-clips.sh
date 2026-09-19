#!/usr/bin/env bash
# ring-motion-clips.sh: review clips for the focus sweep and the idle gate
# (docs/specs/2026-09-18-ring-focus-motion-design.md §7). Four sequences
# from a nested headless instance, each recorded as a burst of full-frame
# screenshots as fast as the async screenshot path allows, under the capture
# protocol (tools/capture-meta: preflight, identity, settle before every
# launch, release). The clips are for the owner's judgment of the
# `ring-sweep-ms` starting value and the ease; this script records, it does
# not grade.
#
# Sequences (SEQUENCES env, space-separated, default all):
#   gain-from-rest      two windows; focus the other, wait 3 s, focus back;
#                       2.5 s of frames from the focus command
#   alt-tab-three       three windows; focus-window across all three at
#                       0.4 s spacing; 4 s of frames
#   loss-mid-lap        focus back, then away at 0.5 s; 2.5 s of frames
#   idle-freeze-resume  idle-after-ms 5000 and a breathing window; frames
#                       from 4 s to 7 s after launch (the freeze), one
#                       `wlrctl pointer move 1 0` spawned inside the
#                       instance, then 3 s more (the resume)
#
# Env: NIRI_MATERIAL_WORK_ROOT (artifacts land under it), CAPTURE_TASK (the
# task id authorizing the run), NIRI (default: this checkout's release build,
# built here). Requires: weston, kitty, swaybg, jq, ImageMagick, wlrctl
# (idle-freeze-resume only).
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root}
TASK=${CAPTURE_TASK:?task id authorizing this run}
RUN=ring-clips-$$-$(date +%s)
OUT=$EVIDENCE/ring-motion-clips-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # short: nested niri panics on long socket paths
SEQUENCES=${SEQUENCES:-"gain-from-rest alt-tab-three loss-mid-lap idle-freeze-resume"}
UNIT=; HOST_SOCKET=; NIRI_PID=; NIRI_SOCKET=; HOST_SEQ=0
mkdir -p "$OUT" "$RT"

capture_meta() { python3 "$ROOT/tools/capture-meta" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
pid_running() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }
msg() { "$NIRI" msg "$@"; }
kitty_ids() { msg -j windows | jq -r '[.[] | select(.app_id=="kitty")] | sort_by(.layout.pos_in_scrolling_layout[0]) | .[].id'; }
kitty_count() { kitty_ids | wc -l; }
wait_kitty() { for _ in $(seq 100); do [ "$(kitty_count)" -ge "$1" ] && return; sleep 0.1; done; fail "only $(kitty_count) kitty windows, wanted $1"; }
focused_id() { msg -j windows | jq -r '.[] | select(.is_focused) | .id'; }

stop_nested() {
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
    fi
    rm -f "$RT"/*
}
cleanup() {
    local rc=$?
    stop_nested
    rm -rf "$RT"
    capture_meta release "$OUT" || true
    exit "$rc"
}
trap cleanup EXIT

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
sha256sum "$NIRI" | tee -a "$OUT/SHA256SUMS"

# --- configs -----------------------------------------------------------------
CHECKER=$OUT/checker.png
magick -size 160x90 pattern:checkerboard -scale 800% "$CHECKER"
KITTY_OPTS='"-o" "cursor_blink_interval=0" "-o" "cursor_stop_blinking_after=0" "-o" "background_opacity=0.6"'
# The gradient focus ring is off so the filament is the only focus cue in
# frame. `ring-sweep-ms 1500` is the default under review; the idle fixture
# turns the gate to 5 s so the freeze lands inside a short capture.
write_config() {   # $1 = path, $2 = idle-after-ms
    cat > "$1" <<EOF
material "tg" { glass {}; response "default" { ring-sweep-ms 1500; }; }
window-rule { match app-id="^kitty$"; material "tg"; }
layout { focus-ring { off; }; gaps 24; }
signal { idle-after-ms $2; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" $KITTY_OPTS "--hold" "true"
EOF
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; fail "$1 does not validate"; }
}
write_config "$OUT/sweep.kdl" 0
write_config "$OUT/idle-5s.kdl" 5000

capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" --input "$0" \
    --input "$OUT/sweep.kdl" --input "$OUT/idle-5s.kdl" || fail "identity refused"

# --- nested instance ---------------------------------------------------------
LAUNCHED_AT=
start_nested() {   # $1 = config, $2 = sub-run name; sets NIRI_SOCKET, LAUNCHED_AT
    capture_meta settle "$OUT" --sub-run "$2" --input "$1" || fail "settle refused before $2; see $OUT/capture.json"
    HOST_SEQ=$((HOST_SEQ + 1))
    local host=$RUN-h$HOST_SEQ
    UNIT=$host-weston; HOST_SOCKET=$XDG_RUNTIME_DIR/$host
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$host" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$HOST_SOCKET" ] && break; sleep 0.1; done
    [ -S "$HOST_SOCKET" ] || fail "Weston socket never appeared at $HOST_SOCKET"
    ln -s "$HOST_SOCKET" "$RT/$host"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$host "$NIRI" -c "$1" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    LAUNCHED_AT=$(date +%s.%N)
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    wait_kitty 1
}
spawn_kitty_to() {   # $1 = total count wanted
    while [ "$(kitty_count)" -lt "$1" ]; do
        local before; before=$(kitty_count)
        msg action spawn -- kitty -o cursor_blink_interval=0 -o cursor_stop_blinking_after=0 -o background_opacity=0.6 --hold true
        wait_kitty $((before + 1))
    done
}
# Every window redraws its open animation and the client settles its first
# frames; wait for two shots 0.6 s apart to agree before driving a sequence.
shot_request() { rm -f "$1"; msg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"; }
shot_wait() { for _ in $(seq 200); do [ -s "$1" ] && magick identify "$1" >/dev/null 2>&1 && return; sleep 0.05; done; fail "shot $1"; }
shot() { shot_request "$1"; shot_wait "$1"; }
settle() {
    local a=$OUT/.settle-a.png b=$OUT/.settle-b.png
    for _ in $(seq 40); do
        shot "$a"; sleep 0.6; shot "$b"
        cmp -s "$a" "$b" && { rm -f "$a" "$b"; return; }
    done
    fail "scene never settled"
}

# --- bursts ------------------------------------------------------------------
# A burst runs in the background so the drive continues while frames are
# taken; each frame's request instant is recorded relative to the burst
# start in frames.txt, the count and span in timing.txt.
BURST_PID=
burst_start() {   # $1 = label, $2 = seconds
    local d=$OUT/$1; mkdir -p "$d"
    (
        local t0 i=0 now f
        t0=$(date +%s.%N)
        while :; do
            now=$(date +%s.%N)
            awk -v a="$now" -v b="$t0" -v s="$2" 'BEGIN { exit !(a - b < s) }' || break
            i=$((i + 1)); f=$d/f$(printf %03d "$i").png
            printf '%s %s\n' "$(basename "$f")" "$(awk -v a="$now" -v b="$t0" 'BEGIN { printf "%.3f", a - b }')" >> "$d/frames.txt"
            shot "$f"
        done
        echo "$1: $i frames in $2 s" >> "$OUT/timing.txt"
    ) &
    BURST_PID=$!
}
burst_wait() {
    wait "$BURST_PID"; BURST_PID=
    local d=$OUT/$1
    [ -n "$(ls "$d"/f*.png 2>/dev/null)" ] || fail "$1: no frames"
    magick -delay 8 -loop 0 "$d"/f*.png -scale 50% "$OUT/$1.gif"
    magick montage "$d"/f*.png -tile 8x -geometry 320x180+2+2 -background '#111' "$OUT/$1-sheet.png"
    echo "$1: $d ($(ls "$d"/f*.png | wc -l) frames); gif $OUT/$1.gif" | tee -a "$OUT/clips.txt"
}

# --- sequences ---------------------------------------------------------------
seq_gain_from_rest() {
    start_nested "$OUT/sweep.kdl" gain-from-rest
    spawn_kitty_to 2
    local ids; ids=($(kitty_ids))
    msg action focus-window --id "${ids[0]}"; settle
    msg action focus-window --id "${ids[1]}"; sleep 3; settle
    msg action focus-window --id "${ids[0]}"
    burst_start gain-from-rest 2.5
    burst_wait gain-from-rest
    stop_nested
}
seq_alt_tab_three() {
    start_nested "$OUT/sweep.kdl" alt-tab-three
    spawn_kitty_to 3
    local ids; ids=($(kitty_ids))
    msg action focus-window --id "${ids[0]}"; settle
    burst_start alt-tab-three 4
    msg action focus-window --id "${ids[1]}"; sleep 0.4
    msg action focus-window --id "${ids[2]}"; sleep 0.4
    msg action focus-window --id "${ids[0]}"
    burst_wait alt-tab-three
    stop_nested
}
seq_loss_mid_lap() {
    start_nested "$OUT/sweep.kdl" loss-mid-lap
    spawn_kitty_to 2
    local ids; ids=($(kitty_ids))
    msg action focus-window --id "${ids[1]}"; sleep 3; settle
    msg action focus-window --id "${ids[0]}"
    burst_start loss-mid-lap 2.5
    sleep 0.5
    msg action focus-window --id "${ids[1]}"
    burst_wait loss-mid-lap
    stop_nested
}
seq_idle_freeze_resume() {
    command -v wlrctl >/dev/null || fail "idle-freeze-resume requires wlrctl (AUR)"
    start_nested "$OUT/idle-5s.kdl" idle-freeze-resume
    local id; id=$(kitty_ids | head -1)
    msg set-window-signal --id "$id" --source demo --accent '#e5a33c' --level demand --motion breathe
    # Frames from 4 s to 7 s after launch: the gate engages at 5 s.
    local since; since=$(awk -v a="$(date +%s.%N)" -v b="$LAUNCHED_AT" 'BEGIN { printf "%.3f", a - b }')
    awk -v s="$since" 'BEGIN { exit !(s < 4) }' || fail "setup took $since s; the freeze window starts at 4 s"
    sleep "$(awk -v s="$since" 'BEGIN { printf "%.3f", 4 - s }')"
    burst_start idle-freeze 3
    burst_wait idle-freeze
    msg action spawn -- wlrctl pointer move 1 0
    burst_start idle-resume 3
    burst_wait idle-resume
    stop_nested
}

for s in $SEQUENCES; do
    case $s in
        gain-from-rest)     seq_gain_from_rest ;;
        alt-tab-three)      seq_alt_tab_three ;;
        loss-mid-lap)       seq_loss_mid_lap ;;
        idle-freeze-resume) seq_idle_freeze_resume ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
