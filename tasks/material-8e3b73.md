---
id: material-8e3b73
title: Establish whether focus material swaps need interpolation
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: material-8e3b73
created: 2026-09-29T22:45:28Z
updated: 2026-10-02T16:03:39Z
started: 2026-09-30T10:08:14Z
completed: 2026-10-02T16:03:37Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
model: claude-opus-5-5
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
- 2026-10-01T03:49:16Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T03:49:16Z (materials-26.04): run: 2.5 min (est 3.5, headless); build 1.5, preflight 0.5, swap 0.5; failed: the burst recorded 10 frames, then the analysis exited on magick compare's exit 1 for differing images under set -e/pipefail; fixed in d1cd7685, which keeps exit 2 fatal (focus-swap-3819314-1790826335)
- 2026-10-01T03:54:53Z (materials-26.04): run: 2.5 min (est 3.5, headless); build 1.5, preflight 0.5, swap 0.5; passed: pilot through its verdict, 10 frames at ~3.2 Hz (0.31 s spacing, not the ~10/s the header claims); focus at 0.527 s, one step f002->f003 of 0.033 RMSE in both panes, 0.001 residue in the next frame, then 0 (focus-swap-3827248-1790826721). Settle load1 was 14.9 from the rebuild that HEAD's new commit forced; the gate does not check load1
- 2026-10-01T04:02:19Z (materials-26.04): run: 4 min (est 5.5, headless); preflight 0.5, six sequences 3.5; passed: full run, all six settled (load1 0.96-1.6, cpu 1.1%, gpu 0%). Per-pane frame-to-frame RMSE at ~3.2 Hz, focus at ~0.52 s: swap 0.033 one-frame step then 0.001; same (control) 0.017 then 0.0005 -- the focus change alone (kitty cursor, ring crossfade) is half the swap's step; seed 0.029 then 0.0008 then 0.0001 -- the reseeded aurora field visibly jumps in seed-step.png; reversal two 0.033 steps; move 0.22 (column motion dominates); beam same as swap -- the 0.31 s burst spacing is too coarse to catch the beam pass (focus-swap-3831771-1790827068)
- 2026-10-01T04:55:23Z (materials-26.04): Clip section filled in the dynamics brief (.worktrees/material-8e3b73 21f73a73); review page with the six clips and step images: https://claude.ai/artifact/Df32vBpd48gbbkiPSn6vuP
- 2026-10-01T04:55:23Z (materials-26.04): parked (waiting on user, review): Owner: judge swap/same/seed/move/beam on https://claude.ai/artifact/Df32vBpd48gbbkiPSn6vuP (recommendation: keep the hard cut; the seed jump is the strongest case for change). Then agent, in .worktrees/material-8e3b73: record the verdict in the brief's Clip section, note material-5a5fff, material-9be53d and material-764d8c, update the adaptive-materials brief, merge and close in one commit
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:03:37Z (material-8e3b73): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:03:37Z (material-8e3b73): review: impl round 1 — verdict: accept; findings: none; reviewer: human
- 2026-10-02T16:03:37Z (material-8e3b73): done
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:03:37Z (material-8e3b73): owner kept the hard cut; verdict recorded in the dynamics brief, adaptive brief updated, ideas noted
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
