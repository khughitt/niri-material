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
| `saturation` | float | inherit | 0–3 | — |
| `noise` | float | inherit | 0–1 | — |
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |
| `noise` `site=` | `glass` / `backdrop` / `film` | `glass` | — | — |
| `noise` `scale=` | float | 1 | 1–16 | px |
| `aurora` | float | 0 | 0–1 | — |
| `aurora` `drift-hz` | float | 4 | 0–30 | Hz |
| `aurora` `color` | color | `#3dffb0` | any color | — |
| `aurora` `color` | color | `#7a5cff` | any color | — |
| `iridescence` | float | 0 | 0–1 | — |

<!-- params:end -->

`jelly-flex` and `jelly-ripple` use thousandths only in their internal
representation; their configuration values and ranges above are unchanged.

The [NVIDIA motion sweep](2026-09-11-jelly-motion-sweep.md) resolves both jelly
parameters during scripted column moves and verifies exact return to settled
pixels. Its sample points cover the current ranges; it does not establish
interactive-drag behavior or a new perceptual range.

`light-ior` multiplies the bend applied to ring and aurora interior-light
paths only; the background taps are unaffected. The light-path index is
`1 + (ior - 1) * light-ior`.

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
grain) and return linear light. Additive glint, ring, aurora and sweeps are
not postprocessed; the chamfer still transmits attenuated grain. A written
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
`backdrop` grains the output's shared effect-buffer texture before the Kawase
blur and roughness pyramids. Every material placing noise there must agree on
the ordered list of its backdrop layers (amount, type, scale); disagreement
refuses the config and names both materials and both lists (`materials "a" and
"b" place different noise at the backdrop ([0.3 fine], [0.1 fine]) ...`).
Grain applies even with `blur { off }`; a window asking for a blurred
`background-effect` also samples it. `film` grains the finished encoded glass
after ring, aurora, glint and sweeps, on glass coverage only. The site's
softening under blur is measured in `2026-10-05-noise-placement-evidence.md`.

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
Optics are listed in render order: `saturation`, `noise`, `aurora`, `iridescence`.
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
Aurora held even while the client updates. `signal { motion
"reduced" }` halves the rate; `motion "off"` and `animations { off }` pin
the field at phase 0. In the llvmpipe smoke, amount 0.5 at 4 Hz cost 5.552 ms
per material draw versus 4.308 ms plain (+28.9%); this software-renderer
measurement does not establish physical-GPU cost. The subsequent
[RTX 3070 hardware checks](2026-09-11-material-hardware-evidence.md) measured
9.728 µs for both plain and aurora median material draws in a small scene,
but clock variation prevented resolving relative overhead. Before input
settling was added, idle redraws were
exactly 4 Hz, 2 Hz with reduced motion, and zero when pinned or off; whole-board
power variation prevented attributing a watt cost. The isolated
[idle-budget measurement](2026-09-11-idle-budget-evidence.md) later resolved it
on DRM at 3440×1440: a lit drifting aurora window costs +0.85 W of board power
at 4 Hz and +0.68 W at 2 Hz over resting jelly (upper bounds 1.11 W and
0.98 W), with the GPU in P8 at 210 MHz throughout. Resting jelly costs nothing
resolvable over plain glass: +0.015 W, upper bound 0.13 W.

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

A preset is one material; the focus split is Prism's, as today. `rainbow`
pairs `chromatic-aberration` with `iridescence`; `aurora` lights a cool slab
with the `aurora` optic at `drift-hz 4`. The evidence docs named in
`../specs/2026-09-10-material-optics-design.md` record how each preset's
values were tuned.

## Signal responses

Each material can map a window signal to glass effects with named `response`
blocks. A material with no response blocks gets this built-in `default`:

