---
id: material-5b3107
title: "Render pass order: what the glass slab refracts"
status: doing
priority: 2
size: m
complexity: high
owner: material-5b3107
created: 2026-09-06T00:38:34Z
updated: 2026-09-12T19:58:07Z
started: 2026-09-12T19:55:05Z
depends: []
tags: [rendering, design]
spec: docs/specs/2026-09-12-material-render-order-design.md
---

Several open questions are one architectural question: in what order the passes run (backdrop blur, glass refraction and bevel, ring of light, noise and saturation postprocess) and therefore what the slab refracts and what lands on top of it. Noise on the glass edges (material-f8b6e9) and the ring embedded inside the glass (material-92edaf) both hinge on it. Document the current order first, then decide the target order once for both.

## Notes

- 2026-09-12T19:24:33Z (materials-26.04): Complexity high: The current pass order is documented in render-pipeline.md, but the target order for edge noise and an embedded ring remains an architectural decision shared by the two open exploratory children.
- 2026-09-12T19:58:07Z (material-5b3107): Drafted depth-ordered hooks for review. Source review found whole-bevel zero grain, a strict two-width ring cutoff, and unchanged rendered brightness are not guaranteed; spec proposes measurable replacements and defines depth from back to front.
- 2026-09-12T19:58:07Z (material-5b3107): parked (waiting on user, review): Review the written spec, especially corrected grain/reach gates and lookup-depth convention; then write the implementation plan.
