# Glass edge optics

**Status:** draft for review, 2026-09-30. Task: `material-be611b`.
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

Let `u = clamp(innerDist / chamfer, 0, 1)`, 0 at the face edge and 1 at the
silhouette, and `R = min(chamfer, thickness)` the rise. The bevel drops from
the face by

    drop(u) = R * (1 - (1 - u^k)^(1/k))

with `k = bevel-profile`. `k = 1` is today's planar chamfer. `k = 2` is a
circular quarter-round, and larger `k` a squircle that stays flat longer and
rolls off harder. For `k > 1` the slope is 0 at the face, so the face joins
the bevel without a crease, and vertical at the silhouette, so Fresnel
reaches 1 there.

- **Normal.** `n = normalize(vec3(g * s, 1))` with `g` the inner-face outward
  gradient, as today, and `s = drop'(u) / chamfer`. `s` is capped at 20
  (normal z of about 0.05): the analytic slope is infinite at `u = 1`.
- **Local height.** `h = thickness - drop(u)` on the bevel and `thickness` on
  the face. At `k = 1` with `thickness <= bevel` the glass thins to zero at the
  silhouette. With `thickness > bevel` a vertical wall of `thickness - R`
  remains, as today.
- **Refraction** displaces by `refract(...).xy * h`, and the anisotropic smear
  scales from `h`. Near the rim the image of the window's interior is drawn
  outward and compressed, which is the lens look of a rounded edge. At `k = 1`
  the displacement now falls linearly to `thickness - R` rather than holding
  constant.
- **Attenuation** uses `h / max(cos, 0.25)`. The tint thins with the glass
  instead of peaking on the bevel.

`slabSurface` gains `h` and `u` as outputs. Ring and aurora keep their
existing depth of `0.2 * thickness`: the ring lands near the face, where
`h = thickness`, and the spill already fades with `u`. The ring's tuning is
not reopened.

`bevel-profile` is a core parameter, `FloatOrInt<1, 8>`, default 1. Prism
key: `glass.bevelProfile`.

### 3.2 Energy-conserving Fresnel and scene reflection

Today the glint is added on top of full transmission. The composition
becomes

    glass = (1 - F) * transmitted + within + specular + emissive

with `F` the existing Schlick term on the structural normal. `within` (ring,
aurora) stays unscaled so the ring's tuning holds. At the rim of a rounded
bevel `F -> 1`, so the edge shows what it reflects rather than a tinted
backdrop.

What a pane edge reflects is the room around it. The nearest stand-in the
compositor has is the scene just beyond the window. The `reflection` optic
adds

    F * w * srgbToLinear(sampleBackground(v + gPerturbed * reach / area))

where:

- `reach = -outerDist + thickness`: sample just beyond the silhouette,
  outward, one thickness further;
- `gPerturbed = normalize(n.xy)` from the perturbed normal, so distortion and
  jelly ripple move the reflection;
- `w = reflection * smoothstep(0, 0.1, u)`, zero on the face so the
  undefined face direction never produces a seam;
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

    L = normalize(vec3(normalize(mat_sig_light.xy), 1))   // 45 degree elevation
    H = normalize(L + vec3(0, 0, 1))
    highlight = edge-highlight * D(dot(nPerturbed, H)) / D(1)

`D` is GGX with `alpha = mix(0.04, 0.5, roughness)`, normalized to peak 1, so
the parameter is peak added linear brightness. On a rounded bevel this draws
a thin line along the light-facing sides and nothing on the far sides. On a
planar chamfer the normal never reaches `H` (`n.H = 0.92`), so the line
appears only as `k` rises: the highlight is a property of rounded edges, as
in the world. It uses `mat_sig_light`, so `attention "rim-orbit"` sways it
with the glint. It is not multiplied by `F`: the lobe stands for a bright
source whose reflected radiance survives `F`, and the gain is art-directed
like the glint. `FloatOrInt<0, 1>`, default 0; neutral at 0. It is gated to
the bevel (`u > 0`). Prism key: `glass.edgeHighlight`.

### 3.4 Specular hook interface

`reflection` needs the fragment position, both normals, `F` and `u`;
`edge-highlight` needs the perturbed normal. The specular hook changes from
`vec3 <name>_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)`
to

    struct Surface { vec2 p; vec2 v; vec3 structural; vec3 perturbed;
                     float cosine; float fresnel; float across; float outerDist; };
    vec3 <name>_specular(vec3 specular, Surface s)

`iridescence` migrates to it unchanged in behaviour. `OPTICS` order within
the specular stage: `reflection`, `edge-highlight`, `iridescence`, so
thin-film hue colours reflections and highlights as it colours the glint.
The accent mix follows, as today. `adding-an-optic.md` and
`render-pipeline.md` (stages 1, 3, 4, 6 and the parameter table) are updated
in the same change.

## 4. What changes at default values

Defaults (`bevel-profile 1`, `reflection 0`, `edge-highlight 0`) do not
reproduce today's pixels. There are two intended corrections:

- **Face:** transmitted light is scaled by `1 - f0`: 1.5 % darker at ior 1.28,
  4 % at 1.5.
- **Bevel:** refraction displacement and attenuation path use the local
  height, so a planar chamfer's tint lightens and its shift shrinks toward
  the silhouette. The largest change is at `thickness <= bevel`.

Pixels outside the slab, and opaque window pixels, are unchanged. No
compatibility switch restores the old bevel.

## 5. Cost

Bevel fragments gain one background sample (reflection) and a GGX
evaluation. Both are skipped at their neutral values and on the face. The
height-field adds a `pow` pair per bevel fragment. Face fragments gain one
multiply. No new textures, passes or redraw clocks.

## 6. Verification

- **Config:** parse, default and bound tests for the three parameters.
  `material_parameter_specs_match_the_parser` covers them through their
  `ParamSpec`s. Docs tables are updated where Rust tests read them.
- **Intended-change evidence (step 1):** before/after captures on the
  headless Weston host at default glass (`glass-optic-smoke-lib.sh`), with
  the ring and aurora off and a fully transparent client. Pixels outside the
  slab are decoded-identical. Face pixels match the before pixel re-derived
  under `(1 - f0)` scaling of transmitted light (the face's specular is the
  constant `0.15 * f0`), within one code value. Bevel pixels differ.
- **Refactor and neutrality evidence:** the step 2 build is decoded-identical
  to step 1 across the iridescence smoke. At default values, the step 3 and
  step 4 builds are decoded-identical to step 2.
- **Visual judgement (user):** a contact sheet from
  `glass-parameter-sweep.sh`, with the bevel ROI crop: `bevel-profile` in
  {1, 2, 4}, crossed with {live focused material, live inactive material,
  weak tint on thin glass}, each with `reflection` 0 and 0.6 and
  `edge-highlight` 0 and 0.5. This is the review gate for the look and for
  Prism's starting values.

## 7. Delivery

1. niri: height-field bevel and `bevel-profile`, local height in refraction
   and attenuation, `(1 - F)` composition; neutral-change evidence.
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
