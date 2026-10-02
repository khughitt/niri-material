# Filament shift cap: keep the half-gap bound, document and guard it

**Status:** draft for owner review, revision 2 (spec review round 1:
anchored check, fixture-scoped optical claims, corrected numbers).

**Task:** `material-a85a18`, within `material-49871a`.
**Changes:** `src/tests/ring_pair.rs`, `docs/materials/material-config.md`,
`docs/notes/2026-09-29-glass-measurement-brief.md`; no shader change.

## Intent

The task asked which model should govern the ring's refracted light shift on
dense glass: the half-`ring-gap` cap, a bevel-relative depth, or an automatic
`light-ior`. On the owner's live Prism glass (`ior 1.28`, `thickness 31.2`,
`bevel 10`, `ring-gap 2`) the shared shift on the chamfer reaches the cap at
every `light-ior`. In the fixture, with chromatic aberration, distortion and
jelly ripple off, `light-ior` 1, 6 and 12 render byte-identically.

Decided with the owner on 2026-10-01, from the pilot below: the second ring
image that appears without the cap is the defect, and the hard half-gap cap is
the intended model. Wherever the shared shift reaches the cap, `light-ior` no
longer moves it, and that is accepted. The work makes the bound explicit in
the documentation and adds a regression test that fails if the second image
returns.

The live glass is not the fixture: it sets `chromatic-aberration 0.36` and
`jelly-ripple 0.23`. There `light-ior` still changes the per-channel
aberration offsets, which are not capped. By the formula below, the blue
offset is about 0.27 px at `light-ior 1` and 0.10 px at 6. It also changes
any pixel whose ripple-tilted normal yields a shift below the cap, which
happens only while jelly is active. Both stay within about a pixel. This spec
neither measures nor changes them.

## Evidence

The pilot used the frozen mid-resize fixture `src/tests/ring_pair.rs`
(`material-0e80c1`), with throwaway edits that were then reverted:

- extra Glass rows (1.24/43.3/9, gap 8 and 1.5/80/12, gap 5);
- a `light-ior 12` variant;
- one run with `float cap = 1e6` in `main.frag`.

Corner crops of every combination are attached to the task as
`cap-vs-nocap-sheet.png`.

- **Without the cap**, the chamfer shows a second copy of the band core
  outside the face edge, with a seam at the face edge and a stippled corner.
  This appears on the live glass at `light-ior` 6 and 12, on 1.24/43.3/9 at
  12, and on 1.5/80/12 even at `light-ior 1`. Across the right edge, the
  light map (ring on minus off) shows a second maximum of 65–83 beside an
  83 core on the live glass, and 73–81 beside 82 on 1.5/80/12.
- **With the cap**, every outward profile falls from the core and then
  shows at most a halo step below half the peak: 12 on 1.24/43.3/9 and 23
  on 1.5/80/12 against a peak of about 82. The live glass and 1.5/80/12
  render `light-ior` 1, 6 and 12 byte-identically. 1.24/43.3/9 renders 6
  and 12 identically. Stock glass (1.5/20/12, gap 8) never reaches the cap
  in the 1–12 range.
- **Rejected alternatives at the live glass.**
  - Measuring the depth from the bevel rise instead of the thickness
    (`0.2 * min(bevel, thickness)`) keeps the knob live, but within a narrow
    range: about 0.40 px at `light-ior 1` and 0.99 px at 6, under the 1 px
    cap.
  - A smooth saturation toward the cap gives about 0.85 px at
    `light-ior 1` and 1.0 px at 6.
  - Redefining `light-ior` as a fraction of the cap needs a grammar change
    and a Prism migration, and still has only 1 px of room at gap 2.

  Each changes the shader or the config grammar to move the chamfer's
  landing point by less than a pixel on the glass that motivated it. The
  owner chose to keep the hard cap.

## The model (unchanged)

The model below is how the shader already works; it is restated here as
the contract that the documentation and the test pin.

The shared ring shift is `refract(-z, n, 1 / ior_eff).xy * 0.2 * thickness`,
with `ior_eff = 1 + (ior - 1) * light-ior`, and its length is clamped to
`0.5 * ring-gap`. `n` is the surface normal after distortion and jelly
ripple perturb it (`main.frag` applies both before the ring). The cap bounds
this shared shift only. The green and blue channels add offsets of
`lightShift(n, ior_eff * (1 + k * ca)) - lightShift(n, ior_eff)` on top
(k = 1 and 2, `ca = chromatic-aberration * 0.1`). These offsets are
uncapped and depend on `light-ior`.

With unperturbed normals, as in the fixture, the face normal is flat, so
face pixels sample their own position and the band core stays put; only
chamfer pixels shift. A chamfer pixel `e` px outside the face edge then
lands at most `0.5 * ring-gap - e` inside the face. That is short of the
core at `ring-gap`, so the chamfer shows the band's outer flank and halo and
never a second core. Distortion, ripple and aberration add small tilts and
offsets on both the face and the chamfer. Aurora uses the same index without
the cap and is out of scope.

For the unperturbed chamfer normal, the shared shift reaches the cap when
`sin(a - asin(sin(a) / ior_eff)) * 0.2 * thickness >= 0.5 * ring-gap`. Here
`a = atan(min(bevel, thickness) / bevel)` is the chamfer tilt, 45° whenever
`thickness >= bevel`. With that tilt, the threshold `light-ior` for the
pilot rows is:

| glass (ior/thickness/bevel, gap) | binds from `light-ior` |
|---|---|
| stock 1.5/20/12, gap 8 | never within 1–12 |
| live Prism 1.28/31.2/10, gap 2 | every value (from about 0.75) |
| 1.24/43.3/9, gap 8 | about 5.6 |
| 1.02/80/12, gap 5 | about 10.1 |
| 1.5/80/12, gap 5 | every value (from about 0.41) |

