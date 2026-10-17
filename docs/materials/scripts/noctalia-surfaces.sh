#!/usr/bin/env bash
# Noctalia surfaces in a nested niri (material-e27dbc): a surface inventory and
# a panel comparison across appearance cases, without the live desktop.
#
# Each case runs a headless Weston, a nested niri and an isolated Noctalia:
# its own config, state, data and cache directories and a private D-Bus
# session, so it cannot run the live hooks, templates or idle actions, or take
# the live notification and tray names. Its lock and IPC socket live in the
# run's own runtime dir. A terminal under the pinned window glass sits beside
# the panels. With the bar mapped, the case records `niri msg layers` and a
# screenshot, then the same with the control center (an attached panel) and
# the launcher (a floating panel) open.
#
# Cases:
#   baseline  the live appearance on 2026-10-15: bar opacity 0.70 (attached
#             panels inherit it), floating panels in "glass" mode (0.55), the
#             live layer rule (real blur, no xray).
#   treated   existing effects and Noctalia styling matched to the window
#             glass: bar and attached panels at 0.55 like the floating ones,
#             panel corners scaled to the windows' 20 px, and the glass's
#             noise (0.03) and saturation (0.95) on the shell's blur.
#   matched   treated, plus the glass's hue and darkness (material-b156e7): a
#             custom palette that keeps the wallpaper palette's accents and
#             text but moves the surface family to the glass's rendered color
#             (noctalia-glass-palette.py), with the bar and attached panels at
#             0.95 and floating panels solid, since the glass passes only a few
#             percent of its backdrop.
#   remap      the matched look from niri instead of a palette: Noctalia on its
#              wallpaper palette, bar and attached panels 0.95, floating panels
#              solid, and a surface-remap rule moving Noctalia's surface family
#              to the pinned glass's face (no hue).
#   hue-quiet  remap, with the probe terminal carrying the accent $ACCENT and
#              hue-chroma 0.015.
#   hue-strong remap with hue-chroma 0.03.
# STRIP=1 adds a transition strip: two accented probes on two workspaces and a
# slowed workspace switch, sampled across the crossfade.
# The baseline, treated and matched cases put no niri material on a Noctalia surface: niri has no layer
# material pass. Every difference is Noctalia styling or niri's layer blur.
#
# Usage: OUT=<dir> WALL=<image> [NIRI=niri] [CASES="matched remap hue-quiet hue-strong"] [ACCENT=#e5533c] [ACCENT2=#3cc6e5] [STRIP=1] noctalia-surfaces.sh
# WALL should be the live wallpaper (`noctalia msg wallpaper-get`), so the
# palette Noctalia derives from it matches the desktop's.
# Requires: weston, niri, noctalia, kitty, dbus-run-session, jq, python3,
# ImageMagick 7.
set -euo pipefail

