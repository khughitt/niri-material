---
id: material-a91346
title: "Cost-tiered inactive glass: concessions land on unfocused windows first"
status: idea
priority: 2
created: 2026-09-11T23:39:09Z
updated: 2026-09-29T21:43:52Z
depends: []
parent: material-5d6b2c
tags: [performance, rendering]
---

Unfocused windows are most of what is on screen, so the cheapest place to save is glass.inactive.*: lower blur/roughness/noise tiers there while the focused window keeps the full optic. Use material-31074f's focused/unfocused cost split to choose the tiers and the perceptual metric to bound the visible change. Related: prism-d54be4's focus-weighting note, prism-d2b31a (focus as modulation), prism-7e4766.

## Notes

- 2026-09-29T21:43:52Z (materials-26.04): scope: briefed; focus-split materials already exist, but cheaper inactive settings need measured whole-scene savings and visual acceptance; reuse material-31074f rather than invent tiers; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
