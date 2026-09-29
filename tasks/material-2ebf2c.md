---
id: material-2ebf2c
title: Perceptual difference metric for visual-vs-cost trade-offs
status: idea
priority: 2
created: 2026-09-11T23:39:09Z
updated: 2026-09-29T21:43:51Z
depends: []
parent: material-5d6b2c
tags: [performance, testing]
---

To trade a slight visual change for a large performance gain, quantify 'slight'. Render fixed backdrops (the parameter-sweep harness, material-8867aa's aperiodic backdrop) at full and reduced settings and score the difference with a perceptual metric (SSIM, LPIPS, or similar) alongside the cost from material-31074f. The pair (visual delta, cost delta) is the decision surface the resource-aware goal asks for.

## Notes

- 2026-09-29T21:43:51Z (materials-26.04): scope: briefed; existing Lab RMSE measures image change but periodic backdrop aliases displacement and no acceptability threshold is established; reuse per-element measurements before choosing a metric; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
