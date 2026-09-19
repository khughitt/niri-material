# Glass noise and saturation composition: design

**Status:** implemented as native commit `7c702e58` and Prism commit
`bcedf4cd`; automated and nested GLES acceptance passed. See
[`2026-09-02-material-noise-saturation-evidence.md`](../materials/2026-09-02-material-noise-saturation-evidence.md).
Superseded in part on 2026-09-05: the "no glass-specific parameters" decision
gave way to optional `glass { noise; saturation }` in
[`2026-09-05-material-glass-noise-saturation-params-design.md`](2026-09-05-material-glass-noise-saturation-params-design.md).
That parameter change preserved composition order and inheritance for omitted values.

**Task:** `material-cad932`

**Render-order update:** [Task 1](2026-09-12-material-render-order-design.md)
committed the superseding placement at `e33aa968`: saturation/noise now
transform the averaged backdrop before attenuation. Its pixel, signal, and
strict-cost acceptance passed. Parameters, inheritance and grain formulas are
preserved; historical evidence here was collected with the old order. Task 2
then committed the completed `within` placement at `e79b226b`.

## Context

Before native commit `7c702e58`, glass could select the globally blurred xray
source with `backdrop-blur`, but did not inherit the same global
`blur { noise; saturation }` postprocess. Those values reached windows only
through the separate background effect. Prism disables that superseded pass
for material windows with per-window `noise 0` and `saturation 1`, so its
glass previously received neither effect.

The two paths have different ownership:

- `background_effect.rs` resolves global defaults and per-window overrides,
  then `postprocess.frag` applies saturation followed by screen-space noise;
- `material.frag` samples the selected sharp or blurred xray textures, computes
  transmission and specular in linear light, converts the result to sRGB, and
  composites it with the window.

Keeping a second background-effect pass would repeat work and compose a second
image beneath the material. Processing shared xray textures would require
cache variants and would blur or refract the noise. The effects therefore
belong in the material shader after its optics.

## Current-build evidence

The suspected independent blur defects do not reproduce on the current tree.
A nested Weston GLES run used niri `v26.04-162-gd9c912d1`, built from source
commit `d9c912d1` (binary SHA-256
`8edc614ce17217ba55af3f89be0e83437c708bcbe32820df34f60a5b8ad8792d`).

- With one pass, changing `blur.offset` from 1 to 8 changed the text-free probe
  ROI by normalized RMSE `0.113418`; grayscale standard deviation fell from
  `0.239053` to `0.171272`. Offset is observable at one pass.
- With `saturation 0`, samples over red and green wallpaper were respectively
  `79/79/79` and `191/191/191`. Saturation zero reaches grayscale.

This task is therefore material integration, not a repair to the existing blur
or postprocess shaders.

## Decision

When glass has effective backdrop blur, it inherits `noise` and `saturation`
from the global `blur` block. No glass-specific parameters are added.

The existing gate remains authoritative:

```text
effective backdrop blur = glass.backdrop-blur && !blur.off
```

If the gate is false, glass uses neutral postprocess values:

```text
noise = 0
saturation = 1
```

If it is true, glass uses the global values exactly. Per-window
`background-effect { noise; saturation }` overrides remain independent and do
not alter the material.

This makes `blur { off }` disable global blur, noise, and saturation together,
matching the existing background-effect contract.

## Composition order

The shader applies the inherited effects to the completed glass color, not the
source textures or the composed window:

```text
sample selected backdrop
→ refraction / roughness / chromatic and anisotropic sampling
→ Beer-Lambert attenuation + specular
→ linear-to-sRGB conversion
→ saturation
→ screen-space noise
→ slab coverage
→ premultiplied window composition
```

Saturation uses the same luma weights and `mix(gray, color, saturation)` rule
as `postprocess.frag`. Noise uses `material.frag`'s existing `hash12` with the
distinct seed `gl_FragCoord.xy + vec2(47.0, 113.0)` and the same centered
amplitude formula as the postprocess shader. The offset avoids correlating
noise brightness with the first optics tap, whose jitter seed is exactly
`gl_FragCoord.xy`. No shared GLSL helper is introduced for these few lines.

The material mirrors `postprocess.frag`'s neutral branches: saturation math
runs only when saturation is not `1.0`, and noise math runs only when noise is
greater than `0.0`. The neutral path therefore executes no new color math;
default-off byte identity does not depend on floating-point identity
operations being exact.

Applying both effects before coverage makes them fade with the slab edge.
Applying them before window composition preserves the material contract:
opaque application pixels pass through untouched, and glass contributes only
through window transparency or outside the window.

