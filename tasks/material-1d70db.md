---
id: material-1d70db
title: "Ring: separate the resting ring's level from the moving beam's glow"
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: material-1d70db
created: 2026-10-02T00:08:50Z
updated: 2026-10-02T00:48:28Z
started: 2026-10-02T00:39:57Z
completed: 2026-10-02T00:48:28Z
depends: []
tags: [rendering, prism]
agent: claude-code/claude-opus-5-5
---

Owner ask (2026-10-01): turn the resting ring off, or down, without losing the comet transition. Today comet() in src/render_helpers/material/ring.rs is BEAM_BASE * glow * (decay*moving + BEAM_REST): ring-glow scales both, and BEAM_REST (0.2) is a constant. Add a native ring-rest level (0 = no resting ring, default 1 = today's BEAM_REST) on the response block, so ring-glow stays the overall gain and ring-rest scales only the resting term. When rest is 0, the tile must stay free of redraws after the run, as it is now. Then expose it in the Prism Ring group as glass.ring.rest (prism task). Check that the head and tail still read at rest 0, since today they ride on top of the rest level.

## Notes

- 2026-10-02T00:09:33Z (materials-26.04): Prism half: prism-f71919 (glass.ring.rest), which depends on this task
- 2026-10-02T00:39:57Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:40:00Z (material-1d70db): resumed
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:48:28Z (material-1d70db): done
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:48:28Z (material-1d70db): ring-rest (0-3, default 1) on the response block scales only BEAM_REST in main.frag and ring.rs comet(); packed as mat_sig_ring.w (vec4). Default renders byte-identical to before (accepted_ring_look unchanged). ring_rest_zero_keeps_the_comet: at ring-rest 0 the pane at rest is byte-identical to focus none, the comet still runs. Config, uniform, CPU-mirror tests; material-config.md and render-pipeline.md. Prism half is prism-f71919, waiting on the installed niri.
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
