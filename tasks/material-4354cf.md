---
id: material-4354cf
title: Design a follow-lag jelly stimulus for interactive drag
status: todo
priority: 1
size: m
complexity: high
process: planned
created: 2026-09-30T10:00:19Z
updated: 2026-09-30T10:00:29Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
agent: claude-code/claude-opus-5-5
---

Outcome: glass responds to pointer drag, not only to the layout's lift and release animations. material-b3ce14 measured the jelly residual at exactly 0 during drag and hold in both layouts (src/layout/tests/drag_dynamics.rs; brief section 'Drag baseline finding'). Candidate contract: during InteractiveMoveState::Moving the tile renders at the pointer while a critically damped point chases that location on the window-movement spring; the point's lag is the jelly residual, decaying to 0 on a hold, and release animate_move_from starts from the lagged point so there is no jump. Owner decisions the spec must put to review: whether a held drag deforms at all, gain (jelly-flex or a separate drag gain), spring (window-movement or dedicated), whether long tiled drops keep saturating at max flex, whether the rubber band flexes. Acceptance: extend drag_dynamics.rs to pin lag decay to zero on hold and release continuity; a nested drag/hold/release clip beside the native column-move control for owner judgment. Preserve finite settling, idle-budget and render order. Out of scope: the max-flex cap, whole-slab shear and ior-independent ripple levers, which stay on material-6d4de5.

## Notes

- 2026-09-30T10:00:29Z (materials-26.04): concerns: material-b3ce14 extension — design the drag stimulus the baseline found missing
