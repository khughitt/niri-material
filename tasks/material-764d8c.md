---
id: material-764d8c
title: Compositional profile model
status: idea
priority: 2
created: 2026-09-11T23:34:15Z
updated: 2026-10-02T16:03:37Z
depends: []
parent: material-0f225e
tags: [quick-add, adaptive, cross-project]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Profiles as composable values: a base profile plus familiar and niri layers, algebraic-data-type style. Enumerate the composition operators (override, blend, constrain) and where they conflict; decide what a named profile shipped with material is in this model. Related: material-9be53d (named animation profiles).

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-11T23:39:09Z (materials-26.04): material-930c55 (familiar bridge) is the concrete familiar layer this model needs to compose; ops-fcdf59 shows the same base+layer question on the keyboard side.
- 2026-09-29T22:56:49Z (materials-26.04): scope: briefed; named material/response selection and Familiar signals already compose at distinct stages; reuse material-8e3b73 to identify a concrete missing composition behavior; brief: docs/notes/2026-09-29-adaptive-materials-brief.md
- 2026-10-02T16:03:37Z (material-8e3b73): material-8e3b73 (2026-10-02): owner kept the hard cut on focus material swaps. Seed replacement is the largest part of the swap's own step; focus selects a definition (hard cut, fresh MaterialState), a named response (in place, ring/signal only) or folded signal accents. See the Focus swap finding in docs/notes/2026-09-29-material-dynamics-brief.md.