NIRI=${NIRI:-niri}
SCRIPTS=$(dirname "$(realpath "$0")")
CASES=${CASES:-matched remap hue-quiet hue-strong}
ACCENT=${ACCENT:-#e5533c}
ACCENT2=${ACCENT2:-#3cc6e5}
STRIP=${STRIP:-0}
WALL=${WALL:-/usr/share/noctalia/assets/noctalia-wallpaper.png}
: "${OUT:?set OUT to an empty output directory}"
[ -r "$WALL" ] || { echo "FAIL: WALL $WALL is not readable" >&2; exit 1; }
mkdir -p "$OUT"
OUT=$(realpath "$OUT")

# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/nsf.XXXXXX")
HOST=$(basename "$RT")-host
WESTON_PID=; NIRI_PID=; NOCT_PID=; DBUS_PID=; NIRI_SOCKET=; NESTED_DISPLAY=; N=

fail() { echo "FAIL: $*" >&2; exit 1; }
stop_pid() { if [ -n "$1" ]; then kill "$1" 2>/dev/null || true; wait "$1" 2>/dev/null || true; fi; }
stop_case() {
    # Noctalia is dbus-run-session's child, not ours: signal it, give it five
    # seconds, then kill it, so the session bus and its runner exit after it.
    if [ -n "$NOCT_PID" ]; then
        kill "$NOCT_PID" 2>/dev/null || true
        for _ in $(seq 50); do kill -0 "$NOCT_PID" 2>/dev/null || break; sleep 0.1; done
        kill -KILL "$NOCT_PID" 2>/dev/null || true
        NOCT_PID=
    fi
    stop_pid "$DBUS_PID"; DBUS_PID=
    # Stopping niri ends the probe terminal it spawned.
    stop_pid "$NIRI_PID"; NIRI_PID=
    stop_pid "$WESTON_PID"; WESTON_PID=
    rm -f "$RT"/wayland-* "$RT"/niri.*.sock "$RT"/noctalia-* "$RT/$HOST"
}
cleanup() {
    local rc=$?
    stop_case
    rm -rf "$RT"
    exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

nmsg() { NIRI_SOCKET=$NIRI_SOCKET "$NIRI" msg "$@"; }
noctalia_env() {
    env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$NESTED_DISPLAY" NIRI_SOCKET="$NIRI_SOCKET" \
        XDG_CURRENT_DESKTOP=niri NOCTALIA_CONFIG_HOME="$N/config" NOCTALIA_STATE_HOME="$N/state" \
        NOCTALIA_DATA_HOME="$N/data" XDG_CACHE_HOME="$N/cache" "$@"
}

# The live terminal glass (Prism's generated "terminal-glass", 2026-10-15),
# pinned so day-to-day Prism changes cannot move the comparison.
GLASS='material "nsf-glass" {
    glass {
        ior 1.28
        light-ior 2
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 12
        chromatic-aberration 0.36
        distortion 0 scale=0.09
        anisotropic-blur 0
        roughness 0.24
        reflection 0.6
        edge-highlight 0
        iridescence 0
        noise 0.03 type="white"
        saturation 0.95
        backdrop-blur true
        jelly-flex 0.0156
        jelly-ripple 0.085
        bevel 12
        bevel-profile 2
        offset-x -6
        offset-y -5
    }
    response "default" {
        accent "ring"
        accent-tint 1
        focus "ring-light"
        attention "rim-orbit"
        ring-color "#5656B5"
        ring-beam-speed 0
        ring-gap 1
        ring-width 0.9
        ring-glow 0.3
        ring-rest 1
        ring-accent 0.4
    }
}
window-rule {
    match app-id="^nsf-probe$"
    material "nsf-glass"
    geometry-corner-radius 20
    clip-to-geometry true
    background-effect { blur false; noise 0; saturation 1; }
}'

write_niri_config() {   # $1 case, $2 path
    local effect remap='' anims='animations { off; }'
    case "$1" in
        baseline) effect='xray false;' ;;
        treated|matched) effect='xray false; noise 0.03; saturation 0.95;' ;;
        remap) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0) ;;
        hue-quiet) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.015) ;;
        hue-strong) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.03) ;;
        strip)
            effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.03)
            anims='animations { slowdown 8; }'
            ;;
        *) fail "unknown case $1" ;;
    esac
    cat > "$2" <<KDL
prefer-no-csd
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
$anims
layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.5; }
    focus-ring { off; }
    border { off; }
}
blur { passes 3; offset 2.0; noise 0.1; saturation 1.0; }
layer-rule {
    match namespace="^noctalia-wallpaper"
    place-within-backdrop true
}
layer-rule {
    match namespace=r#"^noctalia-(bar-[^"]+|notification|dock|panel|attached-panel|osd)$"#
    background-effect { $effect }
}
$remap
$GLASS
KDL
}

# The pinned glass's rendered color over the live wallpaper, measured from the
# probe's interior in the nested run (mean 9,10,10).
GLASS_SURFACE='#090a0a'

# Noctalia's surface family for WALL, the colors Prism will write live.
family_colors() {
    local tokens="$OUT/family.tokens.json"
    if [ ! -s "$tokens" ]; then
        noctalia theme "$WALL" --scheme m3-content --dark -o "$tokens" \
            > "$OUT/family.theme.log" 2>&1 || fail "noctalia theme failed (see $OUT/family.theme.log)"
    fi
    FAM_SURFACE=$(jq -r '(.dark // .).surface' "$tokens")
    FAM_CONTAINER=$(jq -r '(.dark // .).surface_container' "$tokens")
    FAM_OUTLINE=$(jq -r '(.dark // .).outline_variant' "$tokens")
}
remap_rule() {   # $1 hue-chroma
    family_colors
    cat <<KDL
layer-rule {
    match namespace=r#"^noctalia-(bar-[^"]+|attached-panel|panel)$"#
    surface-remap {
        surface "$FAM_SURFACE"
        container "$FAM_CONTAINER"
        outline "$FAM_OUTLINE"
        material "nsf-glass"
        hue-chroma $1
    }
}
KDL
}

