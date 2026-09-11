---
id: material-6102b2
title: "Iridescence optic: thin-film hue on the Fresnel glint, plus the rainbow preset"
status: done
priority: 2
size: s
created: 2026-09-10T09:33:23Z
updated: 2026-09-11T00:55:29Z
depends: [material-397fcb]
parent: material-f0fc7b
tags: [material, rendering]
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec 7.2: specular hook, cosine palette from the view angle, runs before the signal accent mix. Preset resources/materials/rainbow.kdl pairs it with chromatic-aberration. Record cost.

## Notes

- 2026-09-11T00:55:29Z (material-f0fc7b): Iridescence optic implemented: specular hook, rainbow preset, llvmpipe cost 3.918 ms vs 4.070 ms plain (-3.7%)