## Changes

### Regression test: one core per edge

Add a test to `src/tests/ring_pair.rs`, `ring_cap_keeps_one_core`, separate
from `ring_pair_is_reproducible_at_a_frozen_instant` so nextest runs them in
parallel. Move the fixture setup that `matched_pairs` performs into a helper
both tests call: the fixture, output, and shm client window, and the resize
frozen at `MID`.

The test config pins `offset-x 6` and `offset-y 6` (the defaults) and keeps
the fixture's flex 0, distortion 0, ripple 0 and chromatic aberration 0. With
those offsets the face edge sits 12 px inside the window edge on the left and
top, and flush with it on the right and bottom (`face_inset` 12 and 0).

Glass rows, chosen to cover the task's ior 1.02, 1.24 and 1.5 and the live
glass:

| name | ior | thickness | bevel | gap |
|---|---|---|---|---|
| `stock` | 1.5 | 20 | 12 | 8 |
| `binding` | 1.28 | 31.2 | 10 | 2 |
| `ior102` | 1.02 | 80 | 12 | 5 |
| `ior124` | 1.24 | 43.3 | 9 | 8 |
| `ior150` | 1.5 | 80 | 12 | 5 |

For each row and each of `light-ior` 1, 6 and 12, render a matched on/off
pair at `MID` with flex 0. Take the light map's profile on the line through
the window centre across each of the four window edges, using the sample
range `profile` already uses.

**The check.** Per edge profile:

1. **Anchor on the face core.** The core `C` is the largest sample whose
   inward distance exceeds `face_inset`, that is, on the face. `C` must be
   positive, and its inward distance must lie within 2 px of
   `face_inset + ring-gap`. An all-zero profile, or a core in the wrong
   place, fails here.
2. **Scan outward from `C`.** Walk toward smaller inward distance to the
   first local minimum, the first sample whose next outward neighbour is
   strictly greater. Every sample beyond that minimum must be below
   `C / 2`.

A ghost brighter than the core cannot become the anchor, because it lies
outside the face. The outward profile `[82, 60, 20, 83, 50, 0]` fails at
the 83.

On the pilot's right-edge profiles (the only edge measured), this check
passes on every capped render. With the cap removed, it fails on `binding`
and `ior150` at `light-ior` 1 and 6, where the ghost reaches 65–83 against
a core of about 82. It does not catch the faint uncapped ghost on `ior124`
at 12 (17 against 83). That is acceptable for a guard against removal of
the cap. The anchor's expected position has not yet been verified on the
left and top edges; the first implementation run checks it there.

**The report.** The test prints each row's ring-on difference between
`light-ior` 1 and 6 and between 6 and 12 (pixels changed, largest delta).
This records where the cap binds. The test does not assert these
differences, consistent with the existing test, which asserts relations
rather than pixel counts. `RING_PAIR_DUMP` writes each render as before;
the `ior124` renders are the task's requested capture of 1.24/bevel 9.

**Mutation check, once.** During implementation, rerun the new test with
`float cap = 1e6` in `main.frag`. Confirm that it fails on `binding` and
`ior150`, then revert the shader edit. Record the result in a task note.

### Documentation

`docs/materials/material-config.md`, the paragraph after the `light-ior`
definition. Replace the current examples, which predate the measurement and
call the 1.24/43.3 glass below the cap. State:

- the cap as the model, with the reason (it keeps a second core off the
  chamfer);
- the binding condition from *The model (unchanged)* above;
- that the cap bounds the shared shift only: where that shift reaches the
  cap, `light-ior` no longer moves it. `light-ior` still sets the
  per-channel aberration offsets, the shift at pixels whose (distorted or
  rippled) normal yields less than the cap, and aurora's landing point;
- a short form of the table above, for unperturbed normals, with the
  live-glass row stated plainly: at `ring-gap 2` the cap is 1 px, and the
  chamfer's shared shift reaches it at any `light-ior`. On that glass,
  `light-ior` affects only the sub-pixel aberration split and, while jelly
  is active, rippled pixels.

`docs/notes/2026-09-29-glass-measurement-brief.md`: under the matched-state
findings, a short "Cap decision" paragraph that links this spec and the new
test. In *Alternatives*, item 3 ("redesign the ring cap now"), note that the
question is closed as decided.

The `filamentBand` comment in `prelude.frag` already describes the cap
correctly and stays.

### Prism

Prism's niri sink emits `light-ior 6` on the live glass. There it no longer
moves the ring's shared shift on the chamfer, and it acts only through the
sub-pixel aberration split, rippled pixels and aurora. File an `idea` in the
`prism` project that records this, so the palette's definition can say so. That decision belongs to Prism; no Prism change is part of this
task.

## Testing and verification

- `just test-one -p niri ring_pair`: both tests pass. Under the fixture's
  conditions the report shows the `binding` and `ior150` rows
  byte-identical across `light-ior`, and `ior124` identical between 6 and
  12.
- The mutation check above fails on `binding` and `ior150` with the cap
  removed.
- `just test-fast` before commit. `just check` covers the material-config
  docs, which Rust tests read.

## Interaction with other work

`material-22d78f` (the flex centroid check) also extends `ring_pair.rs`. The
shared fixture helper is the only overlap. Whichever task lands second
rebases onto the helper; neither changes the other's assertions.

## Out of scope

- Aurora's uncapped landing shift.
- Any new `light-ior` grammar, warning, or validation.
- Measuring `light-ior`'s residual effect through aberration and ripple on
  the live glass.
- Corners, moving beam, chromatic aberration on, distortion and ripple on,
  textured backdrop and fractional scale. The pilot and the test use the
  existing fixture's conditions only.
