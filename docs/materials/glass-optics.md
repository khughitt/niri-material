# Glass optics

Type: guide; document types 1. What the glass shader computes at a fragment:
the slab and its ray model, the bevel profile, Fresnel transmission, the
reflection and edge-highlight optics and the specular hook they implement,
the accent tint, noise placement and layers, and the cap on the ring's light
shift. Each rule is stated as `prelude.frag`, `main.frag`, the optic programs
under `src/render_helpers/shaders/material/` and their Rust mirrors implement
it, with the tests that guard it. The pass order is in
[render-pipeline.md](render-pipeline.md), each parameter's range and default
in [material-config.md](material-config.md), the stage table in
[pipeline-schema.md](pipeline-schema.md), and the recipe for a new optic in
[adding-an-optic.md](adding-an-optic.md).

Coordinates are element-local logical px, y down, +z toward the viewer.

## Slab and ray model

The glass is a slab behind the window. `material_frame` inflates the window
rect by `bevel` less the larger of `|offset-x|` and `|offset-y|` on every side
(rounded up to physical pixels) and slides it by the offset: that is the
silhouette, which stays fixed. The face is the silhouette narrowed by the
chamfer on every side; it trails the jelly, moved by `mat_jelly_move` and
resized by `mat_jelly_resize`. The chamfer is `bevel`, limited to half the
slab's smaller side less 1 px, so a window too small for the band renders as
all face. The window's corner radii are refitted to the face box by one
proportional reduction (`fitRadii`), and the silhouette's radii are the face's
plus the chamfer. Coverage falls from 1 to 0 over one physical pixel inside
the silhouette.

The glass is a height field over the backdrop plane. The orthographic view
ray, refracted at a normal `n` with `1 / ior`, is `t`; it meets the backdrop
plane under local height `h` after

```text
L = h / max(-t.z, 0.25)          (rayPath)
```

- **Refraction.** Each tap moves its sample by `t.xy * L` (`tap`), with its
  own normal (perturbed by distortion and jelly ripple, then lifted by
  `liftTapNormal` to `n.z >= 0.05`, so no tap ray points upward), its own
  index (chromatic aberration) and its own height (anisotropic blur scales
  `h` per tap).
- **Attenuation.** Beer-Lambert takes `L` from the structural normal at
  `ior`: `attenuation-color` (clamped to [0.001, 1]) raised to
  `L / attenuation-distance`. Distortion moves taps but never lengthens this
  path.

On the face `L` is `thickness` and nothing moves. For any upward normal
`L <= ior * h`; the 0.25 floor binds only for aberration taps whose effective
index passes 4.

