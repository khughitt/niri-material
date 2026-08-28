# Frozen-reference static preflight: design

**Status:** implemented and re-executed 2026-08-26 at `niri-experiments`
instrument commit `d00f81c32cb8e6ee60881eeee4e65493e39ec6bf`; the
frozen-reference preflight passed 9/9 rows. Final capture base
`b851e5208b54cc466d99bf3ae664cc5a52c2317f` and result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f` passed integrity, all 28
implementation rows, and all 14 combined parameters in two independent
captures. The parity gate is complete. Physical DRM acceptance later produced
a valid FAIL at `niri-experiments` result commit
`dbb277557f454b803af15e7dcd933a97aeea2e7b`: four exact-rest machine gates
and physical observation 8 failed, so native materials v1 remains blocked.
The later additive geometry preflight at `niri-experiments` commit
`5d3dfd26c4d4a4ef2c464c1d649dba533ccb27c5` passed 3/3 frozen-reference
geometry rows with a geometry-only 80-pixel layout. It supersedes this
document's assumption that geometry can continue in the 24-pixel static
layout; static optics and motion still use 24 pixels.
**Parent design:** `docs/materials/2026-08-24-v1-parity-design.md`

## Goal

Make the one-time v1 parity gate fail on an invalid frozen reference before
rebuilding or capturing the old native surface. Static metrics must measure
regions selected independently of the parameter under test, and refraction
metrics must retain their expected direction.

This remains purpose-built evidence for the pinned v1 reference. It is not a
general visual-test harness, a CI job, or a reusable region-of-interest API.

## Context

The first parity run passed capture integrity but failed all combined static
rows. Inspection found two analyzer defects:

- the frozen preview cannot expose its diagnostic background without its
  slab, so its phase-equivalent source did not support reliable registration;
- refraction from `ior`, `thickness`, and chromatic aberration exists only in
  the pinned 12-pixel chamfer, which the 24-pixel lattice and 9-by-9 windows
  could not resolve.

The recorded native `ior` and chromatic-aberration fields were already zero,
so this was not reference-only. The synthetic `ior` control missed the defect
because it shifted the whole pane instead of its 12-pixel bevel.

A live-pane spike established a viable replacement. It captured a genuine
slab-free reference source, then used the pane geometry instead of a response
mask to select three independent regions. Its 40-pixel diagnostic tile keeps
the dense grid and adds a pinned deterministic 15% asymmetric watermark so a
local match is unique without erasing the edge profiles.

The frozen shader also predicts the `ior` direction independently. Across the
45-degree chamfer, the refraction coefficient changes from -0.411 at
`ior=1.5` to -0.581 at `ior=2.0`, so the expected differential is
`0.170 * 0.707 * 20 = 2.40` pixels inward. The unwatermarked spike measured
2.23 pixels inward on both edges; the final watermarked tile measured 2.17 and
2.20 pixels. Both agree with the shader's sign and magnitude.

The shader likewise predicts chromatic shifts from base `ior`,
`ior*(1+spread)`, and `ior*(1+2*spread)`: red stays at zero while green and
blue form an approximately 1:2 ladder. The unwatermarked spike measured 0.00,
1.97, and 3.89 pixels. The watermarked control retained zero red and strictly
increasing green and blue shifts on both edges.

For the inner face, local two-dimensional correlation reconstructed all 4,212
undistorted samples as zero displacement. Both distortion fields retained all
4,212 samples and 8,292 neighboring pairs with no search-boundary hits. Their
normalized neighboring-vector frequency rose from 0.275 to 0.508 when
distortion scale changed from 0.5 to 1.5. This operates on recovered offsets,
not the composed image's spectrum.

## Decision

Use the live-pane scene for static optics on both implementations. Derive the
optical pane rectangle from the recorded niri IPC output, workspace, and
window layout state plus the pinned pane geometry. Independently cross-check
that rectangle against the source-to-default pixel response.

Split static measurement geometrically:

- signed bevel profiles use the straight parts of the outer 12-pixel band;
- displacement from distortion uses a local field reconstructed within the
  flat face;
- attenuation uses the rectangle's fixed 32-pixel inset.

Run the frozen reference's nine static rows as a preflight. All nine must pass
before the old native surface is rebuilt or either implementation is captured
for final evidence. A passing preflight is deliberately recaptured during the
full run; no resumable cross-run artifact handoff is added.

## Capture and geometry contract

The preflight requires two environment variables and has no fallback:

- `NIRI_MATERIAL_WORK_ROOT` names the host-local artifact root;
- `PREFLIGHT_NIRI` names an existing executable niri binary.

Unset, empty, invalid, or repository-contained work paths fail before a
compositor starts. The replay records `PREFLIGHT_NIRI`'s version and SHA-256
only in the preflight manifest. This binary is a feasibility host, not the
pinned host for final evidence. After preflight passes, the procedure rebuilds
the old native surface into its dedicated Cargo target; that old binary hosts
both phases of the final capture and supplies the final manifest's niri pins.

Static capture uses the same sequence for native and reference:

1. Capture slab-free `source` duplicates with the live diagnostic backdrop
   visible and no selected pane window. Reuse that exact pair as the geometry
   source; without a pane, the backdrop-only pixels remain valid across the
   later layout transition.
2. Spawn the probe with app ID `v1-parity-probe`; that selected probe produces
   the live material pane for the `default` duplicates.
3. Capture the nine existing static variants and their return gates, then
   capture motion while the probe remains in the 24-pixel scene. Transition
   that probe once to the geometry-only 80-pixel layout and capture geometry.

The final 40-by-40 watermarked tile is a committed diagnostic asset, not a
runtime-random blend. The replay tiles those bytes directly, and
`diagnostic_sha256` hashes that final asset. This supersedes the parent
design's pinned 200-pixel diagnostic SVG and period. The PNG is also the input
to geometry and motion, so `lip` and native `shift-x`/`shift-y` are remeasured
instead of inherited from the old run; the SVG remains as the reproducible
rasterization source. The accepted derivation excludes PNG `date` and `time`
chunks, then yields reproducible SHA-256
`6fafae8c6cf3e3815346128ffdb402d5c730ae3a8749cb060013821ba0fe0316`, sample
`p{20,20}` `srgb(44,54,59)`, and standard deviation `0.0355911`. The hash is
the identity pin; the sample and minimum `0.02` deviation are cheap corruption
gates.

The replay derives the optical pane rectangle from the live IPC geometry and
the pinned `paneLip`, shifts, corner radius, output scale, and transform. It
records only the resulting integer rectangle:

```json
"static_roi": { "x": 0, "y": 0, "width": 1, "height": 1 }
```

Those values show the shape, not fixed coordinates. For both duplicates, a
source-to-default response profile across each derived edge must begin within
the frozen reference shadow's directional reach: 16 pixels above, 24 to the
left, 36 to the right, and 44 below. These bounds follow directly from its
30-pixel spread and model offset of 6 pixels right and 14 pixels down in screen
space. This
catches a bad IPC-to-frame mapping without turning the shadow's larger pixel
bounding box into the optical rectangle. The two pixel cross-checks must
agree.

The source and default may differ outside the optical rectangle only within
those pinned per-edge shadow reaches. Default-to-variant responses must remain
confined to the optical rectangle. Native focus ring, border, and shadow stay
disabled. No reference preview or static anchor participates in this path.

## Manifest contract

The final manifest schema becomes `2`. Both implementations then have the
same exact keys:

```text
static, static_roi, geometry, motion, calibration, first_offset_budget_ms
```

Both `static` objects contain `source` and the same static state keys. The
reference-only `static_source` phase-equivalence object and its registration
path are deleted rather than retained as compatibility behavior. Reusing the
same source duplicate pair across `static.source` and `geometry.source` is
intentional; physical-reuse rejection still applies within each duplicate
pair and across motion bursts.

`reference-static-manifest.json` is a separate exact partial schema, also
version `2`. Its root contains only `schema`, frame dimensions,
`diagnostic_sha256`, `live_backdrop`, `pins`, `tools`, `warning_allowlist`, and
`implementations`. Its exact contracts are:

- `implementations` contains only `reference`, whose only keys are `static`
  and `static_roi`;
- `static` contains `source`, `default`, and the nine static variants;
- `pins` contains exactly `reference_source_commit`,
  `evidence_base_commit`, `preflight_niri_binary_sha256`,
  `preflight_niri_version`, `reference_config_sha256`, and
  `reference_json_sha256`;
- `tools` contains exactly `node`, `quickshell`, `qt`, `weston`,
  `imagemagick`, `jq`, `kitty`, `swaybg`, and `kernel`;
- `live_backdrop` and each duplicate capture pair retain their existing exact
  key sets and validation, while `warning_allowlist` retains its one exact
  accepted warning.

No native commit, native config, Cargo, or Rust pin is admitted to the partial
manifest because none exists or runs before the gate. Old manifests fail with
an explicit schema-version error.

## Analyzer contract

Geometry and support are evidence-integrity checks. The analyzer exits `2`
before emitting semantic rows when:

- either manifest violates its exact schema;
- `static_roi` has keys other than `x`, `y`, `width`, and `height`, contains a
  non-integer or non-positive dimension, or leaves the captured frame;
- the 32-pixel inset is empty;
- either duplicate fails the pixel edge cross-check or the checks disagree;
- source and default reuse files, or duplicates exceed the existing 0.1%
  drift limit;
- any accepted bevel profile or warp-field match has its best integer offset
  on that oracle's search boundary;
- a displacement field accepts fewer than half its geometric lattice
  candidates or contains fewer than half its possible horizontal and vertical
  neighbor pairs.

The proportional field requirements replace the old nine-block floor: on the
pinned inner face they require thousands of samples and, critically, adjacent
support for the frequency oracle.

Valid evidence that moves in the wrong direction is semantic failure and
exits `1`. All nine passing rows exit `0`.

### Signed bevel profiles

The `ior` and chromatic-aberration oracles average linear-light profiles over
the straight top and left bevels, excluding the 28-pixel corner radius. They
compare normal positions 3 through 11, use the normalized first derivative to
remove level changes, and fit the SSD minimum parabolically for a signed
sub-pixel shift. Positive means the variant matches samples farther inward
than the default. The `ior` search is plus-or-minus 4 pixels. Chromatic red,
green, and blue use bounds of plus-or-minus 1, 4, and 5 pixels. The green
bound includes one pixel of port-response margin beyond the shader-derived
reference bound; the first full native capture measured up to 2.61 pixels.
A best integer offset on any boundary is saturated evidence and exits `2`.

For `ior`, top and left must both move in the pinned positive direction, clear
their duplicate noise floors, and agree in sign. An undirected RMS difference
is not sufficient. The 32-pixel inset supplies a separate geometry check. For
each channel and pixel, decode the half-code boundaries around both 8-bit sRGB
captures and intersect their possible linear-light delta intervals. A common
intersection permits native's uniform ior Fresnel change and its independent
quantization in both captures; a gap beyond the duplicate floor rejects
spatial restructuring of the flat face.

For chromatic aberration, red must remain at its duplicate shift noise floor.
On both edges, `green - red` and `blue - green` must each be positive and clear
the duplicate separation noise floor. Red is not required to clear noise.

The thickness change predicts a bevel differential of
`0.411 * 0.707 * (80 - 20) = 17.4` pixels. That is far outside the 4-pixel ior
search and only 2.6 pixels short of a half-period on the 40-pixel backdrop;
widening the profile search therefore cannot distinguish the physical shift
from its periodic alias. This is a scene-geometry limit, not a failed fit.
The thickness bevel half remains explicitly magnitude-only: bevel RMS must
clear duplicate noise. The row is still directional because its independent
inset attenuation half must move in the required direction. This is a
deliberate weakening of only the thickness displacement half, not a claim that
nonzero RMS proves refraction direction.

### Inner-face warp field

Distortion and distortion scale retain registration, but only on the
32-pixel inset. At an 8-pixel lattice, each rendered 17-by-17
linear-luminance patch is matched to the captured source with local
two-dimensional zero-mean correlation and a plus-or-minus 15-pixel search.
Tied minima are rejected. The asymmetric watermark breaks sub-period symmetry
inside each tile. The plus-or-minus 15-pixel bound stays below the 20-pixel
half-period and therefore prevents matches from reaching an adjacent
40-pixel repeat. Any best match on the boundary is saturated evidence and
exits `2`.

The undistorted default field must remain at its duplicate noise floor.
`distortion` must increase displacement-field variance and stay confined to
the pane. `distortion-scale` compares the two recovered fields with
`fieldFrequency`: mean neighboring-vector delta divided by mean displacement
magnitude must increase. No composed-image spectrum enters the oracle.

### Remaining static rows

Attenuation color and distance use the 32-pixel inset and their existing
signed channel and brightness predicates. The inset clears the frozen
12-pixel chamfer and 28-pixel corner radius; it is pinned to that geometry, not
presented as a general constant. It no longer depends on field flatness or a
variant-derived mask.

Anisotropic blur and samples retain their existing transition-width and
high-frequency-residual direction predicates over the optical pane. Results
must report every metric, noise floor, and margin, including the samples row's
small observed margin, rather than recording only a verdict.

## Preflight procedure and artifact lifecycle

Implementation landed on `results/reference-static-preflight`, branched from
`results/slice3` at historical evidence commit `7729dfc`, in the existing
replay, analyzer, and analyzer self-check files. Final instrument commit
`c729dee35db2c96cc41568d9e1090fd1536386c2` supplied the initial replacement;
quantization-aware follow-up commit
`d00f81c32cb8e6ee60881eeee4e65493e39ec6bf` supplies the static-preflight
analyzer and self-check hashes. Final capture base
`b851e5208b54cc466d99bf3ae664cc5a52c2317f` and result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f` record the current 14/14 verdict.
Additive geometry-preflight commit
`5d3dfd26c4d4a4ef2c464c1d649dba533ccb27c5` derives an 80-pixel geometry
scene and records a passing 3/3 feasibility gate; result commit
`af99babfdeed1f64e3bf52817b62a22c9d1c9d72` records it. Replay correction
`705718d410232badb9ddd7e27d2da2e40ca3eb24` captures motion before the one-way
geometry transition because the frozen native surface does not reproduce its
initial optical pixels after an 80-to-24 resize round trip.
The parity result's stale historical `results/v1-parity` branch claim was
corrected to the branch and commit that actually contain that evidence.

