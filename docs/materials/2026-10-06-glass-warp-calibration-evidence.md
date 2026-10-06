# Glass warp calibration: evidence

`material-bb8480`, 2026-10-06. Offline only: no renderer ran.

**Result: adopt, for straight chamfer edges, with a geometry mask.** On a
seeded aperiodic backdrop, a ZNCC block matcher refined by Gauss-Newton
recovers every known chamfer shift from 1 to 20 px to within 0.02 px inside
the strip, and reads photometric-only changes (a ramp, the shader's
chamfer tint and glint) as at most 0.12 px at a 7 px window; a gain that
clips highlights reaches 0.30 px on the windows it does not flag. Across every
case, it trusted no pixel more than 1 px wrong at that window. The 20 px grid
gives it nothing: it refuses every pixel of every case, and the sweep's RMSE
on the grid reads exactly 0 for chamfer shifts of 1, 3 and 20 px. Windows
that straddle the chamfer's edges fit no single shift and are flagged, not
measured, so a measurement reads the strip interior only, located by the
known bevel geometry. This establishes the instrument, not bending: a
rendered control is still needed (*What a render must add*).

## Reproduce

```sh
python3 -I docs/materials/scripts/warp-calibration.py calibrate --out <dir>
python3 -I docs/materials/scripts/warp-calibration.py backdrop <dir>/aperiodic.png
```

Stdlib Python, no ImageMagick. `calibrate` takes about 7 minutes on one
core and writes `results.json` and `results.md` (every case, window,
estimator and region). The tables below are drawn from them.
`tools/test_warp_calibration.py` repeats the core cases on a reduced ROI.

## Inputs

- **Backdrops.** `aperiodic-s20260929`: five octaves of quintic value noise
  at non-integer lattice spacings 2.3 to 31.7 px, hashed from absolute pixel
  position (a crop equals the same region of the full image), contrast 2.4,
  stored as 8-bit sRGB grey. `grid20`: the sweep's `make_backdrop` as
  luminance, four quadrant colours under 1 px lines every 20 px. Both are
  read as their 8-bit PNG would hold them.
- **ROI.** 64x96 at (401, 287), chamfer-sized like the sweep's bevel band:
  backdrop for x < 24, a 12 px chamfer strip at x 24 to 35, flat face beyond.
- **Warps** use the shader's convention, `out(p) = backdrop(p + u(p))` (the
  `tap` in `prelude.frag`), resampled bilinearly and quantized to 8-bit sRGB
  like a capture. *Chamfer s*: `u = (s, 0)` across the strip, zero outside
  and on the face, the field a 45-degree chamfer gives on a straight left
  edge at `distortion 0`. *Rigid*: one shift everywhere. *Smooth*: a 2-D
  field (3 px in x, 1.5 px in y) that varies inside any window.
- **Photometric controls** in linear light: gain 1.35 (clips highlights); a
  horizontal ramp 0.75 to 1.15; *tint+glint*, the shader's terms at a
  45-degree chamfer for attenuation `#dfe8ff` over 60 px at thickness 20,
  per channel and luminance-weighted (face x0.930, chamfer x0.903), plus a
  Schlick glint at ior 1.5 on the chamfer only, at full facing (+0.042).
- **Estimator.** ZNCC over an integer search of ±24 x ±3 px, then either a
  parabola through the peak (*parabola*) or Gauss-Newton on
  `out = g * backdrop(q + u) + b` per window (*gauss-newton*). A pixel is
  flagged, not measured, when its window is flat (std < 0.004), the peak
  sits on the search edge, the best score is under 0.8, a second peak more
  than 1 px away is within 0.02, the refinement leaves the seed's cell, or
  (Gauss-Newton only) the fitted residual exceeds 0.1 of the window's own
  spread (*misfit*).

