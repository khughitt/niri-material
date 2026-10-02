---
id: material-519eeb
title: Pin the owner's accepted ring look as a reference before ring/edge changes land
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-10-02T00:14:51Z
updated: 2026-10-02T00:14:51Z
depends: []
tags: [rendering, harness]
agent: claude-code/claude-opus-5-5
---

Owner-accepted look (2026-10-01, live a18ca619): ring-beam-speed 4350, noise 0.55 @ 12 Hz, decay 4150, ring-gap 6, ring-width 1.1, ring-glow 1.2, light-ior 4.5, color #ccccff, on the owner's Prism glass. Add a Glass row with these values to src/tests/ring_pair.rs so RING_PAIR_DUMP gives a before/after sheet for any change to ring or edge rendering, starting with material-be611b (glow follows the bevel profile) and material-1d70db (resting level). Prism half: save the same values as a named profile so experiments can be reverted; see prism-c45c6a.
