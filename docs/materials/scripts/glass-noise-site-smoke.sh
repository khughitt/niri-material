#!/usr/bin/env bash
# Noise placement baseline smoke (design §7.2). Default: the full matrix.
# NOISE_SITE_PILOT=1 selects fine grain and the blurred roughness endpoints,
# still exercising all seven assertions. OUT must be fresh; BASE_NIRI pins
# the b261ad1a release binary; CAPTURE_TASK and NIRI_MATERIAL_WORK_ROOT are required.
set -eu
BASE_NIRI=${BASE_NIRI:?baseline release binary}
PILOT=${NOISE_SITE_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "NOISE_SITE_PILOT must be 0 or 1" >&2; exit 2 ;; esac
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap 'exit 130' INT
trap 'exit 143' TERM
capture_preflight headless
build_binaries
cp "$BASE_NIRI" "$OUT/niri-baseline"
BASE_NIRI=$OUT/niri-baseline
capture_identity --binary "$BASE_NIRI" --config "pilot=$PILOT" --config baseline=b261ad1a
magick -size 1280x720 xc:'rgb(140,115,90)' \
    \( -size 400x300 -seed 11 plasma:fractal \) -gravity southeast -composite "$WALL"
RING_WALL=$OUT/ring-mid.png
magick -size 1280x720 xc:'rgb(128,128,128)' "$RING_WALL"
KINDS=(white fine lightness); BLURS=(false true); ROUGHNESS=(0 0.5 1)
if [ "$PILOT" = 1 ]; then KINDS=(fine); BLURS=(true); ROUGHNESS=(0 1); fi
GRAIN_FLOOR=$(awk 'BEGIN { printf "%.8f", 0.5 / 255 }')   # sd, normalized
mae() { compare_metric MAE "$1" "$2"; }
film_ae() {
    local out status
    set +e
    out=$(magick compare -metric AE -fuzz 0.392157% "$1" "$2" null: 2>&1 >/dev/null)
    status=$?
    set -e
    [ "$status" -le 1 ] || fail "film comparison failed: $out"
    out=${out%% *}   # ImageMagick 7 appends the normalized value: "0 (0)"
    is_number "$out" || fail "film comparison is not numeric: $out"
    METRIC=$out
}

IDENTITY=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1'
blur_top() { printf 'blur { passes %s; offset 3; noise 0; saturation 1; }' "$1"; }
cell() {   # $1 name, $2 binary, $3 glass extra, $4 blur passes
    GLASS_EXTRA="$IDENTITY"$'\n'"$3" TOP_EXTRA=$(blur_top "$4") write_config "$OUT/$1.kdl"
    start_nested "$2" "$OUT/$1.kdl"
    spawn_probe "$2" "$IDLE"
    probe_rect "$2"
    sleep 2
    shot_twice "$2" "$1"
    roi "$1" "$(face_roi)" face
    stop_nested
}
calibrate_probe_rect "$NIRI" 0

# The spec's matrix (§7.2): site × blur × kind on the identity fixture.
for kind in "${KINDS[@]}"; do
    cell "omitted-$kind"       "$NIRI"      "noise 0.3 type=\"$kind\""                      3
    cell "baseline-$kind"      "$BASE_NIRI" "noise 0.3 type=\"$kind\""                      3
    for site in glass backdrop film; do
        cell "$site-$kind-off" "$NIRI" "noise 0.3 type=\"$kind\" site=\"$site\""             3
    done
done
cell "zero-off" "$NIRI" "noise 0" 3
for p in 1 3; do
    cell "zero-p$p" "$NIRI" $'backdrop-blur true\nnoise 0' "$p"
    for site in glass backdrop film; do
        cell "$site-fine-p$p" "$NIRI" $'backdrop-blur true\nnoise 0.3 type="fine" site="'"$site"'"' "$p"
    done
done
# Roughness fixture: ior 1.5, so the normalized level is roughness.
ROUGH=$'ior 1.5\nattenuation-color "#ffffff"\nsaturation 1'
rough_cell() {   # $1 name, $2 glass extra, $3 passes
    GLASS_EXTRA="$ROUGH"$'\n'"$2" TOP_EXTRA=$(blur_top "$3") write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"; spawn_probe "$NIRI" "$IDLE"; probe_rect "$NIRI"; sleep 2
    shot_twice "$NIRI" "$1"; roi "$1" "$(face_roi)" face; stop_nested
}
for blur in "${BLURS[@]}"; do for r in "${ROUGHNESS[@]}"; do
    tag="b$blur-r$r"
    rough_cell "rough-zero-$tag" $'backdrop-blur '"$blur"$'\nroughness '"$r"$'\nnoise 0' 3
    for site in glass backdrop; do
        rough_cell "rough-$site-$tag" $'backdrop-blur '"$blur"$'\nroughness '"$r"$'\nnoise 0.3 type="fine" site="'"$site"'"' 3
    done
done; done


for kind in "${KINDS[@]}"; do
    ae "$OUT/omitted-$kind.png" "$OUT/glass-$kind-off.png"
    assert_zero "$kind omitted vs glass" "$METRIC"; metric "${kind}_omitted_vs_glass_ae" "$METRIC"
    ae "$OUT/omitted-$kind.png" "$OUT/baseline-$kind.png"
    assert_zero "$kind omitted vs baseline" "$METRIC"; metric "${kind}_baseline_ae" "$METRIC"
    film_ae "$OUT/film-$kind-off-face.png" "$OUT/glass-$kind-off-face.png"
    assert_zero "$kind film within one code" "$METRIC"; metric "${kind}_film_one_code_ae" "$METRIC"
done
mae "$OUT/glass-fine-off-face.png" "$OUT/backdrop-fine-off-face.png"
metric backdrop_vs_glass_face_mae "$METRIC"
assert_less 'backdrop vs glass face MAE' "$METRIC" "$(magick xc: -format '%[fx:2*quantumrange/255]' info:)"
for site in glass backdrop film; do
    for blur in off p1 p3; do
        signed_diff "$OUT/$site-fine-$blur-face.png" "$OUT/zero-$blur-face.png" "$OUT/grain-$site-$blur.png"
        sd "$OUT/grain-$site-$blur.png"; metric "${site}_${blur}_sd" "$METRIC"
        case $blur in off) base=$METRIC ;; p1) first=$METRIC ;; p3) last=$METRIC ;; esac
    done
    if [ "$site" = backdrop ]; then
        assert_greater 'backdrop off > p1' "$base" "$first"
        assert_greater 'backdrop p1 > p3' "$first" "$last"
    else
        assert_about "$site grain under one blur pass" "$first" "$base" 0.05
        assert_about "$site grain under three blur passes" "$last" "$base" 0.05
    fi
