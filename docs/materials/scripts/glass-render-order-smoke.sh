#!/usr/bin/env bash
# Render-order pixel and cost checks. The within phase reuses this lifecycle;
# it adds no parallel Weston/capture framework.
# The existing noise-type, noise/saturation and signal smokes run separately.
set -eu
: "${PHASE:?behind or within}" "${BASE_NIRI:?baseline release binary}"
: "${BASE_NIRI_TRACY:?baseline Tracy binary}" "${CAPTURE_TASK:?task id}"
MODE=${MODE:-verify}
SCOPE=${SCOPE:-all}
case "$PHASE:$MODE" in behind:red|behind:verify|within:verify) ;; *) echo 'unsupported phase/mode' >&2; exit 2 ;; esac
validate_scope() {
    case $SCOPE in pixels) ;; cost|all)
        if [ -n "${CAPTURE_META:-}" ]; then
            echo "FAIL: SCOPE=$SCOPE requires the strict capture-meta tool" >&2
            return 2
        fi
        ;; *) echo "FAIL: SCOPE must be pixels, cost or all" >&2; return 2 ;; esac
}
validate_scope || exit $?
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
capture_preflight headless

select_binaries() {
    if [ "$MODE" = red ]; then
        NIRI=$BASE_NIRI; NIRI_TRACY=$BASE_NIRI_TRACY
    elif [ -n "${CANDIDATE_NIRI:-}${CANDIDATE_NIRI_TRACY:-}${CANDIDATE_BUILD_RECORD:-}" ]; then
        : "${CANDIDATE_NIRI:?retained candidate release binary}"
        : "${CANDIDATE_NIRI_TRACY:?retained candidate Tracy binary}"
        : "${CANDIDATE_BUILD_RECORD:?retained candidate capture.json}"
        [ -x "$CANDIDATE_NIRI" ] || fail "candidate release binary is not executable: $CANDIDATE_NIRI"
        [ -x "$CANDIDATE_NIRI_TRACY" ] || fail "candidate Tracy binary is not executable: $CANDIDATE_NIRI_TRACY"
        [ -f "$CANDIDATE_BUILD_RECORD" ] || fail "candidate build record is missing: $CANDIDATE_BUILD_RECORD"
        NIRI=$CANDIDATE_NIRI; NIRI_TRACY=$CANDIDATE_NIRI_TRACY
    else
        build_binaries
    fi
}
select_binaries
IDENTITY_EXTRA=()
if [ -n "${CANDIDATE_BUILD_RECORD:-}" ]; then
    IDENTITY_EXTRA+=(--input "$CANDIDATE_BUILD_RECORD" \
        --config "candidate-build-run=$(basename "$(dirname "$CANDIDATE_BUILD_RECORD")")")
fi
if [ -n "${CAPTURE_META:-}" ]; then
    IDENTITY_EXTRA+=(--input "$CAPTURE_META" --config gpu-quietness=pixel-only-waiver)
fi
capture_identity --binary "$BASE_NIRI" --binary "$BASE_NIRI_TRACY" \
    --input "$HERE/glass-render-order-metrics.py" \
    --config phase="$PHASE" --config mode="$MODE" --config scope="$SCOPE" \
    --config output=1280x720 --config scale=1 "${IDENTITY_EXTRA[@]}"
case $SCOPE in pixels|all) calibrate_probe_rect "$NIRI" 0 ;; esac
WARM_WALL=$WALL

spawn_client() { # binary, transparent|opaque
    case $2 in
        transparent) spawn_probe "$1" "$IDLE" ;;
        opaque)
            msg "$1" action spawn -- kitty --config NONE --class gos-probe \
                -o background='#222436' -o foreground='#ffffff' -o background_opacity=1 \
                -o cursor_blink_interval=0 sh -c \
                'i=0; while [ "$i" -lt 40 ]; do printf "opaque glyphs: ████████████████████████████████\n"; i=$((i + 1)); done; exec sleep 600'
            for _ in $(seq 100); do [ "$(windows_with "$1" gos-probe)" -ge 1 ] && break; sleep 0.1; done
            [ "$(windows_with "$1" gos-probe)" -eq 1 ] || fail "expected exactly one opaque probe window"
            ;;
        *) fail "unknown client kind $2" ;;
    esac
}

