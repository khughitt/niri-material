---
id: material-3db428
title: Re-derive the ring sample row and reach bound in focus-ring-light.sh and glass-render-order-smoke.sh for ring-gap (measured from the face edge)
status: todo
priority: 3
size: s
complexity: mid
process: direct
needs: [quiet]
created: 2026-09-20T10:34:21Z
updated: 2026-10-03T16:48:43Z
depends: [material-22d78f]
parent: material-2834d7
tags: [rendering]
agent: claude-code/claude-opus-5
---

The ring beam (98013739) moved the filament band from the slab's outer edge to the face edge: `ring-gap` is measured from the face (the slab minus its chamfer), where the retired inset key was measured from the slab's outer edge. The key sweep in the measured old-ring scripts kept the slab-edge derivations: in docs/materials/scripts/focus-ring-light.sh the sample row `FIL_Y = SLAB_TOP + RING_GAP` and the `--inset $RING_GAP` passed to `glass-render-order-metrics.py reach` sit PIN_BEVEL (11) px above the band; in docs/materials/scripts/glass-render-order-smoke.sh `within_ring`, `profile_reach` and `attenuation_reach` pass the gap as the reach model's slab-edge `--inset`, so their bounds and the `profile_reach` row are the bevel (12) px short. The rest-confinement, selectors, resize-flex and within cases will fail or sample the wrong row until the row and `ring_bound` in glass-render-order-metrics.py are re-derived for a face-edge gap (band core at slab edge + bevel + gap; the shader caps the refracted shift at half the gap, not half the slab-edge distance). Neither script is in the ring beam plan's run list (Task 6 runs material-signals-smoke.sh cases and ring-motion-clips.sh), so this is not on that plan's path.

## Notes

- 2026-10-01T09:55:04Z (material-0e80c1): From material-0e80c1: focus-ring-light.sh pins no light-ior, jelly or noise, so it inherits Prism (light-ior 6, jelly-flex 0.0066), and case_tiny's comment cites the retired slabChamfer gate (the shader now gates on hasLine, main.frag). Band core measured at ring-gap inside the face edge; the face sits 2*offset in from the window's left/top and flush right/bottom at offset 6 (brief: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings).
- 2026-10-01T10:00:23Z (materials-26.04): Depends on material-22d78f, which retires case_resize_flex; skip re-deriving that case's row.
