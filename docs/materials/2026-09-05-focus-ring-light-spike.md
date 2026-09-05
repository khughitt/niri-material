# Focus ring light spike

**Result:** a ring of light embedded in the slab and refracted by it is the
treatment to build. It reads as light on glass rather than a drawn border,
stays confined to the bevel so legibility is untouched, moves slowly with a
travelling brightness and breathes with the jelly residuals, and degrades to
a static filament that costs no redraws. Organic noise is the same substrate
with a different brightness field and belongs as a mode of the winner. The
current static gradient ring is invisible against the dark focus glass, so
any of the candidates is a strict improvement over it. Run 2026-09-05
against a release build of `materials-26.04` at `119f1cf3` carrying the
probe patch (binary SHA-256 prefix `fcdc5ed4`) on a 1280 x 720 headless
Weston GL host.

Task: `material-d1f471`.

## Question

The focus ring is a static 4 px gradient from `#ffffff00` to `#ccccff11`
drawn as a border element that knows nothing about the material. Can a
lighting treatment on the focused tile replace it, reading as light in or
on the glass, using the jelly residuals as a motion source, staying subtle,
and degrading to the static ring when animations are off? Six candidates
were named: ring of light, particles, god rays, shadow-based, organic noise,
and canopy light. The owner refined the ring: it should sit inside the
glass and be distorted by it, with slow movement so the light interacts
with different parts of the slab.

## Method

The probe lives inside `material.frag`, not the focus ring shader, because
the material shader already has everything the treatment needs: the slab
distance field, the bevel normal, the light direction the rim-orbit
response moves, the jelly shear, resize, activity, seed and time uniforms,
and the Beer-Lambert attenuation. Every light source is modelled as sitting
inside the slab at some depth. The fragment's orthographic ray is refracted
at the perturbed front-face normal (bevel slope, distortion noise, jelly
ripple), the light field is evaluated where that ray lands, and the result
is added as emissive before the sRGB transfer. Chromatic spread reuses the
material's aberration per channel. Two things had to be exaggerated to make
the glass visibly work on the light:

- Prism's glass has an index of 1.02, where refraction is invisible. The
  light path uses an effective index of `1 + (ior - 1) * 6` (1.12 here).
- The dark focus glass (`#222436` at distance 30, thickness 41.7) swallows
  anything at depth at the full attenuation term; the bevel alone attenuates
  to about 1e-5. The probe applies a fifth of the Beer-Lambert exponent.

The plumbing is one `mat_probe` uniform (candidate, active, seconds), an
active flag captured on the tile at `update_render_elements`, a per-frame
redraw while a probe is active on a tile with a material, an
`NIRI_FOCUS_PROBE_HZ` knob that quantizes the time tick, and a draws/s
counter logged once a second. The selector is `NIRI_FOCUS_PROBE`. The whole
probe is throwaway and is retained as `niri-experiments`
`fixtures/focus-ring-light-probe.patch`; the branch that carries this
write-up does not carry it.

