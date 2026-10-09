---
id: material-abc08c
title: Design a transient rigid tilt of window and glass from the transient-depth brief
status: doing
priority: 1
size: m
complexity: high
process: planned
needs: [owner]
owner: material-abc08c
created: 2026-10-06T21:03:34Z
updated: 2026-10-09T17:33:41Z
started: 2026-10-09T12:24:44Z
depends: [material-89fb6b, material-be611b]
parent: material-53f873
tags: [rendering, camera, dynamics]
source: docs/notes/2026-10-06-transient-depth-brief.md
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-09-transient-rigid-tilt-design.md
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

## Notes

- 2026-10-06T21:12:26Z (materials-26.04): From spike review round 1: the design must also settle whether the faked slab suffices or a real ray-slab intersection (side walls, occlusion) is needed, and adopt one camera model and slab-local ray mapping for refraction and light shift (see the brief's alternative 1).
- 2026-10-06T21:22:26Z (materials-26.04): From spike review round 2: the pinhole camera changes refraction, ring displacement and shading even at zero tilt (about 3 px at 800 px from centre with the accepted glass). The design must return continuously to today's parallel-ray rest optics as the swing settles; an exact-zero bypass does not satisfy this. The ring evaluates its band in slab-local coordinates; only the backdrop sample is projected to screen.
- 2026-10-06T22:10:40Z (materials-26.04): From the spike verdict (material-89fb6b): scope the design to side walls first, interior detail second. Build the walls on material-be611b's height-field edge profile so the resting and tilted glass share one slab model (be611b is parked on owner review of its sheet; this design depends on its accepted profile). Judge stills on a lighter look beside the accepted dark one: the dark terminal fill hides the glass. Reusable: spike/material-89fb6b (camera ray, quad mapping, fixture, sheet script).
- 2026-10-09T12:24:44Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T12:25:01Z (material-abc08c): resumed
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T12:38:15Z (material-abc08c): spec drafted: docs/specs/2026-10-09-transient-rigid-tilt-design.md. Finding: a centred camera hides outer walls until tan(theta) > E/(2D); spike camera (D 2000) shows none below 11-22 deg, active-slab wall is 1-4 px at 10-14 deg. Design: camera distance relative to pane extent (view-distance), perspective weight |theta|/peak so rest optics return continuously, impulse curve starting at 0 (77db8a's starts at its peak), true ray-slab march with height-field top and side walls, wall lit by beam spill, slab-anchored grain seeds, owner still sweep (angle x distance) before defaults
- 2026-10-09T12:38:25Z (material-abc08c): parked (waiting on user, review): Owner reviews docs/specs/2026-10-09-transient-rigid-tilt-design.md in .worktrees/material-abc08c; on approval the agent runs writing-plans there
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T13:46:43Z (material-abc08c): resumed
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T13:46:43Z (material-abc08c): review: spec round 1 — verdict: revise; findings: must 3, should 1; reviewer: codex/gpt-6.1-sol
- 2026-10-09T13:58:14Z (material-abc08c): spec round 1 addressed in 35da7cdf: footprint adds texture corners; reveal rule from camera and wall normals; convex-solid bracketing (golden + bisect) with outline bound; rest-path A/B cost in 8.5 with a program-variant contingency
- 2026-10-09T13:58:14Z (material-abc08c): parked (waiting on user, review): Owner (or a fresh reviewer) reviews the revised docs/specs/2026-10-09-transient-rigid-tilt-design.md in .worktrees/material-abc08c as spec round 2; on approval the agent runs writing-plans there
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T17:33:40Z (material-abc08c): resumed
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T17:33:40Z (material-abc08c): review: spec round 2 — verdict: revise; findings: must 4; reviewer: unknown
