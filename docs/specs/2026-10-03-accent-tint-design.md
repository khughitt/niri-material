# Accent tint of the glass attenuation color

**Status:** draft for spec review round 1.
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
stays dark and light glass stays light (§5).

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
  `are_transitions_ongoing` while the band is visible, and is cleared when
  done.
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
  encoded numbers, used unconverted in `main.frag` as the per-unit-distance
  transmittance in `pow(color, opticalDistance / attenuation-distance)`
  ([render pipeline](../materials/render-pipeline.md) §3, stage 4).
- Stage 0 returns opaque window pixels before attenuation. Stage 5 (ring,
  aurora) multiplies its light by `att ^ 0.2`, the same `att`.
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
already does. Because ring and aurora light passes the same `att ^ 0.2`
(§2), a tinted body also slightly tints that interior light. This is the same
physical rule that already applies to `attenuation-color`, and it needs no
separate handling.

## 5. Tint model

All arithmetic runs on the CPU, once per frame per material element, in the
same encoded number space that `attenuation-color` is written and consumed
in. Let:

- `c`: the glass's `attenuation-color` RGB, as uploaded today;
- `a`: the frame's accent, sRGB-encoded with the exact inverse of
  `color_linear` (so a settled `#rrggbb` accent recovers its own value);
- `Y(x) = 0.2126 x.r + 0.7152 x.g + 0.0722 x.b`: luminance of the
  coefficient. The shader multiplies linear light by these numbers, so this is
  the linear-light weighting.
- `w = accent-tint × presence`.

**Target.** The accent's hue at the glass's density:

1. If `Y(a) < 1e-6` (black), take the gray `t = (Y(c), Y(c), Y(c))`. Any
   other neutral accent reaches that same gray through step 2, so black is
   that gray's limit, not a special case.
2. Else `t = a × Y(c) / Y(a)`.
3. Gamut: if any channel of `t` exceeds 1, pull `t` toward the gray `Y(c)`
   by the largest factor `s ∈ [0, 1]` that keeps every channel ≤ 1, giving
   `t = Y(c) + s (t − Y(c))`. That gray is in gamut whenever `c` is, so `s`
   exists. Luminance is kept and saturation is given up.

**Mix.** `tinted = c + w (t − c)`, alpha unchanged from `c`. Luminance is
linear, and `Y(t) = Y(c)`, so `Y(tinted) = Y(c)` at every weight. The weight
moves hue and saturation only.

**Neutral path.** When `w ≤ 0` or the frame has no accent, the uploaded value
is `c`, bit for bit. This is a branch, not a mix that rounds to `c`.

The density kept is that of the attenuation coefficient. Transmittance is
`tinted ^ p` per channel with `p = optical distance / attenuation-distance`,
so at `p ≠ 1` the transmitted luminance follows `Y(tinted ^ p)`. That moves
with saturation. Worked case: the accepted look (`#0D1D1E`, face
`p ≈ 2.84`) with an orange accent (`#ff6600`) at weight 0.3 transmits about
13 % less luminance than the untinted glass; the plain mix rejected below
transmits about ten times more. The visual acceptance in §8 judges this.

**Rejected:** a plain `mix(c, a, w)`. Weight 1 would equal setting
`attenuation-color` to the accent, but on the owner's dark glass
(`#0D1D1E`) a vivid accent at weight 0.3 raises face transmitted luminance
about ten times. The cue would read as lightening, not hue. Also rejected: mixing in
linear light or Oklab. `attenuation-color` is authored and consumed as encoded
numbers. The luminance match already gives the perceptual property that
matters (density), so an extra color space would add conversion code for no
visible gain.

## 6. Animation and state

The tint has no clock of its own. It follows the frame's `accent` and
`presence`:

- **Accent appears:** presence rises 0 → 1 over the crossfade with the color
  held; `w` rises with it.
- **Accent replaced** (A → B): presence stays 1. The color interpolates in
  linear light, is encoded each frame, and its target is recomputed each frame
  (§5), so luminance holds at `Y(c)` throughout.
- **Accent removed:** presence falls 1 → 0 with A held; the tint fades out to
  exactly `c` (neutral path once `w` reaches 0).
- **Idle input, `motion "off"`/`"reduced"`:** the accent is static and kept,
  so the tint stays. It adds nothing that moves.
- **Level** does not scale the tint. The tint is identity, not urgency.
- **Response or material swap** (focus split, config reload): the weight
  steps to the new value without a crossfade, as every glass parameter does on
  a swap today. Interpolating across a swap is `material-5a5fff`'s scope.

## 7. Rendering site

- `glass_signal_inputs(frame, glass, response)` gains the response and a new
  `attenuation_color: [f32; 4]` field on `GlassSignalInputs`, computed by a pure
  function in a new `src/render_helpers/material/tint.rs`
  (`accent_tint(base, accent_linear, weight) -> [f32; 4]`).
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
- `Y(tinted) = Y(c)` within 1e-6 across weights {0.25, 0.5, 1}, several
  accents (saturated primaries, a mid gray, black, white), and the two bases.
- Gamut: every channel lies in [0, 1] for a light base (`#dfe8ff`) with a
  saturated accent, and saturation is reduced, not luminance.
- At weight 1 the target's chromaticity (`t / Y(t)`) equals the accent's,
  when no gamut pull applies.
- The encode function inverts `color_linear` within 1/4096 on all 256
  levels.
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
  attenuation color equals `accent_tint` of the interpolated frame. After
  removal completes, the render equals the never-tinted render.
- **Settled:** once the crossfade is done, two further renders keep the
  element's commit counter, `are_animations_ongoing` and
  `are_transitions_ongoing` are false, and the output arms no signal timer.

Visual acceptance, owner-judged (headless, no desktop): an `ACCENT_TINT_DUMP=<dir>`
switch on the render test writes PNGs, as `RING_LOOK_DUMP` does for the ring
look:

- the accepted terminal-glass look and the default glass, each with a warm
  and a cool accent, at weights 0, 0.25, 0.5 and 1;
- an A → B replacement and an A → none removal at crossfade fractions 0, ¼,
  ½, ¾ and 1.

The owner judges whether the hue reads as identity without changing the
glass's darkness, and picks the weight to recommend in the docs. The default
stays 0 whatever the pick.

## 9. Documentation and follow-ups

- `docs/materials/material-config.md`: a row in the signal-response table,
  and a paragraph beside the `accent`/`ring-accent` explanation stating the
  §4 independence and the §5 density rule.
- `docs/materials/render-pipeline.md`: stage 4's parameter column and §5
  table gain response `accent-tint`.
- Prism exposure of the field is separate scope: the implementation's
  closeout files it as an idea in the Prism project.
- This task's result commit notes the design finding on `material-6f45a0`
  so that idea can be scoped from this spec and its plan.
