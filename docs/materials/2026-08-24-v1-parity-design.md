# Native materials v1 parity pass: design

**Status:** approved 2026-08-24; amended after source review, then narrowed to
the optics port with a config-surface review added as a separate v1 gate.
Implementation plan drafted; execution paused pending review.
**Parent design:** `docs/materials/2026-08-22-v1-design.md`

## Goal

Run a one-time, documented parity pass between native glass and the frozen
Quickshell reference before physical DRM acceptance. The pass must show that
every parameter shared by the two implementations produces the same semantic
response, without requiring pixel identity between different rendering and
composition pipelines.

The pass verifies the **optics port** — that the native shader reproduces the
reference's parameter responses. It does not evaluate whether the shared
parameter set is the right long-term configuration surface. That question
belongs to the separate config-surface review below, which also gates v1.

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

Both sides run sequentially under the dedicated 1280x720 headless Weston GL
host. They use the same release build of the pinned native compositor: native
mode enables the material, while reference mode disables it and runs the
frozen Quickshell client. The two retained #4147 IPC commits provide the
window and scrolling geometry required by that client. The live desktop is
never used. A per-run `XDG_CONFIG_HOME` contains both
`niri/config.kdl` and `niri/niri-glass.json`; neither program can read the
live-desktop configuration.

Two controlled scenes cover the parameter set:

- **Static optics:** the pinned diagnostic SVG (SHA-256
  `ac7c503e96fbfde40ce3a9cb3b1b069814f381f410d169e56f03bcf4db3308db`)
  tiled at its 200 px period, plus a transparent, static probe.
  The reference side uses its isolated diagnostic preview. The native side
  uses the same diagnostic source behind a material window. Cursor blinking,
  live terminal output, focus glint, and unrelated animation are absent. The
  preview cannot expose a separate slab-free frame, so reference registration
  uses phase-equivalent, unmasked 200 px tiles from each preview capture. This
  keeps the source in the same Qt rendering path as the deformed pixels.
- **Geometry and motion:** one controlled pane/window on an otherwise quiet
  workspace over a generated 1280x720 backdrop made by tiling the pinned SVG
  at 200 px. Both swaybg and the reference wallpaper stub consume that exact
  PNG, so the frozen client's crop-fill mapping is identity rather than a
  6.4x magnification. The generated tile must retain its expected background
  color and nonzero spatial variation; the generated PNG and raw RGB hashes
  are recorded.
  Settled frames measure lip and shift. A deterministic column resize supplies
  capture bursts for jelly flex and ripple, exercising the native
  `window-resize` spring and the reference's matching replay without involving
  `horizontal-view-movement` (which defaults to an easing curve, not that
  spring). The probe uses app ID `v1-parity-probe`; the native fixture assigns
  its material with a matching window rule.

Every default and variant static state is captured twice. Raw RGB drift within
either state must affect less than 0.1% of pixels or the case is rejected
before its parameter response is analyzed.

## Pinned reference controls

The reference fixture writes all 24 `conf` properties explicitly. Fourteen
are the shared parameters in the capture matrix; the other ten are pinned as
follows:

| Reference property | Pinned value | Reason |
| --- | --- | --- |
| `enabled` | `true` | The live-pane geometry and motion scene must render. |
| `layoutGaps` | `24` | Matches an explicit native `layout { gaps 24; }` so IPC-derived pane geometry stays aligned. |
| `paneApps` | `["v1-parity-probe"]` | Selects the same controlled probe as the native window rule. |
| `roughness` | `0` | Removes Qt mip-LOD blur, which native v1 does not implement and which would confound the blur/sample rows. |
| `gridOverlay` | `false` | Removes the live-pane shader's screen-space grid overlay. |
| `calibrate` | `false` | Removes the reference-only calibration overlay. |
| `probeExposure` | `0` | Removes Qt-only environment specular from the differential. |
| `springDampingRatio` | `1.0` | Matches native `window-resize`. |
| `springStiffness` | `800` | Matches native `window-resize`. |
| `springEpsilon` | `0.0001` | Matches native `window-resize`. |

The diagnostic SVG is the controlled optical input; it is distinct from the
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
| `jelly-flex` | Geometry and motion | `0.004 -> 0.02` | Peak movement deformation increases. |
| `jelly-ripple` | Geometry and motion | `0.06 -> 0.5` | Transient optical ripple increases; the settled-frame drift remains below 0.1%. |
| `lip` | Geometry and motion | `6 -> 24` | The data-derived slab extent grows. |
| `shift-x` | Geometry and motion | `6 -> -6` | The slab relocates in the expected horizontal direction without changing extent. |
| `shift-y` | Geometry and motion | `6 -> -6` | The slab relocates in the expected vertical direction without changing extent. |

