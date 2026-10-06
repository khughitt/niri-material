---
id: material-5d6b2c
title: "Resource-aware rendering: know the cost of every visual decision"
status: todo
priority: 1
complexity: high
lane: true
created: 2026-09-11T23:34:15Z
updated: 2026-10-03T16:48:07Z
depends: []
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Make the GPU, CPU and memory cost of material rendering explicit so visual trade-offs can be chosen from measurements. First milestone: finish the remaining settle lifecycle evidence under material-f86183, then run a bounded cost pilot under material-31074f; material-233295 needs no quiet host.

Goal: make the GPU/CPU/memory cost of every material rendering decision explicit, so trade-offs are chosen rather than stumbled into. The aim is not to sacrifice high-quality visuals; it is to know what each decision costs and to find the places where a slight visual change buys a large performance gain.

Children: a concise performance guide, a consistent capture protocol with provenance/environment metadata, explicit per-element estimation and evaluation, and two low-power explorations (skip rendering for invisible windows; inactivity settle mode).

Related: material-265eb0 (idle GPU and power budget), prism-ed6be0 (intermittent slow draws), prism-d54be4 (per-parameter cost estimates for the Prism UI), ops-side observation tooling filed from the same seed.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-12T19:24:30Z (materials-26.04): Complexity high: The goal still includes an unresolved per-element cost model and exploratory visibility, settling, and visual-cost trade-offs; the landed capture tooling supplies measurement infrastructure only.
- 2026-10-02T23:08:32Z (materials-26.04): workstreams: now a lane (tag lane); scope unchanged; brief: docs/notes/2026-10-02-workstreams-brief.md
- 2026-10-02T23:47:23Z (materials-26.04): workstreams: first milestone: remaining settle lifecycle evidence (material-f86183 children), then a bounded cost pilot under material-31074f. material-233295 needs no quiet host.
