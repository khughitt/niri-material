#!/usr/bin/env bash
# Noise layers smoke (design 2026-10-06-noise-layers-design.md §7.1, §7.2).
# Default: the full matrix and the contact sheet. NOISE_LAYERS_PILOT=1 keeps
# fine grain and scale 8, still exercising all six assertions and the sheet.
# OUT must be fresh; BASE_NIRI pins the 37de154e release binary (the noise
# merge's first parent: 4a8b2072 plus what merged beside it, glass edges included);
# CAPTURE_TASK and NIRI_MATERIAL_WORK_ROOT are required.
set -eu
BASE_NIRI=${BASE_NIRI:?baseline release binary}
PILOT=${NOISE_LAYERS_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_LAYERS_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap 'exit 130' INT
trap 'exit 143' TERM
build_binaries
cp "$BASE_NIRI" "$OUT/niri-baseline"
BASE_NIRI=$OUT/niri-baseline
capture_preflight pixels
capture_identity --binary "$BASE_NIRI" --config "pilot=$PILOT" --config baseline=37de154e
magick -size 1280x720 xc:'rgb(140,115,90)' \
    \( -size 400x300 -seed 11 plasma:fractal \) -gravity southeast -composite "$WALL"
KINDS=(white fine lightness); SITES=(glass backdrop film); SCALE_KINDS=(white fine); SCALES=(2 4 8)
if [ "$PILOT" = 1 ]; then KINDS=(fine); SITES=(glass backdrop film); SCALE_KINDS=(fine); SCALES=(8); fi
BINS=$(dirname "$0")/noise-layers-bins.py
IDENTITY=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1'
TOP_DEFAULT='blur { passes 3; offset 3; noise 0; saturation 1; }'
# The window's face less 60 px a side and 120 px top and bottom: the lattice
# classes need far more samples than the lib's 200x400 face.
wide_roi() { echo "$((PW - 120))x$((PH - 240))+$((PX + 60))+$((PY + 120))"; }
cell() {   # $1 name, $2 binary, $3 glass extra, $4 top extra (optional)
    GLASS_EXTRA="$IDENTITY"$'\n'"$3" TOP_EXTRA=${4:-$TOP_DEFAULT} write_config "$OUT/$1.kdl"
    start_nested "$2" "$OUT/$1.kdl"
    spawn_probe "$2" "$IDLE"
    probe_rect "$2"
    sleep 2
    shot_twice "$2" "$1"
    roi "$1" "$(wide_roi)" face
    stop_nested
}
grain_sd() {   # $1 cell, $2 reference cell; sets METRIC
    signed_diff "$OUT/$1-face.png" "$OUT/$2-face.png" "$OUT/grain-$1.png"
    sd "$OUT/grain-$1.png"
}
# Sets BIN_aggregate_sd, BIN_min_bin_ratio, BIN_max_bin_ratio, BIN_lf_ratio,
# clearing them first so a failed run can never leave the previous cell's.
# identify prints no trailing newline, so its output is captured and checked,
# never piped into `read` (which returns 1 at EOF and trips set -e).
bins() {   # $1 cell (its grain image must exist), $2 scale
    local png=$OUT/grain-$1.png raw=$OUT/grain-$1.raw dims w h out key value name
    unset BIN_aggregate_sd BIN_min_bin_ratio BIN_max_bin_ratio BIN_lf_ratio
    magick "$png" -depth 16 -endian MSB "gray:$raw" || fail "raw export of $1 failed"
    dims=$(magick identify -format '%w %h' "$png") || fail "identify of $1 failed"
    [[ $dims =~ ^([0-9]+)\ ([0-9]+)$ ]] || fail "identify of $1 returned '$dims'"
    w=${BASH_REMATCH[1]}; h=${BASH_REMATCH[2]}
    out=$(python3 -I "$BINS" "$raw" "$w" "$h" "$2") || fail "position classes of $1 failed"
    while IFS='=' read -r key value; do
        case $key in
            aggregate_sd|min_bin_ratio|max_bin_ratio|lf_ratio) ;;
            *) fail "position classes of $1: unexpected line '$key=$value'" ;;
        esac
        is_number "$value" || fail "position classes of $1: $key is '$value'"
        printf -v "BIN_$key" '%s' "$value"
    done <<< "$out"
    for name in BIN_aggregate_sd BIN_min_bin_ratio BIN_max_bin_ratio BIN_lf_ratio; do
        [ -n "${!name:-}" ] || fail "position classes of $1: no ${name#BIN_}"
    done
}
calibrate_probe_rect "$NIRI" 0

cell zero "$NIRI" 'noise 0'
# 2: byte identity against the baseline, one node per kind and site.
for kind in "${KINDS[@]}"; do for site in "${SITES[@]}"; do
    cell "one-$kind-$site"  "$NIRI"      "noise 0.3 type=\"$kind\" site=\"$site\""
    cell "base-$kind-$site" "$BASE_NIRI" "noise 0.3 type=\"$kind\" site=\"$site\""
    ae "$OUT/one-$kind-$site.png" "$OUT/base-$kind-$site.png"
    assert_zero "$kind $site vs baseline" "$METRIC"; metric "${kind}_${site}_baseline_ae" "$METRIC"
