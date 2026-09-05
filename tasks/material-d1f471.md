---
id: material-d1f471
title: "Explore a lit, dynamic focus ring to replace the static gradient ring"
status: done
priority: 3
size: l
owner: explore/focus-ring-light
created: 2026-09-05T01:10:50Z
updated: 2026-09-05T20:40:04Z
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

## Notes

- 2026-09-05T09:14:25Z (explore/focus-ring-light): spike approved: six candidates as throwaway branches inside material.frag behind an env-var selector plus active and time uniforms; harness nests the worktree binary on headless weston with two transparent kitty windows, captures stills and a frame burst per candidate; write-up ranks looks, idle redraw cost, and animations-off degrade path. Owner refinement: the ring of light is embedded in the slab and refracted by it, not a surface band, with slow movement so it interacts with different parts of the glass.
- 2026-09-05T10:16:06Z (explore/focus-ring-light): spike done: write-up docs/materials/2026-09-05-focus-ring-light-spike.md, harness docs/materials/scripts/focus-ring-light.sh, probe patch niri-experiments fixtures/focus-ring-light-probe.patch (results/slice3 8e31521, not pushed), captures $NIRI_MATERIAL_WORK_ROOT/focus-ring-light-fcdc5ed4. Ranking: ring of light > organic noise (a mode of the ring) > shadow (free complement) > canopy (attention response material, not focus) > motes > rays. Baseline static ring is invisible on the dark focus glass. All animating candidates cost 60 draws/s while focused-idle; HZ quantization of the fingerprint does not throttle under a per-frame animation flag, a timer is needed. Winner scoped as a child.
- 2026-09-05T13:07:32Z (materials-26.04): niri-experiments probe patch commit rewritten to 536992b (trailer removed); results/slice3, not pushed
- 2026-09-05T17:14:09Z (design/ring-light): winner shipped as material-26dd8a
- 2026-09-05T20:40:04Z (materials-26.04): winner shipped as material-26dd8a
