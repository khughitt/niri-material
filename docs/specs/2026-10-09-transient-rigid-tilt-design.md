# Transient rigid tilt of window and glass

**Status:** draft for the owner's review (2026-10-09). No renderer code before
this spec and its implementation plan are reviewed.
**Task:** `material-abc08c`, lane `material-53f873`.
**Brief:** [transient-depth brief](../notes/2026-10-06-transient-depth-brief.md),
alternative 1, and its spike result (`material-89fb6b`).
**Builds on:** the height-field bevel of
[glass edge optics](2026-09-30-glass-edge-optics-design.md) (`material-be611b`,
merged), the swing and focus origin of the rejected
[focus view tilt](2026-09-22-focus-view-tilt-design.md) (`material-77db8a`,
branch `d51825f9`, unmerged), and the
[settle contract](2026-09-29-sustained-optic-settling-design.md).

## 1. Intent

On focus gain, the pane turns briefly in depth and settles back to exactly
face-on. Window content and glass move as one rigid body. While it turns, the
slab shows that it is a volume: its rounded edge and its side wall come into
view, lit by the focus beam that starts on the same gain. At rest nothing
changes: the same pixels and the same code path as today.

What the owner has said, and what this design assumes:

- Said (spike verdict, 2026-10-06): the planar in-shader tilt reads as one
  object. Text softening while it moves is acceptable. Side walls are missing
  first, interior detail second. The walls build on `material-be611b`'s
  height-field edge, so the resting and tilted glass share one slab model.
  Later stills should show a lighter look beside the accepted dark one.
- Said (brief): the trigger is focus gain first. Move, resize and open come
  later, perhaps with other motions. The static by-position perspective leans
  toward being dropped. The rest path must be today's, and settling must be
  finite.
- Assumed here: a swing lasts under a second and peaks within the first
  fifth. It runs once per focus gain, not continuously. Input does not follow
  the tilt.

Out of scope: interior detail (a non-uniform volume, particles,
`material-4e3e9c`, `material-d0518d`), inner-wall total internal reflection,
other triggers, decorations drawn inside the material pass, and the Prism
keys. Section 9 files the Prism keys.

## 2. Findings that shape the design

### 2.1 A centred camera hides the side walls at small tilts

A box seen from a camera on its centre axis shows none of its outer walls. A
wall comes into view only once the pane turns past the angle that edge
subtends at the camera. For a pane of extent `E` across the axis and a camera
at distance `D`, that is `tan θ > E / (2D)`. Below it, both walls face away.
This was checked numerically for the active slab (thickness 31.2, bevel 12,
so a vertical wall 19.2 px deep below the rounded rim):

| Camera | Wall visible from | Wall width at 10° | at 14° | Keystone at 10° |
| --- | --- | --- | --- | --- |
| spike: `D` = 2000 px, 800 px pane | 11.3° | 0 px | 0.9 px | 7.0 % |
| spike: `D` = 2000 px, 1600 px pane | 21.8° | 0 px | 0 px | 14 % |
| `D` = 4 `E` | 7.1° | 1.0 px | 2.4 px | 4.3 % |
| `D` = 6 `E` | 4.8° | 1.8 px | 3.1 px | 2.9 % |
| `D` = 10 `E` | 2.9° | 2.4 px | 3.8 px | 1.7 % |
| orthographic | 0° | 3.3 px | 4.6 px | 0 % |

(Keystone: how much taller the near edge draws than the far edge. It is the
cue that makes the turn read as a rotation rather than a squeeze.
Orthographic rays show the wall at any tilt but no keystone at all: the pane
only narrows by `1 − cos θ`, which is 1.5 % at 10°.)

Three consequences:

1. Strong perspective and a visible wall pull in opposite directions, and one
   parameter, the camera distance, trades them. The spike's camera, at a fixed
   2000 px, could never have shown a wall at its 2 to 8°. On wide panes it
   cannot below about 20°.
2. A fixed camera distance makes the trade depend on pane size. Section 3.2
   measures the distance in multiples of the pane's own extent, so every pane
   reveals its wall at the same angle.
3. On the active slab, the wall is a sliver of a few pixels at any tilt that
   still reads as transient. It cannot carry the depth on its width. It
   carries it by brightness: seen near grazing, it is mostly Fresnel
   reflection, and the focus beam's light escapes through it (section 4.3). The
   rounded rim turning toward the camera is the larger cue. The inactive slab
   (58.1) has walls about twice as wide, but the swing runs on the gaining,
   active pane.