write_noctalia_config() {   # $1 case, $2 dir
    local bar_opacity corner_scale panel_mode=glass theme
    theme='source = "wallpaper"
wallpaper_scheme = "m3-content"'
    case "$1" in
        baseline) bar_opacity=0.70; corner_scale=1.0 ;;
        # 12 px x 1.05 UI scale x 1.6 = 20 px, the windows' corner radius.
        treated) bar_opacity=0.55; corner_scale=1.6 ;;
        matched)
            bar_opacity=0.95; corner_scale=1.6; panel_mode=solid
            theme='source = "custom"
custom_palette = "glass-matched"'
            mkdir -p "$2/noctalia/palettes"
            noctalia theme "$WALL" --scheme m3-content --dark -o "$OUT/$1.tokens.json" \
                > "$OUT/$1.theme.log" 2>&1 || fail "noctalia theme failed (see $OUT/$1.theme.log)"
            python3 -I "$SCRIPTS/noctalia-glass-palette.py" "$OUT/$1.tokens.json" "$GLASS_SURFACE" \
                "$2/noctalia/palettes/glass-matched.json" > "$OUT/$1.palette.log"
            ;;
        remap|hue-quiet|hue-strong|strip) bar_opacity=0.95; corner_scale=1.6; panel_mode=solid ;;
        *) fail "unknown case $1" ;;
    esac
    mkdir -p "$2/noctalia"
    cat > "$2/noctalia/config.toml" <<TOML
[accessibility]
ui_scale = 1.05

[shell]
setup_wizard_enabled = false
corner_radius_scale = $corner_scale

[shell.panel]
transparency_mode = "$panel_mode"

[bar.default]
background_opacity = $bar_opacity

[theme]
mode = "dark"
$theme

[wallpaper.default]
path = "$WALL"
TOML
}

wait_for() {   # $1 seconds, $2… command that succeeds when ready
    local deadline=$((SECONDS + $1)); shift
    until "$@" >/dev/null 2>&1; do
        [ "$SECONDS" -lt "$deadline" ] || return 1
        sleep 0.2
    done
}
has_layer() { nmsg -j layers | jq -e --arg ns "$1" 'any(.[]; .namespace == $ns)'; }
has_probe() { nmsg -j windows | jq -e 'any(.[]; .app_id == "nsf-probe")'; }
probe_ids() { nmsg -j windows | jq -r '[.[] | select(.app_id == "nsf-probe") | .id] | sort | .[]'; }
accent_window() {   # $1 window id, $2 accent
    nmsg set-window-signal --id "$1" --source familiar --accent "$2"
}
shot() {   # $1 png path
    rm -f "$1"
    nmsg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"
    wait_for 10 test -s "$1" || fail "no screenshot $1"
}
record() {   # $1 case, $2 step
    nmsg -j layers > "$OUT/$1.$2.layers.json"
    shot "$OUT/$1.$2.png"
}
open_panel() {   # $1 case, $2 panel id, $3 namespace
    noctalia_env noctalia msg panel-open "$2"
    wait_for 10 has_layer "$3" || fail "$2 did not map"
    sleep 1.5
    record "$1" "$2"
    noctalia_env noctalia msg panel-close
    sleep 1
}

start_stack() {   # $1 case; $N is set and both configs written
    local c=$1
    weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
        --width=1280 --height=720 --socket="$HOST" > "$OUT/$c.weston.log" 2>&1 &
    WESTON_PID=$!
    wait_for 10 test -S "$XDG_RUNTIME_DIR/$HOST" || fail "no Weston socket (see $OUT/$c.weston.log)"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"

    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$NIRI" -c "$OUT/$c.niri.kdl" > "$OUT/$c.niri.log" 2>&1 &
    NIRI_PID=$!
    wait_for 10 sh -c "ls '$RT'/niri.*.sock" || fail "no niri socket (see $OUT/$c.niri.log)"
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1)
    NESTED_DISPLAY=$(cd "$RT" && ls wayland-? | head -1)
    [ -n "$NESTED_DISPLAY" ] || fail "no nested Wayland socket in $RT"

    # dbus-run-session gives Noctalia a private session bus; the inner shell
    # records Noctalia's own pid before exec so cleanup can stop it first.
    noctalia_env dbus-run-session -- sh -c 'echo $$ > "$1"; exec noctalia' sh "$N/noctalia.pid" \
        > "$OUT/$c.noctalia.log" 2>&1 &
    DBUS_PID=$!
    wait_for 10 test -s "$N/noctalia.pid" || fail "Noctalia did not start (see $OUT/$c.noctalia.log)"
    NOCT_PID=$(cat "$N/noctalia.pid")
    wait_for 30 has_layer noctalia-bar-default || fail "no bar layer (see $OUT/$c.noctalia.log)"
}
spawn_probe() {
    nmsg action spawn -- kitty --config NONE --class nsf-probe -o background_opacity=0 \
        -o cursor_blink_interval=0 -o font_size=11 \
        sh -c 'printf "~/d/niri-material  materials-26.04\n\n  src/layer/mapped.rs\n  src/niri.rs\n  docs/notes/\n"; exec sleep infinity'
}

