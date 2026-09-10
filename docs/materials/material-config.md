# Material configuration

`material` definitions name a visual treatment. v1 provides one type,
`glass`. Every parameter below is visually active in the built-in shader.

`jelly-flex` and `jelly-ripple` consume niri's native move/scrolling and resize
animation residuals and return to their exact rest state when those springs
settle.

Inside the selected workspace, glass composes the workspace background over
the workspace color and retains the backdrop behind any remaining transparency.
Outside it, glass samples the backdrop directly.

```kdl
material "frost" {
    glass {
        thickness 20
        bevel 12
        offset-x 6
        offset-y 6
        attenuation-color "#dfe8ff"
        roughness 0.08
    }
}

window-rule {
    match app-id="org.example.App"
    material "frost"
    geometry-corner-radius 16
}
```

`render-pipeline.md` shows the stage at which each parameter below acts.

All `glass` parameters are optional. Colors use niri's normal color syntax;
lengths are logical pixels.

<!-- params:begin -->
| Parameter | Type | Default | Range | Unit |
| --- | --- | --- | --- | --- |
| `ior` | float | 1.5 | 1–3 | — |
| `thickness` | float | 20 | 0–200 | logical px |
| `attenuation-color` | color | `#dfe8ff` | any color | — |
| `attenuation-distance` | float | 60 | > 0 through 65535 | logical px |
| `chromatic-aberration` | float | 0 | 0–1 | — |
| `distortion` | float | 0 | 0–1 | — |
| `distortion` `scale=` | float | 0.5 | 0–2 | — |
| `anisotropic-blur` | float | 0 | 0–1 | — |
| `roughness` | float | 0 | 0–1 | — |
| `backdrop-blur` | bool | false | true / false | — |
| `jelly-flex` | float | 0.004 | 0–0.02 | — |
| `jelly-ripple` | float | 0.06 | 0–0.5 | — |
| `bevel` | float | 12 | 0–128 | logical px |
| `light-ior` | float | 6 | 1–12 | — |
| `offset-x` | float | 6 | −64–64 | logical px |
| `offset-y` | float | 6 | −64–64 | logical px |
| `iridescence` | float | 0 | 0–1 | — |
| `saturation` | float | inherit | 0–3 | — |
| `noise` | float | inherit | 0–1 | — |
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |

<!-- params:end -->

`jelly-flex` and `jelly-ripple` use thousandths only in their internal
representation; their configuration values and ranges above are unchanged.

`light-ior` multiplies the bend applied to the focus filament's light path
only; the background taps are unaffected. The light-path index is
`1 + (ior - 1) * light-ior`.

The knob is much narrower than its range suggests. The filament's shared
refracted shift is capped at half `ring-inset` — 2.5 px at the default
inset of 5 — so the core always stays inside the bevel, and the shift grows
roughly as `sin(45deg - asin(sin 45deg / n)) * 0.6 * thickness`. At
`light-ior 1`, the minimum, that is already about 3.5 px on the stock
default glass (`ior 1.5`, `thickness 20`) and about 4.6 px on thick glass
near `ior 1.24` with `thickness 43.3`. Both are past the cap, so on such
glass *every* `light-ior` value saturates it: the knob no longer positions
the filament's core, and only widens the chromatic split — which does
nothing at all when `chromatic-aberration` is 0.

`light-ior` only positions the core on very low-index glass near `ior 1.02`
(the calibration the spike used), where the light-path product lands near
0.12 at the default 6.

`backdrop-blur` makes the glass refract the blurred backdrop rather than the
sharp one, which is what produces a frosted appearance: blur and refraction
compose into one image instead of being drawn as two. It is a switch, not a
strength — the amount of blur comes from the global `blur` block's `passes` and
`offset`, shared with every other blur consumer. Setting `blur { off }`
disables it along with all other blur, regardless of this parameter.

`noise` and `saturation` are applied after the glass optics: saturation
first, then screen-space noise. A written value is a material optic and
applies regardless of `backdrop-blur` and of `blur { off }`. An omitted
value inherits the global `blur` block's `noise` or `saturation` while
`backdrop-blur` is effective and is neutral otherwise (`noise 0`,
`saturation 1`); each parameter decides on its own, so `blur { off }` and
material opt-out neutralise only inherited values. Per-window
`background-effect` overrides remain independent and do not alter the
material.

