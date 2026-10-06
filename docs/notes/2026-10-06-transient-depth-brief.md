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

1. **Rigid tilt inside the material pass (current lean).** The fragment shader
   maps each screen pixel back through the tilt's homography into slab-local
   coordinates. It samples the window texture and the slab geometry there,
   and takes the view ray from the tilted slab normal (77db8a's routing
   becomes the optical half). Backdrop taps stay at true screen positions.
   - Gains: window and glass move as one body; refraction of the backdrop is
     physically right; the element and its pass order stay as they are.
   - Costs:
     - A larger element area during the swing.
     - Text resampled bilinearly while it moves.
     - Elements outside the material (border, shadow, a separate focus ring)
       stay flat unless they are handled too.
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

- **Does it read as 3D glass?** Does a rigid tilt of a few degrees on today's
  slab and ring read as 3D glass, and how much text softening is acceptable
  while it moves? Answered by the owner, from the frozen stills of the spike
  task below.
- **What tilts with the pane?** Border, shadow, the separate focus ring, and
  popups or subsurfaces outside the window texture. The spike inventories
  them; the design decides.
- **Which transitions, and what motion each?** The lean is focus gain first,
  reusing 77db8a's curve and focus origin, then move, resize and open. The
  owner chooses the later ones after the first is seen.
- **The pivot.** A rotation about the pane centre, or a door-like hinge facing
  the focus origin. Settled in the design.
- **Interior depth.** Is today's interior non-uniform enough to show depth
  when tilted, or does the payoff need a volumetric interior or particle
  light? The spike's stills show the gap; any interior work joins
  `material-4e3e9c` or `material-d0518d`.

## Proposed decomposition

- `material-6901e0` stays an idea, waiting on the spike and the design below.
- `material-89fb6b`, the spike (P1, direct, owner): frozen stills of a fixed rigid tilt in the
  material pass, with no animation, on a throwaway branch. It records the
  element inventory and the identity check, and it wakes `material-6901e0`.
- `material-abc08c`, the design (P1, planned, high, owner, depends on
  `material-89fb6b`): one transient
  rigid tilt with finite settling. The first trigger is focus gain.
- 77db8a ran a full implementation before the owner saw a single frame.
  Here, the stills come before any design.
