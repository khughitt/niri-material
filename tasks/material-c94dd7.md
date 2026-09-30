---
id: material-c94dd7
title: Build a bounded settling capture and offline verdict
status: doing
priority: 1
size: m
complexity: high
process: direct
owner: material-a1d7da
created: 2026-09-30T10:47:21Z
updated: 2026-09-30T12:48:28Z
started: 2026-09-30T12:09:27Z
depends: [material-6e3bda]
parent: material-f86183
tags: [performance]
source: "docs/plans/2026-09-30-sustained-optic-settling.md#task-4"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
plan: docs/plans/2026-09-30-sustained-optic-settling.md
step: "Task 4: Build a bounded settling capture and offline verdict"
---

Implement the pilot-first driver and stdlib analyzer with marker-aligned windows, positive controls, bounded cleanup and explicit cadence controls. No live capture in this step. Execute only after plan acceptance and completion of material-0db905; the design task itself implements and captures nothing.

## Notes

- 2026-09-30T12:09:27Z (material-a1d7da): started
  provenance: {"harness_session":"codex:01a0f202-290f-70e2-b161-8b244605074f","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T12:43:08Z (material-a1d7da): run: headless development pilot preflight refused at load1 4.31 > 2.0; no compositor or trace started; artifact /mnt/ssd3/niri-material/optic-settling/pilot-dev-20260930-1; continue offline fixture work, then queue live pilot for tasks quiet.
- 2026-09-30T12:48:28Z (material-a1d7da): parked (waiting on agent, quiet; headless, 10 min): On the quiet host, resume this task and run the 18-second headless active-idle pilot with the identified binary; inspect marker/CSV timing and cleanup, then implement the remaining headless check families before matrix capture.
  provenance: {"harness_session":"codex:01a0f202-290f-70e2-b161-8b244605074f","harness_session_source":"CODEX_SESSION_ID"}