assert_clean_log() {
    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
        fail "material error, fallback or panic in niri.log"
    fi
}

validate_config() { # binary, config
    "$1" validate -c "$2" >/dev/null 2>&1 || { "$1" validate -c "$2"; fail "invalid config: $2"; }
}

capture() { # name, glass, binary, focus response, client kind, rule opacity
    local name=$1 glass=$2 binary=${3:-$NIRI} focus=${4:-none} client=${5:-transparent} opacity=${6:-1.0}
    GLASS_EXTRA=$'offset-x 0\noffset-y 0\n'"$glass"
    write_config "$OUT/$name.kdl"
    if [ "$focus" = ring-light ]; then
        sed -i 's/focus "none"/focus "ring-light"/' "$OUT/$name.kdl"
    fi
    sed -i "/^window-rule {/a\\    opacity $opacity" "$OUT/$name.kdl"
    validate_config "$binary" "$OUT/$name.kdl"
    start_nested "$binary" "$OUT/$name.kdl"
    spawn_client "$binary" "$client"
    sleep 2
    probe_rect "$binary"
    shot "$binary" "$name"
    sleep 0.3
    shot "$binary" "$name-repeat"
    ae "$OUT/$name.png" "$OUT/$name-repeat.png"
    assert_zero "$name repeat capture" "$METRIC"
    stop_nested
    assert_clean_log
}

metric_grain() { # output name, on, off, x, y, width, height
    python3 "$HERE/glass-render-order-metrics.py" grain "$OUT/$2.png" "$OUT/$3.png" \
        --rect "$4" "$5" "$6" "$7" > "$OUT/$1.json"
    jq -r .sd "$OUT/$1.json"
}

midrange_share() { # image name, crop geometry; share with at least one unclipped channel
    local out
    out=$(magick "$OUT/$1.png" -crop "$2" +repage -alpha off \
        -fx '(r>0&&r<1)||(g>0&&g<1)||(b>0&&b<1)?1:0' -format '%[fx:mean]' info:) \
        || fail "midrange pixel check failed on $1"
    is_number "$out" || fail "midrange pixel share of $1 is not numeric: $out"
    METRIC=$out
}

assert_informative_grain() { # name, on, off, x, y, width, height
    local sd share one_code_normalized=0.00392156862745098 # 1 / 255; grain_sd is normalized
    sd=$(metric_grain "$1" "$2" "$3" "$4" "$5" "$6" "$7")
    assert_greater "$1 signed grain SD" "$sd" "$one_code_normalized"
    midrange_share "$2" "$6"x"$7"+"$4"+"$5"; share=$METRIC
    assert_positive "$1 midrange pixel share" "$share"
    printf '%s_sd=%s\n%s_midrange_share=%s\n' "$1" "$sd" "$1" "$share" >> "$OUT/metrics.txt"
}

