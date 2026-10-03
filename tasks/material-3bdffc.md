---
id: material-3bdffc
title: Design attenuation tint from the glass signal responses brief
status: done
priority: 2
size: s
complexity: high
process: planned
owner: material-3bdffc
created: 2026-09-29T21:31:05Z
updated: 2026-10-03T11:20:58Z
started: 2026-10-03T09:33:22Z
completed: 2026-10-03T11:20:58Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, design]
source: "docs/notes/2026-09-29-glass-signal-responses-brief.md#attenuation-tint"
model: claude-opus-5-5
agent: codex
spec: docs/specs/2026-10-03-accent-tint-design.md
plan: docs/plans/2026-10-03-accent-tint.md
---

Why: material-6f45a0 has a clear visual outcome and neutral-path check, but the existing accent selector only supports ring/none and does not settle tint weight or composition with ring identity.

Done: produce a reviewable draft design under docs/specs/ covering the response spelling, bounded weight/default, coexistence with ring accent, color space and missing-accent/presence behavior. Reuse Tile::signal_for_frame's existing accent/presence crossfade and the attenuation stage. Specify zero-weight/default byte equivalence, accent replacement/removal captures, unchanged opaque pixels and no redraws once settled. Obtain spec review, then prepare the implementation plan for its separate review; no rendering implementation in this design task.

Where to start: docs/notes/2026-09-29-glass-signal-responses-brief.md; niri-config/src/material/mod.rs (AccentResponse, Response, ResolvedResponse); src/layout/tile.rs (signal_for_frame); src/render_helpers/signal.rs (SignalFrame); src/render_helpers/material/mod.rs; src/render_helpers/shaders/material/main.frag; docs/materials/render-pipeline.md.

Bound: attenuation tint only; fireflies, frost, progress IPC, client desaturation and Prism changes remain separate scope. Record the design finding on material-6f45a0 in the same commit as the result so the idea can be reconsidered.

## Notes

