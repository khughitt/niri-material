---
id: material-e866b0
title: Drag follower lifecycle edges
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: material-4354cf
created: 2026-10-01T12:51:50Z
updated: 2026-10-01T14:26:24Z
started: 2026-10-01T14:15:30Z
completed: 2026-10-01T14:26:24Z
depends: [material-8540ac]
parent: material-4354cf
tags: [dynamics]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-01-drag-follow-lag.md
step: "Task 3: Lifecycle edges: regrab, reload, complete-instantly, event rate"
---

## Notes

- 2026-10-01T14:15:30Z (material-4354cf): started
  provenance: {"harness_session":"codex:01a0f7b2-5d00-73d3-8722-9fc32e652fbd","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T14:21:12Z (material-4354cf): Staggered-event RED exact follower comparison passed, but semi-implicit Euler at 1 us missed the 0.001 px bound by 0.001114 px at frame 3; use 0.1 us integration step and retain event ordering and tolerance. Zero-stiffness creation test lifts with valid default stiffness first; config decoder rejects stiffness below 1.
- 2026-10-01T14:26:24Z (material-4354cf): done
  provenance: {"harness_session":"codex:01a0f7b2-5d00-73d3-8722-9fc32e652fbd","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T14:26:24Z (material-4354cf): Drag follower lifecycle: regrab, reload, complete-instantly, event rate; 22 dynamics, 10 follower, 420 fast tests and just check pass
  provenance: {"harness_session":"codex:01a0f7b2-5d00-73d3-8722-9fc32e652fbd","harness_session_source":"CODEX_SESSION_ID"}
