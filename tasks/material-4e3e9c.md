---
id: material-4e3e9c
title: "Embed the ring in the glass: refraction, scatter and noise interaction"
status: idea
priority: 2
created: 2026-10-02T00:08:50Z
updated: 2026-10-10T13:42:56Z
depends: [material-be611b]
parent: material-6062fd
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
---

Owner priority (2026-10-01): the ring should read as light inside the glass, not a line drawn on it. Today the band sits on the flat face (light-ior only moves its halo on the chamfer, capped at half ring-gap), so the glass barely touches it. Candidates: glass noise/grain modulating the band, the band scattering into the face (frost spill), the beam lighting the chamfer as it passes, refraction of the band through the edge profile from material-be611b. Scope after be611b's edge profile lands, because the band follows innerDist/chamfer.

Scope context (2026-10-06): The current ring already uses a refracted landing point, roughness scattering, attenuation at 20% depth and moving-beam edge spill (main.frag); film-site grain also reaches it. Establish the residual visual gap after material-be611b is accepted and merged before proposing another light model. Reuse the frozen accepted_ring_look reference and keep the settled beam free of new clocks.

## Notes

- 2026-10-06T18:28:46Z (materials-26.04): scope: briefed; refraction, scatter and moving-beam spill already exist; establish the residual visual gap via material-f4143a; brief: docs/notes/2026-10-06-glass-optics-brief.md
- 2026-10-09T23:29:33Z (materials-26.04): 2026-10-09: one concrete residual gap identified from the owner's crop: at rest the ring's bevel spill is zero (beam-only), so the line never touches the edge. Filed as material-58a1a1, gated on spike material-ac926f.
- 2026-10-10T13:42:56Z (materials-26.04): scope: briefed; residual gap narrowed: resting bevel spill is material-58a1a1 (gated on spike material-ac926f's owner verdict); noise and frost-scatter interaction still waits on material-f4143a, whose sheet must include the live looks; brief: docs/notes/2026-10-06-glass-optics-brief.md
