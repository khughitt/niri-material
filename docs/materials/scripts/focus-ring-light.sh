#!/usr/bin/env bash
# Focus-light spike harness (material-d1f471): nests a niri built with the
# focus-ring-light probe patch (niri-experiments
# fixtures/focus-ring-light-probe.patch) under a headless Weston host, opens
# two transparent kitty windows over the dark split glass from the
# focus-glass spike, and for each candidate captures a still per focus side,
# an idle burst (light motion alone), a move burst (jelly residuals, with
# animations slowed 6x), and the probe's draws/s on an idle focused window.
#
# Cases: baseline (the static gradient ring, no probe), ring, motes, rays,
# shadow, noise, canopy, ring-hz20 (ring with NIRI_FOCUS_PROBE_HZ=20).
#
# Env: NIRI (probe binary; default the release build of this checkout),
# CASES (space-separated, default all), NIRI_MATERIAL_WORK_ROOT (captures
# land under it when set, else ./work), SPIKE_WALLPAPER (backdrop image;
# default a generated checkerboard), BURST (frames per burst, 12).
# Requires: weston, kitty, swaybg, jq, ImageMagick, bc, and a Prism prism.kdl
# in $XDG_STATE_HOME/prism/generated for the "terminal-glass" block.
set -eu
HERE=$(dirname "$(readlink -f "$0")")
REPO=$(cd "$HERE/../../.." && pwd)
NIRI=${NIRI:-$(cd "$REPO" && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/niri}
CONTENT=$HERE/focus-glass-content.sh
SHA=$(sha256sum "$NIRI" | cut -c1-8)
WORK=${NIRI_MATERIAL_WORK_ROOT:-$PWD/work}/focus-ring-light-$SHA; mkdir -p "$WORK"
RT=$XDG_RUNTIME_DIR/frl-rt; rm -rf "$RT"; mkdir -p "$RT"
HOST=frl-host-$$; UNIT=$HOST-weston; NIRI_PID=
PRISM_KDL=${XDG_STATE_HOME:-$HOME/.local/state}/prism/generated/prism.kdl
BURST=${BURST:-12}
WALL=${SPIKE_WALLPAPER:-}
if [ -z "$WALL" ]; then WALL=$WORK/checker.png; magick -size 160x90 pattern:checkerboard -scale 800% "$WALL"; fi
cleanup() {
    [ -n "$NIRI_PID" ] && kill "$NIRI_PID" 2>/dev/null; wait 2>/dev/null || true
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -rf "$RT"
}
trap cleanup EXIT
msg() { "$NIRI" msg "$@"; }
kitty_count() { msg -j windows | jq -r '.[] | select(.app_id=="kitty") | .id' | wc -l; }
wait_kitty() { for _ in $(seq 100); do [ "$(kitty_count)" -ge "$1" ] && return; sleep 0.1; done; echo "FAIL: kitty $1" >&2; exit 1; }

ACTIVE=$(sed -n '/^material "terminal-glass"/,/^}/p' "$PRISM_KDL")
[ -n "$ACTIVE" ] || { echo "FAIL: no terminal-glass in $PRISM_KDL" >&2; exit 1; }
DARK_ACTIVE=$(printf '%s\n' "$ACTIVE" | sed \
    -e 's/"terminal-glass"/"dark"/' \
    -e 's/attenuation-color .*/attenuation-color "#222436"/' \
    -e 's/attenuation-distance .*/attenuation-distance 30/')
DARK_INACTIVE=$(printf '%s\n' "$DARK_ACTIVE" | sed \
    -e 's/"dark"/"dark-inactive"/' \
    -e 's/roughness .*/roughness 0.5/' \
    -e 's/chromatic-aberration .*/chromatic-aberration 0.08/' \
    -e 's/distortion .*/distortion 0.10 scale=0.03/' \
    -e 's/attenuation-distance .*/attenuation-distance 70/')

write_config() {   # $1 path, $2 case
    local ring='focus-ring { off; }'
    [ "$2" = baseline ] && ring='focus-ring { width 4; active-gradient from="#ffffff00" to="#ccccff11" angle=45 relative-to="workspace-view"; }'
    {
        cat <<KDL
layout {
    gaps 54
    default-column-width { proportion 0.5; }
    $ring
    border { off; }
    shadow { on; softness 5; spread 1; offset x=-5 y=5; color "#00000070"; }
}
hotkey-overlay { skip-at-startup; }
animations { slowdown 6.0; }
window-rule { match app-id="^kitty$"; geometry-corner-radius 12; clip-to-geometry true; }
window-rule { match app-id="^kitty$" is-active=true; material "dark"; }
window-rule { match app-id="^kitty$" is-active=false; material "dark-inactive"; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
$DARK_ACTIVE
$DARK_INACTIVE
KDL
        if [ "$2" = shadow ]; then
            cat <<'KDL'
window-rule {
    match app-id="^kitty$" is-active=true
    shadow { softness 28; spread 4; offset x=-8 y=14; color "#000000a0"; }
}
window-rule {
    match app-id="^kitty$" is-active=false
    shadow { softness 4; spread 0; offset x=-2 y=2; color "#00000050"; }
}
KDL
        fi
    } > "$1"
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; exit 1; }
}

start_nested() {   # $1 config, $2 probe name or "", $3 hz or ""
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$HOST" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || { echo "FAIL: no Weston socket" >&2; exit 1; }
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST NIRI_FOCUS_PROBE=${2:-} NIRI_FOCUS_PROBE_HZ=${3:-60} \
        "$NIRI" -c "$1" >> "$LOG" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
spawn_kitty() {   # $1 label
    msg action spawn -- kitty --config NONE -o background_opacity=0 -o background=#222436 -o foreground=#c8d3f5 \
        -o cursor_blink_interval=0 -o hide_window_decorations=yes -o font_size=11 -o window_padding_width=8 --hold "$CONTENT" "$1"
}
shot() {   # $1 path
    msg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"
    # The write is async: wait for a complete, decodable PNG, not just a non-empty file.
    for _ in $(seq 200); do [ -s "$1" ] && magick identify "$1" >/dev/null 2>&1 && return; sleep 0.05; done
    echo "FAIL: shot $1" >&2; exit 1
}
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
cost() {   # $1 label: idle focused window for 6 s, then harvest draws/s lines
    local before; before=$(grep -c "focus-probe draws/s" "$LOG" || true)
    sleep 6
    { echo "== $1 (idle, focused)"; grep "focus-probe draws/s" "$LOG" | tail -n +"$((before + 1))" | sed 's/.*focus-probe/focus-probe/'; } >> "$WORK/cost.txt"
}
run_case() {   # $1 case name, $2 probe, $3 hz
    LOG=$WORK/$1.log; : > "$LOG"
    write_config "$WORK/$1.kdl" "$1"
    start_nested "$WORK/$1.kdl" "$2" "$3"
    spawn_kitty left;  wait_kitty 1
    spawn_kitty right; wait_kitty 2
    sleep 2
    cost "$1"
    still "$1-right-focused" right
    burst "$1-idle" right
    msg action focus-column-left; sleep 1.5
    still "$1-left-focused" left
    # Jelly: push the focused (left) column right so it slides and the ring
    # sees residuals; capture from the moment the move starts.
    msg action move-column-right
    burst "$1-move" right
    stop_nested
}

sha256sum "$NIRI" > "$WORK/SHA256SUMS"
"$NIRI" --version >> "$WORK/SHA256SUMS"
git -C "$REPO" rev-parse HEAD >> "$WORK/SHA256SUMS"
for c in ${CASES:-baseline ring motes rays shadow noise canopy ring-hz20}; do
    case $c in
        baseline) run_case baseline "" "" ;;
        ring-hz20) run_case ring-hz20 ring 20 ;;
        *) run_case "$c" "$c" "" ;;
    esac
done
echo "captures: $WORK"
