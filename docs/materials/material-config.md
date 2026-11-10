# Material configuration

Type: data model, guide; document types 1. The configuration is currently
unversioned. The `material` definitions, every `glass` parameter with its
type, default and range, the optics, the presets, and the window rules and
validation errors that use them. `material` names a visual treatment, and
`glass` is the one type; every parameter below is visually active in the
built-in shader. What each optic computes is in
[glass-optics.md](glass-optics.md), the stage each parameter reaches in
[render-pipeline.md](render-pipeline.md), the `response` blocks and the
`signal` policy in [signals.md](signals.md), how jelly, the ring and the
tilt move in [motion.md](motion.md), and the shell's `surface-remap` in
[shell-remap.md](shell-remap.md).

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
| `bevel-profile` | float | 1 | 1–8 | — |
| `light-ior` | float | 6 | 1–12 | — |
| `offset-x` | float | 6 | −64–64 | logical px |
| `offset-y` | float | 6 | −64–64 | logical px |
| `saturation` | float | inherit | 0–3 | — |
| `noise` | float | inherit | 0–1 | — |
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |
| `noise` `site=` | `glass` / `backdrop` / `film` | `glass` | — | — |
| `noise` `scale=` | float | 1 | 1–16 | px |
| `aurora` | float | 0 | 0–1 | — |
| `aurora` `drift-hz` | float | 4 | 0–30 | Hz |
| `aurora` `color` | color | `#3dffb0` | any color | — |
| `aurora` `color` | color | `#7a5cff` | any color | — |
| `reflection` | float | 0 | 0–1 | — |
| `edge-highlight` | float | 0 | 0–1 | — |
| `iridescence` | float | 0 | 0–1 | — |

<!-- params:end -->

`jelly-flex` and `jelly-ripple` use thousandths only in their internal
representation; their configuration values and ranges above are unchanged.

