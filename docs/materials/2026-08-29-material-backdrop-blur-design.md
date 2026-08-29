# Material backdrop blur: design

**Status:** proposed 2026-08-29; not implemented.

## Context

Glass currently refracts a sharp backdrop. Asking niri's own
`background-effect { blur true }` to soften it does not work, and the reason is
structural rather than a matter of tuning.

Two independent images of the same wallpaper are produced and composited. The
background effect blurs what is behind the window and draws it. The material
separately refracts `niri_tex_bg` and `niri_tex_backdrop`, the xray buffers
filled from `Layer::Background`, which the blur pass never touches. Raising
`passes` changes one image while the other stays sharp, so the two can never
agree. Measured on the daily driver: the result reads as two competing images,
and neither looks right.

This is also why the retired `niri-glass` Quickshell layer combined well with
niri's blur. Its panes were a `Bottom`-layer surface sitting behind the window
in compositing order, so the blur pass ran over a composite that already
included the glass. Those passes were sequential. The material lives inside the
window's own render and samples the Background layer directly, so they are
parallel.

The material's existing `anisotropic-blur` does not close the gap. It is drei's
MeshTransmissionMaterial mechanism: a multi-tap loop sampling the backdrop at
jittered thickness values, with the tap count derived by
`tap_count(anisotropic_blur, chromatic_aberration)` and clamped to 8. Exercised
at maximum on real hardware — `anisotropic-blur 1.0`, 8 taps, 100 px of smear —
it magnified the optics and left a checkerboard backdrop still reading as a
checkerboard. Taps along a single refraction axis are a smear, not a
convolution; no parameter value turns that into frost. Its strength is also
`thickness × anisotropic-blur`, and thickness independently drives refraction
offset and Beer-Lambert tint, so "more frost, same glass" is not expressible.

## Decision

The material sources its backdrop from the blurred xray texture rather than the
sharp one, under a new `glass` parameter. Blur and refraction stop competing
because there is one image: a frosted surface physically *is* a refraction of a
blurred backdrop.

Almost all of the machinery already exists, which makes this much smaller than
it first appears. `EffectBuffer` already owns a `BlurOptions`, an
`update_blur_options`, and a `render(frame, blur: bool)` that returns a blurred
texture cached beside the unblurred one. The xray buffers already receive blur
options from the global config every frame
(`src/niri.rs:4115-4123`), and the xray render element already selects blurred
or sharp per consumer (`src/render_helpers/xray.rs:310`). The material is
simply the one consumer that always asks for sharp:

```rust
// src/render_helpers/material.rs:576
let bg_texture = self.bg.borrow_mut().render(frame, false).ok();
let backdrop_texture = self.backdrop.borrow_mut().render(frame, false).ok();
```

The change is to make that `false` follow the material's own parameter — but not
only there.

### Preparation and drawing must agree

`EffectBuffer::prepare(renderer, blur)` allocates the blur textures only when
its flag is set (`effect_buffer.rs:148`), and both material paths currently
prepare with `false`:

```rust
// src/layout/tile.rs:1275 and again at 1460
bg.borrow_mut().prepare(ctx.renderer, false)
    && backdrop.borrow_mut().prepare(ctx.renderer, false)
```

Changing only the `draw` call would therefore not blur. `render(frame, true)`
would find no prepared blur, return `Err`, and the material's existing `.ok()`
fallback would silently drop the window to `plain_window_subdraw` — no glass at
all, with only a `warn!` to show for it. That failure is quiet and easy to miss,
so the design does not rely on remembering three call sites.

The effective flag is computed **once**, where the material is resolved for the
frame, and threaded to the preparation sites and onto the render element that
later draws. `prepare` and `render` read the same value by construction rather
than by convention. A reviewer should treat any future code path that passes a
literal to either call as a defect.

### The effective flag

```rust
fn backdrop_blur_enabled(glass: &ResolvedGlass, blur: &niri_config::Blur) -> bool {
    glass.backdrop_blur && !blur.off
}
```

`blur { off }` means what it says: it disables blur globally. `BlurOptions`
carries only `passes` and `offset` and drops `off` entirely, which is why the
background effect gates separately on `self.options.blur && !self.blur_config.off`
(`background_effect.rs:172`). The material must gate the same way, or `off`
would stop meaning "off" the moment a material opted in. Extracting it as one
pure function makes the rule unit-testable without a GPU.

### Strength comes from the global blur block

`backdrop-blur` is a boolean, not a strength. The blurred texture is cached per
buffer against one `BlurOptions`, applied from `config.blur` for every xray
buffer on the output. Per-material strength would mean caching a blurred
variant per distinct setting, or a buffer per setting, with the VRAM and
invalidation cost that implies.

Taking strength from the existing `blur { passes; offset }` block is the right
trade. It matches how niri already works — one global blur configuration,
per-consumer opt-in — and it restores the operator's original expectation, that
the `blur` block means something for these windows. It means it as the blur of
the image the glass refracts, rather than as a competing pass.

Per-material strength remains available later as a strictly additive change if
one global setting proves too coarse.

## Goals

- Make glass read as frosted, with softness that composes with refraction
  rather than fighting it.
- Reuse niri's existing, tuned blur implementation; add no second blur.
- Keep `background-effect` unchanged for windows without a material.
- Cost one blur per output per frame at most, shared by every material window.

## Non-goals

- Per-material blur strength.
- Noise and saturation, which are a separate follow-up (see below).
- Materials on layer surfaces. `MaterialRef` is window-rule-only
  (`niri-config/src/window_rule.rs:62`); extending it is its own design.
- Removing `anisotropic-blur`. It remains a useful optical smear; it is simply
  not frost.

## Parameter

