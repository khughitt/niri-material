---
id: material-2ee11e
title: "Run the pilot and matrix, review evidence and integrate"
status: doing
priority: 1
size: m
complexity: mid
process: direct
owner: material-a1d7da
created: 2026-09-30T10:47:21Z
updated: 2026-10-02T05:04:47Z
started: 2026-10-02T03:25:21Z
depends: [material-c94dd7]
parent: material-f86183
tags: [performance]
source: "docs/plans/2026-09-30-sustained-optic-settling.md#task-5"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
plan: docs/plans/2026-09-30-sustained-optic-settling.md
step: "Task 5: Run the pilot and matrix, review evidence and integrate"
---

Run an end-to-end pilot before the nine-family matrix, retain unavailable hardware lanes as unverified, obtain owner clip review and publish evidence. No new watt claim or installation. Execute only after plan acceptance and completion of material-0db905; the design task itself implements and captures nothing.

## Notes

- 2026-10-02T03:25:21Z (material-a1d7da): started
  provenance: {"harness_session":"claude-code:2396c14b-dc41-43ce-a03a-8efc21e78f8a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T04:20:28Z (material-a1d7da): run: 25 min (est 27, headless); 28 cases incl. setup, export and analysis; passed: acceptance pilot on reviewed 2f3b5b6a (binary sha256 9c2cc1c1…): lane-passed, 23 passed, 5 unverified (output-removal, screencast, tty-resume, unlock, idle-inhibitor), worst trace-end gap 0.88 s of 1.5, no panics, no leftover processes (pilot-20261002-1)
- 2026-10-02T05:04:46Z (material-a1d7da): run: 39 min (est 45, headless); 32 cases incl. export and analysis; passed: matrix on 2f3b5b6a gated on pilot-20261002-1: lane-passed, 27 passed, 5 unverified, aurora-full held 601.9 s with one flush and equal logical time at pause/resume, worst trace-end gap 0.85 s, no panics, no leftovers (matrix-20261002-1)
- 2026-10-02T05:04:46Z (material-a1d7da): Idle/resume clip (outside capture protocol, same binary): 0 changed pixels across the hold, resume step 44,203 px vs ~45,000 per active half-second step; evidence doc and review page https://claude.ai/artifact/4b8iRwZA54qojFr3T3EEYU