done
for blur in "${BLURS[@]}"; do
    for site in glass backdrop; do
        previous=; reference=
        for r in "${ROUGHNESS[@]}"; do
            tag="b$blur-r$r"
            signed_diff "$OUT/rough-$site-$tag-face.png" "$OUT/rough-zero-$tag-face.png" "$OUT/grain-rough-$site-$tag.png"
            sd "$OUT/grain-rough-$site-$tag.png"; metric "rough_${site}_${tag}_sd" "$METRIC"
            if [ -z "$reference" ]; then reference=$METRIC; fi
            if [ "$site" = glass ]; then
                assert_about "glass grain at $tag" "$METRIC" "$reference" 0.05
            elif [ -n "$previous" ]; then
                # Monotone down to the 8-bit floor: once the grain is below half a
                # code (the in-process presence threshold), the pyramid's upsample
                # may spread a rounded sub-code residue but must not lift it back.
                if awk -v p="$previous" -v f="$GRAIN_FLOOR" 'BEGIN { exit !(p >= f) }'; then
                    assert_greater "backdrop roughness $blur: previous > $r" "$previous" "$METRIC"
                else
                    assert_less "backdrop roughness $blur: stays below the floor at $r" "$METRIC" "$GRAIN_FLOOR"
                fi
            fi
            previous=$METRIC
        done
    done
done

