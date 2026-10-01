---
id: material-4354cf
title: Design a follow-lag jelly stimulus for interactive drag
status: doing
priority: 1
size: m
complexity: high
process: planned
owner: material-4354cf
created: 2026-09-30T10:00:19Z
updated: 2026-10-01T12:10:35Z
started: 2026-10-01T11:31:55Z
depends: []
parent: material-53f873
tags: [dynamics]
source: docs/notes/2026-09-29-material-dynamics-brief.md
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-01-drag-follow-lag-design.md
---

Outcome: glass responds to pointer drag, not only to the layout's lift and release animations. material-b3ce14 measured the jelly residual at exactly 0 during drag and hold in both layouts (src/layout/tests/drag_dynamics.rs; brief section 'Drag baseline finding'). Candidate contract: during InteractiveMoveState::Moving the tile renders at the pointer while a critically damped point chases that location on the window-movement spring; the point's lag is the jelly residual, decaying to 0 on a hold, and release animate_move_from starts from the lagged point so there is no jump. Owner decisions the spec must put to review: whether a held drag deforms at all, gain (jelly-flex or a separate drag gain), spring (window-movement or dedicated), whether long tiled drops keep saturating at max flex, whether the rubber band flexes. Acceptance: extend drag_dynamics.rs to pin lag decay to zero on hold and release continuity; a nested drag/hold/release clip beside the native column-move control for owner judgment. Preserve finite settling, idle-budget and render order. Out of scope: the max-flex cap, whole-slab shear and ior-independent ripple levers, which stay on material-6d4de5.

## Notes

- 2026-09-30T10:00:29Z (materials-26.04): concerns: material-b3ce14 extension — design the drag stimulus the baseline found missing
- 2026-10-01T11:31:55Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:12fc6cfe-a05c-422d-9279-35d443d3ffb2","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T11:31:58Z (material-4354cf): resumed
  provenance: {"harness_session":"claude-code:12fc6cfe-a05c-422d-9279-35d443d3ffb2","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T11:40:48Z (material-4354cf): review: spec round 1 — verdict: revise; findings: P1 3, P2 6, P3 4; reviewer: claude-code/claude-opus-5-5
- 2026-10-01T11:42:00Z (material-4354cf): Spec round 1 dispositions: P1-1 follower anchored at last shift and read lazily at clock.now(), never stepped in advance_animations; P1-2 signed f64 dt with backwards-clock test; P1-3 steady-lag test pins the discrete fixed point (156.79 px recorded) and the ±d/2 band, flex figures corrected to 1.00–1.22 px live. P2-4 staggered 4 ms events via set_unadjusted against a reference follower and Euler; P2-5 replaced baseline assertions named; P2-6 follower survives regrab and ignores stop_move_animations; P2-7 all damping ratios plus Spring::velocity_at; P2-8 update_config drops on off/easing/complete-instantly and re-anchors on spring change; P2-9 clip driver adds virtual keyboard, pilot checks drag starts, xdg_toplevel.move client fallback. P3 zoom rescale, release read timing, unmap snapshot freeze and pointer warps stated.
- 2026-10-01T11:43:46Z (material-4354cf): review: spec round 2 — verdict: revise; findings: P1 1, P2 1, P3 5; reviewer: claude-code/claude-opus-5-5
- 2026-10-01T11:44:04Z (material-4354cf): Spec round 2 dispositions: P1 drag check compares every recorded frame to a reference DragFollower driven through the same events (1e-9), plus the 156.79 px fixed point on a separate 50-frame drag; the ±d/2 band is dropped. P2 complete-instantly is checked where Animation::is_done checks it (advance_animations drops the follower, lag reads zero), not in update_config, which a reload calls before it sets the flag; tested through Op::CompleteAnimations. P3: 485 px reset figure, ±10/±5 px sawtooth, 'delivered events' wording, the anchor is the only state, are_animations_ongoing tests presence.
- 2026-10-01T11:44:45Z (material-4354cf): review: spec round 3 — verdict: accept; findings: P3 1; reviewer: claude-code/claude-opus-5-5
- 2026-10-01T11:44:45Z (material-4354cf): Spec round 3 disposition: the 50-event fixed-point check moved to the DragFollower unit tests, because 2000 px of pointer travel would leave the 1280×720 test output; the layout test keeps the per-frame reference comparison.
- 2026-10-01T11:44:53Z (material-4354cf): parked (waiting on user, review): Owner reviews .worktrees/material-4354cf/docs/specs/2026-10-01-drag-follow-lag-design.md, especially the §4 decisions (no held deformation, reuse jelly-flex, window-movement spring, tiled drops unchanged, no rubber-band flex) and the residual-only lag in §3. Then the agent records the owner round, revises or, on acceptance, drafts the implementation plan for separate review.
  provenance: {"harness_session":"claude-code:12fc6cfe-a05c-422d-9279-35d443d3ffb2","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T12:07:16Z (material-4354cf): review: spec round 4 — verdict: revise; findings: P1 1, P2 2; reviewer: codex/gpt-6
- 2026-10-01T12:07:16Z (material-4354cf): Spec round 4 findings: P1 (§5 lines 189–191): direct Rust probe of existing Spring::value_at, damping-ratio 10 / stiffness 800 / from -40 / velocity 0, yields -inf at 2.512 s and NaN at 3 s before the physical lag settles; require a stable overdamped evaluation and a high-damping hold-to-settle check. P2 (§4.4 lines 168–171): adding L can cancel or reverse pointer-minus-slot residual, so only the cap and position animation are unchanged, not release flex or saturation; document the accepted behavior and check opposing residuals. P2 (§6 lines 239–244): existing traces hold for 30 frames before release, settling the follower; require a release during motion with an asserted nonzero lag in both layouts so release continuity cannot pass vacuously.
- 2026-10-01T12:10:21Z (material-4354cf): Spec round 4 dispositions: P1 the follower uses its own closed form, with the overdamped case as two decaying exponentials (λ = −β ± ω₂), leaving upstream Spring untouched; debug-asserts a finite state; high-damping unit test (ratio 10, 60 s, finite, Euler match, settles). P2 §4.4 now says the cap and window trajectory are unchanged while release flex can fall, cancel, briefly reverse or saturate sooner, with an opposing-release test. P2 a separate release-in-motion trace per layout asserts a lag above 10 px before release continuity is checked.
- 2026-10-01T12:10:35Z (material-4354cf): parked (waiting on user, review): Owner re-reviews .worktrees/material-4354cf/docs/specs/2026-10-01-drag-follow-lag-design.md after round-4 revisions (c89f803a): §5 stable overdamped closed form, §4.4 release flex may change while cap and trajectory stay, §6 release-in-motion and opposing-release tests. On acceptance the agent records the round and drafts the implementation plan for separate review.
  provenance: {"harness_session":"claude-code:12fc6cfe-a05c-422d-9279-35d443d3ffb2","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
