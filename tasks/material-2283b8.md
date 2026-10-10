---
id: material-2283b8
title: GPU cost A/B on the capture lane
status: doing
priority: 1
size: m
complexity: mid
process: direct
needs: [quiet, nested]
owner: material-abc08c
created: 2026-10-09T21:41:52Z
updated: 2026-10-10T11:47:04Z
started: 2026-10-10T11:24:14Z
depends: [material-7e3447, material-abc08c]
parent: material-6901e0
tags: [rendering, camera]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-09-transient-rigid-tilt.md
step: "Task 14: GPU cost A/B on the capture lane"
---

Step 14 of the transient rigid tilt plan (docs/plans/2026-10-09-transient-rigid-tilt.md); spec docs/specs/2026-10-09-transient-rigid-tilt-design.md.

## Notes

- 2026-10-10T11:24:14Z (material-abc08c): started
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-10T11:40:38Z (material-abc08c): run: 13 min (est 12, headless); build 8 (release 3.4, tracy 4.4), load wait 5, preflight not reached; refused: await_load, load1 stayed at or above 1.0 for 5 min after the build (owner on the desktop); no capture_preflight, no sub-run started
- 2026-10-10T11:47:02Z (material-abc08c): review: impl round 1 — verdict: accept; findings: Minor 7; reviewer: claude-code
- 2026-10-10T11:47:02Z (material-abc08c): review: impl round 2 — verdict: accept; findings: none; reviewer: claude-code
- 2026-10-10T11:47:02Z (material-abc08c): parked (waiting on user, quiet; headless, 40 min): On a quiet host with the desktop stopped (headless lane), from .worktrees/material-abc08c: run the pilot, OUT=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/cost-pilot-$(date +%s) CAPTURE_TASK=material-2283b8 VIEW_TILT_COST_AB=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/niri-tracy-23039f67 VIEW_TILT_COST_PILOT=1 docs/materials/scripts/view-tilt-cost.sh (build ~1 warm/8 cold, load wait ≤5, preflight 1, 12 rest sub-runs + swing ~9), read metrics.txt so every gate ran; then the same with VIEW_TILT_COST_PILOT=0 (~20); record a run: note per attempt; judge per Task 14 Step 3 (PASS: fill the evidence Cost section and the performance.md §7 figures; INCONCLUSIVE: rerun; FAIL: Step 4 contingency). Last attempt (2026-10-10) refused at await_load: load1 stayed ≥ 1.0 for 5 min with the owner on the desktop; the per-gain swing deadlines have not run live yet
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
