# Material render pipeline

What runs in what order when a glass window is drawn, what each stage
samples, and which parameter acts at which stage. Read this before touching
`src/render_helpers/material/mod.rs`, `src/render_helpers/material/optics/`,
`src/render_helpers/shaders/material/prelude.frag`,
`src/render_helpers/shaders/material/main.frag`, and one shader file per optic,
or the postprocess and effect-buffer code; it is the frame every rendering
design in this directory assumes.

Sources: `src/niri.rs` (`fill_xray_elements`, `update_xray_render_elements`),
`src/render_helpers/effect_buffer.rs`, `src/render_helpers/blur.rs`,
`src/layout/tile.rs` (`Tile::render_inner`, `resolve_material`),
`src/render_helpers/material/mod.rs`, `src/render_helpers/material/optics/`,
`src/render_helpers/shaders/material/prelude.frag`,
`src/render_helpers/shaders/material/main.frag`, and one shader file per optic,
`src/render_helpers/background_effect.rs`, `src/render_helpers/xray.rs`,
`src/render_helpers/framebuffer_effect.rs`,
`src/render_helpers/shaders/postprocess.frag`.

## 1. The frame, top down

Render element lists are front to back: the first element pushed is the
topmost. Per output, per frame:

1. **Backdrop buffers are filled** (`niri.rs`). Each output owns an `Xray`
   with two `EffectBuffer`s per render target (output, screencast, capture):
   `background`, the Background layer surfaces as they appear inside a
   workspace, and `backdrop`, the same layer drawn as the overview backdrop.
   Both receive the global `blur { passes; offset }` as their blur options.
2. **Each `EffectBuffer` renders lazily** (`effect_buffer.rs`): its elements
   into a full-output offscreen (the *sharp* texture); on request, a
   dual-Kawase blur of that texture (`blur_down.frag` / `blur_up.frag`,
   `blur.rs`), cached until the source or the blur options change; on
   request, a downsample pyramid of either texture for roughness.
3. **Tiles render** (`Tile::render_inner`). Push order, top to bottom:
   popups; the window body (material element, resize element, or the plain
   clipped window); the fullscreen backdrop; the border; the focus ring; the
   shadow; the window's `background-effect` elements.
4. **Below the tiles**: Bottom and Background layer surfaces per workspace,
   the workspace background color, the workspace shadows, then the output
   backdrop.

Two elements in step 3 carry effects, and they are different programs:

| Element | Program | What it draws |
| --- | --- | --- |
| Material element | `material.frag` | The window body composed over the glass slab. Everything in section 2. |
| Background-effect element | `postprocess_and_clip` (`clipped_surface.frag` + `postprocess.frag`) | The sampled backdrop behind the window, with `blur { noise; saturation }` or the window rule's `background-effect` overrides, drawn *under* the window body. |

Prism's terminal window rule writes
`background-effect { blur false; noise 0; saturation 1 }`, so for Prism-managed windows the second element is
inert and every visible effect comes from the material element.

## 2. The material element

`resolve_material` (`tile.rs`) resolves the window rule's material once per
frame, settles the `backdrop-blur && !blur.off` source gate, and attaches the
selected response block. The noise and saturation optics own their
inherit-or-neutral rules and read the effective gate and global `blur` block
through `OpticFrame`.

The window body (the client surfaces, corner clipped, or the resize
crossfade) is first rendered into the tile's offscreen. The material element
then binds five textures: that offscreen (`niri_tex_win`), and a low and
high prefilter level of both the background and the backdrop buffer.
`render_prefiltered` picks the sharp or blurred source from the gate above,
then `roughness * clamp(ior * 2 - 2, 0, 1)` picks the pyramid levels and the
mix between them. Roughness zero binds level zero twice and samples once.

Uniforms carry the slab frame, the corner radius, the jelly state, the
signal state, and every glass parameter.

The program is assembled at compile time from `prelude.frag`, each optic's
GLSL in `OPTICS` order (`src/render_helpers/material/optics/mod.rs`), and
`main.frag`. Each optic appends its uniforms and uploads their values through
`Optic::values`; an animated optic's `next_change` joins the tile's redraw
deadline.

## 3. Inside `material.frag`, per fragment

The order below is the order in the shader. Every step after the first
runs only where the window is transparent or outside the window.

