---
id: material-4241c3
title: Verify material quiescence and bounded animation cadence
status: doing
priority: 2
size: m
owner: material-265eb0
created: 2026-09-11T10:27:54Z
updated: 2026-09-12T10:01:44Z
started: 2026-09-12T09:57:45Z
depends: [material-ec6229]
parent: material-265eb0
tags: [performance]
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
step: "Task 2: Verify quiescence and bounded active cadence"
---

After execution resumes and the fixture is validated, collect 24 short trace observations and one 600 s quiet hold. Verify exact quiet redraw/material-draw zeros, three-second settling and inter-stimulus wait, heartbeat coverage, pixel return, and 4/2 Hz Aurora cadence. Match the complete startup/setup/stimulus IPC journal to trace markers and retain independently reviewable raw evidence. No board-power claim from the shared desktop.

## Notes

- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Prepared only; await user resume and completed fixture task before trace collection.
- 2026-09-12T09:59:44Z (material-265eb0): User resumed trace execution on 2026-09-12. Integrated capture protocol into materials-26.04 at a2b9c58c, experiment results/idle-budget at 9d7d43f, and native execution worktree via 8e09f91b. Readiness preflight precedes expensive build/capture; no threshold overrides or user-session changes.
- 2026-09-12T10:01:44Z (material-265eb0): Readiness on 2026-09-12T06:00:38-04:00 sampled 20 seconds after integration tests finished: exit 1; CPU 25.5% > 10%, load1 12.89 > 2.0, GPU 34.5% > 5%, mixed P5/P8, power IQR 5.64 W > 1.0 W. Graphics clients: Xwayland, firefox, kitty, niri, noctalia, qs; no compute clients. Release exited 0. Retained capture.json and logs under NIRI_MATERIAL_WORK_ROOT/material-265eb0/readiness-20260912-15msz4ks. No release build or compositor capture started.
- 2026-09-12T10:01:44Z (material-265eb0): Merged verification passed: 434 Rust/unit/doc tests, 126 tooling tests, 22 idle-budget fixture tests.
- 2026-09-12T10:01:44Z (material-265eb0): parked (waiting on user, environment): Resume the authorized trace build and approximately 50-minute capture after a real headless readiness preflight passes on a quiet desktop. Keep default thresholds; retained refusal record readiness-20260912-15msz4ks explains current load. Power still requires the operator-arranged dedicated session.
