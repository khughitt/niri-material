---
id: material-f8b6e9
title: Should glass edges be affected by noise?
status: idea
priority: 2
created: 2026-09-06T00:32:53Z
updated: 2026-09-06T00:38:34Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, noise]
---

Noise applied after the glass pass grains the bevel and edge highlights, which seems to break the glass appearance. Options: mask noise out of the edge band, or apply noise before the glass pass so the slab refracts an already-grainy backdrop and its edges stay clean. Compare both in the smoke harness.