For scale, the chamfer shift the shader produces (`refract` at the 45-degree
normal times thickness, bevel 12): ior 1.24 at thickness 20 gives 3.6 px,
ior 1.5 at 20 gives 5.8 px, at 80 gives 23.2 px and at 200 gives 58.1 px.

## Estimator error

Aperiodic backdrop, Gauss-Newton. *Confident* is the share of pixels not
flagged; errors are px over confident pixels; *wrong* counts confident pixels
more than 1 px from the known field.

| case | window | all: confident | p95 | max | wrong | strip: confident | max | boundary: confident |
|---|---|---|---|---|---|---|---|---|
| repeat | 7 | 100% | 0.00 | 0.00 | 0 | 100% | 0.00 | 100% |
| rigid 0.5, 0 | 7 | 100% | 0.01 | 0.04 | 0 | 100% | 0.02 | 100% |
| rigid 3.4, -1.6 | 7 | 99% | 0.01 | 0.17 | 0 | 100% | 0.04 | 99% |
| rigid 20, 0 | 7 | 100% | 0.00 | 0.00 | 0 | 100% | 0.00 | 100% |
| chamfer 1 | 7 | 80% | 0.00 | 0.12 | 0 | 100% | 0.00 | 6% |
| chamfer 3 | 7 | 79% | 0.00 | 0.00 | 0 | 100% | 0.00 | 0% |
| chamfer 6 | 7 | 79% | 0.00 | 0.02 | 0 | 100% | 0.00 | 0% |
| chamfer 12 | 7 | 79% | 0.00 | 0.00 | 0 | 100% | 0.00 | 0% |
| chamfer 20 | 7 | 79% | 0.00 | 0.00 | 0 | 100% | 0.00 | 0% |
| smooth 3, 1.5 | 7 | 24% | 0.14 | 0.31 | 0 | 13% | 0.30 | 10% |
| gain 1.35 | 7 | 55% | 0.03 | 0.30 | 0 | 61% | 0.06 | 50% |
| ramp | 7 | 100% | 0.03 | 0.11 | 0 | 100% | 0.05 | 100% |
| tint+glint | 7 | 94% | 0.04 | 0.12 | 0 | 100% | 0.02 | 74% |
| tint+glint, flat-field | 7 | 100% | 0.01 | 0.02 | 0 | 100% | 0.02 | 100% |
| chamfer 3 + tint+glint | 7 | 79% | 0.01 | 0.02 | 0 | 100% | 0.02 | 0% |
| chamfer 6 + tint+glint | 7 | 79% | 0.01 | 0.02 | 0 | 100% | 0.02 | 0% |

**Window size.** 7 px is the working size. At 5 px the estimator keeps
77-89% of the fractional rigid-shift pixels with errors up to 0.76 px, and
trusts two pixels more than 1 px wrong (one at chamfer 1, one off by 20 px
at chamfer 20). At 11 px every error but the smooth field's (0.22 px) falls
under 0.1 px, but only 2 of the strip's 12 columns hold a whole window,
against 6 at 7 px, and the smooth field loses every strip pixel to
*misfit*.

**Parabola alone is not enough.** Without the Gauss-Newton stage the
identical repeat reads 0.21 px at p95 (max 0.47): a parabola through an
asymmetric correlation peak is biased. Worse, with no residual check it
trusts 40 to 83 boundary pixels per chamfer case, up to 41 px wrong.

## Ambiguous regions

- **The chamfer's edges.** A window across a discontinuity in the warp
  holds two shifts, and the shader's step field duplicates `s` columns of
  face content and drops `s` columns of backdrop. At 7 px, 94-100% of
  boundary windows are flagged at every shift; none is trusted wrongly. Mask
  them from geometry: the window rect from the capture (`measure_rect` in
  the sweep) and the configured bevel and offsets place the strip, and the
  usable interior is the strip less half a window at each side (6 columns at
  bevel 12). A bevel narrower than one window leaves no interior.
