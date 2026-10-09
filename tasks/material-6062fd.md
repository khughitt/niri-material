---
id: material-6062fd
title: "Glass optics: edges, depth and the ring inside the slab"
status: todo
priority: 1
lane: true
created: 2026-10-02T23:07:45Z
updated: 2026-10-09T23:29:33Z
depends: []
tags: [rendering]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
---

Give glass edges, content depth and the embedded ring a shared optical foundation. First milestone: implement the height-field bevel (material-be611b) and have the owner review its appearance.

The edge model (planar chamfer, global thickness, Schlick) underlies the opaque focused edge, the surface-drawn ring and content that floats above the glass. Done when its children land or are dropped; the height-field bevel (material-be611b) is the shared prerequisite. Host: pixel work on the headless fixture; owner judges sheets.

Ring embedding (material-4e3e9c) follows it; prepare its comparison scenes and acceptance criteria meanwhile.

## Notes

- 2026-10-09T23:29:33Z (materials-26.04): Diagnosis 2026-10-09 from the owner's crop (edges flat, material layered): three parallel constant-width features (12 px bevel band, inset ring line, hard silhouette) with nothing varying across the band. Cause: stage 4's path h / -t.z lengthens as the bevel normal tilts by about as much as the thinning shortens it, so at the live active look (att #4c563a over 9 px, thickness 31.2) the bevel transmits 2e-4 at face, mid and silhouette alike; the k=2 profile and reflection 0.6 act only through Schlick f0 1.5 % on the last 1-2 px. The ring's bevel spill is beam-only, so at rest line and band never touch. Next milestone: side-exit attenuation (path clipped at the side wall) and resting ring spill, spiked first on the live looks.
