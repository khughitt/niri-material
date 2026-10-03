---
id: material-80caf4
title: Verify optic settling under a real idle inhibitor
status: doing
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-02T08:06:31Z
updated: 2026-10-02T19:52:11Z
started: 2026-10-02T19:52:10Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-idle-inhibitor"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining idle-inhibitor capture acceptance from material-2ee11e; the real-handler fixture already covers the wiring. Extend the existing bounded driver with a real idle-inhibitor client, prove that inhibition does not notify input activity or restart Aurora deadlines while held, and retain normal client updates. Pilot before matrix, explicit worktree binary, trace liveness controls, hashes, cleanup and run notes. Update the acceptance evidence; do not substitute the fixture test for the missing lifecycle capture.

## Notes

- 2026-10-02T19:52:10Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T19:52:10Z (materials-26.04): approach: real Wayland idle-inhibit client (C, built per run with wayland-scanner; no client is installed), a Tracy 'IdleInhibit inhibited=N' message on niri's inhibited-state change so the trace proves inhibition took hold, the idle-inhibitor case moved into the headless lane, and an analyzer check that a stimulus's required messages fall inside its journaled window. The pilot needs the headless host and is parked for tasks quiet.
