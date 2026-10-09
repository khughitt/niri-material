---
id: material-acd7d3
title: "Launchers outside the lib, and the coverage scan"
status: done
priority: 1
size: s
complexity: low
process: direct
owner: capture-host-conditions
created: 2026-10-08T23:26:54Z
updated: 2026-10-09T00:58:53Z
started: 2026-10-09T00:50:33Z
completed: 2026-10-09T00:58:53Z
depends: [material-8dce3f]
parent: material-18c2a1
tags: [capture]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-08-capture-host-conditions.md
step: "Task 5: Launchers outside the lib, and the coverage scan"
---

## Notes

- 2026-10-09T00:50:33Z (capture-host-conditions): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T00:58:53Z (capture-host-conditions): done
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T00:58:53Z (capture-host-conditions): clip launchers write Weston output to weston.log, launch niri with the GLES renderer log and run capture-meta renderer on this launch's slices after the first client maps; optic-settling start_drm verifies the DRM renderer; a source scan fails any capture fixture that launches without the check
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
