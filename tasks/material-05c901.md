---
id: material-05c901
title: Layered 3d texture embedded in the glass
status: idea
priority: 2
reviewed: true
reviewed_at: 2026-10-20T21:43:07Z
created: 2026-09-11T00:47:47Z
updated: 2026-10-20T21:43:07Z
depends: [material-56f91a]
parent: material-3aa1f2
tags: [quick-add, rendering]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
---

Experiment with an embedded material layer inside the glass (stone, slate, paper, ...) as a reading background that still complements the glass: the texture sits behind or within the slab and is refracted like the backdrop. Distinct from stacked noise layers (material-3fcba2); material-5b3107 (render pass order) decides what the slab refracts and so where such a layer would sit. Later: visual identity expressed as texture.

Source: mindful:thought:3f94e656b70f4e5585c1cb60c166e4da

## Current scope (2026-10-20)

The embedded reading-layer question remains open. Current samplers cover the window and prefiltered backgrounds, not a material-local imported image. Noise layers have landed, but do not settle anchoring, depth, alpha or resource lifetime. Reuse material-56f91a for one frozen-pixel proof proposal and minimum resource requirements; owner visual judgement follows that proposal.

Read [ref:brief] for the existing alternatives, constraints and bounded handoff.

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-material-texture-composition-brief.md — existing scope handoff and follow-up boundaries

## Notes

- 2026-09-11T00:52:15Z (materials-26.04): Counterpart in mindful: mind6-5c0193 (identity-bearing textures shared between mindful structure identity and the glass texture layer).
- 2026-10-06T20:03:51Z (materials-26.04): scope: briefed; backdrop textures are already refracted, but material-local embedded texture coordinates and composition are undefined; investigate the smallest reading-layer proof before a new sampler or stack; brief: docs/notes/2026-10-06-material-texture-composition-brief.md
- 2026-10-20T21:43:07Z (materials-26.04): scope: briefed; revalidated against 52c14d4d; reused material-56f91a and recorded the missing dependency; The embedded reading-layer question remains open. Current samplers cover the window and prefiltered backgrounds, not a material-local imported image. Noise layers have landed, but do not settle anchoring, depth, alpha or resource lifetime. Reuse material-56f91a for one frozen-pixel proof proposal and minimum resource requirements; owner visual judgement follows that proposal.; brief: docs/notes/2026-10-06-material-texture-composition-brief.md
