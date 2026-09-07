#!/usr/bin/env bash
# Glass noise type smoke (material-6e7352): capture a blank transparent kitty
# over a glass material on a headless Weston host, once per noise type at the
# same amount, and assert the properties the design claims for each type
# against an amount-0 capture of the same fixture. Exit 0 means every
# assertion held; any failure exits non-zero with a FAIL line and the trap
# preserves that status.
#
# The backdrop is a flat warm mid-tone rather than the colour bars of
# glass-noise-saturation-smoke.sh: the statistics below need nearly all grain
# to stay inside the gamut (white reaches ±0.25 of full scale at amount 0.5,
# fine ±0.47), and the chroma-invariance check needs a backdrop with chroma
# to hold. A small clipped share is measured as a limitation. Every
# difference image below is against the amount-0 capture of
# the same session type, so static content cancels exactly and only the
# grain remains; that relies on the same determinism the omitted check
# asserts first.
#
# Env: IMPL (niri binary under test), OUT (artifact dir).
# Requires: weston, kitty, swaybg, jq, rg, ImageMagick 7 with Oklab.
set -eu
IMPL=${IMPL:?niri binary under test}
OUT=${OUT:?artifact directory}
mkdir -p "$OUT"
# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/gnt.XXXXXX")
RUN=$(basename "$RT")
HOST=$RUN-host; UNIT=$RUN-weston; NIRI_PID=
fail() { echo "FAIL: $*" >&2; exit 1; }
cleanup() {
    local rc=$?
    if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; fi
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -rf "$RT"
    if [ -S "$XDG_RUNTIME_DIR/$HOST" ]; then echo "WARN: weston socket $HOST still present" >&2; fi
    exit "$rc"
}
trap cleanup EXIT
WALL=$OUT/warm-mid.png
magick -size 1280x720 xc:'rgb(140,115,90)' "$WALL"

write_config() {   # $1 path, $2 glass extra lines, $3 blur extra lines
    cat > "$1" <<KDL
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
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
blur {
    passes 1
    offset 8
    noise 0.08
    saturation 1.5
    $3
}
material "gnt-probe" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
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
        $2
    }
}
window-rule {
    match app-id="^gnt-probe$"
    material "gnt-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
KDL
}

start_nested() {   # $1 niri binary, $2 config
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$HOST" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || fail "no Weston socket"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    "$1" validate -c "$2" || fail "config $2 does not validate with $1"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$1" -c "$2" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1) || fail "no niri socket"
    export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
msg() { "$1" msg "${@:2}"; }
probe_count() { msg "$1" -j windows | jq -r '.[] | select(.app_id=="gnt-probe") | .id' | wc -l; }

capture() {   # $1 name, $2 niri binary, $3 glass extra, $4 blur extra
    local kdl=$OUT/$1.kdl png=$OUT/$1.png
    write_config "$kdl" "$3" "$4"
    start_nested "$2" "$kdl"
    msg "$2" action spawn -- kitty --config NONE --class gnt-probe -o background_opacity=0 \
        -o cursor_blink_interval=0 sh -c 'printf "\033[?25l"; exec sleep 600'
    for _ in $(seq 100); do [ "$(probe_count "$2")" -ge 1 ] && break; sleep 0.1; done
    [ "$(probe_count "$2")" -eq 1 ] || fail "expected exactly one probe window for $1"
    sleep 2
    msg "$2" action screenshot-screen --write-to-disk true --show-pointer false --path "$png"
    for _ in $(seq 50); do [ -s "$png" ] && break; sleep 0.1; done
    [ -s "$png" ] || fail "capture $1 not written"
    magick "$png" -crop 200x400+100+160 +repage -define png:color-type=2 "$OUT/$1-roi.png"
    stop_nested
}

