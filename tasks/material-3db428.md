---
id: material-3db428
title: Re-derive the ring sample row and reach bound in focus-ring-light.sh and glass-render-order-smoke.sh for ring-gap (measured from the face edge)
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-09-20T10:34:21Z
updated: 2026-09-20T10:34:21Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5
---

The ring beam (98013739) moved the filament band from the slab's outer edge to the face edge: `ring-gap` is measured from the face (the slab minus its chamfer), where the retired inset key was measured from the slab's outer edge. The key sweep in the measured old-ring scripts kept the slab-edge derivations: in docs/materials/scripts/focus-ring-light.sh the sample row `FIL_Y = SLAB_TOP + RING_GAP` and the `--inset $RING_GAP` passed to `glass-render-order-metrics.py reach` sit PIN_BEVEL (11) px above the band; in docs/materials/scripts/glass-render-order-smoke.sh `within_ring`, `profile_reach` and `attenuation_reach` pass the gap as the reach model's slab-edge `--inset`, so their bounds and the `profile_reach` row are the bevel (12) px short. The rest-confinement, selectors, resize-flex and within cases will fail or sample the wrong row until the row and `ring_bound` in glass-render-order-metrics.py are re-derived for a face-edge gap (band core at slab edge + bevel + gap; the shader caps the refracted shift at half the gap, not half the slab-edge distance). Neither script is in the ring beam plan's run list (Task 6 runs material-signals-smoke.sh cases and ring-motion-clips.sh), so this is not on that plan's path.