The angle, the distance and the wall's light are therefore chosen from
stills (section 8.3), not fixed here.

### 2.2 77db8a's curve starts at its peak

`θ(u) = θ0 (1 − u)² cos(1.5πu)` starts at θ0. For an optics-only tilt, that
jumped only the view ray. For a rigid tilt, it would snap the text to a
tilted pose in one frame on every focus gain. Section 3.1 replaces it with a
curve that starts at rest.

### 2.3 What the code already gives

- Swing state must live on the `Tile`. A focus gain under `is-active` rules
  replaces the whole `MaterialState`: new id, offscreen and jelly seed
  (`material/mod.rs` `apply_resolved`). 77db8a stored its swing there for
  this reason.
- During a resize the material draws live, at the animated size, around the
  crossfaded body (`tile.rs` near 1919). A swing can therefore run through a
  resize without special handling.
- Open and alpha animations render the whole tile, material element
  included, into an offscreen. The close animation bakes a snapshot through
  `Tile::render`.
- The material element claims no opaque regions. Its damage is whole-geometry
  whenever its commit changes, and smithay damages both the old and new rects
  when the geometry changes. So a footprint that grows for a swing and then
  shrinks back is damaged correctly.
- `InputFingerprint` deliberately leaves out geometry, so a new tilt input
  must enter the fingerprint or a frame would not commit.
- `cull_body` (`material_out_of_view`) bounds the body by the resting slab.
  Workspace crop bounds clip every material element during a workspace switch
  and in the overview.
- Hit testing reads only `render_offset` and `bob_offset`. A visual tilt
  inside the material does not move input.
- Subsurfaces render into the material's window texture. Popups, border,
  focus ring, shadow and the background-effect element are separate elements.

## 3. The motion

### 3.1 Swing curve

With `u = elapsed / settle`:

```
g(u) = (1 − u)² · sin(2πu)
θ(u) = θp · g(u) / G,      G = max g = 0.61003 (at u = 0.1904)
θ(u) = 0 exactly for u ≥ 1
```

- It starts at exactly zero, with an initial angular velocity of
  `10.30 θp / settle`, like a struck body. At the 900 ms default that is a
  peak at 171 ms.
- It crosses zero once, at u = 0.5. The overshoot is −0.1638 θp at u = 0.636.
- It ends at u = 1 with zero value and zero slope, so the run's last moving
  frame is already within a fraction of a pixel of rest. The `u ≥ 1` branch
  makes the end bit-exact, as on 77db8a.

Rejected: 77db8a's curve, which jumps to θ0 (section 2.2). Rejected: a
smooth start with zero initial slope. The spike found that text softening is
a step on leaving the pixel grid, not a function of angle, so what costs
legibility is the time spent off the grid. A slow attack spends that time
before any depth shows.

### 3.2 Axis, pivot, sign and camera

- **Direction.** `d` is 77db8a's unit vector from the focus origin's centre
  to the gaining pane's centre, with its fallback cases unchanged (no origin,
  another output, under 1 px: a seeded direction).
- **Axis and pivot.** The pane rotates about the in-plane axis perpendicular
  to `d`, through the slab's centre on its front face (`z = 0`). Horizontal
  focus moves therefore turn about a vertical axis.
- **Sign.** For the positive lobe, the edge facing the origin recedes. The
  arriving focus presses in the side it came from. The overshoot brings that
  edge briefly forward. The clip review (section 8.4) can flip the sign; the
  machinery does not depend on it.
- **Camera.** It sits on the pivot's normal, at distance
  `D = κ E / w(u)`. `E = |d.x| W + |d.y| H` is the slab's extent along `d`.
  The perspective weight is `w(u) = |θ(u)| / θp`, in [0, 1].
  - When `w` is 0, the rays are exactly parallel, i.e. today's camera.
  - Perspective engages only as far as the pane is turned. So the pinhole's
    zero-tilt optics change (about 3 px of backdrop shift at 800 px from
    centre, up to 55 levels on the bevel) appears only during the motion. It
    returns continuously to the rest optics. This is the continuous
    transition the spike asked for. It is not an exact-zero bypass, because
    rest is the limit of the moving frames rather than a separate case.
  - The keystone scales as `θ·w`, i.e. with θ². The overshoot is nearly
    orthographic, which reads as a slight lean back rather than a second
    turn.
  - The wall's reveal angle is independent of pane size:
    `tan θ > 1 / (2κ)`.