# magick compare exits 0 for an identical pair, 1 for a differing pair, and 2
# on an execution error such as a missing image. The metric is the first
# field on stderr, printed as "<absolute> (<normalized>)". Only statuses 0 and
# 1 are comparisons; anything else is a failure, and a metric that is not a
# number is a failure, so a broken capture can never pass as a positive value.
# The helpers leave their number in METRIC instead of printing it, so `fail`
# runs in this shell and exits the script rather than a $(...) subshell.
is_number() { [[ $1 =~ ^[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }
compare_metric() {   # $1 metric name, $2 image, $3 image; result in METRIC
    local out status
    set +e
    out=$(magick compare -metric "$1" "$2" "$3" null: 2>&1 >/dev/null)
    status=$?
    set -e
    case $status in
        0|1) ;;
        *) fail "magick compare $1 $2 $3 exited $status: $out" ;;
    esac
    out=${out%% *}
    is_number "$out" || fail "magick compare $1 $2 $3 returned a non-numeric metric: $out"
    METRIC=$out
}
ae() { compare_metric AE "$1" "$2"; }
rmse() { compare_metric RMSE "$1" "$2"; }
sd() {   # $1 image; result in METRIC
    local out
    out=$(magick "$1" -colorspace Gray -format '%[fx:standard_deviation]' info:) || fail "magick info on $1 failed"
    is_number "$out" || fail "standard deviation of $1 is not numeric: $out"
    METRIC=$out
}
# Mean of a 0/1 mask image: the share of pixels the expression selected.
share() {   # $1 image, $2 fx expression over u; result in METRIC
    local out
    out=$(magick "$1" -colorspace Gray -fx "$2" -format '%[fx:mean]' info:) || fail "magick share on $1 failed"
    is_number "$out" || fail "share of $1 is not numeric: $out"
    METRIC=$out
}
# Signed difference of two captures offset by 0.5, as a 16-bit gray image,
# for statistics that need the sign (compose Mathematics computes
# source - destination + 0.5; which operand is which does not matter to a
# standard deviation). abs difference is for statistics that do not.
signed_diff() {   # $1 a, $2 b, $3 out
    magick "$1" "$2" -compose Mathematics -define compose:args=0,1,-1,0.5 -composite -colorspace Gray -depth 16 "$3" || fail "signed diff $3 failed"
}
ratio() {   # $1 numerator variance, $2 denominator variance, printed
    awk -v a="$1" -v b="$2" 'BEGIN { if (b == 0) print "nan"; else print a / b }'
}
abs_diff() {   # $1 a, $2 b, $3 out
    magick "$1" "$2" -compose difference -composite -colorspace Gray -depth 16 "$3" || fail "abs diff $3 failed"
}
# Oklab a and b only, L pinned, frozen as raw channels in a 16-bit image.
oklab_ab() {   # $1 in, $2 out
    magick "$1" -colorspace Oklab -channel R -evaluate set 50% +channel -set colorspace sRGB -depth 16 "$2" || fail "oklab ab $2 failed"
}
assert_less() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a < b) }' || fail "$1 expected $2 < $3"
}
assert_close() {   # $1 name, $2 value, $3 reference, $4 relative tolerance
    is_number "$2" && is_number "$3" && is_number "$4" || fail "$1 is not numeric: $2 vs $3 (tol $4)"
    awk -v a="$2" -v b="$3" -v t="$4" 'BEGIN { d = a - b; if (d < 0) d = -d; exit !(d <= t * b) }' \
        || fail "$1 expected $2 within $4 of $3"
}
assert_zero() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v == 0) }' || fail "$1 expected 0, got $2"
}
assert_positive() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v > 0) }' || fail "$1 expected > 0, got $2"
}
assert_greater() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a > b) }' || fail "$1 expected $2 > $3"
}

sha256sum "$IMPL" > "$OUT/binaries.sha256"
"$IMPL" --version > "$OUT/impl.version"

capture zero-first "$IMPL" $'noise 0\n        saturation 1' ""
capture zero-after "$IMPL" $'noise 0\n        saturation 1' ""
capture untyped    "$IMPL" $'noise 0.5\n        saturation 1' ""
capture white      "$IMPL" $'noise 0.5 type="white"\n        saturation 1' ""
capture fine       "$IMPL" $'noise 0.5 type="fine"\n        saturation 1' ""
capture lightness  "$IMPL" $'noise 0.5 type="lightness"\n        saturation 1' ""

ae "$OUT/zero-first.png" "$OUT/zero-after.png";  zero_determinism_ae=$METRIC
ae "$OUT/untyped.png" "$OUT/white.png";          untyped_vs_white_ae=$METRIC

