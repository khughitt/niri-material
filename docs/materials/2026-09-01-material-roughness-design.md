# Material roughness and prefiltered backdrop storage: design

**Status:** accepted 2026-09-01; review amendments incorporated 2026-09-01;
implementation has not started. Tracked by `material-c854bd`. The delivery
includes the matching Prism parameter and niri sink wiring.

## Context

Native glass has no `roughness` control. The accepted v1 design deliberately
omitted it because the per-target background and backdrop buffers had only
level-zero textures; adding a uniform without softened storage would have made
the knob inert. The v1 parity review identified roughness as the highest-value
missing parameter, and the DRM acceptance design named damage-aware,
buffer-owned mipmapped or prefiltered storage as the first post-v1 follow-up.

The renderer now has two possible level-zero sources for each `EffectBuffer`:
the sharp offscreen texture and the full-size globally blurred texture selected
by `backdrop-blur`. Roughness is independent of that switch. The switch first
chooses the source; roughness then softens the chosen source.

The retired reference used hardware mip levels and selected LOD as:

```text
log2(texture width) * roughness * clamp(ior * 2 - 2, 0, 1)
```

Native niri still targets GLES2 renderers and non-power-of-two output sizes.
GLES2 does not universally provide NPOT mipmaps or explicit fragment LOD, and
bias-based implicit LOD changes with overview transforms. The native renderer
therefore uses explicit prefiltered textures rather than hardware mipmaps.

## Decision

Each per-target `EffectBuffer` lazily owns a reusable prefilter pyramid for its
sharp texture and another for its globally blurred texture. The pyramid uses
the existing blur downsample program with a fixed offset of `1`; it does not
inherit the global blur offset or add a second shader.

The blur's existing intermediate textures cannot double as this storage. Its
up pass overwrites every intermediate level except the smallest, and its level
count and filter offset follow the global blur configuration. Roughness needs a
stable, complete downsample chain independent of those settings.

Each pyramid separates policy from GL execution. A plain `PrefilterState`
contains the source size, level count, dirty state, highest prepared level, and
level-selection math, with no renderer or `GlesTexture` dependency. The GL
owner holds the actual texture vector and only executes allocation and
downsample decisions returned by that policy. This follows the existing
`MaterialState` pattern: state transitions and commit decisions remain runnable
without a GPU, while the concrete renderer performs their effects.

Each level halves both dimensions, clamped to one, until `1 x 1`. Levels exclude
the full-size source. Ignoring integer rounding and the one-pixel clamp, their
pixel count is the geometric series:

```text
(w/2 * h/2) + (w/4 * h/4) + ... <= w*h/3
```

The discrete sum can sit slightly above one-third for some aspect ratios; a
`4 x 8` source, for example, derives 11 pixels rather than the idealized 10.67.
The exact universal invariant for every positive source size is that derived
storage remains below one additional source. Normal output sizes remain very
close to one-third, and the exact sum is computed before allocation rather than
assumed from the approximation.

The pyramid is allocated only when a material requests non-zero roughness from
that exact sharp or blurred source. Its textures survive content damage and are
redrawn in place. Size or renderer-context replacement recreates them.

For a pyramid with maximum level `max_level`, selection is:

```text
lod = max_level * roughness * clamp(ior * 2 - 2, 0, 1)
low = floor(lod)
high = min(low + 1, max_level)
mix = lod - low
```

Level zero is the existing full-size source. The material binds the two selected
background levels and two selected backdrop levels alongside the window
texture: five texture units, within GLES2's minimum of eight. The shader samples
only `low` when `mix == 0`; native `roughness 0` therefore takes the existing
level-zero path without a second texture lookup. Its unused `high` sampler is
still bound to level zero, so no incomplete texture is ever attached.

The background and backdrop keep separate level pairs and blend factors even
though current targets size them alike. Their interface remains explicit and
does not rely on that incidental equality.

## Configuration contract

Native niri adds one optional `glass` child, parsed with the existing
`FloatOrInt<0, 1>` boundary type used by other fractional glass controls:

| name | type | default | range |
| --- | --- | --- | --- |
| `roughness` | float | `0` | `0..1` |

The zero default preserves every existing native material. Values outside the
range reject the whole configuration through the existing boundary validation;
there is no clamp or fallback.

Prism restores its historical public parameter:

| Prism parameter | Native KDL | default | range | liveness |
| --- | --- | --- | --- | --- |
| `glass.roughness` | `roughness` | `0.08` | `0..1` | reload |

Prism intentionally chooses a non-zero default so its visible control has an
active result. Unmanaged niri configurations retain native v1 appearance.
Prism emits the value unconditionally in its generated material definition,
beside the other glass optics.

