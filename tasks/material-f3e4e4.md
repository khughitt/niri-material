---
id: material-f3e4e4
title: "Sunlight: cloudy, dim periods before the sun breaks through"
status: idea
priority: 2
reviewed: true
created: 2026-09-06T22:27:52Z
updated: 2026-10-17T02:05:01Z
depends: [material-415d0c]
parent: material-53f873
tags: [rendering, lighting]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
---

Sunlight as a slow weather cycle rather than a steady state: stretches of cloudiness and relative darkness, then the sun starts to poke through. Inspiration: a misty morning. Extends the sunrise item in the lighting spike (material-1c5a30) and the time-of-day ambient baseline (material-f41c54).

## Notes

- 2026-10-06T18:41:53Z (materials-26.04): scope: briefed; organic lighting and weather have no established visual or temporal contract; material-ffd61f will map existing optics to one bounded experiment before implementation; brief: docs/notes/2026-09-29-material-dynamics-brief.md
- 2026-10-15T22:15:20Z (research/material-ffd61f): finding (material-ffd61f): smallest experiment is one finite overcast-to-sun Aurora transition on an in-place definition change; a static warm Aurora preset works now (config only); the one missing behavior is an envelope easing Aurora amount and colors to the new values, which needs a reviewed design (planned). A weather cycle is a clock driving such transitions and waits on that look and the clock decision. Proposal: docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility-material-ffd61f
- 2026-10-17T02:05:01Z (materials-26.04): scope: briefed; a weather cycle is a clock driving finite transitions, so it waits on the Aurora envelope design material-415d0c (now a dependency) and then on the clock decision (activity, real time or event) in the brief's unanswered questions; brief: docs/notes/2026-09-29-material-dynamics-brief.md
