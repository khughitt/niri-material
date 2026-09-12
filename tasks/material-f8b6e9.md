---
id: material-f8b6e9
title: "Behind hook: noise and saturation"
status: todo
priority: 2
size: m
complexity: high
created: 2026-09-06T00:32:53Z
updated: 2026-09-12T20:26:18Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, noise]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
step: "Task 1: Behind hook: noise and saturation"
---

Implement the reviewed behind hook for noise and saturation as Task 1 of the render-order plan. Preserve signed sRGB formulas, neutral branches, inheritance and opaque bypass; isolate additive-light effects with quantization-aware comparisons. Transmitted chamfer grain is accepted without a mask. Record face/bevel grain and frame cost and update stage/spec docs in the same implementation commit. Implementation waits for written-plan review.

## Notes

- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review the two-step render-order implementation plan before starting Task 1.
