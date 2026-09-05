# Ring of light focus response: design

**Status:** approved design, not started. Supersedes the static gradient
focus ring for material windows; the spike that chose it is
[`2026-09-05-focus-ring-light-spike.md`](../materials/2026-09-05-focus-ring-light-spike.md).

**Task:** `material-26dd8a`, piece of `material-d1f471`.

## Context

The focus ring is a 4 px gradient border element that knows nothing about
the material under it. On the dark focus glass the daily driver uses it is
below one gray level and does not register. The spike prototyped six
lighting treatments inside the material shader and ranked an embedded,
refracted ring of light first: a filament inside the slab, bent by the
front-face normal, confined to the bevel so text is untouched, drifting
slowly and breathing with the jelly residuals.

Three pieces of machinery already exist and this design reuses them rather
than adding parallels:

- The signal accent ring (`accent "ring"`) is an emissive band in the bevel
  at `ring-inset` and `ring-width`, lit by level and breath, on any window.
- The tile crossfades signal level and accent through the `material-signal`
  animation entry, and reports the crossfade as an ongoing transition.
- Sustained signal motion wakes through one bucket timer per output, with
  every oscillator exposing `next_boundary` from the pure solver, so a
  quiet window costs zero redraws and zero wakeups.

Two constraints came out of the spike by measurement. Quantizing the time
tick in the damage fingerprint does not throttle anything while a tile
reports itself as transitioning, because the offscreen re-renders every
frame; drift must be timer-driven. And Prism's glass index of 1.02 bends
nothing visible; the light path needs an exaggerated index.

## Decisions

Settled with the owner during design:

- **Focus change fades the light only.** Filament intensity crossfades
  0 to 1 over the `material-signal` animation. The `is-active` material
  swap the daily driver uses stays a hard cut; interpolating glass
  parameters across it remains `material-5a5fff`.
- **One filament, accent tints it.** The focus light and the signal accent
  ring are the same band. Focus sets brightness and drift; a live accent
  sets color. An unfocused accent window looks as it does today, except
  refracted. A focused window with no accent shows the base color with
  drift. Both together show the drifting filament in the accent color.
- **The gradient ring is turned off explicitly.** No automatic suppression.
  The docs tell material users to set `focus-ring { off }`, globally or per
  window rule; non-material windows keep whatever ring the config gives.

## Inputs and motion

**Active state.** `Tile::update_render_elements` already receives
`is_active`; the tile records it. A change starts a focus crossfade on the
`material-signal` animation entry with the configured duration and curve,
so `animations { off }` completes it instantly and the global slowdown
applies. While the crossfade runs it sets the transitions term, exactly
like the level crossfade, and clears it when done.

**Drift.** A new sustained oscillator in `src/render_helpers/signal.rs`: a
travelling brightness around the perimeter, product of two slow sines of
the perimeter angle and phase, as in the spike. Period 10 s. Phase is the
unadjusted clock plus the per-window jelly seed so panes do not move in
lockstep; boundaries are aligned to the unadjusted clock alone, so every
drifting window on an output shares them and ten windows cost the wakeups
of one. `ring-drift-hz` sets the bucket rate, so the oscillator quantizes
its phase to `1 / hz` and exposes `next_boundary(now)`. Rate 0 pins the
phase to a constant and reports no boundary.

**Motion policy.** `signal { motion "reduced" }` halves the drift rate;
`"off"` pins it. `animations { off }` pins it. A pinned drift with a
finished crossfade is a static refracted filament that costs nothing per
frame; that is the degrade path.

**Jelly breath.** Filament brightness scales by `1 + 2 * activity`. Activity
already produces per-frame fingerprints while nonzero, so no new
scheduling.

**Accent presence.** Today the tile crossfades the accent RGB toward black
when a signal expires, and `SignalUniforms::from_frame` sets the accent
alpha to 0 or 1 from bare presence. Mixing the filament color by that alpha
would snap. `SignalCrossfade` therefore gains a `presence` pair (0 or 1 at
each end) crossfaded on the same animation as level and accent, and the
uniform's accent alpha carries the crossfaded value. An interrupted fade
starts from the current interpolated presence, the way level and accent
already start from `SignalCrossfade::current`. `SignalFingerprint`
quantizes presence to 1/256 next to the accent channels.

**What reaches the shader.** `SignalUniforms` gains `focus` (0 to 1,
crossfaded), `drift` (bucketed phase, radians), and the filament's base
color; `accent.w` becomes the crossfaded presence; `ring` keeps inset and
width; `response` gains the focus selector. `ResolvedGlass` gains
`light_ior`. `SignalFingerprint` quantizes `focus` to 1/256 and `drift` to
its bucket. The drift phase is pinned to a constant whenever the rate is 0
or motion is off, so a quiet unfocused window and a focused static window
both fingerprint to constants.

## Rendering

The accent ring term in `material.frag` is replaced by the embedded
filament. For the fragment at element position `p` with perturbed normal
`n`, the ray refracts through the light-path index `1 + (ior - 1) *
light_ior` and lands at 60% of the slab thickness; the band is evaluated
there per channel with the material's chromatic aberration scaled by 0.1,
as `exp(-2 d^2)` over `(depth - inset) / width` plus a halo of 0.3 at
`inset + 2` with width 9. The result is attenuated by the Beer-Lambert
term raised to 0.2 so dense dark glass does not swallow it. Those three
constants (depth fraction, aberration scale, attenuation exponent) are the
spike's calibrated values and stay constants.

