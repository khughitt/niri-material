---
id: material-f72d5c
title: "Side-exit attenuation: clip the bevel's Beer-Lambert path at the slab's side wall"
status: todo
priority: 1
size: m
complexity: mid
process: planned
created: 2026-10-09T23:29:33Z
updated: 2026-10-09T23:29:33Z
depends: [material-ac926f]
parent: material-6062fd
tags: [rendering, material]
source: material-ad1780
agent: claude-code/claude-fable-5-1
---

Stage 4 attenuates along h / -t.z, as if the slab extended sideways forever, so a tilted bevel normal lengthens the path and the bevel transmits less than the face: at the live active look the band is uniformly black (2e-4) and the height-field profile cannot show. Physically, light entering the bevel near the silhouette leaves through the slab's side, so its path is bounded by the distance to the side wall along the ray. Design: define the attenuation path as min(h / -t.z, side-wall distance along the refracted ray), using outerDist and acrossDir from slabSurface; taps keep their landing rule; the face is byte-identical (its side-wall distance exceeds its height). Amend docs/specs/2026-09-30-glass-edge-optics §3.1 ray model and render-pipeline.md stage 4; mirror in bevel.rs with a test pinning the face unchanged and the rim brighter than the face edge; regenerate pipeline.json if a stage read changes. Evidence: frozen dumps before/after (face identical, outside identical, bevel brighter toward the silhouette), and an owner-judged sheet on the live looks. Proceed only if the spike's sheet reads as one slab.
