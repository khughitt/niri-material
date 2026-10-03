---
id: material-3bdffc
title: Design attenuation tint from the glass signal responses brief
status: doing
priority: 2
size: s
complexity: high
process: planned
owner: materials-26.04
created: 2026-09-29T21:31:05Z
updated: 2026-10-03T09:33:22Z
started: 2026-10-03T09:33:22Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, design]
source: "docs/notes/2026-09-29-glass-signal-responses-brief.md#attenuation-tint"
agent: codex
---

Why: material-6f45a0 has a clear visual outcome and neutral-path check, but the existing accent selector only supports ring/none and does not settle tint weight or composition with ring identity.

Done: produce a reviewable draft design under docs/specs/ covering the response spelling, bounded weight/default, coexistence with ring accent, color space and missing-accent/presence behavior. Reuse Tile::signal_for_frame's existing accent/presence crossfade and the attenuation stage. Specify zero-weight/default byte equivalence, accent replacement/removal captures, unchanged opaque pixels and no redraws once settled. Obtain spec review, then prepare the implementation plan for its separate review; no rendering implementation in this design task.

Where to start: docs/notes/2026-09-29-glass-signal-responses-brief.md; niri-config/src/material/mod.rs (AccentResponse, Response, ResolvedResponse); src/layout/tile.rs (signal_for_frame); src/render_helpers/signal.rs (SignalFrame); src/render_helpers/material/mod.rs; src/render_helpers/shaders/material/main.frag; docs/materials/render-pipeline.md.

Bound: attenuation tint only; fireflies, frost, progress IPC, client desaturation and Prism changes remain separate scope. Record the design finding on material-6f45a0 in the same commit as the result so the idea can be reconsidered.

## Notes

- 2026-10-03T09:33:22Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:e945f374-1ee7-46ac-8b9e-b3e3f024538c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
