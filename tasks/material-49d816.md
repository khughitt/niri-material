---
id: material-49d816
title: Pixel-test backdrop grain above scale 1
status: done
priority: 3
size: s
complexity: low
process: direct
owner: noise-followups
created: 2026-10-07T03:19:43Z
updated: 2026-10-08T09:45:05Z
started: 2026-10-08T09:39:41Z
completed: 2026-10-08T09:45:05Z
depends: []
parent: material-3aa1f2
tags: [testing, noise]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

Whole-branch review of material-3fcba2 (round 1, Minor): lattice normalisation, position classes and fractional scale are pixel-tested only at the glass site; noise_source at scale > 1 has only a damage test. A wrong backdrop scale wiring in GrainOptions::uniforms or grain.rs would leave backdrop grain at scale 1 with no failing render test. Add an in-process test rendering a backdrop layer at scale 4 and checking its low-frequency ratio rises over scale 1 and its deviation holds.

## Notes

- 2026-10-08T09:39:41Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:39:59Z (noise-followups): resumed
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:45:05Z (noise-followups): done
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:45:05Z (noise-followups): src/tests/noise_layers.rs: backdrop_grain_coarsens_with_scale_and_keeps_its_deviation renders a fine backdrop layer at scale 1 and 4; sd holds within 10% and the 4x4 low-frequency ratio rises 0.123 -> 0.778 (asserts +0.2). Forcing GrainOptions::uniforms' scale to 1 fails it (0.123 vs 0.123).
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
