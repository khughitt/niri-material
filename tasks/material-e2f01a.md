---
id: material-e2f01a
title: "Render chain as a permutable, learnable sequence"
status: shelved
priority: 2
created: 2026-09-11T23:34:15Z
updated: 2026-09-29T22:56:49Z
depends: []
parent: material-0f225e
tags: [quick-add, adaptive, rendering]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Follow-up to the adaptive-glass goal filed from the same seed. Treat the order in which effects are applied as a sequence that can be permuted within implementation constraints, let users modulate it (or randomly modulate to collect data), or optimize a data-driven objective such as final-stage dynamic range. Related: material-5b3107 (render pass order design), material-a1d4bf (pass order documented).

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-11T23:39:09Z (materials-26.04): Implementation prerequisites live in prism: prism-a03862 (ordered device chain) and prism-542904 (reorderable devices once the shader honours an order). This idea supplies the learning side once order is permutable.
- 2026-09-29T22:56:49Z (materials-26.04): shelved: A renderer-supported, constrained effect-order mechanism exists and a reproducible visual or cost objective justifies comparing legal orders; verify Prism rack/reordering status before unshelving.
- 2026-09-29T22:56:49Z (materials-26.04): scope: shelved; shader hook calls and physical stages are fixed; reconsider only with a supported reorder mechanism and a measured objective; brief: docs/notes/2026-09-29-adaptive-materials-brief.md
