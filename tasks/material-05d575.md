---
id: material-05d575
title: "Silent clean commits: set quiet and tasks check -q in the justfile"
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-19T12:50:30Z
updated: 2026-09-19T12:58:02Z
started: 2026-09-19T12:58:02Z
completed: 2026-09-19T12:58:02Z
depends: []
tags: []
model: "claude-opus-5[1m]"
agent: "claude-code/claude-opus-5[1m]"
---

Piece of ops ops-4b96bf. Add `set quiet` beside the other settings at the top of the justfile so just no longer echoes recipe lines (`just --verbose` still shows them). Nothing else: `tasks check` itself prints nothing on a clean tree since tasks-9af90a. A clean commit then prints nothing; failures are unchanged.

## Notes

- 2026-09-19T12:58:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:3111d755-79c1-4c78-8c1a-8c2478835ee8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T12:58:02Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:3111d755-79c1-4c78-8c1a-8c2478835ee8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T12:58:02Z (materials-26.04): set quiet in the justfile; commit output verified silent
  provenance: {"harness_session":"claude-code:3111d755-79c1-4c78-8c1a-8c2478835ee8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
