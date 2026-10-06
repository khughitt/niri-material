---
id: material-933a8b
title: Alternate glass pane x-offset per column so adjacent edges point toward or away from each other
status: shelved
priority: 2
created: 2026-09-06T00:32:54Z
updated: 2026-10-06T18:28:46Z
depends: []
parent: material-6062fd
tags: [layout, rendering]
---

Option to flip the sign of the pane x-offset on alternating columns so neighboring bevels point toward (or away from) each other. Back burner: generalize to zebra-striping columns by other subtle properties. The downside is complexity unless it can be one small abstraction (symmetry, tiling, or auto-nudge parameters) rather than per-property switches.

## Notes

- 2026-10-06T18:28:01Z (materials-26.04): shelved: After the height-field bevel is accepted and merged, the owner requests a concrete alternating-column edge look; scope parity under column insertion, reorder and floating windows before adding controls.
- 2026-10-06T18:28:46Z (materials-26.04): scope: shelved; back-burner layout variation kept with an explicit wake condition; brief: docs/notes/2026-10-06-glass-optics-brief.md
