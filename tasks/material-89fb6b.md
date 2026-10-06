---
id: material-89fb6b
title: Determine whether a planar in-shader tilt of window and glass reads as 3D glass
status: todo
priority: 1
size: m
complexity: mid
process: direct
needs: [owner]
created: 2026-10-06T21:03:34Z
updated: 2026-10-06T21:12:20Z
depends: []
parent: material-53f873
tags: [rendering, camera]
source: docs/notes/2026-10-06-transient-depth-brief.md
agent: claude-code/claude-opus-5-5
---

Question: Does a fixed planar tilt of a few degrees read as 3D glass, and how soft does text get? The tilt is applied inside the material pass, so window content and glass move as one body, using today's fragment-faked slab shading and ring. The verdict covers only that planar projection. If it does not read as glass, record missing geometric thickness (side walls, occlusion) separately from missing interior detail. A negative result does not show that a rigid tilt needs particles or a richer interior.

Where to start: docs/notes/2026-10-06-transient-depth-brief.md (alternative 1); src/render_helpers/shaders/material/main.frag (the niri_tex_win sample, the tap call sites) and prelude.frag (slabSurface, tap, lightShift); src/render_helpers/material/mod.rs (element area near 816, draw near 830); src/tests/ring_look.rs (frozen renders, RING_LOOK_DUMP); src/window/mapped.rs render_normal (subsurfaces render into the window texture; popups render separately); the view-ray routing on the unmerged branch material-77db8a.

Bound:
- Projection, pinned for every still:
  - A pinhole camera on the pane's centre axis, at d = 2000 logical px in front of the screen plane.
  - Rotation about the pane's vertical centre axis, with the pivot on the slab's front face.
  - Tilts of 0, 2, 4 and 8 degrees.
- Per fragment:
  1. Cast the camera ray through the screen pixel and rotate it into slab-local coordinates.
  2. Intersect the front-face plane, and sample the window texture and slabSurface at that point.
  3. Refract the slab-local ray at the slab normal and carry it through thickness.
  4. Rotate the exit point back and project it to screen coordinates for the backdrop sample.
  5. Apply the same mapping to the ring's light shift.

  Do not add slab-local offsets to element coordinates. State the approximations in the result: faked slab, backdrop as the untilted screen plane.
- Enlarge the element area to the tilted footprint.
- No animation, triggers, config keys or live host.

Fixture (identical inputs for every render):
- A fixed patterned backdrop: a checker or grid background surface.
- A window buffer of opaque glyph-like strokes over a translucent fill.
- Frozen renders at rest and mid-beam.
- Comparisons:
  - Clean appearance: decorations off. A pinned main baseline commit is rendered with the same inputs; the tilt-0 render is compared to it byte for byte.
  - Decoration inventory, as a separate set: border, shadow and focus ring on, plus a popup and a subsurface, recording which follow the tilt.
- Native-resolution crops of text, the bevel edge and the ring, next to the full frames.

Expected result:
- Experimental code stays on an unmerged branch.
- Results return to main in one commit:
  - tasks attach of the contact sheet and crops;
  - a note with the branch commit, the baseline commit and the reproduction command;
  - a result section in the brief.
- Recorded:
  - the tilt-0 identity check;
  - the elements that stay flat;
  - grain behaviour;
  - text softness by angle;
  - the owner's verdict (need owner), separating geometric thickness from interior detail if it does not read as glass.

Ideas it wakes: On completion, run tasks note on material-6901e0 with the finding, in the same commit as this result.

## Notes

- 2026-10-06T21:11:20Z (materials-26.04): review: spec round 1 — verdict: revise; findings: P1 1, P2 3, correction 1; reviewer: codex/gpt-6-astra
- 2026-10-06T21:12:19Z (materials-26.04): review round 1 applied: verdict bounded to a planar projection with faked slab shading; pinned camera (d 2000 px, vertical axis, front-face pivot) and slab-local ray mapping replacing tap()'s element-UV offsets; patterned backdrop, glyphs over translucent fill, pinned main baseline and native crops, decoration inventory kept separate; code unmerged, evidence returned to main via tasks attach; subsurfaces follow the window texture, popups do not
