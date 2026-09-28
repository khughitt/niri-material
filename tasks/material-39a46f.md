---
id: material-39a46f
title: Full idle-budget power run on the current binary
status: doing
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T08:21:20Z
updated: 2026-09-28T08:21:25Z
started: 2026-09-28T08:21:25Z
depends: []
parent: material-53f873
tags: [performance]
agent: claude-code/claude-opus-5-5
---

The last full power run (power-full-20260927T031335, 48 windows, 93 min) used a binary from 191ad747, before the ring-focus merge and the attention idle-edge fix (material-cd7deb, 721b8df7). The trace lane was re-verified on 721b8df7 (material-d248a6); the power lane was only piloted (pilot-power-20260927T231857 on 4607d054: sham block 1 passed, 11.39 to 11.47 W). Build a power binary from current HEAD (build-power, about 3 min with a warm cache) and run the full power lane from a TTY with the desktop stopped: idle-budget-supervise.sh just --justfile <niri-experiments results/idle-budget>/fixtures/idle-budget.just power with POWER_OUTPUT=DP-1 POWER_MODE=3440x1440@59.999 POWER_SCALE=1. Launched from an agent shell, start the supervisor with env --default-signal=INT. Record the result in docs/materials/2026-09-11-idle-budget-evidence.md.

## Notes

- 2026-09-28T08:21:25Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T08:21:25Z (materials-26.04): parked (waiting on user, quiet; headless, 100 min): From a TTY with the desktop stopped: check out niri-experiments results/idle-budget in a worktree, build-power from niri-material HEAD (3 min), wait for load1 < 1.0, then run the full supervised power lane (about 95 min; 48 windows took 93 min on 2026-09-27). Pilot passed 2026-09-27 on 4607d054. Agent then validates verdicts, analysis, SHA256SUMS, files the run note, and updates the evidence doc
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
