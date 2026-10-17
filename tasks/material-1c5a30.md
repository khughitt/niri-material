---
id: material-1c5a30
title: "Lighting spike: sunrise, lanterns, fireflies, warm organic light"
status: idea
priority: 2
created: 2026-09-06T22:07:49Z
updated: 2026-10-17T02:04:44Z
depends: [material-415d0c]
parent: material-53f873
tags: [rendering, lighting, spike]
source: "mindful:thought:fee58ae2c54b4072ad2b0ba02a2cf1d1"
---

Explore lighting effects for the glass materials beyond the ring of light:
- sunrise (ties to material-f41c54, ambient baseline from time of day)
- flickering lantern(s)
- fireflies (material-54bcac already covers the signal-driven version; this spike is the broader lighting family)
- warm / cozy; organic / nature-inspired
- interact with the mouse?

Spike shape: enumerate options, measure performance cost, plan experiments. Related directions: clouds, fluids.

## Notes

- 2026-10-06T18:41:53Z (materials-26.04): scope: briefed; organic lighting and weather have no established visual or temporal contract; material-ffd61f will map existing optics to one bounded experiment before implementation; brief: docs/notes/2026-09-29-material-dynamics-brief.md
- 2026-10-15T22:15:14Z (research/material-ffd61f): finding (material-ffd61f): smallest experiment is one finite overcast-to-sun Aurora transition on an in-place definition change; a static warm Aurora preset works now (config only); the one missing behavior is an envelope easing Aurora amount and colors to the new values, which needs a reviewed design (planned). A weather cycle is a clock driving such transitions and waits on that look and the clock decision. Proposal: docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility-material-ffd61f
