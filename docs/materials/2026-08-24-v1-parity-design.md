# Native materials v1 parity pass: design

**Status:** approved 2026-08-24. Implementation planning pending.
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
| `niri-experiments` | `e59786070eced3b8891bb61e0cc241733c7f81ed` | Slice 3 evidence and reusable capture method |

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
never used.

Two controlled scenes cover the parameter set:

- **Static optics:** a fixed diagnostic grid and transparent, static probe.
  The reference side uses its isolated diagnostic preview. The native side
  uses the same diagnostic source behind a material window. Cursor blinking,
  live terminal output, focus glint, and unrelated animation are absent.
- **Geometry and motion:** one controlled pane/window on an otherwise quiet
  workspace. Settled frames measure lip and shift. A single deterministic
  move supplies short capture bursts for jelly flex and ripple; peak response
  is selected from the burst rather than assuming the two animation engines
  reach their peaks at the same wall-clock offset.

Every static input is captured twice before its variant. Raw RGB drift must
affect less than 0.1% of pixels or the case is rejected before its parameter
response is analyzed.

## Capture matrix

There is one default static capture set, one default motion burst, and one
high-signal variant per shared v1 parameter:

| Parameter | Default -> variant | Response oracle |
| --- | --- | --- |
| `ior` | `1.5 -> 2.0` | Refraction displacement and Fresnel edge response increase. |
| `thickness` | `20 -> 80` | Refraction displacement and attenuation increase. |
| `attenuation-color` | `#dfe8ff -> #80ffff` | Attenuation changes in the predicted channels; blue remains an identity channel. |
| `attenuation-distance` | `60 -> 5` | Beer-Lambert attenuation increases. |
| `chromatic-aberration` | `0 -> 0.5` | RGB edge separation increases. |
| `distortion` | `0 -> 0.5` | A nonzero, spatially nonuniform response remains confined to the material extent. |
| `distortion-scale` | `0.5 -> 1.5` | Warp spatial frequency increases relative to the `distortion=0.5` capture. |
| `anisotropic-blur` | `0 -> 0.5` | Directional sampling spread widens. |
| `samples` | `4 -> 8` | Sampling becomes smoother relative to the `anisotropic-blur=0.5` capture. |
| `jelly-flex` | `0.004 -> 0.02` | Peak movement deformation increases. |
| `jelly-ripple` | `0.06 -> 0.5` | Transient optical ripple increases while the settled frame remains unchanged. |
| `lip` | `6 -> 24` | The data-derived slab extent grows. |
| `shift-x` | `6 -> -6` | The slab relocates in the expected horizontal direction without changing extent. |
| `shift-y` | `6 -> -6` | The slab relocates in the expected vertical direction without changing extent. |

`distortion-scale` is inert while `distortion` is zero, so its reference is
the already-required `distortion=0.5` variant. Likewise, `samples` is inert
when both multi-tap effects are zero, so its reference is the already-required
`anisotropic-blur=0.5` variant. These dependency edges preserve isolation
without adding prerequisite-only captures.

## Gates

The run fails early if any of these conditions holds:

- a repository commit, binary, config, fixture, output size, or capture hash
  cannot be recorded;
- either compositor mode reports invalid config, material fallback, shader
  failure, panic, or an unexpected warning/error;
- Quickshell reports a QML, shader, IPC, or wallpaper-input failure;
- duplicate static captures reach the 0.1% drift ceiling;
- a response escapes its data-derived material extent;
- either implementation fails any row's response oracle; or
- an analysis check cannot be shown to fail on a synthetic bad input.

ImageMagick supplies raw 8-bit RGB dumps, but its HDRI `AE` metric is not used
for pixel counts. Ratio and attenuation arithmetic decode sRGB to linear
first. Geometry is derived from responsive pixels rather than assumed from
window bounds. Synthetic controls cover each analysis class used by the pass:
channel/sign, spatial structure or frequency, geometry, and motion.

All 14 rows must pass in both implementations. Results may report magnitude
differences, but no unreviewed numeric cross-renderer similarity threshold is
introduced.

## Exclusions

The results document records these reference-only controls as intentional
non-parity surface:

- `roughness`, which native v1 deliberately omits until background mip
  storage exists;
- `gridOverlay`, `calibrate`, and `probeExposure`, which configure the
  Quickshell scene or instrument rather than the glass contract;
- `paneApps`, an external-client allowlist replaced by native window rules;
- `springDampingRatio`, `springStiffness`, and `springEpsilon`, which replay
  compositor motion externally and are unnecessary for native jelly; and
- Qt-only environment/specular behavior and focus glint.

## Evidence and repository boundaries

Generated PNGs, raw RGB dumps, and logs remain untracked in a unique temporary
artifact directory. `niri-experiments` receives only the minimum replay and
config fixtures needed to reproduce the run, capture hashes, measured tables,
and `docs/results/2026-08-24-v1-parity.md`. The frozen `niri-glass` source is
not modified.

The production implementation is also unchanged by this pass. If parity
fails, execution stops and the discrepancy becomes a separately designed
bugfix. A passing evidence commit permits `niri-material` to update the v1
design and materials status to “parity complete; physical DRM pending.” It
does not itself satisfy the physical DRM gate or v1 acceptance.

## Alternatives rejected

- **Normalized cross-renderer image similarity:** rejected because Qt scene
  lighting, wallpaper duplication, and native window composition make the
  number sensitive to intentional pipeline differences.
- **Manual side-by-side approval:** rejected because it supplies no
  falsifiable per-parameter gate.
- **Reusable parity harness:** deferred because this is a frozen-reference,
  one-time release gate. The retained fixtures are sufficient to audit or
  repeat the evidence if the v1 verdict is challenged.
