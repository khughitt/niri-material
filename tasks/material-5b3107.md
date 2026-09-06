---
id: material-5b3107
title: "Render pass order: what the glass slab refracts"
status: todo
priority: 2
size: m
created: 2026-09-06T00:38:34Z
updated: 2026-09-06T00:38:34Z
depends: []
tags: [rendering, design]
---

Several open questions are one architectural question: in what order the passes run (backdrop blur, glass refraction and bevel, ring of light, noise and saturation postprocess) and therefore what the slab refracts and what lands on top of it. Noise on the glass edges (material-f8b6e9) and the ring embedded inside the glass (material-92edaf) both hinge on it. Document the current order first, then decide the target order once for both.
