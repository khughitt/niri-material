---
id: material-a4d874
title: "Noise layers docs, nested smoke, contact sheet and evidence"
status: done
priority: 2
complexity: mid
process: direct
needs: [quiet]
owner: material-3fcba2
created: 2026-10-06T10:17:17Z
updated: 2026-10-07T03:14:27Z
started: 2026-10-06T11:19:53Z
completed: 2026-10-07T03:14:27Z
depends: []
parent: material-3fcba2
tags: []
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-06-noise-layers.md
step: "Task 4: Docs, the nested-Weston smoke and contact sheet, the evidence document"
---

## Notes

- 2026-10-06T11:19:53Z (material-3fcba2): started
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:21:34Z (material-3fcba2): run: 0.67 min (est 6, headless); preflight 0.67; refused: CPU 16.5% > 10%, load1 4.92 > 2, GPU 34% > 5%, P5 instead of P8; no cells completed; hold rolled back
- 2026-10-06T11:21:34Z (material-3fcba2): parked (waiting on user, quiet; headless, 21 min): Agent: build the 4a8b2072 baseline in .worktrees/material-3fcba2-baseline, run the smoke pilot (NOISE_LAYERS_PILOT=1, ~6 min: build 4, cells 2), then the full run (~15 min: cells 15), then write and attach the evidence document; prior preflight refused on CPU/load/GPU P5, no cells ran
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:23:32Z (material-3fcba2): Preparation complete: layer and pipeline docs, smoke script, position-class helper, shared capture helpers; synthetic bins min/max 0.9585/1.0415 and LF 0.2489, broken helper exits 1 without stale values; full tooling 409 passed with 2 optional retained-binary skips. Baseline worktree created and release build tracked by this session.
- 2026-10-06T11:24:53Z (material-3fcba2): Validation detail: full tooling ran 409 cases, 407 passed and 2 optional retained-binary checks skipped; preflight record: /mnt/ssd3/niri-material/noise-layers-preflight-1791285645/capture.json
- 2026-10-06T11:31:16Z (material-3fcba2): Baseline 4a8b2072 built successfully in 3m 26s after just setup in the locked .worktrees/material-3fcba2-baseline; binary: .worktrees/material-3fcba2-baseline/target/release/niri. Prep committed as daa55b22; owner sheet and smoke evidence not yet produced.
- 2026-10-06T11:31:16Z (material-3fcba2): parked (waiting on user, quiet; headless, 21 min): Agent: run the smoke pilot in .worktrees/material-3fcba2 with BASE_NIRI=.worktrees/material-3fcba2-baseline/target/release/niri resolved from the main checkout (NOISE_LAYERS_PILOT=1, ~6 min: candidate builds 4, cells 2), then full (~15 min: cells 15), then write and attach evidence; baseline already built; prior preflight refused CPU/load/GPU P5, no cells ran
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-07T01:45:52Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T02:00:06Z (material-3fcba2): run: 13.5 min (est 6, headless); build 4.7, cells 8.5; refused: capture release disturbed by systemd-tmpfiles-clean.timer at 21:57:01; every cell completed, all assertions passed (quadrature 1.4159, s8 bins 0.983/1.017, lf 0.125->0.936, slot sds within 0.4%), deterministic AE 0 throughout; daily timer, next due in 23h, so full run proceeds without a repeat pilot
- 2026-10-07T02:07:34Z (material-3fcba2): run: 6.3 min (est 15, headless); build 0.1, cells 6.2; failed: lightness glass vs baseline AE 1 pixel ((219,578) green 121 vs 120, deterministic in both sessions); white and fine byte-identical at all three sites; remaining assertions not reached; run dir noise-layers-full-1791338406
- 2026-10-07T02:24:47Z (material-3fcba2): run: 16.8 min (est 15, headless); build 0.1, cells 16.7; passed with lightness glass baseline identity recorded, not asserted (scratch copy of the smoke, one line changed): lightness_glass_baseline_ae=0.000980392 (one pixel, one code); all other assertions pass (9 site identities less that one, inherit, scale=1 x3, quadrature 1.4159, scale sd within 1.2%, bins within 1.7%, lf rising, slots); clean release; run dir noise-layers-full-soft-1791338868
- 2026-10-07T02:45:15Z (material-3fcba2): Lightness glass identity fixed by a lone-slot-0 scale-1 path in noise_behind running the 4a8b2072 body verbatim (the layered path computes the same values; the driver compiles its lightness arithmetic differently). Glass-site identity run (white, fine, lightness): all AE 0, run dir noise-layers-ident-1791340828; in-process noise 27/27, test-fast 504 passed
- 2026-10-07T03:05:46Z (material-3fcba2): run: 19.4 min (est 15, headless); build 2.7, cells 16.7; passed: all six assertions strict at 558bfa02, every baseline identity AE 0, 44 cells deterministic; clean release; run dir noise-layers-full2-1791341162
- 2026-10-07T03:14:27Z (material-3fcba2): done
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:14:27Z (material-3fcba2): Nested smoke passes all six assertions strict at 558bfa02 against 4a8b2072 (after fixing a one-pixel lightness glass identity miss); evidence document and annotated contact sheet
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
