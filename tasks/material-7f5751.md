---
id: material-7f5751
title: "Glue terminal content into the glass: shared film, content depth, halation, sidechain"
status: idea
priority: 2
created: 2026-09-30T23:27:46Z
updated: 2026-10-06T18:28:46Z
depends: [material-be611b]
parent: material-6062fd
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
---

Put window content (text, graphics) and the glass in the same optical world, like bus compression and saturation glue tracks in a mix.

Options (2026-09-30): G1 shared film: a post stage after the composite with one screen-seeded grain field and a soft-knee tone curve over window and glass together. G2 content depth: the window sits at a depth inside the slab, seen through the face normal (distortion and jelly bend it), attenuated by the glass above it, with face sheen over it; if content joins the transmitted path before the behind hooks, grain and saturation land on text and glass alike. G3 halation: a blurred, attenuation-tinted copy of the content scattered into the glass, scaled by roughness. G4 sidechain: glass ducks (dims or blurs the backdrop) where content is dense, or content brightness follows the backdrop. G3 and G4 share a blurred low-res window texture.

Lean: G2 core with the G1 tone curve as a companion knob. Every option breaks the v1 section 2 contract (opaque pixels bypass, main.frag early return) for opted-in windows, as material-987655 does; design the opt-in content stage once for both. G2 needs the local height field from material-be611b. Cost: opted-in windows lose the opaque early return.

Scope context (2026-10-06): Film-site noise is before the window composite and after the opaque-pixel return, so it does not implement G1 shared client/glass film. A reviewed opt-in content-stage design is still needed. Coordinate coverage and bypass semantics with material-987655 without reparenting that idea; preserve the default opaque-pixel contract, readability and single application of window opacity.

## Notes

- 2026-10-06T18:28:46Z (materials-26.04): scope: briefed; film still bypasses opaque clients; shared opt-in content-stage design filed as material-d257d9; brief: docs/notes/2026-10-06-glass-optics-brief.md
