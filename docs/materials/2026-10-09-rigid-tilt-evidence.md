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

After the cost A/B failed the rest bound, the view path moved into a second
program variant (design §6 "Contingency"; plan Task 14 Step 4): the rest
program is the base's shader text, the files stripped of their
`#ifdef MATERIAL_VIEW` blocks diff empty against `23039f67`'s, and the view
variant is bound only while a swing runs. The same renders dumped at the commit
`perf(material): rigid tilt view path as a second program variant` (parent
`2f86b3ac`) on `NVIDIA GeForce RTX 3070/PCIe/SSE2`, 137 files, are
byte-identical to the base dumps above (`$EV` = `contingency-20261016T0749`).
`view_tilt`'s identity check now compares the two programs: the forced
identity pose binds the view variant, and each render's material draws record
the program they bound, so the check asserts the rest program for the rest
render and the view variant for the forced one.

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

## Cost

`docs/materials/scripts/view-tilt-cost.sh` (design §8.5): a same-session A/B
of this tree's `niri-tracy` (a) against `niri-tracy-23039f67`, built at the
branch's base (b), alternating a, b, a, b. Each rest case replaces the
wallpaper 20 times at 1 Hz with the live look and the beam off; the bound is
a's median material draw within 1 % of b's or 0.002 ms, whichever is larger.
All runs on 2026-10-16, TTY with the desktop stopped, clean capture
verdicts. The medians are later-mode draws at the GPU's P8 idle clock (the
clock attribution experiment in the noise layers evidence); the three-step
pilots ran inside the start-up P0 burst and are not compared with these.

Before the contingency (`2f86b3ac`, `cost-full-1792146671`), the single
program failed the bound by about 13 %:

| Case | a1, a2 (ms) | b1, b2 (ms) | a − b (ms) | Bound (ms) | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| `view-tilt 0` | 0.7465, 0.7613 | 0.6574, 0.6564 | 0.0970 | 0.0066 | INCONCLUSIVE (a1/a2 apart) |
| `view-tilt 10`, no swing | 0.7465, 0.7485 | 0.6564, 0.6559 | 0.0914 | 0.0066 | FAIL |
| one fine noise layer | 0.8182, 0.8151 | 0.7301, 0.7311 | 0.0860 | 0.0073 | FAIL |

With the view path in its own program variant (`perf(material): rigid tilt
view path as a second program variant`, `cost-full-1792152702`), the rest
path costs what the base does:

| Case | a1, a2 (ms) | b1, b2 (ms) | a − b (ms) | Bound (ms) | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| `view-tilt 0` | 0.6569, 0.6584 | 0.6569, 0.6554 | 0.0015 | 0.0066 | PASS |
| `view-tilt 10`, no swing | 0.6615, 0.6564 | 0.6595, 0.6579 | 0.0003 | 0.0066 | PASS |
| one fine noise layer | 0.7291, 0.7332 | 0.7306, 0.7316 | 0.0000 | 0.0073 | PASS |

The swing case (a only, `view-tilt 10`, κ 6, six focus gains 3 s apart)
records the material draws within 0.9 s of each gain: 330 frames, median
1.466 ms, maximum 1.951 ms, and 32.4 ms of GPU time per gain above the
`view-tilt 10` rest median (0.659 ms). Before the contingency the same case
read median 1.425, maximum 1.713 and 25.7 ms per gain. The swing figures carry
no bound.

Start-up: `Shaders::init` now compiles the material program four times (the
rest program and the view variant, each with smithay's debug twin). Timed on
2026-10-16 on `NVIDIA GeForce RTX 3070/PCIe/SSE2` (surfaceless EGL as the
headless backend builds it, three rounds each) with a throwaway test, not kept:

| Program | Cold (`__GL_SHADER_DISK_CACHE=0`) | Warm (disk cache) |
| --- | ---: | ---: |
| all of `Shaders::compile` | 11.6–12.1 s | 61–75 ms |
| material rest program | 4.5–4.8 s | 23–24 ms |
| material view variant | 6.7–7.4 s | 31–35 ms |
| the single program before the split (`2f86b3ac`'s files) | 7.4 s | — |

The split adds about 4.5 s to a cold start, which the driver pays once after
any change to the shader text or the driver, and about 23 ms to a warm one.

## Clip

Four swings at the owner's pick (`view-tilt 10; view-distance 6`) on the
nested lane: `$EV` = `ring-motion-clips-a29c11e9/ring-clips-1973688-1792292030`
(`tilt-swing`, `tilt-swing-beam`, `tilt-swing-slow` at 1400 ms, and
`tilt-swing-flipped` with the sign reversed), each a 2.5 s burst from before
the gain with 4x corner crops at 0.15, 0.3, 0.45 and 0.7 s. Each burst took
25 or 26 frames; the crops landed 0.122–0.128, 0.317–0.332, 0.413–0.427 and
0.703–0.738 s after the gain. Every sub-run settled with its renderer
verified, and the release scan was clean. `swing-review-sheet.png` lays the
16 crops out by sequence and instant, and each sequence's GIF shows the motion.

Owner's verdict: pending.
