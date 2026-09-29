---
id: material-8e3b73
title: Establish whether focus material swaps need interpolation
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-29T22:45:28Z
updated: 2026-09-29T22:56:49Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
agent: codex
---

Question: Does a current focus-dependent material-name swap create an objectionable discontinuity, and what state and parameter semantics would an interpolation have to preserve?
Where to start: docs/notes/2026-09-29-material-dynamics-brief.md; docs/materials/2026-09-04-focus-glass-spike.md; src/layout/tile.rs::resolve_material/refresh_material/update_render_elements; src/render_helpers/material/mod.rs::apply_resolved/MaterialState and its identity tests; niri-config/src/material/mod.rs and params.rs; niri-config/src/animations.rs::MaterialSignalAnim.
Bound: Trace the state lifetime and produce one short repeatable nested focus-toggle clip using two named glass definitions, with a same-definition control. Record exact parameter differences, source/binary/config, rapid reversal, and a toggle during an existing move animation. Separate glass-parameter discontinuity and seed replacement from the finite focus beam. Use existing capture helpers, run a pilot through its verdict first, and park any quiet-host requirement with the command and duration. No production interpolation or Prism schema changes.
Expected result: Save the clip and record an owner-review verdict, or explicitly leave visual acceptance pending. Inventory continuous values, colors, discrete gates/response selectors, missing materials, reload and reversal semantics that a design would need. Correct the old assumption that a name swap resets layout residuals: those live on Tile, while MaterialState replaces its seed, offscreen and element identity. Recommend retaining the hard cut unless the demonstrated visual problem warrants a reviewed interpolation design; describe remaining unknowns rather than choosing blend semantics speculatively.
Ideas it wakes: On completion, run tasks note on material-5a5fff and material-9be53d with the finding, in the same commit as this result.

## Notes

- 2026-09-29T22:56:49Z (materials-26.04): Additional scope handoff: docs/notes/2026-09-29-adaptive-materials-brief.md. The existing state/parameter inventory also informs material-764d8c: distinguish selected glass definition, named response overrides and folded signal accents before recommending generic profile blend/override/constrain operators. No additional capture lane is required by this note. Ideas it wakes: On completion, run tasks note on material-764d8c with the relevant finding in the same commit as this result, alongside material-5a5fff and material-9be53d, and update the adaptive-materials brief.
