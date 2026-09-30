#!/usr/bin/env bash
# ring-motion-clips.sh: review clips for the ring beam
# (docs/specs/2026-09-19-ring-beam-design.md §6, "Sheets"). Eight sequences
# from a nested headless instance, each one focus gain recorded as a burst of
# full-frame screenshots as fast as the async screenshot path allows, plus 4x
# crops of the focused pane's corners: top-left at rest and as the tail
# clears, and at mid-pass the corner the beam head is nearest (it has left
# the top-left corner by then), under the capture protocol
# (tools/capture-meta: preflight, identity, settle before every launch,
# release). The clips are for the
# owner's judgment of head, tail, glow, spill and gap; this script records,
# it does not grade.
#
# Sequences (SEQUENCES env, space-separated, default all):
#   beam-run      two panes; focus the right one, settle, focus the left one:
#                 RUN_S of frames from before the gain; `bevel 10; ring-gap 8`
#   beam-gap16    the same at `ring-gap 16`
#   beam-nospill  the same with BEAM_SPILL at 0, from a scratch build
#   beam-splash   the same with BEAM_ENVELOPE at Splash, from a scratch build
#   beam-fast     the same at `ring-beam-speed 900`
#   beam-wander   the same at `ring-beam-noise 0.5; ring-beam-noise-hz 4`:
#                 the head's brightness wanders as it travels
#   beam-decay    the same at `ring-beam-decay 1500`: the comet darkens and
#                 is gone after 1500 px, before the lap closes
#   beam-bevel0   the same on a flat slab: `bevel 0; offset-x 0; offset-y 0`
#                 (no chamfer, so no spill)
#
# The scratch sequences build the tree twice more with one constant changed
# each: a tracked-files copy under $OUT/src-<name> is edited with sed (the
# worktree is never touched) and built into $OUT/target-scratch. Set
# BEAM_SCRATCH=0 to skip both when that build is too slow for the host; the
# skip is recorded in clips.txt.
#
# Env: NIRI_MATERIAL_WORK_ROOT (artifacts land under it), CAPTURE_TASK (the
# task id authorizing the run), NIRI (default: this checkout's release build,
# built here), BEAM_SCRATCH (default 1). Requires: weston, kitty, swaybg, jq,
# ImageMagick, cargo (scratch sequences).
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root}
TASK=${CAPTURE_TASK:?task id authorizing this run}
RUN=ring-clips-$$-$(date +%s)
OUT=$EVIDENCE/ring-motion-clips-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # short: nested niri panics on long socket paths
SEQUENCES=${SEQUENCES:-"beam-run beam-gap16 beam-nospill beam-splash beam-fast beam-wander beam-decay beam-bevel0"}
BEAM_SCRATCH=${BEAM_SCRATCH:-1}
RUN_S=15   # (P + L) / 300 on a two-column 1280x720 pane is under 12 s; 15 s keeps the tail clearing in frame
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
        # Weston's GL renderer holds the GPU at P0; after the unit stops it
        # steps down through P5 to P8 over about two seconds. The next
        # sequence's settle samples at once and must not see our own tail.
        sleep 4
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
# The gradient focus ring is off so the beam is the only focus cue in frame,
# and the startup hotkey overlay is skipped so nothing sits over the scene.
# `bevel 10; ring-gap 8` is the spec's pairing under review, with the shipped
# `ring-beam-speed 300` and `ring-glow 1`; SPEED, GAP and GLASS vary one of
# them per fixture. The idle gate is off: the nested instance sees no input.
write_config() {   # $1 = path; SPEED, GAP, GLASS, NOISE, NOISE_HZ, DECAY override the fixture's defaults
    cat > "$1" <<EOF
material "tg" { glass { ${GLASS:-bevel 10;} }; response "default" { ring-beam-speed ${SPEED:-300}; ring-gap ${GAP:-8}; ring-glow 1; ring-beam-noise ${NOISE:-0}; ring-beam-noise-hz ${NOISE_HZ:-3}; ring-beam-decay ${DECAY:-0}; }; }
window-rule { match app-id="^kitty$"; material "tg"; }
layout { focus-ring { off; }; gaps 24; }
hotkey-overlay { skip-at-startup; }
signal { idle-after-ms 0; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" $KITTY_OPTS "--hold" "true"
EOF
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; fail "$1 does not validate"; }
}
write_config "$OUT/beam.kdl"
GAP=16 write_config "$OUT/beam-gap16.kdl"
SPEED=900 write_config "$OUT/beam-fast.kdl"
NOISE=0.5 NOISE_HZ=4 write_config "$OUT/beam-wander.kdl"
DECAY=1500 write_config "$OUT/beam-decay.kdl"
GLASS='bevel 0; offset-x 0; offset-y 0;' write_config "$OUT/beam-bevel0.kdl"

