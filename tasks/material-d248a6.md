---
id: material-d248a6
title: Full idle-budget trace on the fixed binary after the ring-focus merge
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T03:56:41Z
updated: 2026-09-28T04:57:01Z
started: 2026-09-28T03:56:45Z
completed: 2026-09-28T04:57:01Z
depends: []
parent: material-53f873
tags: [performance]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

The last full idle-budget trace (trace-20260925T220712) ran on d91a6024, before the ring-focus merge. The 2026-09-27 pilots on 4607d054 found the attention idle-edge redraw (material-cd7deb, fixed in 721b8df7) that failed every quiet case. Run the full fail-fast trace from a TTY with the desktop stopped on NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target-20260927T234054 (source 721b8df7): idle-budget-supervise.sh just --justfile <fixtures>/idle-budget.just trace. Pass: every case verdict passes, analysis complete, no stop.json, SHA256SUMS ok. Record the result in docs/materials/2026-09-11-idle-budget-evidence.md.

## Notes

- 2026-09-28T03:56:45Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T04:57:01Z (materials-26.04): run: 58 min (est 60, headless); trace 58; passed: 25/25 cases (A, B, P, O quiet at 0 redraws / 0 material draws / pixels equal, B-long-move-1 600 s hold 0, C 80 x3, D 40 x3), analysis complete, no stop.json, SHA256SUMS ok, no leftover processes (trace-20260927T235810, binary trace-target-20260927T234054 from 721b8df7)
- 2026-09-28T04:57:01Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T04:57:01Z (materials-26.04): Full idle-budget trace on the fixed binary passed all 25 cases in 58 min; evidence doc updated
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
