---
id: material-ce4134
title: Drop the invented face hue for neutral glass in the surface remap
status: todo
priority: 2
size: xs
complexity: low
process: direct
created: 2026-10-17T14:42:48Z
updated: 2026-10-17T14:42:48Z
depends: []
parent: material-55380c
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

Final review of material-62347d (deferred minor): RemapSetup::with_face (src/render_helpers/surface_remap.rs) treats face chroma > 1e-9 as a hue direction. For neutral glass the Oklab b-row coefficients sum to 3.7e-8, so the spec's #191919 / IOR 1 example gets face chroma 4.6e-9 and face_dir ≈ (0, 1): the outline takes ΔC ≈ 0.012 toward yellow. Use a perceptual threshold (~1e-4) and add a test against #191919.
