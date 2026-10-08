---
id: material-4394f3
title: Guard the material program's idle cost against a kept baseline binary
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-08T09:13:39Z
updated: 2026-10-08T09:13:39Z
depends: []
parent: material-5d6b2c
tags: [performance, harness]
agent: claude-code/claude-opus-5-5
---

The B-spline grain added +0.034 ms to every material draw with noise off (material-03fdf3), and only a review caught it; the cost fixture compared runs across nights, where a common-mode shift passed for host state. noise-layers-cost.sh now has a same-session A/B mode (NOISE_LAYERS_COST_AB). Generalise it: a quiet-lane check that runs the 'none' case (and one active layer) alternating this tree's niri-tracy with the last kept baseline binary, whenever src/render_helpers/shaders/material/ changes, and fails above a set idle delta. Decide where the kept baseline lives and when it advances.
