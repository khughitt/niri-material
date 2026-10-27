---
id: material-6db557
title: Switch the quiet-session display step to blank-at-start in the agent guide
status: doing
priority: 1
size: xs
complexity: low
process: direct
owner: material-6db557
created: 2026-10-27T11:34:21Z
updated: 2026-10-27T11:44:31Z
started: 2026-10-27T11:34:55Z
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
