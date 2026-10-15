---
id: material-bb3fe5
title: "Material: ice"
status: todo
priority: 2
size: m
complexity: mid
process: direct
created: 2026-09-06T22:27:52Z
updated: 2026-10-15T18:40:19Z
depends: [material-397fcb]
parent: material-3aa1f2
tags: [material, rendering]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
spec: docs/specs/2026-09-10-material-optics-design.md
---

Why: ice is the material-optics spec's remaining preset (docs/specs/2026-09-10-material-optics-design.md §7.1 cracks optic, §8 ice preset; the spec's Status line lists both open, with Prism follow-ups prism-b35d40 and prism-fc791c waiting). Original scope: the cracks optic (normal + specular hooks, cellular edge pattern, scale= property), the resources/materials/ice.kdl preset tuned on the headless harness, install plumbing for /usr/share/niri/materials/, the docs section, and a smoke recording cost. Frost grain comes from backdrop-blur, roughness and lightness noise, which exist.

Established 2026-10-15 (2dd3b9fa): resources/materials/ and its install plumbing landed with material-f0fc7b; the PKGBUILD installs *.kdl by glob, so only the two Cargo.toml asset lists need ice.kdl. The optic API from material-397fcb (done) is the authoring path; ice stays one glass definition plus optics under parent material-3aa1f2, not a layer composition. The rest of the changed premises are in the first child.

Decomposed:
- material-19dd35: the cracks optic and ice preset, tuned headless with an owner-judged sheet (no quiet host).
- material-718ec9: the smoke with recorded draw cost (quiet), after material-19dd35.

Done when both children are done; this task closes them out.

## Notes

- 2026-09-11T00:22:25Z (material-f0fc7b): resources/materials/ and its install plumbing landed with material-f0fc7b; ice adds ice.kdl to PKGBUILD package() and both Cargo.toml asset lists
- 2026-09-12T19:24:31Z (materials-26.04): Complexity mid: Optics design sections 7.1 and 8 define the cracks algorithm, hooks, bounds, preset, and smoke criteria; the optic registry and preset installation pattern exist. Gradient implementation and visual tuning remain bounded choices.
- 2026-10-15T18:40:19Z (materials-26.04): scope: rescoped; premises checked at 2dd3b9fa: spec §7.1/§8 still open and unsuperseded; changed: PKGBUILD glob (only Cargo.toml lists), specular hook takes Surface, normal hook has no call site yet, pipeline.rs stage + Interaction required, no cellular noise to reuse (~18 hash12), §8 values predate the height-field bevel; decomposed into material-19dd35 (optic+preset, headless, owner sheet) and material-718ec9 (cost smoke, quiet); quiet moved to the cost child; process direct (closeout)
