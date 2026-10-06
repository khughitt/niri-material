---
id: material-2a1689
title: "Docs, the nested-Weston smoke against a baseline binary, and the evidence document"
status: done
priority: 2
size: m
complexity: mid
process: direct
owner: material-cf32e5
created: 2026-10-05T14:01:37Z
updated: 2026-10-06T03:43:51Z
started: 2026-10-05T15:14:43Z
completed: 2026-10-06T03:43:51Z
depends: [material-0bed95]
parent: material-cf32e5
tags: []
model: claude-opus-5-5
agent: claude-code
plan: docs/plans/2026-10-05-noise-placement.md
step: "Task 5: Docs, the nested-Weston smoke against a baseline binary, and the evidence document"
---

Task 5 of docs/plans/2026-10-05-noise-placement.md.

## Notes

- 2026-10-05T15:14:43Z (material-cf32e5): started
  provenance: {"harness_session":"codex:01a10c84-1457-7892-bb5d-ed0227e1d702","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T15:21:05Z (material-cf32e5): run: 0.4 min (est 1, idle); preflight 0.4; refused: load1 3.02 > 2.0, GPU utilization 12% > 5%, active browser GPU compute client
- 2026-10-05T15:42:03Z (material-cf32e5): Prepared smoke with a seven-assertion pilot, references and evidence record; offline simulation and baseline/implementation builds passed. Nested captures remain unrun after the readiness preflight refusal.
- 2026-10-05T16:56:59Z (material-cf32e5): Readiness timing correction: the recorded 20-second sample also had setup/restore overhead, about 37 seconds end to end (0.6 min); the earlier run note counted only the sampling interval.
- 2026-10-05T16:56:59Z (material-cf32e5): parked (waiting on user, quiet; headless, 44 min): Agent from a TTY with desktop stopped: in .worktrees/material-cf32e5 set NIRI_NOISE_ARTIFACTS to $(dirname "$NIRI_MATERIAL_WORK_ROOT")/tmp/material-cf32e5; display-dim status then set 0.25; run the evidence doc smoke pilot (13 min), read its verdict, then full matrix (25 min), analysis/evidence (6 min). Baseline binary is retained in noise-binaries-8db25b43. Prior preflight refused load 3.02, GPU 12%, browser compute; latest GPU 16%/P3. Record run notes; restore dimming, close this child in the evidence commit, then execute material-40b563.
  provenance: {"harness_session":"codex:01a10c84-1457-7892-bb5d-ed0227e1d702","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T17:38:14Z (material-cf32e5): Owner confirmed quiet-queue resumption later tonight. Keep the desktop-stopped requirement: smoke pilot/full/evidence 44 min, followed by material-40b563 cost pilot/full/evidence 16 min; agent resumes the recorded commands in this worktree.
- 2026-10-06T02:12:54Z (material-cf32e5): resumed
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T02:24:52Z (material-cf32e5): run: 10.7 min (est 13, headless); preflight+build+captures 10.7; failed: smoke script defect, film_ae did not strip ImageMagick 7's normalized suffix ("0 (0)"); 20 cells deterministic, omitted=glass and baseline AE 0 before the stop. Fixed film_ae like the lib's compare_metric; rerunning the pilot. Display-dim: monitor rejected DDC (DDCRC_VERIFY), displays undimmed.
- 2026-10-06T02:35:13Z (material-cf32e5): run: 8.0 min (est 13, headless); preflight+build+captures 8.0; failed: assertion 5 roughness, backdrop btrue r0 sd 0 (AE 0 vs zero cell) then r1 sd 0.000144 (0.037 codes, max 1 code, 63% survives 8x downsample: pyramid-spread rounding residue). All other pilot metrics in range (film one-code AE 0, backdrop/glass MAE 0.0075 quantum). Amended assertion 5: strict fall while sd >= 0.5 code (src/tests/noise_site.rs presence threshold), below it later cells must stay below; spec §7.2 updated to match. Rerunning the pilot.
- 2026-10-06T02:48:46Z (material-cf32e5): run: 10.0 min (est 13, headless); preflight+build+captures 10.0; failed: assertion 7 fixture, ring band level 0.6 outside 0.85..0.95. Assertions 1-6 passed incl. the amended roughness floor. Cause: band crop at PX+PW-14 ignored offset-x 6; measured ring peak at PX+PW-6.5 (left/top at +17.5 = bevel 12 + offset 6). Crop moved to PX+PW-8; offline recompute from the pilot's captures: e_b 0.924, glass ratio 0.4554 vs expected 0.4516, film 0.9942 vs 1.0. Rerunning the pilot.
- 2026-10-06T02:59:02Z (material-cf32e5): run: 10.0 min (est 13, headless); preflight+build+captures 10.0; passed: pilot, all seven assertions (ring e_b 0.924, glass 0.4554 vs 0.4516, film 0.9942). Starting the full matrix.
- 2026-10-06T03:17:17Z (material-cf32e5): run: 17.9 min (est 25, headless); preflight+build+captures 17.9; passed: full matrix, all seven assertions. Backdrop softens far faster than the §7.1 model: p1 sd 0.00357 (4.1% of glass; model fine ~19%), p3 0; roughness 0.5 already 0 without blur.
- 2026-10-06T03:43:51Z (material-cf32e5): done
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T03:43:51Z (material-cf32e5): Nested-Weston smoke passes all seven assertions against the b261ad1a baseline (full matrix noise-site-20261005-225906); evidence written. Fixture corrections: film_ae strips IM7's normalized suffix, ring band crop follows offset-x, assertion 5 monotone down to the 8-bit floor (spec §7.2 amended).
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
