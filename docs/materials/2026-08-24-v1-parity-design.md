# Native materials v1 parity pass: design

**Status:** re-executed twice 2026-08-27 at `niri-experiments` capture base
`b851e5208b54cc466d99bf3ae664cc5a52c2317f` and graded at result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f`. Both captures passed integrity,
all 28 implementation rows, and all 14 combined parameters. The
config-surface review and frozen-reference parity gate are complete. The
corrected physical DRM run passed all 19 machine gates and all nine physical
observations at `niri-experiments` result commit
`16d2b4aa957637fa9339fc4bc0c4f7bd7b4aca34`. Native materials v1 is
accepted. The corrected full capture uses the 80-pixel geometry scene proven
by the earlier 3/3 preflight at `af99babfdeed1f64e3bf52817b62a22c9d1c9d72`.
**Parent design:** `docs/materials/2026-08-22-v1-design.md`

## Goal

Run a one-time, documented parity pass between native glass and the frozen
Quickshell reference before physical DRM acceptance. The pass must show that
every parameter shared by the two implementations produces the same semantic
response, without requiring pixel identity between different rendering and
composition pipelines.

The pass verifies the **optics port** — that the native shader reproduces the
reference's parameter responses. It does not evaluate whether the shared
parameter set is the right long-term configuration surface. That question was
settled by the separate config-surface review recorded below.

## Pinned starting state

| Repository | Commit | Role |
| --- | --- | --- |
| `niri-material` | `5411ac05de730e79d4bbacf57b8bbd2b43cec310` | Native implementation and the compositor used for both sides |
| `niri-glass` | `70f26af4324635bb827d07f0eaf1fc227acb43ea` | Frozen Quickshell parameter reference |
| `niri-experiments` | `e59786070eced3b8891bb61e0cc241733c7f81ed` (`results/slice3`) | Slice 3 evidence and reusable capture method |

All three trees were clean when this design was recorded. Execution records
the actual commits and binary/config hashes again; these design-time pins are
not substitutes for run-time identity checks.

## Decision

Parity means **paired semantic response**, not whole-frame similarity. For
each implementation, compare a high-signal parameter variant with that
implementation's own controlled default. A parameter passes when both
implementations satisfy the same predeclared response oracle.

This boundary is necessary because the implementations deliberately differ:
native glass composes compositor background/backdrop inputs with window
pixels, while the Quickshell client renders a bottom-layer Qt Quick 3D scene
that samples a duplicated wallpaper and includes Qt-specific environment
lighting. Direct native-to-Qt RMSE would measure these architectural
differences along with the parameter under test.

Attempt 5 reproduced the same Weston 15 kiosk-shell SIGSEGV in two independent
fresh PIDs at `kiosk_shell_output_set_active_surface_tree` when reference
mapped after native completed. A whole-replay fresh-host retry is therefore
disproven. Native and reference must instead be separate phases under separate
fresh hosts; their only shared state is an explicitly verified artifact
handoff. This changes capture isolation, not the per-renderer metric boundary.

The pass is one-time v1 evidence. It reuses the Slice 3 headless startup and
capture pattern plus the raw-RGB analysis method in
`niri-experiments/docs/research/legacy-visual-verification.md`. It does not
create a generalized parity framework or CI job.

A passing matrix therefore establishes port fidelity under the pinned
controls, and nothing more. It is measured with reference `roughness` pinned
to 0, so the frozen client's shipped 0.08 appearance is never compared; and it
says nothing about whether a parameter should exist, be named as it is, or be
user-facing at all.

## Environment and scenes

Native and reference run sequentially under separate, identical dedicated
1280x720 headless Weston GL/kiosk hosts, using the same socket name, Weston
flags, and pinned release niri binary. Native mode enables the material;
reference mode disables it and runs the frozen Quickshell client. They share
one artifact tree only through a pinned phase handoff that verifies the binary,
repository, config, JSON, diagnostic asset, replay, backdrop, and
native-manifest hashes
before reference capture. The two retained #4147 IPC commits provide the
window and scrolling geometry required by that client. The live desktop is
never used. A per-run `XDG_CONFIG_HOME` contains both
`niri/config.kdl` and `niri/niri-glass.json`; neither program can read the
live-desktop configuration.

Three phase-specific scene states cover the parameter set:

- **Static optics:** the committed 40-by-40 diagnostic PNG (SHA-256
  `6fafae8c6cf3e3815346128ffdb402d5c730ae3a8749cb060013821ba0fe0316`)
  tiled across the live backdrop. Both implementations first capture a
  slab-free source, then map the same static probe to render the material
  pane. The pane ROI comes from IPC geometry and a bounded pixel-response
  cross-check; no preview or phase-equivalent source participates.
- **Geometry:** the same controlled pane/window and committed diagnostic PNG
  with both niri configs and reference `layoutGaps` derived from 24 to 80. The
  static source pair remains a valid slab-free geometry source. The larger
  inset keeps the frozen reference's 24/16/36/44 directional shadow reaches
  inside the output, so settled frames can measure lip and shift without
  clipping.
- **Motion:** continue from static optics in the committed 24-pixel layout.
  A deterministic `move-column-right` then `move-column-left` reorder supplies
  capture bursts for jelly flex and ripple, exercising the native
  `window-movement` spring and the reference's matching replay without
  involving `horizontal-view-movement` (which defaults to an easing curve,
  not that spring). The probe uses app ID `v1-parity-probe`; the native fixture
  assigns its material with a matching window rule.

Every default and variant static state is captured twice. Raw RGB drift within
either state must affect less than 0.1% of pixels or the case is rejected
before its parameter response is analyzed.
After motion, the geometry default continues from the same static probe through
a one-way bounded layout transition; no preview or transparent anchor
participates. A return to 24 pixels is intentionally not part of the evidence:
the frozen native surface restores IPC window geometry after an 80-to-24
round trip but does not reproduce its initial optical pixels.

## Pinned reference controls

The reference fixture writes all 24 `conf` properties explicitly. Fourteen
are the shared parameters in the capture matrix; the other ten are pinned as
follows:

| Reference property | Pinned value | Reason |
| --- | --- | --- |
| `enabled` | `true` | The live-pane geometry and motion scene must render. |
| `layoutGaps` | `24` base; `80` during geometry only | Matches native gaps in each phase. The 80-pixel geometry derivation prevents directional-shadow clipping; static and motion remain at 24 before geometry. |
| `paneApps` | `["v1-parity-probe"]` | Selects the same controlled probe as the native window rule. |
| `roughness` | `0` | Removes Qt mip-LOD blur, which native v1 does not implement and which would confound the blur/sample rows. |
| `gridOverlay` | `false` | Removes the live-pane shader's screen-space grid overlay. |
| `calibrate` | `false` | Removes the reference-only calibration overlay. |
| `probeExposure` | `0` | Removes Qt-only environment specular from the differential. |
| `springDampingRatio` | `1.0` | Matches native `window-movement`. |
| `springStiffness` | `100` | Matches the slowed native `window-movement` instrument. |
| `springEpsilon` | `0.0001` | Matches native `window-movement`. |

The committed diagnostic PNG is the controlled optical input; it is distinct from the
`gridOverlay` knob. `PreviewSurface` does not pass `gridOverlay` and therefore
uses `GlassMaterial`'s `false` default, while the explicit fixture value keeps
the live-pane scene equally free of the procedural overlay.

## Capture matrix

There is one default static capture set, one default motion burst, and one
high-signal variant per shared v1 parameter:

| Parameter | Scene | Default -> variant | Response oracle |
| --- | --- | --- | --- |
| `ior` | Static optics | `1.5 -> 2.0` | Refraction displacement increases. |
| `thickness` | Static optics | `20 -> 80` | Refraction displacement and attenuation increase. |
| `attenuation-color` | Static optics | `#dfe8ff -> #80ffff` | Red attenuation increases, green attenuation decreases, and blue remains an identity channel. |
| `attenuation-distance` | Static optics | `60 -> 5` | Beer-Lambert attenuation increases. |
| `chromatic-aberration` | Static optics | `0 -> 0.5` | RGB edge separation increases. |
| `distortion` | Static optics | `0 -> 0.5` | A nonzero, spatially nonuniform response remains confined to the material extent. |
| `distortion-scale` | Static optics | `0.5 -> 1.5` | The reconstructed warp field's spatial frequency increases relative to the `distortion=0.5` capture. |
| `anisotropic-blur` | Static optics | `0 -> 0.5` | Directional sampling spread widens. |
| `samples` | Static optics | `4 -> 8` | Sampling roughness decreases relative to the `anisotropic-blur=0.5` capture. |
| `jelly-flex` | Geometry and motion | `0.004 -> 0.02` | Mean positive directional bevel shear increases. |
| `jelly-ripple` | Geometry and motion | `0.06 -> 0.5` | Peak translation-removed inner-face ripple increases; the settled-frame drift remains below 0.1%. |
| `lip` | Geometry and motion | `6 -> 24` | The data-derived slab extent grows. |
| `shift-x` | Geometry and motion | `6 -> -6` | The slab relocates in the expected horizontal direction without changing extent. |
| `shift-y` | Geometry and motion | `6 -> -6` | The slab relocates in the expected vertical direction without changing extent. |

