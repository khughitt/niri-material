---
id: material-2a1689
title: "Docs, the nested-Weston smoke against a baseline binary, and the evidence document"
status: doing
priority: 2
size: m
complexity: mid
process: direct
owner: material-cf32e5
created: 2026-10-05T14:01:37Z
updated: 2026-10-05T17:38:14Z
started: 2026-10-05T15:14:43Z
depends: [material-0bed95]
parent: material-cf32e5
tags: []
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
