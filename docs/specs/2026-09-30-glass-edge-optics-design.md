# Glass edge optics

**Status:** revised after spec review rounds 1 to 3 (ray-consistent path
and its bound, lifted tap normals, two-boundary coordinate with a stable,
odd softened outer gradient, highlight transition, interior-light
attenuation), 2026-09-30; pending review. Task: `material-be611b`.
Follow-on idea that builds on this geometry: `material-7f5751` (content in the
glass).

## 1. Problem

Glass edges read as a flat, dark or oversaturated stripe whenever the tint is
strong (short `attenuation-distance`) or the slab is thick. Only a weak tint
on thin glass gives a legible edge, and those values rarely suit the face.
The goal is an edge that reads as a clear, lit edge of *this* glass at any
tint and thickness, by physically motivated means.

## 2. Diagnosis

Every term on the bevel is constant across its width
(`shaders/material/prelude.frag` `slabSurface`, `tap`; `main.frag`):

- **Geometry.** The bevel is a planar ramp: one normal from the inner edge to
  the silhouette, with rise `min(bevel, thickness)`. Once
  `thickness >= bevel` the slope locks at 45 degrees.
- **Refraction.** Each tap displaces by `refract(...).xy * thickness`, the
  global thickness, so the whole band shows one translated copy of the
  backdrop, with a step where the face ends.
- **Attenuation.** The path is `thickness / cos`, again the global thickness,
  so a 45 degree bevel always carries 1.41 times the face's exponent. The
  model treats the bevel as full-thickness glass, where real glass thins.
- **Fresnel.** Schlick at 45 degrees is `f0 + (1 - f0) * 0.0022`, about
  `f0`: 1.5 % at ior 1.28. The glint is that times the fixed top-left
  facing weight, and nothing else lifts the band.

At the live Prism settings (2026-09-30) transmission is already gone before
the bevel matters: the focused material (`attenuation-color #0D1D1E`,
distance 11, thickness 31.2) transmits about 1e-7 of the backdrop on the
face and less on the bevel; the inactive material (`#131415`, 88, 75.3)
transmits 1.3 % on the face and 0.2 % on the bevel. The edge is a black band
carrying a uniform 1.5 % white glint. Dark glass in the world gets its edges
from reflection, not transmission, and this model has no reflection that
varies across the edge.

## 3. Design

Three changes, each grounded in how a polished glass edge looks:

1. a **height-field bevel** with a profile control, whose local height drives
   refraction and attenuation;
2. **energy-conserving Fresnel** with a **scene reflection** term;
3. a **light-facing edge highlight**.

Total internal reflection lines on thick edges are out of scope (section 8).

### 3.1 Height-field bevel (core)

**Coordinate.** The bevel lies between two boundaries. The inner face
(`innerDist`) trails the jelly move and resize. The silhouette (`outerDist`,
negative inside) stays fixed. A coordinate measured against `chamfer` alone
breaks under motion: a 3 px face shift at chamfer 12 leaves widths of 9 and
15 on opposite sides. The profile is therefore measured against both
boundaries:

    w = innerDist - outerDist         // local bevel width
    u = innerDist / w                 // 0 at the face edge, 1 at the silhouette

`u` reaches both ends on every side, however the face is displaced. At rest
the two boxes are concentric offsets (`outer_r = inner_r + chamfer`), so
`w = chamfer` and `u = innerDist / chamfer` everywhere, corners included.
Where `w` falls below one physical pixel (`1 / niri_scale`) the fragment is
rim (`u = 1`).

**Profile.** `R = min(chamfer, thickness)` is the rise, constant around the
perimeter: the face is a rigid plane at height `thickness`, and the silhouette
sits `R` below it whatever the jelly does. Local height is

    h = thickness - R * f(u),   f(u) = 1 - (1 - u^k)^(1/k)

with `k = bevel-profile`. `k = 1` is today's planar chamfer. `k = 2` is a
circular quarter-round, and larger `k` a squircle that stays flat longer and
rolls off harder. For `k > 1`, `f'(0) = 0`, so the face joins the bevel
without a crease, and `f'(1)` is unbounded, so analytically Fresnel reaches 1
at the rim. The slope cap below stops it at `n.z = 0.05`, where Schlick
gives `F = 0.777` at ior 1.28.
On the face, `h = thickness`. With `thickness <= bevel` the glass thins to
zero at the silhouette. With `thickness > bevel` a vertical wall of
`thickness - R` remains, as today.

