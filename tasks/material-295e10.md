---
id: material-295e10
title: Art-directed rim width for glint and reflection on low-ior glass
status: idea
priority: 2
created: 2026-10-09T23:29:33Z
updated: 2026-10-10T13:42:56Z
depends: []
parent: material-6062fd
tags: [rendering, material]
agent: claude-code/claude-fable-5-1
---

At ior 1.28 Schlick's f0 is 1.5 % and the (1 - cos)^5 term lights only the last one or two pixels of the bevel, so the glint and the reflection optic (which rides the same Fresnel) are invisible except as a sliver at the silhouette. Candidate: a rim term with its own width (a lower exponent, or a falloff on u) that the glint and reflection use in place of raw Schlick, neutral at the current behaviour. Hold until side-exit attenuation and resting spill are judged: each extra term on the bevel risks adding another parallel band, which is the defect being fixed.

## Notes

- 2026-10-10T13:42:56Z (materials-26.04): scope: drop; f72d5c (edge light, retitled in 3d66b8a6 after this idea was filed) already replaces raw Schlick with a silhouette-peaked rim weight for the reflection optic; the glint's rim weight is left to f72d5c's design; brief: docs/notes/2026-10-06-glass-optics-brief.md; proposal: drop as covered by material-f72d5c
