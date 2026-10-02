#!/usr/bin/env bash
# focus-swap-clips.sh: review clips for the focus-conditioned material swap
# (material-8e3b73). The live Prism config selects `terminal-glass` for the
# active kitty and `terminal-glass-inactive` for the others; a focus change
# replaces both tiles' MaterialState, so the glass changes in one frame. Each
# sequence is one focus drive on a two-pane scene, recorded as a burst of
# full-frame screenshots (about three a second on the headless host, the
# async screenshot path's rate) under the capture protocol (tools/capture-meta:
# preflight, identity, settle before every launch, release). Per sequence it
# writes a GIF, a contact sheet, per-pane frame-to-frame RMSE (diffs.txt) and
# the largest step's before/after pair. The clips are for the owner's
# judgment; this script records, it does not grade.
#
# The two definitions are pinned from prism.kdl of 2026-09-30 (the harness
# never reads the generated file, which drifts). Every sequence except
# `beam` sets `ring-beam-speed 0`, so the finite focus beam is out of frame
# and what changes on focus is the glass swap and the 400 ms ring-light
# crossfade (ring-glow 0.1).
#
# Sequences (SEQUENCES env, space-separated, default all):
#   swap       live pair; right pane focused and settled, then focus left
#   same       control: `terminal-glass` in both focus states, same drive
#   seed       two definitions identical but for their names, with a pinned
#              aurora field (`aurora 0.6 { drift-hz 0; }`): only the seed
#              replacement can change the glass
#   reversal   live pair; focus left, then right again REVERSAL_MS later
#   move       live pair; move the focused column left, and MOVE_MS later
#              focus the other column while both columns are in motion
#   beam       live pair with the live response (ring-beam-speed 3200,
#              noise 0.9, decay 600): the swap as it is seen daily
#
# Env: NIRI_MATERIAL_WORK_ROOT (artifacts land under it), CAPTURE_TASK (the
# task id authorizing the run), NIRI (default: this checkout's release build,
# built here), BURST_S (default 3), REVERSAL_MS (default 120), MOVE_MS
# (default 100). Requires: weston, kitty, swaybg, jq, ImageMagick.
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root}
TASK=${CAPTURE_TASK:?task id authorizing this run}
RUN=focus-swap-$$-$(date +%s)
OUT=$EVIDENCE/focus-swap-clips-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # short: nested niri panics on long socket paths
SEQUENCES=${SEQUENCES:-"swap same seed reversal move beam"}
BURST_S=${BURST_S:-3}
REVERSAL_MS=${REVERSAL_MS:-120}
MOVE_MS=${MOVE_MS:-100}
UNIT=; HOST_SOCKET=; NIRI_PID=; NIRI_SOCKET=; HOST_SEQ=0
mkdir -p "$OUT" "$RT"