`backdrop-blur` remains an independent boolean. With it off, roughness samples
the sharp source's pyramid. With it on, roughness samples a pyramid generated
from the globally blurred full-size source. `blur { off }` continues to disable
the global blur selection but does not disable roughness of the resulting sharp
source.

Roughness controls prefilter LOD only. It does not alter anisotropic smear,
sample count, distortion, chromatic aberration, or global blur strength.

## Renderer ownership and data flow

`niri-config/src/material.rs` parses and resolves `roughness`. Parameter edits
reuse the existing `MaterialState`, bump its commit, and preserve its window
offscreen storage.

`src/render_helpers/blur.rs` gains the minimum reusable prefilter storage and a
downsample-only path extracted from the existing blur program. Its GL layer
owns textures and executes `PrefilterState` decisions; it owns no policy about
materials, targets, damage, or optics.

`src/render_helpers/effect_buffer.rs` owns the two pyramids and their dirty
state. Each `EffectBuffer` owns one `PrefilterState` for its sharp cache and one
for its blurred cache, and hands their decisions to the GL executor in
`blur.rs`. Its render path:

1. selects the sharp or globally blurred full-size source using
   `backdrop-blur`;
2. returns level zero directly when roughness selects it;
3. otherwise prepares or refreshes that source's pyramid;
4. returns the two adjacent textures and blend factor.

`src/render_helpers/material.rs` passes the resolved roughness and IOR to both
effect buffers, binds both returned level pairs, and supplies their blend
factors to the shader. Workspace/background/backdrop composition remains in
the shader and remains in the same order; only each texture sample gains an
explicit prefilter blend.

The xray owner remains unchanged:
`[Rc<RefCell<EffectBuffer>>; RenderTarget::COUNT]` already isolates background
and backdrop buffers for each render target. Prefilter storage lives below that
boundary, so normal output, screencast, and capture targets cannot share or
invalidate one another's textures.

## Cache and damage contract

Actual level-zero damage marks the sharp pyramid dirty and invalidates the
full-size blurred result. A new blurred result marks the blurred pyramid dirty.
Changing blur options invalidates only the full-size blurred result and its
pyramid. Parameter-only roughness changes do not invalidate either pyramid;
they select different already-derived levels.

A dirty pyramid regenerates in full on its next non-zero roughness request.
Unchanged frames perform no downsample work and allocate no textures. This is
damage-triggered caching, not region-incremental mip propagation:

```rust
// ponytail: regenerate the full pyramid after any source damage; propagate
// only through the highest level requested this frame if profiling shows the
// full chain dominates changed frames; add per-level damage regions after that.
```

The ceiling is deliberate. A full regeneration touches roughly one-third as
many pixels as a normal output-sized source, is shared by every material window
sampling that source and target, and avoids a second damage-coordinate
implementation before measurement demonstrates a need.

The universal maximum additional storage per `EffectBuffer` is less than one
sharp source plus one blurred source; for normal outputs it is approximately
one-third of each. Both are lazy. A session that uses only sharp roughness never
allocates the blurred pyramid; roughness zero allocates neither.

Generating or selecting derived levels does not increment the effect-buffer
commit. The level-zero commit already records content and blur-option changes,
and a roughness parameter edit already advances the material element commit.

## Failure behavior

Invalid roughness is a configuration error and never reaches the renderer.

Texture allocation, downsample, missing-program, or renderer-context errors
remain renderer failures. `PrefilterState` records `Failed` for that dirty
period. The transition emits one warning naming the prefilter operation; later
material windows skip the same allocation/downsample attempt and take the
existing plain-window fallback without repeating the warning. Source damage,
blur-option change, size change, or context replacement starts a new dirty
period and permits one new attempt. There is no silent level-zero substitution,
because that would make a valid roughness value appear accepted while leaving
it inert.

## Prism integration

Prism updates the existing native niri sink rather than adding another
integration:

- `defs/glass.yaml` restores `glass.roughness` with range `0..1`, default
  `0.08`, percent presentation, and the historical Glass blur label;
- `integrations/niri/manifest.yaml` binds it with reload liveness;
- `integrations/niri/render.js` emits `roughness` in the glass definition;
- the definition and exact-KDL tests add it to the native surface and remove it
  from the retired-parameter list.

The authoritative native-sink design is corrected in the same Prism change.
The task-migration document currently says roughness requires no unfinished
Prism work; that outward claim is corrected at the same time. No compatibility
alias or second parameter is introduced.

Material owns the renderer work and its design. Prism owns its parameter,
generated KDL, sink tests, and documentation. The repositories land separate
conventional commits, with Prism depending on the native grammar being
available.

## Instrumentation and performance evidence

The permanent instrumentation answers two operational questions:

1. Why was a prefilter pyramid allocated or regenerated?
2. Did an unchanged frame perform prefilter work?

