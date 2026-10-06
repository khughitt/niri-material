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
  `set_time` helper and a headless renderer fixture. `material-0e80c1`
  established that they produce byte-identical matched pairs once the test
  client attaches shm buffers; see
  [matched-state ring findings](#matched-state-ring-findings).
- The old zero-light-on-face requirement is obsolete. The
  [within-ring amendment](../specs/2026-09-12-material-render-order-design.md)
  deliberately permits light through translucent client pixels; the within
  implementation landed in `e79b226b`. The beam change `98013739` subsequently
  moved the band to the face edge. Current
  `src/render_helpers/shaders/material/main.frag` uses `0.2 * thickness`,
  caps the shared shift at half `ring-gap`, then adds chromatic offsets.
  `material-0e80c1` rederived `material-a85a18`'s numbers for the current
  shader (table under the findings below).
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
   defect has been established by this pass. Since closed: the test seam
   sufficed (`material-0e80c1`), and the cap stays as it is
   (`material-a85a18`, see *Cap decision* under the findings).

## Unanswered questions

- Answered by `material-0e80c1`: one fixture holds geometry, client buffers,
  signal state and beam time constant for a byte-identical on/off pair. The
  missing seam was a test-client shm buffer.
- Answered by `material-0e80c1`: the cap-binding settings are tabulated
  below, and a matched pair measures band position to about 0.01 px under
  flex. The owner judges any later proposed visual change; this pass does not
  require that judgment.
- Answered by `material-bb8480`: on a seeded aperiodic backdrop, ZNCC
  matching refined by Gauss-Newton recovers a chamfer-shaped warp to 0.02 px
  inside the strip and rejects photometric-only change, with windows across
  the strip's edges flagged and masked by geometry. A rendered flat-field
  pair is still needed before a reading counts as bending; see
  [warp calibration findings](#warp-calibration-findings).
- Answered by `material-bb8480`: the sweep reuses the calibrated backdrop,
  with `grid20` kept by name for comparability.

## Matched-state ring findings
<a id="matched-state-ring-findings"></a>

`material-0e80c1`, 2026-10-01. Fixture: `src/tests/ring_pair.rs` (run
`just test-one -p niri ring_pair --no-capture`; set `RING_PAIR_DUMP=<dir>`
for PNGs). Renderer: headless surfaceless GLES, NVIDIA GeForce RTX 3070 on
the recording host; CI runs the same test on its own GL stack. The test
asserts only relations (repeats equal, controls differ, opaque bypass), not
these pixel counts.

**Reproducibility.** One fixture renders every variant at one frozen instant,
500 ms into a 1000 ms linear resize from 640 to 840 px. Ring on and off are
two response-only config reloads, which keep the material state, seed and
resize animation. A repeat after an off/on reload is byte-identical (0 px).
A second repeat after all the variant reloads below is also 0 px. A control
shifted by 10 ms differs in 22,396–23,956 px (max channel delta 42–48), so
the instrument resolves a 2 px slab move by a wide margin.

**State held fixed, and how.** Clock: `set_time` before every render
(dispatch and reloads unfreeze it). Resize progress: linear curve, a fixed
instant. Client buffers: a 1×1 ARGB8888 shm buffer, viewport-scaled and fully
damaged. Focus: `update_keyboard_focus` before the resize, crossfade
complete. Beam: `ring-beam-speed 0`, so no beam starts and its independent
time never enters. Signal: none over IPC; accent `none`. Off for
determinism: jelly ripple, distortion, aurora, chromatic aberration,
backdrop blur, noise, shadow, border, focus ring and the hotkey overlay.
Backdrop: the plain backdrop colour, with no layer surfaces. The jelly seed
comes from a process-global counter and is printed with each report (it
reaches no pixel with ripple, aurora and the beam off).

**Missing seams found.** (1) The test client's single-pixel buffers never
become textures (smithay skips importing them), so the resize snapshot is
empty. The tile then falls back to a plain render without the material, and
on/off were identical for that reason. The fix is a test-client
`attach_new_shm_buffer`; production code is unchanged. (2) An undamaged commit
leaves niri's window offscreen on the old contents, so the shm attach
damages the buffer. (3) The hotkey overlay covered the right edge. No
production clock IPC was needed.

**Light map (on minus off).** The profile is taken through the window centre
on each edge. Distances are inward from the window edge; `c` is the centroid
of the half-maximum run.

| case | left / top c | right / bottom c | peak value | lit px |
|---|---|---|---|---|
| stock 1.5/20/12, gap 8, mid | +20.02 | +8.02 | 82 | 65,219 |
| stock, at rest | +20.03 | +8.03 | 64 | 68,179 |
| binding 1.28/31.2/10, gap 2, mid | +13.61 | +1.59 | 83 | 58,177 |
| binding, at rest | +13.62 | +1.60 | 65 | 60,278 |

The band core sits `ring-gap` inside the face edge. The face sits 12 px in
from the window's left/top edges and flush with its right/bottom edges at the
default `offset-x`/`offset-y` of 6 (narrowed by 12, translated by 6). At flex
0, mid-resize and rest agree to 0.01 px. The ring is brighter mid-resize (peak
82–83 against 64–65 at rest), so pairs must share their instant. At gap 2, the
half-maximum run reaches 0.5 px outside the window edge.

**Under flex.** These runs use the same instant and reload the glass
parameter only. The band centroid moves inward on the resizing axis alone:
+0.44 px at `jelly-flex 0.0066` (Prism) and +0.96 px at `0.02` (range
maximum), on both geometries. Top and bottom stay at +0.00. Ring-on renders
differ from flex 0 in 16–19k px, max delta 9 and 22–23.

**Cap binding, measured.** Light-ior 1 and 6 at the binding geometry render
byte-identically (0 px): the cap holds both, as the formula predicts
(1.24 and 3.09 px against 1.0). At stock geometry, below the cap, the same
change alters 10,861 px by at most 2 levels. The shift acts only on tilted
normals (the chamfer), never on the flat face where the band core lies.

**Cap decision.** `material-a85a18`, 2026-10-01: the hard half-gap cap is
the intended model. Without it, dense glass shows a second copy of the band
core on the chamfer. Where the shared shift reaches the cap, `light-ior` no
longer moves it. The [design](../specs/2026-10-01-filament-shift-cap-design.md)
records the alternatives and why they were rejected. `ring_cap_keeps_one_core`
in `src/tests/ring_pair.rs` renders ior 1.02, 1.24 (bevel 9), 1.28 and 1.5
and fails without the cap on 1.28 and 1.5; the faint ghost at 1.02 and 1.24
stays under its threshold.

**Opaque-client bypass.** With an opaque client at rest, the ring lights
0 px inside the window rect. It lights 5,962 px (stock) and 10,799 px
(binding) outside it, where the halo crosses the chamfer. Over a translucent
face, light scales with `1 - client alpha` (`main.frag` compositing); the
retired zero-light-on-face premise does not apply.

**Shift bound (formula, not measured).** The values below are
`refract(-z, n, 1/ior_eff).xy * 0.2 * thickness`, with
`ior_eff = 1 + (ior - 1) * light-ior`. The chamfer normal uses
`rise = min(bevel, thickness)` over a run of `bevel`. The cap is
`0.5 * ring-gap`.

| ior | thickness | bevel | gap | light-ior | shift px | cap px | binds |
|---|---|---|---|---|---|---|---|
| 1.02 | 20 | 12 | 5 | 1 / 6 / 12 | 0.08 / 0.41 / 0.71 | 2.5 | no |
| 1.02 | 80 | 12 | 5 | 1 / 6 / 12 | 0.31 / 1.63 / 2.84 | 2.5 | at 12 |
| 1.24 | 43.3 | 9 | 8 | 1 / 6 / 8 | 1.54 / 4.09 / 4.46 | 4 | at 6, 8 |
| 1.28 | 31.2 | 10 | 2 | 1 / 6 | 1.24 / 3.09 | 1 | both |
| 1.22 | 75.3 | 10 | 2 | 6 | 6.90 | 1 | yes |
| 1.5 | 20 | 12 | 8 | 1 / 6 | 1.16 / 2.28 | 4 | no |
| 1.5 | 80 | 12 | 5 | 6 | 9.14 | 2.5 | yes |

**Deterministic motion check.** `material-22d78f`, 2026-10-04:
`ring_tracks_face_during_resize` now grades all four edges at the same
500 ms instant, for stock and binding glass and flex 0.0066 and 0.02.
The independently derived inward face displacement is

```text
cap = 0.25 * min(bevel, thickness)
resize = cap * tanh(-flex * 100 / cap)
slab_width = 740 + 2 * (bevel - 6)
inward = -(slab_width / 2 - bevel) * resize / slab_width
```

The 100 px residual is half the fixture's 640 → 840 linear resize.
Predicted x displacements are 0.31441 / 0.84628 px for stock and
0.31392 / 0.80785 px for binding; y displacement is zero.

A raw half-maximum centroid is not a continuous displacement estimator:
at stock flex 0.0066 a fifth pixel enters the run, giving a 0.44 px
centroid change for a 0.31441 px face shift. The previous 0.01 px
mid-versus-rest residual does not bound that sampling error. Instead,
a second matched pair, with flex zero and `ring-gap` increased by the
predicted displacement, supplies the sampled expected centroid. Horizontal
edges must match this control within 0.01 px; vertical edges must match the
flex-zero baseline within the same bound. This control uses neither the
production `jelly_state` nor the CPU face helper to derive its displacement.

The motion test pins ring-gap 8 on both glass geometries so the measured
core clears the refracted chamfer. The original binding gap 2 remains
covered by `ring_cap_keeps_one_core`; this test does not establish a
continuous centroid estimator for that profile. Corners, a moving beam,
chromatic aberration, a textured backdrop, partial client alpha, vertical
or shrinking resizes, and fractional scale remain outside this check.

Run `just test-one -p niri ring_tracks_face_during_resize --no-capture`,
or `CASES=resize-flex docs/materials/scripts/focus-ring-light.sh`. The script
runs the test before capture initialization, propagates failure, and starts
no nested compositor for this case. Other capture cases retain their
existing host requirements. The old two-host resize comparison and its
skew-only metrics are retired; its historical captures remain history.

## Warp calibration findings
<a id="warp-calibration-findings"></a>

`material-bb8480`, 2026-10-06, offline; the
[evidence](../materials/2026-10-06-glass-warp-calibration-evidence.md)
holds the tables and `docs/materials/scripts/warp-calibration.py`
reproduces them.

- **Adopted, for straight chamfer edges.** At a 7 px window, chamfer shifts
  of 1 to 20 px read to 0.02 px in the strip interior; no case trusted a
  pixel more than 1 px wrong. A parabola-only subpixel fit is biased by
  0.2 px at p95 and, without the residual check, trusts boundary windows up
  to 41 px wrong.
- **The 20 px grid measures nothing.** Every pixel is refused, and its RMSE
  is exactly 0 for 1, 3 and 20 px chamfer shifts while a tint alone scores
  17.2. RMSE on the aperiodic field is not monotonic in the shift either.
- **Masks and limits.** Boundary windows (half a window at each strip edge)
  are masked from the window rect and bevel. Fields that vary inside a
  window (rounded corners, `distortion` above 0) fail the residual check;
  `material-79fb49` holds the affine-warp model for them. Clipped
  highlights are flagged. The range is the search radius, 24 px.
- **Rendered control.** Render the glass over two uniform backdrops beside
  the textured one: attenuation multiplies and the Fresnel glint adds, both
  independent of backdrop content, so the pair divides them out per pixel.
  `ior 1` and `thickness 0` are not clean controls; each also changes
  Fresnel or the bevel normal.

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
