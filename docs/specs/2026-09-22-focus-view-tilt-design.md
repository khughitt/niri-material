# Focus view tilt: a perspective camera and an arrival swing

**Status:** rejected 2026-10-06, not merged. Implemented on branch
material-77db8a (5852ce36…0c7b66f5, evidence in that branch's
`docs/materials/2026-09-22-focus-view-tilt-evidence.md`); the owner's review
of the clips rejected the optical-only premise: the terminal text stays
static while the glass moves, which breaks the one cohesive material. Any
tilt must move window content and glass together (`material-6901e0`).
Design of 2026-09-22, revised after two
design reviews (swings started by the Layout from the focused window's
identity, a focus origin in output space observed once per update pass,
the parallax gain definition, the coordinate frame, a finite swing
endpoint, motion policy decoupled from the ring, a baseline-versus-candidate
identity capture). **Task:** `material-77db8a`.
Follow-ups: `material-6901e0` (a geometric pane tilt), `material-d0518d`
(focus lighting beyond the beam).

## Context

The ring beam ([2026-09-19-ring-beam-design.md](2026-09-19-ring-beam-design.md))
gives focus a light that moves inside the glass, but the glass itself is
always seen from straight above. The owner's review: the camera is always
dead on to the glass surface, so the pane feels static and shows fewer
optical and material effects than it could. The comparison is the Mindful v3
particles view, a three.js scene whose `MeshTransmissionMaterial` title is
seen through a perspective camera placed off to one side: the oblique view
changes path length, refraction and glint across the surface, and every
change of view changes them again.

`material.frag` hard-codes the orthographic incident ray `(0, 0, -1)` in
five places (render-pipeline.md §3):

| Stage | Where | What the fixed ray pins |
| --- | --- | --- |
| 3 | `tap()` in `prelude.frag` | the refraction displacement of the backdrop |
| 5 | `lightShift()` in `prelude.frag` | where the ring and aurora land in the slab |
| 4 | `surfaceCosine = surfaceNormal.z` | the Beer-Lambert path length |
| 6 | the same `surfaceCosine` | the Fresnel glint |
| 6 | `iridescence_specular(…, surfaceCosine)` | the thin-film hue |

The request: a subtle camera movement, transient, settling fairly quickly.

## Decisions

Made in the brainstorm of 2026-09-22, with the alternatives at the end.

- **Optical only.** The view vector changes; window pixels, the slab
  silhouette, element geometry, damage and hit-testing do not. Text never
  skews. A geometric tilt is `material-6901e0`, judged after this is seen.
- **A perspective camera at rest.** Each fragment's view ray leans by its
  position on the output, so windows toward the edges are seen obliquely
  and a window that moves across the screen changes its optics as it goes.
  This is static: it costs no clock.
- **An arrival swing on focus gain.** The view starts tipped as if the
  camera had just come over from the previously focused window, and settles
  with one soft overshoot to exactly the resting view at a fixed time.
- **A parallax gain on the view's contribution only.** At physical values a
  20° view moves the ring about 1.5 px (it lies 20 % of a ~26 px slab deep)
  and the backdrop about 7 px, while the chamfer glint responds strongly.
  The gain deepens the parallax without making the glint glare, and never
  amplifies today's refraction.
- **Everything defaults to off.** The shipped appearance is unchanged until
  a key is set, and with every view key at its default the output is
  pixel-identical to the renderer before this change.

## 1. The view model

The incident ray at a fragment is

```
I = normalize(vec3(s, -1))        s = s_persp + s_swing
```

`s` is the ray's slope in the glass plane, a vector in output-local logical
coordinates (x right, y down, the frame `backdrop_rect` already maps into).
`s = 0` is today's ray exactly.

**Perspective.** With `c` the fragment's output-local logical position,
`o` the output's logical centre and `h` the output's logical half-diagonal:

```
s_persp = k · (c − o) / h
```

`k` is `view-perspective`: the slope at the output's corners, so `k = 0.36`
is a 20° lean there. The fragment position comes from the existing mapping:
`c = (mat_backdrop_rect.xy + v · mat_backdrop_rect.zw) · output_size`. The
backdrop buffer covers the whole output in logical space after the output
transform, so rotation and fractional scale are already applied; dividing by
the half-diagonal keeps the lean independent of resolution and aspect.

**Swing.** With `θ(t)` in degrees and `d` a unit direction (§3):

```
s_swing = tan(radians(θ(t))) · d
```

The pixel-identity rule: when `k = 0` and no swing is running, the shader
takes today's code path unchanged, not a numerically equal one.

## 2. The shader

A new uniform `mat_view` (`vec4`: `s_swing.x`, `s_swing.y`, `k`, `gain`)
and `mat_view_output` (`vec2`: output logical size). `main.frag` computes
`I` once per fragment after `slabSurface` and passes it down.

**Displacements** (`tap`, `lightShift`). Today's displacement is
`refract(I0, n, 1/ior).xy · depth` with `I0 = (0, 0, -1)`. With a view:

```
shift = shift(I0) + gain · (shift(I) − shift(I0))
```

The gain scales only what the new view adds: at `gain = 1` this is
`shift(I)`, one `refract`, and the shader computes it that way; otherwise two
`refract`s per tap. Today's chamfer refraction and its `mat_distortion` and
jelly perturbations are not amplified. The ring's existing cap (the shared
shift clamped to half the gap) applies after the gain, so a large gain on a
narrow gap is limited by the cap; `material-config.md` says so beside the key.

**Cosines** (Beer-Lambert, Fresnel, iridescence). `surfaceCosine` becomes
`clamp(dot(-I, surfaceNormal), 0, 1)`. The gain does not apply: it is a
parallax control, and a steeper cosine is what makes the glint move across
the chamfers. The 0.25 floor on the Beer-Lambert cosine stays. On the flat
face the Schlick term barely changes at these angles (f0 ≈ 0.008 at
`ior 1.2`); that is expected, and a moving face reflection belongs to
`material-d0518d`.

The Fresnel `facing` weight (toward `mat_sig_light`) is a light direction,
not a view direction, and is unchanged.

## 3. The arrival swing

**Curve.** With `u = t / settle`, clamped to `[0, 1]`:

```
θ(u) = θ0 · (1 − u)² · cos(1.5π · u)
```

It starts at `θ0`, crosses zero once at `u = 1/3`, overshoots to
`−0.1777 θ0` near `u = 0.514`, and reaches zero at `u = 1` with zero value
and zero slope: `(1 − u)²` and the cosine both vanish there, a triple root,
so the end is still. There is no second crossing. At `u ≥ 1` the swing is
exactly absent and the run ends. `θ0` (`view-tilt`) and `settle`
(`view-tilt-settle`) are snapshotted per run; a reload mid-run does not
jump the view.

**Who swings.** A swing is started by the `Layout`, on the tile of the one
globally focused window, and never by a tile's own `active` flag. That flag
cannot name one tile: `Monitor::update_render_elements` passes the same
monitor-level `is_active` to every visible workspace, so during a workspace
switch or in the overview several workspace-selected tiles are "active" at
once. A tile only holds its run; `Tile::start_view_swing(d)` begins one if
the tile's motion policy and response allow it (below).

**The focus origin.** A `FocusOrigin` owned by the `Layout`: the focused
window's identity, its output, and its centre in output-local logical
coordinates. Its semantics are "the last focus observed by an update
pass". Each call of `Layout::update_render_elements(output)` does this
on the focused window's monitor, inside its monitor loop, after
`set_overview_progress` has synced that monitor's overview state and
before `Monitor::update_render_elements`:

1. Take the focused window and its output from `Layout::focus_with_output`.
   If there is none, or the call does not visit that output, or the window
   is the one being interactively moved, the origin is left untouched.
2. Compute the window's centre on its output: find its workspace among
   `Monitor::workspaces_with_render_geo`, take its tile's workspace-local
   position from `Workspace::tiles_with_render_positions` and its animated
   size (`Tile::animated_tile_size`, so a resizing tile's centre matches its
   rendered one), and map it the way `Monitor::window_under` inverts it,
   `geo.loc + zoom · (pos + size / 2)`
   with the monitor's current overview zoom. A new `Monitor` helper returns
   this for a window identity, so the workspace translation and zoom of the
   same instant are applied to every centre; a focused window whose
   workspace is culled off the output leaves the origin untouched.
3. If the focused identity differs from the origin's, start a swing on its
   tile with `d` = the normalized vector from the origin's centre to the new
   centre, unless the origin is absent, on another output, or within 1 px,
   in which case `d` is the fallback: a unit vector seeded from the pane's
   jelly seed mixed with the run's start instant (as the beam's wander seed
   is).
4. Record the focused window, output and centre as the origin. A focused
   window that moves keeps its origin current.

Screencopy and screenshots call `update_render_elements(Some(output))` too.
They take part rather than being excluded: focus is global state, so a
capture's pass observes the same change the next output pass would, and
at most starts the swing one pass earlier. A call that visits only an
inactive output changes nothing (step 1).

Consequences, all intended: focus changes A → B → C with no pass in
between swing C from A, since no pass observed B. A → B → A with no pass
between starts no swing: the identity at the next pass is still A. Focus
leaving for an empty workspace and returning to a new window swings from
where the last focused window was seen.

**Lifecycle.** The swing is one run on the animation loop, like the beam:
it is reported through `are_animations_ongoing` while the slab band is in
view, has no bucket clock and no deadline, and is not a layout transition.

- A swing start replaces a running run on that tile; the new run starts at
  its own `θ0` in its own direction.
- A focus loss does not end the run: it finishes settling, so the view
  never snaps.
- A settled tile has no clock and a constant fingerprint.

