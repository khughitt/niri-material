---
id: material-d873bf
title: "Short subtle transients: rotation, light travel and glints on state changes"
status: idea
priority: 2
created: 2026-10-02T00:08:50Z
updated: 2026-10-02T00:14:51Z
depends: []
parent: material-53f873
tags: [dynamics, rendering]
agent: claude-code/claude-opus-5-5
---

Owner priority (2026-10-01): brief, subtle animations on transitions (focus, open, move, resize): light moving across the face, a slight rotation of the light direction, glints running along an edge. These are bounded one-shots like the ring beam, not sustained motion, so they must end and leave the tile quiescent (material-f86183). Overlaps material-6d4de5 (revisit dynamics); scope them together.

## Notes

- 2026-10-02T00:14:51Z (materials-26.04): First concrete case (2026-10-01): a focus-loss transient. Today the ring only animates on focus gain and snaps off on loss; a short drain of the light would reuse the comet's run/decay timing and end in the quiescent state.
