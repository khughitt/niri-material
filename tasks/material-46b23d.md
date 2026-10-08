---
id: material-46b23d
title: Capture a sustained optic under an opaque cover (hidden-window optic mode)
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet, nested]
created: 2026-10-08T14:54:18Z
updated: 2026-10-08T16:27:09Z
depends: [material-d21ff0]
parent: material-5d6b2c
tags: [capture, performance]
agent: claude-code/claude-opus-5-5
---

Run the optic mode prepared by the preparation task: pilot (optic-visible, optic-covered-idle) through verdict, then the matrix. Record redraws, Tile::render, OffscreenBuffer::render and material draws per case in a results section of docs/materials/2026-10-08-culling-damage-boundaries-audit.md (or a new evidence doc linked from it), and note the outcome on material-7afc31 and material-7f6d0e. Estimate: build 2, pilot ~5, matrix ~8 min.

Also rerun the existing offscreen-column case in the same session, since material-7afc31 landed: the client should stay at the 1.0 fps fallback and the material OffscreenBuffer::render count for the hidden tile should fall to 0 (the audit's falsifying check for the cull). Adds about 1 min.

## Notes

- 2026-10-08T16:27:08Z (materials-26.04): scope: folded in the offscreen-column rerun that verifies material-7afc31 on the host; same fixture and quiet session.