uniform_rgb_error() { # image name, x, y, width, height, expected r, g, b
    local values rmin rmax gmin gmax bmin bmax
    values=$(magick "$OUT/$1.png" -crop "$4"x"$5"+"$2"+"$3" +repage -alpha off \
        -format '%[fx:minima.r*255] %[fx:maxima.r*255] %[fx:minima.g*255] %[fx:maxima.g*255] %[fx:minima.b*255] %[fx:maxima.b*255]' info:) \
        || fail "uniform RGB check failed on $1"
    read -r rmin rmax gmin gmax bmin bmax <<< "$values"
    for value in $values; do is_number "$value" || fail "uniform RGB extrema of $1 are not numeric: $values"; done
    METRIC=$(awk -v rmin="$rmin" -v rmax="$rmax" -v gmin="$gmin" -v gmax="$gmax" \
        -v bmin="$bmin" -v bmax="$bmax" -v r="$6" -v g="$7" -v b="$8" '
        function abs(x) { return x < 0 ? -x : x }
        BEGIN {
            e=abs(rmin-r); if (abs(rmax-r)>e) e=abs(rmax-r)
            if (abs(gmin-g)>e) e=abs(gmin-g); if (abs(gmax-g)>e) e=abs(gmax-g)
            if (abs(bmin-b)>e) e=abs(bmin-b); if (abs(bmax-b)>e) e=abs(bmax-b)
            print e
        }')
}

assert_uniform_rgb() { # metric name, image, x, y, width, height, expected r, g, b
    uniform_rgb_error "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9"
    awk -v e="$METRIC" 'BEGIN { exit !(e <= 1) }' \
        || fail "$1 expected uniform RGB ($7,$8,$9) within one code, max error $METRIC"
    printf '%s_max_code_error=%s\n' "$1" "$METRIC" >> "$OUT/metrics.txt"
}

retain_failure() {
    (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > SHA256SUMS)
}

additive_case() { # name, light-off glass, light-on glass, light-on focus
    local name=$1 off=$2 on=$3 focus=${4:-none} common x y w h
    for noise in 0 0.5; do
        common="noise $noise type=\"fine\"
saturation 1"
        capture "$name-n$noise-off" "$common
$off" "$NIRI" none
        capture "$name-n$noise-on" "$common
$on" "$NIRI" "$focus"
    done
    case $name in
        ring|iridescence) x=$((PX - 4)); y=$((PY + 120)); w=20; h=400 ;;
        *) x=$((PX + 60)); y=$((PY + 120)); w=200; h=400 ;;
    esac
    local rc=0
    python3 "$HERE/glass-render-order-metrics.py" additive \
        "$OUT/$name-n0.5-on.png" "$OUT/$name-n0.5-off.png" \
        "$OUT/$name-n0-on.png" "$OUT/$name-n0-off.png" \
        --rect "$x" "$y" "$w" "$h" \
        > "$OUT/$name-additive.json" || rc=$?
    cat "$OUT/$name-additive.json"
    printf '%s\n' "$rc" > "$OUT/$name-additive-exit-status"
    if [ "$rc" -ne 0 ]; then retain_failure; return "$rc"; fi
}

neutral_identity() {
    local neutral=$'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }'
    capture neutral-baseline "$neutral" "$BASE_NIRI" none
    capture neutral-candidate "$neutral" "$NIRI" none
    ae "$OUT/neutral-baseline.png" "$OUT/neutral-candidate.png"
    assert_zero "neutral baseline/candidate decoded identity" "$METRIC"
    printf 'neutral_baseline_candidate_ae=%s\n' "$METRIC" >> "$OUT/metrics.txt"
}

grain_preservation() {
    local type common base_sd candidate_sd
    common=$'ior 1\nattenuation-color "#ffffff"\nattenuation-distance 60\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }'
    for type in white fine lightness; do
        capture "grain-$type-base-off" "$common
noise 0 type=\"$type\"" "$BASE_NIRI" none
        capture "grain-$type-base-on" "$common
noise 0.02 type=\"$type\"" "$BASE_NIRI" none
        capture "grain-$type-candidate-off" "$common
noise 0 type=\"$type\"" "$NIRI" none
        capture "grain-$type-candidate-on" "$common
noise 0.02 type=\"$type\"" "$NIRI" none
        base_sd=$(metric_grain "grain-$type-base" "grain-$type-base-on" "grain-$type-base-off" \
            "$((PX + 60))" "$((PY + 120))" 200 400)
        candidate_sd=$(metric_grain "grain-$type-candidate" "grain-$type-candidate-on" \
            "grain-$type-candidate-off" "$((PX + 60))" "$((PY + 120))" 200 400)
        assert_about "$type face grain SD" "$candidate_sd" "$base_sd" 0.1
        printf 'grain_%s_base_sd=%s\ngrain_%s_candidate_sd=%s\n' \
            "$type" "$base_sd" "$type" "$candidate_sd" >> "$OUT/metrics.txt"
    done
}

