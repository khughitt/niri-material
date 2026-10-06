---
id: material-a4d874
title: "Noise layers docs, nested smoke, contact sheet and evidence"
status: doing
priority: 2
complexity: mid
process: direct
needs: [quiet]
owner: material-3fcba2
created: 2026-10-06T10:17:17Z
updated: 2026-10-06T11:24:53Z
started: 2026-10-06T11:19:53Z
depends: []
parent: material-3fcba2
tags: []
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-06-noise-layers.md
step: "Task 4: Docs, the nested-Weston smoke and contact sheet, the evidence document"
---

## Notes

- 2026-10-06T11:19:53Z (material-3fcba2): started
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:21:34Z (material-3fcba2): run: 0.67 min (est 6, headless); preflight 0.67; refused: CPU 16.5% > 10%, load1 4.92 > 2, GPU 34% > 5%, P5 instead of P8; no cells completed; hold rolled back
- 2026-10-06T11:21:34Z (material-3fcba2): parked (waiting on user, quiet; headless, 21 min): Agent: build the 4a8b2072 baseline in .worktrees/material-3fcba2-baseline, run the smoke pilot (NOISE_LAYERS_PILOT=1, ~6 min: build 4, cells 2), then the full run (~15 min: cells 15), then write and attach the evidence document; prior preflight refused on CPU/load/GPU P5, no cells ran
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:23:32Z (material-3fcba2): Preparation complete: layer and pipeline docs, smoke script, position-class helper, shared capture helpers; synthetic bins min/max 0.9585/1.0415 and LF 0.2489, broken helper exits 1 without stale values; full tooling 409 passed with 2 optional retained-binary skips. Baseline worktree created and release build tracked by this session.
- 2026-10-06T11:24:53Z (material-3fcba2): Validation detail: full tooling ran 409 cases, 407 passed and 2 optional retained-binary checks skipped; preflight record: /mnt/ssd3/niri-material/noise-layers-preflight-1791285645/capture.json