`distortion-scale` is inert while `distortion` is zero, so its reference is
the already-required `distortion=0.5` variant. Likewise, `samples` is inert
when both multi-tap effects are zero, so its reference is the already-required
`anisotropic-blur=0.5` variant. These dependency edges preserve isolation
without adding prerequisite-only captures.

The distortion-scale oracle measures normalized neighbouring-vector delta on
the recovered local displacement field, not the composed image's spectrum.
The asymmetric watermark breaks sub-period ambiguity, while the plus-or-minus
15-pixel search remains inside the diagnostic tile's 20-pixel half-period. A
best displacement on the search boundary rejects the evidence rather than
capping the response.

Native default/variant captures keep the same compositor process, window,
material definition name, and material assignment. Parameter changes use an
in-place config reload, which preserves `MaterialState` and its per-state
`jelly_seed` allocated from the process-global counter; restarting the
compositor or swapping the material name within a pair invalidates the pair.
The reference likewise keeps the same window ID, which is its jelly-noise
seed.

Each motion setting is run twice. The pinned critical spring uses stiffness
100 and every burst requests captures at 0, 25, 50, 75, 100, 150, 200, 300,
400, 600, 800, and 1100 ms. Every burst must retain at least three actual
captures before 326 ms, a capture at or after 400 ms, and a settled capture at
or after 500 ms. Actual capture offsets are recorded. Before evidence capture,
each implementation runs one default calibration burst through the same
column-reorder and screenshot path. Its first actual offset sets that implementation's
evidence budget, rounded up to the next 10 ms plus a 20 ms scheduling
allowance; the calibration must itself satisfy the actual-capture coverage
gate.
Within a burst, record each actual offset when its screenshot IPC returns, but
do not wait for or rename any asynchronously encoded PNG until every scheduled
screenshot IPC has been issued. PNG encoding latency is not part of the
sampling schedule.
Each frame recovers pane translation and an independent progress coordinate
from its source-to-frame response box before bevel registration correction.
For flex, long left/right edge strips measure normalized directional bevel
shear. A plus-or-minus 96-pixel bevel-location search covers the observed
72–76-pixel frozen-reference travel and an 80-pixel synthetic control; a
winner at 96 rejects integrity. This location bound is independent of the
tile-period-limited displacement search. For ripple, whole-pane translation is removed before measuring the
high-frequency residual inside the independent 32-pixel face inset, with
every inset pixel—including zero residuals—counted in the denominator. The four
bursts are linearly interpolated over their common pane-progress range; pixels
are never interpolated. Each duplicate variant burst is paired with its
default duplicate. Signal is the mean of the two paired deltas, noise is their
full separation, and both deltas must be positive. Missing capture support,
non-monotonic pane progress, or search saturation rejects artifact integrity
with analyzer exit `2`; a valid wrong-direction pair is a semantic failure
with exit `1`.

