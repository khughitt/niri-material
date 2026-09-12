---
id: material-f8b6e9
title: "Behind hook: noise and saturation"
status: doing
priority: 2
size: m
complexity: high
owner: material-5b3107
created: 2026-09-06T00:32:53Z
updated: 2026-09-12T21:10:25Z
started: 2026-09-12T20:31:50Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, noise]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
step: "Task 1: Behind hook: noise and saturation"
---

Implement the reviewed behind hook for noise and saturation as Task 1 of the render-order plan. Preserve signed sRGB formulas, neutral branches, inheritance and opaque bypass; isolate additive-light effects with quantization-aware comparisons. Transmitted chamfer grain is accepted without a mask. Record face/bevel grain and frame cost and update stage/spec docs in the same implementation commit. Plan approved for inline sequential execution; capture readiness and predecessor completion still gate implementation.

## Notes

- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review the two-step render-order implementation plan before starting Task 1.
- 2026-09-12T20:35:43Z (material-5b3107): Execution preflight refused: CPU 16.6% >10%, load1 5.53 >2, GPU 26% >5%, P3/P5/P8, power IQR 7.463 W >1 W, compute client BitwigStudio. Retained render-order-readiness.pCbsOC. This is NOT the old-build additive regression failure. Only offline grain/additive metric preparation is complete; no shader changes or baseline build.
- 2026-09-12T20:35:43Z (material-5b3107): parked (waiting on user, environment): On a quiet host, rerun default capture preflight, snapshot baseline 522a09fe binaries, finish the smoke and retain its actual additive failure against the old shader before production edits. Offline metric CLI and synthetic tests are prepared.
- 2026-09-12T21:06:29Z (material-5b3107): Retry after user closed Bitwig window: preflight render-order-readiness.03bDro still refused (CPU 18.3%, load 7.31, GPU 41%, P5, BitwigStudio compute client). Independent nvidia-smi query confirms live BitwigStudio PID 2467300 using 293 MiB; ps confirms its audio engine PID 2467825. No process terminated; no shader edits or regression capture.
- 2026-09-12T21:06:29Z (material-5b3107): parked (waiting on user, environment): BitwigStudio PID 2467300 and its audio engine remain active. Resume after the app fully exits and default preflight passes; then obtain the old-build additive failure before shader edits.
- 2026-09-12T21:10:25Z (material-5b3107): Confirmed Bitwig exited: no compute clients. Retry ZVFSho refused (CPU 7.4%, load 5.06, GPU 32%, variable P-states/power). After settling, TFm260 passes CPU 6.3%, load 1.91, power IQR 0.537 W; remaining refusals are GPU 23.5% >5% and P5/P8 instead of P8 throughout. Separate pmon sampling observed niri/Noctalia GPU activity. No threshold override or shader changes.
- 2026-09-12T21:10:25Z (material-5b3107): parked (waiting on user, environment): Bitwig is gone; wait for desktop GPU activity to settle below default thresholds (latest TFm260: 23.5%, P5/P8). Then resume baseline and actual old-shader additive capture before implementation.