run_case() {   # $1 case
    local c=$1
    family_colors
    N=$(mktemp -d "$OUT/$c.noctalia.XXXX")
    write_niri_config "$c" "$OUT/$c.niri.kdl"
    write_noctalia_config "$c" "$N/config"
    noctalia_env noctalia config validate > "$OUT/$c.noctalia-validate.log" 2>&1 \
        || fail "Noctalia rejected the $c config (see $OUT/$c.noctalia-validate.log)"
    start_stack "$c"

    spawn_probe
    wait_for 10 has_probe || fail "no probe terminal"
    case $c in
        hue-quiet|hue-strong)
            accent_window "$(probe_ids | head -1)" "$ACCENT"
            ;;
    esac
    # Let the palette derive from the wallpaper and the wallpaper fade in.
    sleep 3
    record "$c" bar

    open_panel "$c" control-center noctalia-attached-panel
    open_panel "$c" launcher noctalia-panel

    stop_case
    echo "case $c: done"
}

run_strip() {
    local c=strip i
    family_colors
    N=$(mktemp -d "$OUT/$c.noctalia.XXXX")
    write_niri_config "$c" "$OUT/$c.niri.kdl"
    write_noctalia_config "$c" "$N/config"
    start_stack "$c"

    spawn_probe
    wait_for 10 has_probe || fail "no first probe"
    accent_window "$(probe_ids | head -1)" "$ACCENT"
    sleep 6   # dwell, then the slowed fade
    nmsg action focus-workspace-down
    spawn_probe
    wait_for 10 sh -c "[ \$(NIRI_SOCKET='$NIRI_SOCKET' '$NIRI' msg -j windows | jq '[.[] | select(.app_id == \"nsf-probe\")] | length') -ge 2 ]" \
        || fail "no second probe"
    accent_window "$(probe_ids | tail -1)" "$ACCENT2"
    sleep 6
    nmsg action focus-workspace-up
    for i in $(seq 0 11); do shot "$OUT/strip.$i.png"; sleep 0.2; done

    local frames=()
    for i in $(seq 0 11); do
        magick "$OUT/strip.$i.png" -crop 1280x110+0+0 +repage -gravity northwest \
            -fill '#00000099' -draw 'rectangle 0,0 60,34' -fill '#ffffff' \
            -font DejaVu-Sans-Bold -pointsize 22 -annotate +8+5 "F$i" "$OUT/strip.crop.$i.png"
        frames+=("$OUT/strip.crop.$i.png")
    done
    label "Does the bar crossfade from cyan to red without a snap? Frames F0-F11 in order, about 0.3 s apart, of a workspace switch slowed 8x: workspace 2's agent ($ACCENT2) to workspace 1's agent ($ACCENT), hue-chroma 0.03. Top 110 px of the nested output." \
        1280 20 DejaVu-Sans "$OUT/strip.title.png"
    magick "$OUT/strip.title.png" "${frames[@]}" -background '#1b1b1b' -append +repage -depth 8 \
        -define png:compression-level=9 "$OUT/noctalia-hue-transition.png"
    stop_case
    echo "strip: $OUT/noctalia-hue-transition.png"
}