**Motion policy.** Today `beam_allowed` combines the motion policy with the
ring's eligibility (`focus ring-light`). The motion part is factored out as
`motion_allowed` (`signal motion full`, animations on); the beam keeps
`motion_allowed && focus ring-light`, and the swing uses `motion_allowed`
alone, so a material with no ring can still swing. A swing may start when
`motion_allowed` and `view-tilt > 0`, and a running swing ends at once, at
rest, when `motion_allowed` turns false. Perspective is not motion and is
unaffected by the policy.

## 4. Configuration

| Native key | Block | Default | Range | Prism key |
| --- | --- | --- | --- | --- |
| `view-perspective` | material | 0 | 0–1 | `glass.view.perspective` |
| `view-parallax` | material | 1 | 0–8 | `glass.view.parallax` |
| `view-tilt` | response | 0 | 0–45 (degrees) | `glass.view.tilt` |
| `view-tilt-settle` | response | 900 | 100–5000 (ms) | `glass.view.tiltSettle` |

Perspective and parallax describe the glass and the camera, so they are
material parameters; the swing is a reaction to focus, so it sits in the
response block beside the `ring-beam-*` keys. Validation rejects values
outside the ranges. Prism's keys are a Prism task filed with the
implementation plan.

## 5. Cost and redraw

- One `vec4` and one `vec2` uniform; per fragment one `normalize`, one
  `dot`, and at `gain ≠ 1` a second `refract` per tap. Cost is recorded
  on llvmpipe the way the optics were (`material-6102b2`).
- Perspective depends on the element's position, which `backdrop_rect`
  already carries in the material fingerprint; a test asserts that moving
  an element with `k > 0` changes the fingerprint.
- A running swing changes `mat_view` every frame and joins the fingerprint
  through it; once settled `s_swing` is exactly zero and the fingerprint is
  constant again.

## 6. Testing and judging

- **Curve** (Rust, beside `ring.rs`): starts at `θ0`; one sign change; the
  minimum is `−0.1777 θ0` to within 1e-3; exactly 0 at and after
  `u = 1`; the end slope is 0 to within the step.
- **Direction and origin** (layout tests): the direction points from
  origin to the new focus; each fallback case (no origin, other output,
  coincident) takes the seeded direction, which differs between panes;
  A → B → C with no intervening pass swings C from A; A → B → A starts no
  swing; a pass that visits only an inactive output leaves the origin
  unchanged; a screencopy pass counts as an observation.
- **One swinger:** during a workspace switch with both workspaces visible,
  and in the overview, exactly one tile starts a swing, the focused one.
- **Centres in output space:** the recorded centre of a tile matches its
  rendered position on the output mid workspace switch and at overview
  zoom.
- **Lifecycle:** a start begins a run; a second start replaces it with a fresh
  snapshot; a loss lets it finish; the run ends at `settle` and leaves no
  deadline and a constant fingerprint; reduced motion or animations off
  cuts it at rest; a material with `focus none` still swings.
- **Shader source:** the curve runs on the CPU and the shader receives
  only the slope, so there are no shared curve constants. The source test
  asserts the view uniforms, that every orthographic `refract` sits inside
  the one view helper, and that the program compiles under
  `glslangValidator`.
- **Identity:** the neutral baseline-versus-candidate capture of
  `glass-render-order-smoke.sh`, with the baseline built from this branch's
  base commit and the candidate from the branch, every view key omitted,
  clocks and seeds fixed: zero difference. Comparing omitted keys with
  explicit zeros inside the candidate alone would miss a regression both
  share, so it is not the gate.
- **Judging:** `view-swing` and `view-perspective` sequences in
  `ring-motion-clips.sh`, rendered for the owner's review sheets at a small
  grid of `view-tilt` × `view-parallax` (and `view-perspective` for the
  static sheet). The defaults for Prism's presets are set from that review.

## Alternatives rejected

- **A geometric tilt of the pane.** It reads more literally as 3D but
  softens text while moving and touches element geometry, damage and
  hit-testing. Deferred to `material-6901e0`.
- **A constant resting tilt.** Every window leaned the same way. Perspective
  gives each window its own angle from its place on the output, which is
  what a camera does.
- **A settling noise drift, or swing plus drift.** Organic, but with no
  direction it does not read as arriving; the owner chose the swing.
- **A physical underdamped spring.** It never reaches zero, so the run
  would need a threshold and a snap. The polynomial envelope ends exactly,
  and still.
- **Scaling the whole displacement by the gain.** It would amplify today's
  chamfer refraction too, changing the resting look of every material.
- **Only larger angles instead of a gain.** A larger angle steepens the
  glint along with the parallax; the gain separates them.
- **The origin taken from the last rendered active tile, or written by
  tiles in the update pass.** Tiles cannot tell the focused tile from the
  other workspace-selected tiles (§3), and their `view_rect` is
  workspace-relative. The `Layout` knows the focused window and can map it
  to output space.
- **Snapshotting the origin at each focus-changing call site.** Many
  layout paths change focus; one observation per update pass covers them
  all, and "last observed" is the right notion of where the camera was.
- **Threading the origin as an argument** through the five
  `update_render_elements` levels. A layout-owned value touches fewer
  upstream signatures (`upstream-divergence.md`).
