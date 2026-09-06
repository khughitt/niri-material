#!/usr/bin/env bash
# Glass noise and saturation parameters smoke (material-1293e8): capture a
# blank transparent kitty over a glass material on a headless Weston host and
# assert that written noise/saturation render with backdrop-blur off and with
# blur { off }, and that omitted values are byte-identical to the pre-change
# binary. Exit 0 means every assertion held; any failure exits non-zero with
# a FAIL line and the trap preserves that status.
#
# BASE must be no older than f8bcb34c. That commit added the ring of light,
# which draws at the window edge under the default focus response and so lands
# inside the full-frame captures omitted_identity_ae compares; a BASE from
# before it renders no ring and cannot be byte-identical to IMPL (the ring's
# own contribution here measures AE 3675). It also cannot be pinned away: the
# `focus` key does not parse on a pre-f8bcb34c binary, so writing
# `response "default" { focus "none" }` into this shared config would make
# BASE fail validation instead. See material-0af212.
#
# The ring itself is stable here. `animations { off; }` below pins its drift
# phase (`drift_rate` returns 0 when animations are off,
# src/render_helpers/signal.rs), and two runs of this script produce metrics
# identical to the last digit. That dependency is load-bearing: dropping the
# animations line would make every full-frame assertion below flaky.
#
# Env: IMPL (implementation niri), BASE (pre-change niri), OUT (artifact dir).
# Requires: weston, kitty, swaybg, jq, rg, ImageMagick.
set -eu
IMPL=${IMPL:?implementation niri binary}
BASE=${BASE:?pre-change niri binary}
OUT=${OUT:?artifact directory}
mkdir -p "$OUT"
# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/gns.XXXXXX")
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
WALL=$OUT/color-bars.png
magick -size 425x720 xc:'rgb(255,32,32)' -size 427x720 xc:'rgb(32,255,32)' -size 428x720 xc:'rgb(32,32,255)' +append "$WALL"

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
material "gns-probe" {
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
    match app-id="^gns-probe$"
    material "gns-probe"
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
probe_count() { msg "$1" -j windows | jq -r '.[] | select(.app_id=="gns-probe") | .id' | wc -l; }

capture() {   # $1 name, $2 niri binary, $3 glass extra, $4 blur extra
    local kdl=$OUT/$1.kdl png=$OUT/$1.png
    write_config "$kdl" "$3" "$4"
    start_nested "$2" "$kdl"
    msg "$2" action spawn -- kitty --config NONE --class gns-probe -o background_opacity=0 \
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

sha256sum "$IMPL" "$BASE" > "$OUT/binaries.sha256"
"$BASE" --version > "$OUT/base.version"; "$IMPL" --version > "$OUT/impl.version"

capture omitted-before   "$BASE" "" ""
capture omitted-after    "$IMPL" "" ""
capture written-neutral  "$IMPL" $'noise 0\n        saturation 1' ""
capture written-sat-0    "$IMPL" $'noise 0\n        saturation 0' ""
capture written-noise-05 "$IMPL" $'noise 0.5\n        saturation 1' ""
capture written-blur-off "$IMPL" $'noise 0.5\n        saturation 0' "off"

ae "$OUT/omitted-before.png" "$OUT/omitted-after.png";   omitted_identity_ae=$METRIC
ae "$OUT/written-neutral.png" "$OUT/omitted-after.png";  written_neutral_vs_omitted_ae=$METRIC
for ch in R G B; do magick "$OUT/written-sat-0-roi.png" -channel $ch -separate "$OUT/sat0-$ch.png"; done
ae "$OUT/sat0-R.png" "$OUT/sat0-G.png";                   sat0_rg_ae=$METRIC
ae "$OUT/sat0-R.png" "$OUT/sat0-B.png";                   sat0_rb_ae=$METRIC
rmse "$OUT/written-neutral-roi.png" "$OUT/written-noise-05-roi.png"; noise_roi_rmse=$METRIC
sd "$OUT/written-neutral-roi.png";                        neutral_sd=$METRIC
sd "$OUT/written-noise-05-roi.png";                       noise05_sd=$METRIC
for ch in R G B; do magick "$OUT/written-blur-off-roi.png" -channel $ch -separate "$OUT/bluroff-$ch.png"; done
ae "$OUT/bluroff-R.png" "$OUT/bluroff-G.png";             bluroff_rg_ae=$METRIC
ae "$OUT/written-blur-off.png" "$OUT/omitted-after.png";  bluroff_vs_omitted_ae=$METRIC

for v in omitted_identity_ae written_neutral_vs_omitted_ae sat0_rg_ae sat0_rb_ae noise_roi_rmse neutral_sd noise05_sd bluroff_rg_ae bluroff_vs_omitted_ae; do
    printf '%s=%s\n' "$v" "${!v}"
done | tee "$OUT/metrics.txt"

assert_zero omitted_identity_ae "$omitted_identity_ae"                    # omitted values: byte-identical to the pre-change binary (BASE must be >= f8bcb34c; see the header)
assert_zero written_neutral_vs_omitted_ae "$written_neutral_vs_omitted_ae"  # writing the neutral pair is indistinguishable from omission when backdrop blur is off (the override claim itself is covered by the written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur unit test in src/layout/tile.rs)
assert_zero sat0_rg_ae "$sat0_rg_ae"                                      # written saturation 0 renders grayscale with backdrop-blur off
assert_zero sat0_rb_ae "$sat0_rb_ae"
assert_positive noise_roi_rmse "$noise_roi_rmse"                          # written noise changes the ROI
assert_greater noise05_sd "$noise05_sd" "$neutral_sd"                     # and raises its variance
assert_zero bluroff_rg_ae "$bluroff_rg_ae"                                # written values survive blur { off }
assert_positive bluroff_vs_omitted_ae "$bluroff_vs_omitted_ae"

sha256sum "$OUT"/*.png "$OUT"/*.kdl >> "$OUT/SHA256SUMS"
if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
    fail "material error, fallback or panic in niri.log"
fi
echo "PASS: artifacts in $OUT"
