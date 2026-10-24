---
id: material-095049
title: Update the optic recipe for the current layered Noise API
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-24T13:48:26Z
updated: 2026-10-24T13:48:33Z
depends: []
tags: [docs]
source: material-10a999
agent: codex
---

Why: curate-docs at bfea33f0 found docs/materials/adding-an-optic.md:12-55 still teaches ResolvedNoise { amount, kind, site }, resolve(Option<Noise>) and g.noise.amount; :132-135 packs three scalar uniforms. Current niri-config/src/material/optics/noise.rs defines four layers, scale, ResolvedNoise.layers and resolve(&[Noise]); src/render_helpers/material/optics/noise.rs packs four vec4 uniforms.

Done: update the worked config and renderer snippets together to the layered representation, or explicitly identify another current scalar optic as the example and explain noise multi-placement separately. Check the recipe against source and existing documentation checks; no new capture is needed. This API correction is separate from material-f6e284 cost-entry documentation.

## Notes

- 2026-10-24T13:48:33Z (materials-26.04): concerns: material-3fcba2 defect — The layered Noise API landed while the optic recipe's worked config and uniform examples retained the single-node representation.