sd "$OUT/zero-after-roi.png"; zero_sd=$METRIC
oklab_ab "$OUT/zero-after-roi.png" "$OUT/zero-ab.png"
for t in white fine lightness; do
    sd "$OUT/$t-roi.png"; printf -v "${t}_sd" '%s' "$METRIC"
    abs_diff "$OUT/$t-roi.png" "$OUT/zero-after-roi.png" "$OUT/$t-absdiff.png"
    signed_diff "$OUT/$t-roi.png" "$OUT/zero-after-roi.png" "$OUT/$t-signed.png"
    # Deviation statistics in units of the amount, which is 0.5 of full
    # scale: white's bound 0.5*amount is 0.25 of full scale (0.2549 allows
    # for 8-bit rounding), white's top band starts at 0.4*amount = 0.2, and
    # white's median 0.25*amount is 0.125.
    share "$OUT/$t-absdiff.png" 'u>=0.2 && u<=0.2549 ? 1 : 0'; printf -v "${t}_top_band" '%s' "$METRIC"
    share "$OUT/$t-absdiff.png" 'u<0.125 ? 1 : 0';             printf -v "${t}_below_median" '%s' "$METRIC"
    share "$OUT/$t-absdiff.png" 'u>0.2549 ? 1 : 0';            printf -v "${t}_beyond_white" '%s' "$METRIC"
    sd "$OUT/$t-signed.png"; printf -v "${t}_full_sd" '%s' "$METRIC"
    magick "$OUT/$t-signed.png" -filter box -resize 25% "$OUT/$t-signed-down.png"
    sd "$OUT/$t-signed-down.png"; printf -v "${t}_down_sd" '%s' "$METRIC"
    down=${t}_down_sd; full=${t}_full_sd
    down_variance=$(awk -v v="${!down}" 'BEGIN { print v * v }')
    full_variance=$(awk -v v="${!full}" 'BEGIN { print v * v }')
    printf -v "${t}_lowfreq_ratio" '%s' "$(ratio "$down_variance" "$full_variance")"
    oklab_ab "$OUT/$t-roi.png" "$OUT/$t-ab.png"
    rmse "$OUT/$t-ab.png" "$OUT/zero-ab.png"; printf -v "${t}_ab_rmse" '%s' "$METRIC"
done

for v in zero_determinism_ae untyped_vs_white_ae zero_sd; do
    printf '%s=%s\n' "$v" "${!v}"
done | tee "$OUT/metrics.txt"
for t in white fine lightness; do
    for m in sd full_sd down_sd lowfreq_ratio top_band below_median beyond_white ab_rmse; do
        v=${t}_$m
        printf '%s=%s\n' "$v" "${!v}"
    done
done | tee -a "$OUT/metrics.txt"

assert_zero zero_determinism_ae "$zero_determinism_ae"
assert_zero untyped_vs_white_ae "$untyped_vs_white_ae"
assert_greater white_sd "$white_sd" "$zero_sd"
assert_greater fine_sd "$fine_sd" "$zero_sd"
assert_greater lightness_sd "$lightness_sd" "$zero_sd"
assert_close fine_full_sd "$fine_full_sd" "$white_full_sd" 0.1
assert_less fine_lowfreq_ratio "$fine_lowfreq_ratio" "$white_lowfreq_ratio"   # variance ratio expected ≈0.0144 vs ≈0.0625
assert_less fine_top_band "$fine_top_band" "$white_top_band"
assert_greater fine_below_median "$fine_below_median" "$white_below_median"
assert_less lightness_ab_rmse "$lightness_ab_rmse" "$white_ab_rmse"
assert_less lightness_ab_rmse "$lightness_ab_rmse" "$fine_ab_rmse"
one_code=$(magick xc: -format '%[fx:quantumrange/255]' info:) || fail "magick quantum range failed"
assert_less lightness_ab_rmse_one_code "$lightness_ab_rmse" "$one_code"      # raw RMSE stays below one 8-bit code at ImageMagick's native quantum depth
# fine_beyond_white is recorded, not asserted: a few percent of fine's pixels
# exceed white's bound by design.

sha256sum "$OUT"/*.png "$OUT"/*.kdl >> "$OUT/SHA256SUMS"
if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
    fail "material error, fallback or panic in niri.log"
fi
echo "PASS: artifacts in $OUT"