Rejected: the pivot as a hinge on the edge facing the origin. For the same
angle, it moves the free edge twice as far and the whole pane in depth, so
the pane also grows or shrinks. That means more text displacement, more
damage, and a size change that is not part of the cue. Rejected: a camera at
a fixed logical distance (section 2.1). Rejected: a camera at the output
centre, which is the by-position perspective in another form.

### 3.3 Configuration

These live in the response block, so named responses inherit them from
`default` as 77db8a's keys did:

| Key | Range | Default | Meaning |
| --- | --- | --- | --- |
| `view-tilt` | 0–20 (degrees) | 0 (off) | peak angle θp |
| `view-tilt-settle` | 100–5000 (ms) | 900 | run length |
| `view-distance` | 1.5–50 | 6 | κ: camera distance in multiples of the pane's extent across the axis |

The niri default for `view-tilt` stays 0. Starting values come from the
stills and the clip (sections 8.3 and 8.4), and Prism writes them. The
`view-distance` default of 6 reveals the wall from 4.8°. It is a placeholder
until the sheet decides.

The static by-position perspective is dropped. 77db8a's `view-perspective`
and `view-parallax` keys are not ported. At rest the camera faces the glass,
and content does not skew.

## 4. The slab under a tilt

All geometry is in element-local logical px: y down, `+z` toward the
viewer, the front face at `z = 0`, and the slab's back face at
`z = −thickness`. As today, the backdrop is the untilted screen plane. A
point inside the glass is mapped to the screen by rotating it back and
projecting it along the camera ray. This is the spike's approximation, kept.

### 4.1 One ray per screen pixel

For screen pixel `s` (at `z = 0`), with `R` the rotation and `P` the pivot:

1. **Camera ray.** `dir = ((s − P) · w / (κE), −1)`. At `w = 0` this is
   exactly `(0, 0, −1)`.
2. **Slab-local ray.** `o = s + (R⁻¹ − I)(s − P)` and `r = R⁻¹ dir`.
3. **Window plane.** The window content is a decal on `z = 0`, as on the
   spike. Its hit is `h₀ = o + r · (−o.z / r.z)`, and the window texture is
   sampled at `h₀.xy`. An opaque texel returns immediately, as today.
4. **Glass hit.** If `h₀` lies on the flat face (`innerDist < 0`), the glass
   is hit at `h₀`. Otherwise the ray is marched from `z = 0` to
   `z = −thickness`. A point is inside the slab when it lies within the
   silhouette and at or below the height-field top
   (`z ≤ height − thickness`). The march takes a fixed number of steps, then
   bisects. The plan sets the counts from the worst bevel slope and a Rust
   mirror. The first inside point is the hit. If it crossed the top surface,
   it is a **top hit**. If it crossed the silhouette below the rim
   (`z < −rise`), it is a **wall hit**. A ray that finds no inside point
   misses: no glass at this pixel, and the real scene beneath shows through.
5. **Back to the screen.** Every sample of the backdrop or scene passes
   through one helper, `toScreen(q)`. It takes a slab-local point to world
   coordinates (`q + (R − I)(q − P)`), then projects it onto `z = 0`:
   `e.xy + (e.xy − P) · (f − 1)`, with
   `f − 1 = (e.z w / κE) / (1 − e.z w / κE)`. The reflection optic's scene
   sample routes through it too.

Each transform is written as the identity plus a correction that is exactly
zero at identity (`R − I`, `f − 1`, `dir.xy`). At θ = 0 and `w = 0`, every
intermediate value therefore equals today's: `h₀ = s`, the hit's `xy` is `s`,
and `toScreen(q) = q.xy`. That identity is a test (section 8.2), not just a
limit. It is what makes the last moving frame and the first rest frame agree.

If `r.z ≥ 0` the pixel is transparent, as on the spike. This cannot happen
within the configured ranges; a test asserts that.

### 4.2 Top hits

Shading at a top hit is today's stages 1–11 (render-pipeline.md §3), applied
at the hit point instead of the pixel:

- Geometry: `slabSurface(hit.xy)`, with jelly included unchanged.
- Normal perturbation, ripple, aurora, ring band, arc position and sweeps
  take the slab-local position `hit.xy` where they take `p` today. These
  patterns stay fixed to the glass while it moves.
- The refraction taps refract `r̂` instead of `(0, 0, −1)` and travel
  `rayPath(t, height)` to the back plane. The exit point goes through
  `toScreen` to the backdrop sample. Beer–Lambert uses that same path.
