---
id: material-265eb0
title: Idle GPU and power budget for a never-static glass
status: doing
priority: 2
size: m
owner: material-265eb0
created: 2026-09-11T00:52:15Z
updated: 2026-09-11T11:47:34Z
depends: []
parent: material-53f873
tags: [quick-add, dynamics, performance]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
---

Verify existing material dynamics become quiescent after finite move/resize stimuli, then establish repeatability-bounded isolated board-power cost for settled dynamics and low-rate Aurora. Existing fingerprint/deadline code already supports rest; the proposed roughly300s idle micro-movement is not implemented by this task. Fixture preparation is complete on experiments results/idle-budget at 8db5dc8, with offline tests and review. Release builds and trace collection require user resume; power collection additionally requires an operator-provided dedicated DRM session with no other GPU clients. Proposed rules and precision target are in the attached spec. Reuse material-36e968 motion evidence and material-300b87 trace tools; coordinate redesign with material-6d4de5 and reporting with prism-d54be4.

## Notes

- 2026-09-11T09:04:58Z (material-300b87): Aurora/iridescence hardware evidence (docs/materials/2026-09-11-material-hardware-evidence.md) records 18 idle traces: exact 0/4/2 Hz cadence, but whole-board run medians 13.720–28.960 W and varying clocks prevent attributable power deltas. Use an isolated workload for the broader budget; no optic regression established.
- 2026-09-11T10:28:52Z (material-265eb0): Preparation only, per user: design and3-task execution plan written; no fixture code, build, compositor launch, or GPU sampling. Existing rest fingerprint/deadline mechanisms verified in source; proposed rules are2s settle,20s and600s covered quiet intervals, and1W isolated-power resolution target. Power stage needs operator-arranged dedicated DRM session (~72min). Execution remains paused for review.
- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Review docs/specs/2026-09-11-material-idle-budget-design.md and docs/plans/2026-09-11-material-idle-budget.md, especially2s settling,1W precision target, and dedicated-session requirement; execution explicitly paused at user request.
- 2026-09-11T10:57:07Z (material-265eb0): Preparation review supersedes the earlier 2 s proposal: both docs now use 3 s settling and inter-stimulus wait, full startup IPC marker accounting, separate sham/cancelled/raw power floors, GPU-client positive controls, and absolute sampler deadlines. Extracted reducer assertions and spring math checked offline; no fixture implementation or measurement.
- 2026-09-11T10:57:38Z (material-265eb0): parked (waiting on user): Preparation review addressed in both docs: 3 s settling and inter-stimulus wait, startup marker accounting, separate power floors, visibility controls, and absolute sampling deadlines. Await explicit user resume before starting any execution child.
- 2026-09-11T11:02:29Z (material-265eb0): Second preparation review: pixel-return uses the pre-stimulus settled image and takes the after image and post-observation geometry actions only after Tracy capture exits. Marker matching counts successful IPC calls only; validation rejections are retained separately, and unknown outcomes abort. Execution remains paused.
- 2026-09-11T11:02:29Z (material-265eb0): parked (waiting on user): Both marker-journal clarifications are documented; await explicit user resume before starting execution children.
- 2026-09-11T11:47:34Z (material-265eb0): parked (waiting on user): Fixture material-ec6229 complete at experiment 8db5dc8. Await user resume of material-4241c3 to build Tracy binary and collect traces; material-5f9dee still requires a later isolated-session checkpoint.