Preflight artifacts, final capture artifacts, and the dedicated old-surface
Cargo target must be children of `NIRI_MATERIAL_WORK_ROOT`. Project code and
documentation use the variable rather than embedding a machine-specific path.
There is no `/tmp` fallback.

Each preflight uses a unique directory. A passing preflight is deleted
immediately. A failing preflight is retained with its printed path while it is
the active diagnostic artifact, then deleted once a later run supersedes it.
After final evidence is distilled and committed, its raw capture is deleted
and Cargo cleans the dedicated old-surface target.

Normal process cleanup remains the replay script's responsibility. It stops
owned compositor, Quickshell, terminal, wallpaper, and anchor processes and
removes their runtime directory on both success and failure.

## Verification

Implementation extends the existing replay and analyzer self-checks rather
than adding another test framework. The checks cover:

- exact schema-2 final and partial manifests, including rejection of schema 1;
- accepted IPC-derived ROI, pixel edge cross-check, and fixed inset;
- empty, out-of-bounds, disagreeing, and shadow-swallowed rectangles;
- a signed 12-pixel-band displacement control and its wrong-direction twin;
- saturated bevel and warp searches as exit `2`;
- independently quantized a varied flat-face backdrop and allowed its known
  uniform linear-light shift while rejecting a checker perturbation;
