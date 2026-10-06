---
id: material-bb3fe5
title: "Material: ice"
status: todo
priority: 2
size: m
complexity: mid
needs: [quiet]
created: 2026-09-06T22:27:52Z
updated: 2026-10-03T16:53:06Z
depends: [material-397fcb]
parent: material-3aa1f2
tags: [material, rendering]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec section 7.1 and 8: the cracks optic (normal + specular hooks, cellular edge pattern, scale= property), the resources/materials/ice.kdl preset tuned on the headless harness, install plumbing for /usr/share/niri/materials/ in PKGBUILD and the rpm/deb asset lists, the docs section, and a smoke recording cost. Frost grain comes from backdrop-blur, roughness, and lightness noise that already exist.

## Notes

- 2026-09-11T00:22:25Z (material-f0fc7b): resources/materials/ and its install plumbing landed with material-f0fc7b; ice adds ice.kdl to PKGBUILD package() and both Cargo.toml asset lists
- 2026-09-12T19:24:31Z (materials-26.04): Complexity mid: Optics design sections 7.1 and 8 define the cracks algorithm, hooks, bounds, preset, and smoke criteria; the optic registry and preset installation pattern exist. Gradient implementation and visual tuning remain bounded choices.