How the jelly responds to moves, resizes and drags, and that it returns
exactly to rest, is in [motion.md](motion.md#jelly).

`light-ior` multiplies the bend applied to ring and aurora interior-light
paths only; the background taps are unaffected. The light-path index is
`1 + (ior - 1) * light-ior`.

`bevel-profile` shapes the bevel between the face and the silhouette:
`1` is a planar chamfer, `2` a quarter-round, and larger values a squircle
that stays flat longer and rolls off harder. The bevel is a height field: the
glass thins from `thickness` at the face edge to `thickness - min(bevel,
thickness)` at the silhouette, refraction and attenuation follow the refracted
ray to the backdrop plane under that local height, and the reflected share
`F` of the light is not transmitted
([glass-optics.md](glass-optics.md#bevel-profile)).
At `bevel-profile 1` on a straight side the normal, and so the ring cap table
below, is unchanged; for larger values the tilt varies across the bevel.

The ring's shared refracted shift is capped at half `ring-gap`, 4 px at the
default gap of 8. The cap keeps a second copy of the band core off the
chamfer. Without it, dense glass shows one there, with a seam at the face
edge. The cap bounds the shared shift only: the per-channel aberration
offsets ride on top of it uncapped, and aurora uses the same index with no
cap at all.

For an undistorted chamfer normal, the shared shift reaches the cap when
`sin(a - asin(sin(a) / n)) * 0.2 * thickness >= 0.5 * ring-gap`. Here `n`
is the light-path index and `a = atan(min(bevel, thickness) / bevel)` is
the chamfer tilt, 45° whenever `thickness >= bevel`. Beyond that point a
higher `light-ior` no longer moves the ring's landing point on the chamfer.
It still changes the aberration split, pixels whose distorted or rippled
normal yields less than the cap, and aurora.

| glass (`ior`/`thickness`/`bevel`, `ring-gap`) | reaches the cap from `light-ior` |
| --- | --- |
| 1.5/20/12, 8 (stock) | never within 1–12 |
| 1.24/43.3/9, 8 | about 5.6 |
| 1.02/80/12, 5 | about 10.1 |
| 1.28/31.2/10, 2 | every value |
| 1.5/80/12, 5 | every value |

How much `light-ior` still does past the cap depends on those other terms.
The aberration offsets grow with `chromatic-aberration`: at
`chromatic-aberration 1`, the 1.5/80/12 row's blue offset is about 1.32 px
at `light-ior 1`. On Prism's terminal glass (1.28/31.2/10,
`ring-gap 2`, `chromatic-aberration 0.36`, `distortion 0`, `aurora 0`,
`jelly-ripple 0.23`), the cap is 1 px and the chamfer's shared shift reaches
it at any `light-ior`. There `light-ior` acts only through the per-channel
aberration offsets (blue, the larger, is about 0.27 px at `light-ior 1` and
0.10 px at 6) and, while jelly is active, through rippled pixels.
`ring_cap_keeps_one_core` in `src/tests/ring_pair.rs` renders the table's
rows and fails if a second core at least half the peak's brightness appears.

`backdrop-blur` makes the glass refract the blurred backdrop rather than the
sharp one, which is what produces a frosted appearance: blur and refraction
compose into one image instead of being drawn as two. It is a switch, not a
strength — the amount of blur comes from the global `blur` block's `passes` and
`offset`, shared with every other blur consumer. Setting `blur { off }`
disables it along with all other blur, regardless of this parameter.

`saturation` then glass-site `noise` transform the averaged backdrop before attenuation,
through the `behind` hook. Their formulas run in sRGB (Oklab for lightness
grain) and return linear light. Additive glint, reflection, edge highlight,
ring, aurora and sweeps are not postprocessed; the bevel still transmits
attenuated grain. A written
value is a material optic and applies regardless of `backdrop-blur` and of
`blur { off }`. An omitted
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

`noise` also accepts `site="glass"` (default), `"backdrop"`, or `"film"`.
`backdrop` grains the output's shared backdrop buffers before the Kawase
blur and roughness pyramids. Every material placing noise there must agree on
the ordered list of its backdrop layers (amount, type, scale); disagreement
refuses the config and names both materials and both lists (`materials "a" and
"b" place different noise at the backdrop ([0.3 fine], [0.1 fine]) ...`).
Grain applies even with `blur { off }`; a window asking for a blurred
`background-effect` also samples it. `film` grains the finished encoded glass
after ring, aurora, glint and sweeps, on glass coverage only.
[performance.md](performance.md) gives each site's cost.

A material may write `noise` up to four times; each node is a layer with its
own amount, `type=`, `site=` and `scale=`, and a fifth node refuses the
config. Each site applies its layers in the order written; sites run backdrop,
glass, film. `scale=` is the grain's cell size in physical output pixels, 1
(the default, per-pixel grain) to 16; above 1 the grain is smooth value noise
normalised so an amount means the same strength at every size. A layer's
grain pattern follows its position: glass and film layers their position among
the material's nodes, backdrop layers theirs among the backdrop layers. Layers
in different positions get different patterns; the k-th backdrop layer shares
its pattern with a glass or film layer in position k, which blur and
refraction decorrelate.
Independent white or fine layers of one type and size at one site, with
nothing clipping, have the variance of one layer of amount `sqrt(a² + b²)` but
not its distribution, so they are close to redundant: layers are for
different types, sites or sizes.

`anisotropic-blur` instead smears the refraction itself along one axis.

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
Optics are listed in render order: `saturation`, `noise`, `aurora`, `reflection`,
`edge-highlight`, `iridescence`.
Contributors: see `adding-an-optic.md`.

### saturation

Stage 3a (`behind`). `saturation <amount>` mixes the encoded averaged backdrop
toward its luma, then returns linear light before attenuation. Its explicit
neutral is 1; 0 makes the backdrop grayscale, but tint and additive light can
still colour the result. An omitted amount inherits
the global `blur` block's `saturation` while backdrop blur is effective and
resolves to 1 otherwise.

### noise

Stage 3b (`behind`) at `site="glass"`; a source grain pass at
`site="backdrop"`; stage 9 (`post`) at `site="film"`.
`noise <amount> type=<type> site=<site> scale=<px>`, up to four nodes, grains
the selected carrier; `white`, `fine`, and `lightness` are described above.
Its explicit neutral is amount 0, which keeps the node's slot. When no node is
written, the amount inherits the global `blur` block's `noise` while backdrop
blur is effective and resolves to 0 otherwise, as white grain at the glass
site; a written node never inherits, and type, site and scale never do.

### iridescence

Stage 6. `iridescence <amount>` gives the Fresnel glint a thin-film hue
from the view angle: `hue = fract(2.5 * (1 - cos))` through the cosine
palette `0.5 + 0.5 * cos(2π (hue + (0, ⅓, ⅔)))`, and the glint becomes
`mix(glint, glint * palette * 2, amount)`. It runs before the signal accent
mix, so an accent still tints the result. Its explicit neutral is 0, and
omission is 0; nothing inherits. The `rainbow` preset pairs it with
`chromatic-aberration`, which is the dispersion the refracted image carries;
iridescence colours the edge light.

### reflection

Stage 6. `reflection <amount>` adds what the bevel reflects: the scene just
beyond the silhouette, sampled outward across the bevel and one thickness
further, weighted by the Fresnel term and faded in over the first tenth of
the bevel. It is not tinted by `attenuation-color` (reflection happens at the
surface), so a dark edge still shows the wallpaper's colours, and it goes
through the same prefilter as the refracted image, so `roughness` softens it.
Distortion and jelly ripple bend its direction. Its explicit neutral is 0, and
omission is 0; nothing inherits.

### edge-highlight

Stage 6. `edge-highlight <amount>` adds a key-light lobe on the bevel where
its normal bisects the view and a light at 45 degrees' elevation in the signal
light's direction (top left at rest; `attention "rim-orbit"` sways it with the
glint). The lobe is GGX, peak-normalized so the amount is the peak added
linear brightness, and `roughness` widens it. It fades in with the bevel's
tilt, so a rounded face join stays dark; a planar facet lights uniformly and
flashes when its slope matches the light's half-angle (`R / bevel = 0.414`).
Its explicit neutral is 0, and omission is 0; nothing inherits.

### aurora

Within stage. `aurora <amount> { drift-hz <hz>; color <a>; color <b>; }` adds a
slow colour field inside the glass: two octaves of simplex noise on the
element position (one noise unit is 250 px), offset by the window seed,
mix the two colours, and a coarser octave sets the brightness; the light
is weighted by `att ^ 0.2` like the ring, so it sits inside the slab. The
first `color` node is the field's start (noise 0), the second its end
(noise 1); both may be omitted, and any other count is the error
`aurora: expected two color nodes`. Its explicit neutral is amount 0;
nothing inherits.

`drift-hz` is the field's clock: `0` pins the field, otherwise at least 1,
and the error is `aurora drift-hz must be 0 or at least 1`. The field's
lookup point traces a small circle in noise space once per 600 s, so the
loop closes seamlessly; the clock
steps the phase `drift-hz` times per second in buckets on a shared optic clock.
A lit, visible aurora window redraws at that rate while input is active,
whether or not it is focused or carries a signal. After the input idle
threshold, its phase and uniforms hold and it requests no optic redraws.
Input resumes the held phase without catching up; client and backdrop damage
still render. Reading or watching video without input can therefore leave
Aurora held even while the client updates.
`signal { motion "reduced" }` halves the rate; `motion "off"` and `animations { off }` pin
the field at phase 0. A lit, moving aurora is the one optic that keeps a
window redrawing: [signals.md](signals.md#redraw-and-idle-budget) says when,
and [performance.md](performance.md) gives its measured cost.

## Presets

`resources/materials/` holds one file per preset material, named after the
material it defines, installed to `/usr/share/niri/materials/`. `include`
accepts an absolute path, so a preset is used with

````kdl
include "/usr/share/niri/materials/aurora.kdl"

window-rule {
    match app-id="^kitty$"
    material "aurora"
}
````

A preset is one material; the focus split is Prism's. `rainbow`
pairs `chromatic-aberration` with `iridescence`; `aurora` lights a cool slab
with the `aurora` optic at `drift-hz 4`.

## Signals

A material's `response` blocks, the response fields and their defaults,
and the top-level `signal` policy with the `material-signal` crossfade are
in [signals.md](signals.md#signal-responses).

## Window rules and validation

`material "name"` is a scalar window-rule field. When several matching rules
set it, the last matching rule wins.

The whole configuration is rejected with these validation errors:

- duplicate material names: `duplicate material: <name>`;
- a definition without exactly one `glass` block:
  ``missing node `glass` `` or
  ``duplicate node `glass`, single node expected``;
- a numeric value outside its parameter range:
  `value must be between <min> and <max>` (or, for `attenuation-distance`,
  `value must be greater than 0 and at most 65535`);
- an offset wider than the bevel: `offset must not exceed bevel`;
- response blocks without `default`:
  `material <name>: missing response "default"`;
- duplicate response names: `duplicate response: <name>`;
- a zero or negative ring width: `ring-width must be positive`;
- the retired lap time:
  `ring-sweep-ms was replaced by ring-beam-speed; see material-config.md`;
- the retired outer-edge inset:
  `ring-inset was replaced by ring-gap; see material-config.md`;
- the retired drift rate:
  `ring-drift-hz was replaced by ring-beam-speed; see material-config.md`;
- an unknown window-rule reference: `unknown material: <name>`;
- an unknown response reference:
  `material <name>: unknown response: <response>`;
- a `surface-remap` that names unsupported glass: the errors in
  [shell-remap.md](shell-remap.md#failures).

Includes participate in the same validation, including duplicate names and
unknown references.

On reload, editing parameters of the same resolved material name updates the
existing material state in place. Changing the resolved name swaps that state.
An invalid configuration is rejected as a whole and niri continues using the
previous configuration.