- The Fresnel cosine, the iridescence angle and the edge highlight's
  half-vector use the slab-local direction toward the viewer, `−r̂`, instead
  of `+z`. `Surface` gains a `view` field. Every specular hook reads it, and
  at rest it is `(0, 0, 1)`.
- The ring's light shift refracts the same slab-local ray with the light
  index. The band is evaluated slab-local, keeping the half-gap cap on the
  shared shift, as the spike's review required.
- Coverage is today's anti-aliased silhouette coverage, evaluated at the hit
  point. At identity this is exactly today's value.

### 4.3 Wall hits

The wall is the vertical side of the slab between the rim (`z = −rise`) and
the back face. Under a tilt it is seen near grazing.

- **Normal.** The silhouette's outward softened gradient, `(ĝOut, 0)`, from
  the same `softOuterGrad` the bevel uses. Its corners are therefore
  continuous with the rim.
- **Fresnel.** Schlick from `ior` at `dot(−r̂, n)`, which is near 1 at
  grazing incidence. Transmission is scaled by `1 − F`, as on the face.
- **Transmission.** Refract into the wall, follow the ray to the back plane,
  `toScreen` the exit point, sample, and attenuate over that path.
- **Specular.** The same hooks, with a wall `Surface` (`across = 1`,
  `acrossDir = ĝOut`, `outerDist = 0`, the wall normal as both normals). The
  reflection optic, at its configured amount, mirrors the scene just beyond
  the silhouette. With `reflection 0` the wall keeps only the Schlick glint.
- **Beam spill.** The bevel's spill fades to zero at the silhouette (the
  `1 − u` factor). The wall is where guided light leaves the slab, so it
  carries the spill at the face-edge strength: the moving beam's brightness at
  the nearest point of the line, times `BEAM_SPILL`, times `att^0.2`. It is
  lit only while the beam runs. This is an art choice, and the sheet shows it
  on and off (section 8.3).
- **Coverage.** The outline of a wall, or of a rim seen from above, is
  anti-aliased from the ray's miss distance to that edge, at most one
  physical pixel wide. The plan pins the formula against the Rust mirror.
- **Grain.** Film grain applies as on the face. Glass-site grain applies to
  the wall's transmitted light.

The window decal does not reach the wall: the window lies inside the
silhouette.

### 4.4 Grain and jitter seeds

The glass-site grain, the film grain and the tap jitter are seeded from
`gl_FragCoord` today, so under a tilt they would stay fixed to the screen
while the glass moves. Under a tilt they take a slab-anchored seed instead:

```
seed = gl_FragCoord.xy + round(Δ)
Δ = (hit.xy − s) · scale, in framebuffer orientation
```

- Rounding to whole pixels keeps the per-pixel hashes on their lattice. The
  grain moves with the glass as a nearest-neighbour pattern. Unrounded seeds
  would re-hash every frame and make the grain boil.
- Value-noise layers above `scale=` 1 take `Δ` unrounded, so they glide.
- At identity `Δ = 0`, and the seed is exactly `gl_FragCoord`.
- Backdrop-site grain lives in the backdrop buffers, which do not tilt, and
  is unchanged.

## 5. What tilts, what stays flat, when it runs

