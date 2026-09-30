---
id: material-8e3b73
title: Establish whether focus material swaps need interpolation
status: doing
priority: 2
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-29T22:45:28Z
updated: 2026-09-30T10:23:38Z
started: 2026-09-30T10:08:14Z
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
- 2026-09-30T10:08:14Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:90867bba-6d94-4787-81ee-3e6a3b57b0fe","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T10:19:37Z (materials-26.04): run: 0.5 min (est 2, idle); preflight 0.5; refused: cpu_busy 25%, load1 22.7, gpu_util 22%, chromium GPU client (SEQUENCES=swap pilot, focus-swap-3637538-1790763542)
- 2026-09-30T10:23:38Z (materials-26.04): parked (waiting on user, quiet; idle, 10 min): Quiet capture, then owner review. In .worktrees/material-8e3b73 (commit 870c4694; tests, fixture and finding are committed), with the host idle (browser closed, monitors off, wali-rotate.timer stopped): pilot CAPTURE_TASK=material-8e3b73 SEQUENCES=swap docs/materials/scripts/focus-swap-clips.sh (release build 3 min, preflight 0.5, swap 1) and read swap/diffs.txt and swap.gif; then the full run without SEQUENCES (preflight 0.5, six sequences about 5). Earlier attempt: pilot refused at preflight on CPU, load1, GPU and a Chromium GPU client. After: fill the Clip section of the dynamics brief with the run path and RMSE steps, ask the owner for a verdict on swap/same/seed/move/beam, then in the done commit note material-5a5fff, material-9be53d and material-764d8c and update the adaptive-materials brief.
  provenance: {"harness_session":"claude-code:90867bba-6d94-4787-81ee-3e6a3b57b0fe","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
