---
id: material-ef9eec
title: Verify and document roughness delivery
status: done
priority: 2
size: m
owner: feat/material-c854bd
created: 2026-09-02T03:14:58Z
updated: 2026-09-02T10:08:20Z
depends: [material-46f6b7]
tags: [rendering]
plan: docs/plans/2026-09-01-material-roughness.md
step: "Task 5: Verify and document roughness delivery"
---

## Notes

- 2026-09-02T03:38:53Z (feat/material-c854bd): User reference and implementation status updated; task remains doing for focused Weston/Tracy evidence and application of Prism handoff commit f1a65dd to the requested read-only checkout.
- 2026-09-02T03:49:52Z (feat/material-c854bd): Nested Weston GLES smoke at b220152d passed for roughness 0/0.08/0.5, overview, and worst-case optics with no fallback warning; retained under NIRI_MATERIAL_WORK_ROOT/material-roughness-b220152d. Tracy, idle/damage counts, and calibrated overview measurement remain.
- 2026-09-02T04:02:11Z (feat/material-c854bd): Tracy acceptance remains open: niri embeds Tracy 0.13.1/protocol 76, installed capture/export are 0.14.0 and reject the client; a bounded matching-tool build was stopped before producing data. All owned profiler/compositor processes and sockets were cleaned up.
- 2026-09-02T10:08:20Z (feat/material-c854bd): Prism landed on main at c3c459d. Matching Tracy 0.13.1 traces pass the 80-second idle reuse gate, two-source damage regeneration, and matched 14-sample GPU timing; the calibrated 0.5 overview differs by at most 0.491 source px against a 2 px tolerance.