done; done
INHERIT_TOP='blur { passes 3; offset 3; noise 0.05; saturation 1; }'
cell inherit      "$NIRI"      'backdrop-blur true' "$INHERIT_TOP"
cell base-inherit "$BASE_NIRI" 'backdrop-blur true' "$INHERIT_TOP"
ae "$OUT/inherit.png" "$OUT/base-inherit.png"
assert_zero "inherited vs baseline" "$METRIC"; metric inherit_baseline_ae "$METRIC"
# 3: scale=1 written equals omitted.
for kind in "${KINDS[@]}"; do
    cell "scale1-$kind" "$NIRI" "noise 0.3 type=\"$kind\" site=\"glass\" scale=1"
    ae "$OUT/scale1-$kind.png" "$OUT/one-$kind-glass.png"
    assert_zero "$kind scale=1 vs omitted" "$METRIC"; metric "${kind}_scale1_ae" "$METRIC"
done
# 4: two independent fine layers add in quadrature.
cell quad-one "$NIRI" 'noise 0.2 type="fine"'
cell quad-two "$NIRI" $'noise 0.2 type="fine"\nnoise 0.2 type="fine"'
grain_sd quad-one zero; one=$METRIC; grain_sd quad-two zero; two=$METRIC
ratio=$(awk -v a="$two" -v b="$one" 'BEGIN { printf "%.4f", a/b }')
metric quadrature_ratio "$ratio"
assert_about "two fine layers over one" "$ratio" 1.41421356 0.05
# 5: deviation by scale and by position in the cell; low frequency rises.
for kind in "${SCALE_KINDS[@]}"; do
    cell "s1-$kind" "$NIRI" "noise 0.3 type=\"$kind\""
    grain_sd "s1-$kind" zero; base=$METRIC; metric "${kind}_s1_sd" "$base"
    bins "s1-$kind" 1; previous=$BIN_lf_ratio; metric "${kind}_s1_lf" "$previous"
    for s in "${SCALES[@]}"; do
        cell "s$s-$kind" "$NIRI" "noise 0.3 type=\"$kind\" scale=$s"
        grain_sd "s$s-$kind" zero; metric "${kind}_s${s}_sd" "$METRIC"
        assert_about "$kind scale $s sd" "$METRIC" "$base" 0.10
        bins "s$s-$kind" "$s"
        metric "${kind}_s${s}_min_bin" "$BIN_min_bin_ratio"; metric "${kind}_s${s}_max_bin" "$BIN_max_bin_ratio"
        metric "${kind}_s${s}_lf" "$BIN_lf_ratio"
        assert_about "$kind scale $s weakest position" "$BIN_min_bin_ratio" 1 0.10
        assert_about "$kind scale $s strongest position" "$BIN_max_bin_ratio" 1 0.10
        assert_greater "$kind scale $s low frequency rises" "$BIN_lf_ratio" "$previous"
        previous=$BIN_lf_ratio
    done
done
# 6: every slot applied and seeded apart.
cell slot0 "$NIRI" 'noise 0.3 type="fine"'
grain_sd slot0 zero; s0=$METRIC
for k in 1 2 3; do
    lines=; for _ in $(seq "$k"); do lines+=$'noise 0\n'; done
    cell "slot$k" "$NIRI" "${lines}noise 0.3 type=\"fine\""
    grain_sd "slot$k" zero; metric "slot${k}_sd" "$METRIC"
    assert_about "slot $k sd" "$METRIC" "$s0" 0.05
    signed_diff "$OUT/slot$k-face.png" "$OUT/slot0-face.png" "$OUT/slot$k-vs-0.png"
    sd "$OUT/slot$k-vs-0.png"; metric "slot${k}_vs_slot0_sd" "$METRIC"
    assert_greater "slot $k differs from slot 0" "$METRIC" "$s0"
done
metric slot0_sd "$s0"

# 7.1: the contact sheet.
SHEET=(s1-fine s8-fine)
sheet_cell() { cell "$1" "$NIRI" "$2"; SHEET+=("$1"); }
if [ "$PILOT" = 0 ]; then
    SHEET=(s1-fine s2-fine s4-fine s8-fine s1-white s2-white s4-white s8-white)
    sheet_cell l1 'noise 0.3 type="lightness"'
    sheet_cell l4 'noise 0.3 type="lightness" scale=4'
fi
sheet_cell stack-white4-fine1 $'noise 0.3 type="white" scale=4\nnoise 0.3 type="fine"'
sheet_cell stack-lightness-film $'noise 0.3 type="lightness"\nnoise 0.3 type="white" site="film"'
sheet_cell four-fine-015 $'noise 0.15 type="fine"\nnoise 0.15 type="fine"\nnoise 0.15 type="fine"\nnoise 0.15 type="fine"'
sheet_cell one-fine-03 'noise 0.3 type="fine"'
args=()
for name in "${SHEET[@]}"; do args+=(-label "$name" "$OUT/$name-face.png"); done
magick montage "${args[@]}" -tile 4x -geometry 360x360+6+6 -background '#202020' -fill white "$OUT/contact-sheet.png" \
    || fail "contact sheet failed"

finish
printf "PASS: noise layers (pilot=%s)\n" "$PILOT"
