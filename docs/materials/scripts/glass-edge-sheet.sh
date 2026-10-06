#!/usr/bin/env bash
# Glass edge contact sheet (material-be611b, spec §6 "Visual judgement"): the
# probe's top-left corner over a real wallpaper, for bevel-profile {1, 2, 4}
# crossed with three looks, each at reflection {0, 0.6} x edge-highlight
# {0, 0.5} x roughness {0, 1}, plus a planar facet row at R / bevel = 0.414.
# The owner judges the look and Prism's starting values from it.
#
# Env: OUT (fresh artifact dir), NIRI_MATERIAL_WORK_ROOT, CAPTURE_TASK (task id
# authorizing this run), SHEET_WALL (wallpaper image; the owner's current one),
# SHEET_PILOT=1 for one lit cell (focused, k 2, reflection 0.6, edge-highlight
# 0.5, roughness 0) to check the crop before the full run.
set -eu
: "${CAPTURE_TASK:?task id authorizing this run}"
: "${SHEET_WALL:?wallpaper image to put behind the glass}"
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
[ -r "$SHEET_WALL" ] || fail "SHEET_WALL is not a readable file: $SHEET_WALL"
capture_preflight headless
build_binaries
capture_identity --input "$SHEET_WALL" --config preset=glass-edge-sheet --config output=1280x720 --config scale=1 --config vrr=off
calibrate_probe_rect "$NIRI" 0
WALL=$OUT/wall.png
magick "$SHEET_WALL" -resize 1280x720^ -gravity center -extent 1280x720 "$WALL"

# The looks, pinned. live-*: Prism's terminal glass on 2026-10-02.
declare -A LOOK=(
    [focused]=$'ior 1.28\nthickness 31.2\nattenuation-color "#2e3034"\nattenuation-distance 11\nchromatic-aberration 0.36\nbevel 15\noffset-x -6\noffset-y -5'
    [inactive]=$'ior 1.2\nthickness 62.3\nattenuation-color "#2e3034"\nattenuation-distance 42\nchromatic-aberration 0.32\ndistortion 0.2 scale=0.08\nbevel 15\noffset-x -6\noffset-y -5'
    [weak-thin]=$'ior 1.5\nthickness 6\nattenuation-color "#dfe8ff"\nattenuation-distance 60\nbevel 12\noffset-x 6\noffset-y 6'
)
corner_roi() { echo "160x160+$((PX > 40 ? PX - 40 : 0))+$((PY > 40 ? PY - 40 : 0))"; }

cell() {   # $1 name, $2 glass lines
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    sleep 2
    probe_rect "$NIRI"
    shot "$NIRI" "$1"
    magick "$OUT/$1.png" -crop "$(corner_roi)" +repage "$OUT/cell-$1.png"
    stop_nested
}

ROWS=()
if [ "${SHEET_PILOT:-0}" = 1 ]; then
    LOOKS=(focused); KS=(2); REFLS=(0.6); HLS=(0.5); ROUGHS=(0)
else
    LOOKS=(focused inactive weak-thin); KS=(1 2 4); REFLS=(0 0.6); HLS=(0 0.5); ROUGHS=(0 1)
fi
for look in "${LOOKS[@]}"; do
    for k in "${KS[@]}"; do
        row=()
        for refl in "${REFLS[@]}"; do
            for hl in "${HLS[@]}"; do
                for rough in "${ROUGHS[@]}"; do
                    name="$look-k$k-r$refl-h$hl-g$rough"
                    cell "$name" "${LOOK[$look]}"$'\n'"bevel-profile $k"$'\n'"reflection $refl"$'\n'"edge-highlight $hl"$'\n'"roughness $rough"
                    row+=("$OUT/cell-$name.png")
                done
            done
        done
        ROWS+=("$look k$k:${row[*]}")
    done
done
# The planar flash: R / bevel = tan 22.5 degrees.
row=()
[ "${SHEET_PILOT:-0}" = 1 ] && ROUGHS=() || ROUGHS=(0 1)
for rough in "${ROUGHS[@]}"; do
    for hl in 0 0.5; do
        name="facet-h$hl-g$rough"
        cell "$name" $'ior 1.5\nthickness 4.97\nbevel 12\noffset-x 6\noffset-y 6\nbevel-profile 1'$'\n'"edge-highlight $hl"$'\n'"roughness $rough"
        row+=("$OUT/cell-$name.png")
    done
done
[ ${#row[@]} -gt 0 ] && ROWS+=("facet:${row[*]}")

args=()
for entry in "${ROWS[@]}"; do
    for f in ${entry#*:}; do
        args+=(-label "$(basename "$f" .png | sed 's/^cell-//')" "$f")
    done
done
magick montage "${args[@]}" -tile 8x -geometry +4+4 -pointsize 10 "$OUT/glass-edge-sheet.png"
echo "sheet: $OUT/glass-edge-sheet.png"
finish
