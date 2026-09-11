---
id: material-5f9dee
title: Measure isolated idle power and publish the material budget
status: todo
priority: 2
size: m
created: 2026-09-11T10:28:52Z
updated: 2026-09-11T10:57:07Z
depends: [material-4241c3]
parent: material-265eb0
tags: [performance]
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
step: "Task 3: Measure isolated power and publish the budget verdict"
---

After user resume and trace validation, use an operator-provided dedicated DRM session with complete GPU-client visibility and no other clients. Run uninstrumented shamA/A plus A/B,B/C,B/D repeated comparisons:48 windows,60s warmup+30s observation,1Hz,about72min. Apply proposed1W resolution target and conservative repeat floor, report idle increment and active cost bounds, archive evidence, and close parent only with valid conclusions. Do not log out user, switch VT, stop services, change clocks, or substitute shared-desktop power measurements.

## Notes

- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Prepared only; await user resume, trace validation, and operator-arranged isolated DRM session before power collection.
- 2026-09-11T10:57:07Z (material-265eb0): Preparation review: report sham_floor_w/sham_precision_ok, abba_floor_w, raw_repeat_floor_w, and combined floor_w/precision_ok. The conservative gate intentionally rejects raw repeat drift even when ABBA cancels it; require positive compositor/kitty PID visibility. No power run authorized yet.
