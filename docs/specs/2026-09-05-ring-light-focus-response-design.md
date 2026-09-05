# Ring of light focus response: design

**Status:** implemented on design/ring-light; smoke evidence in
[2026-09-05-ring-light-focus-smoke.md](../materials/2026-09-05-ring-light-focus-smoke.md);
DRM acceptance awaiting the operator run described there. Supersedes the
static gradient focus ring for material windows; the spike that chose it is
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
of one. `ring-drift-hz` sets the bucket rate: the 10 s period is divided into
`round(hz * 10)` equal buckets anchored to period starts in integer
nanoseconds, so boundaries repeat exactly over any uptime and every
drifting window on an output shares them; the oscillator exposes
`next_boundary(now)` as the first nanosecond of the next bucket. Rate 0
pins the phase to 0 and reports no boundary. A configured rate is 0 or at
least 1 Hz; values between are a config error, since they have no sensible
bucket.

**Motion policy.** `signal { motion "reduced" }` halves the drift rate;
`"off"` pins it. `animations { off }` pins it. A pinned drift with a
finished crossfade is a static refracted filament that costs nothing per
frame; that is the degrade path.

**Jelly breath.** Filament brightness scales by `1 + 2 * activity`. Activity
already produces per-frame fingerprints while nonzero, so no new
scheduling.

**Accent presence and color.** Today `SignalCrossfade::current` scales the
accent RGB by the fade (toward black on expiry, from black on arrival) and
`SignalUniforms::from_frame` sets the accent alpha to 0 or 1 from bare
presence. Weighting that RGB by a fading presence would count the fade
twice: halfway through arrival the filament would be half base and a
quarter accent. The crossfade therefore carries the two separately:

- `presence` fades 0 to 1 on arrival and 1 to 0 on expiry, on the
  `material-signal` animation.
- `accent_rgb` is straight, unpremultiplied color. On arrival it is the
  arriving color for the whole fade. On expiry it holds the last live color
  for the whole fade. When one live accent replaces another, presence stays
  1 and the RGB interpolates straight between the two colors in linear
  light.

An interrupted fade starts from the current interpolated presence and the
current RGB, the way level already starts from `SignalCrossfade::current`.
The uniform's accent alpha carries presence and its RGB carries the
straight color; every existing consumer of the accent uniform (rim-orbit
tint, impulse colors) multiplies by alpha where it needs the faded value.
`SignalFingerprint` quantizes presence to 1/256 next to the accent
channels.

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
light_ior` and lands at 60% of the slab thickness. The shared part of that
in-plane shift is capped at half `ring-inset`, so however dense the glass
the core stays inside the bevel: at the stock default glass the uncapped
shift is about 6.9 px, more than the 5 px inset, and would carry the core
out past the silhouette. The cap saturates on any glass much denser than
the spike's calibration index of 1.02 — on the stock default glass and on
Prism's `ior 1.24` it is already reached at `light-ior 1` — so there
`light-ior` no longer positions the core and only widens the chromatic
split. The per-channel aberration offsets ride on top of the capped
shift, so dense glass keeps its chromatic split. The band is evaluated at
those landing points per channel with the material's chromatic aberration
scaled by 0.1, as `exp(-2 d^2)` over `(depth - inset) / width` plus a halo
of 0.3 at `inset + 2` with width 9. The result is attenuated by the
Beer-Lambert term raised to 0.2 so dense dark glass does not swallow it.
Those three constants (depth fraction, aberration scale, attenuation
exponent) are the spike's calibrated values and stay constants.

**Bevel confinement.** The refracted displacement is bounded by the cap
above, but the Gaussian tails and the halo are not bounded by `ring-inset
+ ring-width <= bevel`, so the band is masked to the rendered bevel at the
displayed fragment. `slabSurface` publishes the clamped rendered chamfer
and the inner face distance it already computes for the fragment (`di`,
after the tiny-slab chamfer clamp and the jelly shear and resize of the
inner face). The mask is `chamfer > 0 ? smoothstep(0, 1, di) : 0`,
evaluated at the original fragment position, never at the refracted one: it
is exactly 0 on the face and everywhere `di <= 0`, ramps to 1 over the
first pixel of the bevel, and is identically 0 when the rendered chamfer is
0, including the silhouette. Only the filament sampling is refracted; the
mask is not, so distortion and jelly ripple can move light within the bevel
but never onto the face. The mask follows the deformed inner edge, and on a
window too small to carry a chamfer the filament vanishes with the bevel.
The existing inset plus width rule stays as the guarantee that the
filament's center lies in the bevel.

Brightness and color, in linear light, with the selectors made explicit:

```text
show_accent  = response.accent == ring  && presence > 0
show_focus   = response.focus  == ring-light
pulse        = response.attention == ring-pulse ? breath : 0   // today's gate
accent_glow  = show_accent ? (0.15 + 0.35 * level) * (1 + pulse * level) : 0
focus_glow   = show_focus  ? focus * 0.7 * (0.55 + 0.45 * travel(drift, angle)) : 0
glow         = (accent_glow * presence + focus_glow) * (1 + 2 * jelly_activity)
color        = show_accent ? mix(ring_color, accent_rgb, presence) : ring_color
mask         = chamfer > 0 ? smoothstep(0, 1, di_at_fragment) : 0
emissive    += color * glow * band * mask
```

`presence` and `accent_rgb` are the crossfaded presence and the straight
color from the inputs section, so halfway through arrival the color is half
base and half accent, and color and accent glow move smoothly when a signal
arrives, expires, changes, or is interrupted mid-fade. `travel(drift, angle)` is
`sin(2 * angle + drift) * sin(3 * angle - 2 * drift)`, the spike's two-wave
product over the perimeter angle from the slab center with integer drift
multiples so the 2π wrap is continuous; the solver carries the same function
for tests. Breath modulates the filament only
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
    ring-drift-hz 15              // 0, or 1..30 (tenths of a hertz); 0 pins the drift
}
```

