---
id: material-1d70db
title: "Ring: separate the resting ring's level from the moving beam's glow"
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-10-02T00:08:50Z
updated: 2026-10-02T00:09:33Z
depends: []
tags: [rendering, prism]
agent: claude-code/claude-opus-5-5
---

Owner ask (2026-10-01): turn the resting ring off, or down, without losing the comet transition. Today comet() in src/render_helpers/material/ring.rs is BEAM_BASE * glow * (decay*moving + BEAM_REST): ring-glow scales both, and BEAM_REST (0.2) is a constant. Add a native ring-rest level (0 = no resting ring, default 1 = today's BEAM_REST) on the response block, so ring-glow stays the overall gain and ring-rest scales only the resting term. When rest is 0, the tile must stay free of redraws after the run, as it is now. Then expose it in the Prism Ring group as glass.ring.rest (prism task). Check that the head and tail still read at rest 0, since today they ride on top of the rest level.

## Notes

- 2026-10-02T00:09:33Z (materials-26.04): Prism half: prism-f71919 (glass.ring.rest), which depends on this task