additive_matrix() {
    additive_case aurora 'aurora 0 { drift-hz 0; }' 'aurora 0.5 { drift-hz 0; }' none
    additive_case ring '' '' ring-light
    additive_case iridescence 'iridescence 0' 'iridescence 0.8' none
}

dense_grain() {
    local type common face_sd bevel_sd
    common=$'ior 1\nthickness 80\nattenuation-color "#222436"\nattenuation-distance 30\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }'
    for type in white fine lightness; do
        capture "dense-$type-off" "$common
noise 0 type=\"$type\"" "$NIRI" none
        capture "dense-$type-on" "$common
noise 0.02 type=\"$type\"" "$NIRI" none
        face_sd=$(metric_grain "dense-$type-face" "dense-$type-on" "dense-$type-off" \
            "$((PX + 60))" "$((PY + 120))" 200 400)
        bevel_sd=$(metric_grain "dense-$type-bevel" "dense-$type-on" "dense-$type-off" \
            "$((PX - 4))" "$((PY + 120))" 20 400)
        printf 'dense_%s_face_sd=%s\ndense_%s_bevel_sd=%s\n' \
            "$type" "$face_sd" "$type" "$bevel_sd" >> "$OUT/metrics.txt"
    done
}

signed_transfer_probes() {
    local type stem common
    magick -size 1280x720 xc:black "$OUT/black.png"
    magick -size 1280x720 xc:'rgb(2,2,2)' "$OUT/near-black.png"
    magick -size 1280x720 xc:'rgb(230,20,70)' "$OUT/saturated.png"
    for WALL in "$OUT/black.png" "$OUT/near-black.png"; do
        stem=$(basename "$WALL" .png)
        common=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }'
        capture "$stem-control" "$common
noise 0" "$NIRI" none
        for type in white fine lightness; do
            capture "$stem-$type" "$common
noise 1 type=\"$type\"" "$NIRI" none
            assert_informative_grain "${stem}_${type}" "$stem-$type" "$stem-control" \
                "$((PX + 60))" "$((PY + 120))" 200 400
        done
    done
    WALL=$OUT/saturated.png
    capture saturated-s3 $'ior 1\nattenuation-color "#ffffff"\nnoise 0\nsaturation 3\niridescence 0\naurora 0 { drift-hz 0; }' "$NIRI" none
    # luma(rgb(230,20,70)) = 68.256; saturation 3 yields
    # (553.488,-76.512,73.488), clamped by the output to (255,0,73/74).
    assert_uniform_rgb saturated_s3 saturated-s3 "$((PX + 60))" "$((PY + 120))" \
        200 400 255 0 73.488
    WALL=$WARM_WALL
}

opaque_bypass() {
    local opacity common
    common=$'ior 1\nattenuation-color "#ffffff"\niridescence 0\naurora 0 { drift-hz 0; }'
    for opacity in 1.0 0.7; do
        capture "opaque-$opacity-off" "$common
noise 0
saturation 1" "$NIRI" none opaque "$opacity"
        capture "opaque-$opacity-on" "$common
noise 0.5 type=\"fine\"
saturation 0" "$NIRI" none opaque "$opacity"
        roi "opaque-$opacity-off" "$(face_roi)" face
        roi "opaque-$opacity-on" "$(face_roi)" face
        ae "$OUT/opaque-$opacity-off-face.png" "$OUT/opaque-$opacity-on-face.png"
        assert_zero "opaque client and glyph identity at opacity $opacity" "$METRIC"
        printf 'opaque_%s_ae=%s\n' "${opacity/./_}" "$METRIC" >> "$OUT/metrics.txt"
    done
}

