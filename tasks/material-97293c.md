---
id: material-97293c
title: Render the type in the material shader
status: done
priority: 2
size: s
owner: glass-noise-type
created: 2026-09-07T01:39:52Z
updated: 2026-09-07T09:55:12Z
depends: [material-e0efec]
parent: material-6e7352
tags: [rendering, noise]
plan: docs/plans/2026-09-06-material-glass-noise-type.md
step: "Task 2: Render the type in the material shader"
---

mat_noise_type float uniform from ResolvedGlass, fineGrain and Oklab helpers, three-way noise branch with the white branch unchanged, damage and resolver pins.

## Notes

- 2026-09-07T09:26:09Z (glass-noise-type): took over session sid:1654439 (owner glass-noise-type, host titan, pid 1654439, worktree /mnt/ssd/Dropbox/niri-material/.worktrees/glass-noise-type, since 2026-09-07T09:25:23Z, age 46s, stale: pid 1654439 is gone)
- 2026-09-07T09:32:47Z (glass-noise-type): Rendered white, fine, and Oklab-lightness glass noise through the material shader; added resolver and damage regression pins; updated pipeline documentation.
- 2026-09-07T09:55:12Z (glass-noise-type): Final review corrected the pipeline table: glass.noise remains mapped; the noise type integration is pending prism-51f23b.
