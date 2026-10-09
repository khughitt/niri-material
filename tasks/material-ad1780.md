---
id: material-ad1780
title: Focused glass edges render opaque while unfocused edges stay transparent (glass6)
status: todo
priority: 2
size: s
created: 2026-10-02T19:21:48Z
updated: 2026-10-09T23:29:33Z
depends: []
parent: material-6062fd
tags: [quick-add]
source: "mindful:thought:6258221f80b07e869303a0449ee939cc"
agent: claude-code/claude-opus-5-5
---

With the glass6 setting, unfocused window edges look right (transparent, with fringing) but focused edges look opaque. Find the cause (tint distance or color, or something else) and keep the edge's dynamic range when focused. Near: material-be611b (realistic edges), material-48b514 (edge tint).

## Notes

- 2026-10-09T23:29:33Z (materials-26.04): Cause found 2026-10-09: not a tint colour but the attenuation path on the bevel. rayPath = h / max(-t.z, 0.25) lengthens the path as the structural normal tilts, cancelling the height-field thinning; at the active look's thickness/attenuation-distance ratio 3.5 the whole bevel transmits ~2e-4, while the inactive look's ratio 1.7 still passes light. Fix candidate: clip the attenuation path at the slab's side wall (light near the silhouette exits through the side), which brightens the edge toward the rim without lightening the face. Spiked and then implemented under the sibling tasks filed today.
