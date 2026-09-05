---
id: material-cb348e
title: Focus-state glass treatment for fully transparent terminals
status: todo
priority: 2
size: m
created: 2026-09-05T01:10:50Z
updated: 2026-09-05T01:10:58Z
depends: []
tags: [material, integration]
---

Replace terminal background opacity as the focus cue with the glass itself. Verify an is-focused conditioned window-rule can select between two material names (active/inactive) and that the MaterialState swap is cheap enough to happen on every focus change. Define the active and inactive parameter sets (candidates: thickness, attenuation-color/distance, roughness, backdrop-blur) so an unfocused terminal recedes without a hard-edged opacity change. Acceptance: nested screenshots of two adjacent kitty windows at background_opacity 0 show no visible seam between cell background and glass, focused and unfocused, at corners and edges. Depends on Prism emitting the variants (see prism piece under ops goal ops-500adb).

## Notes

- 2026-09-05T01:10:58Z (materials-26.04): Open question: the swap is a hard cut. If it reads as a pop, interpolating parameters across the swap is material-5a5fff.
