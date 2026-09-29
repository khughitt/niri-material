---
id: material-bb8480
title: Calibrate a glass warp measurement against photometric controls
status: todo
priority: 2
size: s
complexity: high
process: direct
created: 2026-09-29T22:38:51Z
updated: 2026-09-29T22:38:51Z
depends: []
parent: material-49871a
tags: [harness]
source: "docs/notes/2026-09-29-glass-measurement-brief.md#warp-calibration"
agent: codex
---

Question: Can a deterministic aperiodic backdrop and local displacement estimator recover known spatially varying refraction-like warps while rejecting brightness-only changes, at an error low enough to test the glass shader?
Where to start: docs/materials/scripts/glass-parameter-sweep.sh::make_backdrop and its table stage; docs/materials/2026-09-06-glass-parameter-sweep-evidence.md (Bending is not measured here and Instrument caveat); src/render_helpers/shaders/material/prelude.frag::tap; docs/notes/2026-09-29-glass-measurement-brief.md. Related cost work material-31074f and perceptual-difference idea material-2ebf2c do not establish displacement.
Bound: Offline calibration only: one seeded aperiodic field compared with the current 20 px grid, a no-warp repeat, a known rigid shift (including one grid period), a known spatially varying warp across a chamfer-sized ROI, and photometric-only and combined warp/photometric controls. Record estimator error, ambiguous regions and the supported displacement range; aperiodicity alone does not make RMSE monotonic or prove bending. Prefer available image tools; stop with an explicit failure if these cases cannot establish a usable instrument. No new runtime instrumentation, dependency selection by name, full GPU sweep or perceptual-quality model.
Expected result: A small reproducible calibration artifact, results and an adopt/reject recommendation on this task and in the brief. State what additional rendered control would distinguish actual shader bending from Fresnel/attenuation before promoting material-343f27. Recommend whether material-8867aa needs a separate backdrop change or can reuse this artifact; preserve old sweep comparability with named backdrop provenance.
Ideas it wakes: On completion, run tasks note on material-343f27 and material-8867aa with the findings, in the same commit as this result, and update docs/notes/2026-09-29-glass-measurement-brief.md.
