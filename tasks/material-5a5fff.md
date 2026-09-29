---
id: material-5a5fff
title: Interpolate glass parameters across a focus material swap
status: idea
priority: 2
created: 2026-09-05T01:10:50Z
updated: 2026-09-29T22:44:56Z
depends: []
parent: material-53f873
tags: [material]
---

If the is-focused material swap reads as a visual pop, animate between the two resolved parameter sets using niri's existing animation/spring machinery rather than a hard cut. Only worth scoping after material-cb348e lands and the pop is observed.

## Notes

- 2026-09-05T01:46:44Z (materials-26.04): apply_resolved in src/render_helpers/material.rs replaces MaterialState on a name change, so jelly residuals restart at the swap; interpolation must keep the state and blend configs instead.
- 2026-09-08T22:56:59Z (materials-26.04): Prism now splits 12 optics by focus state (prism-a4ef9a), so a swap can change tint, refraction, and depth as well as the surface optics - more for an interpolation to cover. Per-state slab geometry (bevel, offsets) is still deliberately withheld on the Prism side precisely because the swap is a hard cut; interpolation here is what would unblock it.
- 2026-09-29T22:44:56Z (materials-26.04): scope: briefed; name swaps replace the offscreen, element identity and seed; layout residuals remain on Tile; capture must establish whether a visible pop warrants interpolation; brief: docs/notes/2026-09-29-material-dynamics-brief.md
