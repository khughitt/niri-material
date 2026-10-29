---
id: material-ff2fcf
title: Merge or remove the leftover material-77db8a and material-89fb6b worktrees
status: doing
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-25T17:04:57Z
updated: 2026-10-29T19:13:29Z
started: 2026-10-29T19:13:29Z
depends: []
tags: []
source: ops-38c523
agent: claude-code
origin: agent
---

material-77db8a (dropped) and material-89fb6b (done) still have worktrees whose branches hold record and doc commits that materials-26.04 lacks (22 and 5 commits). records-migrate refuses to move material's records while they do (ops-38c523). Merge the record and doc changes that should survive, then remove the worktrees by the usual worktree-removal checks.

## Notes

- 2026-10-25T17:04:57Z (add): created
  provenance: {"harness_session":"claude-code:6868fadb-a6fa-442d-a878-d2615487c638","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"agent"}
- 2026-10-29T19:13:29Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:2db0caf2-46d5-402c-b4aa-03b22c49e8bc","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