# --- scratch builds ----------------------------------------------------------
# A build with one constant changed, for the sheets that compare it: the
# tracked files of HEAD are copied under $OUT/src-<name>, the sed edits are
# applied there and checked, and the copy builds into a target directory of
# its own under $OUT (shared by both scratch builds, so the dependencies
# compile once). Prints the binary's path. The worktree is never edited.
scratch_build() {   # $1 = name, $2... = "file|sed expression|expected line" edits
    local name=$1 edit file expr want; shift
    local src=$OUT/src-$name
    mkdir -p "$src"
    git archive HEAD | tar -x -C "$src"
    for edit in "$@"; do
        IFS='|' read -r file expr want <<< "$edit"
        sed -i "$expr" "$src/$file"
        grep -qF "$want" "$src/$file" || fail "scratch $name: $file lacks '$want' after '$expr'"
    done
    (cd "$src" && CARGO_TARGET_DIR=$OUT/target-scratch cargo build --release >> "$OUT/scratch-build.log" 2>&1) \
        || fail "scratch $name: build failed; see $OUT/scratch-build.log"
    cp "$OUT/target-scratch/release/niri" "$OUT/niri-$name"
    sha256sum "$OUT/niri-$name" | tee -a "$OUT/SHA256SUMS" >&2
    echo "$OUT/niri-$name"
}
wants_scratch() {   # $1 = sequence
    [ "$BEAM_SCRATCH" != 0 ] || return 1
    case " $SEQUENCES " in *" $1 "*) return 0 ;; *) return 1 ;; esac
}
NIRI_NOSPILL=; NIRI_SPLASH=; BINARIES=(--binary "$NIRI")
if wants_scratch beam-nospill; then
    NIRI_NOSPILL=$(scratch_build nospill \
        'src/render_helpers/material/ring.rs|s/^pub const BEAM_SPILL: f32 = 0.25;$/pub const BEAM_SPILL: f32 = 0.00;/|pub const BEAM_SPILL: f32 = 0.00;' \
        'src/render_helpers/shaders/material/main.frag|s/^\( *\)const float BEAM_SPILL = 0.25;$/\1const float BEAM_SPILL = 0.00;/|const float BEAM_SPILL = 0.00;')
    BINARIES+=(--binary "$NIRI_NOSPILL")
fi
if wants_scratch beam-splash; then
    NIRI_SPLASH=$(scratch_build splash \
        'src/render_helpers/material/ring.rs|s/^pub const BEAM_ENVELOPE: Envelope = Envelope::Plateau;$/pub const BEAM_ENVELOPE: Envelope = Envelope::Splash;/|pub const BEAM_ENVELOPE: Envelope = Envelope::Splash;')
    BINARIES+=(--binary "$NIRI_SPLASH")
fi

capture_meta identity "$OUT" --source "$ROOT" "${BINARIES[@]}" --input "$0" \
    --input "$OUT/beam.kdl" --input "$OUT/beam-gap16.kdl" --input "$OUT/beam-fast.kdl" \
    --input "$OUT/beam-wander.kdl" --input "$OUT/beam-decay.kdl" --input "$OUT/beam-bevel0.kdl" || fail "identity refused"

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
# 4x corner crops of the focused pane: the rest shot taken before the gain at
# the top-left corner (the beam launches from the end of the top-left arc),
# the burst frame nearest 2 s (mid-pass) cropped at the corner the head is
# nearest by then, and the frame nearest 13 s (tail clear) back at the
# top-left.
corner_crops() {   # $1 = label, $2 = rest shot, $3 = config
    local d=$OUT/$1-corner b=$OUT/$1 at f name atxy
    mkdir -p "$d"
    magick "$2" -crop 320x240+0+0 +repage -scale 400% "$d/rest.png"
    for at in 2 13; do
        f=$(awk -v at="$at" '{ d = $2 - at; if (d < 0) d = -d; if (best == "" || d < bd) { best = $1; bd = d } } END { print best }' "$b/frames.txt")
        [ -n "$f" ] || fail "$1: no frame near $at s in $b/frames.txt"
        name=mid-pass; [ "$at" = 13 ] && name=tail-clear
        atxy=0+0; [ "$at" = 2 ] && atxy=$(head_corner_crop "$3" "$at")
        magick "$b/$f" -crop "320x240+$atxy" +repage -scale 400% "$d/$name.png"
        echo "$1-corner: $name is $f (${at} s at +$atxy)" >> "$OUT/clips.txt"
    done
}

