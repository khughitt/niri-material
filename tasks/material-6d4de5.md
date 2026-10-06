---
id: material-6d4de5
title: "Revisit the dynamics: drag, move, focus, flex, ripple"
status: idea
priority: 1
created: 2026-09-06T00:32:54Z
updated: 2026-10-06T18:41:37Z
depends: [prism-66b025]
parent: material-53f873
tags: [rendering, dynamics]
---

Remaining outcome: decide whether the existing bevel-dependent flex cap, inner-face-only shear, or ior-dependent ripple needs a redesign after the accepted drag follow-lag implementation. The absent drag stimulus was fixed by material-4354cf (f8a890d3); it is no longer a renderer bug to fix here.

Done for a future design: identify a reproducible unacceptable deformation or ripple appearance on current code, name the desired change, preserve finite settling and opaque client pixels, and define deterministic and owner-reviewed visual checks before changing the cap or optical model. Where to look: docs/notes/2026-09-29-material-dynamics-brief.md; docs/specs/2026-10-01-drag-follow-lag-design.md; src/layout/tile.rs::material_dynamics/motion_residual; src/render_helpers/material/mod.rs::jelly_state; src/render_helpers/shaders/material/prelude.frag::slabSurface and main.frag's ripple perturbation. The existing motion sweep establishes response, not a preferred gain or slab deformation. This remains an idea while that visual target is unresolved.

## Original captured request

Reassess how the glass responds to drag, move, and focus changes. Prism reports that the Flex and Ripple sliders have no visible effect; if the values reach the renderer, the jelly math (render_helpers/material.rs) or its gating may need to change.

## Notes

- 2026-09-06T00:47:17Z (materials-26.04): From prism-66b025 (2026-09-05): jelly-flex has no visible effect because max_flex = 0.25 * bevel_depth (3.75 px at bevel 15) caps the inner-face shear, the shear reaches the shader only through mat_jelly_move on the chamfer's inner edge, and animation_residual excludes the interactive-move grab offset, so a drag shows no jelly at all. jelly-ripple tilts the normal by activity * ripple (activity = tanh(residual / half-diagonal)); at ior 1.2 and thickness 26.3 the tap displacement peaks near 1 px. Levers to revisit: the 0.25 cap and its tie to bevel, whether the whole slab (not just the inner edge) should shear, including the grab offset or its velocity during drag, and scaling ripple so its displacement is independent of ior.
- 2026-09-11T00:47:53Z (materials-26.04): Reparented under the dynamics sprint goal material-53f873 and raised to p1 (quick-add from mindful:thought:3f94e656b70f4e5585c1cb60c166e4da).
- 2026-09-11T10:04:41Z (material-36e968): Motion sweep evidence now in docs/materials/2026-09-11-jelly-motion-sweep.md: installed7526af1d RTX3070 column moves resolve all sampled neighbor steps (flex0/.004/.01/.02, ripple0/.06/.25/.5) above repeat variation; all16 bursts settle pixel-identically. This establishes working response to native animation residuals in the pinned scene, not interactive-pointer drag behavior or appropriate Prism ranges. Reuse experiments results/jelly-motion fixture for redesign comparisons.
- 2026-09-29T22:44:56Z (materials-26.04): scope: briefed; native column-move response is measured; interactive drag and release need a current baseline before choosing deformation or gain changes; brief: docs/notes/2026-09-29-material-dynamics-brief.md
- 2026-09-30T09:51:39Z (material-b3ce14): material-b3ce14 finding: during interactive drag and hold the jelly residual is exactly 0 (tile pinned to the pointer, grab offset excluded), so flex and ripple are dead however fast the pointer moves; glass flexes only on lift (tiled 1.46 px, floating 0.18 px) and tiled release (2.96 px, capped at max flex). Smallest missing contract: a follow-lag residual (a critically damped point chasing the pointer-anchored render location on the window-movement spring) fed to jelly during Moving. Owner choices listed in the dynamics brief.
- 2026-09-30T10:00:29Z (materials-26.04): Drag stimulus design split out to material-4354cf; this idea keeps the max-flex cap, whole-slab shear and ior-independent ripple levers.
- 2026-10-06T18:41:36Z (materials-26.04): scope: briefed; retained cap, whole-slab shear and ior-independent ripple questions; drag follow-lag is implemented and accepted, so a concrete residual visual gap must precede redesign; brief: docs/notes/2026-09-29-material-dynamics-brief.md
