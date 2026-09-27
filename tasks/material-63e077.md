---
id: material-63e077
title: The input-activity gate for sustained attention motion
status: done
priority: 1
size: m
complexity: mid
process: direct
owner: material-82323e
created: 2026-09-19T02:49:04Z
updated: 2026-09-19T10:30:35Z
started: 2026-09-19T10:20:10Z
completed: 2026-09-19T10:30:35Z
depends: [material-b4e8d4]
parent: material-0e130e
tags: [signals, performance]
model: "claude-opus-5[1m]"
agent: claude-code/claude-opus-5
plan: docs/plans/2026-09-18-ring-focus-motion.md
step: "Task 3: The input-activity gate"
---

## Notes

- 2026-09-19T10:20:10Z (material-82323e): started
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T10:30:35Z (material-82323e): TestWindow gained set_signal so the tile test asserts the motion rule (pulse active, static idle, level untouched, no deadline). niri-visual-tests' tile case updated for the new signature.
- 2026-09-19T10:30:35Z (material-82323e): done
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T10:30:35Z (material-82323e): InputActivity (src/activity.rs) owned by Niri with a drop-and-rearm timer; notify_activity feeds it; reload recomputes; effective(.., input_active) forces sustained motion Static; threaded through the layout chain; fixture tests via virtual pointer; signals design documents the gate
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
