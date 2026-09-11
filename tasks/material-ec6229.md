---
id: material-ec6229
title: Build and validate the idle-budget measurement fixture
status: todo
priority: 2
size: m
created: 2026-09-11T10:27:39Z
updated: 2026-09-11T10:30:13Z
depends: []
parent: material-265eb0
tags: [performance]
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
step: "Task 1: Build and validate the bounded measurement fixture"
---

Implement and test the bounded trace/power fixture and offline interval/repeat reducer in niri-experiments after the user resumes execution. Reuse existing host/Tracy/motion helpers; validate identity, coverage, full case inventory, power noise floors, and process ownership. No measurements during preparation; dedicated power session remains a later operator checkpoint.

## Notes

- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Prepared only; await explicit user resume of material-265eb0 before fixture implementation.