`noise` takes an optional `type="..."` property naming its grain. `white` is the
original per-pixel uniform hash added equally to the three channels. `fine`
subtracts each pixel's eight-neighbour mean from that hash and rescales it, so
the grain loses its low-frequency clumps and its hard extremes while keeping
the same strength for the same amount; it is still achromatic. `lightness`
applies the `fine` value to Oklab lightness instead, so the backdrop's chroma
and hue hold except where the result leaves the sRGB gamut and clamps. The
type has no inheritance and an omitted type is `white`.

It is unrelated to `anisotropic-blur`, which smears the refraction itself along
one axis and does not soften the backdrop.

`roughness` progressively softens detail refracted through the glass. It first
uses `backdrop-blur` to choose the sharp or globally blurred source, then
filters that source; setting `blur { off }` disables the blurred-source switch
but does not disable roughness on the sharp source. The native default is `0`,
which preserves the v1 appearance and performs no prefilter allocation or
downsample work. Non-zero roughness lazily builds a damage-aware texture
pyramid for the selected per-target source and reuses it until that source
changes.

The slab uses `thickness` as its depth. Its frame is the window rectangle
inflated by `bevel - max(abs(offset-x), abs(offset-y))`, then translated by
the offsets. Its inner corners follow the window's effective
`geometry-corner-radius`; the outer corners add the drawn chamfer.

## Optics

The glass pipeline is a slab plus an ordered list of optics. Each optic owns
its node, resolved values, uniforms, and GLSL stage. Its explicit neutral
changes nothing; omission can instead inherit where stated below.
Contributors: see `adding-an-optic.md`.

### iridescence

Stage 5. `iridescence <amount>` gives the Fresnel glint a thin-film hue
from the view angle: `hue = fract(2.5 * (1 - cos))` through the cosine
palette `0.5 + 0.5 * cos(2π (hue + (0, ⅓, ⅔)))`, and the glint becomes
`mix(glint, glint * palette * 2, amount)`. It runs before the signal accent
mix, so an accent still tints the result. Its explicit neutral is 0, and
omission is 0; nothing inherits. The `rainbow` preset pairs it with
`chromatic-aberration`, which is the dispersion the refracted image carries;
iridescence colours the edge light.

### saturation

Stage 9. `saturation <amount>` mixes the encoded glass colour toward its
luma. Its explicit neutral is 1; 0 is grayscale. An omitted amount inherits
the global `blur` block's `saturation` while backdrop blur is effective and
resolves to 1 otherwise.

### noise

Stage 10. `noise <amount> type=<type>` grains the encoded glass colour per
screen pixel; `white`, `fine`, and `lightness` are described above. Its
explicit neutral is amount 0. An omitted amount inherits the global `blur`
block's `noise` while backdrop blur is effective and resolves to 0 otherwise;
the type never inherits.

## Signal responses

Each material can map a window signal to glass effects with named `response`
blocks. A material with no response blocks gets this built-in `default`:

| Parameter | Values | Default |
| --- | --- | --- |
| `accent` | `ring`, `none` | `ring` |
| `focus` | `ring-light`, `none` | `ring-light` |
| `attention` | `rim-orbit`, `ring-pulse`, `none` | `rim-orbit` |
| `ping` | `ripple`, `flash`, `sweep`, `none` | `ripple` |
| `done` | `ripple`, `flash`, `sweep`, `none` | `sweep` |
| `error` | `ripple`, `flash`, `sweep`, `none` | `flash` |
| `ring-inset` | 0–128 logical px | 5 logical px |
| `ring-width` | > 0, up to 128 logical px | 2.6 logical px |
| `ring-color` | `"#rrggbb"` | `#ccccff` |
| `ring-drift-hz` | 0, or 1–30 (quantized to tenths of a hertz) | 15 |

When any `response` block is present, one must be named `default`. Named
responses inherit omitted fields from that block; the `default` block itself
inherits omitted fields from the built-in defaults.

```kdl
material "terminal-glass" {
    glass {
        bevel 16
    }

    response "default" {
        accent "ring"
        focus "ring-light"
        attention "rim-orbit"
        ping "ripple"
        done "sweep"
        error "flash"
        ring-inset 5
        ring-width 2.6
        ring-color "#ccccff"
        ring-drift-hz 15
    }

    response "loud" {
        attention "ring-pulse"
    }
}

window-rule {
    match app-id="^kitty$" signal-source="^familiar$" signal-tag="^cats/"
    material "terminal-glass" response="loud"
}
```

