---
id: material-cb348e
title: Focus-state glass treatment for fully transparent terminals
status: done
priority: 2
size: m
owner: materials-26.04
created: 2026-09-05T01:10:50Z
updated: 2026-09-05T01:47:05Z
depends: []
tags: [material, integration]
---

Replace terminal background opacity as the focus cue with the glass itself. Verify an is-focused conditioned window-rule can select between two material names (active/inactive) and that the MaterialState swap is cheap enough to happen on every focus change. Define the active and inactive parameter sets (candidates: thickness, attenuation-color/distance, roughness, backdrop-blur) so an unfocused terminal recedes without a hard-edged opacity change. Acceptance: nested screenshots of two adjacent kitty windows at background_opacity 0 show no visible seam between cell background and glass, focused and unfocused, at corners and edges. Depends on Prism emitting the variants (see prism piece under ops goal ops-500adb).

## Notes

- 2026-09-05T01:10:58Z (materials-26.04): Open question: the swap is a hard cut. If it reads as a pop, interpolating parameters across the swap is material-5a5fff.
- 2026-09-05T01:36:06Z (materials-26.04): Deployed compositor is feat/material-signals 663202b1 (unmerged, not on origin); materials-26.04 lacks signals. Spike runs config-only on the installed binary under a headless weston host: prism terminal-glass plus an inactive variant selected by is-active=false, kitty at background_opacity 0.
- 2026-09-05T01:46:44Z (materials-26.04): Spike passed (docs/materials/2026-09-04-focus-glass-spike.md): kitty at 0 over an is-active material swap removes the seam; legibility needs the glass to carry the terminal background color (#222436 at attenuation-distance 30), inactive variant roughness 0.5 / distance 70. Base treatment is config-only; the swap is a hard cut (new MaterialState on name change).
- 2026-09-05T01:47:05Z (materials-26.04): Spike passed on the installed 663202b1: kitty at 0 over an is-active material swap shows no seam at corners or edges in either focus state; active set = Prism glass with attenuation-color #222436 at distance 30, inactive set = roughness 0.5, chromatic-aberration 0.08, distortion 0.10, distance 70. Swap is a uniform-set replacement, no compile. Native side needs no change; emission moves to prism-5bc782.