capture_meta() { python3 "$ROOT/tools/capture-meta" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
pid_running() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }
msg() { "$NIRI" msg "$@"; }
kitty_ids() { msg -j windows | jq -r '[.[] | select(.app_id=="kitty")] | sort_by(.layout.pos_in_scrolling_layout[0]) | .[].id'; }
kitty_count() { kitty_ids | wc -l; }
wait_kitty() { for _ in $(seq 100); do [ "$(kitty_count)" -ge "$1" ] && return; sleep 0.1; done; fail "only $(kitty_count) kitty windows, wanted $1"; }
ms_sleep() { sleep "$(awk -v ms="$1" 'BEGIN { printf "%.3f", ms / 1000 }')"; }

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
        # Weston's GL renderer holds the GPU at P0 and steps down over about
        # two seconds; the next settle must not see our own tail.
        sleep 4
    fi
    rm -f "$RT"/*
}
cleanup() {
    local rc=$?
    [ -n "${BURST_PID:-}" ] && kill "$BURST_PID" 2>/dev/null || true
    stop_nested
    rm -rf "$RT"
    capture_meta release "$OUT" || true
    exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

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
TRANSCRIPT=$OUT/transcript.txt
sed -n '1,40p' "$0" > "$TRANSCRIPT"
KITTY=(kitty -o cursor_blink_interval=0 -o cursor_stop_blinking_after=0 -o background_opacity=0 --hold cat "$TRANSCRIPT")
kdl_args() { local a out=; for a in "$@"; do out+="\"$a\" "; done; echo "$out"; }

# Glass of the two live definitions, pinned; aurora is appended per fixture.
ACTIVE_GLASS='ior 1.28; light-ior 2.5; thickness 31.2; attenuation-color "#152F30"; attenuation-distance 16; chromatic-aberration 0.36; distortion 0 scale=0.09; anisotropic-blur 0; roughness 0.24; iridescence 0; noise 0.03 type="white"; saturation 0.95; backdrop-blur true; jelly-flex 0.0066; jelly-ripple 0.23; bevel 12; offset-x 0; offset-y 0;'
INACTIVE_GLASS='ior 1.28; light-ior 2.5; thickness 49.5; attenuation-color "#0C1314"; attenuation-distance 28; chromatic-aberration 0.34; distortion 0.42 scale=0.08; anisotropic-blur 0; roughness 0.03; iridescence 0; noise 0.03 type="white"; saturation 0.8; backdrop-blur false; jelly-flex 0.0066; jelly-ripple 0.23; bevel 12; offset-x 0; offset-y 0;'
AURORA_OFF='aurora 0 { drift-hz 4; color "#3dffb0"; color "#7a5cff"; };'
AURORA_PINNED='aurora 0.6 { drift-hz 0; color "#3dffb0"; color "#7a5cff"; };'
response() {   # $1 = ring-beam-speed
    echo "response \"default\" { accent \"ring\"; focus \"ring-light\"; ring-color \"#ccccff\"; ring-beam-speed $1; ring-beam-noise 0.9; ring-beam-noise-hz 0.5; ring-beam-decay 600; ring-gap 8; ring-width 0.6; ring-glow 0.1; }"
}
# $1 = path, $2 = active material name, $3 = inactive material name, then
# "name|glass|beam speed" definitions.
write_config() {
    local path=$1 active=$2 inactive=$3 def name glass speed; shift 3
    {
        for def in "$@"; do
            IFS='|' read -r name glass speed <<< "$def"
            echo "material \"$name\" { glass { $glass }; $(response "$speed"); }"
        done
        cat <<EOF
window-rule { match app-id="^kitty$" is-active=true; material "$active"; }
window-rule { match app-id="^kitty$" is-active=false; material "$inactive"; }
layout { focus-ring { off; }; gaps 34; }
hotkey-overlay { skip-at-startup; }
signal { idle-after-ms 0; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup $(kdl_args "${KITTY[@]}")
EOF
    } > "$path"
    "$NIRI" validate -c "$path" >/dev/null 2>&1 || { "$NIRI" validate -c "$path"; fail "$path does not validate"; }
}
write_config "$OUT/pair.kdl" terminal-glass terminal-glass-inactive \
    "terminal-glass|$ACTIVE_GLASS $AURORA_OFF|0" "terminal-glass-inactive|$INACTIVE_GLASS $AURORA_OFF|0"
write_config "$OUT/same.kdl" terminal-glass terminal-glass \
    "terminal-glass|$ACTIVE_GLASS $AURORA_OFF|0"
write_config "$OUT/seed.kdl" seed-a seed-b \
    "seed-a|$ACTIVE_GLASS $AURORA_PINNED|0" "seed-b|$ACTIVE_GLASS $AURORA_PINNED|0"
write_config "$OUT/beam.kdl" terminal-glass terminal-glass-inactive \
    "terminal-glass|$ACTIVE_GLASS $AURORA_OFF|3200" "terminal-glass-inactive|$INACTIVE_GLASS $AURORA_OFF|3200"

capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" --input "$0" \
    --input "$OUT/pair.kdl" --input "$OUT/same.kdl" --input "$OUT/seed.kdl" \
    --input "$OUT/beam.kdl" || fail "identity refused"

# --- nested instance ---------------------------------------------------------
start_nested() {   # $1 = config, $2 = sub-run name; sets NIRI_SOCKET
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
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    wait_kitty 1
}
spawn_kitty_to() {   # $1 = total count wanted
    while [ "$(kitty_count)" -lt "$1" ]; do
        local before; before=$(kitty_count)
        msg action spawn -- "${KITTY[@]}"
        wait_kitty $((before + 1))
    done
}
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
# start in frames.txt, and the drive's instants in drive.txt.
BURST_PID=; T0=
burst_start() {   # $1 = label
    local d=$OUT/$1; mkdir -p "$d"
    T0=$(date +%s.%N)
    (
        local i=0 now f
        while :; do
            now=$(date +%s.%N)
            awk -v a="$now" -v b="$T0" -v s="$BURST_S" 'BEGIN { exit !(a - b < s) }' || break
            i=$((i + 1)); f=$d/f$(printf %03d "$i").png
            printf '%s %s\n' "$(basename "$f")" "$(awk -v a="$now" -v b="$T0" 'BEGIN { printf "%.3f", a - b }')" >> "$d/frames.txt"
            shot "$f"
        done
        echo "$1: $i frames in $BURST_S s" >> "$OUT/timing.txt"
    ) &
    BURST_PID=$!
}
drive() {   # $1 = label, $2... = niri msg action
    local label=$1; shift
    msg action "$@"
    printf '%s %s\n' "$(awk -v a="$(date +%s.%N)" -v b="$T0" 'BEGIN { printf "%.3f", a - b }')" "$*" >> "$OUT/$label/drive.txt"
}
# Per-pane RMSE of each frame against the one before it, on the 640 px
# halves of the 1280 x 720 output; the pair with the largest step is kept
# side by side as <label>-step.png.
pane_rmse() {   # $1 $2 = frames, $3 = x offset
    # compare exits 1 when the images differ, which is the measurement; 2 is an error.
    local out rc=0
    out=$(magick compare -metric RMSE <(magick "$1" -crop 640x720+"$3"+0 +repage png:-) \
        <(magick "$2" -crop 640x720+"$3"+0 +repage png:-) null: 2>&1) || rc=$?
    [ "$rc" -le 1 ] || fail "compare $1 $2 failed ($rc): $out"
    sed -n 's/.*(\([0-9.e-]*\)).*/\1/p' <<< "$out"
}
diffs() {   # $1 = label
    local d=$OUT/$1 prev='' f t left right
    echo "# frame t_s left_rmse right_rmse (normalized, against the previous frame)" > "$d/diffs.txt"
    while read -r f t; do
        if [ -n "$prev" ]; then
            left=$(pane_rmse "$d/$prev" "$d/$f" 0); right=$(pane_rmse "$d/$prev" "$d/$f" 640)
            echo "$f $t $left $right" >> "$d/diffs.txt"
        fi
        prev=$f
    done < "$d/frames.txt"
    local step before
    step=$(awk '!/^#/ { m = ($3 > $4) ? $3 : $4; if (m > best) { best = m; f = $1 } } END { print f }' "$d/diffs.txt")
    [ -n "$step" ] || { echo "$1: no nonzero step" >> "$OUT/clips.txt"; return; }
    before=$(awk -v f="$step" '$1 == f { print prev } { prev = $1 }' "$d/frames.txt")
    magick "$d/$before" "$d/$step" +append -scale 50% "$OUT/$1-step.png"
    echo "$1: largest step $before -> $step; $OUT/$1-step.png" >> "$OUT/clips.txt"
}
burst_wait() {   # $1 = label
    wait "$BURST_PID"; BURST_PID=
    local d=$OUT/$1
    [ -n "$(ls "$d"/f*.png 2>/dev/null)" ] || fail "$1: no frames"
    magick -delay 10 -loop 0 "$d"/f*.png -scale 50% "$OUT/$1.gif"
    magick montage "$d"/f*.png -tile 8x -geometry 320x180+2+2 -background '#111' "$OUT/$1-sheet.png"
    diffs "$1"
    echo "$1: $d ($(ls "$d"/f*.png | wc -l) frames); gif $OUT/$1.gif" | tee -a "$OUT/clips.txt"
}

