# Transient depth scoping brief

2026-10-06. Handoff for `material-6901e0` under the motion lane
`material-53f873`, not an approved design.

## Problem

The glass is modelled as a 3D slab, but it is seen only face-on, so its depth
never shows: the ring beam reads as a coloured rectangle rather than light
travelling through a non-uniform volume. The owner wants that depth visible
for a moment at key transitions (focus change, move, resize, open), then the
pane settles back to exactly face-on, with text at rest unchanged. The
reference look is three.js `MeshTransmissionMaterial` with particles moving
through it, seen at an angle. Long term, different transitions may carry
different motions.

The window's content and the glass must move as one body. That is the lesson
of `material-77db8a`.

## Current behaviour and evidence

- **Optical-only tilt, rejected.** `material-77db8a` swung the view ray on
  focus gain while window pixels and the slab silhouette stayed still. The
  owner rejected it on 2026-10-06: static text under moving glass breaks the
  one cohesive material. Its spec and plan are on main with that status. The
  code stays on the unmerged branch `material-77db8a` (`d51825f9`). The
  pieces that can be reused, all pure and tested on that branch:
  - `render_helpers/material/view.rs`: a swing curve that starts at θ0,
    overshoots once to about −0.18 θ0, and ends at exactly zero with zero slope.
  - `layout/focus_origin.rs`: a focus origin that the Layout observes once per
    update pass, and which starts the swing.
  - The shader's view-ray routing.
  - The smoke and clip scripts.
- **The material pass already draws the window.** The window body is rendered
  into the material's own offscreen buffer (`src/layout/tile.rs` near 2093).
  `main.frag` samples it as `niri_tex_win` and composites it with the slab in
  one fragment pass, and the glass element replaces the window body in the
  push order.
  - The backdrop is sampled through `xray_pos.pos_in_backdrop`, not through
    the framebuffer, so the glass can also be drawn offscreen. The open
    animation already does that: `Tile::render`, `opening_window.rs`.
- **No perspective draw path.** niri's and smithay's texture vertex shaders
  write w = 1. Custom programs (`shader_element.rs`) take only a fragment
  shader over an upright quad. A projective draw is therefore either computed
  per pixel in the fragment shader inside an enlarged quad, or done with a
  new vertex shader that has real w.
- **Input and damage.** Hit-testing uses layout position plus
  `render_offset` (`scrolling.rs` near 3027; `Tile::hit`), so a visual tilt
  does not move input. The glass element damages only when its inputs change
  (`material/mod.rs` near 683). An animated tilt would redraw its whole,
  larger footprint on every frame of the swing.
- **Frozen-frame tooling.** `src/tests/ring_look.rs` renders through a real
  focus gain on a pinned clock, and `RING_LOOK_DUMP=<dir>` writes PNGs, with
  no host needed.

## Constraints

- Hard requirements:
  - The [render order](../materials/render-pipeline.md) holds.
  - At rest, opaque client pixels stay byte-identical, and the rest path is
    today's code path.
  - Settling is finite, per the
    [settle contract](../specs/2026-09-29-sustained-optic-settling-design.md),
    including reduced and off motion.
  - Text never jostles at rest.
- Adjacent open work:
  - The focus-loss cue (`material-ccda38`) and transient light (`material-d873bf`).
  - The content optical stage (`material-d257d9`) and the bevel profile
    (`material-be611b`).
- Grain and tap jitter are seeded from `gl_FragCoord`: under a tilt they stay
  fixed to the screen unless they are re-seeded in slab space.

## Alternatives

