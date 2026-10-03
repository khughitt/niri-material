# Accent tint of the glass attenuation color

**Status:** accepted in spec review round 3 (round 1: revise, density moved to
the face transmittance; round 2: revise, tint chromaticity crossfades between
endpoints, interior-light and colored-backdrop effects restated).
**Task:** `material-3bdffc`; wakes `material-6f45a0` after design and plan review.
**Baseline:** `63538aef` (the task-start commit; it changes only the task).
**Brief:** [glass signal responses](../notes/2026-09-29-glass-signal-responses-brief.md#unanswered-questions).

## 1. Intent and boundary

A window that carries a signal accent (a project or session hue) should be
able to show that hue in the glass body itself, at rest, as an identity cue.
This is the original signals design's deferred "accent tint" response
([signals design](../materials/2026-09-02-material-signals-design.md) §11):
mix `attenuation-color` toward the accent.

The response is opt-in and defaults off. With it off, at weight zero, or with
no accent, every pixel is identical to today's. Opaque client pixels never
change. Once the accent and its crossfade have settled, the tint adds no redraw.

The owner chose the tint model on 2026-10-03: the tint moves hue and
saturation toward the accent and keeps the glass's own density, so dark glass
stays dark and light glass stays light (§5). Density is held exactly where
the glass is seen most, on the flat face, for a neutral backdrop. A colored
backdrop shifts brighter or darker with the tint, and the chamfer and the
interior ring and aurora light move with it too (§5, §4).

This task delivers the reviewed design and then an implementation plan for
separate review. It implements, captures and installs nothing, and changes
nothing in Prism. Fireflies, frost, progress, client desaturation and the
other briefed responses are out of scope.

## 2. What the code establishes

- `Response`/`ResolvedResponse` in `niri-config/src/material/mod.rs` hold the
  response vocabulary. `accent` is the selector `ring | none` (default `ring`).
  Response blocks inherit omitted fields from the material's `default` block,
  which inherits from `ResolvedResponse::default()`.
- `Tile::signal_for_frame` (`src/layout/tile.rs`) crossfades level, accent and
  presence over `animations.material-signal`. Accent colors are straight
  linear RGB (`color_linear`). Between two accents the color interpolates
  linearly. When either side has no accent, the other side's color is held for
  the whole fade and `presence` carries it. The crossfade is reported through
  `are_transitions_ongoing` while `signal_render_visible` holds, which is
  `slab_in_view` (the slab in view, not the band), and is cleared when done.
- The input-inactivity gate (`effective`) makes sustained motion static and
  leaves level, accent and impulses alone. A static accent survives idle input
  and `signal { motion "off" }`.
- `glass_signal_inputs` (`src/render_helpers/material/mod.rs`) is where glass
  reads a `SignalFrame` into per-frame glass values. It already rewrites
  `mat_chromatic_aberration` and `mat_distortion` from flash impulses.
  `GlassSignalFingerprint` quantizes those values into the element's
  `InputFingerprint`, so a change commits damage and a constant value does
  not.
- Every uniform is rebuilt per draw. `mat_attenuation_color` is uploaded from
  `ResolvedGlass::attenuation_color.to_array_unpremul()`: the config's
  encoded numbers, used unconverted in `main.frag` as linear-light
  multipliers, `pow(clamp(color, 0.001, 1), opticalDistance /
  attenuation-distance)`
  ([render pipeline](../materials/render-pipeline.md) §3, stage 4). The
  optical distance divides `thickness` by the structural normal's cosine,
  floored at 0.25: exactly `thickness` on the flat face, up to four times it
  on the chamfer.
- Stage 0 returns opaque window pixels before attenuation. Stage 5 (ring,
  aurora) multiplies its light by `att ^ 0.2`, the same `att`. The ring's
  light color is `mix(ring-color, accent, presence)` under `accent "ring"`,
  for the accent band and the focus filament alike, and `ring-color` under
  `accent "none"`.
- Signal-free tiles skip `solve` and upload `GlassSignalInputs::quiet(glass)`.

## 3. Configuration

A new response field, beside the existing `ring-*` levels:

```kdl
material "terminal-glass" {
    glass { attenuation-color "#0D1D1E"; attenuation-distance 11 }
    response "default" {
        accent "ring"
        accent-tint 0.4
    }
}
```

| Field | Values | Default |
| --- | --- | --- |
| `accent-tint` | 0–1 | 0 (off) |

- `FloatOrInt<0, 1>` in `Response`, `accent_tint: f64` in `ResolvedResponse`,
  inherited like every other response field. Out-of-range values fail to
  parse with the existing range error.
- There is no separate on/off selector. The weight is both the switch and the
  amount, so zero is off and every existing config is unchanged.
- Being a response field, it can differ per named response and per material
  definition (focused and unfocused materials under Prism's focus split).

## 4. Coexistence with the other accent uses

`accent-tint` is independent of `accent`, `ring-accent` and `attention`:

| Config | Ring band | Glass body |
| --- | --- | --- |
| `accent "ring"`, `accent-tint 0` | accent band (today) | unchanged |
| `accent "ring"`, `accent-tint 0.4` | accent band | tinted |
| `accent "none"`, `accent-tint 0.4` | no accent band | tinted |
| `ring-accent 0`, `accent-tint 0.4` | no accent outline; focus light still hued | tinted |

`accent "none"` keeps its documented meaning: it turns off the accent on the
band alone. The glass body has its own control, as the attention glint
already does.

Ring and aurora light passes the same attenuation at a fifth of the face's
path (`att ^ 0.2`, §2), so the tint changes that interior light. This is the
physical rule `attenuation-color` already follows, and the design does not
compensate for it. The direction depends on the light's color. Luminance of
the ring's light relative to untinted glass, on the accepted look with the
accent settled:

| Accent | Accent band (`accent "ring"`), w = 0.5 / 1 | `ring-color` band (`accent "none"`), w = 0.5 / 1 |
| --- | --- | --- |
| magenta `#ff00ff` | 1.50× / 1.69× | 1.02× / 0.47× |
| blue `#0066ff` | 1.08× / 1.10× | 0.98× / 0.86× |
| orange `#ff6600` | 1.37× / 1.46× | 1.02× / 0.87× |

Under `accent "ring"`, the band and the focus filament carry the accent, and
tinting the glass toward the accent's hue opens the channels that light lives
in, so the ring brightens, by half already at weight 0.5 with magenta. Under
`accent "none"`, the near-white `ring-color` loses the channels the tint
closes. The owner judges both on the §8 dumps, which frame the ring band.

## 5. Tint model

All arithmetic runs on the CPU in `f64`, once per frame per material
element. The model matches the glass's density where it is seen, the flat
face's transmittance, and puts the accent's own linear hue there, the same
hue the ring band draws. Let:

- `c`: the glass's `attenuation-color` RGB clamped to `[0.001, 1]`, as the
  shader clamps it;
- `p_f = thickness / attenuation-distance`: the face's exponent (surface
  cosine 1);
- `T_c = c ^ p_f` per channel: the face transmittance of the untinted glass;
- `a`: the frame's accent, straight linear RGB as `SignalFrame` carries it;
- `Y(x) = 0.2126 x.r + 0.7152 x.g + 0.0722 x.b`: linear-light luminance;
- `lo = 0.001 ^ p_f`: the lowest face transmittance a channel can reach
  through the shader's clamp;
- `w = accent-tint × presence`.

**Target.** The accent's hue at the face's density:

1. The tint chromaticity `k` of a settled accent `a` is `a / Y(a)`, or
   `(1, 1, 1)` when `Y(a) < 1e-6`. `Y(k) = 1` either way. This is a stated
   choice: a black accent has no hue, so it desaturates the glass toward a
   neutral gray at its own density, as any exactly neutral accent does. `k`
   depends only on the accent's chromaticity, not its brightness, so a
   near-black *colored* accent tints as strongly as a bright one of the same
   hue. During a crossfade, `k` is not derived from the interpolated accent;
   it interpolates between the endpoints' `k` (§6).
2. `T_t = k × Y(T_c)`.
3. Gamut: if any channel of `T_t` lies outside `[lo, 1]`, pull `T_t` toward
   the gray `Y(T_c)` by the largest factor `s ∈ [0, 1]` that brings every
   channel inside, `T_t = Y(T_c) + s (T_t − Y(T_c))`. The gray lies inside,
   since `lo ≤ min(T_c) ≤ Y(T_c) ≤ max(T_c) ≤ 1`, so `s` exists. Luminance
   is kept and saturation is given up.

**Mix and upload.** `T = T_c + w (T_t − T_c)` per channel, then upload
`T ^ (1 / p_f)` with alpha unchanged from the configured color. Luminance is
linear in `T` and `Y(T_t) = Y(T_c)`, so the face transmits exactly the
untinted luminance of a neutral backdrop at every weight. Every channel of `T` lies in `[lo, 1]`,
so the uploaded coefficient lies in `[0.001, 1]` and the shader's clamp
changes nothing. At weight 1 without a gamut pull, the face's transmitted
chromaticity is the accent's linear chromaticity.

**Neutral path.** The uploaded value is the configured
`attenuation_color.to_array_unpremul()`, bit for bit, unclamped, when
`w ≤ 0`, when the frame has no accent, when `p_f = 0` (zero thickness: no
attenuation to tint), or when `Y(T_c)` underflows to 0 (a face that
transmits nothing). This is a branch, not a computation that rounds back.

**Colored backdrops.** The shader renders `sampled × T` per channel, so
`Y(T)` fixes the transmitted luminance only when the backdrop is neutral.
Through a saturated backdrop, channels the tint opens brighten and the others
darken. On the accepted look at weight 1, an orange tint passes a pure red
backdrop 25.8× brighter, a pure green one at 0.35× and a pure blue one at
almost nothing; magenta passes red 27.9× and blue 2.6×. The model keeps this
simple behavior: compensating would mean reading the backdrop per pixel and
changing the shader, which makes the tint a different response. The density
promise is stated for neutral and near-neutral backdrops, and the §8 dumps
include saturated ones.

**Where density still moves.** Only the face is matched. The chamfer's path
runs up to `4 p_f` and the ring and aurora light's `0.2 p_f`, and at
exponents other than `p_f` the tinted coefficient no longer transmits the
untinted luminance. On the accepted look the chamfer at weight 1 transmits
13× to 50× its untinted luminance, but from about `2e-11` to under `1e-9`,
still black. The interior light is §4's change.

**Light glass has little room for hue.** Keeping density is the owner's
choice, and near-white glass has almost no saturation left at its own
luminance. On the default glass (`#dfe8ff`, `p_f = 1/3`, face luminance
0.97) the gamut pull leaves a magenta accent about 1 % of its saturation, so
the tint is nearly invisible there. The §8 dumps show it, and the docs say
so.

**Rejected:**

- *A plain `mix(attenuation-color, accent, w)`.* Weight 1 would equal setting
  `attenuation-color` to the accent, but on the owner's dark glass a vivid
  accent at weight 0.3 raises face luminance about ten times. The cue would
  read as lightening, not hue.
- *Matching luminance on the coefficient* (round 1 of this spec). It holds
  density only at `p = 1`. At the accepted look's `p_f ≈ 2.84` magenta at
  weight 1 brightened the face 8.7×. Treating encoded accent numbers as
  transmittance also shifted the hue (`#ff6600` showed as about `#ff4d00`),
  away from the band's color.
- *A luminance knee near black* (fading `k` toward gray as `Y(a)` falls)
  instead of interpolating `k` between endpoints. A black → colored fade
  would still cross the knee in the first few percent of its fraction, and
  near-black colored accents would lose the hue that step 1 gives them.
- *Oklab or another perceptual space.* Face luminance is the property that
  matters, it is linear, and matching it exactly needs no further
  conversion.

## 6. Animation and state

The tint has no clock of its own. It follows the frame's `accent` and
`presence`:

- **Accent appears:** presence rises 0 → 1 over the crossfade with the color
  held; `w` rises with it.
- **Accent replaced** (A → B): presence stays 1. The accent the ring draws
  still interpolates in linear light. The tint chromaticity interpolates
  separately, `k = k(A) + f (k(B) − k(A))` with the crossfade's fraction `f`,
  and the target is recomputed each frame from it (§5), so face luminance
  holds at `Y(T_c)` throughout. Interpolating `k` rather than normalizing the
  interpolated accent is what makes black ↔ colored continuous: from black,
  the interpolated accent is `f × B`, and normalizing it would jump to `k(B)`
  on the first frame (and, in reverse, hold `k(A)` until the last).
- **Interrupted crossfade:** a new crossfade starts its `k` from the running
  one's current `k`, as level, accent and presence already do in
  `crossfade_origin`.
- **Accent removed:** presence falls 1 → 0 with A and `k(A)` held; the tint
  fades out to exactly the configured color (neutral path once `w` reaches
  0).
- **Idle input, `motion "off"`/`"reduced"`:** the accent is static and kept,
  so the tint stays. It adds nothing that moves.
- **Level** does not scale the tint. The tint is identity, not urgency.
- **Response or material swap** (focus split, config reload): the weight
  steps to the new value without a crossfade, as every glass parameter does on
  a swap today. Interpolating across a swap is `material-5a5fff`'s scope.

## 7. Rendering site

- `accent_chroma(a) -> [f32; 3]` in `src/render_helpers/signal.rs` computes
  `k`. `SignalCrossfade` holds `k` for both endpoints next to the accents,
  `SignalCrossfade::current` and `crossfade_origin` return it, and
  `FrameInputs` and `SignalFrame` carry it as `tint_chroma: Option<[f32; 3]>`
  (`None` exactly when `accent` is `None`). The accent itself and every
  existing consumer of it are unchanged.
- `glass_signal_inputs(frame, glass, response)` gains the response and a new
  `attenuation_color: [f32; 4]` field on `GlassSignalInputs`, computed by a pure
  function in a new `src/render_helpers/material/tint.rs`
  (`accent_tint(base, thickness, attenuation_distance, tint_chroma,
  weight) -> [f32; 4]`).
  `GlassSignalInputs::quiet` sets it to the glass's own value.
- The element uploads `mat_attenuation_color` from `GlassSignalInputs`
  instead of from `ResolvedGlass` directly.
- `GlassSignalFingerprint` gains the quantized attenuation color (1/1024, as
  its other fields), so a reload that changes only `accent-tint` commits
  damage.
- No shader change. `main.frag`, the uniform list and the program are
  untouched. The pass order in `render-pipeline.md` is unchanged; only stage
  4's parameter column gains the response field.

## 8. Verification

Unit tests (`tint.rs`, `material/mod.rs`):

- Neutral path: at weight 0, at presence 0, and with no accent, the result is
  bitwise `glass.attenuation_color.to_array_unpremul()`, for the default glass
  and the accepted look.
- Face density: `Y(x ^ p_f) = Y(c ^ p_f)` within a relative 1e-5, where
  `x` is the uploaded coefficient clamped as the shader clamps it, across
  weights {0.25, 0.5, 1}; accents `#ff6600`, `#0066ff`, `#ff00ff`, a
  near-black colored accent, mid gray, black and white; and the default
  glass, the accepted look, and a mid-density glass with `p_f < 1`.
- At weight 1 the face transmittance's chromaticity (`x ^ p_f / Σ`) equals
  the accent's linear chromaticity within 1e-5, when no gamut pull applies.
- `accent_chroma`: `Y(k) = 1`; a near-black colored accent has the same `k`
  as its bright counterpart; black and mid gray give `(1, 1, 1)`.
- Black ↔ colored continuity: for black → `#ff6600` and the reverse,
  `SignalCrossfade::current`'s `k` equals the endpoint interpolation at
  fractions 0, 1e-5, ¼, ½, ¾ and 1. The uploaded attenuation color at
  fraction 1e-5 differs from the fraction-0 value by less than 1e-2 per
  channel, and likewise at 1 − 1e-5 against fraction 1, where round 2's
  model jumped to the endpoint hue. The same holds for an interrupted
  crossfade restarted mid-way.
- Gamut: every uploaded channel lies in `[0.001, 1]`. On the default glass
  with a saturated accent, saturation is reduced and luminance is not.
- The neutral cases of §5: zero thickness and an underflowing face return the
  configured color bitwise.
- The fingerprint: with the same frame and accent, `GlassSignalFingerprint`
  differs between `accent-tint 0.4` and `0.6`, so a reload that changes only
  the weight commits damage.
- `ResolvedResponse` inheritance and the 0–1 range.

Headless render tests (`src/tests/`, frozen clock, the `ring_pair` render
helpers):

- **Default and zero-weight equivalence:** with an accent signal, a response
  without `accent-tint` and one with `accent-tint 0` render identically
  (`diff == (0, 0)`). `accent-tint 1` with no signal renders identically to
  `accent-tint 0` with no signal.
- **Opaque pixels unchanged:** a window with an opaque region renders those
  pixels identically at `accent-tint 1` with an accent and at weight 0.
- **Visible tint:** `accent-tint 1` with an accent differs from weight 0 in
  the slab band and through translucent content.
- **Replacement and removal:** at crossfade fractions 0, ½ and 1 the uploaded
  attenuation color equals `accent_tint` of the frame's interpolated
  `tint_chroma`, for orange → blue and black ↔ orange. After removal
  completes, the render equals the never-tinted render.
- **Settled:** once the crossfade is done, two further renders keep the
  element's commit counter, `are_animations_ongoing` and
  `are_transitions_ongoing` are false, and the output arms no signal timer.

Visual acceptance, owner-judged (headless, no desktop): an `ACCENT_TINT_DUMP=<dir>`
switch on the render test writes PNGs, as `RING_LOOK_DUMP` does for the ring
look:

- the accepted terminal-glass look and the default glass, each with orange
  (`#ff6600`), blue (`#0066ff`) and magenta (`#ff00ff`) accents, at weights
  0, 0.25, 0.5 and 1, framed so the ring band and chamfer are in view
  alongside the face, under both `accent "ring"` and `accent "none"`;
- the same weights over a neutral gray, a saturated red and a saturated blue
  backdrop (`layout { background-color }`, which the glass refracts);
- an orange → blue replacement, black → orange and orange → black
  replacements, and an orange → none removal, at crossfade fractions 0, ¼, ½,
  ¾ and 1.

The owner judges whether the hue reads as identity without changing the
glass's darkness over a neutral backdrop; whether the brighter accent band
and the dimmer `ring-color` band (§4) are acceptable; how far saturated
backdrops shift (§5); and how little shows on light glass (§5). They pick
the weight to recommend in the docs. The default
stays 0 whatever the pick.

## 9. Documentation and follow-ups

- `docs/materials/material-config.md`: a row in the signal-response table,
  and a paragraph beside the `accent`/`ring-accent` explanation stating the
  §4 independence, the §5 face-density rule for neutral backdrops, the
  interior-light change, colored backdrops, and the near-invisible tint on
  light glass.
- `docs/materials/render-pipeline.md`: stage 4's parameter column and §5
  table gain response `accent-tint`.
- Prism exposure of the field is separate scope: the implementation's
  closeout files it as an idea in the Prism project.
- This task's result commit notes the design finding on `material-6f45a0`
  so that idea can be scoped from this spec and its plan.
