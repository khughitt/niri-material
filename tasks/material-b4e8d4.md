---
id: material-b4e8d4
title: "The sweep replaces the drift clock: solver, tile, fixtures"
status: done
priority: 1
size: m
complexity: mid
process: direct
owner: material-82323e
created: 2026-09-19T02:49:04Z
updated: 2026-09-19T10:19:04Z
started: 2026-09-19T10:04:37Z
completed: 2026-09-19T10:19:04Z
depends: [material-7fd09c]
parent: material-0e130e
tags: [ring, dynamics]
model: "claude-opus-5[1m]"
agent: claude-code/claude-opus-5
plan: docs/plans/2026-09-18-ring-focus-motion.md
step: "Task 2: The sweep replaces the drift clock"
---

## Notes

- 2026-09-19T10:04:37Z (material-82323e): started
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T10:19:04Z (material-82323e): Deviations from the plan: FocusSweep::is_done honours Clock::should_complete_instantly like Animation (fixtures fast-forward the lap; in production the flag mirrors animations.off, already a cut); the five focused smoke case lines were switched to steady_zero here so the script has no dangling drift.kdl references; focus-ring-light.sh's moving case uses ring-sweep-ms 10000 so its bursts land inside the lap; tools/test_glass_optic_smoke.py fixtures follow.
- 2026-09-19T10:19:04Z (material-82323e): done
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T10:19:04Z (material-82323e): sweep_phase replaces the bucketed drift clock; FocusSweep per tile with start and cut rules; settled focus reports no deadline, constant fingerprint, no transition; uniforms, smoke fixtures and render-pipeline.md follow
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