# The owner's sheet: columns are the cases (A the live baseline), rows the
# surfaces, each cell labelled with its id so a pick can be named back.
label() {   # $1 text, $2 width, $3 point size, $4 font, $5 out
    magick -background '#1b1b1b' -fill '#eeeeee' -font "$4" -pointsize "$3" \
        -size "$2"x caption:"$1" -bordercolor '#1b1b1b' -border 12 "$5"
}
case_note() {   # $1 case
    case "$1" in
        baseline) echo "baseline (control). Live settings: bar and attached panels 0.70 opaque, floating panels 0.55, wallpaper palette, panel corners 12.6 px, shell blur noise 0.1, saturation 1.0." ;;
        treated) echo "treated. All panels 0.55 opaque, wallpaper palette, panel corners 20 px like the windows, shell blur noise 0.03 and saturation 0.95 like the glass." ;;
        remap) echo "remap. No palette change: Noctalia's wallpaper palette, niri's surface-remap to the glass face, hue-chroma 0; bar and attached panels 0.95, floating panels solid. Should read as A." ;;
        hue-quiet) echo "hue, quiet. B, plus the probe's accent $ACCENT held for the workspace: hue-chroma 0.015. Is the hue present but quiet?" ;;
        hue-strong) echo "hue, strong. As C at hue-chroma 0.03. Which of C and D reads as the project, legibly?" ;;
        matched) echo "matched. B, plus the glass's hue and darkness: surfaces $GLASS_SURFACE (cards and outlines keep their steps above it), accents unchanged; bar and attached panels 0.95 opaque, floating panels solid." ;;
    esac
}
make_sheet() {
    local s="$OUT/sheet" cw col row i ids=(A B C D) cells=() rows=() heads=() width
    local title look scale=60
    if [[ " ${CASE_LIST[*]} " == *" remap "* ]]; then
        title="Does the niri remap read as the approved look, and which hue strength reads as the project?"
        look="Column A is the approved look (control): Noctalia's custom glass palette. B is niri's surface remap over Noctalia's own wallpaper palette, with no hue. C and D give the probe terminal the accent $ACCENT and remap the bar and panels toward its hue at two strengths. Nested niri, 1280x720, live wallpaper, cells at 50%."
        scale=50
    else
        title="Do the bar and panels read as the same dark glass as the terminal?"
        look="No column puts a niri material on the bar or panels: niri has none for layer surfaces yet. Every difference is Noctalia styling or niri's blur. Compare each column's bar and panels with the glass terminal on the left. Nested niri, 1280x720, live wallpaper, cells at 60%."
    fi
    [ -z "${SCALE:-}" ] || scale=$SCALE
    cw=$((1280 * scale / 100))
    mkdir -p "$s"
    width=$(( ${#CASE_LIST[@]} * (cw + 20) - 24 ))
    label "$title" \
        "$width" 34 DejaVu-Sans-Bold "$s/title.png"
    label "$look" \
        "$width" 22 DejaVu-Sans "$s/look.png"
    for i in "${!CASE_LIST[@]}"; do
        label "${ids[$i]}: $(case_note "${CASE_LIST[$i]}")" "$cw" 20 DejaVu-Sans "$s/col-$i.png"
        heads+=("$s/col-$i.png")
    done
    magick "${heads[@]}" -background '#1b1b1b' -gravity north +append "$s/cols.png"
    row=0
    for step in control-center launcher bar; do
        row=$((row + 1)); cells=()
        for i in "${!CASE_LIST[@]}"; do
            col=${ids[$i]}
            magick "$OUT/${CASE_LIST[$i]}.$step.png" -resize ${scale}% -gravity northwest \
                -fill '#00000099' -draw 'rectangle 0,0 64,40' -fill '#ffffff' \
                -font DejaVu-Sans-Bold -pointsize 28 -annotate +9+5 "$col$row" \
                -bordercolor '#1b1b1b' -border 10 "$s/cell-$col$row.png"
            cells+=("$s/cell-$col$row.png")
        done
        case $step in
            control-center) title="Row $row: control center (attached panel, noctalia-attached-panel)" ;;
            launcher) title="Row $row: launcher (floating panel, noctalia-panel)" ;;
            bar) title="Row $row: bar alone (noctalia-bar-default)" ;;
        esac
        label "$title" "$width" 24 DejaVu-Sans-Bold "$s/row$row.png"
        magick "${cells[@]}" -background '#1b1b1b' +append "$s/cells$row.png"
        magick "$s/row$row.png" "$s/cells$row.png" -background '#1b1b1b' -append "$s/block$row.png"
        rows+=("$s/block$row.png")
    done
    magick "$s/title.png" "$s/look.png" "$s/cols.png" "${rows[@]}" -background '#1b1b1b' \
        -gravity west -append +repage -depth 8 -define png:compression-level=9 \
        "$OUT/noctalia-cohesion-sheet.png"
    echo "sheet: $OUT/noctalia-cohesion-sheet.png"
}

read -r -a CASE_LIST <<< "$CASES"
for c in "${CASE_LIST[@]}"; do run_case "$c"; done
[ "${#CASE_LIST[@]}" -lt 2 ] || make_sheet
[ "$STRIP" = 0 ] || run_strip
echo "OUT=$OUT"
