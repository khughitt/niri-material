---
id: material-8db3b0
title: "Aurora optic: slow colour field on the generalised drift clock, plus the aurora preset"
status: done
priority: 2
size: m
created: 2026-09-10T09:33:23Z
updated: 2026-09-11T01:22:16Z
depends: [material-397fcb]
parent: material-f0fc7b
tags: [material, rendering]
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec 7.3: block node with drift-hz and two color children, phase_on/next_boundary_on generalising drift over a 600 s period, emissive hook, next_change drives redraws at drift-hz per second, motion policy through drift_rate. Preset resources/materials/aurora.kdl. Record redraw rate against drift-hz and frame cost.

## Notes

- 2026-09-11T01:22:16Z (material-f0fc7b): Aurora optic landed: emissive hook on the 600 s clock, redraws track drift-hz (4/s at 4 Hz), cost 5.552 ms vs 4.308 ms plain (+28.9% llvmpipe)