The implementation adds focused Tracy spans around pyramid preparation and
downsampling, plus trace-level allocation diagnostics consistent with the
existing offscreen and blur paths. Failures use warnings. It adds no metrics,
alerting, or new logging subsystem.

Performance is measured rather than inferred:

- record a before-change trace from the same nested scene and binary profile;
- record candidate traces at native roughness `0` and Prism roughness `0.08`;
- record a large-window worst case with a fractional LOD (Prism roughness
  `0.08` at the fixture size), anisotropic blur `1`, and chromatic aberration
  `1`: eight samples and three channel-separated taps make 24 background taps,
  whose worst-case fetch count rises from 48 at level zero to 96 with two-level
  prefilter blending;
- require zero pyramid allocations and zero downsample spans during a 60-second
  unchanged interval after warm-up;
- induce one background change and confirm one regeneration for each selected
  source and target, followed by reuse;
- compare matched GPU `render` samples only when sample counts differ by less
  than 10%, and report the measured delta rather than inventing a threshold.

If the prefilter path is within run-to-run noise, the architecture stays for
the new behavior but no speculative optimization is added. If it causes an
unexplained default-path regression, the change does not ship until that
regression is removed.

## Runnable verification

GPU-free Rust tests cover:

- parsing, resolving, defaulting, and rejecting `roughness`;
- `PrefilterState` transitions among empty, dirty, clean, and failed periods
  without constructing a renderer or texture;
- pyramid dimensions through `1 x 1`, exact pixel totals for representative
  output sizes, and the less-than-one-source bound for every positive size;
- level selection at zero, adjacent fractional levels, maximum roughness, and
  the IOR gate;
- source damage and blur-option invalidation versus parameter-only selection;
- roughness parameter edits advancing material damage while target fingerprints
  remain isolated.

Prism's Node tests cover the restored definition, range, default, UI metadata,
reload binding, exact generated KDL, and removal from the unsupported list.

Nested visual evidence uses the existing isolated Weston GL host and records:

- sharp-source captures at roughness `0`, `0.08`, and `1`;
- the same roughness values with `backdrop-blur` enabled;
- a level-zero capture compared with the current accepted output;
- a settled/overview pair at roughness `0.5`, so the checkerboard-edge
  10%-to-90% transition spans several captured pixels; each width is converted
  back to source-buffer pixels using the recorded zoom, and the difference must
  be no more than one captured screen pixel converted through that zoom
  (`1 / zoom` source pixels), while the on-screen width scales with the zoom;
- ROI statistics showing detail softening as roughness increases;
- logs free of prefilter or material fallback warnings.

Raw captures, traces, logs, and hashes remain outside the product tree under a
unique work directory, following the existing material evidence convention. A
tracked evidence document records the source commit, binary hash, artifact
manifests, measurements, observations, and cleanup proof.

## Documentation updates when implementation lands

Material updates:

- `docs/materials/material-config.md` adds the native parameter, default,
  interaction with `backdrop-blur`, and storage/cost summary;
- `docs/materials/README.md` links this design and records the implemented and
  verified result;
- a focused roughness evidence document records the nested and Tracy results;
- `material-c854bd` is completed only after both repositories land, in the
  final Material evidence and documentation commit.

The historical v1, parity, and DRM documents remain historically accurate:
they correctly say roughness was deferred at those decisions. They are not
rewritten to imply it existed in v1.

Prism updates its current native-sink design and the task-migration claim that
roughness had no pending Prism work, then greps user-facing documentation for
the same stale statement. Its integration task is completed with the generated
KDL and tests in the same Prism commit.

## Alternatives rejected

### Hardware GL mipmaps

This is the smallest apparent implementation, but not the smallest reliable
one. GLES2 NPOT mipmap support is not universal, explicit fragment LOD is not
core, and implicit bias varies with the element's output transform. Capability
variants or a fallback pyramid would add more code than one portable explicit
pyramid.

### Full-size texture per roughness value

This makes sampling simple but storage grows with the number of distinct
material values. It duplicates the existing global blur machinery and changes
an output-shared cache into a per-material cache. The bound and invalidation
contract become worse for no visual benefit.

### One fixed softened texture

A binary sharp/soft choice cannot implement a continuous `0..1` public
parameter. Interpolating between level zero and one fixed blur also bunches most
of the useful range into one transition and does not match the reference's
progressive LOD behavior.

## Non-goals

- Region-incremental prefilter updates before profiling justifies them.
- Per-material global blur passes, strength, offset, or pass count.
- Removing or redefining `backdrop-blur` or `anisotropic-blur`.
- Restoring retired sample-count, preview, calibration, or spring controls.
- Materials on layer surfaces, HDR/linear-light pipeline work, direct scanout,
  or a generic renderer texture-cache framework.
