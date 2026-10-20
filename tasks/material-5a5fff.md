---
id: material-5a5fff
title: Interpolate glass parameters across a focus material swap
status: dropped
priority: 2
created: 2026-09-05T01:10:50Z
updated: 2026-10-20T22:41:20Z
closed: 2026-10-20T22:41:20Z
depends: []
parent: material-53f873
tags: [material]
---

If the is-focused material swap reads as a visual pop, animate between the two resolved parameter sets using niri's existing animation/spring machinery rather than a hard cut. Only worth scoping after material-cb348e lands and the pop is observed.

## Notes

- 2026-09-05T01:46:44Z (materials-26.04): apply_resolved in src/render_helpers/material.rs replaces MaterialState on a name change, so jelly residuals restart at the swap; interpolation must keep the state and blend configs instead.
- 2026-09-08T22:56:59Z (materials-26.04): Prism now splits 12 optics by focus state (prism-a4ef9a), so a swap can change tint, refraction, and depth as well as the surface optics - more for an interpolation to cover. Per-state slab geometry (bevel, offsets) is still deliberately withheld on the Prism side precisely because the swap is a hard cut; interpolation here is what would unblock it.
- 2026-09-29T22:44:56Z (materials-26.04): scope: briefed; name swaps replace the offscreen, element identity and seed; layout residuals remain on Tile; capture must establish whether a visible pop warrants interpolation; brief: docs/notes/2026-09-29-material-dynamics-brief.md
- 2026-10-02T16:03:37Z (material-8e3b73): material-8e3b73 (2026-10-02): owner kept the hard cut on focus material swaps. Seed replacement is the largest part of the swap's own step; focus selects a definition (hard cut, fresh MaterialState), a named response (in place, ring/signal only) or folded signal accents. See the Focus swap finding in docs/notes/2026-09-29-material-dynamics-brief.md.
- 2026-10-06T18:41:36Z (materials-26.04): scope: drop; owner kept hard focus cuts in material-8e3b73 (7f5ee9e2); brief: docs/notes/2026-09-29-material-dynamics-brief.md; proposal: drop focus interpolation because the reviewed clips did not justify it
- 2026-10-20T21:43:28Z (materials-26.04): scope: drop; rechecked apply_resolved at 52c14d4d and the owner acceptance in material-8e3b73 (7f5ee9e2); named-material swaps still replace MaterialState and the accepted hard cut remains intentional; brief: docs/notes/2026-09-29-material-dynamics-brief.md; proposal: drop focus interpolation because the owner-reviewed clips did not justify it
- 2026-10-20T22:41:20Z (materials-26.04): dropped
  provenance: {"harness_session":"claude-code:04a1668e-d112-46ed-a7cd-a0b1cca36d84","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-20T22:41:20Z (materials-26.04): owner approved drop (2026-10-20): hard focus cuts kept in material-8e3b73 (7f5ee9e2); apply_resolved still replaces MaterialState by design, so interpolation has no premise
  provenance: {"harness_session":"claude-code:04a1668e-d112-46ed-a7cd-a0b1cca36d84","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
