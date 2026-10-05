---
id: material-511efe
title: "Quiet-run protocol: display-dim is a prerequisite, not optional"
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: display-dim-protocol
created: 2026-10-05T08:54:13Z
updated: 2026-10-05T08:54:35Z
started: 2026-10-05T08:54:13Z
completed: 2026-10-05T08:54:35Z
depends: []
tags: [tooling]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

AGENTS.md says to dim 'when ops's display-dim is installed', so a session that finds no display-dim on PATH skips dimming without reporting it (2026-10-05 quiet run, after ops forgot to link it: ops-379403). State it as a prerequisite from ops 'just install', and treat a missing command as a setup fault to report; a monitor rejecting DDC still stays undimmed. List it beside host-budget/host-load in capture-host-setup.md.

## Notes

- 2026-10-05T08:54:13Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T08:54:13Z (materials-26.04): halt override: started past material-cd7782 by 36e424bd-8c56-41e9-a164-005ea47ccb82: doc-only quiet-run protocol fix found by the quiet session; independent of the pre-commit latency incident
- 2026-10-05T08:54:22Z (display-dim-protocol): resumed
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T08:54:35Z (display-dim-protocol): done
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T08:54:35Z (display-dim-protocol): AGENTS.md makes display-dim a prerequisite from ops just install and a missing command a reported setup fault; capture-host-setup lists it with host-budget/host-load
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
