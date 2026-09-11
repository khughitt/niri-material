---
id: material-f0fc7b
title: "Material: rainbow glass, glass + aurora, particles"
status: done
priority: 2
size: l
owner: material-f0fc7b
created: 2026-09-06T22:27:52Z
updated: 2026-09-11T01:39:21Z
depends: []
tags: [material, rendering]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
spec: docs/specs/2026-09-10-material-optics-design.md
plan: docs/plans/2026-09-10-material-aurora-iridescence.md
---

Goal for spec sections 7.2, 7.3, and 8: the aurora and iridescence optics and their presets. Particles are deferred to the lighting spike (material-1c5a30) and fireflies (material-54bcac); they need a sprite pass, not a per-fragment stage.

## Notes

- 2026-09-10T23:00:48Z (material-f0fc7b): parked (waiting on user): User reviews docs/plans/2026-09-10-material-aurora-iridescence.md; on approval pick an execution mode and start Task 1 (material-b42087) in the material-f0fc7b worktree
- 2026-09-10T23:06:45Z (material-f0fc7b): parked (waiting on user): Plan revised after review (6f888762): user confirms and picks an execution mode; then start Task 1 (material-b42087) in the material-f0fc7b worktree
- 2026-09-11T01:39:21Z (materials-26.04): Iridescence and aurora optics with the rainbow and aurora presets, packaged; particles deferred to material-1c5a30 and material-54bcac