**Bevel confinement.** The Gaussian tails, the halo, and the refracted
displacement are not bounded by `ring-inset + ring-width <= bevel`, so the
band is masked to the rendered bevel: `slabSurface` publishes the inner
face distance it already computes (`di`, after the tiny-slab chamfer clamp
and the jelly shear and resize of the inner face), the mask is
`smoothstep(-1, 0, di)` evaluated at the refracted position, and the band
is multiplied by it. Nothing lights the face, the mask follows the
deformed inner edge, and on a window too small to carry a chamfer the mask
is zero and the filament vanishes with the bevel. The existing inset plus
width rule stays as the guarantee that the filament's center lies in the
bevel.

Brightness and color, in linear light, with the selectors made explicit:

```text
show_accent  = response.accent == ring  && presence > 0
show_focus   = response.focus  == ring-light
pulse        = response.attention == ring-pulse ? breath : 0   // today's gate
accent_glow  = show_accent ? (0.15 + 0.35 * level) * (1 + pulse * level) : 0
focus_glow   = show_focus  ? focus * 0.7 * (0.55 + 0.45 * travel(drift, angle)) : 0
glow         = (accent_glow * presence + focus_glow) * (1 + 2 * jelly_activity)
color        = show_accent ? mix(ring_color, accent_rgb, presence) : ring_color
emissive    += color * glow * band * mask
```

`presence` is the crossfaded accent presence from the inputs section, so
color and accent glow move smoothly when a signal arrives, expires, or is
interrupted mid-fade. `travel` is the spike's two-wave product over the
perimeter angle from the slab center. Breath modulates the filament only
under `attention "ring-pulse"`, as it does today; `rim-orbit` moves the
Fresnel glint and leaves the filament alone. When `glow` is zero the band
and mask are not evaluated. The four selector combinations of `accent`
and `focus` are independent: `accent "none"` with `focus "ring-light"`
shows the focus filament in the base color and never tints it;
`accent "ring"` with `focus "none"` is today's accent ring, refracted.

## Configuration and Prism

In the material `response` block, next to `accent` and `attention`:

```kdl
response "default" {
    accent        "ring"          // unchanged: whether signals light the filament
    focus         "ring-light"    // "ring-light" | "none"; default "ring-light"
    ring-inset    5               // default moves from 6 to 5
    ring-width    2.6             // default moves from 2 to 2.6
    ring-color    "#ccccff"       // filament base color; alpha ignored
    ring-drift-hz 15              // 0..30; 0 pins the drift
}
```

`ring-inset + ring-width <= bevel` is validated as before. `ring-width`
must be positive: the new band law divides by it, and 0 becomes the
validation error `ring-width must be positive` rather than a degenerate
band. Named responses inherit omitted fields from `default` as before.

In the `glass` block: `light-ior` in 1 to 12, default 6, a multiplier on
the bend applied to the filament's light path only, never to the
background taps. Prism's generated block emits it next to the other glass
parameters so the palette piece can tune it; no other Prism change.

`material-config.md` documents the fields, the shared-filament rule, and
the instruction to set `focus-ring { off }` for material windows.

## Redraw and damage contract

| State | Wakeups | Damage |
| --- | --- | --- |
| Unfocused, no accent | none | none (unchanged) |
| Unfocused, accent, static motion | none | none (unchanged) |
| Focused, drift 15 | 15 per second per output, shared boundaries | once per bucket |
| Focused, drift 0, or motion off, or animations off | none | none once the crossfade lands |
| Focus crossfade running | refresh rate for its duration | per frame |

## Implementation surface

| Piece | Where |
| --- | --- |
| `focus` and drift fields, parsing, defaults, ranges, `light-ior` | `niri-config/src/material.rs`, `ResolvedResponse`, `ResolvedGlass` |
| Drift oscillator, `next_boundary`, motion policy | `src/render_helpers/signal.rs` |
| Active flag, focus crossfade, transitions term, bucket reporting | `src/layout/tile.rs` |
| `SignalUniforms`, `SignalFingerprint`, new uniforms | `src/render_helpers/material.rs`, `shaders/mod.rs` |
| Filament in place of the accent band | `src/render_helpers/shaders/material.frag` |
| Docs | `docs/materials/material-config.md`, default config comment |
| Prism | `light-ior` in the generated glass block |

## Verification

- Solver tests: drift value and `next_boundary` at fixed clock values;
  pinned to a constant at rate 0, under `motion "off"`, and with animations
  off; halved rate under `"reduced"`; shared boundaries across seeds.
- Config tests: new fields, defaults, ranges, the inset plus width rule
  with the new defaults, `ring-width 0` rejected and a fractional width
  accepted, `light-ior` range, inheritance in named responses.
- Uniform tests: presence crossfades 0 to 1 and back on the animation,
  an interrupted fade starts from the current value, and the accent alpha
  uniform equals it; the four `accent` and `focus` selector combinations
  produce the expected show flags and breath gate.
- A nested GLES capture with a small window below the chamfer clamp and one
  with a full-flex jelly resize confirms no filament light on the face.
- Fingerprint tests: a focused drifting window changes once per bucket and
  never within one; a focused static window is constant after the
  crossfade; the quiet fingerprint tests extend to focus 0 and 1.
- Shader compile check covers the GLSL; the spike's validator step stays in
  the harness.
- Smoke: the material-signals smoke harness gains a focused-window case
  measuring wakeups per second at drift 15, drift 0, and animations off,
  judged by the criteria it already applies to Breathe. The focus ring
  light harness reruns against the real binary with `focus "ring-light"`
  replacing the probe and its corner crop is compared to the spike's ring
  case. The DRM acceptance gates run once with a drifting focused window.

## Non-goals

- Interpolating glass parameters across the `is-active` swap
  (`material-5a5fff`).
- Canopy light as an attention response.
- Automatic suppression of the gradient focus ring on material windows.
- Exposing the depth fraction, aberration scale, or attenuation exponent.
