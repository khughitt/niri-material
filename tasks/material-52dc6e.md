---
id: material-52dc6e
title: Pilot the pixels lane on a desktop in use with glass-noise-layers-smoke.sh
status: doing
priority: 2
size: xs
complexity: low
process: direct
needs: [nested, owner]
owner: materials-26.04
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T04:25:14Z
started: 2026-10-09T04:25:14Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §8.2 pilot 1. Run glass-noise-layers-smoke.sh on --lane pixels on the desktop in use, with the owner's go-ahead for host use. Pass: every AE-0 assertion holds; the record has no baseline or GPU sampling, renderers recorded, timers held and restored; metrics.txt equals a retained run with identical provenance.binaries if one exists (else say so in the run note).

## Notes

- 2026-10-09T04:25:14Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
