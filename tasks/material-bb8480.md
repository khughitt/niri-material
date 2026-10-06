---
id: material-bb8480
title: Calibrate a glass warp measurement against photometric controls
status: done
priority: 2
size: s
complexity: high
process: direct
owner: material-bb8480
created: 2026-09-29T22:38:51Z
updated: 2026-10-06T17:34:43Z
started: 2026-10-06T17:02:02Z
completed: 2026-10-06T17:34:43Z
depends: []
parent: material-49871a
tags: [harness]
source: "docs/notes/2026-09-29-glass-measurement-brief.md#warp-calibration"
model: claude-opus-5-5
agent: codex
---

Question: Can a deterministic aperiodic backdrop and local displacement estimator recover known spatially varying refraction-like warps while rejecting brightness-only changes, at an error low enough to test the glass shader?
Where to start: docs/materials/scripts/glass-parameter-sweep.sh::make_backdrop and its table stage; docs/materials/2026-09-06-glass-parameter-sweep-evidence.md (Bending is not measured here and Instrument caveat); src/render_helpers/shaders/material/prelude.frag::tap; docs/notes/2026-09-29-glass-measurement-brief.md. Related cost work material-31074f and perceptual-difference idea material-2ebf2c do not establish displacement.
Bound: Offline calibration only: one seeded aperiodic field compared with the current 20 px grid, a no-warp repeat, a known rigid shift (including one grid period), a known spatially varying warp across a chamfer-sized ROI, and photometric-only and combined warp/photometric controls. Record estimator error, ambiguous regions and the supported displacement range; aperiodicity alone does not make RMSE monotonic or prove bending. Prefer available image tools; stop with an explicit failure if these cases cannot establish a usable instrument. No new runtime instrumentation, dependency selection by name, full GPU sweep or perceptual-quality model.
Expected result: A small reproducible calibration artifact, results and an adopt/reject recommendation on this task and in the brief. State what additional rendered control would distinguish actual shader bending from Fresnel/attenuation before promoting material-343f27. Recommend whether material-8867aa needs a separate backdrop change or can reuse this artifact; preserve old sweep comparability with named backdrop provenance.
Ideas it wakes: On completion, run tasks note on material-343f27 and material-8867aa with the findings, in the same commit as this result, and update docs/notes/2026-09-29-glass-measurement-brief.md.

## Notes

- 2026-10-06T17:02:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T17:02:12Z (material-bb8480): resumed
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T17:34:43Z (material-bb8480): done
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T17:34:43Z (material-bb8480): Adopted for straight chamfer edges with a geometry mask: ZNCC + Gauss-Newton on the seeded aperiodic backdrop (docs/materials/scripts/warp-calibration.py, 7 px window) reads chamfer shifts of 1-20 px to 0.02 px in the strip interior, rejects photometric-only change, and trusts no pixel >1 px wrong; the 20 px grid refuses every pixel and its RMSE is 0 for 1/3/20 px shifts. Rendered control still needed: a flat-field pair over two uniform backdrops divides out attenuation and the Fresnel glint (ior 1 and thickness 0 are confounded). Evidence 2026-10-06-glass-warp-calibration-evidence.md; brief updated; notes on material-343f27 and material-8867aa; affine model filed as material-79fb49.
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
