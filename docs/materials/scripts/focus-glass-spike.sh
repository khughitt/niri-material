#!/usr/bin/env bash
# Focus-state glass spike (material-cb348e): screenshot two adjacent kitty
# windows under the installed niri, nested on a headless Weston host, to
# compare terminal background opacity against fully transparent terminals
# whose focus state is carried by an is-active material swap.
#
# Cases: baseline (kitty 0.44 unfocused / 0.98 focused over the Prism glass),
# zero-same (kitty 0 over the Prism glass), zero-split (kitty 0, Prism glass
# active / frosted variant inactive), zero-dark (kitty 0 over a dark dense
# glass), zero-dark-split (dark glass active / lighter frosted dark inactive).
#
# `animations { off; }` pins the ring of light. The focus response defaults to
# RingLight, which runs one lap of its travelling light on focus gain unless
# animations are off (`Tile::sweep_allowed`, src/layout/tile.rs).
# The ring is drawn at the window edge, which is where this spike's corner crops
# are, so without it two runs of the same case differed by ~1000 px per frame -
# measured at AE 994 full frame and Lab RMSE 0.015 on the bottom crop. With it
# the ring is still drawn, just static, and every corner crop is byte-identical
# across runs. Captures taken before this line was added are not comparable with
# ones taken after.
#
# One residue remains and is not the compositor: after `focus-column-left`, the
# full frame can differ by ~7 px in a single character cell inside the newly
# focused terminal, where kitty has repainted its cursor. It is ~195 px in from
# the window edge and outside every crop this script takes.
#
# Env: NIRI (default /usr/bin/niri), CASES (space-separated, default all),
# SPIKE_WALLPAPER (backdrop image; default a generated checkerboard),
# DARK_DIST / DARK_DIST_INACTIVE (dark attenuation distances, 30 / 70),
# NIRI_MATERIAL_WORK_ROOT (captures land under it when set, else ./work).
# Requires: weston, kitty, swaybg, jq, ImageMagick, and a Prism prism.kdl in
# $XDG_STATE_HOME/prism/generated for the "terminal-glass" block.
set -eu
HERE=$(dirname "$(readlink -f "$0")")
WORK=${NIRI_MATERIAL_WORK_ROOT:-$PWD/work}/focus-glass-spike; rm -rf "$WORK"; mkdir -p "$WORK"
RT=$XDG_RUNTIME_DIR/fg-rt; rm -rf "$RT"; mkdir -p "$RT"   # short: nested niri panics on long socket paths
NIRI=${NIRI:-/usr/bin/niri}
HOST=fg-host-$$; UNIT=$HOST-weston; NIRI_PID=
PRISM_KDL=${XDG_STATE_HOME:-$HOME/.local/state}/prism/generated/prism.kdl
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

ACTIVE_MATERIAL=$(sed -n '/^material "terminal-glass"/,/^}/p' "$PRISM_KDL")
[ -n "$ACTIVE_MATERIAL" ] || { echo "FAIL: no terminal-glass material in $PRISM_KDL" >&2; exit 1; }
DARK_ACTIVE=$(printf '%s\n' "$ACTIVE_MATERIAL" | sed \
    -e 's/"terminal-glass"/"terminal-glass-dark"/' \
    -e 's/attenuation-color .*/attenuation-color "#222436"/' \
    -e "s/attenuation-distance .*/attenuation-distance ${DARK_DIST:-30}/")
DARK_INACTIVE=$(printf '%s\n' "$DARK_ACTIVE" | sed \
    -e 's/"terminal-glass-dark"/"terminal-glass-dark-inactive"/' \
    -e 's/roughness .*/roughness 0.5/' \
    -e 's/chromatic-aberration .*/chromatic-aberration 0.08/' \
    -e 's/distortion .*/distortion 0.10 scale=0.03/' \
    -e "s/attenuation-distance .*/attenuation-distance ${DARK_DIST_INACTIVE:-70}/")
