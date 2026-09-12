#!/usr/bin/env bash
# Glass iridescence smoke (material-6102b2): the optic's neutral renders
# identically to the unconfigured material, its hue lands on the chamfer
# rather than the face, and its frame cost is recorded.
#
# Captures: plain (no node), zero (`iridescence 0`), on (`iridescence 0.8`),
# and rainbow (the preset body) for the evidence doc. The chamfer ROI samples
# the visible chamfer; the face ROI is inside the window. Oklab a/b RMSE against
# the zero capture measures added chroma. Cost is the GPU median of the
# material draw for plain, zero, and on, three rounds in rotated order.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT, CAPTURE_TASK (task id
# authorizing this run).
set -eu
: "${CAPTURE_TASK:?task id authorizing this run}"
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
capture_preflight headless
build_binaries
capture_identity --config preset=iridescence --config output=1280x720 --config scale=1 --config vrr=off
calibrate_probe_rect "$NIRI" 0

capture() {
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    sleep 2
    probe_rect "$NIRI"
    shot "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    roi "$1" "$(chamfer_roi)" chamfer
    stop_nested
}
capture plain ""
capture zero "iridescence 0"
capture on "iridescence 0.8"
capture rainbow $'ior 1.7\nchromatic-aberration 0.5\niridescence 0.8'

ae "$OUT/plain.png" "$OUT/zero.png"; zero_vs_plain_ae=$METRIC
for n in zero on; do
    oklab_ab "$OUT/$n-chamfer.png" "$OUT/$n-chamfer-ab.png"
    oklab_ab "$OUT/$n-face.png" "$OUT/$n-face-ab.png"
done
rmse "$OUT/on-chamfer-ab.png" "$OUT/zero-chamfer-ab.png"; chamfer_ab_rmse=$METRIC
rmse "$OUT/on-face-ab.png" "$OUT/zero-face-ab.png";       face_ab_rmse=$METRIC
rmse "$OUT/on-chamfer.png" "$OUT/zero-chamfer.png";       chamfer_rmse=$METRIC
rmse "$OUT/on-face.png" "$OUT/zero-face.png";             face_rmse=$METRIC
one_code=$(one_code)
assert_zero zero_vs_plain_ae "$zero_vs_plain_ae"
assert_greater chamfer_ab_rmse_over_one_code "$chamfer_ab_rmse" "$one_code"
assert_greater chamfer_ab_rmse_over_face "$chamfer_ab_rmse" "$face_ab_rmse"

# Cost: three rounds, order rotated to balance host drift.
tools_ready; reserve_tracy_port
gpu_case() {
    case $1 in plain) GLASS_EXTRA= ;; zero) GLASS_EXTRA="iridescence 0" ;; on) GLASS_EXTRA="iridescence 0.8" ;; esac
    trace_run "gpu-$1-$2" "$TICK" 0
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}
for c in plain zero on; do gpu_case "$c" 1; done
for c in zero on plain; do gpu_case "$c" 2; done
for c in on plain zero; do gpu_case "$c" 3; done
plain_ns=$(median3 "$OUT/gpu-plain.medians")
zero_ns=$(median3 "$OUT/gpu-zero.medians")
on_ns=$(median3 "$OUT/gpu-on.medians")

{
    printf '%s=%s\n' zero_vs_plain_ae "$zero_vs_plain_ae" chamfer_ab_rmse "$chamfer_ab_rmse" \
        face_ab_rmse "$face_ab_rmse" chamfer_rmse "$chamfer_rmse" face_rmse "$face_rmse" one_code "$one_code" \
        gpu_plain_ms "$(ns_to_ms "$plain_ns")" gpu_zero_ms "$(ns_to_ms "$zero_ns")" gpu_on_ms "$(ns_to_ms "$on_ns")" \
        gpu_zero_vs_plain_pct "$(pct_delta "$plain_ns" "$zero_ns")" gpu_on_vs_plain_pct "$(pct_delta "$plain_ns" "$on_ns")"
} | tee "$OUT/metrics.txt"

finish
