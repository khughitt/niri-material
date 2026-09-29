# Glass displacement and motion: scoping brief

## Problem

Make glass measurements distinguish optical changes from capture skew and
backdrop repetition before using them to tune the renderer. This pass covers
`material-22d78f`, `material-a85a18`, `material-343f27` and `material-8867aa`,
now grouped under `material-49871a`. All four remain briefed ideas.

## Current behaviour and evidence

- `docs/materials/scripts/focus-ring-light.sh::case_resize_flex` still starts
  two nested hosts and samples them after a sleep. The
  [original evidence](../materials/2026-09-05-ring-light-focus-smoke.md)
  records mismatched layout and client glyphs; its mid-resize deltas cannot
  isolate ring light. `src/tests/animations.rs` already has a frozen-clock
  `set_time` helper and a headless renderer fixture. Whether these can produce
  matched material pixels is unverified, particularly with the beam's
  independent time and client buffer state.
- The old zero-light-on-face requirement is obsolete. The
  [within-ring amendment](../specs/2026-09-12-material-render-order-design.md)
  deliberately permits light through translucent client pixels; the within
  implementation landed in `e79b226b`. The beam change `98013739` subsequently
  moved the band to the face edge. Current
  `src/render_helpers/shaders/material/main.frag` uses `0.2 * thickness`,
  caps the shared shift at half `ring-gap`, then adds chromatic offsets.
  `material-a85a18`'s historical numeric examples need fresh derivation.
- Existing `material-3db428` owns stale sample rows and reach bounds in the
  old ring scripts. Reuse it; deterministic capture is a separate gap.
- `docs/materials/scripts/glass-parameter-sweep.sh::make_backdrop` still
  generates a 20 px grid. The
  [sweep evidence](../materials/2026-09-06-glass-parameter-sweep-evidence.md)
  records displacement aliasing, a rigid-template matcher pinned by
  non-bending pixels, and a rejected subtraction of nonlinear RMSE scores.
  Regional Lab RMSE reports image change, not its optical cause. Removing
  repetition alone does not make RMSE monotonic or establish bending.

## Constraints

Use the [current pipeline](../materials/render-pipeline.md), slab coverage
and opaque-client bypass as the contract; do not restore the retired face
mask. Preserve source task bodies and historical captures. Keep backdrop
identity, seed, geometry and renderer provenance with any new evidence.
No broad capture matrix precedes a passing end-to-end pilot; applicable
capture preflight remains in force. Existing `material-31074f` owns cost
measurement; `material-2ebf2c` concerns perceptual trade-offs. Neither
establishes geometric displacement, and no open local research task found
in this pass already answers the two questions below.

## Alternatives

1. **Current lean: calibrate small instruments using existing fixtures.**
   Test frozen-state rendering for the ring, and a seeded aperiodic field
   against known spatial warps and photometric controls. Record limitations
   before selecting an implementation.
2. Improve only the sweep backdrop. This can reduce periodic ambiguity but
   still cannot distinguish Fresnel, attenuation and displacement; it also
   leaves motion skew unresolved. Share calibration before making this change.
3. Add production clock controls or redesign the ring cap now. Defer both:
   the existing test seam may suffice, and no current cap-binding visual
   defect has been established by this pass.

## Unanswered questions

- Can one fixture hold geometry, client buffers, signal state and beam time
  constant for a repeatable on/off pair? `material-0e80c1` establishes
  feasibility and the smallest missing seam, if any.
- Which explicit dense-glass settings bind the current cap, and what can a
  matched pair measure under flex? The same investigation recomputes the
  bounds, including ior 1.24 / bevel 9. The owner judges any later proposed
  visual change; this pass does not require that judgment.
- Can a local estimator recover a known non-rigid warp while rejecting
  photometric-only change? `material-bb8480` quantifies error and failure
  cases, then recommends a rendered control before claiming actual bending.
- Should the sweep use that calibrated backdrop or a separate one?
  `material-bb8480` recommends the smallest reusable artifact with explicit
  comparability to the old grid.

## Proposed decomposition

- `material-49871a` groups the four ideas and two investigations; no selected
  idea was started or claimed.
- <a id="matched-state-ring"></a>`material-0e80c1`: P2, small, mid complexity,
  direct investigation of matched-state capture and current shift bounds;
  wakes `material-22d78f` and `material-a85a18`. Reuses `material-3db428` for
  old-script geometry corrections without expanding that task.
- <a id="warp-calibration"></a>`material-bb8480`: P2, small, high complexity,
  direct offline calibration; wakes `material-343f27` and `material-8867aa`.
  Its output may reject the candidate instrument. It does not promise a
  GPU measurement or an acceptable perceptual-quality metric.

Both investigations record findings on their waiting ideas in the same
commit as their results and update this brief. Design work is filed only
if those findings establish a remaining design decision.
