---
id: material-416004
title: Complete the surface-remap docs and comments
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-17T15:13:15Z
updated: 2026-10-17T15:13:15Z
depends: []
parent: material-55380c
tags: [docs]
agent: claude-code/claude-opus-5-5
---

Deferred from material-62347d's reviews. (1) docs/wiki/Configuration:-Layer-Rules.md: the material bullet names only the 1% range error; add the other load errors validate_surface_remaps raises (material with aurora or iridescence, material with response=, surface lighter than Oklab L 0.4). (2) docs/materials/render-pipeline.md: the table lead-in 'Elements that carry effects, and they are different programs:' is a fragment. (3) src/layer/mapped.rs: the remap_drawn comment says 'last drawn' but holds the last damaged uniforms; document that update_config relies on the per-frame set_shell_hue to damage a reload. (4) src/render_helpers/surface_remap.rs: OutputHue::animating says 'a fade or switch blend' but only fades set it; the switch relies on the layout's own animation. (5) src/niri.rs update_render_elements: the unwrap_or_default on monitor_for_output needs a comment naming the invariant.
