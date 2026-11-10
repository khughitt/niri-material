# Shell remap

Type: guide; document types 1. How a shell's opaque layer surfaces are
recolored to match a glass material: which layers take the remap and from
which material, the face color the shell is matched to and the glass that
supports it, the per-pixel move in Oklab, and the project hue a workspace
holds. The `surface-remap` options and an example are in
[Configuration: Layer Rules](../wiki/Configuration:-Layer-Rules.md#surface-remap);
the glass parameters in [material-config.md](material-config.md); the
window accents the hue comes from in [signals.md](signals.md); where the
remap sits among the passes in [render-pipeline.md](render-pipeline.md).
Sources: `niri-config/src/layer_rule.rs`, `niri-config/src/oklab.rs`,
`niri-config/src/material/face.rs`, `src/render_helpers/surface_remap.rs`,
`src/render_helpers/surface_remap_element.rs`,
`src/render_helpers/shaders/surface_remap.frag`, `src/layout/shell_hue.rs`
and `src/layer/mapped.rs`.

## Layer materials

A layer rule's `surface-remap` names three source colors (the shell's
surface, container and outline, together the family) and a material. Rules
resolve like other layer rules, except that a later matching rule replaces
the whole node, so every node a surface gets was validated at load. The
material lends only its glass: a `response=` on it is an error. A layer with
a remap draws its surface, its subsurfaces and its own xdg pop-ups through
`surface_remap.frag` instead of the default texture program; input-method
pop-ups belong to the input method and are drawn as sent. Geometry, damage
and opaque regions stay the client's. A change in what the remap draws
damages the surface and its pop-ups once. If the program fails to compile,
niri logs `error compiling surface remap shader` and draws the layer
unremapped.

## Face effects

The target is the flat face of the named glass over a neutral reference
backdrop, linear 0.2 (`face_srgb`):

```text
T    = clamp(attenuation-color, 0.001, 1) ^ (thickness / attenuation-distance)
f0   = ((ior - 1) / (ior + 1))^2
face = backdrop * T * (1 - f0) + 0.15 * f0
```

These are `main.frag`'s flat-face terms, and a test fails if the shader's
constants drift from them. The real glass shows its own backdrop, which the
opaque shell cannot follow, so the match holds only for dark glass: at most
1% of any backdrop channel may pass, max(T) × (1 - f0) ≤ 0.01
(`check_range`). Within that range the face differs from the swatch by
-0.047 to +0.04501 in Oklab L for neutral backdrops between 0.05 and 0.5,
the worst case being `ior` 1; a test sweeps accepted glass against that
bound. Reflection, the edge highlight and the ring leave the flat face
alone. Aurora lights the whole face and iridescence recolors its specular
floor, so a remap naming glass with either is refused (`check_effects`).
Noise and saturation are left out: noise is zero-mean, and saturation leaves
a neutral backdrop unchanged.

## Oklab remap

`RemapSetup` turns the node and the face into the target family: the
surface takes the face's Oklab L, a and b; the container and the outline
keep their L offset and their chroma excess over the source surface and take
the target's hue. A hue (next section) changes only the target's chroma and
hue.

Per pixel, `surface_remap.frag` (mirrored by `RemapUniforms::apply`):

1. unpremultiplies the pixel and converts it to Oklab;
2. finds its distance d to the polyline surface → container → outline, and
   the offset (target minus source) interpolated at the nearest point;
3. moves it by that offset times w = 1 - smoothstep(0.008, 0.02, d) (`R0`,
   `R1`);
4. converts back, clamps to gamut and premultiplies.

Pixels on or near the family, including low-alpha overlays of near-neutral
text, move with it. Text, icons, saturated accents, error colors and black
shadows lie far from it and keep their colors. The source surface must be
dark: Oklab L at most 0.4 (`MAX_SOURCE_SURFACE_L`).

## Project hue

Each workspace may hold the hue of one window (`ShellHue`):

- a window counts once it has stayed the workspace's active window for
  1000 ms (`HOLD_DWELL`), whatever the keyboard focus;
- a counted window with a signal accent becomes the workspace's record; a
  counted window without one leaves the record as it is;
- the workspace shows the record's current accent, and goes neutral when
  that window closes, leaves the workspace or loses its accent.

A hued target keeps the face's L and takes the accent's Oklab hue at chroma
`hue-chroma`; an accent with Oklab chroma under 0.02 (`MIN_ACCENT_CHROMA`)
has no hue and gives the neutral target. With `hue-chroma 0` the remap stays
neutral whatever the workspace holds. A change of target crossfades over
`animations { material-signal }`, from wherever a running fade has got, as a
linear blend of target families in Oklab. Each output shows its active
workspace's hue, and during a switch blends the two workspaces it shows by
the fractional render index; every remapped layer on an output reads that
one value. While a fade or blend runs the layers count as animating, and a
timer wakes the outputs when a pending window reaches the dwell.
`src/tests/shell_hue.rs` and the tests in `shell_hue.rs` pin these rules.

## Failures

Each is a config error at load, and the whole config is refused:

| Error | Fix |
| --- | --- |
| `surface-remap: material "<name>" takes no response` | Drop `response=` from the node's `material`. |
| `surface-remap: the source surface has Oklab L <l>; the remap supports a dark Noctalia scheme (L up to 0.4)` | Use a dark shell scheme. |
| `surface-remap: material "<name>" transmits <x> in its most open channel; the shell match supports dark glass up to 0.01` | Darken the glass: a darker `attenuation-color`, more `thickness` or less `attenuation-distance`. |
| `surface-remap: material "<name>" sets aurora, which lights the flat face; the shell match supports glass without them` (or `iridescence`) | Set that optic's amount to 0, or name other glass. |

An unknown material name is reported by the general material-reference
check.