1. **Rigid tilt inside the material pass (current lean).** One pinhole
   camera model:
   - The camera sits on the pane's centre axis, at a fixed distance `d` in
     front of the screen plane.
   - The pane rotates about one in-plane axis through its centre, with the
     pivot on the slab's front face.

   For each screen pixel, the shader:
   1. Casts the camera ray through that pixel.
   2. Rotates the ray into slab-local coordinates.
   3. Intersects it with the front-face plane, giving the slab-local point
      where the window texture and `slabSurface` are sampled.
   4. Refracts the slab-local ray at the slab normal and carries it through
      `thickness`.
   5. Maps the exit point back to screen coordinates (rotate, then project)
      before the backdrop sample. Only the backdrop sample is projected.
   6. For the ring, refracts the same slab-local ray with the light IOR to
      the interior depth and evaluates the band (`filamentBand`,
      `arcPosition`) in slab-local coordinates, keeping the half-gap cap.

   This replaces `tap()`'s fixed straight-down ray and its direct addition of
   offsets to element coordinates (`prelude.frag`). 77db8a's view routing
   then becomes this camera ray, not a separate tilt.

   The approximation, stated plainly: the slab stays the fragment-faked one
   (a 2D silhouette with simulated normals; `slabSurface` intersects no
   volume), and the backdrop is a plane lying on the untilted screen. A
   planar tilt therefore exposes no side walls and no depth-dependent
   occlusion.

   A pinhole camera also changes the optics at zero tilt, because its rays
   are oblique away from the pane centre while today's are parallel. With
   the accepted glass (thickness 31.2, IOR 1.28) and `d` = 2000, the backdrop
   sample moves about 3 px at 800 px from the centre. Keeping the rest path
   unchanged therefore needs a continuous transition back to parallel rays
   as the swing settles, for example the camera distance growing without
   bound. A special case at exactly zero tilt would hide a jump instead.
   The spike measures the size of this effect; `material-abc08c` designs
   the transition.
   - Gains:
     - Window and glass move as one body.
     - The backdrop is re-sampled through one consistent ray rather than
       warped with the pane.
     - The element and its pass order stay as they are.
   - Costs:
     - A larger element area during the swing.
     - Text resampled bilinearly while it moves.
     - Elements outside the material stay flat unless they are handled too:
       border, shadow, a separate focus ring and popups. Subsurfaces render
       into the material's window texture (`render_normal()` in
       `src/window/mapped.rs`), so they follow the tilt.
2. **Warp an offscreen of the whole tile**, as the open animation does.
   - Gains: the tile's other elements tilt for free.
   - Costs:
     - The backdrop is warped with the pane instead of re-sampled.
     - The framebuffer-copy background effect captures an empty texture.
     - Grain follows the pane.
     - A new perspective draw path.

   This remains the route for open and close, which already go offscreen.
3. **Move only the light.** Keep the camera and pane still and send light
   through the volume: an oblique beam through a non-uniform interior,
   scatter, and spill onto the edge (`material-d873bf`, `material-d0518d`,
   `material-4e3e9c`). This is coherent by construction, since nothing
   implies a moving camera. Face-on, though, depth shows only through
   refraction and scatter cues. It complements a tilt rather than replacing
   it: the tilt reveals the volume, and moving light gives it something to
   reveal.

Content placed at a depth inside the slab (G2 in `material-7f5751`) would also
make content and glass coherent under a view swing, but the text would then
shift inside the glass. It stays with `material-d257d9`.

The static perspective by screen position should be dropped. At rest the
camera faces the glass, and content must not skew at rest.

## Unanswered questions

- **Does it read as 3D glass?** Does a planar tilt of a few degrees, with
  today's faked slab shading and ring, read as 3D glass, and how much text
  softening is acceptable while it moves? Answered by the owner, from the
  frozen stills of the spike task below. The verdict covers only that
  planar projection: a negative result does not show that a rigid tilt
  needs particles or a richer interior.
- **What tilts with the pane?** Border, shadow, the separate focus ring and
  popups. Subsurfaces already follow the window texture. The spike
  inventories them; the design decides.
- **Which transitions, and what motion each?** The lean is focus gain first,
  reusing 77db8a's curve and focus origin, then move, resize and open. The
  owner chooses the later ones after the first is seen.
- **The pivot.** A rotation about the pane centre, or a door-like hinge facing
  the focus origin. Settled in the design.
- **What is missing, if it does not read as glass?** Two gaps are recorded
  separately:
  - Geometric thickness: side walls and occlusion would need a real
    ray-slab intersection in place of the faked slab.
  - Interior detail: a non-uniform volume, or particle light.

  Interior work joins `material-4e3e9c` or `material-d0518d`. Geometric
  thickness would be a new design question for `material-abc08c`.

## Proposed decomposition

- `material-6901e0` stays an idea, waiting on the spike and the design below.
- `material-89fb6b`, the spike (P1, direct, owner): frozen stills of a fixed planar tilt under
  the pinned camera model, with no animation.
  - The experimental code stays on an unmerged branch.
  - The results return to main: the contact sheet attached to the task, the
    branch commit, the reproduction command, and a result section here.
  - A clean appearance comparison against a pinned main baseline is kept
    separate from the decoration inventory.
  - It wakes `material-6901e0`.
- `material-abc08c`, the design (P1, planned, high, owner, depends on
  `material-89fb6b`): one transient
  rigid tilt with finite settling. The first trigger is focus gain.
- 77db8a ran a full implementation before the owner saw a single frame.
  Here, the stills come before any design.
