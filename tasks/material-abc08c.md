---
id: material-abc08c
title: Design a transient rigid tilt of window and glass from the transient-depth brief
status: todo
priority: 1
size: m
complexity: high
process: planned
needs: [owner]
created: 2026-10-06T21:03:34Z
updated: 2026-10-06T21:03:34Z
depends: [material-89fb6b]
parent: material-53f873
tags: [rendering, camera, dynamics]
source: docs/notes/2026-10-06-transient-depth-brief.md
agent: claude-code/claude-opus-5-5
---

Design one transient rigid tilt in which window content and glass move as one body and settle to exactly face-on, from docs/notes/2026-10-06-transient-depth-brief.md, after the owner's verdict on the spike stills (material-89fb6b). First trigger: focus gain, reusing the swing curve and focus origin from the unmerged branch material-77db8a (render_helpers/material/view.rs, layout/focus_origin.rs). Move, resize and open come later, possibly with different motions.

Done: a written design and implementation plan, reviewed by the owner before renderer code. It must settle:
- the pivot and axis (centre or a hinge toward the focus origin), magnitude and timing;
- which elements tilt with the pane (border, shadow, focus ring, popups, subsurfaces);
- grain seeding in slab space;
- element area and damage during the swing, and the rest path byte-identical to today;
- interaction with resize, open/close offscreen paths, jelly flex and the focus-loss cue (material-ccda38);
- repeated or rapid focus changes, reduced or off motion and animations off;
- whether the static by-position perspective is dropped (the brief leans drop).

Verification contract: frozen-clock swing sequences, exact rest and no deadlines after the finite run, identity at tilt 0, and an owner-reviewed clip on the nested lane before any defaults. Any live capture gets its own preparation step and smallest pilot.

On the design result, update the brief and run tasks note on material-6901e0 in the same commit.
