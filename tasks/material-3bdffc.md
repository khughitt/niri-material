---
id: material-3bdffc
title: Design attenuation tint from the glass signal responses brief
status: doing
priority: 2
size: s
complexity: high
process: planned
owner: material-3bdffc
created: 2026-09-29T21:31:05Z
updated: 2026-10-03T10:25:15Z
started: 2026-10-03T09:33:22Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, design]
source: "docs/notes/2026-09-29-glass-signal-responses-brief.md#attenuation-tint"
agent: codex
spec: docs/specs/2026-10-03-accent-tint-design.md
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
