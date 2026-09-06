---
id: material-a1d4bf
title: Document the render pass order
status: done
priority: 2
size: s
owner: materials-26.04
created: 2026-09-06T00:38:34Z
updated: 2026-09-06T00:43:34Z
depends: []
parent: material-5b3107
tags: [docs]
---

Write a concise overview of the material render pipeline: each pass in order, its inputs (what it samples) and outputs, and where noise, saturation, blur, bevel, jelly motion, and the ring of light sit. Put it in docs/materials (the root README is upstream niri's), link it from docs/materials/README.md and material-config.md, and add a one-line pointer in AGENTS.md so agents read it before touching the shaders. Cite the source: render_helpers/material.rs, framebuffer_effect.rs, background_effect.rs, and shaders/*.frag.

## Notes

- 2026-09-06T00:43:34Z (materials-26.04): docs/materials/render-pipeline.md: the frame top down, the material element's inputs, the 13 shader stages with their parameters, what the order settles, and the native-to-Prism parameter map; linked from the materials README, material-config.md, and AGENTS.md