`ring-inset + ring-width <= bevel` is validated as before. `ring-width`
must be positive: the new band law divides by it, and 0 becomes the
validation error `ring-width must be positive` rather than a degenerate
band. `ring-drift-hz` strictly between 0 and 1 is the validation error
`ring-drift-hz must be 0 or at least 1`. Named responses inherit omitted
fields from `default` as before.

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
| Docs | `docs/materials/material-config.md` (none in the default config, which has no material block) |
| Prism | `light-ior` in the generated glass block |

## Verification

- Solver tests: drift value and `next_boundary` at fixed clock values;
  pinned to a constant at rate 0, under `motion "off"`, and with animations
  off; halved rate under `"reduced"`; shared boundaries across seeds.
- Config tests: new fields, defaults, ranges, the inset plus width rule
  with the new defaults, `ring-width 0` rejected and a fractional width
  accepted, `light-ior` range, inheritance in named responses.
- Uniform tests: presence crossfades 0 to 1 and back on the animation,
  RGB holds the arriving or last color through a fade and interpolates
  straight between two live colors, an interrupted fade starts from the
  current presence and color, the mixed filament color at the midpoint of
  arrival is half base and half accent, and the accent alpha uniform equals
  presence; the four `accent` and `focus` selector combinations produce the
  expected show flags and breath gate.
- Shader-side mask tests through the fingerprint-free path are not
  possible, so the nested captures are the check.
- Nested GLES captures (`docs/materials/scripts/focus-ring-light.sh`,
  recorded in the smoke evidence) render each scene twice, once with the
  filament and once with `focus "none"; accent "none"`, so only the
  filament differs. Gated: at rest no face pixel changes (face absolute
  error 0). Measured from the same frames: the whole-frame filament
  difference is bounded by the slab rectangle; the accent fade measures a red share of 0.498 at half fade,
  the straight mix; and the four `accent` and `focus` selector
  combinations are independent in both face error and filament color.
- `resize-flex` — face confinement under a running jelly flex, and the
  breath ratio — is recorded as informational and **not verified by
  measurement**: the two lockstep hosts skew mid-resize, so client glyph
  differences swamp whatever face light there might be. A deterministic
  mid-flex probe is filed as `material-22d78f`.
- `tiny` (zero chamfer) is **not verified by render**: no client on the
  host produces a window small enough to reach that branch, so it is
  covered by the Rust-side `material_frame` tests only.
- Fingerprint tests: a focused drifting window changes once per bucket and
  never within one; a focused static window is constant after the
  crossfade; the quiet fingerprint tests extend to focus 0 and 1.
- The GLSL was validated with `glslangValidator` by hand during Task 5;
  the harness carries no validator step.
- Smoke: the material-signals smoke harness gains focused-window cases
  measuring wakeups per second at drift 15, drift 0, and animations off,
  judged by the criteria it already applies to Breathe, plus a focus-toggle
  case measured against a no-material client control. The DRM acceptance
  gates run once pinned and once with a drifting focused window; both are
  awaiting the operator run described in the smoke evidence.

## Non-goals

- Interpolating glass parameters across the `is-active` swap
  (`material-5a5fff`).
- Canopy light as an attention response.
- Automatic suppression of the gradient focus ring on material windows.
- Exposing the depth fraction, aberration scale, or attenuation exponent.