| Parameter | Values | Default |
| --- | --- | --- |
| `accent` | `ring`, `none` | `ring` |
| `accent-tint` | 0–1 | 0 (off) |
| `focus` | `ring-light`, `none` | `ring-light` |
| `attention` | `rim-orbit`, `ring-pulse`, `none` | `rim-orbit` |
| `ping` | `ripple`, `flash`, `sweep`, `none` | `ripple` |
| `done` | `ripple`, `flash`, `sweep`, `none` | `sweep` |
| `error` | `ripple`, `flash`, `sweep`, `none` | `flash` |
| `ring-beam-speed` | 0–5000 logical px/s | 300 |
| `ring-beam-noise` | 0–1 | 0 |
| `ring-beam-noise-hz` | 0–30 wanders per second | 3 |
| `ring-beam-decay` | 0–20000 logical px | 0 (no decay) |
| `ring-gap` | 0–128 logical px | 8 logical px |
| `ring-glow` | 0–3 | 1.0 |
| `ring-rest` | 0–3 | 1.0 |
| `ring-accent` | 0–3 | 1.0 |
| `ring-width` | > 0, up to 128 logical px | 2.6 logical px |
| `ring-color` | `"#rrggbb"` | `#ccccff` |

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
        accent-tint 0
        focus "ring-light"
        attention "rim-orbit"
        ping "ripple"
        done "sweep"
        error "flash"
        ring-beam-speed 300
        ring-beam-noise 0
        ring-beam-noise-hz 3
        ring-beam-decay 0
        ring-gap 8
        ring-glow 1.0
        ring-rest 1.0
        ring-accent 1.0
        ring-width 2.6
        ring-color "#ccccff"
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
lights it on the focused window, in `ring-color`. On every focus gain one
beam of that light runs the perimeter of the band once, clockwise from the
top-left corner, at `ring-beam-speed` logical px/s: a bright head a few tens
of pixels wide with a tail diffusing over a quarter of the perimeter (at most
1200 px), fading in as it sets off and out as it returns to its start, the
tail draining behind it. A lap takes `perimeter / ring-beam-speed`; at the
default 300 px/s a full 1280×720 pane (perimeter about 3800 px) runs about
16 s, tail included, and `ring-beam-speed 900` runs it three times as fast. The ring then settles to a
dim, even resting glow and costs no redraws until the next focus gain (no
deadline, a constant fingerprint). `ring-beam-speed 0` shows the resting glow
without a beam; `signal { motion "reduced" }`, `motion "off"`, and
`animations { off }` skip the beam the same way. A focus change mid-run
restarts the beam on the newly focused window; focus loss ends it at once.
`ring-glow` scales the whole focus light — head, tail, resting glow and
spill together — so their ratios hold while the total is tuned.
`ring-rest` scales only the resting glow, the ring that stays on a focused
window once the beam has passed: `0` leaves no resting ring while the beam
still runs on every focus gain, and above `1` the resting ring is brighter
than the beam's base. The comet rides on top of the resting level, so it
keeps its own brightness at any `ring-rest`.

`ring-beam-noise` makes the head's brightness wander as it travels, so the
comet reads as a living light rather than a lamp on a track. It is the
fraction either side of the head's own brightness that the wander reaches:
`0` is a steady head (the default, and the appearance every earlier release
shipped), `0.3` a gentle breathing, `1.0` a head that flares and nearly dies.
`ring-beam-noise-hz` is how fast it wanders, in wanders per second; either
value at `0` leaves the head steady.

The wander rides the head alone. The tail is a trail the head has already
left and stays smooth, and the resting glow is untouched — the quiet ring
still costs no redraws and keeps its constant fingerprint, so a wandering
head never puts a settled window back on a clock. It is seeded per window
and per run, so two panes that gain focus together do not flicker in step
and one pane's successive runs do not replay. The motion policies that skip
the beam skip the wander with it.

`ring-beam-decay` lets the comet die out on its way round: it is the
distance, in logical px along the band, over which head and tail darken
together until they are gone, falling as `(1 − distance / decay)²` so the
light eases out rather than cutting off. `ring-beam-decay 1500` at the
default speed is dark after 5 s, well before a full pane's lap closes. It is
a distance, not a time, so the comet's reach is the same on every window and
at every `ring-beam-speed`. The run ends where the comet goes dark, so a
short decay also stops the redraws sooner; the resting glow is unaffected.
`0`, the default, is no decay: the comet keeps its brightness for the whole
lap, as every earlier release did. A decay longer than the lap and its tail
still dims the comet along the way but never shortens the run. The motion
policies that skip the beam skip the decay with it.

`accepted_ring_look` in `src/tests/ring_look.rs` holds the owner's accepted
terminal glass and ring (2026-10-01: `ring-beam-decay 4150`, speed 4350,
`ring-gap 6`, `ring-glow 1.2`). It renders that look through a real focus
gain, on a pane longer than the decay and on one shorter than it, from the
inactive material through the comet to rest. The renders are frozen and
byte-identical from run to run. Set `RING_LOOK_DUMP=<dir>` before and after
a change to ring or edge rendering, and compare the PNGs. The backdrop is
flat gray, so refraction of the backdrop does not show there.

