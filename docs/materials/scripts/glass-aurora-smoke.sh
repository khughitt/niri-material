#!/usr/bin/env bash
# Glass aurora smoke (material-8db3b0): the optic's neutral renders
# identically to the unconfigured material; a pinned field (`drift-hz 0`)
# is deterministic; a moving field on an unfocused, signal-free window
# changes across a bucket boundary and not within one; the field adds
# light on the face; redraws per second track drift-hz under each motion
# policy; and frame cost is recorded.
#
# Bucket check: at `drift-hz 1` a bucket is 1 s. Four captures taken back
# to back span well under a second, so at least one consecutive pair lies
# in one bucket and must be identical; a capture 2.5 s after the first lies
# in a later bucket and must differ. Redraw rates come from Tracy counts of
# `Niri::redraw` in the final 20 s of a 30 s capture with both windows idle.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT, CAPTURE_TASK (task id
# authorizing this run).
set -eu
: "${CAPTURE_TASK:?task id authorizing this run}"
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
capture_preflight headless
build_binaries
capture_identity --config preset=aurora --config output=1280x720 --config scale=1 --config vrr=off
TOP_EXTRA='signal { idle-after-ms 0; }'
calibrate_probe_rect "$NIRI" 1

session() {   # $1 name, $2 glass extra; leaves the session running, probe unfocused
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    steal_focus "$NIRI"
    sleep 2
    probe_rect "$NIRI"
}
capture() {   # $1 name, $2 glass extra: one shot with ROIs
    session "$1" "$2"
    shot "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    stop_nested
}

capture plain ""
capture zero "aurora 0 { drift-hz 4; }"
capture preset $'attenuation-color "#cfe0ff"\naurora 0.5 { drift-hz 0; color "#3dffb0"; color "#7a5cff"; }'

# Pinned: two shots 2 s apart are identical, and the field lights the face.
session pinned "aurora 0.5 { drift-hz 0; }"
shot "$NIRI" pinned-a; roi pinned-a "$(face_roi)" face
sleep 2
shot "$NIRI" pinned-b
stop_nested

# Moving at 1 Hz, unfocused, signal-free.
session moving "aurora 0.5 { drift-hz 1; }"
for i in 1 2 3 4; do shot "$NIRI" "moving-$i"; done
sleep 2.5
shot "$NIRI" moving-late
stop_nested

ae "$OUT/plain.png" "$OUT/zero.png";           zero_vs_plain_ae=$METRIC
ae "$OUT/pinned-a.png" "$OUT/pinned-b.png";    pinned_ae=$METRIC
rmse "$OUT/pinned-a-face.png" "$OUT/zero-face.png"; face_rmse=$METRIC
mean "$OUT/pinned-a-face.png"; lit_face_mean=$METRIC
mean "$OUT/zero-face.png";     zero_face_mean=$METRIC
within_min=
for i in 1 2 3; do
    ae "$OUT/moving-$i.png" "$OUT/moving-$((i + 1)).png"
    printf -v "moving_ae_$i" '%s' "$METRIC"
    if [ -z "$within_min" ] || awk -v a="$METRIC" -v b="$within_min" 'BEGIN { exit !(a < b) }'; then within_min=$METRIC; fi
done
ae "$OUT/moving-1.png" "$OUT/moving-late.png"; across_ae=$METRIC
one_code=$(one_code)

# Fail visual regressions before spending more than nine minutes tracing.
assert_zero zero_vs_plain_ae "$zero_vs_plain_ae"
assert_zero pinned_ae "$pinned_ae"
assert_greater face_rmse_over_one_code "$face_rmse" "$one_code"
assert_greater lit_face_brighter "$lit_face_mean" "$zero_face_mean"
assert_zero within_bucket_min_ae "$within_min"
assert_positive across_bucket_ae "$across_ae"

# Redraw rate against drift-hz: idle windows, the probe unfocused.
tools_ready; reserve_tracy_port
rate_case() {   # $1 name, $2 glass extra, $3 signal body
    GLASS_EXTRA=$2; TOP_EXTRA="signal { idle-after-ms 0; $3 }"
    trace_run "rate-$1" "$IDLE" 1
    count_last20 "rate-$1"
}
rate_plain=$(rate_case plain "" "")
rate_pinned=$(rate_case pinned "aurora 0.5 { drift-hz 0; }" "")
rate_4hz=$(rate_case 4hz "aurora 0.5 { drift-hz 4; }" "")
rate_reduced=$(rate_case reduced "aurora 0.5 { drift-hz 4; }" 'motion "reduced";')
rate_off=$(rate_case off "aurora 0.5 { drift-hz 4; }" 'motion "off";')
TOP_EXTRA='signal { idle-after-ms 0; }'

# Cost: three rounds, order rotated.
gpu_case() {   # $1 case, $2 round
    local cooldown_name
    case $1 in plain) GLASS_EXTRA= ;; zero) GLASS_EXTRA="aurora 0 { drift-hz 4; }" ;; on) GLASS_EXTRA="aurora 0.5 { drift-hz 4; }" ;; esac
    cooldown_name="gpu-$1-$2"
    printf '%s %s cooldown start (30s)\n' "$(date --iso-8601=seconds)" "$cooldown_name" \
        | tee -a "$OUT/cost-cooldown.log" >&2
    sleep 30
    printf '%s %s cooldown complete\n' "$(date --iso-8601=seconds)" "$cooldown_name" \
        | tee -a "$OUT/cost-cooldown.log" >&2
    trace_run "$cooldown_name" "$TICK" 0
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}
for c in plain zero on; do gpu_case "$c" 1; done
for c in zero on plain; do gpu_case "$c" 2; done
for c in on plain zero; do gpu_case "$c" 3; done
plain_ns=$(median3 "$OUT/gpu-plain.medians")
zero_ns=$(median3 "$OUT/gpu-zero.medians")
on_ns=$(median3 "$OUT/gpu-on.medians")

{
    printf '%s=%s\n' zero_vs_plain_ae "$zero_vs_plain_ae" pinned_ae "$pinned_ae" face_rmse "$face_rmse" \
        lit_face_mean "$lit_face_mean" zero_face_mean "$zero_face_mean" one_code "$one_code" \
        moving_ae_1 "$moving_ae_1" moving_ae_2 "$moving_ae_2" moving_ae_3 "$moving_ae_3" \
        within_bucket_min_ae "$within_min" across_bucket_ae "$across_ae" \
        redraws_20s_plain "$rate_plain" redraws_20s_pinned "$rate_pinned" redraws_20s_4hz "$rate_4hz" \
        redraws_20s_reduced "$rate_reduced" redraws_20s_off "$rate_off" \
        gpu_plain_ms "$(ns_to_ms "$plain_ns")" gpu_zero_ms "$(ns_to_ms "$zero_ns")" gpu_on_ms "$(ns_to_ms "$on_ns")" \
        gpu_zero_vs_plain_pct "$(pct_delta "$plain_ns" "$zero_ns")" gpu_on_vs_plain_pct "$(pct_delta "$plain_ns" "$on_ns")"
} | tee "$OUT/metrics.txt"

assert_zero redraws_20s_plain "$rate_plain"
assert_zero redraws_20s_pinned "$rate_pinned"
assert_about redraws_20s_4hz "$rate_4hz" 80 0.25
assert_about redraws_20s_reduced "$rate_reduced" 40 0.25
assert_zero redraws_20s_off "$rate_off"
finish
