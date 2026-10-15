---
id: material-1c7ecc
title: "Glass edge render coverage: ring and aurora at bevel-profile 2, distorted profile over a pattern"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-03T11:31:46Z
updated: 2026-10-15T18:30:58Z
depends: [material-be611b]
tags: [rendering, material]
agent: claude-code
---

Why: material-be611b's final review found two gaps in the frozen-clock renders (src/tests/glass_edge.rs, src/tests/ring_pair.rs). Both now cover the live look: the owner picked bevel-profile 2 as the Prism starting value (material-be611b note, 2026-10-06), and the edge light (material-f72d5c) and resting ring spill (material-58a1a1) will change the bevel at that profile.

Open (checked 2026-10-15 against b212c1a0):
1. No render covers ring spill or aurora at bevel-profile > 1, where steep rim normals enlarge the per-channel aberration offsets. ring_pair.rs ring_cap_keeps_one_core runs its five glasses (STOCK, BINDING, IOR102, IOR124, IOR150) at the default profile 1, and glass_edge.rs cases() pairs bevel-profile 2 only with RING_OFF (the 'on' cases with AURORA_ON and RING_ON use profile 1). Add a k=2 variant to ring_cap_keeps_one_core, or a ring-on k2 case appended at the end of cases().
2. The profile test's distorted case (the_rounded_rim_is_rounded_at_rest_mid_resize_and_mid_scroll, 'distortion 1 scale=0.5') renders identical ratios to plain on the flat backdrop, so liftTapNormal and height-scaled tap displacement are never exercised by a render. Run the distorted case over the checkered Background layer: patterned_backdrop() (glass_edge.rs near 567) already exists and is used only by the reflection test.

Established, no work: REFLECTION_MOTION_MIN = 38 (calibrated on one GPU) passed on CI's renderer in run 38158694977 (2026-10-11, 477446ff; all 547 niri unit tests ran, 1 ignored, not this one). The rule stands: if a later CI renderer fails it, strengthen the stimulus and recalibrate, never lower the constant.

Done: both renders exist and fail when their effect is removed (profile forced to 1 for item 1; tap displacement zeroed for item 2), GLASS_EDGE_DUMP writes the new cases, just test-fast is green.

## Notes

- 2026-10-15T18:30:58Z (materials-26.04): scope: rescoped; premises checked against b212c1a0: no ring/aurora render at bevel-profile 2 (ring_cap_keeps_one_core at profile 1, k2 cases RING_OFF) and the distorted profile case still on a flat backdrop (patterned_backdrop used only by the reflection test); REFLECTION_MOTION_MIN item answered by CI run 38158694977 (477446ff, passed), kept as a rule; process direct, size/complexity unchanged
