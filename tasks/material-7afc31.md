---
id: material-7afc31
title: Skip material rendering for windows that are not visible
status: idea
priority: 1
created: 2026-09-11T23:34:15Z
updated: 2026-09-29T21:43:50Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Check whether occluded windows, windows on inactive workspaces, or windows on other outputs still run the material shader or its prefilter passes. If they do, gate that work on visibility. Measure the saving with the capture protocol before deciding whether the complexity is worth it.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

2026-09-16 priority: reduce wasted work for invisible windows before adding more persistent dynamics. User reports the Kitty window sampled at 34% GPU was on a workspace that was not visible. This is evidence of client GPU activity, not yet proof that niri rendered its material. Trace visibility, occlusion, frame callbacks and animation/redraw scheduling as well as compositor material/prefilter draws; separate client work from compositor work, include inactive workspaces and off-screen windows, and preserve visibility on other active outputs. Compare visible, hidden-workspace and empty-workspace cases with the same workload. The subsequent empty-workspace preflight passed at 3% GPU/P8, so do not assume every hidden window always consumes the observed load.

## Notes

- 2026-09-16T12:00:25Z (materials-26.04): Raised P2 to P1 at user request: prioritize avoiding invisible-window and idle resource waste; distinguish observed client GPU use from unproven compositor rendering.
- 2026-09-29T21:43:50Z (materials-26.04): scope: briefed; hidden-workspace/tab/offscreen attention draws already gate to zero; broader material/prefilter and client attribution still need a bounded audit; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
