# Glass noise type: design

**Status:** design approved 2026-09-06; not implemented.

**Task:** `material-6e7352`. Hub goal Prism `prism-d6b600`; Prism piece
`prism-51f23b`, designed in Prism
`docs/specs/2026-09-06-glass-noise-type-design.md`.

## Context

Glass composes `noise` after its optics
([`2026-09-05-material-glass-noise-saturation-params-design.md`](2026-09-05-material-glass-noise-saturation-params-design.md)):
in `material.frag`, after saturation and before coverage, one
`hash12`-derived uniform-random value per fragment is added equally to the
three sRGB-encoded channels. The grain is therefore already achromatic. What
it is not is fine: white noise carries as much energy at low spatial
frequencies as at high, which the eye reads as clumps, and a uniform
distribution spends a quarter of its samples at the extremes. Observed on
the desktop on 2026-09-05, it looks coarse even at Prism's default unfocused
amplitude of 0.02.

Prism wants to offer a finer grain, and a variant that grains brightness
without touching the backdrop's colour. Both need a native shader selector.

## Decision

The glass `noise` node gains an optional `type` property:

```kdl
noise 0.02 type=fine
```

| Value | Grain |
| --- | --- |
| `white` | today's grain, character for character; the default when `type` is omitted |
| `fine` | high-pass hash noise: the fragment's hash minus the mean of its eight neighbours' hashes |
| `lightness` | the `fine` value applied to Oklab lightness; chroma and hue hold exactly |

An unknown value is a parse error, like every other glass parameter.

`fine` gets both wanted properties from one operation. Subtracting the
neighbourhood mean removes the low-frequency energy, and a weighted sum of
nine independent uniforms is already bell-shaped. The result is scaled so
that its standard deviation equals `white`'s at the same amount, so an
amount means the same strength whichever type is active. The variance of
`centre - mean(8 neighbours)` is `(1/12)(1 + 1/8)`; the scale is
`sqrt(8/9)`. It stays achromatic and additive in sRGB-encoded space, exactly
where `white` adds.

`lightness` converts the post-saturation colour from sRGB-encoded to
linear to Oklab, adds the scaled `fine` value to L, and converts back. The
amount is applied in Oklab L units, which lie in 0–1 like sRGB, so the same
amount reads as a comparable strength.

The seed stays a function of `gl_FragCoord` alone, as today, so the grain is
static across frames. Nothing changes about when noise runs: still after
saturation, still inside coverage, still grained over the bevel band
(`material-f8b6e9` is a separate question and this work neither fixes nor
worsens it).

The property rides the `noise` node rather than standing as its own child
for the reason `distortion` carries `scale=`: a type does nothing while the
amount is zero, so it should not be writable without naming the amount it
belongs to.

## Configuration contract

`niri_config::material::Glass.noise` changes from
`Option<FloatOrInt<0, 1>>` to `Option<Noise>`:

```rust
/// `noise <amount> type=<type>`.
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property, str)]
    pub r#type: Option<NoiseType>,
}

pub enum NoiseType { White, Fine, Lightness }
```

`NoiseType` parses from the three lowercase names and defaults to `White`.

`ResolvedGlass` gains `noise_type: NoiseType`. The existing
`noise: Option<f64>` keeps its meaning and its inheritance rule; `resolve`
copies `amount` through as before and takes `r#type.unwrap_or_default()`.
The type has no inheritance: the global `blur` block has no notion of it,
so an omitted type is `White` regardless of `backdrop-blur` or `blur { off }`.

`MaterialRenderConfig` gains `noise_type: NoiseType`;
`resolve_material` in `src/layout/tile.rs` copies it from the resolved
glass. `MaterialState` includes it in the commit-counter comparison the way
it includes the amount, so a type change alone redraws the material.

## Shader

One new integer uniform, `mat_noise_type` (0 white, 1 fine, 2 lightness),
set beside `mat_noise`. The noise branch becomes:

```glsl
if (mat_noise > 0.0) {
    vec2 seed = gl_FragCoord.xy + vec2(47.0, 113.0);
    if (mat_noise_type == 0) {
        glassColor += (hash12(seed) - 0.5) * mat_noise;
    } else {
        float g = fineGrain(seed) * mat_noise;
        if (mat_noise_type == 1) {
            glassColor += vec3(g);
        } else {
            vec3 lab = linearToOklab(srgbToLinear(glassColor));
            lab.x += g;
            glassColor = linearToSrgb(oklabToLinear(lab));
        }
    }
}
```

`fineGrain` samples `hash12` at the seed and its eight unit-offset
neighbours, returns `(centre - mean) * sqrt(8.0 / 9.0)`, and is zero-mean.
The Oklab helpers are the standard matrices (Björn Ottosson, public
domain); `srgbToLinear` and `linearToSrgb` already exist in the fragment
shader. Nine hashes and two matrix conversions per fragment,
inside glass coverage only, and only when `mat_noise > 0`.

The `white` branch is the existing two lines unchanged, so an omitted or
`white` type renders byte-identical to today's build.

## Documentation updates when implementation lands

- `docs/materials/material-config.md`: a `noise` `type=` row in the
  parameter table, and one paragraph after the noise-and-saturation
  paragraph describing the three types.
- `docs/materials/README.md`: index entries for this spec and its evidence.
- This document's status header.

## Verification

GPU-free:

- config parse accepts `noise 0.02 type=white`, `type=fine` and
  `type=lightness`; rejects `type=blue`; resolves an omitted type to
  `White` and keeps the amount's `None`-means-inherit behaviour intact,
  including under `backdrop-blur false` and `blur { off }`;
- `resolve_material` carries the type into `MaterialRenderConfig`;
- changing only the type advances the material's commit counter;
- `just check` and `just test` pass.

Nested GLES smoke on the headless Weston unit, never the desktop session,
extending `glass-noise-saturation-smoke.sh` with a per-type matrix. All
captures use the same material at amount 0.5, compared against amount 0
and against each other on the text-free glass ROI:

- every type raises ROI variance against amount 0, so each renders;
- `fine` has less low-frequency energy than `white`: the variance of the
  ROI after a 4x4 box downsample, divided by its full-resolution variance,
  is lower for `fine`;
- `fine` has lighter tails than `white` at matched standard deviation: its
  peak absolute deviation from the amount-0 capture is smaller;
- `lightness` leaves per-pixel Oklab chroma unchanged within tolerance
  against amount 0, where `white` and `fine` shift it;
- an omitted type and an explicit `type=white` are byte-identical on the
  same binary.

Results are recorded as
`docs/materials/2026-09-06-material-glass-noise-type-evidence.md`.

Desktop acceptance, after the package is installed: compare the three types
by eye on an unfocused pane at Prism's default amount. This is the gate for
`lightness`: today's grain is added in sRGB-encoded space, which is already
roughly perceptual, so `lightness` differs from `fine` mainly by holding
colour fixed and the difference may be subtle. If it is indistinguishable,
the value is dropped from both specs before Prism exposes it.

## Alternatives rejected

### A blue-noise texture tile

A precomputed 64x64 blue-noise tile bound as an extra sampler has the best
spectrum, but it adds an embedded asset and a texture binding to the
material shader, and a 64 px period can show on large flat glass. At the
amplitudes in use the high-pass hash is fine enough.

### Two orthogonal selectors

Pattern (white, fine) and colour space (sRGB, Oklab lightness) are
independent axes, but four combinations is ceremony for one desktop
setting. Three named looks read better in a select. A fourth look is one
more value, not a second axis.

### A separate `noise-type` child node

Writable without an amount, and inert when written alone. The `distortion
<amount> scale=` idiom already answers this.

## Non-goals

- Chroma or hue grain, and blue-noise dithering.
- Masking noise out of the bevel band (`material-f8b6e9`).
- Animated grain.
- Changing the global `blur` block or the background effect.
