---
id: material-511efe
title: "Quiet-run protocol: display-dim is a prerequisite, not optional"
status: doing
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-05T08:54:13Z
updated: 2026-10-05T08:54:14Z
started: 2026-10-05T08:54:13Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

AGENTS.md says to dim 'when ops's display-dim is installed', so a session that finds no display-dim on PATH skips dimming without reporting it (2026-10-05 quiet run, after ops forgot to link it: ops-379403). State it as a prerequisite from ops 'just install', and treat a missing command as a setup fault to report; a monitor rejecting DDC still stays undimmed. List it beside host-budget/host-load in capture-host-setup.md.

## Notes

- 2026-10-05T08:54:13Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T08:54:13Z (materials-26.04): halt override: started past material-cd7782 by 36e424bd-8c56-41e9-a164-005ea47ccb82: doc-only quiet-run protocol fix found by the quiet session; independent of the pre-commit latency incident
