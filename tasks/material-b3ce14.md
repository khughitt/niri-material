---
id: material-b3ce14
title: Establish glass deformation during pointer drag and release
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-29T22:45:28Z
updated: 2026-09-30T09:52:55Z
started: 2026-09-30T09:44:58Z
completed: 2026-09-30T09:52:55Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
model: claude-opus-5-5
agent: codex
---

Question: What material deformation does a pointer drag, hold and release actually produce in scrolling and floating layouts, and which missing stimulus explains any gap versus a native column move?
Where to start: docs/notes/2026-09-29-material-dynamics-brief.md; docs/materials/2026-09-11-jelly-motion-sweep.md and its archived fixture reference; src/layout/mod.rs interactive_move_update/interactive_move_end and render_interactive_move_for_output; src/layout/scrolling.rs and floating.rs render paths; src/layout/tile.rs::animation_residual/material_dynamics; src/render_helpers/material/mod.rs::jelly_state.
Bound: Trace all callers of animation_residual and the move/resize residual route. Build the smallest repeatable nested drag/hold/release capture with neutral and nonzero flex/ripple settings, one scrolling and one floating window, plus a native column-move positive control. Start with one complete pilot through its verdict; park any quiet-host requirement with the exact command and duration. Pin source/binary/config and record actual motion timing. Do not change production deformation math, slider ranges, or introduce a motion engine; raw grab offset is not velocity. Coordinate fixture work through the referenced experiment repository's instructions when needed.
Expected result: Record the input sequence, observed residuals/deformation, repeat variation and settled behavior on this task and in the brief. Separate a capture failure from absent response. Recommend the smallest missing input or response contract and list the visual choices requiring owner review; do not choose an aesthetic gain from a numerical difference alone. Identify whether existing native animation and named response settings suffice for this behavior.
Ideas it wakes: On completion, run tasks note on material-6d4de5 and material-9be53d with the finding, in the same commit as this result.

## Notes

- 2026-09-30T09:44:58Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:56be2ffb-8b2b-4784-aced-d868b49b6398","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T09:51:39Z (material-b3ce14): Probe (src/layout/tests/drag_dynamics.rs, pinned clock, 16 ms frames, default window-movement spring = live config; flex 0.01 at bevel 12/thickness 20, cap 3 px): scrolling rubber-band 0, lift 1.46 px/240 ms, drag 40 px/frame 0, hold 0, release 2.96 px capped/288 ms, cancel 0.75 px; floating lift 0.18 px (first pointer step), drag/hold/release 0; native column move control 1.11 px/240 ms. Deterministic, zero repeat variation. Absent stimulus, not capture failure. Deviation: no nested pixel capture — zero residual renders settled glass and the jelly-motion sweep already measured shader response to nonzero residuals; a clip for the owner's visual judgment belongs with any follow-lag design.
- 2026-09-30T09:52:55Z (material-b3ce14): done
  provenance: {"harness_session":"claude-code:56be2ffb-8b2b-4784-aced-d868b49b6398","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T09:52:55Z (material-b3ce14): Drag baseline: jelly residual is 0 during drag/hold in both layouts; flex only on lift and tiled release (capped). Probe src/layout/tests/drag_dynamics.rs; finding, follow-lag contract and owner choices in the dynamics brief; noted on material-6d4de5 and material-9be53d.
  provenance: {"harness_session":"claude-code:56be2ffb-8b2b-4784-aced-d868b49b6398","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
