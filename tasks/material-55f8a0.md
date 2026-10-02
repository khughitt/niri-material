---
id: material-55f8a0
title: "Drag-lag clip driver, pilot, clips and brief"
status: doing
priority: 1
size: m
complexity: mid
process: direct
owner: material-4354cf
created: 2026-10-01T12:51:50Z
updated: 2026-10-02T03:14:23Z
started: 2026-10-01T14:31:57Z
depends: [material-e866b0]
parent: material-4354cf
tags: [dynamics, capture]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-01-drag-follow-lag.md
step: "Task 4: Clip driver, pilot, clips and the brief"
---

## Notes

- 2026-10-01T14:31:57Z (material-4354cf): started
  provenance: {"harness_session":"codex:01a0f7b2-5d00-73d3-8722-9fc32e652fbd","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T14:42:53Z (material-4354cf): run: 6.47 min (est 8, idle); build 6.03, preflight 0.44; refused: CPU/load/GPU thresholds and compute client
- 2026-10-01T14:55:39Z (material-4354cf): review: impl round 1 — verdict: accept; findings: P3 2; reviewer: codex/gpt-6.1-sol
- 2026-10-01T14:56:30Z (material-4354cf): parked (waiting on user, quiet; idle, 11 min): Agent tonight on idle host: in .worktrees/material-4354cf run the scroll-fast pilot with CAPTURE_TASK=material-55f8a0 SEQUENCES=scroll-fast docs/materials/scripts/drag-lag-clips.sh, inspect ready/done, Moving IPC check and flex frames; only if pilot passes run all five sequences, publish review clips, update brief, then park for owner visual judgment. Phases: build 6.5 min, preflight 0.5, pilot 1, full 3 (11 min). Earlier pilot refused before launch on CPU/load/GPU thresholds and Bitwig compute client; no frames captured. No host pointer changes.
  provenance: {"harness_session":"codex:01a0f7b2-5d00-73d3-8722-9fc32e652fbd","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T15:06:46Z (material-4354cf): Prepared implementation approved by final branch review; GIF playback caveat added and scoped review accepted in 8e736f2f. Runtime pilot/full clips, publication and owner judgment remain open; user deferred idle host until tonight.
- 2026-10-02T03:01:52Z (material-4354cf): resumed
  provenance: {"harness_session":"claude-code:2396c14b-dc41-43ce-a03a-8efc21e78f8a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T03:09:04Z (material-4354cf): run: 3 min (est 1.5, idle); build 1.7, preflight+settle 0.3, pilot 1; passed: scroll-fast pilot 39 frames/4 s, interactive move confirmed, lift and drop visible in sheet; run drag-lag-1375033-1790910448 (TTY, desktop stopped)
- 2026-10-02T03:14:23Z (material-4354cf): run: 2.3 min (est 3, idle); preflight+settle 0.3, scroll-fast 0.7, scroll-slow 0.7, float-fast 0.6; failed: float-fast vdrag timed out waiting for button event (reproduced alone). Cause: pointer starts at output centre over the centred floating window, so vdrag sees enter at map, then the walk's first step leaves; vdrag ignored leave and pressed over kitty. Fix: clear entered on wl_pointer.leave.
