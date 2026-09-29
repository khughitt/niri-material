---
id: material-31074f
title: Explicit performance estimation and evaluation per material element
status: todo
priority: 2
size: m
complexity: high
created: 2026-09-11T23:34:15Z
updated: 2026-09-29T22:56:49Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Using the capture protocol, measure the cost of each render pass and each exposed parameter across its range, including the main interactions (e.g. frosted backdrop with high distortion detail) and the focused/unfocused split. Produce an estimate that can evaluate a whole preset's cost. This is the measurement side; prism-d54be4 is the consumer that wants these estimates to shape exposed ranges and warn on demanding settings in the Noctalia panel.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-12T19:24:29Z (materials-26.04): Complexity high: The capture protocol establishes provenance and quietness, but the parameter interaction matrix, whole-preset cost estimator, and long-run drift treatment remain unresolved measurement/modeling work.
- 2026-09-29T21:44:30Z (materials-26.04): Scope handoff: docs/notes/2026-09-29-resource-aware-rendering-brief.md. Reuse this existing per-element and focused/unfocused measurement task for material-2ebf2c, material-a0cbb0 and material-a91346; no duplicate cost study. Alongside measured cost, record the repeated-capture noise floor, which inactive settings save whole-scene work (including shared blur/prefilter caches), and what quantities a continuous consumer actually needs before recommending GPU queries or quality tiers. Ideas it wakes: On completion, run tasks note on material-2ebf2c, material-a0cbb0 and material-a91346 with the relevant findings, in the same commit as this result, and update the brief.
- 2026-09-29T22:56:49Z (materials-26.04): Additional scope handoff: docs/notes/2026-09-29-adaptive-materials-brief.md. Reuse the existing interaction measurements for material-0c7eed; report which joint parameter effects are supported by repeatable evidence and which remain unknown. Cost sensitivity does not establish visual preference or an innate good region. Ideas it wakes: On completion, run tasks note on material-0c7eed with the finding in the same commit as this result, alongside the previously named waiting ideas, and update the adaptive-materials brief.
