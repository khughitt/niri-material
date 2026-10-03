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
`Optic::values` using shared logical time. Each optic's `next_change` returns
a logical deadline; the registry suppresses it while input is idle and maps
the earliest one to real time when active. Aurora uniforms hold through idle
client or backdrop damage and resume from the held phase. Attention keeps its
separate absolute-clock deadline and resume behavior.

The focus beam is one run on the animation loop — the head's lap plus its
tail, `(perimeter + tail) / ring-beam-speed` long, or `ring-beam-decay /
ring-beam-speed` when the decay darkens the comet first, ended by the frame
whose geometry sees the tail clear or the comet dark — reported through `are_animations_ongoing`
while the band is in view; it has no bucket clock and no deadline. The
`ring-beam-noise` wander on the head's brightness is computed per frame
inside that run and adds no clock of its own: it multiplies the head
amplitude the run already sends, and is exactly absent once the head's lap
is over. It is not
a layout transition: `are_transitions_ongoing`, which also gates the
pointer-focus refresh in `Niri::refresh_pointer_contents`, does not report it. Settled
focus reports no deadline and a constant fingerprint. Sustained attention motion keeps its bucket deadline
(`Tile::tick_deadline`) while the band is in view.

## 3. Inside `material.frag`, per fragment

The order below is the order in the shader. Every step after the first
runs only where the window is transparent or outside the window.

| # | Stage | Samples | Parameters |
| --- | --- | --- | --- |
| 0 | **Window sample.** Read the offscreen inside the window rect. An opaque pixel returns immediately, untouched. | `niri_tex_win` | — |
| 1 | **Slab geometry** (`slabSurface`). Signed distance to the slab silhouette gives anti-aliased coverage. The bevel is a height field between two boundaries, the fixed silhouette and the face that trails the jelly shear and resize: `u = innerDist / (innerDist - outerDist)` runs from 0 at the face edge to 1 at the silhouette on every side, the local height is `thickness - R * f(u)` with `R = min(bevel, thickness)` and `f(u) = 1 - (1 - u^k)^(1/k)`, and the structural normal comes from that height field, its slope capped at 20. `k = 1` is a planar chamfer. | — | `bevel`, `bevel-profile`, `offset-x`, `offset-y`, `thickness`, window corner radius, `jelly-flex` (through the move and resize residuals) |
| 2 | **Normal perturbation.** Fractal simplex noise on element position bends the normal; then, while a spring runs, two-octave simplex ripple scaled by jelly activity. | — | `distortion`, `distortion scale=`, `jelly-ripple` |
| 3 | **Refraction taps** (`tap`). Refract the orthographic ray at the perturbed normal, lifted to `n.z >= 0.05`, with `ior`, follow it to the backdrop plane under the local height (`L = h / max(-t.z, 0.25)`), and sample the composed background there, in linear light. Composition per tap: background buffer over the workspace color, over the backdrop buffer over the backdrop color; outside the workspace rect, the backdrop alone. One tap when both smear controls are zero; otherwise up to eight jittered taps, per channel under aberration. | `niri_tex_bg[_high]`, `niri_tex_backdrop[_high]` | `ior`, `thickness`, `backdrop-blur`, `roughness`, `anisotropic-blur`, `chromatic-aberration` |
| 3a | **Behind: saturation.** Encode the averaged linear sample, apply `mix(luma(encoded), encoded, saturation)`, then decode. Neutral returns before conversion. | — | `saturation`; neutral 1 |
| 3b | **Behind: noise.** Grain the averaged backdrop once, using the existing sRGB white/fine formulas or Oklab lightness formula, then return linear light. Grain stays screen-seeded; lightness keeps its gamut clamp. | — | `noise`, `noise type=`; neutral 0 |
| 4 | **Beer-Lambert attenuation.** `attenuation-color ^ (L / attenuation-distance)`, where `L` is the structural ray's path under the local height: the face's path is `thickness`; the bevel thins toward the silhouette. Transmitted light is then scaled by `1 - F` (Schlick, stage 6). | — | `attenuation-color`, `attenuation-distance`, `thickness` |
| 5 | **Within: ring and aurora.** The ring beam lands through the light-path index at 20 % of thickness, measured from the face edge (`ring-gap` inward from where the flat face begins, on the face globals `slabSurface` stores); its brightness along the band is an arc-length comet — a Gaussian head at `arcPosition`, whose amplitude carries the `ring-beam-noise` wander, a tail behind it, both darkened together over `ring-beam-decay` px — over a resting glow scaled by `ring-rest`; edge spill on the bevel, fading from the face edge to the silhouette along `u`. Aurora lands the same way. Each contributes `att ^ 0.2`. The ring has no face mask; roughness scatters its Gaussian core and halo with integral conservation. | — | `light-ior`, `roughness`, `ring-gap`, `ring-width`, `ring-color`, `ring-beam-speed`, `ring-beam-noise`, `ring-beam-noise-hz`, `ring-beam-decay`, `ring-glow`, `ring-rest`, `ring-accent`, response `focus` / `accent` / `attention`, `aurora`, `aurora drift-hz`, `aurora color` (optic `aurora`; neutral 0) |
| 6 | **Fresnel glint.** Schlick from `ior` on the structural normal, weighted toward the signal light direction; accent-tinted under `attention "rim-orbit"`. Additive. Then the `iridescence` optic hues the glint from the view angle, before the accent mix. Specular hooks take the fragment's `Surface` (position, both normals, cosine, Fresnel, `u` and its direction, silhouette distance). | — | `ior`, `iridescence` (optic `iridescence`; neutral 0), response `attention`, signal accent |
| 7 | **Emissive: sweeps.** A diagonal Gaussian sweep per impulse whose response is `sweep`. The other impulse responses act earlier: `ripple` adds to the jelly activity of step 2, `flash` raises aberration and distortion for steps 2 and 3. Additive. | — | response `ping` / `done` / `error` |
| 8 | **Encode.** `glass = linearToSrgb((1 - F) * sampled * att + within + specular + emissive)`. | — | — |
| 9 | **Post (film).** Reserved for screen-space effects on encoded glass; currently empty. | — | — |
| 10 | **Coverage.** Multiply by the slab coverage from step 1; the result is premultiplied. | — | — |
| 11 | **Composite.** `out = win + (1 - win.a) * glass`, then `* niri_alpha` (window-rule opacity, applied exactly once). | — | window-rule `opacity` |