gpu_case() { # baseline|neutral|active, round
    local binary glass cooldown_name FOCUS_RESPONSE=none RESPONSE_EXTRA=
    if [ "${PHASE:-behind}" = within ]; then
        case $1 in
            baseline) binary=$BASE_NIRI_TRACY; FOCUS_RESPONSE=ring-light; RESPONSE_EXTRA=$'ring-gap 5\nring-width 2.6'; glass=$'noise 0\nsaturation 1\naurora 0.5 { drift-hz 0; }' ;;
            neutral) binary=$NIRI_TRACY; glass=$'noise 0\nsaturation 1\naurora 0 { drift-hz 0; }' ;;
            active) binary=$NIRI_TRACY; FOCUS_RESPONSE=ring-light; RESPONSE_EXTRA=$'ring-gap 5\nring-width 2.6'; glass=$'noise 0\nsaturation 1\naurora 0.5 { drift-hz 0; }' ;;
            *) fail "unknown GPU case $1" ;;
        esac
    else case $1 in
        baseline) binary=$BASE_NIRI_TRACY; glass=$'noise 0.5 type="fine"\nsaturation 0' ;;
        neutral) binary=$NIRI_TRACY; glass=$'noise 0\nsaturation 1' ;;
        active) binary=$NIRI_TRACY; glass=$'noise 0.5 type="fine"\nsaturation 0' ;;
        *) fail "unknown GPU case $1" ;;
    esac; fi
    local NIRI_TRACY=$binary GLASS_EXTRA=$'offset-x 0\noffset-y 0\n'"$glass"
    cooldown_name="gpu-$1-$2"
    printf '%s %s cooldown start (30s)\n' "$(date --iso-8601=seconds)" "$cooldown_name" \
        | tee -a "$OUT/cost-cooldown.log" >&2
    sleep 30
    printf '%s %s cooldown complete\n' "$(date --iso-8601=seconds)" "$cooldown_name" \
        | tee -a "$OUT/cost-cooldown.log" >&2
    trace_run "$cooldown_name" "$TICK" 0
    assert_clean_log
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}

cost_matrix() {
    local c baseline_ns neutral_ns active_ns
    tools_ready; reserve_tracy_port
    for c in baseline neutral active; do gpu_case "$c" 1; done
    for c in neutral active baseline; do gpu_case "$c" 2; done
    for c in active baseline neutral; do gpu_case "$c" 3; done
    baseline_ns=$(median3 "$OUT/gpu-baseline.medians")
    neutral_ns=$(median3 "$OUT/gpu-neutral.medians")
    active_ns=$(median3 "$OUT/gpu-active.medians")
    printf 'gpu_baseline_ms=%s\ngpu_neutral_ms=%s\ngpu_active_ms=%s\n' \
        "$(ns_to_ms "$baseline_ns")" "$(ns_to_ms "$neutral_ns")" "$(ns_to_ms "$active_ns")" >> "$OUT/metrics.txt"
    printf 'gpu_neutral_vs_baseline_pct=%s\ngpu_active_vs_baseline_pct=%s\n' \
        "$(pct_delta "$baseline_ns" "$neutral_ns")" "$(pct_delta "$baseline_ns" "$active_ns")" >> "$OUT/metrics.txt"
}

pixel_matrix() {
    neutral_identity
    grain_preservation
    additive_matrix
    dense_grain
    signed_transfer_probes
    opaque_bypass
}

behind_matrix() {
    pixel_matrix
    cost_matrix
}

reach_case() { # name, on, off, gap, width, thickness, bevel, scatter
    local rc=0
    python3 "$HERE/glass-render-order-metrics.py" reach "$OUT/$2.png" "$OUT/$3.png" \
        --window "$PX" "$PY" "$PW" "$PH" --bevel "$7" --thickness "$6" \
        --gap "$4" --width "$5" --scatter "$8" > "$OUT/$1-reach.json" || rc=$?
    cat "$OUT/$1-reach.json"
    [ "$rc" -eq 0 ] || fail "$1 exceeded its derived rest reach bound"
}

profile_reach() { # name, on, off, gap, width, thickness, bevel, scatter, x, y, w, h
    local rc=0
    python3 "$HERE/glass-render-order-metrics.py" reach "$OUT/$2.png" "$OUT/$3.png" \
        --window "$PX" "$PY" "$PW" "$PH" --bevel "$7" --thickness "$6" \
        --gap "$4" --width "$5" --scatter "$8" --profile "$9" "${10}" "${11}" "${12}" \
        > "$OUT/$1-reach.json" || rc=$?
    cat "$OUT/$1-reach.json"
    [ "$rc" -eq 0 ] || fail "$1 profile/reach measurement failed"
}