- 2026-10-03T09:33:22Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T09:33:33Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T09:42:43Z (material-3bdffc): spec drafted: docs/specs/2026-10-03-accent-tint-design.md; owner chose density-preserving hue tint (accent rescaled to attenuation-color luminance) over plain mix on 2026-10-03; CPU-side via glass_signal_inputs, no shader change
- 2026-10-03T09:42:54Z (material-3bdffc): parked (waiting on user, review): Owner reviews docs/specs/2026-10-03-accent-tint-design.md in .worktrees/material-3bdffc (spec round 1); on accept, agent writes the implementation plan in docs/plans/ for its own review
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T09:54:43Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T09:54:43Z (material-3bdffc): review: spec round 1 — verdict: revise; findings: P1 1, P2 1, P3 3; reviewer: claude-code/opus-5.5
- 2026-10-03T09:56:50Z (material-3bdffc): spec revised for round 2: density matched on face transmittance (T-space, p_f = thickness/attenuation-distance, two-sided gamut [0.001^p_f, 1]); stated chamfer/ring movement and near-invisible tint on light glass; added fingerprint test, magenta dumps framing the ring band; sRGB encode dropped
- 2026-10-03T09:56:53Z (material-3bdffc): parked (waiting on user, review): Owner reviews docs/specs/2026-10-03-accent-tint-design.md in .worktrees/material-3bdffc (spec round 2); on accept, agent writes the implementation plan in docs/plans/ for its own review
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:23:27Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:23:27Z (material-3bdffc): review: spec round 2 — verdict: revise; findings: P2 3; reviewer: unknown (pasted review)
- 2026-10-03T10:25:15Z (material-3bdffc): spec revised for round 3: tint chromaticity k interpolates between crossfade endpoints (black<->colored continuous; FrameInputs/SignalFrame carry tint_chroma); ring table restated with accent-colored light (accent band brightens 1.1-1.7x, ring-color band dims to 0.47-0.87x at w=1); face-density promise qualified to neutral backdrops; dumps add colored backdrops and black<->orange fades
- 2026-10-03T10:25:18Z (material-3bdffc): parked (waiting on user, review): Owner reviews docs/specs/2026-10-03-accent-tint-design.md in .worktrees/material-3bdffc (spec round 3); on accept, agent writes the implementation plan in docs/plans/ for its own review
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:34:27Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:34:27Z (material-3bdffc): correction: spec round 2 reviewer was codex/gpt-6 (recorded above as unknown)
- 2026-10-03T10:34:27Z (material-3bdffc): review: spec round 3 — verdict: accept; findings: none; reviewer: codex/gpt-6
- 2026-10-03T10:43:00Z (material-3bdffc): Implementation plan drafted: docs/plans/2026-10-03-accent-tint.md (6 tasks). Self-review: every spec section mapped to a task; five review-focus cases (focus-split swap, weight-only reload, animations off, accent none, neighbor accent) carry tests in Task 5. Execution children under material-6f45a0: material-01ddd7, material-558ce5, material-49e73d, material-77cb15, material-e2a68a, material-76a30f, chained from this design task. Native sequential execution recommended.
- 2026-10-03T10:43:05Z (material-3bdffc): parked (waiting on user, review): Owner reviews docs/plans/2026-10-03-accent-tint.md in .worktrees/material-3bdffc (plan round 1) and picks an execution method; agent records the review, revises or, on accept, notes reviewed decisions on material-6f45a0 and closes this design task in the result commit
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:51:13Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:51:13Z (material-3bdffc): review: plan round 1 — verdict: revise; findings: P2 5; reviewer: codex/gpt-6
- 2026-10-03T10:54:57Z (material-3bdffc): Plan round 1 dispositions: P2-1 Task 4 adds a tile-level end-to-end test (folded signal -> signal_for_frame -> solve -> glass_signal_inputs -> uploaded attenuation) over a linear fade for black<->orange at the spec fractions and a real interrupted restart; P2-2 animations-off test injects the accent into a settled window and checks the first frame at the same instant against the settled tint, with a 400 ms control that is still untinted; P2-3 focus test swaps one settled fixture away and back, checking each first frame; P2-4 reload test asserts the material element's id@commit on RenderTarget::Output changes once after a weight-only reload and then holds; P2-5 dumps use the accepted look's full glass and response (ring-gap 6, width 1.1, glow 1.2, beam settings). Native execution chosen by owner; whole-branch review placed before the local merge in Task 6.
- 2026-10-03T10:55:01Z (material-3bdffc): parked (waiting on user, review): Owner reviews the round-1 revision of docs/plans/2026-10-03-accent-tint.md in .worktrees/material-3bdffc (plan round 2); on accept, agent notes reviewed decisions on material-6f45a0, closes this design task, and executes natively from material-01ddd7
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:14:18Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:14:18Z (material-3bdffc): review: plan round 2 — verdict: revise; findings: P2 1; reviewer: codex/gpt-6
- 2026-10-03T11:14:59Z (material-3bdffc): Plan round 2 disposition: P2 confirmed (fixture.rs niri_complete_animations ends with set_complete_instantly(false); animations { off } is that flag via Niri::new/reload). Task 5 Step 1 now saves and restores should_complete_instantly() in the helper; added completing_animations_keeps_the_configured_clock_setting for both initial settings; first-frame assertions unchanged. No existing test configures animations off, so the helper change is neutral for them.
- 2026-10-03T11:15:02Z (material-3bdffc): parked (waiting on user, review): Owner reviews the round-2 revision of docs/plans/2026-10-03-accent-tint.md in .worktrees/material-3bdffc (plan round 3); on accept, agent notes reviewed decisions on material-6f45a0, closes this design task, and executes natively from material-01ddd7
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:20:47Z (material-3bdffc): resumed
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:20:47Z (material-3bdffc): review: plan round 3 — verdict: accept; findings: none; reviewer: codex/gpt-6
- 2026-10-03T11:20:58Z (material-3bdffc): done
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:20:58Z (material-3bdffc): Spec and plan accepted (round 3 each); design finding recorded on material-6f45a0; execution children chained from material-01ddd7
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
