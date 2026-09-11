---
id: material-6d4de5
title: "Revisit the dynamics: drag, move, focus, flex, ripple"
status: idea
priority: 1
created: 2026-09-06T00:32:54Z
updated: 2026-09-11T10:04:41Z
depends: [prism-66b025]
parent: material-53f873
tags: [rendering, dynamics]
---

Reassess how the glass responds to drag, move, and focus changes. Prism reports that the Flex and Ripple sliders have no visible effect; if the values reach the renderer, the jelly math (render_helpers/material.rs) or its gating may need to change.

## Notes

- 2026-09-06T00:47:17Z (materials-26.04): From prism-66b025 (2026-09-05): jelly-flex has no visible effect because max_flex = 0.25 * bevel_depth (3.75 px at bevel 15) caps the inner-face shear, the shear reaches the shader only through mat_jelly_move on the chamfer's inner edge, and animation_residual excludes the interactive-move grab offset, so a drag shows no jelly at all. jelly-ripple tilts the normal by activity * ripple (activity = tanh(residual / half-diagonal)); at ior 1.2 and thickness 26.3 the tap displacement peaks near 1 px. Levers to revisit: the 0.25 cap and its tie to bevel, whether the whole slab (not just the inner edge) should shear, including the grab offset or its velocity during drag, and scaling ripple so its displacement is independent of ior.
- 2026-09-11T00:47:53Z (materials-26.04): Reparented under the dynamics sprint goal material-53f873 and raised to p1 (quick-add from mindful:thought:3f94e656b70f4e5585c1cb60c166e4da).
- 2026-09-11T10:04:41Z (material-36e968): Motion sweep evidence now in docs/materials/2026-09-11-jelly-motion-sweep.md: installed7526af1d RTX3070 column moves resolve all sampled neighbor steps (flex0/.004/.01/.02, ripple0/.06/.25/.5) above repeat variation; all16 bursts settle pixel-identically. This establishes working response to native animation residuals in the pinned scene, not interactive-pointer drag behavior or appropriate Prism ranges. Reuse experiments results/jelly-motion fixture for redesign comparisons.