`response=` selects a named response on the referenced material; omitting it
selects `default`. `signal-source` and `signal-tag` are regex matches:
`signal-source` matches any source slot (including niri's native `niri`
slot), while `signal-tag` matches the window's folded signal tag. A window
with no signal matches neither.

The filament is one band shared by focus and signals. `focus "ring-light"`
lights it on the focused window, in `ring-color`, drifting at
`ring-drift-hz` steps per second (0 pins it; otherwise at least 1, and the
rate is quantized to tenths of a hertz so the steps divide the 10 s drift
period evenly). `accent "ring"` lets a
window signal light and tint the same band on any window. Both together show
the drifting filament in the accent color. The band sits `ring-inset` px
inward from the slab's outer edge, is refracted through the glass, and is
masked to the bevel, so it never lights the window face. Every resolved
response must satisfy `ring-inset + ring-width <= bevel` and `ring-width > 0`.
The filament shows through the slab's exterior band and through translucent
window pixels; an opaque window shows a full ring only when
`bevel >= 2 * max(|offset-x|, |offset-y|) + ring-inset + ring-width`.
Set `focus-ring { off }` (globally or in a window rule) for material
windows so the gradient ring does not draw a second ring; non-material
windows keep whatever ring the layout configures.

**What changes on upgrade.** The focus filament is on by default, so a
material window that never configured a `response` block now shows a
drifting ring of light in its bevel whenever it is focused. The signal
accent ring changed shape at the same time: it was a box band with +/-0.5 px
soft edges and is now a Gaussian core with a halo, at new defaults of
`ring-inset 5` and `ring-width 2.6` (previously 6 and 2). To go back to an
unlit focused window, set `focus "none"` in the material's `default`
response; the
filament and the accent ring are otherwise the same band, so `accent "none"`
turns off the signal tint alone. If the upgrade leaves two rings on screen,
that is the layout's gradient ring underneath — turn it off with
`focus-ring { off; }`.

## Signal motion and animation

The top-level signal policy defaults to `full`:

```kdl
signal {
    motion "reduced" // full | reduced | off
}

animations {
    material-signal {
        duration-ms 400
        curve "ease-out-cubic"
    }
}
```

`reduced` changes sustained `flash` to `pulse`, `pulse` to `breathe`, and an
impulse `flash` response to `sweep`. `off` makes sustained motion static and
drops impulse effects, while retaining the static accent and level
crossfade. `material-signal` is the baseline signal crossfade; it defaults to
400 ms with `ease-out-cubic` and follows the normal animation configuration,
including `animations { off }`. The focus filament fades in and out with
`material-signal` too, and `reduced` halves `ring-drift-hz` while `off` and
`animations { off }` pin the drift.

## Window rules and validation

`material "name"` is a scalar window-rule field. When several matching rules
set it, the last matching rule wins.

The whole configuration is rejected with these validation errors:

- duplicate material names: `duplicate material: <name>`;
- a definition without exactly one `glass` block: `missing node \`glass\`` or
  `duplicate node \`glass\`, single node expected`;
- a numeric value outside its parameter range: `value must be between <min> and
  <max>` (or, for `attenuation-distance`, `value must be greater than 0 and at
  most 65535`);
- an offset wider than the bevel: `offset must not exceed bevel`;
- response blocks without `default`: `material <name>: missing response
  "default"`;
- duplicate response names: `duplicate response: <name>`;
- a ring outside the bevel: `ring-inset + ring-width must not exceed bevel`;
- a zero or negative ring width: `ring-width must be positive`;
- a drift rate strictly between 0 and 1: `ring-drift-hz must be 0 or at
  least 1`;
- an unknown window-rule reference: `unknown material: <name>`.
- an unknown response reference: `material <name>: unknown response:
  <response>`.

Includes participate in the same validation, including duplicate names and
unknown references.

On reload, editing parameters of the same resolved material name updates the
existing material state in place. Changing the resolved name swaps that state.
An invalid configuration is rejected as a whole and niri continues using the
previous configuration.
