---
id: material-89fb6b
title: Determine whether a planar in-shader tilt of window and glass reads as 3D glass
status: doing
priority: 1
size: m
complexity: mid
process: direct
needs: [owner]
owner: spike/material-89fb6b
created: 2026-10-06T21:03:34Z
updated: 2026-10-06T21:51:02Z
started: 2026-10-06T21:29:41Z
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
  4. Rotate the exit point back and project it to screen coordinates for the backdrop sample. Only the backdrop sample is projected to screen.
  5. The ring uses the same slab-local incident ray, refracted with its light IOR to its interior depth (0.2 * thickness). Its band (filamentBand, arcPosition) is evaluated in slab-local coordinates. Keep the existing half-gap cap on the shared shift, applied to the slab-local displacement.

  Do not add slab-local offsets to element coordinates. State the approximations in the result: faked slab, backdrop as the untilted screen plane.
- Enlarge the element area to the tilted footprint.
- No animation, triggers, config keys or live host.

Fixture (identical inputs for every render):
- A fixed patterned backdrop: a checker or grid background surface.
- A window buffer of opaque glyph-like strokes over a translucent fill.
- Frozen renders at rest and mid-beam.
- Comparisons:
  - Clean appearance: decorations off, with two controls rendered from the same inputs.
    - Unchanged main, at a pinned baseline commit.
    - The unrotated pinhole render (tilt 0 under the camera model).
    The pinhole camera's rays are oblique away from the pane centre while today's are parallel, so even at 0 degrees it changes backdrop refraction, ring displacement and view-dependent shading. Under the accepted settings (thickness 31.2, IOR 1.28) the backdrop sample moves about 3 px at 800 px from the centre.
    - Compare tilted renders against the unrotated pinhole, not against main.
    - Record the main vs unrotated-pinhole difference on its own, as the cost of the camera model.
    - Do not special-case exactly 0 degrees to match main. That would hide a jump as the swing settles, and a continuous transition back to today's rest optics is left to material-abc08c.
  - Decoration inventory, as a separate set: border, shadow and focus ring on, plus a popup and a subsurface, recording which follow the tilt.
- Native-resolution crops of text, the bevel edge and the ring, next to the full frames.

Expected result:
- Experimental code stays on an unmerged branch.
- Results return to main in one commit:
  - tasks attach of the contact sheet and crops;
  - a note with the branch commit, the baseline commit and the reproduction command;
  - a result section in the brief.
- Recorded:
  - the main vs unrotated-pinhole difference (where it moves, by how much);
  - the elements that stay flat;
  - grain behaviour;
  - text softness by angle;
  - the owner's verdict (need owner), separating geometric thickness from interior detail if it does not read as glass.

Ideas it wakes: On completion, run tasks note on material-6901e0 with the finding, in the same commit as this result.

## Notes

- 2026-10-06T21:11:20Z (materials-26.04): review: spec round 1 — verdict: revise; findings: P1 1, P2 3, correction 1; reviewer: codex/gpt-6-astra
- 2026-10-06T21:12:19Z (materials-26.04): review round 1 applied: verdict bounded to a planar projection with faked slab shading; pinned camera (d 2000 px, vertical axis, front-face pivot) and slab-local ray mapping replacing tap()'s element-UV offsets; patterned backdrop, glyphs over translucent fill, pinned main baseline and native crops, decoration inventory kept separate; code unmerged, evidence returned to main via tasks attach; subsurfaces follow the window texture, popups do not
- 2026-10-06T21:21:59Z (materials-26.04): review: spec round 2 — verdict: revise; findings: P1 1, P2 1; reviewer: codex/gpt-6-astra
- 2026-10-06T21:22:26Z (materials-26.04): review round 2 applied: controls are unchanged main and the unrotated pinhole; tilted renders compare against the unrotated pinhole; the main-vs-pinhole difference is recorded as the camera model's cost; no exact-zero special case; ring band stays slab-local with the half-gap cap, and only the backdrop sample is projected
- 2026-10-06T21:29:41Z (materials-26.04): review: spec round 3 — verdict: accept; findings: none; reviewer: codex/gpt-6-astra
- 2026-10-06T21:29:41Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T21:29:53Z (spike/material-89fb6b): resumed
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T21:50:04Z (spike/material-89fb6b): attached: overview-clean.png (1078838 bytes): Clean scene, pane region at 40%: main, 0-degree pinhole control, 2/4/8 degrees, at rest and mid-beam
- 2026-10-06T21:50:04Z (spike/material-89fb6b): attached: crops-clean.png (221301 bytes): Native crops per variant at fixed screen rects: near/far text, bevel corner (1x and 3x), ring mid-beam
- 2026-10-06T21:50:05Z (spike/material-89fb6b): attached: overview-deco.png (987276 bytes): Decoration inventory at rest: border, focus ring, shadow, subsurface (red) and popup (blue) per variant
- 2026-10-06T21:50:05Z (spike/material-89fb6b): attached: diff-pin0-main.png (25403 bytes): |0-degree pinhole - main| at rest, x8: the camera model's cost, on the bevel band
- 2026-10-06T21:50:05Z (spike/material-89fb6b): attached: tilt-steps.gif (591315 bytes): Stepped 0-2-4-8-4-2 degrees at rest, 60%: not real timing, a stepped view of the same stills
- 2026-10-06T21:50:05Z (spike/material-89fb6b): result: branch spike/material-89fb6b, baseline 69431712 (fixture only, renderer as main), spike 5bfba071. Reproduce: at either commit, TILT_SPIKE_DUMP=<dir> just test-one -p niri --run-ignored only -E 'test(tilt_spike_stills)', then python3 docs/materials/scripts/tilt-spike-sheet.py <dir> <out> at 5bfba071. Dumps in $NIRI_MATERIAL_WORK_ROOT/tilt-spike-89fb6b/{baseline,spike,sheets}.
- 2026-10-06T21:50:05Z (spike/material-89fb6b): finding: spike-off renders byte-identical to the baseline commit (4/4). 0-degree pinhole vs main: glyphs identical, face interior <=1 level, bevel band up to 55 levels (edge farthest from the centre) plus faint ring lines. Text sharpness (Laplacian variance vs control) 0.33-0.39 at near/far sides from 2 degrees on, 0.69/0.62/0.39 at the pivot: softening is a step on leaving the pixel grid, not proportional to angle. Subsurface follows the tilt; popup, border, focus ring and shadow stay flat. Grain and tap jitter stay screen-fixed (gl_FragCoord). Reads, to the agent, as a thick-framed flat card turned in perspective: no side walls (faked slab). Owner verdict pending.
- 2026-10-06T21:51:02Z (spike/material-89fb6b): parked (waiting on user, review): Owner: judge the stills attached to this task (crops-clean.png and tilt-steps.gif first; files in $NIRI_MATERIAL_WORK_ROOT/tilt-spike-89fb6b/sheets/): does a planar tilt read as 3D glass, is the text softening acceptable while moving, and if not, is the gap geometric thickness (side walls) or interior detail? Then agent: record the verdict as a note and in the brief's result section, note material-6901e0 with the finding, bring the record to main, tasks done.
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
