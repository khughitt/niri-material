---
id: material-f8b6e9
title: "Behind hook: noise and saturation"
status: idea
priority: 2
size: m
complexity: high
created: 2026-09-06T00:32:53Z
updated: 2026-09-12T19:58:07Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, noise]
spec: docs/specs/2026-09-12-material-render-order-design.md
---

Move saturation and noise from post to behind, once on the averaged linear backdrop before attenuation, retaining sRGB formulas, neutral branches and inheritance. Define signed colour conversion safely. Update grain and saturation evidence and pipeline/optic docs. Draft spec proposes isolating additive-light grain from transmitted bevel grain; whole-bevel zero noise is not guaranteed. Await written-spec review and implementation plan before implementation.
