#!/usr/bin/env bash
# Glass parameter sweep (material-37cec9): render one KDL parameter at N values
# on a headless Weston host and report the perceptual delta between neighbouring
# captures, so the region where the effect stops changing is visible.
#
# This is a measurement tool, not a gate. A flat curve is a finding, not a
# failure; exit is non-zero only when the harness itself could not measure.
#
# Design: docs/specs/2026-09-06-glass-parameter-sweep-design.md
#
# Env:
#   NIRI          niri binary to run                                (required)
#   OUT           artifact directory                                (required)
#   BLOCK         glass | blur - which KDL block receives the key   (required)
#   KEY           KDL key, e.g. ior, thickness, noise               (required)
#   VALUES        two or more ascending values, whitespace-separated(required)
#   ROI_BEVEL     override the derived bevel-band crop
#   ROI_FACE      override the derived face-interior crop
#   TABLE_ONLY    1 to recompute the table from captures already in OUT
#
# The columns report WHERE the rendered image changes, not WHY. Neither region
# is evidence of ray bending: Fresnel varies with ior everywhere coverage > 0,
# and the focus filament refracts through ior too, so a bevel delta is
# consistent with backdrop refraction being broken. Measuring bending needs a
# separately validated instrument: material-343f27.
#
# Requires: weston, kitty, swaybg, jq, rg, ImageMagick 7.
set -eu

OUT=${OUT:?artifact directory}
BLOCK=${BLOCK:?glass or blur}
KEY=${KEY:?KDL key to sweep}
TABLE_ONLY=${TABLE_ONLY:-0}

fail() { echo "FAIL: $*" >&2; exit 1; }
is_number() { [[ $1 =~ ^-?[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }

mkdir -p "$OUT"

case $BLOCK in
    glass|blur) ;;
    *) fail "BLOCK must be glass or blur, got $BLOCK" ;;
esac

# jelly-flex and jelly-ripple are absent on purpose: jelly activity comes from
# motion residuals (render_helpers/material.rs) and the shader gates ripple
# behind mat_jelly_activity > 0, so a settled window renders them identically at
# every value. Measuring them needs a motion stimulus: material-36e968.
case $BLOCK:$KEY in
    glass:ior|glass:thickness|glass:attenuation-distance) ;;
    glass:chromatic-aberration|glass:distortion|glass:anisotropic-blur) ;;
    glass:roughness|glass:bevel|glass:light-ior|glass:noise|glass:saturation) ;;
    glass:iridescence|glass:aurora) ;;
    blur:noise|blur:saturation|blur:passes|blur:offset) ;;
    glass:jelly-flex|glass:jelly-ripple)
        fail "$KEY needs a motion stimulus and is out of scope here (material-36e968)" ;;
    *) fail "unsupported key for block $BLOCK: $KEY" ;;
esac

# resolve_material inherits the global blur noise/saturation only while backdrop
# blur is effective, and the gate is `glass.backdrop_blur && !blur.off`. Sweeping
# a blur key against a backdrop-blur-false baseline would resolve to the neutral
# value at every step and report a flat curve that is an artifact of the gate.
if [ "$BLOCK" = blur ]; then BACKDROP_BLUR=true; else BACKDROP_BLUR=false; fi

# ---------------------------------------------------------------- arguments

if [ "$TABLE_ONLY" = 1 ]; then
    [ -s "$OUT/values.txt" ] || fail "TABLE_ONLY needs values.txt in $OUT"
    mapfile -t VALUE_LIST < "$OUT/values.txt"
else
    VALUES=${VALUES:?two or more ascending values}
    read -r -a VALUE_LIST <<< "$VALUES"
fi

# Every metric here is a difference, so one value has nothing to report.
[ "${#VALUE_LIST[@]}" -ge 2 ] \
    || fail "need at least two values, got ${#VALUE_LIST[@]}"
for v in "${VALUE_LIST[@]}"; do
    is_number "$v" || fail "non-numeric value: $v"