attenuation_ratio() { # hex channel, thickness, optical distance at the flat normal
    awk -v channel="$1" -v thickness="$2" -v distance="$3" \
        'BEGIN { print exp(log(channel / 255) * thickness * 0.2 / distance) }'
}

attenuation_reach() { # name, dense on/off, white on/off, gap, width, thickness, bevel, x, y, w, h
    local r g b rc=0
    r=$(attenuation_ratio 34 "$8" 30); g=$(attenuation_ratio 36 "$8" 30); b=$(attenuation_ratio 54 "$8" 30)
    python3 "$HERE/glass-render-order-metrics.py" reach "$OUT/$2.png" "$OUT/$3.png" \
        --window "$PX" "$PY" "$PW" "$PH" --bevel "$9" --thickness "$8" \
        --gap "$6" --width "$7" --attenuation-only --attenuation-images "$OUT/$2.png" "$OUT/$3.png" \
        "$OUT/$4.png" "$OUT/$5.png" --attenuation-rect "${10}" "${11}" "${12}" "${13}" \
        --attenuation-ratio "$r" "$g" "$b" > "$OUT/$1-attenuation.json" || rc=$?
    cat "$OUT/$1-attenuation.json"
    [ "$rc" -eq 0 ] || fail "$1 attenuation interval gate failed"
}

assert_changed() { # name, on, off, crop; requires at least one delta > 1 code
    local changed
    changed=$(magick \( "$OUT/$2.png" -crop "$4" +repage \) \
        \( "$OUT/$3.png" -crop "$4" +repage \) -compose difference -composite \
        -separate +channel -evaluate-sequence max -threshold 0.4% \
        -format '%[fx:mean*w*h]' info:)
    assert_greater "$1 changed pixels" "$changed" 0
    printf '%s_changed_pixels=%s\n' "$1" "$changed" >> "$OUT/metrics.txt"
}

# `ring-gap` is measured from the face edge (the slab minus its chamfer), and so
# is the reach model's `--gap`. With offset 0 the face's top edge is the
# window's, so a band at gap g has its core on row PY + g.
within_ring() { # name, glass, gap, width, thickness, bevel, scatter
    RESPONSE_EXTRA="ring-color \"#ffffff\"
ring-gap $3
ring-width $4"
    capture "$1-off" "$2" "$NIRI" none
    capture "$1-on" "$2" "$NIRI" ring-light
    RESPONSE_EXTRA=
    reach_case "$1" "$1-on" "$1-off" "$3" "$4" "$5" "$6" "$7"
}

within_pinned_ring() {
    local common=$'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 20\nbevel 12'
    within_ring within-pinned "$common"$'\nchromatic-aberration 0' 5 2.6 20 12 0
    profile_reach within-pinned within-pinned-on within-pinned-off 5 2.6 20 12 0 \
        "$((PX + PW / 2))" "$((PY - 12))" 1 40
    within_ring within-pinned-aberration "$common"$'\nchromatic-aberration 0.6' 5 2.6 20 12 0
}

within_dense_ring() {
    within_ring within-dense $'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 80\nbevel 12\nattenuation-color "#222436"\nattenuation-distance 30' 5 2.6 80 12 0
    assert_changed within-dense-face within-dense-on within-dense-off "8x400+$PX+$((PY + 120))"
}

within_wide_ring() {
    within_ring within-wide $'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 20\nbevel 32' 5 20 20 32 0
    assert_changed within-wide-face within-wide-on within-wide-off "8x400+$PX+$((PY + 120))"
}

