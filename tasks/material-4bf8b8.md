---
id: material-4bf8b8
title: "Glass response: frost-on-idle"
status: idea
priority: 2
size: m
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T21:32:24Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, rendering]
---

Outcome: roughness rises slowly while a window sits at Quiet and clears with a wipe on focus or Active, using the existing prefilter pyramids. Decide the idle timer ownership (solver vs. slot decay) and the wipe geometry. Acceptance: response name in the glass vocabulary, prefilter cache reuse unchanged, no redraws once fully frosted. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; distinguished Quiet duration from existing input-inactivity gate; growth and wipe semantics still need design; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