| name | type | default | range |
|------|------|---------|-------|
| `backdrop-blur` | bool | `false` | — |

The default preserves the accepted v1 appearance exactly: every existing
material renders unchanged until the parameter is set.

Both textures follow the one flag. `niri_tex_bg` and `niri_tex_backdrop` serve
different composite paths — the latter carries `place-within-backdrop` content
seen in the overview — and they must agree, or entering the overview would
change the glass.

## Implementation surface

- `niri-config`: add `backdrop_blur: bool` to the glass block and its resolved
  form, with parsing and the existing unknown-field diagnostics. No range
  validation is needed for a boolean.
- `src/layout/tile.rs`: compute the effective flag where the material is
  resolved, and pass it to `prepare` at both 1275 and 1460 instead of `false`.
- `src/render_helpers/material.rs`: carry the same value onto the render element
  and pass it to both `EffectBuffer::render` calls at 576-577.
- `docs/materials/material-config.md`: add `backdrop-blur` to the parameter
  table at line 35 and note that its strength comes from the global `blur`
  block and that `blur { off }` overrides it.
- Everything else is untouched. Blur options, texture allocation, the blurred
  texture cache, `commit_counter` invalidation on option change, and damage
  tracking all already exist and already account for the blurred texture.

## Interactions

**Distortion** now warps an already-blurred image. That is the correct order for
frost — the surface irregularity displaces a diffuse backdrop — and it means
`distortion-scale` will read differently against a blurred source than the
values tuned against a sharp one. Existing values may want retuning; nothing
breaks.

**Cost.** The material always requests two independent buffers, `background` and
`backdrop` (`material.rs:576`), and each is held per render target —
`[Rc<RefCell<EffectBuffer>>; RenderTarget::COUNT]` with `COUNT == 3`, kept
separate so screencasting does not force constant rerendering (`xray.rs:27`).
Each carries its own blur cache.

The ceiling is therefore **two blur computations per changed source per render
target**, not one per output. In the common case only the primary target renders
and both sources change together, so it is two per frame; screencasting or a
second target multiplies that. It remains cheaper than `background-effect` blur,
which pays per window rather than per source, and it is shared by every material
window on the target. Marginal cost is zero only when both of those exact target
buffers are already blurred for another consumer — not merely when global blur
is in use somewhere.

**`xray false`.** The global window rule sets `xray false`, which governs the
background effect's xray path, not the material's sampling. The material reads
the xray buffers directly and is unaffected.

## Failure behavior

`EffectBuffer::render` already returns `Err` when blur is unavailable — no blur
program, wrong renderer context — and the material's existing `.ok()` plus the
`warn!` and `plain_window_subdraw` fallback covers it: the window renders
without the material rather than failing the frame. Requesting blur adds no new
failure mode, only an existing one on a second path.

## Verification

What the GPU-free suite can actually prove, and nothing beyond it:

- `niri-config` round-trips `backdrop-blur`, defaults it to false, and rejects a
  non-boolean with the standard unknown-value diagnostic;
- `backdrop_blur_enabled` returns false when the material opts out, false when
  `blur { off }` is set regardless of the material, and true only when the
  material opts in and global blur is on. This is the whole `off` interaction,
  as a pure function;
- a material resolved with `backdrop-blur true` yields a render element carrying
  the enabled flag, pinning that the value reaches the draw path.

What it cannot prove: which texture `draw()` ultimately samples. That decision
runs inside the concrete `GlesFrame` path, which the unit suite does not
execute, and the `.snap` files under `src/tests/snapshots` are layout snapshots
rather than pixel captures. The earlier draft claimed a byte-identical
default-preserves-v1 check here; that claim is withdrawn as unprovable at this
level.

Pixel equivalence belongs to the existing DRM acceptance harness
(`docs/materials/2026-08-27-v1-drm-acceptance-design.md`), which already
captures settled frames on real hardware and already gates on paired settled
frames being byte-identical. The default-preserves-v1 claim is verified there,
by running the existing scenarios unchanged against a build carrying this
parameter and requiring the captures to match the accepted run.

Manual, against the Prism debug backdrop, whose checkerboard makes softening
legible in a way a photograph cannot:

- `backdrop-blur true` visibly softens the checkerboard edges seen through
  glass, and raising `blur { passes }` deepens it;
- the window still renders with glass — the quiet `prepare`/`render` mismatch
  described above shows up precisely as glass disappearing, so this check is
  what catches it;
- `blur { off }` removes the softening while leaving the material otherwise
  intact;
- turning `backdrop-blur` off restores the current appearance exactly;
- the overview shows the same treatment, confirming both textures agree.

## Follow-up: noise and saturation

The operator's `blur { noise; saturation }` values reach windows through the
background effect, not the effect buffer — `background_effect.rs:202` passes
them alongside the blur options — so they do not follow the backdrop into the
material. Applying them after transmission in `material.frag` is a handful of
lines and gives the glass block one coherent control surface.

Two measurements taken while diagnosing this belong with that work, and look
like bugs independent of the material: `blur.offset` appeared to have no
observable effect at `passes 1`, and `saturation 0` did not reach grayscale.
Both want confirming against a build before anything is changed.

## Alternatives rejected

### Reimplement blur inside the material shader

The user's initial instinct, and the wrong shape. It duplicates a blur that
already exists and is already tuned, adds a second implementation to maintain,
and would run per window rather than once per output. The material does not need
its own blur; it needs to sample a blurred texture, which the effect buffer
already produces.

### Keep both passes and tune them

Not tunable in principle. The passes produce two images of the same wallpaper
and composite them; no values make a sharp refraction and a separate blur agree.

### Per-material blur strength now

Requires caching a blurred variant per distinct setting or a buffer per setting.
The cost is real and the need is unproven, and the design stays open to it.
