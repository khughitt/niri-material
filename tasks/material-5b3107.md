---
id: material-5b3107
title: "Render pass order: what the glass slab refracts"
status: doing
priority: 2
size: m
complexity: high
owner: material-5b3107
created: 2026-09-06T00:38:34Z
updated: 2026-09-12T21:10:25Z
started: 2026-09-12T19:55:05Z
depends: []
tags: [rendering, design]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
---

Several open questions are one architectural question: in what order the passes run (backdrop blur, glass refraction and bevel, ring of light, noise and saturation postprocess) and therefore what the slab refracts and what lands on top of it. Noise on the glass edges (material-f8b6e9) and the ring embedded inside the glass (material-92edaf) both hinge on it. Document the current order first, then decide the target order once for both.

## Notes

- 2026-09-12T19:24:33Z (materials-26.04): Complexity high: The current pass order is documented in render-pipeline.md, but the target order for edge noise and an embedded ring remains an architectural decision shared by the two open exploratory children.
- 2026-09-12T19:58:07Z (material-5b3107): Drafted depth-ordered hooks for review. Source review found whole-bevel zero grain, a strict two-width ring cutoff, and unchanged rendered brightness are not guaranteed; spec proposes measurable replacements and defines depth from back to front.
- 2026-09-12T19:58:07Z (material-5b3107): parked (waiting on user, review): Review the written spec, especially corrected grain/reach gates and lookup-depth convention; then write the implementation plan.
- 2026-09-12T20:18:34Z (material-5b3107): Spec review accepted for planning, including transmitted chamfer grain and no mask. Preparing one sequential two-step plan using the existing children.
- 2026-09-12T20:26:18Z (material-5b3107): Plan self-review derived 37/61/61 px one-code rest bounds, specified quantization-aware additive comparisons, and corrected the cap-binding threshold claim. Reused both existing children sequentially; no implementation or captures run.
- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review docs/plans/2026-09-12-material-render-order.md; then execute the two existing children sequentially with executing-plans.
- 2026-09-12T20:35:43Z (material-5b3107): Approved plan execution began inline. Task 1 is environment-blocked before shader edits by retained capture preflight render-order-readiness.pCbsOC; offline metrics prepared. Task 2 remains dependent on Task 1.
- 2026-09-12T20:35:43Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 after the capture host is quiet; preserve the required old-build additive failure and later wide-core positive face-strip gate.
- 2026-09-12T21:06:29Z (material-5b3107): Second capture readiness run 03bDro refused; independent process/GPU queries confirm BitwigStudio remains running after its window was closed. Task 1 remains gated before shader changes.
- 2026-09-12T21:06:29Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 when Bitwig has fully exited and the capture host passes default quietness checks.
- 2026-09-12T21:10:25Z (material-5b3107): Bitwig closure verified; after a settling interval CPU, load and power variance pass. Task 1 remains blocked only on GPU utilization/P-state in readiness TFm260.
- 2026-09-12T21:10:25Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 when desktop GPU activity passes the unchanged preflight thresholds; no regression capture has run.
