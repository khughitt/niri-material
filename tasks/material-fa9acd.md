---
id: material-fa9acd
title: "Pipeline schema: one rule for stage reads, and pin edge-highlight as attention-driven"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-06T23:34:36Z
updated: 2026-10-06T23:34:36Z
depends: []
tags: [rendering]
source: material-be611b
agent: claude-code/claude-opus-5-5
---

From material-be611b's final review (2026-10-06, Minor 1 and 2), after the height-field bevel merged with main's pipeline schema.

1. niri-config/src/material/pipeline.rs: the reads lists follow no single rule. The local height and the across-bevel coordinate u depend on offset-x, offset-y and jelly-flex, which no height-reading stage lists (refraction, fringing, directional-blur, tint, ring, reflection). edge-highlight and ring list bevel-profile though they see it only through the normal, while glint, using the same normal, lists only ior. Choose one rule, either 'parameters the stage's code uses, including through the height and u' or 'uniforms read directly', apply it to every stage, regenerate resources/materials/pipeline.json (MATERIAL_DOCS_UPDATE=1), and note the rule in the Stage doc comment. Prism's rack reads this file, so mention the change on the Prism side.
2. src/render_helpers/material/mod.rs pipeline_signal_driven_stages_are_declared: nothing pins edge-highlight (or glint) as moved by attention. Add a frame under attention rim-orbit vs the quiet light and declared("edge-highlight", &["attention"]) and declared("glint", &["attention"]).

Done: one documented rule, the schema and json consistent with it, the new assertions passing and failing when animated or the attention response is removed.