# --- sequences ---------------------------------------------------------------
# Two panes, the right one focused and settled; the burst starts, and the
# drive runs half a second in, so the frames before it are the rest state.
sequence() {   # $1 = label, $2 = config, $3 = drive function
    local ids
    start_nested "$2" "$1"
    spawn_kitty_to 2
    mapfile -t ids < <(kitty_ids)
    msg action focus-window --id "${ids[1]}"; settle
    shot "$OUT/$1-rest.png"
    burst_start "$1"
    sleep 0.5
    "$3" "$1" "${ids[@]}"
    burst_wait "$1"
    stop_nested
}
drive_focus_left() { drive "$1" focus-window --id "$2"; }
drive_reversal() { drive "$1" focus-window --id "$2"; ms_sleep "$REVERSAL_MS"; drive "$1" focus-window --id "$3"; }
drive_move() { drive "$1" move-column-left; ms_sleep "$MOVE_MS"; drive "$1" focus-column-right; }

for s in $SEQUENCES; do
    case $s in
        swap)     sequence swap     "$OUT/pair.kdl" drive_focus_left ;;
        same)     sequence same     "$OUT/same.kdl" drive_focus_left ;;
        seed)     sequence seed     "$OUT/seed.kdl" drive_focus_left ;;
        reversal) sequence reversal "$OUT/pair.kdl" drive_reversal ;;
        move)     sequence move     "$OUT/pair.kdl" drive_move ;;
        beam)     sequence beam     "$OUT/beam.kdl" drive_focus_left ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
