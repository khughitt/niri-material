---
id: material-58a1a1
title: Ring resting glow spills into the bevel
status: todo
priority: 2
size: xs
complexity: low
process: direct
created: 2026-10-09T23:29:33Z
updated: 2026-10-09T23:29:33Z
depends: [material-ac926f]
parent: material-6062fd
tags: [rendering, signals]
source: material-4e3e9c
agent: claude-code/claude-fable-5-1
---

main.frag scales the ring's bevel spill by the beam term alone (moving), so once the beam has passed the line sits on the face with a dark gap of bevel outside it and reads as a separate stroke. Change: spill = focus * BEAM_BASE * ringGlow * BEAM_SPILL * (moving + BEAM_REST * ringRest) * (1 - u), the same sum focusGlow uses, so the resting line leaks into the edge along u. ring.rs mirrors the constants; add a frozen ring_look assertion that the chamfer carries resting light proportional to ring-rest and none at ring-rest 0 with the beam over; settled focus still reports no deadline. Re-pin the accepted ring look with the owner. Proceed only if the spike's sheet reads as one slab.
