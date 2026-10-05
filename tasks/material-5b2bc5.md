---
id: material-5b2bc5
title: "Pipeline schema: the sweeps stage reads the accent response"
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-05T11:49:38Z
updated: 2026-10-05T11:49:39Z
depends: []
parent: material-3aa1f2
tags: [material, docs]
agent: claude-code/claude-opus-5-5
---

The sweeps row in niri-config/src/material/pipeline.rs lists responses ping, done, error only. When an impulse carries no accent of its own, its sweep colour falls back to the frame's accent scaled by presence (src/render_helpers/material/mod.rs, the impulse_rgb fallback in the uniform build), so a sweep running while presence ramps changes colour every frame from the accent signal. Add "accent" to the sweeps row, extend pipeline_signal_driven_stages_are_declared (src/render_helpers/material/mod.rs tests) with a case that drives the fallback and asserts sweeps lists accent (watch it fail first), regenerate resources/materials/pipeline.json with MATERIAL_DOCS_UPDATE=1, and tell prism the vendored copy changed.

## Notes

- 2026-10-05T11:49:38Z (materials-26.04): concerns: material-a00785 defect — sweeps row omits the accent response that drives its fallback colour