The result composes under the window: an opaque window pixel returns as it
is; otherwise `out = win + (1 - win.a) * glass * coverage`, premultiplied, and
`niri_alpha` (the window rule's opacity) multiplies once at the end.

## Bevel profile

The bevel lies between the face edge and the silhouette. Both boundaries
count, so the coordinate reaches both ends on every side however the jelly
displaces the face:

```text
w = innerDist - outerDist        local bevel width
u = innerDist / w                0 at the face edge, 1 at the silhouette, 0 on the face
h = thickness - R * f(u)         R = min(chamfer, thickness)
f(u) = 1 - (1 - u^k)^(1/k)       k = bevel-profile
```

Where `w` is under a physical pixel the fragment is rim (`u = 1`). `k = 1` is
a planar chamfer, 2 a quarter-round, larger values a squircle that stays flat
longer and rolls off harder; for `k > 1` the face joins the bevel without a
crease. With `thickness <= bevel` the glass thins to zero at the silhouette;
otherwise a wall of `thickness - R` remains.

The structural normal is `normalize(vec3(R * f'(u) * grad_u, 1))`, its slope
capped at `BEVEL_SLOPE_CAP` (20, so `n.z >= 0.05`). `grad_u` blends the face
box's exact outward gradient with the silhouette's softened one
(`softOuterGrad`: softplus for each positive part, `tanhs` for each sign, in
the log domain); the exact silhouette gradient would crease the normal where
jelly exposes the box's medial axis. On the ridge between parallel sides the
softened gradient vanishes and the face's stands in. `slabSurface` returns
the coverage, the normal, both distances, `h`, `u` and its direction.

`bevel.rs` mirrors these functions in f64. When
`the_shader_names_the_same_constants_and_formulas` fails, a constant or
formula line differs between the two: change both. `the_reference_table_holds`
pins reference values of `h`, `n.z`, `L` and displacement; the continuity
tests beside it fail if a deformed corner or a stadium's centerline creases
the normal.

## Fresnel transmission

`F` is Schlick's term from `ior` on the structural normal, with the cosine
`n.z` clamped to [0, 1]. What the surface reflects is not transmitted:

```text
glass = linearToSrgb((1 - F) * sampled * att + within + specular + emissive)
```

`within` (ring and aurora light, already inside the glass) is not scaled by
`1 - F`; it carries its own `att ^ 0.2` factor. The default glint is
`F * (0.15 + 0.85 * facing)`, where `facing` is how far the structural normal
points toward the signal light.

## Reflection

The `reflection` optic adds what the bevel reflects: the scene just beyond
the silhouette. It samples the composed background outward along the
across-bevel direction bent by the perturbation (`acrossDir` plus the
perturbed normal's tilt less the structural one's), `-outerDist + thickness`
px away, and adds it weighted by `F` and by `reflection` times
`smoothstep(0, 0.1, u)`, so it is zero on the face. It goes through the same
prefilter as the taps, so `roughness` softens it. It is not tinted by
`attenuation-color`: reflection happens before light enters the glass, which
keeps a dark edge carrying the wallpaper's colours. Neutral at 0.

## Edge highlight

The `edge-highlight` optic adds a key-light lobe where the bevel's normal
bisects the view and a light at 45 degrees' elevation in the signal light's
direction (`H` is 22.5 degrees off vertical). The lobe is GGX with
`alpha = 0.04 + 0.46 * roughness` (`highlight_alpha`, uploaded as
`mat_edge_highlight_alpha`), divided by its peak, so the amount is the peak
added linear brightness. A `tilt` factor, `smoothstep` of the structural
normal's tilt against `H`'s, fades it in from a flat surface: a rounded face
join stays dark, while a planar facet lights uniformly and flashes when its
slope matches `H` (`R / bevel = 0.414`). It is not multiplied by `F`.
`attention "rim-orbit"` sways it with the glint. Neutral at 0.
`highlight_weight` in `bevel.rs` mirrors it, and
`the_highlight_shader_names_the_same_formulas` pins the shared lines.

## Specular hook

A specular optic defines `vec3 <name>_specular(vec3 specular, Surface s)`.
It receives the specular light accumulated so far (the glint, then each
earlier specular optic's result) and returns the new total, so the site is a
sequence: `reflection` and `edge-highlight` add to it, and `iridescence` hues
everything before it. `main.frag` calls the hooks in `OPTICS` order, then
mixes in the signal accent. `Surface` (declared in `prelude.frag`) is the
fragment's surface:

| Field | Meaning |
| --- | --- |
| `p` | element-local logical px |
| `v` | element UV |
| `structural` | the slab's normal from the bevel profile |
| `perturbed` | the normal after distortion and jelly ripple |
| `cosine` | `structural.z` clamped to [0, 1] (under a swing, the view direction's cosine) |
| `fresnel` | `F`, Schlick at the structural normal |
| `across` | `u`: 0 at the face edge, 1 at the silhouette, 0 on the face |
| `acrossDir` | `normalize(grad u)`, outward across the bevel |
| `outerDist` | signed distance to the silhouette, negative inside |
| `view` | toward the viewer; declared only in the view variant, +z at rest |

A hook must return `specular` unchanged at its neutral value; it runs on
every glass fragment. The hook table for the other sites is in
[adding-an-optic.md](adding-an-optic.md).

## Accent tint

The response field `accent-tint` tints the attenuation color toward the
window's accent, on the CPU in f64, once per frame per material element
(`accent_tint` in `tint.rs`); the shader is unchanged. With
`p = thickness / attenuation-distance`, the face transmittance of the
configured color `T_c = c ^ p` and its luminance `Y`:

1. The tint chromaticity `k` is the linear accent divided by its luminance
   (`accent_chroma`), or neutral for black. During a signal crossfade `k`
   interpolates between the endpoints' values, so black to colour is
   continuous.
2. The target is `k * Y`, pulled toward the gray `Y` just far enough to put
   every channel in `[0.001 ^ p, 1]`.
3. `T = T_c + w * (target - T_c)` with `w = min(accent-tint * presence, 1)`;
   the uploaded color is `T ^ (1 / p)`, alpha unchanged.

So the face passes the untinted luminance of a neutral backdrop at every
weight, and only its hue changes. The configured color is uploaded bit for
bit when there is no accent, `w <= 0`, `p <= 0`, or `Y` is 0. The bevel's
longer path and the interior light's shorter one are not matched. Near-white
glass has little room for hue at its own luminance, so the tint is faint
there. Presence fades it in and out; a response or material swap steps it.
`tint.rs` and `src/tests/accent_tint.rs` hold its tests.

## Noise placement

`noise site=` places a layer at one of three sites:

- **`backdrop`**: a grain pass in each output's effect buffers
  (`grain.frag`, `noise_source`), over the sharp texture before the blur and
  the prefilter pyramids, so blur and roughness soften it, and every glass
  window and blurred background-effect element sees it. It is one setting per
  output: materials that place noise there must agree on their ordered
  backdrop layers, or the config is refused naming both materials. A
  transparent texel is left alone; the result is clamped once, since it is
  stored in an 8-bit premultiplied texture. If the pass fails, the sharp
  texture stands in until the next damage.
- **`glass`** (the default): `noise_behind`, once on the averaged linear
  backdrop after the taps and before attenuation. It encodes, adds the grain
  and returns linear light, so the bevel's thinner path attenuates it less.
- **`film`**: `noise_post`, on the encoded glass after the transmitted light,
  ring, aurora, specular light and sweeps, before coverage.

White and fine grain add in encoding, signed and unclamped; lightness moves
Oklab lightness on the clamped colour. Grain is screen-seeded and does not
refract with the image. Amount 0 is neutral at every site.
`src/tests/noise_site.rs` checks the identities between sites: an omitted
site equals `glass`, and on a flat face at `ior` 1 `film` and `backdrop` match
`glass` within one and two codes.

## Noise layers

A material writes up to four `noise` nodes (`NOISE_LAYERS`); a fifth is a
validation error. Each is a layer with its own amount, `type=`, `site=` and
`scale=`, packed into slot `k` of the four `vec4` noise uniforms in config
order. A node at amount 0 keeps its slot. Each site applies its layers in slot
order; white and fine layers commute, a lightness layer does not. Slot `k`
seeds with `NOISE_SEED_k`; backdrop layers use their place in the backdrop
list, not their material slot. Only when no node is written does slot 0
inherit the global `blur` block's `noise` (while backdrop blur is effective),
as white grain at the glass site, scale 1.

`scale=` is the grain cell in physical output pixels. At 1 or below each
pixel hashes on its own. Above 1 `noiseValue` reconstructs lattice values with
the cubic B-spline over the 4x4 points around the pixel and divides by the
reconstruction's own deviation, so `amount` means the same deviation at every
scale and position in a cell. Crossing 1 re-rolls the pattern once. A lone
slot-0 glass layer at scale 1 runs the single-layer arithmetic verbatim.
`src/tests/noise_layers.rs` checks slot order, seeding, the deviation across
scales and that the lattice does not show.

## Filament shift cap

The ring's light lands through the light-path index
`1 + (ior - 1) * light-ior`, a fifth of `thickness` into the slab, along the
perturbed normal (`lightShift`). `main.frag` caps the length of that shared
shift at half `ring-gap` before `filamentBand` evaluates the band. A chamfer
pixel then lands short of the core, so the chamfer shows the band's flank and
halo and never a second copy of the core. The per-channel aberration offsets
ride on top uncapped, and aurora uses the same index with no cap. Once the
shift reaches the cap, a higher `light-ior` no longer moves the ring's landing
point there; material-config.md tabulates where that happens.
`ring_cap_keeps_one_core` in `src/tests/ring_pair.rs` renders dense glass and
fails if a second core at least half the peak's brightness appears.
