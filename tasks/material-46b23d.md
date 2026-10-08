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
updated: 2026-10-08T14:54:19Z
depends: [material-d21ff0]
parent: material-5d6b2c
tags: [capture, performance]
agent: claude-code/claude-opus-5-5
---

Run the optic mode prepared by the preparation task: pilot (optic-visible, optic-covered-idle) through verdict, then the matrix. Record redraws, Tile::render, OffscreenBuffer::render and material draws per case in a results section of docs/materials/2026-10-08-culling-damage-boundaries-audit.md (or a new evidence doc linked from it), and note the outcome on material-7afc31 and material-7f6d0e. Estimate: build 2, pilot ~5, matrix ~8 min.