within_rough_ring() {
    within_ring within-rough $'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nroughness 0.5\nthickness 20\nbevel 12' 5 2.6 20 12 0.5
    profile_reach within-rough within-rough-on within-rough-off 5 2.6 20 12 0.5 \
        "$((PX + PW / 2))" "$((PY - 12))" 1 40
    local pinned_fwhm rough_fwhm pinned_peak rough_peak
    pinned_fwhm=$(jq -r .profile_fwhm "$OUT/within-pinned-reach.json")
    rough_fwhm=$(jq -r .profile_fwhm "$OUT/within-rough-reach.json")
    pinned_peak=$(jq -r .profile_peak_delta "$OUT/within-pinned-reach.json")
    rough_peak=$(jq -r .profile_peak_delta "$OUT/within-rough-reach.json")
    assert_greater 'rough ring FWHM' "$rough_fwhm" "$pinned_fwhm"
    awk -v rough="$rough_peak" -v pinned="$pinned_peak" 'BEGIN { exit !(rough < pinned) }' \
        || fail "rough ring peak $rough_peak is not lower than pinned $pinned_peak"
    printf 'pinned_ring_fwhm=%s\nrough_ring_fwhm=%s\npinned_ring_peak=%s\nrough_ring_peak=%s\n' \
        "$pinned_fwhm" "$rough_fwhm" "$pinned_peak" "$rough_peak" >> "$OUT/metrics.txt"
}

within_face_ring() {
    within_ring within-face $'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 20\nbevel 12' 20 2.6 20 12 0
    # The 8 px strip straddles the gap-20 band core on column PX + 20.
    assert_changed within-face-strip within-face-on within-face-off "8x400+$((PX + 16))+$((PY + 120))"
}

within_aurora() {
    local common
    # All static captures share 200 px: 20 and 80 leave the left chamfer
    # below the >1-code gate, while this also keeps the distortion pair matched.
    common=$'noise 0\nsaturation 1\niridescence 0\naurora 0.5 { drift-hz 0; }\nior 1.5\nthickness 200\nbevel 12\ndistortion 0'
    capture within-aurora-light-ior-1 "$common"$'\nlight-ior 1' "$NIRI" none
    capture within-aurora-light-ior-6 "$common"$'\nlight-ior 6' "$NIRI" none
    roi within-aurora-light-ior-1 "$(face_roi)" face
    roi within-aurora-light-ior-6 "$(face_roi)" face
    ae "$OUT/within-aurora-light-ior-1-face.png" "$OUT/within-aurora-light-ior-6-face.png"
    assert_zero "aurora flat-face landing identity" "$METRIC"
    assert_changed aurora-chamfer within-aurora-light-ior-1 within-aurora-light-ior-6 \
        "20x400+$((PX - 4))+$((PY + 120))"
    capture within-aurora-distortion "${common/distortion 0/distortion 0.4}"$'\nlight-ior 6' "$NIRI" none
    assert_changed aurora-distortion-face within-aurora-distortion within-aurora-light-ior-6 \
        "200x400+$((PX + 60))+$((PY + 120))"
}

within_aurora_motion() {
    local glass i
    glass=$'noise 0\nsaturation 1\niridescence 0\naurora 0.5 { drift-hz 1; }\nior 1.5\nthickness 20\nbevel 12\ndistortion 0.4\njelly-flex 0.004\njelly-ripple 0'
    GLASS_EXTRA=$'offset-x 0\noffset-y 0\n'"$glass"
    write_config "$OUT/aurora-motion.kdl"
    validate_config "$NIRI" "$OUT/aurora-motion.kdl"
    start_nested "$NIRI" "$OUT/aurora-motion.kdl"
    spawn_client "$NIRI" transparent
    sleep 2
    msg "$NIRI" action set-column-width +100
    for i in 1 2 3; do
        printf '%s aurora-motion-%s\n' "$(date --iso-8601=seconds)" "$i" >> "$OUT/metrics.txt"
        msg "$NIRI" -j windows >> "$OUT/aurora-motion-geometry.txt"
        shot "$NIRI" "aurora-motion-$i"
    done
    stop_nested
    assert_clean_log
}

