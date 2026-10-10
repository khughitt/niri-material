# Rigid tilt evidence

Design: [transient rigid tilt](../specs/2026-10-09-transient-rigid-tilt-design.md).
Plan: [implementation plan](../plans/2026-10-09-transient-rigid-tilt.md).
Task: `material-6901e0`.

## Rest identity

The `glass_edge` and `ring_look` frozen renders (beam, aberration, reflection,
grain, opaque and translucent content) were dumped at the branch's base
`23039f67` and at `c0e7589b` on `NVIDIA GeForce RTX 3070/PCIe/SSE2`, 137
files: byte-identical (`$EV` = `baseline-20261009T2350`). Within the branch,
`view_tilt` checks that the view branch forced on at θ = 0 and w = 0 renders
byte-identically to rest, and that a frame after a swing equals a never-swung
render.

## Execution rulings

Each of these was decided while executing the plan and refines or narrows a
statement in the spec, which stays as accepted.

1. Spec §4.1, kinds of hit (the wall rule). Ψ at the rim depth is evaluated
   with the quadrant of the piece that contains the rim depth (the earlier
   piece on a boundary), not the entry piece's. With unequal corner radii a
   ray entering through a quadrant seam below the rim was otherwise
   classified as a top hit (52 of 200 rays on the asymmetric fixture). The
   rule is identical at the identity. Commits 7efbb358 (mirror) and 1860780b
   (shader).
2. Spec §4.5, error bound. The bound (1/16 px on sides, 1/2 px on corners) is
   asserted wherever coverage depends on m, that is unless m > 0 and the
   exact value is > 0; NaN is never exempt. Outside the outline at thickness
   200, 20°, κ 1.5 axis swings, the face end of an edge-on wall takes the
   minimiser and m overshoots (up to 0.92 px) where coverage is 0 either way.
   Measured on coverage-relevant points: sides worst 0.0027 px, corners worst
   0.3150 px. Commits 3d040817 and 2c1ca6b1.
3. Spec §4.5, seam continuity. Continuity within a piece is tested by
   refinement rather than a slope limit: a 0.01 px step that moves coverage
   by more than 0.1 is subdivided 1000×, and every sub-step must move it by
   less than 1e-3. The seam's steep but continuous ramp (12.9 per px) is
   allowed. Commit 3d040817.
4. Spec §8.1, entry depth. The tolerance is 1/1024 logical px (1/64 physical
   px up to scale 16); the worst measured is 1.15e-5 px. Commit e4a3a2cf.
5. Spec §8.2, "with grain on". Offscreen comparisons (the alpha path's view
   crop, the close snapshot) run with grain off. Grain seeds from
   gl_FragCoord and moves with the offscreen origin (203 989 px, up to 59
   levels at rest, pre-existing behaviour), so a grain-on comparison cannot
   hold. Commits 2cb538d4 and 4cbddddc.
6. Spec §5 and §5.1, the clock. The swing runs on the unadjusted clock, so
   `animations.slowdown` does not stretch it, as it does the beam. Commit
   d23e336f.
7. Spec §6, culling. A slab that cannot take a swing (zero extent along the
   direction) renders at rest and is culled by its resting rect;
   `Camera::new` asserts a positive extent. Commit 6bc3bade.
8. Spec §8.2, the coverage probe. The −2 probe flags non-finite or
   out-of-range coverage with a sentinel. A non-finite m that smoothstep
   clamps to coverage 1 where the mirror predicts 1 is not flagged, since it
   renders the right pixel. Commit c0e7589b.

## Stills

Sheet, crops and filmstrip: `$EV` = `stills-20261010T0056` (`sheet.png`,
`crops.png`, `filmstrip.png`), rendered by `view_tilt::still_sheet_frames` on
`NVIDIA GeForce RTX 3070/PCIe/SSE2` and assembled by
`docs/materials/scripts/view-tilt-sheet.py`. Cells A1–C3 are θp ∈ {6, 10, 14}°
by κ ∈ {2.5, 6, 10}; S1/S2 the wall spill off and on at B2; K the spike's
control (4° with the camera at 2000 px, κ = 2000 / 1408 px slab width ≈ 1.42).

- **Seeds.** Each sheet fixture restarts the jelly seed sequence
  (`view::hooks::reset_jelly_seed`), so the accepted look's beam head wander
  is the same in every frame; two renders of one cell are byte-identical.
- **Direction.** The second window is 400 × 900, as tall as the pane, so the
  gain's swing is horizontal: direction (−1, 0), the left edge advancing.
- **Walls.** On the pane's middle row at the peak, the left wall is 0, 0, 2 px
  wide at 6°, 0, 2, 2 px at 10° and 1, 3, 5 px at 14° (κ 2.5, 6, 10), from
  the wall probe. crops.png shows every cell's edge at 4×.
- **Spill.** In the B2 swing the head runs along the top from the top-left
  corner while the left edge lifts, and never lies over a wall. S1 and S2
  (30 ms, the head nearest the lifted wall) are identical. A scan during the
  fix round (not part of the test) found spill off and on identical at 10, 30,
  …, 290 ms and at 300, 320, …, 900 ms. Where the head does
  cross a wall, under a forced pose that lifts the top edge (10°, κ 6, 171 ms),
  the spill brightens 1207 wall pixels by up to 11 levels and changes nothing
  else (crops I, J). The render asserts that the spill changes only wall-probe
  pixels and only brightens them.
- **Corner.** Under the forced diagonal pose (10°, κ 6, toward the bottom
  right) the corner wall is 2 px wide on the bottom edge and 1 px on the right.
- **Forced poses.** The pose hook tilts every glass tile, the second window
  included, so every crop stops at x 1448, short of all the second window draws
  in any frame of the sheet; the render asserts the pane lies wholly left of it.

Owner's verdict (2026-10-10): B2 (θp 10°, κ 6), S2 (wall spill on). With S2 `view::wall_spill` keeps its value 1; on a horizontal swing at B2 the spill changes no pixel (the beam head never reaches the lifted wall), so it shows only on swings that lift the top or bottom edge.