**Normal.** From the height field:

    grad_u = (-outerDist * gIn + innerDist * gOut) / (w * w)
    n      = normalize(vec3(R * f'(u) * grad_u, 1))

This is the exact gradient of `u`, equivalently
`((1 - u) * gIn + u * gOut) / w`. `gIn` is the outward unit gradient of the
inner rounded box (`sdRoundedBoxGrad`). On the bevel it is continuous, because
the exterior distance of a convex box is C1.

`gOut` must not use the exact gradient. The fixed outer box's interior
gradient switches between nearest sides along its medial axis. At rest that
region lies inside the face. Jelly motion exposes it when the face moves
further than the corner radius covers. At outer 112, inner 88, chamfer 12,
inner radius 0 and a 3 px shift, the exact gradient puts a crease in the
normal: 0.07 at `k = 2` and 0.23 at `k = 1`, and the reflection sample jumps
with it. `gOut` therefore comes from a softened gradient. Per axis `i`,
with `s = 0.5` logical px:

    q_i  = abs(p_i) - b_i + r                // as in sdRoundedBox
    l_i  = lsp(q_i / s)                      // log(softplus), evaluated stably
    c_i  = exp(l_i - max(l_x, l_y)) * tanhs(p_i / s)
    gOut = c / length(c)

Each part has a reason:

- **Softplus** replaces each positive part `max(q_i, 0)`. On corner arcs and
  straight sides it equals `q_i` to within `s * e^(-|q_i| / s)`, so it agrees
  with the exact gradient there. In the core it blends the two nearest sides
  smoothly.
- **`tanhs(t) = tanh(clamp(t, -10, 10))`**, written with `exp`, since GLSL
  ES 1.00 has no `tanh`. It replaces `sign(p_i)`. Softplus is positive even
  where `p_i = 0`, so a hard sign would reverse a nonzero component across
  the box's centerline. At maximum radius (outer half-size (10, 100),
  chamfer 6, inner radius 4, `k = 2`) that is a 0.038 normal jump even at
  rest. With `tanhs` the component passes through zero continuously, as the
  exact radial gradient does.
- **Log domain.** Literal softplus overflows in fp32 (`exp(128)` at
  `q = 64`) and rounds to zero far inside (`1 + exp(-20) = 1`). Working with
  `l_i` and subtracting the maximum keeps the dominant component's magnitude
  at exactly 1. The other component lies in (0, 1], so the normalization
  never sees infinity, and sees zero only on the ridge below. `lsp(t)` is
  `log(t + log(1 + exp(-t)))` for `t > 0`, and `t + log(log(1 + x) / x)`
  with `x = exp(t)` for `t <= 0`, replacing the ratio by `1 - x / 2` when
  `x < 1e-3`. Every `exp` argument is at most 0, except in `tanhs`, whose
clamp bounds it at 20.
- **Ridge guard.** Where `length(c) < 1e-4`, `gOut = gIn`. In a 2 million
  point fp32 sweep (half-sizes 2 to 4000 px, every radius) that happens only
  on the outer box's ridge between parallel sides. That ridge is the slab's
  centerline at depth equal to its half-size, a true crease of any distance
  field. It reaches the bevel only if jelly moves the face further than
  `min(half-size) - chamfer`, which is at least 1 px under the tiny-slab
  guard. That crease is accepted. Everywhere else in the sweep the fp32
  direction is within 0.0005 rad of f64.

Under motion, measured normal jumps fall in proportion to the sampling step
in every case tried: radius 0, 4 and 8 with 3 px diagonal and axial shifts,
and the maximum-radius stadium at rest and shifted 2 px, crossing both
centerlines, at `k = 1` and `k = 2`. That is continuity. At rest,
`gIn = gOut` exactly on straight sides, so `k = 1` reproduces today's normal
there. Within about a pixel of the junctions between corner arcs and
straight sides, it departs from today's normal by at most 0.03, on at most
3 % of bevel pixels (inner radius 0, the worst case).

Stable-normal policy: the slope `|R * f'(u) * grad_u|` is capped at 20
(`n.z >= 0.05`).

**Ray model.** The glass is a height field over the backdrop plane. A ray
refracted at the surface, `t = refract((0, 0, -1), n, 1 / ior)`, meets that
plane after the length

    L = h / max(-t.z, 0.25)

Both effects follow this one ray:

- **Refraction.** Each tap displaces by `t.xy * L`, using its own normal
  (perturbed, then lifted as below), its own ior (chromatic aberration) and its own height
  (`h * (1 + anisotropic-blur * (i + r) / count)`, today's smear rule applied
  to `h`).
- **Attenuation.** Beer-Lambert takes `L` from the structural normal at
  `ior`: `att = attenuation-color ^ (L / attenuation-distance)`. Distortion is
  surface microstructure: it moves taps but does not lengthen the path, so face
  attenuation is today's exactly.

On the face, `n = (0, 0, 1)` gives `L = thickness` and no displacement,
today's values. Under distortion, face displacement grows by `1 / -t.z`. For
normals within 20 degrees that is 0.3 % at ior 1.28 and 2.8 % at ior 3.

**Tap normals.** Distortion and jelly ripple can tip the perturbed normal
past horizontal. With unit noise components that takes distortion above
about 0.93. A downward-facing normal sends the refracted ray sideways or
upward: reproducing today's noise formula gives `-t.z = 0.12` at ior 3, with
no aberration. Before refraction, each tap's normal is lifted into the
structural cap:

    nTap = normalize(vec3(n.xy, max(n.z, 0.05)))

This is continuous: a normalized `n` with `n.z = 0.05` is returned
unchanged. It leaves every normal at or above the cap untouched, so it
changes today's output only where the normal tipped past 87 degrees. With
`nTap.z >= 0.05`, the refracted ray always points downward, so no tap
receives an upward ray. The structural normal is already capped by the
slope policy, and attenuation uses it.

**Bound.** The refracted ray tilts from vertical by `theta_i - theta_t`,
which increases with the incidence angle. At grazing incidence,
`theta_t = asin(1 / ior)` and `-t.z = 1 / ior`. For any upward-facing
normal, therefore, `-t.z >= 1 / ior` and `L <= ior * h`. That covers the
structural normal and every lifted tap normal. With the cap at
`n.z >= 0.05`, the lowest value sits slightly above the bound: 0.81 at
ior 1.28, 0.38 at ior 3. The 0.25 floor binds at an effective index above 4. The two kinds of
ray reach different indices:

- **Attenuation, and the red or single tap**, use the configured `ior`,
  1 to 3, on upward normals (structural, or lifted), so the floor never binds
  and `L` is the true ray length.
- **Chromatic-aberration taps** use `ior * (1 + spread)` and
  `ior * (1 + 2 * spread)`, with `spread` up to `chromatic-aberration` (at
  most 1), so the effective index reaches 9. On steep normals the floor caps
  those taps at `L = 4 h`: at index 9 the capped rim has `-t.z = 0.16`. The
  floor is the stable-ray policy for aberration only.

Reference values at chamfer 12, ior 1.28 (`u = 0+` is the bevel side of the
face edge). These are the test vectors for the Rust mirror (section 6):

| thickness | k | u | h | n.z | L | displacement | today: path, displacement |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 6 (thin) | 1 | 0+ | 6.00 | 0.894 | 6.03 | 0.64 | 6.71, 0.64 |
| 6 | 1 | 0.9 | 0.60 | 0.894 | 0.60 | 0.06 | 6.71, 0.64 |
| 6 | 2 | 0.9 | 2.62 | 0.696 | 2.67 | 0.55 | 6.71, 0.64 |
| 12 (equal) | 1 | 0.5 | 6.00 | 0.707 | 6.12 | 1.22 | 16.97, 2.39 |
| 12 | 2 | 0.9 | 5.23 | 0.436 | 5.55 | 1.85 | 16.97, 2.39 |
| 31.2 (thick) | 1 | 1 | 19.20 | 0.707 | 19.59 | 3.89 | 44.12, 6.20 |
| 31.2 | 2 | 0.9 | 24.43 | 0.436 | 25.91 | 8.64 | 44.12, 6.20 |
| 31.2 | 2 | 1 | 19.20 | 0.050 | 23.69 | 13.87 | 44.12, 6.20 |
| 6 | 2 | 1 | 0.00 | 0.050 | 0.00 | 0.00 | 6.71, 0.64 |
| 12 | 2 | 1 | 0.00 | 0.050 | 0.00 | 0.00 | 16.97, 2.39 |
| 75.3 (inactive) | 2 | 0.9 | 68.53 | 0.436 | 72.69 | 24.24 | 106.49, 14.97 |
| 75.3 | 2 | 1 | 63.30 | 0.050 | 78.10 | 45.74 | 106.49, 14.97 |
| 12.1 (just above bevel) | 2 | 0.9 | 5.33 | 0.436 | 5.65 | 1.89 | 17.11, 2.41 |
| 12.1 | 2 | 1 | 0.10 | 0.050 | 0.12 | 0.07 | 17.11, 2.41 |

What the model guarantees:

- `L <= ior * h` everywhere, and `L = thickness` on the face. Today's
  bevel-wide `1.41 * thickness` plateau is gone.
- **The path is not monotonic in general.** It falls while the glass thins
  faster than the ray tilts. On thick glass over a short bevel, the tilt
  outgrows the height loss near the rim: at thickness 75.3, bevel 12, `k = 2`,
  the path runs 75.30, then 72.69 at `u = 0.9`, then 78.10 at the rim, 3.7 %
  above the face. On thin and equal glass it falls to zero at the rim.
- **Displacement is not monotonic in general either.** It is the product of
  the ray's sideways tilt, which grows toward the rim, and the remaining
  height, which falls. On a rounded bevel over a tall wall, the tilt wins and
  the image compresses toward the rim: the lens look of a bullnose edge
  (31.2: 3.64, then 8.64, then 13.87). Where little or no wall remains,
  height wins near the rim and the displacement falls back. At 12.1, just
  above the bevel, it drops from 1.89 at `u = 0.9` to 0.07 at the rim; at or
  below the bevel it reaches 0.

`slabSurface` gains `h`, `u` and `w` as outputs. The ring's spill switches
from `innerDist / slabChamfer` to `u`: identical at rest, and under jelly the
spill now reaches the silhouette on every side.

`bevel-profile` is a core parameter, `FloatOrInt<1, 8>`, default 1. Prism
key: `glass.bevelProfile`.

### 3.2 Energy-conserving Fresnel and scene reflection

Today the glint is added on top of full transmission. The composition
becomes

    glass = (1 - F) * transmitted + within + specular + emissive

with `F` the existing Schlick term on the structural normal. At the rim of a
rounded bevel `F` rises to 0.777 (ior 1.28, at the slope cap; 1 in the
analytic limit), so the edge shows mostly what it reflects rather than a
tinted backdrop. `within` is not scaled by `1 - F`: it is light already
inside the glass, and its exit through the surface is part of the ring and
aurora gains, which stay as tuned.

**Interior light attenuation.** Ring and aurora keep their `pow(att, 0.2)`
factor, applied to the new `att`. Interior light crosses the same slab, so it
follows the same ray, a fifth of the path. This is an intended change, and
it is confined to the bevel:

- on the face, where the ring band runs, `L = thickness` and the factor is
  today's exactly;
- on the bevel, spill and aurora brighten where the glass thins. Example:
  thickness = chamfer = 12, `u = 0.5`, `k = 1`, attenuation 0.1 at distance
  10: the factor rises from 0.458 to 0.754.

Rejected alternative: keeping today's path for interior light only. That
gives one slab two path models, a compatibility layer under another name.

What a pane edge reflects is the room around it. The nearest stand-in the
compositor has is the scene just beyond the window. The `reflection` optic
adds

    F * wr * srgbToLinear(sampleBackground(v + gPerturbed * reach / area))

where:

- `reach = -outerDist + thickness`: sample just beyond the silhouette,
  outward, one thickness further;
- `gPerturbed = normalize(normalize(grad_u) + nPerturbed.xy - nStructural.xy)`:
  the across-bevel direction from 3.1, which is continuous and nonzero on the
  bevel even where `n.xy` vanishes (`k > 1` at the face join), plus the
  perturbation, so distortion and jelly ripple move the reflection;
- `wr = reflection * smoothstep(0, 0.1, u)`: zero on the face, where the
  direction is undefined, and continuous for every `k`;
- the sample goes through the existing prefilter binding, so roughness
  softens the reflection as it softens refraction, and a smooth pane reflects
  sharply.

This reflection is not tinted by the attenuation color: reflection happens at
the surface, before light enters the glass. That is the property that keeps a
dark edge clear. It still belongs to the scene, because it carries the
wallpaper's colours.

The existing white glint `F * (0.15 + 0.85 * facing)` stays as the key-light
part of the environment. `reflection` is `FloatOrInt<0, 1>`, default 0, and
at 0 the optic returns its input. Prism key: `glass.reflection`.

### 3.3 Edge highlight

The `edge-highlight` optic adds a light reflection lobe that lands where the
bevel's normal bisects the view and the key light:

    L     = normalize(vec3(normalize(mat_sig_light.xy), 1))   // 45 degree elevation
    H     = normalize(L + vec3(0, 0, 1))                      // 22.5 degrees off vertical
    tilt  = smoothstep(0, 1, (1 - nStructural.z) / (1 - cos(22.5 degrees)))
    highlight = edge-highlight * tilt * D(max(dot(nPerturbed, H), 0)) / D(1)

`D` is GGX (Trowbridge-Reitz) with `alpha = mix(0.04, 0.5, roughness)`,
normalized to peak 1, so the parameter is peak added linear brightness. The
lobe stands for a bright source whose reflected radiance survives `F`, so it
is not multiplied by `F`; the gain is art-directed like the glint. It uses
`mat_sig_light`, so `attention "rim-orbit"` sways it with the glint.

`tilt` is the transition. It is 0 where the structural surface is flat and
reaches 1 where it tilts as far as `H`. It carries the profile's continuity:

- **`k > 1`.** `n.z -> 1` at the face join, so the highlight falls to zero
  there continuously at every roughness. Without `tilt`, roughness 1 and
  gain 0.5 would add 0.24 there, beside an unlit face.
- **`k = 1`.** A planar facet has one normal, so its highlight is uniform
  across the facet and steps at the crease, as the geometry does. Planar
  facets do flash, intentionally: this is the cut-glass facet catching the
  light. A facet whose slope equals `H`'s (`R / chamfer = tan 22.5 degrees =
  0.414`) lights at full gain on its light-facing side. A 45 degree facet
  receives `D(cos 22.5 degrees) / D(1)`: 0.0001 of the gain at roughness 0,
  0.48 at roughness 1.
- **Far sides.** `n.H` is clamped at 0. A far-side facet is lit only by the
  rough tail of the lobe, and its brightness is bounded by `D` at that angle.

`FloatOrInt<0, 1>`, default 0; neutral at 0. Prism key:
`glass.edgeHighlight`.

### 3.4 Specular hook interface

`reflection` needs the fragment position, both normals, `F`, `u` and the
across-bevel direction;
`edge-highlight` needs the perturbed normal. The specular hook changes from
`vec3 <name>_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)`
to

    struct Surface { vec2 p; vec2 v; vec3 structural; vec3 perturbed;
                     float cosine; float fresnel; float across; vec2 acrossDir;
                     float outerDist; };

`across` is `u`, and `acrossDir` is `normalize(grad_u)` from 3.1.
    vec3 <name>_specular(vec3 specular, Surface s)

`iridescence` migrates to it unchanged in behaviour. `OPTICS` order within
the specular stage: `reflection`, `edge-highlight`, `iridescence`, so
thin-film hue colours reflections and highlights as it colours the glint.
The accent mix follows, as today. `adding-an-optic.md` and
`render-pipeline.md` (stages 1, 3, 4, 6 and the parameter table) are updated
in the same change.

## 4. What changes at default values

Defaults (`bevel-profile 1`, `reflection 0`, `edge-highlight 0`) do not
reproduce today's pixels. The intended corrections:

- **Face, transmitted:** scaled by `1 - f0`: 1.5 % darker at ior 1.28, 4 % at
  1.5. Face attenuation and the ring band's interior factor are exact.
- **Face, distortion:** displacement grows by `1 / -t.z`. For normals within
  20 degrees that is 0.3 % at ior 1.28 and 2.8 % at ior 3.
- **Bevel corners at rest:** the softened outer gradient moves the normal by
  at most 0.03, within about a pixel of the arc-to-side junctions.
- **Bevel:** path and displacement follow the ray model (table in 3.1). A
  planar chamfer's tint lightens and its shift shrinks toward the silhouette.
  The largest change is at `thickness <= bevel`.
- **Bevel, interior light:** spill and aurora brighten where the glass thins
  (3.2). Under jelly, the spill reaches the silhouette on every side.

Pixels outside the slab, and opaque window pixels, are unchanged. No
compatibility switch restores the old bevel.

## 5. Cost

Each bevel fragment gains a second rounded-box gradient, a `pow` pair for
the profile and one `refract` for the attenuation path. With the optics on,
it gains one background sample (reflection) and a GGX evaluation; both are
skipped at their neutral values and on the face. Face fragments gain one
multiply. No new textures, passes or redraw clocks.

## 6. Verification

- **Config:** parse, default and bound tests for the three parameters.
  `material_parameter_specs_match_the_parser` covers them through their
  `ParamSpec`s. Docs tables are updated where Rust tests read them.
- **Rust mirror.** `src/render_helpers/material/bevel.rs` holds the
  coordinate, profile, normal, ray path and highlight weight in f64. A test
  checks that the shader's lines match it, as `ring.rs` does for the beam.
  Its tests cover:
  - the section 3.1 table, within 0.01, including the thick-glass
    non-monotonic rows and the thin and equal rim rows;
  - `-t.z >= 1 / ior` for upward normals at ior 1, 1.28 and 3, and the 0.25
    floor binding only at aberration indices above 4;
  - `k = 1` at rest reproducing today's normal exactly on straight sides, and
    within 0.03 near the corner junctions;
  - normal continuity through deformed corners: outer 112, inner 88,
    chamfer 12, inner radius 0 and 8, 3 px diagonal and axial shifts, `k = 1`
    and `k = 2`. The maximum jump between neighbours falls in proportion to the
    step, from 0.02 px to 0.005 px; the exact gradient fails this at radius 0;
  - the same test across both centerlines of a maximum-radius stadium (outer
    half-size (10, 100), chamfer 6, inner radius 4), at rest and with a 2 px
    shift;
  - an fp32 build of the softened gradient against f64 over half-sizes 2 to
    4000 px and every radius. It must have no non-finite value, agree within
    0.001 rad, and trip the ridge guard only on the parallel-side ridge;
  - lifted tap normals: continuity at `n.z = 0.05`, normals above the cap
    unchanged, and `-t.z >= 1 / ior` for lifted normals at ior 1, 1.28 and 3;
  - `u` exactly 0 at the face and 1 at the silhouette on both opposite sides
    under a 3 px face shift at chamfer 12, and under a jelly resize, with `u`
    strictly increasing across the band (no plateau, no truncated rim);
  - the highlight weight tending to 0 at the face join for `k = 2`, at
    roughness 0 and 1;
  - a planar facet with `R / chamfer = 0.414` facing the light at full
    gain;
  - a 45 degree facet at 0.0001 and 0.48 of the gain at roughness 0 and 1.
- **Intended-change evidence (step 1).** Before/after captures on the
  headless Weston host at default glass (`glass-optic-smoke-lib.sh`), with a
  fully transparent client, each build captured with ring and aurora off and
  with both on:
  - outside the slab, decoded-identical;
  - off: face pixels match the before pixel re-derived under `(1 - f0)`
    scaling of transmitted light (the face's specular is the constant
    `0.15 * f0`), within one code value;
  - on: face pixels, ring band included, match
    `(1 - f0) * T + S + W`, with `T + S` from the off capture and
    `W = before_on - before_off` in linear light, within one code value. This
    shows the ring's face attenuation is unchanged;
  - bevel differences are reported, not asserted.
- **Motion.** The jelly motion companion
  (`2026-09-11-jelly-motion-sweep.md`) at `bevel-profile 2`, mid-move and
  mid-resize frames. On the leading and the trailing side, the brightness
  profile across the bevel ROI runs from the face to the silhouette with no
  flat run before the outline. Settled frames are captured as well.
- **Refactor and neutrality evidence:** the step 2 build is decoded-identical
  to step 1 across the iridescence smoke. At default values, the step 3 and
  step 4 builds are decoded-identical to step 2.
- **Visual judgement (user):** a contact sheet from
  `glass-parameter-sweep.sh`, with the bevel ROI crop: `bevel-profile` in
  {1, 2, 4}, crossed with {live focused material, live inactive material,
  weak tint on thin glass}, each with `reflection` 0 and 0.6 and
  `edge-highlight` 0 and 0.5 at roughness 0 and 1. A thin-chamfer facet row
  (`R / chamfer = 0.414`) shows the planar flash. This is the review gate for
  the look and for Prism's starting values.

## 7. Delivery

1. niri: two-boundary coordinate, height-field bevel and `bevel-profile`, the
   ray model for refraction and attenuation, `(1 - F)` composition, spill on
   `u`; Rust mirror; intended-change and motion evidence.
2. niri: `Surface` specular hook; `iridescence` migration.
3. niri: `reflection` optic.
4. niri: `edge-highlight` optic; contact sheet for review.
5. Prism: `glass.bevelProfile`, `glass.reflection`, `glass.edgeHighlight`
   keys and starting values chosen from the sheet (a cross-project piece
   filed at planning).

## 8. Out of scope

- Total internal reflection lines (the dark and bright double line on thick
  glass walls). Candidate follow-up once the rounded profile exists to host
  it.
- Caustic brightening of the backdrop just outside the edge.
- Content in the glass and shared film (`material-7f5751`), which will reuse
  the local height `h` from 3.1.
- Per-side profiles or asymmetric bevels.