- channel-ordered chromatic displacement confined to the bevel;
- zero default warp, nonuniform distortion, and increased frequency in the
  recovered displacement field rather than the image spectrum;
- proportional lattice and neighbor-pair support failures as exit `2`;
- valid wrong-direction rows as exit `1`;
- reference preflight success and proof that either failure exit cannot reach
  a native build step.

After those checks pass, execution requires a real 9/9 frozen-reference
preflight. Only then may the old native surface be rebuilt and the full parity
capture rerun. Result and design status documents are updated from the new
committed evidence, followed by a user-facing documentation grep for the old
acceptance and evidence-branch claims.

## Alternatives rejected

**Hard-coded pane coordinates:** fewer IPC calculations, but the evidence
would depend on host placement rather than derive the measured pane each run.

**Pixel bounding box as the pane:** direct measurement, but it includes the
reference shadow. The bounded pixel edge cross-check catches placement errors
without changing the optical region.

**Morph a parameter response mask:** a smaller analyzer diff, but the
parameter under test would still select its own measurement region.

**Undirected bevel RMS for `ior`:** easy and nonzero, but it accepts weaker or
reversed refraction. The signed one-dimensional fit is stable on both pinned
straight edges.

**Composed-image frequency for distortion scale:** shorter than reconstructing
the field, but it measures the diagnostic backdrop's own periodicity.

**General ROI or resumable-capture framework:** unnecessary for this frozen,
one-time gate. Recapturing after preflight is simpler and keeps failed
diagnostics separate from final evidence.
