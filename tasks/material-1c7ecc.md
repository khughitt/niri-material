---
id: material-1c7ecc
title: "Glass edge render coverage: ring and aurora at bevel-profile 2, distorted profile over a pattern"
status: todo
priority: 3
size: s
complexity: mid
created: 2026-10-03T11:31:46Z
updated: 2026-10-03T11:31:46Z
depends: [material-be611b]
tags: [rendering, material]
agent: claude-code
---

Gaps the final review of material-be611b found in the frozen-clock renders (src/tests/glass_edge.rs, src/tests/ring_pair.rs). (1) No render covers ring spill or aurora at bevel-profile > 1, where steep rim normals enlarge the per-channel aberration offsets: add a k=2 variant to ring_cap_keeps_one_core, or a ring-on cell to the contact sheet. (2) The profile test's distorted case renders identical ratios to plain on the flat backdrop, so liftTapNormal and height-scaled tap displacement are never exercised by a render: run the distorted case over the Task 7 checkered Background layer. (3) REFLECTION_MOTION_MIN = 38 was calibrated on one GPU; if CI's renderer fails it, strengthen the stimulus and recalibrate, never lower the constant.