| # | Stage | Samples | Parameters |
| --- | --- | --- | --- |
| 0 | **Window sample.** Read the offscreen inside the window rect. An opaque pixel returns immediately, untouched. | `niri_tex_win` | — |
| 1 | **Slab geometry** (`slabSurface`). Signed distance to the slab silhouette gives anti-aliased coverage and the structural normal: flat on the face, sloped on the chamfer with run `bevel` and rise `min(bevel, thickness)`. The inner face (chamfer's inner edge) trails the jelly shear and resize. | — | `bevel`, `offset-x`, `offset-y`, `thickness`, window corner radius, `jelly-flex` (through the move and resize residuals) |
| 2 | **Normal perturbation.** Fractal simplex noise on element position bends the normal; then, while a spring runs, two-octave simplex ripple scaled by jelly activity. | — | `distortion`, `distortion scale=`, `jelly-ripple` |
| 3 | **Refraction taps** (`tap`). Refract the orthographic ray at the perturbed normal with `ior`, displace by `thickness`, and sample the composed background there, in linear light. Composition per tap: background buffer over the workspace color, over the backdrop buffer over the backdrop color; outside the workspace rect, the backdrop alone. One tap when both smear controls are zero; otherwise up to eight jittered taps, per channel under aberration. | `niri_tex_bg[_high]`, `niri_tex_backdrop[_high]` | `ior`, `thickness`, `backdrop-blur`, `roughness`, `anisotropic-blur`, `chromatic-aberration` |
| 4 | **Beer-Lambert attenuation.** `attenuation-color ^ (optical distance / attenuation-distance)`, where optical distance is `thickness / cos(structural normal)`, floored at a quarter, so the chamfer tints more than the face. | — | `attenuation-color`, `attenuation-distance`, `thickness` |
| 5 | **Fresnel glint.** Schlick from `ior` on the structural normal, weighted toward the signal light direction; accent-tinted under `attention "rim-orbit"`. Additive. Then the `iridescence` optic hues the glint from the view angle, before the accent mix. | — | `ior`, `iridescence` (optic `iridescence`; neutral 0), response `attention`, signal accent |
| 6 | **Emissive: ring of light.** Only inside the bevel mask (`smoothstep(0, 1, inner distance)`, zero on the face). The band's *sampling position* is refracted through the light-path index `1 + (ior - 1) * light-ior` to 60 % of the thickness, capped at half `ring-inset`; the band is a Gaussian at `ring-inset` of width `ring-width` plus a halo, per channel under aberration, times `att ^ 0.2`. Focus glow drifts around the perimeter; accent glow follows signal level and breath. Additive. Then the `aurora` optic adds its colour field, weighted by the same `att ^ 0.2`. | — | `light-ior`, `ring-inset`, `ring-width`, `ring-color`, `ring-drift-hz`, response `focus` / `accent` / `attention`, `aurora`, `aurora drift-hz`, `aurora color` (optic `aurora`; neutral 0) |
| 7 | **Emissive: sweeps.** A diagonal Gaussian sweep per impulse whose response is `sweep`. The other impulse responses act earlier: `ripple` adds to the jelly activity of step 2, `flash` raises aberration and distortion for steps 2 and 3. Additive. | — | response `ping` / `done` / `error` |
| 8 | **Encode.** `glass = linearToSrgb(transmitted + specular + emissive)`. | — | — |
| 9 | **Saturation** (optic `saturation`). `mix(luma(glass), glass, saturation)` in sRGB. | — | `saturation`; neutral 1 |
| 10 | **Noise** (optic `noise`). `white` adds one hash value per screen pixel; `fine` adds high-pass achromatic hash grain; `lightness` applies that fine grain to Oklab L. All operate in the finished sRGB glass color. | — | `noise`, `noise type=`; neutral 0 |
| 11 | **Coverage.** Multiply by the slab coverage from step 1; the result is premultiplied. | — | — |
| 12 | **Composite.** `out = win + (1 - win.a) * glass`, then `* niri_alpha` (window-rule opacity, applied exactly once). | — | window-rule `opacity` |

Steps 2 to 7 all act on or through the *normal* and the *sampled
background*; they never read window pixels. Steps 9 and 10 act on the
finished glass color; they never read the background either.

Stages 2, 5, 6, and 9–10 are the four optic hooks `normal`, `specular`,
`emissive`, and `post`; see `adding-an-optic.md`.

## 4. What the order settles

- **Blur is upstream of everything.** `backdrop-blur` chooses the image the
  glass refracts; `roughness` prefilters that image. Neither is a pass over
  the glass. A frosted look is a refraction of a blurred backdrop, not a
  blur of a refraction.
- **Noise and saturation are the last color operations.** They apply to the
  face, the chamfer, the Fresnel glint, and the ring alike, after
  attenuation and after emissive terms. Grain on the bevel or on the ring
  is a consequence of step 10's position, not of the noise itself. Moving
  noise before step 3 (onto the sampled background) would leave the edge
  optics and the ring clean and make the grain refract with the image;
  masking it by inner distance would leave the face grainy and the bevel
  clean.
- **The ring is emissive, not refracted content.** Only where the band is
  *looked up* is refracted (step 6); its light is added after attenuation
  and is confined to the bevel by the mask. To read as embedded in the
  glass it would have to be sampled as part of the background in step 3, or
  the mask and landing depth (a constant 0.6 of thickness) would have to
  change. Today it competes with the chamfer because it lives only there.
- **Distortion and ripple never reach the window.** They perturb the normal
  before the taps, so a flat opaque terminal shows them only through its
  transparency and in the slab band outside it.
- **Opaque pixels bypass the whole shader.** Every effect is visible only
  through window transparency and in the band the slab extends past the
  window. A window with `opacity 1` and an opaque surface shows glass only
  on its bevel.
- **Two noise sites exist.** `postprocess.frag` grains the background-effect
  element under the window; `material.frag` grains the glass. Prism pins the
  first to neutral. A design that changes "the noise" must say which.

## 5. Parameter to stage, with Prism names

| Native | Prism key | Stage |
| --- | --- | --- |
| `bevel` | `glass.paneLip` (+ max pane shift) | 1 |
| `offset-x` / `offset-y` | `glass.paneShiftX` / `glass.paneShiftY` | 1 |
| `thickness` | `glass.thickness` | 1, 3, 4 |
| `jelly-flex` | `glass.jellyFlex` | 1 |
| `jelly-ripple` | `glass.jellyRipple` | 2 |
| `distortion`, `scale=` | `glass.distortion`, `glass.distortionScale` | 2 |
| `ior` | `glass.ior` | 3, 5, 6 |
| `backdrop-blur` | `glass.backdropBlur` | source selection before 3 |
| `roughness` | `glass.roughness` | source selection before 3 |
| `anisotropic-blur` | `glass.anisotropicBlur` | 3 |
| `chromatic-aberration` | `glass.chromaticAberration` | 3, 6 |
| `iridescence` | (pending, prism-763054) | 5 |
| `aurora`, `drift-hz`, `color` × 2 | (pending, prism-763054) | 6 |
| `attenuation-color` | `glass.attenuationColor` | 4 |
| `attenuation-distance` | `glass.attenuationDistance` | 4 |
| `light-ior` | (pending, prism-0ea68f) | 6 |
| `ring-inset`, `ring-width`, `ring-color`, `ring-drift-hz` | (pending, prism-28e29c) | 6 |
| `saturation` | `glass.saturation` | 9 |
| `noise` `type=` | `glass.noise`; `type=` (pending, prism-51f23b) | 10 |

Prism's `glass.inactive.*` keys write the same native parameters into the
unfocused material definition; `glass.focusSplit` decides whether that
second definition exists.

## Related designs

- `2026-08-22-v1-design.md` §2 and §4: the compositing contract and the
  slab.
- `2026-08-29-material-backdrop-blur-design.md`: why blur is a source
  selection, not a pass.
- `2026-09-01-material-roughness-design.md`: the prefilter pyramids.
- `../specs/2026-09-02-material-noise-saturation-design.md` and
  `../specs/2026-09-05-material-glass-noise-saturation-params-design.md`:
  the position of steps 9 and 10 and the inherit rule.
- `../specs/2026-09-05-ring-light-focus-response-design.md`: step 6.
- `2026-09-02-material-signals-design.md`: the signal inputs to steps 5
  to 7.