INACTIVE_MATERIAL=$(printf '%s\n' "$ACTIVE_MATERIAL" | sed \
    -e 's/"terminal-glass"/"terminal-glass-inactive"/' \
    -e 's/roughness .*/roughness 0.45/' \
    -e 's/chromatic-aberration .*/chromatic-aberration 0.08/' \
    -e 's/distortion .*/distortion 0.10 scale=0.03/' \
    -e 's/attenuation-distance .*/attenuation-distance 300/')

write_config() {   # $1 path, $2 case
    {
        cat <<KDL
layout {
    gaps 54
    default-column-width { proportion 0.5; }
    focus-ring { width 4; active-gradient from="#ffffff00" to="#ccccff11" angle=45 relative-to="workspace-view"; }
    border { off; }
    shadow { on; softness 5; spread 1; offset x=-5 y=5; color "#00000070"; }
}
animations { off; }
hotkey-overlay { skip-at-startup; }
window-rule { match app-id="^kitty$"; geometry-corner-radius 12; clip-to-geometry true; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
$ACTIVE_MATERIAL
$INACTIVE_MATERIAL
$DARK_ACTIVE
$DARK_INACTIVE
KDL
        case $2 in
            baseline|zero-same)
                echo 'window-rule { match app-id="^kitty$"; material "terminal-glass"; }' ;;
            zero-split)
                echo 'window-rule { match app-id="^kitty$" is-active=true; material "terminal-glass"; }'
                echo 'window-rule { match app-id="^kitty$" is-active=false; material "terminal-glass-inactive"; }' ;;
            zero-dark)
                echo 'window-rule { match app-id="^kitty$"; material "terminal-glass-dark"; }' ;;
            zero-dark-split)
                echo 'window-rule { match app-id="^kitty$" is-active=true; material "terminal-glass-dark"; }'
                echo 'window-rule { match app-id="^kitty$" is-active=false; material "terminal-glass-dark-inactive"; }' ;;
            *) echo "FAIL: unknown case $2" >&2; exit 1 ;;
        esac
    } > "$1"
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; exit 1; }
}

start_nested() {
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$HOST" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || { echo "FAIL: no Weston socket" >&2; exit 1; }
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$NIRI" -c "$1" >> "$WORK/niri.log" 2>&1 &
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
spawn_kitty() {   # $1 background opacity, $2 label
    msg action spawn -- kitty --config NONE -o "background_opacity=$1" -o background=#222436 -o foreground=#c8d3f5 \
        -o cursor_blink_interval=0 -o font_size=11 -o window_padding_width=8 --hold "$HERE/focus-glass-content.sh" "$2"
}
shot() {   # $1 label; full frame plus 3x crops of the inner top and bottom corners
    local f=$WORK/$1.png
    msg action screenshot-screen --write-to-disk true --show-pointer false --path "$f"
    for _ in $(seq 50); do [ -s "$f" ] && break; sleep 0.1; done
    [ -s "$f" ] || { echo "FAIL: shot $1" >&2; exit 1; }
    magick "$f" -crop 260x160+510+40 +repage -scale 300% "$WORK/$1-top.png"
    magick "$f" -crop 260x160+510+520 +repage -scale 300% "$WORK/$1-bottom.png"
    sha256sum "$f" >> "$WORK/SHA256SUMS"
}
run_case() {   # $1 case, $2 left opacity, $3 right opacity
    write_config "$WORK/$1.kdl" "$1"
    start_nested "$WORK/$1.kdl"
    spawn_kitty "$2" left;  wait_kitty 1
    spawn_kitty "$3" right; wait_kitty 2
    sleep 2
    shot "$1-right-focused"
    msg action focus-column-left; sleep 2
    shot "$1-left-focused"
    stop_nested
}

sha256sum "$NIRI" > "$WORK/SHA256SUMS"
"$NIRI" --version >> "$WORK/SHA256SUMS"
for c in ${CASES:-baseline zero-same zero-split zero-dark zero-dark-split}; do
    case $c in baseline) run_case baseline 0.44 0.98 ;; *) run_case "$c" 0 0 ;; esac
done
echo "captures: $WORK"