`distortion-scale` is inert while `distortion` is zero, so its reference is
the already-required `distortion=0.5` variant. Likewise, `samples` is inert
when both multi-tap effects are zero, so its reference is the already-required
`anisotropic-blur=0.5` variant. These dependency edges preserve isolation
without adding prerequisite-only captures.

The distortion-scale oracle registers the variant against the undeformed
default grid and measures frequency on the resulting displacement field, not
on the composed image's spectrum. Registration accepts a block only when both
its row-mean and column-mean luminance profiles clear the texture threshold;
this prevents a one-axis grid edge from silently supplying a zero for the
other axis. A best displacement on the search boundary rejects the row rather
than capping the measured response. The same pinned SVG and 200 px tiling
period on both sides prevent the background itself from changing that
estimate.

Native default/variant captures keep the same compositor process, window,
material definition name, and material assignment. Parameter changes use an
in-place config reload, which preserves `MaterialState` and its per-state
`jelly_seed` allocated from the process-global counter; restarting the
compositor or swapping the material name within a pair invalidates the pair.
The reference likewise keeps the same window ID, which is its jelly-noise
seed.

Each motion setting is run twice. The pinned critical spring has an analytical
settle duration of about 326 ms (`-ln(0.0001) / sqrt(800)`), so every burst
must contain at least three actual captures before 326 ms, a capture at or
after 400 ms, and a settled capture at or after 500 ms. Actual capture offsets
are recorded. Before evidence capture, each implementation runs one default
calibration burst through the same resize and screenshot path. Its first
actual offset sets that implementation's evidence budget, rounded up to the
next 10 ms plus 10 ms; the calibration must itself satisfy the actual-capture
coverage gate.
For each default/variant case, its four duplicate bursts are linearly
interpolated onto a shared 10 ms grid over their common actual-offset range;
only scalar deformation and ripple metrics are interpolated, never pixels.
The grid must begin within the measured first-offset budget and extend through
400 and 500 ms. Duplicate default and variant curves establish the motion
noise floor from their aligned peak-metric spread. Missing actual-capture
coverage, non-monotonic offsets, or disagreement in response direction between
duplicate curves rejects the motion evidence.

## Gates

The run fails early if any integrity condition holds:

- a repository commit, binary, config, fixture, output size, or capture hash
  cannot be recorded;
- either compositor mode reports invalid config, material fallback, shader
  failure, panic, or an unexpected warning/error;
- Quickshell reports a QML, shader, IPC, or wallpaper-input failure;
- the generated tile's pixel at `(20,20)` is not exactly `srgb(38,50,56)` or
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
exceed the maximum within-setting duplicate-burst peak spread. A nonzero
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

## Config surface review (out of scope here, gating v1)

The Quickshell client was a prototype for the idea, not a proposal for the
long-term API, and `material { glass { ... } }` becomes a released
compatibility surface at v1. This pass deliberately changes no parameter, so
the following are recorded as the required input to a separate config-surface
design that must complete before v1 acceptance:

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

Generated PNGs, raw RGB dumps, and logs remain untracked in a unique temporary
artifact directory. A `results/v1-parity` branch and worktree based on
`e597860` receive only the minimum replay and config fixtures needed to
reproduce the run, capture hashes, measured tables, and
`docs/results/2026-08-24-v1-parity.md`. The frozen `niri-glass` source is not
modified.

The production implementation is also unchanged by this pass. If any oracle
fails, the full independent matrix is still recorded, then production and DRM
work stop and the discrepancies become separately designed bugfixes. A
passing evidence commit permits `niri-material` to update the v1 design and
materials status to “optics port verified; config-surface review and physical
DRM pending.” It does not itself satisfy the config-surface review, the
physical DRM gate, or v1 acceptance.

## Alternatives rejected

- **Normalized cross-renderer image similarity:** rejected because Qt scene
  lighting, wallpaper duplication, and native window composition make the
  number sensitive to intentional pipeline differences.
- **Manual side-by-side approval:** rejected because it supplies no
  falsifiable per-parameter gate.
- **Reusable parity harness:** deferred because this is a frozen-reference,
  one-time release gate. The retained fixtures are sufficient to audit or
  repeat the evidence if the v1 verdict is challenged.
