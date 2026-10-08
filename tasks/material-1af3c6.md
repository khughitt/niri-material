---
id: material-1af3c6
title: Verify optic settling when one of two outputs is removed
status: dropped
priority: 1
size: m
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-02T08:06:31Z
updated: 2026-10-07T08:12:15Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-output-removal"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining output-removal acceptance from material-2ee11e. Use a real two-output lane with one output still lit; remove or disable the other while active and while input idle. The shared timeline must continue configured active cadence on the lit output or remain held with no optic deadlines according to global input state. Add the missing bounded capture lane, run its end-to-end pilot before matrix, and publish topology, provenance, transition/draw counts, hashes, cleanup and run notes. Use an explicitly identified worktree binary and preserve unavailable hardware cases as unverified; no installed-compositor changes.

## Notes

- 2026-10-04T03:28:50Z (materials-26.04): 2026-10-03 quiet TTY session: not runnable here, only one output is connected on titan (card1-DP-1; every other connector disconnected). Needs a second monitor attached (or a host with two) before its pilot.
- 2026-10-06T09:08:25Z (materials-26.04): parked (waiting on user, environment): Owner: attach a second monitor to titan (or name a two-output host); then agent: add the bounded two-output capture lane and run its end-to-end pilot before the matrix
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T08:12:15Z (materials-26.04): dropped
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T08:12:15Z (materials-26.04): Cancelled by the owner 2026-10-07: no second monitor, none expected. The real two-output capture lane stays unverified; replaced by deterministic in-process coverage of output removal (settling spec §7 item 4)
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
