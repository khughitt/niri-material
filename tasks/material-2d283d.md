---
id: material-2d283d
title: Persist the 2026-09-27 quiet-capture lessons in agent-facing docs
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T08:29:23Z
updated: 2026-09-28T08:30:57Z
started: 2026-09-28T08:29:29Z
completed: 2026-09-28T08:30:57Z
depends: []
tags: [docs]
agent: claude-code/claude-opus-5-5
---

Root AGENTS.md: agents may run tasks quiet work themselves when the user hands over a TTY session with the desktop stopped. niri-experiments results/idle-budget docs/results/2026-09-11-idle-budget.md reproduction notes: start an agent-launched supervisor with env --default-signal=INT, and rebuild saved binaries when niri validate refuses a case config.

## Notes

- 2026-09-28T08:29:29Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T08:30:57Z (material-2d283d): done
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T08:30:57Z (material-2d283d): AGENTS.md: agents run the quiet queue when handed a TTY session; niri-experiments results/idle-budget 05bf987: supervisor INT pitfall and stale saved binaries
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
