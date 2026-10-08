---
id: material-7afc31
title: Skip material rendering for windows that are not visible
status: idea
priority: 1
needs: [quiet]
created: 2026-09-11T23:34:15Z
updated: 2026-10-08T14:54:47Z
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
- 2026-10-01T04:39:26Z (materials-26.04): material-d09741 finding (docs/materials/2026-09-30-hidden-window-attribution-evidence.md): hidden material tiles (inactive workspace, hidden tab, offscreen column, opaque cover) draw no material and never rebuild the prefilter; a frame-callback-paced client drops from 60 to the 1 Hz fallback cadence. Residue for this idea: each hidden-client commit queues an output redraw, and for an offscreen column or a tile under an opaque window Tile::render_inner re-renders the tile's OffscreenBuffer per commit though nothing draws it (ScrollingSpace::render walks every column; MaterialRenderElement declares no opaque region). Unmeasured: a sustained optic under an opaque cover (tick_deadline rejects only out-of-view tiles) and a second lit output. The reported 34% Kitty GPU on a hidden workspace is client work: compositor material draws are zero while hidden, so a client still busy there is not paced by frame callbacks.
- 2026-10-06T15:07:38Z (material-233295): From material-233295: Tile::tick_deadline rejects only windows out of view, not covered ones, so a sustained optic under an opaque cover may still schedule redraws (unmeasured); include that case in the visibility matrix.
- 2026-10-06T20:11:16Z (materials-26.04): scope: briefed; measured hidden tiles draw no material but offscreen preparation persists; safe culling and covered-optic scheduling need one shared boundary audit before implementation; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
- 2026-10-08T14:54:47Z (culling-boundaries): Woken by material-82e4bc (docs/materials/2026-10-08-culling-damage-boundaries-audit.md §2, §4): skipping the material preparation for out-of-view tiles is conditionally safe (reveal repaints from current surface state; frame callbacks unchanged; tile renders only on its own monitor). Conditions: overview-correct view first (material-2e97ff), extent = slab band ∪ Window::bbox() ∪ bob/open/resize areas, never inside render_snapshot or the open/alpha (0,0) call, same predicate as signal_render_visible (else the beam never ends). Direct with the audit as spec; saving is the ~1 Hz offscreen residue. Covered-tile culling is unknown: no coverage exists before render.