| Element | Under a swing | Why |
| --- | --- | --- |
| Window body, subsurfaces | tilt | they are the material's window texture |
| Glass slab, its optics, ring, grain | tilt | section 4 |
| Shadow | stays flat | it lies on the backdrop plane, which does not tilt; a far edge that recedes shows a few px of shadow beneath, which is what a lifted pane shows |
| Popups | stay flat | separate surfaces; rarely open across a focus gain. Drawing them in the pass is the follow-up below |
| Border, focus ring | swing not started | they would visibly detach (the spike: at 4°) |
| Background-effect element, non-neutral | swing not started | its flat rectangle would show where the far edge recedes. Neutral (Prism's `blur false; noise 0; saturation 1`) is invisible and allowed |

A swing starts on a focus gain observed by 77db8a's `FocusOrigin` once per
update pass, under these conditions:

- the tile has a material, `view-tilt > 0`, and motion is allowed (`signal
  motion full`, animations on);
- the tile is not fullscreen and no open animation is running on it (opening
  is its own arrival);
- border and focus ring are off for the tile, and its background effect is
  neutral;
- no swing is already running on it.

The last condition replaces 77db8a's restart, which would now be a jump back
to zero. A pane refocused mid-swing finishes the swing it has. Holding a key
that walks focus across columns gives each pane one swing. The live config
meets these conditions: border and focus ring are off, the background effect
is neutral, and the shadow is on.

These conditions are documented in `material-config.md` beside the beam's
`ring-light` condition. Drawing border, focus ring and popups inside the
material pass, so that they tilt too, is filed as a follow-up idea when this
design is accepted.

### 5.1 Interactions

- **Focus loss mid-swing.** The swing finishes on the now-unfocused pane.
  The material swap is the accepted hard cut (`material-8e3b73`). Under a
  tilt, the cut also changes the slab's thickness, and so the rim and wall,
  in one frame. This is accepted with the cut, and the frozen sequence checks
  it (section 8.2). The focus-loss cue (`material-ccda38`) adds no tilt.
  Should it want one later, it reuses this machinery with its own curve.
- **Resize.** It runs through: the material draws live at the animated size
  (section 2.3). The jelly resize stays slab-local. The rigid transform
  applies after it.
- **Jelly flex and ripple.** They compose. They deform the face inside the
  slab's frame, and the tilt moves that frame. Ripple activity reads the
  slab-local position.
- **Move and scroll.** They compose. A focus change that also scrolls the
  view gives jelly from the scroll and the swing from the gain.
- **Open.** No swing starts while it runs (section 5).
- **Close mid-swing.** The unmap snapshot renders through `Tile::render`,
  so it captures the current pose. The close animation starts from the pose
  shown, with no jump.
- **Alpha animation, overview, workspace switch.** The tilted element
  renders inside those offscreens and crops like any material element. A
  tilt that crosses a workspace crop bound is clipped there. This is
  accepted.
- **Reduced or off motion, animations off.** No swing starts. A policy
  change mid-swing cuts it to rest on the next frame (77db8a's
  `cut_view_swing_if_forbidden`, kept). Animations off also completes
  instantly. A reload that changes only `view-tilt` or `view-distance` keeps
  the running snapshot.
- **Hidden panes, other outputs.** The run follows the real animation
  clock. It is reported ongoing only while the band is visible, as the beam
  is, and it ends on time whether drawn or not.
- **Screencopy before rule refresh.** 77db8a's accepted caveat holds: one
  frame may start the swing under the previous material.

## 6. Element area, damage and cost

- **Footprint.** On the CPU, `view.rs` projects the slab's eight corners
  (front at `z = 0`, back at `z = −thickness`) through the frame's camera.
  The element's geometry for that frame is the resting area merged with that
  bounding box, rounded out to physical pixels. The quad's UV maps back to
  the resting area through a 4-vector, as on the spike. At rest the geometry
  is `frame.area`, exactly today's.
- **Commit.** The view state (θ, axis, `w`, κ) joins `InputFingerprint`, so
  every swing frame commits. At rest the state is `None`, and the
  fingerprint and commit sequence are today's.
- **Damage.** Whole-footprint each swing frame. The frame that returns to
  rest damages the larger previous rect (smithay, section 2.3).
- **Culling.** During a swing, `material_out_of_view` uses the footprint.
- **Opaque regions.** None, as today.
- **Cost at rest.** Nothing new runs: the view branch is one uniform test,
  and the shading receives today's inputs. A rest render must be
  byte-identical to the pinned baseline (section 8.2).
- **Cost during a swing.** The general path runs over the enlarged quad for
  at most `settle`. The march runs only on rays that miss the flat face,
  i.e. the bevel band and the wall. The clip preparation records its GPU cost
  once, the way the glass-edge work did.

## 7. Code structure

| Unit | Does | Depends on |
| --- | --- | --- |
| `render_helpers/material/view.rs` (from 77db8a, rewritten) | the curve `g`, weight `w`, direction helpers, the f64 mirror of section 4.1 (camera ray, `toScreen`, footprint), the wall-reveal angle | nothing |
| `layout/focus_origin.rs` (from 77db8a) | observes focus once per pass and starts a swing | `view.rs`, `Tile` |
| `Tile` swing state (from 77db8a, adapted) | `ViewSwing` snapshot, the eligibility of section 5, the policy cut, `are_animations_ongoing`, view state into `MaterialDynamics` | `view.rs` |
| `MaterialRenderElement` | footprint geometry, quad mapping, view uniforms, fingerprint | `view.rs` |
| `prelude.frag` | the view uniforms, `toScreen`, slab-local ray, slab march, wall surface, `Surface.view`, the seed helper | — |
| `main.frag` | one uniform branch, which builds the hit and shades it with today's stages | prelude |
| optics (`reflection`, `edge_highlight`, `iridescence`, `noise`) | read `Surface.view`, `toScreen` and the seed helper | prelude |

77db8a's tile tests and layout tests (`view_swing.rs`) port with the
current `update_render_elements` signature. Its shader view routing and
static perspective do not port.

## 8. Verification

### 8.1 Pure (Rust)

- The curve starts at exactly 0. It peaks at 1 at u = 0.1904 ± 1e-3, crosses
  zero once at 0.5, and overshoots −0.1638 ± 1e-3. It is exactly 0 at u ≥ 1,
  and `|θ(1 − 1e-4)| < 1e-8 θp`.
- The weight is in [0, 1] and is 0 exactly where θ is.
- The mirror's identity: at θ = 0 and `w = 0`, the camera ray, slab-local
  ray, plane hit and `toScreen` reproduce their inputs bit-exactly.
- The footprint contains every projected slab point, sampled densely, at
  θp ∈ {2, 10, 20}° and κ ∈ {1.5, 6, 50}.
- The wall's reveal angle matches `atan(1 / 2κ)` against a ray-cast of the
  mirror. The 10° widths in section 2.1 match to 0.1 px.
- No configured range gives `r.z ≥ 0`.

### 8.2 Frozen renders (headless, no host)

These go on the `ring_look` / `glass_edge` harnesses, with the beam,
aberration, reflection and grain on:

- **Identity at tilt 0.** A test hook forces the view branch on, with θ = 0
  and `w = 0`. Its output is byte-identical to the rest path's, frame for
  frame, mid-beam included.
- **Rest identity.** With `view-tilt 0`, and again one frame after a swing
  has ended, renders are byte-identical to the pinned pre-change baseline.
  That covers opaque client pixels and translucent glass.
- **Swing sequence through a real focus gain.** Frames at fixed offsets
  across `settle`, then at `settle` and later. Checked:
  - the frame at `settle` is byte-identical to rest;
  - after the run, `are_animations_ongoing` is false, `tick_deadline` is
    `None`, and the fingerprint is constant at 1, 1.5, 3 and 10 s;
  - the element geometry equals the footprint for each frame, and returns to
    `frame.area`.
- **Containment.** A test hook draws the quad oversized at the peak. No
  nonzero pixel falls outside the computed geometry.
- **Walls.** At the peak, the advancing edge shows wall coverage of the width
  the mirror predicts (±1 px), and the receding edge shows none. Below the
  reveal angle, neither edge does.
- **Sequences.** Focus loss mid-swing (the cut, then the finish), refocus
  mid-swing (no restart), resize mid-swing, policy cut, animations off, an
  ineligible tile (border on, fullscreen, opening), and close mid-swing (the
  snapshot pose).
- **Layout tests.** 77db8a's origin cases, ported.

### 8.3 Owner stills (headless, before any defaults)

A self-explaining sheet, per the review-sheet rule:

- **Title.** The question: "How far, and how much perspective, for the focus
  tilt?"
- **Grid.** Rows θp ∈ {6, 10, 14}°. Columns κ ∈ {2.5, 6, 10}. Each cell is
  the peak frame of the accepted dark look and, beside it, a lighter look.
- **Extra row.** The wall spill off and on at the middle cell.
- **Crops.** Native-resolution crops of the advancing edge and of text.
- **Labels.** Each cell has a stable id. The legend names the wall, the rim
  and the keystone.
- **Control.** The spike's 4° at `D` = 2000 px, marked as the control.
- **Filmstrip.** One swing at the middle cell.

The owner picks θp, κ and the spill. If no cell shows enough depth, section
2.1 names the remaining levers: a larger angle, or a thicker active slab.
The slab is a material choice, outside this design.

### 8.4 Owner clip (nested lane, before Prism defaults)

The clip is a real swing on the nested compositor, on the headless weston
unit, never the desktop. It is its own preparation step, with the smallest
pilot through its verdict first, `ring-motion-clips.sh`'s swing sequence
adapted, and GPU cost recorded. The owner judges the motion, the sign and
`settle`. Only then does Prism get starting values.

## 9. Follow-ups filed on acceptance

- A Prism task: `view-tilt`, `view-tilt-settle`, `view-distance` (names
  as 77db8a: `glass.view.tilt`, `glass.view.tiltSettle`, plus
  `glass.view.distance`), after the clip.
- An idea: border, focus ring and popups drawn inside the material pass,
  so that they tilt.
- An idea: other triggers (move, resize, open) with their own curves, after
  the owner has seen focus gain.
- The brief's result section and a note on `material-6901e0`, in the commit
  that records this design's acceptance.
