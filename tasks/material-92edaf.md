---
id: material-92edaf
title: "Within hook: ring and aurora"
status: idea
priority: 2
size: m
complexity: high
created: 2026-09-06T00:32:54Z
updated: 2026-09-12T19:58:07Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, ring]
spec: docs/specs/2026-09-12-material-render-order-design.md
---

Move ring and aurora to within at depth fraction 0.8, retaining att^0.2, removing the ring mask and evaluating both at refracted landing points. Preserve ring light-ior, aberration and half-inset cap. Define lookup depth consistently; replace face-zero gates with measured bounded reach and record motion bleed. Evaluate existing motion before adding animation. Update pipeline, optic docs and evidence. Await written-spec review and implementation plan; Gaussian tails and uncapped channel offsets require the reviewed reach criterion.
