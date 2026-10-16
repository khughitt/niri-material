---
id: material-31074f
title: Explicit performance estimation and evaluation per material element
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-11T23:34:15Z
updated: 2026-10-15T18:42:09Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Why: know the cost of each render pass and each exposed parameter across its range, including the main interactions (e.g. frosted backdrop with high distortion detail) and the focused/unfocused split, and produce an estimate that can evaluate a whole preset's cost. This is the measurement side of lane material-5d6b2c; prism-d54be4 (shelved) is the consumer that wants these estimates to shape exposed ranges and warn on demanding settings in the Noctalia panel. Source: mindful:thought:a476e6bcd1fd4297b70824758235d821. Handoffs: docs/notes/2026-09-29-resource-aware-rendering-brief.md and docs/notes/2026-09-29-adaptive-materials-brief.md (notes below). Alongside measured cost, it records the repeated-capture noise floor, which inactive settings save whole-scene work (including shared blur/prefilter caches), what quantities a continuous consumer actually needs before recommending GPU queries or quality tiers, and which joint parameter effects repeatable evidence supports. Cost sensitivity does not establish visual preference.

Established 2026-10-15 (2dd3b9fa; do not re-derive): no cost work has started (no commit cites this task). Tracy GPU spans exist for the material draw (material/mod.rs ~:963), blur, prefilter and grain; the sharp backdrop render and window offscreen have CPU spans only. Existing numbers are scattered single runs on one RTX 3070: backdrop chain ~1.85 ms per damage at 1280x720 (blur 0.45, prefilter 0.27, grain 0.09), the noise-layers table and its B-spline idle +0.034 ms, aurora +0.85 W at 4 Hz / +0.68 W at 2 Hz, settled glass zero draws; NVIDIA timer medians step at 1024 ns. performance.md §7 parameter costs are code-derived, not measured. Neutral optics are not compiled out, so per-stage cost exists only as deltas in whole-draw time.

Decomposed:
- material-0df22c: design the capture matrix and whole-preset estimator, and prepare the run (planned, no host).
- material-864f87: the quiet pilot and capture matrix (quiet, nested), after material-0df22c.

Done (this task's closeout): the estimate and its uncertainty recorded where the design says, both briefs updated, and the wake notes run.
Ideas it wakes: on completion, run tasks note --no-reviewed on material-2ebf2c, material-a0cbb0, material-a91346 and material-0c7eed with the relevant finding, and tasks note on material-f6e284 (the cost-entry template), in the same commit as this result.

## Notes

- 2026-09-12T19:24:29Z (materials-26.04): Complexity high: The capture protocol establishes provenance and quietness, but the parameter interaction matrix, whole-preset cost estimator, and long-run drift treatment remain unresolved measurement/modeling work.
- 2026-09-29T21:44:30Z (materials-26.04): Scope handoff: docs/notes/2026-09-29-resource-aware-rendering-brief.md. Reuse this existing per-element and focused/unfocused measurement task for material-2ebf2c, material-a0cbb0 and material-a91346; no duplicate cost study. Alongside measured cost, record the repeated-capture noise floor, which inactive settings save whole-scene work (including shared blur/prefilter caches), and what quantities a continuous consumer actually needs before recommending GPU queries or quality tiers. Ideas it wakes: On completion, run tasks note on material-2ebf2c, material-a0cbb0 and material-a91346 with the relevant findings, in the same commit as this result, and update the brief.
- 2026-09-29T22:56:49Z (materials-26.04): Additional scope handoff: docs/notes/2026-09-29-adaptive-materials-brief.md. Reuse the existing interaction measurements for material-0c7eed; report which joint parameter effects are supported by repeatable evidence and which remain unknown. Cost sensitivity does not establish visual preference or an innate good region. Ideas it wakes: On completion, run tasks note on material-0c7eed with the finding in the same commit as this result, alongside the previously named waiting ideas, and update the adaptive-materials brief.
- 2026-10-02T23:47:23Z (materials-26.04): workstreams: when taken up, split preparation (capture matrix, script, pilot definition; no host) from the quiet measurement run, so the preparation is not held behind the quiet queue. Preset estimates must account for shared blur/prefilter caches and interactions, not sum per-effect prices.
- 2026-10-15T18:42:09Z (materials-26.04): scope: rescoped; premises checked at 2dd3b9fa: capture protocol, Tracy GPU spans and cost-script pattern hold, no work started, no per-parameter sweep or noise floor measured, no whole-preset estimator design anywhere (only 'not summed per-effect prices'); decomposed per the 2026-10-02 note into material-0df22c (design + no-host prep, planned, high) and material-864f87 (quiet pilot + matrix, quiet+nested); quiet moved to the run; this task is the closeout (s, mid, direct); wake edges added from material-2ebf2c, material-a0cbb0, material-a91346, material-0c7eed; priority unchanged