# The 320x240 crop offset for the mid-pass frame: the corner of the focused
# pane the beam head is nearest at $2 s. The head leaves the top-left corner
# along the top edge at the config's ring-beam-speed, so its arc position is
# speed x t and the corners sit at 0, w, w + h and 2w + h on the perimeter.
# The focused pane sits one gap in from the frame's top-left (the scene's
# only column and row), so its corners come from the config's gaps plus the
# focused window's offset and size in its tile. The crop keeps the corner
# one gap inside it, mirroring the rest and tail-clear crops at the
# top-left. Prints "x+y" for the crop's geometry suffix.
head_corner_crop() {   # $1 = config, $2 = seconds
    local speed gaps geo
    speed=$(grep -oE 'ring-beam-speed [0-9]+' "$1") || fail "$1: no ring-beam-speed"
    gaps=$(grep -oE 'gaps [0-9]+' "$1") || fail "$1: no gaps"
    geo=$(msg -j windows | jq -r '.[] | select(.is_focused) | [.layout.window_size[0], .layout.window_size[1], .layout.window_offset_in_tile[0], .layout.window_offset_in_tile[1]] | @tsv')
    [ -n "$geo" ] || fail "no focused window for the mid-pass corner"
    awk -v t="$2" -v s="${speed##* }" -v g="${gaps##* }" '
        $1 > 0 && $2 > 0 {
            x = g + $3; y = g + $4; w = $1; h = $2
            per = 2 * (w + h)
            a = s * t; a -= int(a / per) * per
            n = split("0," w "," w + h "," 2 * w + h, ca, ",")
            split(x "," x + w "," x + w "," x, cx, ",")
            split(y "," y "," y + h "," y + h, cy, ",")
            split("top-left,top-right,bottom-right,bottom-left", cn, ",")
            best = -1
            for (i = 1; i <= n; i++) {
                d = a - ca[i]; if (d < 0) d = -d
                if (per - d < d) d = per - d
                if (best < 0 || d < best) { best = d; bx = cx[i]; by = cy[i]; bn = cn[i] }
            }
            ox = bx - g; oy = by - g
            if (bn ~ /right/) ox = bx + g - 320
            if (bn ~ /bottom/) oy = by + g - 240
            printf "%d+%d\n", ox, oy
        }' <<< "$geo"
}
# One focus gain on a two-pane scene: the right pane focused and settled,
# the burst started, then focus to the left pane. The burst samples at its
# natural rate for RUN_S from before the gain, so the whole run and the tail
# clearing are in frame.
beam_sequence() {   # $1 = label, $2 = config, $3 = binary
    local saved=$NIRI ids
    NIRI=$3
    start_nested "$2" "$1"
    spawn_kitty_to 2
    ids=($(kitty_ids))
    msg action focus-window --id "${ids[1]}"; settle
    shot "$OUT/$1-rest.png"
    burst_start "$1" "$RUN_S"
    msg action focus-window --id "${ids[0]}"
    burst_wait "$1"
    corner_crops "$1" "$OUT/$1-rest.png" "$2"
    stop_nested
    NIRI=$saved
}
seq_beam_run()     { beam_sequence beam-run     "$OUT/beam.kdl"        "$NIRI"; }
seq_beam_gap16()   { beam_sequence beam-gap16   "$OUT/beam-gap16.kdl"  "$NIRI"; }
seq_beam_nospill() { beam_sequence beam-nospill "$OUT/beam.kdl"        "$NIRI_NOSPILL"; }
seq_beam_splash()  { beam_sequence beam-splash  "$OUT/beam.kdl"        "$NIRI_SPLASH"; }
seq_beam_fast()    { beam_sequence beam-fast    "$OUT/beam-fast.kdl"   "$NIRI"; }
seq_beam_wander()  { beam_sequence beam-wander  "$OUT/beam-wander.kdl" "$NIRI"; }
seq_beam_decay()   { beam_sequence beam-decay   "$OUT/beam-decay.kdl"  "$NIRI"; }
seq_beam_bevel0()  { beam_sequence beam-bevel0  "$OUT/beam-bevel0.kdl" "$NIRI"; }

for s in $SEQUENCES; do
    case $s in
        beam-run)     seq_beam_run ;;
        beam-gap16)   seq_beam_gap16 ;;
        beam-nospill) if [ "$BEAM_SCRATCH" = 0 ]; then echo "$s: skipped (BEAM_SCRATCH=0)" | tee -a "$OUT/clips.txt"; else seq_beam_nospill; fi ;;
        beam-splash)  if [ "$BEAM_SCRATCH" = 0 ]; then echo "$s: skipped (BEAM_SCRATCH=0)" | tee -a "$OUT/clips.txt"; else seq_beam_splash; fi ;;
        beam-fast)    seq_beam_fast ;;
        beam-wander)  seq_beam_wander ;;
        beam-decay)   seq_beam_decay ;;
        beam-bevel0)  seq_beam_bevel0 ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