## Render state and damage

The effective pair is private render-time state; it is not added to the public
`niri_config::Glass` or `ResolvedGlass` grammar. A render-side
`MaterialRenderConfig` wraps the `ResolvedMaterial` with the effective `noise`
and `saturation` values. `resolve_material` returns this wrapper, and both
`MaterialState::new` and `apply_resolved` accept it. `MaterialState` owns the
wrapper, keeping the pair beside the resolved glass configuration.

Material resolution computes the gate and pair once. `MaterialState` copies
the pair onto each `MaterialRenderElement`, which sends two scalar uniforms to
the shader. The same-name branch of `apply_resolved` compares the glass and
effective pair, replaces the stored wrapper, and bumps the existing material
commit counter when either changes, without rebuilding the offscreen buffer or
element identity. This makes an in-place global `blur { noise; saturation }`
edit damage material windows. Both normal and resize render paths consume the
same stored values.

`EffectBuffer`, sharp/blurred selection, roughness pyramids, and their damage
contracts do not change. The new state causes no texture allocation or cache
variant.

## Configuration and Prism

The native public grammar remains unchanged. `backdrop-blur` is still the sole
material opt-in; global `blur.noise` and `blur.saturation` remain the controls.

Prism already emits `backdrop-blur` and deliberately keeps its material-window
background effect inert. It gains no restored terminal controls, glass aliases,
or ownership of niri's global blur block. Its generated KDL must remain
byte-identical. Only the `glass.backdropBlur` description changes to say that
the global blur block supplies blur strength, saturation, and noise;
`test/glass-defs.test.js` gains an explicit assertion for that description.

## Compatibility

- Native and Prism defaults remain `backdrop-blur false`, so default rendering
  is pixel-identical.
- Existing opt-in configurations intentionally gain the global values. With
  niri defaults, those are `noise 0.02` and `saturation 1.5`.
- `blur { off }` produces the same sharp, neutral-postprocess result as material
  opt-out.
- Per-window background-effect behavior is unchanged.
- No compatibility alias or migration layer is added.

## Failure and cost

No new fallible resource operation is introduced. If the material program is
unavailable, the existing plain-window fallback remains authoritative. Global
values have already passed the existing blur configuration validation.

The enabled path adds two uniforms and scalar color math. It adds no texture
fetch, loop, render pass, allocation, or cache invalidation, so dedicated
performance profiling is out of scope.

## Implementation surface

- `src/layout/tile.rs`: return a `MaterialRenderConfig` containing the resolved
  material and effective inherited values from the existing backdrop-blur
  gate.
- `src/render_helpers/material.rs`: define the wrapper; make
  `MaterialState::new` and `apply_resolved` consume it; include the pair in the
  same-name update comparison; and store, damage-track, and bind the two
  values.
- `src/render_helpers/shaders/material.frag`: apply saturation then noise at
  the approved composition point, behind neutral branches and with the
  distinct noise seed.
- `docs/materials/material-config.md`: document inheritance and independence
  from per-window background-effect overrides.
- `docs/materials/2026-08-29-material-backdrop-blur-design.md`: replace the
  stale follow-up claim that offset and saturation looked independently broken
  with the current-build measurements and conclusion, then grep user-facing
  docs for the same stale claim.
- Prism `defs/glass.yaml` and `test/glass-defs.test.js`: clarify the existing
  `glass.backdropBlur` contract and assert its description without changing
  emitted KDL.

No config parser, `EffectBuffer`, blur shader, postprocess shader, or Prism
renderer change is required.

## Verification

GPU-free checks:

- effective policy returns `(0, 1)` for opt-out and global off, and exact
  global values only for effective backdrop blur;
- changing inherited noise or saturation advances the material commit without
  changing its identity;
- the full Rust workspace suite passes;
- Prism's full suite passes and its generated KDL remains unchanged.

Nested GLES captures:

- `saturation 0` makes a text-free glass ROI grayscale;
- nonzero noise measurably changes a uniform glass ROI;
- opaque window pixels remain byte-identical between neutral and active
  postprocess captures;
- `blur { off }` is byte-identical to material opt-out;
- the default-off fixture is byte-identical to its pre-change capture;
- the real material program compiles and no material fallback warning appears.

## Non-goals

- Glass-specific noise or saturation knobs.
- Per-window overrides for material postprocessing.
- Refactoring duplicate small GLSL color helpers.
- Changing blur offset, saturation math, or the background-effect path.
- Performance instrumentation for scalar-only shader work.
