---
id: material-b3ce14
title: Establish glass deformation during pointer drag and release
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-09-29T22:45:28Z
updated: 2026-09-29T22:46:47Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
agent: codex
---

Question: What material deformation does a pointer drag, hold and release actually produce in scrolling and floating layouts, and which missing stimulus explains any gap versus a native column move?
Where to start: docs/notes/2026-09-29-material-dynamics-brief.md; docs/materials/2026-09-11-jelly-motion-sweep.md and its archived fixture reference; src/layout/mod.rs interactive_move_update/interactive_move_end and render_interactive_move_for_output; src/layout/scrolling.rs and floating.rs render paths; src/layout/tile.rs::animation_residual/material_dynamics; src/render_helpers/material/mod.rs::jelly_state.
Bound: Trace all callers of animation_residual and the move/resize residual route. Build the smallest repeatable nested drag/hold/release capture with neutral and nonzero flex/ripple settings, one scrolling and one floating window, plus a native column-move positive control. Start with one complete pilot through its verdict; park any quiet-host requirement with the exact command and duration. Pin source/binary/config and record actual motion timing. Do not change production deformation math, slider ranges, or introduce a motion engine; raw grab offset is not velocity. Coordinate fixture work through the referenced experiment repository's instructions when needed.
Expected result: Record the input sequence, observed residuals/deformation, repeat variation and settled behavior on this task and in the brief. Separate a capture failure from absent response. Recommend the smallest missing input or response contract and list the visual choices requiring owner review; do not choose an aesthetic gain from a numerical difference alone. Identify whether existing native animation and named response settings suffice for this behavior.
Ideas it wakes: On completion, run tasks note on material-6d4de5 and material-9be53d with the finding, in the same commit as this result.
