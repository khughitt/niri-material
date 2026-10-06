---
id: material-89fb6b
title: Determine whether a rigid in-shader tilt of window and glass reads as 3D glass
status: todo
priority: 1
size: m
complexity: mid
process: direct
needs: [owner]
created: 2026-10-06T21:03:34Z
updated: 2026-10-06T21:03:34Z
depends: []
parent: material-53f873
tags: [rendering, camera]
source: docs/notes/2026-10-06-transient-depth-brief.md
agent: claude-code/claude-opus-5-5
---

Question: Does a fixed rigid tilt of a few degrees, applied inside the material pass so window content and glass move as one body, make today's slab and ring read as 3D glass, and how soft does text get?
Where to start: docs/notes/2026-10-06-transient-depth-brief.md (alternative 1); src/render_helpers/shaders/material/main.frag (niri_tex_win sample, slab geometry, view ray) and prelude.frag; src/render_helpers/material/mod.rs (element area near 816, draw near 830); src/tests/ring_look.rs (frozen renders, RING_LOOK_DUMP); the unmerged branch material-77db8a for its view-ray routing.
Bound: A throwaway branch, never merged. Inverse-map each fragment through a fixed homography (pivot at the pane centre, tilt 0, 2, 4 and 8 degrees about one axis) for the window sample, slab geometry and view ray; backdrop taps stay at true screen position; enlarge the element area to the footprint. No animation, triggers, config keys or live host. Render frozen stills through the ring_look path with a high-contrast text-like window texture, at rest and mid-beam.
Expected result: A contact sheet for the owner (need owner) and a note here and in the brief recording: whether tilt 0 is byte-identical to main; the elements that stay flat (border, shadow, focus ring, popups, subsurfaces); grain behaviour; text softness by angle; and the owner's verdict on whether it reads as 3D glass, or what interior depth is missing.
Ideas it wakes: On completion, run tasks note on material-6901e0 with the finding, in the same commit as this result.
