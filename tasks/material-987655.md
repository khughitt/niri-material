---
id: material-987655
title: Inactive desaturation post-process
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-10-06T20:15:24Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, rendering]
---

Outcome: decide and, if accepted, implement a desaturation of the window texture, not the glass, for windows at Quiet or inactive, so focus reads even when the material is quiet. Must compose with the existing background-effect postprocess (see material-cad932). Acceptance: control surface, compositing order, and cost recorded before code. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; confirmed backdrop saturation does not alter client pixels; trigger and client coverage require a visual decision and reviewed design; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
- 2026-09-30T23:27:51Z (materials-26.04): related: material-7f5751 designs one opt-in content stage (breaking the opaque-pixel contract) that this desaturation could share
- 2026-10-06T20:15:24Z (materials-26.04): scope: briefed; backdrop saturation cannot alter client pixels; reuse material-d257d9 for the opt-in content boundary, with Quiet versus unfocused trigger and coverage still requiring reviewed design; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
