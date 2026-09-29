---
id: material-a85a18
title: Revisit the filament shift cap versus light-ior for dense glass
status: idea
priority: 2
created: 2026-09-05T17:14:14Z
updated: 2026-09-29T22:38:51Z
depends: []
parent: material-49871a
tags: []
---

The render-order change shortened the filament's remaining path from 0.6 to
0.2 of thickness. The half-`ring-inset` cap remains a high-shift safety limit,
but no longer saturates stock glass or Prism's `ior 1.24` at every
`light-ior` value: at `light-ior 1` those examples are roughly 1.16 px and
1.54 px, respectively, against the default 2.5 px cap. Revisit the cap only
where a dense or high-`light-ior` setting actually reaches it. Decide whether
the cap, a bevel-relative depth, or an auto light-ior is the intended model,
with captures at ior 1.02, 1.24, and 1.5. Take one capture at ior 1.24 /
bevel 9 once the deterministic flex probe (material-22d78f) exists so the
owner's own glass has evidence.

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; current shader caps shared shift at half ring-gap with a 0.2-thickness path; old numeric examples need recomputation before any cap redesign; brief: docs/notes/2026-09-29-glass-measurement-brief.md