## Gates

The run fails early if any integrity condition holds:

- a repository commit, binary, config, fixture, output size, or capture hash
  cannot be recorded;
- a phase handoff has a mismatched pin/hash, wrong phase, missing shared file,
  partial reference directory, or pre-existing final manifest;
- either compositor mode reports invalid config, material fallback, shader
  failure, panic, or an unexpected warning/error;
- Quickshell reports a QML, shader, IPC, or wallpaper-input failure;
- the committed tile's pixel at `(20,20)` is not exactly `srgb(44,54,59)` or
  its ImageMagick normalized standard deviation is below `0.02`;
- duplicate static captures reach the 0.1% drift ceiling;
- an analysis check cannot be shown to fail on a synthetic bad input.

ImageMagick supplies raw 8-bit RGB dumps, but its HDRI `AE` metric is not used
for pixel counts. Ratio and attenuation arithmetic decode sRGB to linear
first. Geometry is derived from responsive pixels rather than assumed from
window bounds. Synthetic controls cover each analysis class used by the pass:
channel/sign, spatial structure or frequency, geometry, and motion.

Every row uses its own oracle metric for both signal and noise. For static
rows, the minimum default-to-variant response across the duplicate captures
must be strictly greater than the maximum within-default or within-variant
duplicate drift measured with that metric. Motion variants must similarly
exceed their paired-delta separation. A nonzero
response that does not clear its case's measured floor fails. A response that
escapes its data-derived material extent also fails its row.

All 14 rows must pass in both implementations. Results may report magnitude
differences, but no unreviewed numeric cross-renderer similarity threshold is
introduced. Oracle failures are recorded and the remaining independent rows
continue so the one-time run produces a complete discrepancy table; any such
failure blocks production changes, physical DRM smoke, and v1 acceptance.

## Exclusions

The results document records the pinned controls above as intentional
non-parity or instrument surface:

- `roughness`, which native v1 deliberately omits until background mip
  storage exists;
- `enabled`, `layoutGaps`, and `paneApps`, which select and align the external
  client's panes rather than define glass optics;
