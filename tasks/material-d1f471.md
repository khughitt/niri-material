---
id: material-d1f471
title: "Explore a lit, dynamic focus ring to replace the static gradient ring"
status: todo
priority: 3
size: l
created: 2026-09-05T01:10:50Z
updated: 2026-09-05T01:10:50Z
depends: []
tags: [material, focus-ring]
---

The current focus ring is a static 4px gradient (#ffffff00 to #ccccff11). Explore replacing it with a subtle 3D lighting treatment on the focused tile that reads as light on glass rather than a drawn border, using the recent niri animation/dynamics work as the motion source. Candidates to prototype, each as a throwaway shader behind a debug flag, screenshot/video captured on the headless nested host:
- ring of light: soft emissive rim that breathes with the jelly residuals
- particles: sparse drifting motes along the rim
- god rays: directional volumetric streaks from the bevel
- shadow-based: focus expressed only through shadow depth/softness, no ring
- organic noise: slow domain-warped noise modulating rim brightness
- canopy light: dappled light-through-leaves pattern moving across the pane
Output: a ranked write-up with captures; the winner becomes a scoped child task with a config surface. Constraints: must stay subtle, degrade to the static ring when animations are off, and not regress the DRM acceptance gates.
