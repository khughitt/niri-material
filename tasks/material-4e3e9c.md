---
id: material-4e3e9c
title: "Embed the ring in the glass: refraction, scatter and noise interaction"
status: idea
priority: 2
created: 2026-10-02T00:08:50Z
updated: 2026-10-02T23:07:56Z
depends: []
parent: material-6062fd
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
---

Owner priority (2026-10-01): the ring should read as light inside the glass, not a line drawn on it. Today the band sits on the flat face (light-ior only moves its halo on the chamfer, capped at half ring-gap), so the glass barely touches it. Candidates: glass noise/grain modulating the band, the band scattering into the face (frost spill), the beam lighting the chamfer as it passes, refraction of the band through the edge profile from material-be611b. Scope after be611b's edge profile lands, because the band follows innerDist/chamfer.
