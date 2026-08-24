# Native materials v1 parity pass: design

**Status:** approved 2026-08-24; amended after source review. Implementation
planning pending.
**Parent design:** `docs/materials/2026-08-22-v1-design.md`

## Goal

Run a one-time, documented parity pass between native glass and the frozen
Quickshell reference before physical DRM acceptance. The pass must show that
every parameter shared by the two implementations produces the same semantic
response, without requiring pixel identity between different rendering and
composition pipelines.

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
  live terminal output, focus glint, and unrelated animation are absent.
- **Geometry and motion:** one controlled pane/window on an otherwise quiet
  workspace. Settled frames measure lip and shift. A deterministic column
  resize supplies capture bursts for jelly flex and ripple, exercising the
  native `window-resize` spring and the reference's matching replay without
  involving `horizontal-view-movement` (which defaults to an easing curve,
  not that spring). Peak response is selected from each burst rather than
  assuming the two animation engines reach their peaks at the same wall-clock
  offset.

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
| `paneApps` | `["v1-parity-probe"]` | Selects only the controlled probe. |
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
on the composed image's spectrum. The same pinned SVG and 200 px tiling period
on both sides prevent the background itself from changing that estimate.

Native default/variant captures keep the same compositor process, window,
material definition name, and material assignment. Parameter changes use an
in-place config reload, which preserves `MaterialState` and its process-global
`jelly_seed`; restarting the compositor or swapping the material name within
a pair invalidates the pair. The reference likewise keeps the same window ID,
which is its jelly-noise seed.

Each motion setting is run twice. The pinned critical spring has an analytical
settle duration of about 326 ms (`-ln(0.0001) / sqrt(800)`), so every burst
must contain an active sample and extend through at least 400 ms, followed by
a settled capture at 500 ms or later. Actual capture offsets are recorded.
Duplicate default and variant bursts establish the motion noise floor from
their peak-metric spread. A missing active or settled sample, or disagreement
in response direction between duplicate bursts, rejects the motion evidence.

## Gates

The run fails early if any integrity condition holds:

- a repository commit, binary, config, fixture, output size, or capture hash
  cannot be recorded;
- either compositor mode reports invalid config, material fallback, shader
  failure, panic, or an unexpected warning/error;
- Quickshell reports a QML, shader, IPC, or wallpaper-input failure;
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
materials status to “parity complete; physical DRM pending.” It does not
itself satisfy the physical DRM gate or v1 acceptance.

## Alternatives rejected

- **Normalized cross-renderer image similarity:** rejected because Qt scene
  lighting, wallpaper duplication, and native window composition make the
  number sensitive to intentional pipeline differences.
- **Manual side-by-side approval:** rejected because it supplies no
  falsifiable per-parameter gate.
- **Reusable parity harness:** deferred because this is a frozen-reference,
  one-time release gate. The retained fixtures are sufficient to audit or
  repeat the evidence if the v1 verdict is challenged.
