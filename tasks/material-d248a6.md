---
id: material-d248a6
title: Full idle-budget trace on the fixed binary after the ring-focus merge
status: doing
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T03:56:41Z
updated: 2026-09-28T03:56:45Z
started: 2026-09-28T03:56:45Z
depends: []
parent: material-53f873
tags: [performance]
agent: claude-code/claude-opus-5-5
---

The last full idle-budget trace (trace-20260925T220712) ran on d91a6024, before the ring-focus merge. The 2026-09-27 pilots on 4607d054 found the attention idle-edge redraw (material-cd7deb, fixed in 721b8df7) that failed every quiet case. Run the full fail-fast trace from a TTY with the desktop stopped on NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target-20260927T234054 (source 721b8df7): idle-budget-supervise.sh just --justfile <fixtures>/idle-budget.just trace. Pass: every case verdict passes, analysis complete, no stop.json, SHA256SUMS ok. Record the result in docs/materials/2026-09-11-idle-budget-evidence.md.

## Notes

- 2026-09-28T03:56:45Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
