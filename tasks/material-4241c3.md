---
id: material-4241c3
title: Verify material quiescence and bounded animation cadence
status: todo
priority: 2
size: m
created: 2026-09-11T10:27:54Z
updated: 2026-09-11T10:57:07Z
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