Stages 2 to 7 operate on the normal, sampled backdrop, or additive light;
they never read window pixels. The six hook sites are `normal`, `behind`,
`within`, `specular`, `emissive`, and `post`; see `adding-an-optic.md`.

## 4. What the order settles

- **Blur is upstream of everything.** `backdrop-blur` chooses the image the
  glass refracts; `roughness` prefilters that image. Neither is a pass over
  the glass. A frosted look is a refraction of a blurred backdrop, not a
  blur of a refraction.
- **Noise and saturation belong to the transmitted backdrop.** They run
  once after the averaged taps and before attenuation. The chamfer can still
  transmit grain, attenuated more strongly than on the face. Glint, ring,
  aurora and sweeps are added afterward, so their light is not grained or
  desaturated. Grain remains screen-seeded; it does not refract with the image.
  No bevel mask is applied to noise. Neutral hooks return before conversion;
  signed white/fine grain retains the transfer helpers' signed linear branch.
- **Interior light is evaluated within the slab.** Ring and aurora use the
  perturbed normal at refracted landing points, a fixed remaining path of
  `0.2 * thickness`, and one `att ^ 0.2` factor. The ring may show through
  translucent face pixels; opaque pixels still bypass the shader.
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
| `bevel-profile` | glass.bevelProfile (pending, prism-7024c4) | 1, 3, 4 |
| `offset-x` / `offset-y` | `glass.paneShiftX` / `glass.paneShiftY` | 1 |
| `thickness` | `glass.thickness` | 1, 3, 4 |
| `jelly-flex` | `glass.jellyFlex` | 1 |
| `jelly-ripple` | `glass.jellyRipple` | 2 |
| `distortion`, `scale=` | `glass.distortion`, `glass.distortionScale` | 2 |
| `ior` | `glass.ior` | 3, 5, 6 |
| `backdrop-blur` | `glass.backdropBlur` | source selection before 3 |
| `roughness` | `glass.roughness` | source selection before 3; 5 scattering |
| `anisotropic-blur` | `glass.anisotropicBlur` | 3 |
| `chromatic-aberration` | `glass.chromaticAberration` | 3, 5 |
| `iridescence` | `glass.iridescence` | 6 |
| `aurora`, `drift-hz`, `color` × 2 | `glass.aurora`, `glass.auroraDriftHz`, `glass.auroraColorA`, `glass.auroraColorB` | 5 |
| `attenuation-color` | `glass.attenuationColor` | 4 |
| `attenuation-distance` | `glass.attenuationDistance` | 4 |
| `light-ior` | (pending, prism-0ea68f) | 5 |
| `ring-gap`, `ring-width`, `ring-color`, `ring-beam-speed`, `ring-beam-noise`, `ring-beam-noise-hz`, `ring-beam-decay`, `ring-glow`, `ring-rest`, `ring-accent` | `glass.ring.gap`, (none), `glass.ring.color`, `glass.ring.beamSpeed`, `glass.ring.beamNoise`, `glass.ring.beamNoiseHz`, `glass.ring.decay`, `glass.ring.glow`, `glass.ring.rest`, `glass.ring.accent` (prism-f67834, pending) (`gap`, `beamSpeed`, `glow` on Prism's `prism-1514d3`, merged at the rollout) | 5 |
| `saturation` | `glass.saturation` | 3a |
| `noise` `type=` | `glass.noise`; `type=` (pending, prism-51f23b) | 3b |

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
  the original post placement and the unchanged inherit rule. Current behind
  placement: `../specs/2026-09-12-material-render-order-design.md`.
- `../specs/2026-09-05-ring-light-focus-response-design.md`: step 5.
- `2026-09-02-material-signals-design.md`: the signal inputs to steps 5
  to 7.
