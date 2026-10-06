---
id: material-79fb49
title: "Warp estimator: a local affine-warp model for rounded corners and distortion"
status: shelved
priority: 3
created: 2026-10-06T17:24:43Z
updated: 2026-10-06T19:24:25Z
depends: []
parent: material-49871a
tags: [harness]
agent: claude-code/claude-opus-5-5
---

material-bb8480's calibrated estimator fits one translation (plus gain and offset) per window. That holds on a straight chamfer edge, where the shader's displacement is constant across the strip, but a field that varies inside the window fails its residual check: the smooth 3/1.5 px test warp kept only 24% of pixels confident at a 7 px window (errors up to 0.31 px on the rest). Rounded corners and any distortion > 0 produce exactly that kind of field. Wake this only when material-343f27 needs measurements there: extend the Gauss-Newton refinement in docs/materials/scripts/warp-calibration.py to a six-parameter affine warp, add a corner-shaped and a distortion-like known field to the calibration, and report confident share and error as bb8480 does. Out of scope until then: straight-edge chamfer measurement needs no change.

## Notes

- 2026-10-06T19:24:24Z (materials-26.04): shelved: A rendered bending measurement needs rounded corners or nonzero distortion, and the calibrated translation model fails its stated residual or confidence limits in those regions.
- 2026-10-06T19:24:24Z (materials-26.04): scope: shelved; straight-edge flat-field validation needs no affine model; wake only when a concrete rendered measurement needs varying displacement within the matching window; brief: docs/notes/2026-09-29-glass-measurement-brief.md
