# Material motion

Type: guide; document types 1. How glass moves: the ring of light and the
beam that runs it on focus, what gates focus and attention motion, the
jelly and the follow lag that feeds it during a drag, and the rigid tilt
with its origin and its traced slab. Each rule is stated as
`src/render_helpers/material/ring.rs`, `src/render_helpers/material/view/`,
`src/layout/tile.rs`, `src/layout/drag_follower.rs`,
`src/layout/focus_origin.rs`, `src/activity.rs` and the material shaders
implement it. The `response` fields that configure the ring and the tilt are
in [signals.md](signals.md#signal-responses), `jelly-flex` and
`jelly-ripple` in [material-config.md](material-config.md), the optics the
ring passes through in [glass-optics.md](glass-optics.md), and the pass order
in [render-pipeline.md](render-pipeline.md).

Coordinates are element-local logical px, y down, +z toward the viewer.

## Ring of light

The ring lies on the beam line: the face the shader draws this frame, jelly
included, inset by `ring-gap`, its corner radii reduced by `ring-gap` and
floored at 0 (`ring::face`, `beam_perimeter`). A face whose half-width or
half-height is at most `ring-gap` has no line, and no ring is drawn on it.
Positions along the line (`arc_position`, mirrored by `arcPosition` in
`prelude.frag`) run clockwise from the end of the top-left corner's arc.

The focus light at arc position s is

```text
light(s) = 0.7 * ring-glow * (decay * (head(s) + tail(s)) + 0.2 * ring-rest)
```

(`comet`, mirrored by `main.frag`). The resting term is all that remains
between runs; it is constant, so a lit focused window at rest has a constant
fingerprint and no deadline. On the chamfer the moving part of the light,
head and tail, spills outward at a quarter of its strength (`BEAM_SPILL`),
fading to the silhouette; during a swing the side wall carries it at that
strength. Where the
light lands through the glass is capped as
[glass-optics.md](glass-optics.md#filament-shift-cap) describes.

## Ring beam

A run's head travels `ring-beam-speed` px per second from arc position 0
(`beam_frame`):

- **Head.** A Gaussian of the circular distance to the head, sigma 20 px
  (`BEAM_HEAD_SIGMA`). Its amplitude fades in over 300 ms (`BEAM_FADE`) and
  out over the last 300 ms of the lap, each at most half the lap, and is
  exactly 0 from the lap's end on.
- **Tail.** Behind the head, a quarter of the perimeter long, at most
  1200 px (`tail_length`): zero at the head, rising within a head width,
  scaled by 0.6 (`BEAM_TAIL_START`), and tapering to 0 with zero slope at
  its end (`tail_term`).
- **Wander.** With `ring-beam-noise` above 0, the head's amplitude is
  multiplied by one octave of value noise at `ring-beam-noise-hz`, within
  1 +/- `ring-beam-noise` and floored at 0 (`head_gain`), seeded by the
  pane's seed and the run's start. The tail and the resting glow do not
  wander.
- **Decay.** With `ring-beam-decay` above 0, head and tail share the factor
  (1 - d / decay)², d the head's distance travelled, reaching 0 at the decay
  distance (`distance_decay`).

The run ends at the head distance `run_length`: the perimeter plus the tail,
or the decay distance if shorter. The end is decided on a rendered frame,
against the perimeter that frame computed, so a pane that grows mid-run
keeps its beam until the tail clears the new perimeter. A beam that is never
rendered expires after 120 s (`BEAM_MAX_RUN`). The speed is snapshotted at
the start, so a reload that only changes `ring-beam-speed` leaves a running
beam alone.

## Focus activity

Every focus gain on a tile whose response has `focus "ring-light"` starts
the focus crossfade over `animations { material-signal }` and, when
`ring-beam-speed` is above 0, motion is `full` and animations are on, a new
beam, replacing a running one. Focus loss ends the beam at once. A running
beam is cut to rest when the policy, `animations { off }` or the response's
`focus` stops allowing it.

Focus and attention motion are gated two ways:

- **Visibility.** The beam, crossfades and impulses drive the animation
  loop only while the slab band is in view on a rendered workspace. A beam
  out of view keeps its clock, so it shows wherever its run has reached when
  the band comes back, usually rest. Sustained attention motion schedules
  its own deadlines under the same test.
- **Input activity.** After `signal { idle-after-ms }` without input,
  sustained attention motion becomes static and resumes on its absolute
  clock. The focus beam is finite and is not gated: a programmatic focus
  change while idle costs one run.

[signals.md](signals.md#redraw-and-idle-budget) gives the scheduler and the
idle gate in full. The beam is not a layout transition, so a run never holds
up the pointer-focus refresh.

## Jelly

Jelly makes the face trail motion against the fixed silhouette
(`jelly_state` in `src/render_helpers/material/mod.rs`). Its input is the
motion residual, the tile's render position minus its target from move
animations plus the drag lag below, and the size residual of a resize:

- **Shear.** The face moves by `jelly-flex` times the residual, toward
  where the window came from, saturated by tanh at a quarter of
  min(chamfer, thickness); the uniform is `mat_jelly_move`.
- **Resize.** Each axis's size residual, scaled and saturated the same way,
  resizes the face (`mat_jelly_resize`).
- **Activity.** tanh of the combined move and size residual's magnitude
  over half the target's diagonal, plus any `ripple` impulse's envelope, at most 0.999. While it is
  above 0 and `jelly-ripple` is set, `main.frag` perturbs the normal with
  two octaves of simplex noise moving with time, scaled by
  activity × `jelly-ripple`, and the ring glows up to three times brighter.

A zero residual is exactly rest. `JellyFingerprint` pins its time term at
zero activity, so resting jelly adds no damage.

## Drag follow lag

During an interactive move the tile follows the pointer exactly, and a
follower point chases it on a spring (`DragFollower`). The follower's lag
joins the motion residual, so only the jelly shows it. The spring is
`animations { window-movement }`'s, unit mass, evaluated in closed form from
its last anchor; each pointer step shifts the lag by the step and keeps the
velocity. There is no follower when that animation is off, an easing curve,
or of stiffness 0, and a changed spring takes over with lag and velocity
continuous. The follower outlives the drop and is dropped once its lag is
under 0.05 px and its speed under 1 px/s. Like the beam, it never holds up
the pointer-focus refresh.

## Rigid tilt

With `view-tilt` above 0, a focus gain swings the pane and its glass as one
rigid body (`ViewSwing` in `tile.rs`). With u = elapsed / `view-tilt-settle`:

```text
theta(u) = view-tilt * (1 - u)^2 * sin(2 pi u) / 0.61003     theta = 0 for u >= 1
```

It starts at exactly 0, peaks at u = 0.19 (171 ms at the default 900 ms),
crosses zero once at u = 0.5, overshoots to -0.164 of the peak at u = 0.636
and is exactly 0 from u = 1 (`swing_shape`). The axis lies in the glass plane
through the slab's centre on its front face, perpendicular to the swing
direction; the side facing the focus origin presses in first.

The camera sits on the pivot's normal at `view-distance` × E / w, E being
the slab's extent along the direction and w = |theta| / `view-tilt` the
perspective weight, so perspective engages only as far as the pane is
turned and rest is orthographic. A wall h px from the pivot, with outward
normal n, faces the camera when D sin|theta| (n · s) > h, D the camera
distance and s the direction signed by theta (`wall_faces_camera`); on a
horizontal swing the advancing wall shows on every moving frame once
sin(`view-tilt`) exceeds 1 / (2 × `view-distance`).

The swing's peak, settle, distance and direction are snapshotted at the
start. The tile must be eligible as
[signals.md](signals.md#focus-tilt) lists; a swing already running is
finished rather than restarted; a policy that stops allowing motion cuts it
to rest. The material program's `MATERIAL_VIEW` variant is bound only while
a swing runs, and the element's geometry is then the footprint described
in [render-pipeline.md](render-pipeline.md#under-a-swing-the-rigid-tilt).

## Swing origin

The `Layout` observes the globally focused window on its monitor once per
update pass and keeps the last observation (`FocusOrigin`: window, output,
tile centre in output-local coordinates). A newly focused window swings from
the unit vector between the two centres when the previous focus was on the
same output at least 1 px away (`COINCIDENT_PX`). Otherwise (no previous
focus, another output, coincident centres) it takes a direction seeded by
the pane's seed and the start instant (`seeded_direction`). Refocusing the
same window starts nothing. A focused window whose workspace is culled off
the output leaves the origin as it was.

## Tilted slab trace

During a swing every fragment of the footprint traces its own ray through
the rotated slab (`viewBegin`, `viewTrace` and `viewHit` in `prelude.frag`;
`view/solid.rs` mirrors them in f64):

1. The camera ray through the screen point is turned into slab-local
   coordinates. The window is a decal on the front plane: it is sampled at
   the ray's plane hit, and an opaque texel is returned as it is.
2. A plane hit on the flat face is the glass hit. Elsewhere the ray's
   segment from the front plane to the back plane is intersected with the
   resting slab as a convex solid: split where it crosses the silhouette's
   centre lines (at most three pieces), each piece minimised by golden-section
   search (`VIEW_GOLDEN_STEPS`, 28 steps), and the entry found by bisection
   (`VIEW_BISECT_STEPS`, 24). An entry above the rim is a top hit, one below
   it a wall hit, no entry a miss, which shows the scene beneath.
3. The hit is shaded with the resting optics at the hit point; the
   refracted, reflected and attenuated paths use the slab-local ray, and
   every backdrop sample goes back to the screen through the same
   projection (`viewScreenShift`).
4. Coverage comes from the projected outline; grain and tap jitter seeds
   move with the glass by the hit's whole-pixel offset.

No point drawn is nearer the camera than three quarters of its distance
(`VIEW_NEAR_CLIP`); the decal is dropped past it, and the footprint is
clipped at the same plane. Jelly shapes the shading at the hit but not the
traced solid. Every correction is exactly zero at theta = 0, so the last
moving frame meets the rest frame. `src/tests/view_tilt.rs` pins frozen
renders of the swing.
