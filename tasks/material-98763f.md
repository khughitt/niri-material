---
id: material-98763f
title: Validate rendered flat-field isolation for straight-edge glass bending
status: todo
priority: 2
size: m
complexity: high
process: direct
created: 2026-10-06T19:24:49Z
updated: 2026-10-06T19:24:49Z
depends: []
parent: material-49871a
tags: [harness]
source: "docs/notes/2026-09-29-glass-measurement-brief.md#rendered-flat-field"
agent: codex
---

Question: Can paired uniform-backdrop renders isolate straight-edge glass displacement from attenuation and Fresnel in the real shader, within the calibrated translation estimator's supported range?
Where to start: docs/notes/2026-09-29-glass-measurement-brief.md; docs/materials/2026-10-06-glass-warp-calibration-evidence.md, especially What a render must add; src/tests/ring_pair.rs::render_at/set_time and the test client's shm/layer-surface helpers; docs/materials/scripts/warp-calibration.py::Field/estimate/refine/flat_field/correct; tools/test_warp_calibration.py; src/render_helpers/shaders/material/main.frag and prelude.frag::tap.
Bound: Reuse the in-process surfaceless renderer for one fixed straight chamfer, with a geometry mask leaving full-coverage strip interior at least a complete 7 px matching window wide. Render the same geometry, material identity/seed and frozen clock over two distinct uniform linear-light inputs plus the calibrated grayscale textured backdrop; repeat for a small pair of nonconfounded optical settings whose expected shifts fit the calibrated search (for example ior 1.24 and 1.5 at thickness 20, bevel 12). Disable ring, aurora, noise, saturation changes, blur, roughness, smear/aberration, distortion and motion; exclude opaque-client pixels, corners, antialiased boundaries, clipped samples and near-zero recovered gain. Decode sRGB to linear per channel before att=(R2-R1)/(g2-g1), spec=R1-att*g1 and corrected=(R-spec)/att. Use actual uploaded/quantized backdrop values and account for coordinate/texture filtering. Do not use ior 1 or thickness 0 as an unconfounded rendered photometric control. Keep the experiment to a small interior ROI; run the smallest rendered case through its verdict before expanding. No compositor production changes, affine fit, live desktop, timed/power capture or full parameter sweep.
Expected result: Save reproducible source/renderer/geometry/input provenance, matched-state repeat checks, corrected displacement orientation/magnitude, confidence and residual-error report, with independent straight-edge Snell expectations and a photometric-only negative control. Adopt or reject the rendered measurement with an explicit valid mask/range; do not transfer the synthetic 0.02 px bound to the real renderer without measuring it, or call unsupported pixels zero displacement. Record findings on this task and in the brief. Use existing Python image support and the calibrated estimator rather than adding a library or a generic capture interface.
Ideas it wakes: On completion, run tasks note on material-343f27 with the finding, in the same commit as this result. If the result demonstrates that corners or distortion are actually required, report the evidence and use tasks unshelve material-79fb49, then add the finding note; do not wake it speculatively.
