---
id: material-2f2af4
title: Noise layers cost captures with Tracy
status: doing
priority: 2
complexity: mid
process: direct
needs: [quiet]
owner: material-3fcba2
created: 2026-10-06T10:17:17Z
updated: 2026-10-06T11:24:53Z
started: 2026-10-06T11:21:34Z
depends: []
parent: material-3fcba2
tags: []
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-06-noise-layers.md
step: "Task 5: Cost captures with Tracy"
---

## Notes

- 2026-10-06T11:21:34Z (material-3fcba2): started
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:23:32Z (material-3fcba2): Cost script and shared wallpaper cleanup prepared; focused cleanup/report tests 4 passed, full tooling 409 passed with 2 optional retained-binary skips. Captures not attempted: the smoke preflight measured GPU 34% at P5 plus CPU/load over limits; no environment change or retry.
- 2026-10-06T11:23:32Z (material-3fcba2): parked (waiting on user, quiet; headless, 18 min): Agent: run noise-layers-cost.sh pilot (~8 min: build 4, six cases 4), then full (~10 min: cases 10), then add the Cost section to the smoke evidence document; smoke preflight refused CPU/load/GPU P5, no cost cases attempted
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:24:53Z (material-3fcba2): Validation detail: full tooling ran 409 cases, 407 passed and 2 optional retained-binary checks skipped