write_ring_config() {   # $1 path, $2 site, $3 amount
    cat > "$1" <<KDL
prefer-no-csd
layout { gaps 40; background-color "transparent"; default-column-width { proportion 0.4; }; focus-ring { off; }; border { off; }; shadow { off; }; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$RING_WALL"
blur { passes 3; offset 3; noise 0; saturation 1; }
material "gos-probe" {
    glass {
        ior 1
        thickness 20
        attenuation-color "#ffffff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        roughness 0
        backdrop-blur false
        jelly-flex 0
        jelly-ripple 0
        bevel 12
        offset-x 6
        offset-y 6
        saturation 1
        noise $3 type="white" site="$2"
    }
    response "default" {
        focus "ring-light"
        accent "none"
        ring-gap 6
        ring-width 3
        ring-color "#ffffff"
        ring-glow 2
        ring-rest 2
        ring-beam-speed 400
        ring-beam-decay 0
    }
}
window-rule {
    match app-id="^gos-probe$"
    material "gos-probe"
    geometry-corner-radius 0
    background-effect { blur false; noise 0; saturation 1; }
}
KDL
}
ring_cell() {   # $1 name, $2 site, $3 amount
    WALL=$RING_WALL
    write_ring_config "$OUT/$1.kdl" "$2" "$3"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    probe_rect "$NIRI"
    # The comet's whole run: a lap (perimeter) plus its tail (25 % of it),
    # at 400 px/s, plus a margin; only the rest glow remains afterwards.
    sleep "$(awk -v w="$PW" -v h="$PH" 'BEGIN { printf "%d", 2*(w+h)*1.25/400 + 3 }')"
    shot_twice "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    # The band: the ring follows the face, the window inset by bevel (12) and
    # shifted by offset-x (6), so its right edge peaks at PX + PW - 6.5
    # (measured in the 2026-10-05 pilot); a 3 px strip centred there, 300 px tall.
    roi "$1" "3x300+$((PX + PW - 12 + 6 - 2))+$((PY + 200))" band
    stop_nested
}
for site in glass film; do
    ring_cell "ring-$site-on"  "$site" 0.1
    ring_cell "ring-$site-off" "$site" 0
done
mean "$OUT/ring-glass-off-band.png"; e_b=$METRIC
awk -v e="$e_b" 'BEGIN { exit !(e >= 0.85 && e <= 0.95) }' \
    || fail "ring band level $e_b outside 0.85..0.95: the band crop or the glow/rest values need adjusting (a fixture fix, not a renderer finding)"
expected_glass=$(awk -v e="$e_b" 'function dec(x) { return x <= 0.04045 ? x/12.92 : ((x+0.055)/1.055)^2.4 }
    BEGIN { d1 = 2.4/1.055 * ((0.5+0.055)/1.055)^1.4; y = dec(e); d2 = 1.055/2.4 * y^(1/2.4 - 1); printf "%.4f", d1*d2 }')
for site in glass film; do
    signed_diff "$OUT/ring-$site-on-face.png" "$OUT/ring-$site-off-face.png" "$OUT/ring-$site-face-grain.png"; sd "$OUT/ring-$site-face-grain.png"; face_sd=$METRIC
    signed_diff "$OUT/ring-$site-on-band.png" "$OUT/ring-$site-off-band.png" "$OUT/ring-$site-band-grain.png"; sd "$OUT/ring-$site-band-grain.png"; band_sd=$METRIC
    ratio=$(awk -v a="$band_sd" -v b="$face_sd" 'BEGIN { printf "%.4f", a/b }')
    printf 'ring_%s_band_over_face=%s\n' "$site" "$ratio" >> "$OUT/metrics.txt"
    case $site in
        film)  assert_about "film grain in the ring band" "$ratio" 1.0 0.1 ;;
        glass) assert_about "glass grain compressed under the ring" "$ratio" "$expected_glass" 0.1 ;;
    esac
done
printf 'ring_band_level=%s\nring_expected_glass_ratio=%s\n' "$e_b" "$expected_glass" >> "$OUT/metrics.txt"

finish
printf "PASS: noise placement (pilot=%s)\n" "$PILOT"