- `gridOverlay`, `calibrate`, and `probeExposure`, which configure the
  Quickshell scene or instrument rather than the glass contract;
- `springDampingRatio`, `springStiffness`, and `springEpsilon`, which replay
  compositor motion externally and are unnecessary for native jelly; and
- Qt-only environment/specular behavior and focus glint.

## Config surface review (resolved)

The Quickshell client was a prototype for the idea, not a proposal for the
long-term API, and `material { glass { ... } }` becomes a released
compatibility surface at v1. This pass deliberately changes no parameter, so
the following are recorded as the required input to a separate config-surface
design that must complete before v1 acceptance. That design is implemented in
`e579dae5`, `0c5f809f`, `6cd06de2`, `73a733db`, and `b8fe7b84`; the historical
findings below remain the input to that work and the parity pass remains
pinned to the old surface. The shader's quadrant selection and rendered slab
radii still have no automated or cheap manual check in this repository, so
rendered appearance remains explicitly unverified:

- **`thickness` is not the slab's thickness.** `SLAB_DEPTH` is fixed at 12
  logical px while `thickness` defaults to 20 and ranges to 200. It is a
  strength knob wearing a length's name, and it drives three unrelated
  quantities: screen-space refraction displacement, the Beer-Lambert optical
  path, and the anisotropic smear width. Either make it the real depth and let
  it drive the geometry, or rename it for what it does and let
  `attenuation-distance` own absorption alone.
- **The corner radius ignores the window's.** `SLAB_CORNER_RADIUS` is fixed at
  28 logical px, while `geometry-corner-radius` is already a per-window rule
  the tile resolves. The in-compositor implementation has the real geometry the
  external client never had and should use it.
- **`lip` plus `shift-x`/`shift-y` is prototype-shaped.** These exist because
  the external client faked a pedestal by offsetting a same-size pane. What a
  user sees is the chamfer band, which is derived as
  `lip + max(abs(shift-x), abs(shift-y))`; and because the bevel depth is
  `min(chamfer, depth)`, raising `lip` past the fixed 12 px depth changes the
  bevel angle rather than only its width. The shader also clamps the chamfer to
  `min(half_ext.x, half_ext.y) - 1`, so on small windows the configured values
  silently stop taking effect. Expose the bevel directly.
- **`samples` is a performance dial presented as an appearance knob.** It is
  inert unless a multi-tap effect is active — the reason the capture matrix
  needs a dependency edge for it. Derive it from a quality setting or from the
  effect magnitudes. `distortion-scale` has the same inert-until-activated
  shape and is a candidate to fold into `distortion`.

`roughness` remains the highest-value missing parameter. It appears under
Exclusions because native v1 cannot express it (deferred to G4 in the parent
design), not because it is unwanted.

Modelling should follow drei's `MeshTransmissionMaterial`, the original
inspiration, for the optics rather than the prototype's exact API. Most of
what the prototype did not port should stay unported: backside and resolution
controls are Three.js mesh concerns, and temporal distortion is superseded by
jelly riding real compositor animation residuals.

## Evidence and repository boundaries

Generated PNGs, raw RGB dumps, phase state, and logs remain untracked in a
unique directory under `NIRI_MATERIAL_WORK_ROOT`. A failed phase rejects the
entire attempt and cannot resume partially. Historical implementation branch
`results/reference-static-preflight` was based on `results/slice3` at
`7729dfc151eae41c946d2495ef67010c1cd20635`. Capture base
`b851e5208b54cc466d99bf3ae664cc5a52c2317f` contains the final replay and
config fixtures used for both captures. Result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f` contains the final analyzer,
self-checks, capture hashes, measured tables, and
`docs/results/2026-08-24-v1-parity.md`. Additive result commit
`af99babfdeed1f64e3bf52817b62a22c9d1c9d72` records the geometry preflight.
The frozen `niri-glass` source is not modified.

The production implementation is unchanged by this pass. Result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f` records two corrected complete
captures and their semantic PASS: 14/14 combined parameters pass. The
config-surface review is already implemented. The corrected physical DRM run
passed at `niri-experiments` result commit
`16d2b4aa957637fa9339fc4bc0c4f7bd7b4aca34`; native materials v1 is
accepted.

## Alternatives rejected

- **Normalized cross-renderer image similarity:** rejected because Qt scene
  lighting, wallpaper duplication, and native window composition make the
  number sensitive to intentional pipeline differences.
- **Manual side-by-side approval:** rejected because it supplies no
  falsifiable per-parameter gate.
- **Reusable parity harness:** deferred because this is a frozen-reference,
  one-time release gate. The retained fixtures are sufficient to audit or
  repeat the evidence if the v1 verdict is challenged.