`accent "ring"` lets a window signal light and tint the same band on any
window. Both together show the filament in the accent color. `ring-accent`
scales only the accent's own light on the band: at `0` a signal no longer
outlines its window, but the accent still colors the focus light, so the
comet and the resting ring run in the window's hue. The edge glint that
`attention "rim-orbit"` tints with the accent is separate; `attention "none"`
turns it off. The band sits
`ring-gap` px inward from the edge of the flat face (where the chamfer ends),
so it is always under the face and the bevel may shrink to make room; the
light is refracted through the glass at its remaining interior depth and
shows through translucent window pixels. On the chamfer the beam spills a
little light outward from the face edge, fading to the outer edge, so the
frame reads as lit by the beam. `ring-width` must be positive.

`accent-tint` tints the glass body toward the signal accent's hue,
independently of `accent`: `accent "none"` takes the accent off the band and
leaves the tint. The weight moves hue and saturation only. The flat face
transmits the same luminance as the untinted glass over a neutral backdrop,
so dark glass stays dark; a saturated backdrop shifts, brighter in the
channels the tint opens and darker in the others. The chamfer and the ring's
light pass the same tinted glass: under `accent "ring"` the band and the
focus light brighten with the tint (magenta at weight 1 by about 1.7× on
dark glass), and under `accent "none"` a near-white `ring-color` band dims.
Near-white glass has little room for hue at its own luminance, so the tint
barely shows on the default glass. Without an accent the glass is unchanged,
and the tint follows the accent's crossfade in and out. Design:
[accent tint](../specs/2026-10-03-accent-tint-design.md).
On dark glass the recommended weight is `1`. The hue reads mainly through
the band and the focus light, which pass the tinted glass; the body keeps
its luminance, so under a translucent window it shifts only a few levels in
hue, and lower weights barely register. The default stays 0.

Two limits follow from the placement. An opaque window shows no ring: the
band lies wholly under the face and there is no fallback band on the
chamfer. A face narrower than `2 * ring-gap` on either axis has no beam
line, and the ring — accent band, beam and resting glow — is not drawn
on it. A `ring-sweep-ms`, `ring-inset`
or `ring-drift-hz` line is rejected with the replacement named. Set
`focus-ring { off }` (globally or in a window rule) for material windows so
the gradient ring does not draw a second ring; non-material windows keep
whatever ring the layout configures.

**What changes on upgrade.** The focus filament is on by default, so a
material window that never configured a `response` block shows a ring of
light under its face whenever it is focused, with one beam running the
ring on every focus gain. `ring-sweep-ms` and `ring-inset` are rejected by
name (`ring-sweep-ms was replaced by ring-beam-speed`, `ring-inset was
replaced by ring-gap`; neither is reinterpreted); Prism users run `prism
migrate`, which maps `glass.ring.sweepMs` to `glass.ring.beamSpeed` (`0`
stays `0`, any positive value becomes the default 300). The band moved from
`ring-inset` px inside the slab's outer edge to `ring-gap` px inside the
face edge, so it sits under the face instead of on the chamfer: opaque
windows show no ring, and a face narrower than `2·ring-gap` shows no beam.
The signal accent ring is a Gaussian core with a halo, `ring-width 2.6`
(the box band with +/-0.5 px soft edges was retired earlier). To go back to
an unlit focused window, set `focus "none"` in the material's `default`
response; the filament and the accent ring are otherwise the same band, so
`accent "none"` turns off the signal tint alone. If the upgrade leaves two
rings on screen, that is the layout's gradient ring underneath — turn it off
with `focus-ring { off; }`.

## Signal motion and animation

The top-level signal policy defaults to `full`:

```kdl
signal {
    motion "reduced" // full | reduced | off
    idle-after-ms 30000
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
`material-signal` too; `reduced`, `off`, and `animations { off }` skip its
focus-gain sweep.

`idle-after-ms <int>` — sustained attention motion (`breathe`, `pulse`,
`flash`) settles to the static indication (level and accent lit, no pulse),
and Aurora holds its phase, once no input has arrived for this long. Attention
resumes on its absolute clock; Aurora resumes from its held phase. Default
30000; `0` disables both gates; at most 3600000, and the error
is `idle-after-ms must be at most 3600000`.

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
- a zero or negative ring width: `ring-width must be positive`;
- the retired lap time: `ring-sweep-ms was replaced by ring-beam-speed; see
  material-config.md`;
- the retired outer-edge inset: `ring-inset was replaced by ring-gap; see
  material-config.md`;
- the retired drift rate: `ring-drift-hz was replaced by ring-beam-speed; see
  material-config.md`;
- an unknown window-rule reference: `unknown material: <name>`.
- an unknown response reference: `material <name>: unknown response:
  <response>`.

Includes participate in the same validation, including duplicate names and
unknown references.

On reload, editing parameters of the same resolved material name updates the
existing material state in place. Changing the resolved name swaps that state.
An invalid configuration is rejected as a whole and niri continues using the
previous configuration.