`scripts/focus-ring-light.sh` nests the probe binary under a headless
Weston host, spawns two kitty windows at background opacity 0 with no
decorations showing the focus-glass transcript, over the dark split glass
the 2026-09-04 spike settled on (dense dark active, lighter frosted
inactive, both derived from Prism's `terminal-glass`), with the static ring
off and the daily-driver shadow on. Per candidate it captures a still of
each focus side with a 3x corner crop, a 12-frame idle burst of the focused
window (about 2.6 frames per second, the async screenshot's rate), a
12-frame burst after `move-column-right` with animations slowed 6x so the
jelly residuals are on screen for several frames, and six seconds of the
probe's draws/s on an idle focused window. The `baseline` case runs the
static ring with no probe. The `shadow` case adds `is-active` window rules
with a deep soft shadow for the focused window and a tight faint one for
the unfocused. `ring-hz20` runs the ring with the time tick quantized to
20 Hz.

## Observations

Corner crops of all seven cases side by side are `candidates-corners.png`
in the retained artifacts.

- **baseline.** The static ring does not register. At `#ccccff11` peak over
  the dense dark glass it is below one gray level; the pane's edge reads as
  a plain dark rim. Both focus sides look the same at the rim.
- **ring.** A soft lavender filament mid-bevel, 5 px in from the slab edge,
  with a halo bleeding a few px into the glass and a faint blue-outside,
  warm-inside chromatic fringe. The travelling wave makes the right and
  bottom edges brighter than the top and left in one frame and shifts over
  seconds. It is unmistakably the focused pane and the text field is
  untouched. Between the first and last idle frame the mean absolute change
  over the window crop is 0.0009 (max 0.097 on a 0 to 1 scale): motion that
  is felt, not seen as flicker.
- **motes.** Fourteen bluish points with soft halos beading along the rim,
  drifting slowly. Legible and clearly alive (mean change 0.0003, max
  0.26), but it reads as decoration hung on the edge, not as light in the
  glass, and the angular parametrization crowds the short edges.
- **rays.** Fine combed streaks in the bevel on the two edges facing the
  rim light, fading within 14 px. The first tuning, reaching 64 px, painted
  vertical bands across the whole pane like blinds. Confined to the bevel it
  is a texture rather than volume: a 45-degree chamfer 11 px wide has no
  depth for rays to travel through. Barely moves (max change 0.025).
- **shadow.** Config only; the shader adds a bevel glint lift that does not
  register. The focused pane's deeper, offset shadow reads as lifted off the
  backdrop (`shadow-vs-baseline.png`). Zero redraws while idle. Over a busy
  bright wallpaper the cue is weak, and the swap is a hard cut on focus
  change, the same cut `material-5a5fff` is about.
- **noise.** The ring's filament with a slow domain-warped noise field for
  brightness: bright and dark stretches crawl along the rim like caustics
  in the glass edge. Subtler than the ring, patchier, and the strongest
  motion of the rim candidates (mean change 0.0006, max 0.28). It is a
  brightness mode of the ring, not a separate treatment.
- **canopy.** Dappled light drifting across the whole pane, concentrated at
  the bevel. The most atmospheric candidate and the only one that moves
  visibly at the capture rate (mean change 0.0097, ten times the ring), but
  the dapples sit under the text and read as wallpaper motion rather than
  focus.

Redraw cost while a focused window is otherwise idle:

| Case | Draws per second |
| --- | --- |
| baseline, shadow | 0 |
| ring, motes, rays, noise, canopy | 60 |
| ring-hz20 | 60 |

`ring-hz20` shows that quantizing the time tick in the damage fingerprint
does not throttle anything under this plumbing: the probe's per-frame
animation flag re-renders the offscreen every frame regardless. A real
implementation needs a timer at the chosen rate, the way the signal
oscillators wake, so a 15 or 20 Hz drift costs 15 or 20 draws a second, and
it needs the same "pinned time when static" rule the signals design applies.

The move bursts show the pane sliding under the slowed animation; at 2.6
frames per second the jelly breath on the ring is not separable from the
motion blur of the slide. The activity coupling is in the shader
(`1 + 2 * activity` on the ring's glow) and was verified only by reading,
not by capture.

## Ranking

1. **Ring of light.** Reads as light in glass, confined to the bevel, moves
   slowly, breathes with residuals, degrades to a static filament at zero
   cost. Build this.
2. **Organic noise.** Same substrate, different brightness field. Ship as a
   mode of the ring (`wave` or `caustic`), not a separate treatment.
3. **Shadow-based.** Free, and a good complement: the focused pane lifts.
   Not a replacement on its own over a busy backdrop. Worth adding to the
   daily-driver config independently of any compositor change.
4. **Canopy.** Beautiful and the most alive, but it competes with text and
   is not a focus cue. Possibly a material `attention` response for a
   window demanding focus, where loudness is the point.
5. **Motes.** Decorative rather than material. Drop.
6. **God rays.** The slab has no depth for them. Drop.

## Consequences for the winner

- The treatment is a material glass response, not a focus ring option. It
  needs a real active input to the material element, which today only
  exists as the `is-active` material swap. Crossfading it on focus change
  is `material-5a5fff`'s question and applies here unchanged.
- Config surface, in the material `response` block next to `accent` and
  `attention`: `focus "ring-light" | "none"`, `focus-inset`, `focus-width`,
  `focus-color`, a brightness mode, and a drift rate in Hz where 0 means
  static. The light-path index multiplier should be a glass parameter so
  Prism can tune it with the rest.
- Animations off, or drift rate 0, pins the time and the ring is a static
  refracted filament with no per-frame cost; that is the degrade path the
  task asked for, and it is strictly more visible than the gradient ring it
  replaces. The gradient ring stays available for non-material windows.
- The DRM acceptance gates are untouched by the probe by construction (it
  is off unless the environment variable is set); the real implementation
  must be measured against them with the drift running.

## Retained artifacts

Captures, generated configs, nested logs, cost and timing records, the
labelled corner comparison, the shadow comparison, the idle-motion
difference montage, per-burst contact sheets and GIFs are outside the
product tree under:

```text
$NIRI_MATERIAL_WORK_ROOT/focus-ring-light-fcdc5ed4
```

`SHA256SUMS` there covers the binary and every capture. The probe patch is
`niri-experiments` `fixtures/focus-ring-light-probe.patch` and applies
cleanly to `materials-26.04` at `119f1cf3`. The harness is
`scripts/focus-ring-light.sh` and was re-run from its committed location
for the `baseline` case.
