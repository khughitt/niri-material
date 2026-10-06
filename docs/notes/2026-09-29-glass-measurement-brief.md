# Glass displacement and motion: scoping brief

Updated 2026-10-06. Handoff for `material-49871a`, not an approved design.

## Problem

Measure glass displacement without confusing it with tint, glint, repeated
backdrops or capture skew. This pass re-scopes `material-343f27` and
`material-8867aa` after calibration, and dispositions the new affine-model
idea `material-79fb49`. Earlier ring investigations and their implementation
are complete; they do not authorize a broader renderer redesign.

## Current behaviour and evidence

### Matched-state ring findings

`material-0e80c1` established frozen-time surfaceless rendering with real
client shm buffers; `material-22d78f` subsequently implemented the deterministic
mid-flex check. `src/tests/ring_pair.rs::render_at` returns RGBA pixels and
holds the clock immediately before rendering. The completed `material-a85a18`
kept the filament shift cap. [Detailed ring findings](2026-10-06-glass-measurement-findings.md#matched-state-ring-findings)
retain their measured bounds, captures and decisions.

### Warp calibration findings

`material-bb8480` adopted ZNCC plus Gauss-Newton for straight chamfer strips
at a 7 px window (`479a3a8f`). Tested shifts from 1 to 20 px recover to
0.02 px in the strip interior; geometry-crossing windows are excluded.
The [calibration evidence](../materials/2026-10-06-glass-warp-calibration-evidence.md)
records confidence, photometric controls and limitations. Smooth fields fail
many residual checks; the search is finite and boundary peaks are rejected.
These are synthetic results, not a rendered bending measurement.

`warp-calibration.py backdrop` already writes the deterministic
`aperiodic-s20260929` grayscale field with seed/octave provenance. The sweep
still generates only its colored 20 px grid. Aperiodicity removes periodic
aliasing but does not make RMSE monotonic or distinguish bending from
photometric change. Grayscale does not excite saturation.

The shader attenuates transmitted light and adds Fresnel light in linear
space. Paired uniform-backdrop renders can estimate both terms before measuring
a textured render, provided geometry, coverage, material state and clock match.
The current in-process fixture can host a bounded investigation; the real
rendered correction and its error remain unvalidated.

## Constraints

Preserve the [render pipeline](../materials/render-pipeline.md), opaque-client
bypass and existing accepted ring appearance. Restrict initial bending work to
full-coverage straight-strip interiors, away from corners and warp discontinuities;
reject clipping, low recovered gain and unsupported matches. Keep source,
renderer, backdrop identity/hash, seed, geometry and clock with results. Do not
infer displacement or acceptable quality from RMSE. Cost remains
`material-31074f`'s subject. Offline wiring and frozen-pixel checks need no
quiet host; they do not establish timing or board power.

## Alternatives

1. **Current lean: reuse the calibrated grayscale field and translation model.**
   Offer it explicitly beside `grid20`, and validate one rendered straight edge
   with paired flat fields before claiming bending.
2. Tint the field or replace the grid everywhere. Tinting clipped highlights
   in calibration; grayscale would remove the stimulus from saturation sweeps.
   Keep the existing colored grid as the default and for those comparisons.
3. Extend to affine fitting now. Shelve this until a concrete measurement
   requires corners or distortion and demonstrates the translation model's limit.

## Unanswered questions

- Does per-channel linear flat-field correction isolate displacement in actual
  rendered pixels, with adequate confidence and repeatability? `material-98763f`
  answers on a small ROI and records its own error, rather than borrowing 0.02 px.
- Are corners or nonzero distortion necessary for that measurement? Only a
  demonstrated need wakes `material-79fb49`; straight edges need no affine fit.
- Backdrop color and adoption are settled for this task: use the calibrated gray
  option for displacement work, retain `grid20` for color-sensitive measurements.
  The accepted-look fixture is a separate future consumer, not part of this wiring.

## Proposed decomposition

- `material-8867aa`: **scoped**, P2, small, mid complexity, direct. Add explicit
  backdrop selection, deterministic artifact reuse, retained table identity and
  offline checks; preserve original grid output/table columns, reject unknown
  names and grayscale saturation, and keep old unlabelled provenance explicit.
- `material-343f27`: **briefed**, awaiting the rendered-control result.
- `material-79fb49`: **shelved**, waking only when a rendered measurement needs
  corners/distortion and the calibrated translation model fails there.

### Rendered flat field

- `material-98763f`: P2, medium, high complexity, direct research. Render a small
  matched set over two uniform inputs and the calibrated texture, decode/correct
  per channel, and quantify straight-edge displacement against independent optical
  expectations and a photometric-only control. No production shader change or live
  capture. Completion updates this brief and notes `material-343f27` in the same commit.
- <a id="matched-state-ring"></a>`material-0e80c1` and
  <a id="warp-calibration"></a>`material-bb8480` are completed investigations;
  their detailed findings remain in the linked findings note. Reuse the existing
  measurement goal; no new goal, generic capture framework or estimator dependency.
