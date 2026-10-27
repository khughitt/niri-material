---
id: material-6db557
title: Switch the quiet-session display step to blank-at-start in the agent guide
status: done
priority: 1
size: xs
complexity: low
process: direct
owner: material-6db557
created: 2026-10-27T11:34:21Z
updated: 2026-10-27T11:49:19Z
started: 2026-10-27T11:34:55Z
completed: 2026-10-27T11:49:19Z
closed: 2026-10-27T11:49:19Z
depends: []
tags: [docs]
agent: claude-code
origin: user
---

Owner, 2026-10-27: fb0 screen blanking now works consistently and DDC dimming consistently does not. The quiet-session paragraph in AGENTS.md should skip the display-dim attempt and blank the display from the start. Change it to: on a bare TTY, check that no compositor is running and power the panel down once at queue start (echo 4 to /sys/class/graphics/fb0/blank under the fb-blank sudoers rule; confirm through the connector's dpms file). Do not restore at the end: the owner's input wakes it. With the desktop up, the capture hold powers the monitors off. Record the sudoers rule in docs/materials/capture-host-setup.md. ops-cab84f will later replace the raw sysfs write with a tool.

## Notes

- 2026-10-27T11:34:21Z (add): created
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"user"}
- 2026-10-27T11:34:55Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-27T11:35:03Z (material-6db557): resumed
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-27T11:44:31Z (material-6db557): review: impl round 1 — verdict: revise; findings: Important 1, Minor 7; reviewer: claude-code/claude-opus-5-5
- 2026-10-27T11:48:58Z (material-6db557): review: impl round 2 — verdict: accept; findings: Minor 3; reviewer: claude-code/claude-opus-5-5
- 2026-10-27T11:49:05Z (material-6db557): review: impl round 3 — verdict: accept; findings: Minor 1; reviewer: claude-code/claude-opus-5-5
- 2026-10-27T11:49:12Z (material-6db557): minor (deferred): N3 — the display-dim status reason covers sessions from now on; an older session's saved-brightness record could in principle exist (none today: display-dim status printed 'no saved brightness').
- 2026-10-27T11:49:19Z (material-6db557): done
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-27T11:49:19Z (material-6db557): AGENTS.md quiet-session display step now blanks fb0 once at queue start, with no dimming attempt and no restore or re-blank; capture-host-setup.md records the fb-blank sudoers rule, its NOPASSWD check, and why DDC dimming is skipped on the reference host. Three review rounds (revise, accept, accept).
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