done
# Ascending and distinct: a zero step would divide by zero in per_step, and an
# unsorted list makes "neighbour" mean nothing.
for ((i = 1; i < ${#VALUE_LIST[@]}; i++)); do
    awk -v a="${VALUE_LIST[i-1]}" -v b="${VALUE_LIST[i]}" 'BEGIN { exit !(b > a) }' \
        || fail "values must be strictly ascending: ${VALUE_LIST[i-1]} then ${VALUE_LIST[i]}"
done

# ------------------------------------------------------------------ backdrop

# A high-frequency field, so a change that displaces the backdrop registers
# across the ROI rather than only where it crosses an edge. Periodic, which is
# fine for RMSE and would be wrong for template matching - see the bending note
# in the header.
GRID_PERIOD=20
# Used only by the geometry probe, which renders opaque and without a material
# so its bounding box is the window rect. Absent from the backdrop palette, so
# nothing else in the frame can match it.
GEOM_COLOR='#ff00ff'
make_backdrop() {   # $1 out path
    local dst=$1 x y
    local args=(magick -size 1280x720 'xc:rgb(38,44,62)'
                -fill 'rgb(150,60,52)' -draw "rectangle 0,0 639,359"
                -fill 'rgb(48,120,80)' -draw "rectangle 640,360 1279,719"
                -fill 'rgb(120,96,40)' -draw "rectangle 640,0 1279,359"
                -stroke 'rgb(226,226,226)' -strokewidth 1)
    for ((x = 0; x < 1280; x += GRID_PERIOD)); do args+=(-draw "line $x,0 $x,719"); done
    for ((y = 0; y < 720; y += GRID_PERIOD)); do args+=(-draw "line 0,$y 1279,$y"); done
    args+=("$dst")
    "${args[@]}" || fail "backdrop generation failed"
}

# ------------------------------------------------------------ config writing

# Pinned baseline. Not derived from a generated prism.kdl: that file drifts
# (ior moved 1.02 to 1.24 within a week), which would make two runs of the same
# sweep incomparable. distortion is 0 here, which is what makes the flat face
# refract nothing - the shader perturbs the face normal only when distortion > 0.
# The swept key replaces its baseline line rather than being appended: KDL
# rejects a duplicate single node, so emitting both is a parse error.
GLASS_BASELINE=(
    "ior 1.5"
    "thickness 20"
    "attenuation-color \"#dfe8ff\""
    "attenuation-distance 60"
    "chromatic-aberration 0"
    "distortion 0 scale=0.5"
    "anisotropic-blur 0"
    "roughness 0"
    "jelly-flex 0"
    "jelly-ripple 0"
    "bevel 12"
    "offset-x 0"
    "offset-y 0"
)
BLUR_BASELINE=(
    "passes 1"
    "offset 8"
    "noise 0"
    "saturation 1"
)
emit_block() {   # $1 swept key or empty, $2 swept value, $3.. baseline lines
    local swept=$1 value=$2; shift 2
    local line
    for line in "$@"; do
        [ -n "$swept" ] && [ "${line%% *}" = "$swept" ] && continue
        printf '        %s\n' "$line"
    done
    [ -n "$swept" ] && printf '        %s %s\n' "$swept" "$value"
    return 0
}

write_config() {   # $1 path, $2 wallpaper, $3 swept value, $4 with-material 0|1
    local dst=$1 wall=$2 value=$3 with_material=$4
    local glass_key='' blur_key=''
    if [ "$with_material" = 1 ]; then
        if [ "$BLOCK" = glass ]; then glass_key=$KEY; else blur_key=$KEY; fi
    fi
    {
        cat <<KDL
prefer-no-csd
layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.6; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}
animations { off; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$wall"
blur {
KDL
        emit_block "$blur_key" "$value" "${BLUR_BASELINE[@]}"
        printf '}\n'
        if [ "$with_material" = 1 ]; then
            printf 'material "sweep-probe" {\n    glass {\n'
            printf '        backdrop-blur %s\n' "$BACKDROP_BLUR"
            emit_block "$glass_key" "$value" "${GLASS_BASELINE[@]}"
            # The focus response defaults to RingLight, which is drawn at the
            # window edge - exactly the bevel band. Pinned off because this
            # script measures glass optics and the ring is a separate feature
            # with its own parameters, not because the ring is unstable here:
            # `animations { off; }` above already pins its drift phase, and two
            # runs with the ring left on compare at AE 0 (material-0af212).
            printf '    }\n    response "default" {\n        focus "none"\n    }\n}\n'
            cat <<'KDL'
window-rule {
    match app-id="^sweep-probe$"
    material "sweep-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
KDL
        fi
    } > "$dst"
    # Assert the gate structurally rather than trusting it: with BLOCK=blur the
    # emitted config must enable backdrop blur and must not switch it off.
    if [ "$BLOCK" = blur ] && [ "$with_material" = 1 ]; then
        grep -q 'backdrop-blur true' "$dst" \
            || fail "blur sweep emitted a config without backdrop-blur true"
        grep -qE '^[[:space:]]*off[[:space:]]*$' "$dst" \
            && fail "blur sweep emitted a config containing blur { off }"
    fi
    return 0
}


# --------------------------------------------------------- nested niri host

WESTON_PID=; NIRI_PID=
if [ "$TABLE_ONLY" != 1 ]; then
    NIRI=${NIRI:?niri binary}
    RT=$(mktemp -d "$XDG_RUNTIME_DIR/gps.XXXXXX")
    RUN=$(basename "$RT")
    HOST=$RUN-host
    remove_runtime_dir() {
        python3 - "$RT" <<'PY'
import shutil
import sys

shutil.rmtree(sys.argv[1])
PY
    }
    stop_weston() {
        if [ -n "$WESTON_PID" ]; then
            kill "$WESTON_PID" 2>/dev/null || true
            wait "$WESTON_PID" 2>/dev/null || true
            WESTON_PID=
        fi
        for _ in $(seq 50); do [ ! -e "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
        [ ! -e "$XDG_RUNTIME_DIR/$HOST" ] || fail "Weston socket $HOST still present"
    }
    cleanup() {
        local rc=$?
        if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; fi
        stop_weston || rc=1
        remove_runtime_dir || rc=1
        exit "$rc"
    }
    trap cleanup EXIT
fi

start_nested() {   # $1 config
    local _
    weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
        --width=1280 --height=720 --socket="$HOST" >> "$OUT/weston.log" 2>&1 &
    WESTON_PID=$!
    for _ in $(seq 100); do
        [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break
        kill -0 "$WESTON_PID" 2>/dev/null || break
        sleep 0.1
    done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || fail "no Weston socket (see $OUT/weston.log)"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    "$NIRI" validate -c "$1" || fail "config $1 does not validate"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$NIRI" -c "$1" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1) || fail "no niri socket"
    export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    stop_weston
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
probe_count() {   # $1 app-id
    "$NIRI" msg -j windows | jq -r --arg a "$1" '.[] | select(.app_id==$a) | .id' | wc -l
}
spawn_probe() {   # $1 app-id, $2 background color, $3 opacity
    local _ id=$1 bg=$2 op=$3
    "$NIRI" msg action spawn -- kitty --config NONE --class "$id" \
        -o "background=$bg" -o "background_opacity=$op" -o cursor_blink_interval=0 \
        sh -c 'printf "\033[?25l"; exec sleep 600'
    for _ in $(seq 100); do [ "$(probe_count "$id")" -ge 1 ] && break; sleep 0.1; done
    [ "$(probe_count "$id")" -eq 1 ] || fail "expected exactly one $id window"
    sleep 2
}
shoot() {   # $1 destination png
    local _
    "$NIRI" msg action screenshot-screen --write-to-disk true --show-pointer false \
        --path "$1"
    for _ in $(seq 50); do [ -s "$1" ] && break; sleep 0.1; done
    [ -s "$1" ] || fail "capture $1 was not written"
}

# The window rect is measured from a capture, not read from the IPC. The IPC
# reports tile_pos_in_workspace_view as null for this window, and `null[0] + 0`
# is 0 in jq, so that route silently yields a plausible-looking 0,0 instead of
# failing. Measuring also lands the rect in screenshot pixel space directly,
# which is the space the crops are applied in - the headless output reports a
# Flipped180 transform, so logical coordinates are not guaranteed to match.
measure_rect() {   # $1 capture of the opaque geometry probe; prints "x y w h"
    local bbox
    bbox=$(magick "$1" -fuzz 12% +transparent "$GEOM_COLOR" -format '%@' info:) \
        || fail "bounding-box measurement failed on $1"
    [[ $bbox =~ ^([0-9]+)x([0-9]+)\+([0-9]+)\+([0-9]+)$ ]] \
        || fail "could not locate the geometry probe in $1 (got '$bbox')"
    printf '%s %s %s %s\n' "${BASH_REMATCH[3]}" "${BASH_REMATCH[4]}" \
        "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
}

CAP() { printf '%s/cap-%03d.png' "$OUT" "$1"; }

# ------------------------------------------------------------ capture pass

if [ "$TABLE_ONLY" != 1 ]; then
    printf '%s\n' "${VALUE_LIST[@]}" > "$OUT/values.txt"
    "$NIRI" --version > "$OUT/niri.version"

    # Phase 0: measure the window rect, which the ROIs derive from. The probe is
    # opaque and carries no material, so its bounding box is the window rect
    # exactly; under the glass optics its edges would be tinted and refracted.
    make_backdrop "$OUT/backdrop.png"
    write_config "$OUT/geom.kdl" "$OUT/backdrop.png" "" 0
    start_nested "$OUT/geom.kdl"
    spawn_probe sweep-geom "$GEOM_COLOR" 1
    shoot "$OUT/geom.png"
    stop_nested
    read -r WX WY WW WH <<< "$(measure_rect "$OUT/geom.png")"
    echo "window ${WW}x${WH}+${WX}+${WY}" | tee "$OUT/geometry.txt"

    # The ROIs are fixed for the whole sweep. RMSE between crops taken at
    # different positions measures the crop, and between crops of different
    # sizes it does not compute at all. When bevel is the swept key the band is
    # sized from the widest chamfer in the sweep so it still frames it.
    CHAMFER=12
    if [ "$KEY" = bevel ]; then
        CHAMFER=$(printf '%s\n' "${VALUE_LIST[@]}" | sort -g | tail -1)
        CHAMFER=${CHAMFER%.*}
        [ "${CHAMFER:-0}" -gt 0 ] \
            || fail "sweeping bevel with every value 0 leaves a zero-width band"
    fi
    # The band straddles the window's left edge: OUTSET px of backdrop outside
    # it, the chamfer and a little face inside. Clamped to the screen, because a
    # window flush against the edge would otherwise produce a negative offset
    # and magick would reject the crop.
    BAND=$((CHAMFER + 10))
    OUTSET=5
    [ "$WX" -lt "$OUTSET" ] && OUTSET=$WX
    BEVEL_X=$((WX - OUTSET)); BEVEL_W=$((BAND + OUTSET))
    BEVEL_Y=$((WY + WH / 4)); BEVEL_H=$((WH / 2))
    [ "$BEVEL_W" -gt 0 ] && [ "$BEVEL_H" -gt 0 ] \
        || fail "derived a degenerate bevel band: ${BEVEL_W}x${BEVEL_H}"
    ROI_BEVEL=${ROI_BEVEL:-${BEVEL_W}x${BEVEL_H}+${BEVEL_X}+${BEVEL_Y}}
    FACE_W=$((WW / 3)); FACE_H=$((WH / 3))
    ROI_FACE=${ROI_FACE:-${FACE_W}x${FACE_H}+$((WX + WW / 2 - FACE_W / 2))+$((WY + WH / 2 - FACE_H / 2))}
    printf 'bevel\t%s\nface\t%s\n' "$ROI_BEVEL" "$ROI_FACE" | tee "$OUT/roi.txt"

    i=0
    for v in "${VALUE_LIST[@]}"; do
        kdl=$(printf '%s/cfg-%03d.kdl' "$OUT" "$i")
        png=$(CAP "$i")
        write_config "$kdl" "$OUT/backdrop.png" "$v" 1
        start_nested "$kdl"
        spawn_probe sweep-probe none 0
        shoot "$png"
        stop_nested
        i=$((i + 1))
    done

    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
        fail "material error, fallback or panic in niri.log"
    fi
fi

# -------------------------------------------------------------- table pass
# Reads the captures back from disk, in a pass separate from producing them, so
# it can be pointed at a prepared directory. That separation is what makes the
# zero-normalization case checkable without a compositor.

[ -s "$OUT/roi.txt" ] || fail "no roi.txt in $OUT"
ROI_BEVEL=$(awk -F'\t' '$1=="bevel"{print $2}' "$OUT/roi.txt")
ROI_FACE=$(awk -F'\t' '$1=="face"{print $2}' "$OUT/roi.txt")
[ -n "$ROI_BEVEL" ] && [ -n "$ROI_FACE" ] || fail "roi.txt is missing a region"

# compare exits 0 for an identical pair and 1 for a differing pair; anything
# else is an execution error. -colorspace Lab is honoured directly, so no
# intermediate file is written: PNG cannot store Lab and silently returns the
# sRGB number, and a TIFF round-trip silently returns 0 - indistinguishable
# from the flat sweep this script exists to report.
lab_rmse() {   # $1 image, $2 image; result in METRIC
    local out status
    set +e
    out=$(magick compare -colorspace Lab -metric RMSE "$1" "$2" null: 2>&1 >/dev/null)
    status=$?
    set -e
    case $status in
        0|1) ;;
        *) fail "magick compare on $1 $2 exited $status: $out" ;;
    esac
    out=${out##*(}; out=${out%)*}
    is_number "$out" || fail "non-numeric metric for $1 $2: $out"
    METRIC=$out
}

n=${#VALUE_LIST[@]}
for ((i = 0; i < n; i++)); do
    [ -s "$(CAP "$i")" ] || fail "missing capture $(CAP "$i")"
    magick "$(CAP "$i")" -crop "$ROI_BEVEL" +repage "$OUT/roi-bevel-$i.png" \
        || fail "bevel crop failed on capture $i"
    magick "$(CAP "$i")" -crop "$ROI_FACE" +repage "$OUT/roi-face-$i.png" \
        || fail "face crop failed on capture $i"
done

: > "$OUT/raw.tsv"
for ((i = 0; i < n; i++)); do
    if [ "$i" -eq 0 ]; then
        bn=-; fn=-; step=-
    else
        lab_rmse "$OUT/roi-bevel-$((i-1)).png" "$OUT/roi-bevel-$i.png"; bn=$METRIC
        lab_rmse "$OUT/roi-face-$((i-1)).png"  "$OUT/roi-face-$i.png";  fn=$METRIC
        step=$(awk -v a="${VALUE_LIST[i-1]}" -v b="${VALUE_LIST[i]}" \
            'BEGIN { printf "%.6g", b - a }')
    fi
    lab_rmse "$OUT/roi-bevel-0.png" "$OUT/roi-bevel-$i.png"; bc=$METRIC
    lab_rmse "$OUT/roi-face-0.png"  "$OUT/roi-face-$i.png";  fc=$METRIC
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
        "${VALUE_LIST[i]}" "$step" "$bn" "$bc" "$fn" "$fc" >> "$OUT/raw.tsv"
done
# ------------------------------------------------------------------- output
# per_step divides the neighbour delta by the parameter step, because a caller
# sampling densely in one region and coarsely in another gets smaller neighbour
# deltas from the smaller step alone. normalized is computed from per_step for
# the same reason, and is 0 for every row when every per_step is 0 - a flat
# sweep is a supported outcome and must still produce a readable table.
awk -F'\t' -v key="$KEY" '
BEGIN { OFS = "\t" }
{
    val[NR] = $1; step[NR] = $2; bn[NR] = $3; bc[NR] = $4; fn[NR] = $5; fc[NR] = $6
    if (NR > 1) {
        bps[NR] = bn[NR] / step[NR]
        fps[NR] = fn[NR] / step[NR]
        if (bps[NR] > bmax) bmax = bps[NR]
        if (fps[NR] > fmax) fmax = fps[NR]
    }
    n = NR
}
END {
    hdr = "key" OFS "value" OFS "step" OFS "bevel_neighbor" OFS "bevel_per_step" \
        OFS "bevel_cumulative" OFS "bevel_normalized" OFS "face_neighbor" \
        OFS "face_per_step" OFS "face_cumulative" OFS "face_normalized"
    print hdr
    for (i = 1; i <= n; i++) {
        if (i == 1) {
            # No predecessor: the step-derived columns are sentinels, and
            # cumulative is 0 by definition. This contract and the all-zero
            # contract below apply to different rows.
            bps_s = "-"; fps_s = "-"; bnorm = "-"; fnorm = "-"
            bn_s = "-"; fn_s = "-"
        } else {
            bn_s = sprintf("%.6g", bn[i]); fn_s = sprintf("%.6g", fn[i])
            bps_s = sprintf("%.6g", bps[i]); fps_s = sprintf("%.6g", fps[i])
            bnorm = sprintf("%.4f", bmax > 0 ? bps[i] / bmax : 0)
            fnorm = sprintf("%.4f", fmax > 0 ? fps[i] / fmax : 0)
        }
        row = key OFS val[i] OFS step[i] OFS bn_s OFS bps_s \
            OFS sprintf("%.6g", bc[i]) OFS bnorm OFS fn_s OFS fps_s \
            OFS sprintf("%.6g", fc[i]) OFS fnorm
        print row
    }
}' "$OUT/raw.tsv" | tee "$OUT/sweep.tsv" | column -t -s "$(printf '\t')"

if [ "$TABLE_ONLY" != 1 ]; then
    sha256sum "$OUT"/*.png "$OUT"/*.kdl > "$OUT/SHA256SUMS"
fi
echo "artifacts in $OUT" >&2