within_attenuation() {
    local white dense aurora_white aurora_dense
    white=$'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 80\nbevel 12\nattenuation-color "#ffffff"\nattenuation-distance 30'
    dense=$'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }\nior 1.5\nthickness 80\nbevel 12\nattenuation-color "#222436"\nattenuation-distance 30'
    within_ring within-white "$white" 20 2.6 80 12 0
    within_ring within-dense-attenuation "$dense" 20 2.6 80 12 0
    attenuation_reach within-ring within-dense-attenuation-on within-dense-attenuation-off \
        within-white-on within-white-off 20 2.6 80 12 "$((PX + 60))" "$((PY + 19))" 200 3
    aurora_white=$'noise 0\nsaturation 1\niridescence 0\naurora 0.5 { drift-hz 0; }\nior 1.5\nthickness 80\nbevel 12\ndistortion 0\nattenuation-color "#ffffff"\nattenuation-distance 30'
    aurora_dense=$'noise 0\nsaturation 1\niridescence 0\naurora 0.5 { drift-hz 0; }\nior 1.5\nthickness 80\nbevel 12\ndistortion 0\nattenuation-color "#222436"\nattenuation-distance 30'
    capture within-aurora-white-off "${aurora_white/aurora 0.5/aurora 0}" "$NIRI" none
    capture within-aurora-white-on "$aurora_white" "$NIRI" none
    capture within-aurora-dense-off "${aurora_dense/aurora 0.5/aurora 0}" "$NIRI" none
    capture within-aurora-dense-on "$aurora_dense" "$NIRI" none
    attenuation_reach within-aurora within-aurora-dense-on within-aurora-dense-off \
        within-aurora-white-on within-aurora-white-off 5 2.6 80 12 "$((PX + 60))" "$((PY + 120))" 200 400
}

within_opaque() {
    local neutral active
    neutral=$'noise 0\nsaturation 1\niridescence 0\naurora 0 { drift-hz 0; }'
    active=$'noise 0.5 type="fine"\nsaturation 0\niridescence 0.8\naurora 0.5 { drift-hz 0; }'
    RESPONSE_EXTRA=$'ring-gap 20\nring-width 2.6'
    capture within-opaque-off "$neutral" "$NIRI" none opaque 1.0
    capture within-opaque-on "$active" "$NIRI" ring-light opaque 1.0
    RESPONSE_EXTRA=
    # Over the gap-20 band core (column PX + 20), which the opaque client hides.
    roi within-opaque-off "8x400+$((PX + 16))+$((PY + 120))" ring
    roi within-opaque-on "8x400+$((PX + 16))+$((PY + 120))" ring
    ae "$OUT/within-opaque-off-ring.png" "$OUT/within-opaque-on-ring.png"
    assert_zero "within opaque glyph identity" "$METRIC"
    roi within-opaque-off "$(face_roi)" glyph
    roi within-opaque-on "$(face_roi)" glyph
    ae "$OUT/within-opaque-off-glyph.png" "$OUT/within-opaque-on-glyph.png"
    assert_zero "within opaque face identity" "$METRIC"
}

within_additive() {
    additive_case aurora 'aurora 0 { drift-hz 0; }' 'aurora 0.5 { drift-hz 0; }' none
    RESPONSE_EXTRA=$'ring-gap 5\nring-width 2.6'
    additive_case ring '' '' ring-light
    RESPONSE_EXTRA=
    additive_case iridescence 'iridescence 0' 'iridescence 0.8' none
}

within_matrix() {
    neutral_identity
    within_pinned_ring
    within_dense_ring
    within_wide_ring
    within_rough_ring
    within_face_ring
    within_aurora
    within_aurora_motion
    within_attenuation
    within_opaque
    within_additive
}

run_scope() {
    if [ "$MODE" = red ]; then
        [ "$SCOPE" != cost ] || fail 'MODE=red has no cost scope'
        additive_case aurora 'aurora 0 { drift-hz 0; }' 'aurora 0.5 { drift-hz 0; }' none
        return
    fi
    if [ "${PHASE:-behind}" = within ]; then
        case $SCOPE in pixels) within_matrix ;; cost) cost_matrix ;; all) within_matrix; cost_matrix ;; esac
    else
        case $SCOPE in pixels) pixel_matrix ;; cost) cost_matrix ;; all) behind_matrix ;; esac
    fi
}
run_scope
finish
