---
id: material-f4143a
title: Determine whether existing film grain and ring optics satisfy the accepted bevel look
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-06T18:28:01Z
updated: 2026-10-09T23:29:33Z
depends: [material-be611b]
parent: material-6062fd
tags: [rendering, harness]
source: docs/notes/2026-10-06-glass-optics-brief.md
agent: codex
---

Question: After the accepted height-field bevel lands, do existing film-site grain, ring scattering and edge spill already provide grain on dark edges and a ring that reads inside the slab?
Where to start: docs/notes/2026-10-06-glass-optics-brief.md; docs/materials/render-pipeline.md; src/render_helpers/shaders/material/main.frag and noise.frag; src/tests/ring_look.rs, ring_pair.rs and noise_site.rs; material-be611b and material-519eeb.
Bound: After material-be611b is accepted and merged, reuse surfaceless GLES frozen-clock fixtures for one matched sheet: noise zero/glass/film at equal fine-grain strength, dark and light tint, each at rest and one moving-beam instant. Keep geometry, seed, clock and response values fixed; preserve the accepted ring reference. No production shader, new grain layers, normal perturbation, content-stage implementation, live desktop capture or performance benchmark. Run the smallest end-to-end fixture first.
Expected result: Save matched images and a brief evidence note separating observed edge/ring changes from owner judgement. Check repeated renders are identical, neutral output and opaque client pixels are unchanged, and settled state adds no animation deadline. Recommend existing controls or name the residual visual gap that needs design; leave visual acceptance to the owner. Verification goes through just test-one -p niri <fixture-filter>, then just test-fast for any fixture changes. Record the finding on this task and in the brief.
Ideas it wakes: On completion, run tasks note on material-1aa3af and material-4e3e9c with the finding, in the same commit as this result.

## Notes

- 2026-10-09T23:29:33Z (materials-26.04): 2026-10-09: the be611b pick (bevel-profile 2, reflection 0.6) was judged on the sheet's inactive look at a thickness/attenuation-distance ratio of 1.5, where the bevel transmits; the live active look's ratio is 3.5 and transmits ~2e-4 across the whole bevel, so the pick cannot show there. Any matched sheet here should include the live active and inactive looks as rendered by prism.kdl, not only the sheet's looks. See material-ac926f.
