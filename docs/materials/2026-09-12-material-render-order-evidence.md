# Material render order: execution evidence

**Status:** preparation only. The plan was approved for inline execution,
but the first capture preflight refused the host. No baseline binary build,
shader change, material capture or frame-cost measurement has run. Neither
implementation child is complete.

**Tasks:** `material-5b3107`, `material-f8b6e9`; Task 2 (`material-92edaf`)
remains dependent on Task 1.

## Capture readiness, 2026-09-12

Retained artifact: `$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.pCbsOC/capture.json`.
SHA-256: `e56e846026688734b097b2a4e9113e6fe375610e2b220a5a95f0c0a7341c59bb`.
The record identifies host `titan`, run start `2026-09-12T16:32:19-04:00`,
the headless lane and Task 1. The separate readiness probe names the planned
smoke as its fixture; that smoke has not been created or launched yet.

The command used the default 20-second sample and unmodified thresholds:

```bash
python3 tools/capture-meta preflight "$run_dir" --lane headless \
    --task material-f8b6e9 --fixture glass-render-order-smoke.sh \
    --owner-pid $$ --tool weston --tool kitty --tool tracy=0.13.1
```

Exit status was **1, preflight refused**, with these observations:

| Measure | Observed | Required |
| --- | --- | --- |
| CPU busy | 16.6% | <=10% |
| Load average | 5.53 | <=2 |
| GPU utilization | 26% | <=5% |
| GPU performance states | P3, P5, P8 | P8 throughout |
| GPU power interquartile range | 7.463 W | <=1 W |
| Compute clients | BitwigStudio | None |

The capture lock was released. No client was terminated and no threshold
was changed. **This is not the required old-shader additive failure.**
Resume with a fresh readiness record after the host is quiet, snapshot the
reviewed baseline, and obtain the actual additive regression before shader
implementation. Task 2 must still retain a positive wide-core face-strip
test; a reach upper bound alone would accept the old mask.

## Offline preparation

Added `glass-render-order-metrics.py` beside the existing smoke scripts:

- `grain`: signed sRGB luma residual standard deviation.
- `additive`: four-image linear-light comparison with per-code quantization
  intervals, clipping counts and minimum informative-sample requirements.
- Exit 0 means a reported grain measure or passing additive gate; exit 1
  means an informative additive comparison failed; exit 2 means invalid or
  uninformative input. These statuses are separate from capture preflight.

The tests use signed positive/negative residuals, unchanged and deliberately
changed additive light, and real ImageMagick decoding of synthetic PPM
images. They distinguish passing comparisons, measured failure, clipping,
invalid crops and corrupt image files. ImageMagick-dependent checks skip
on machines without that capture dependency; they ran on this host.

Validation command:

```bash
just --set fast_cmd 'python3 -m unittest discover -s tools -p test_glass_render_order_metrics.py' test-fast
```

Both tests passed. The earlier test run failed because the metric module
did not exist yet; it is only tooling-development history, not evidence
about the shader. No grain-preservation, additive-invariance, opaque-pixel,
reach, motion or cost acceptance result is claimed here.