- **Fields that vary inside a window** (rounded corners, any `distortion`
  above 0) fail the residual check: the smooth field keeps 24% of pixels.
  A local affine-warp model is the fix; `material-79fb49` holds it until a
  measurement needs those regions.
- **Clipped highlights.** Gain 1.35 clips the brightest texels, which no
  gain and offset fits: 45% of windows are flagged and the trusted rest stay
  within 0.30 px. A rendered backdrop should keep its peak below white after
  the glass's gain.
- **The grid.** Every pixel of every case is refused at every window: flat
  between lines, non-unique along a single line, and periodic within the
  search wherever lines cross.

## Supported range

Shifts from 0 to the search radius, 24 px, at 0.02 px inside the strip
interior. The range is the search, not the texture: the field has no repeat,
and the 20 px rigid shift reads exactly. Settings past it (ior 1.5 at
thickness 80 is 23.2 px; at 200 it is 58 px) need a wider search, at linear
cost in run time, and a strip interior still wider than the window.

## Why regional RMSE cannot answer the question

Luminance RMSE in 8-bit code values over the strip, against the unchanged
backdrop:

| case | aperiodic | grid20 |
|---|---|---|
| chamfer 1 | 33.4 | 0.0 |
| chamfer 3 | 68.3 | 0.0 |
| chamfer 6 | 83.7 | 41.8 |
| chamfer 12 | 79.8 | 41.8 |
| chamfer 20 | 86.2 | 0.0 |
| rigid 20, 0 | 86.2 | 0.0 |
| gain 1.35 | 19.1 | 14.2 |
| tint+glint | 16.3 | 17.2 |
| tint+glint, flat-field | 0.8 | 0.5 |
| chamfer 3 + tint+glint | 64.8 | 17.2 |
| chamfer 3 + tint+glint, flat-field | 68.4 | 0.5 |

On the grid, a 1, 3 or 20 px chamfer shift is invisible here: no grid line
enters or leaves the strip. A tint alone scores more than those shifts
(17.2 against 0), so the sweep's grid tables could not have separated
bending from tint. On the aperiodic field RMSE is not monotonic (83.7 at
6 px, 79.8 at 12), and photometric-only changes still score 16 to 19.
Aperiodicity removes aliasing; it does not turn RMSE into displacement.

## What a render must add

The calibration rejects photometric-only change, but a rendered chamfer
also tints and glints on exactly the pixels it bends. The control that
separates them comes from the shader's structure: attenuation
(`main.frag`, Beer-Lambert on the structural normal) multiplies and the
Fresnel glint adds, and neither depends on the backdrop's content. Render
the same glass over two uniform backdrops, `g1` and `g2`, beside the
textured render; per pixel `att = (R2 - R1) / (g2 - g1)` and
`spec = R1 - att * g1`, and `(R - spec) / att` is the backdrop as the glass
displaced it, with Fresnel and attenuation divided out. In the simulation
this takes the tint+glint residual from 16.3 to 0.8 code values while a
3 px warp keeps 68.4, and brings boundary windows from 74% to 100% trusted
at 0.02 px. Any residual displacement left after the correction is bending.

The tempting single controls are confounded: `ior 1` removes bending but
also zeroes Fresnel's `f0`, and `thickness 0` removes the tap's shift but
also flattens the bevel normal (`bevel = min(chamfer, thickness)`), which
changes both Fresnel and attenuation. The flat-field pair changes only the
backdrop. Ring light, aurora, noise, blur and roughness must be off, as in
the sweep baseline, so nothing else depends on the backdrop's content.

## Backdrop for the sweep

`material-8867aa` can reuse this artifact rather than build its own: the
`backdrop` command writes the calibrated field as a 1280x720 PNG with its
seed and octave settings in a `tEXt` chunk. Keep `grid20` available by
name next to it, so the sweep's earlier tables stay comparable, and record
the backdrop name in each run's provenance. The PNG is grey on purpose:
tinting it by the grid's quadrant colours clips the bright quadrants, the
failure the gain case measures.
