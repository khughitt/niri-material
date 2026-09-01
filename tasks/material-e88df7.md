---
id: material-e88df7
title: Fix frost loss during interactive material-window drag
status: todo
priority: 3
size: m
owner: debug/overview-drag-frost
created: 2026-09-01T14:35:48Z
updated: 2026-09-01T23:35:58Z
depends: []
tags: [migration, bug]
---

Outcome: frosted backdrop sampling remains visually stable while a material window is interactively dragged between workspaces in overview and normal scrolling. Acceptance evidence: remove temporary FROSTDBG instrumentation; add the smallest runnable regression check for the corrected sampling geometry; pass the source-neutral Rust gates; and record a focused live verification showing the dragged and stationary elements retain equivalent frost without fallback, sharp-texture selection, or sampling-coordinate divergence. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md and the preserved debug/overview-drag-frost five-file diff. Uncertainty: four probe rounds isolate the fault to sampling geometry passed to two elements reading identical blurred textures, but the exact bad coordinate or scale is not yet identified.

## Notes

- 2026-09-01T23:35:58Z (materials-26.04): Deferred by owner: cosmetic, transient, judged acceptable in burn-in. Four probe rounds eliminated by measurement: the flag (blur=true on all 173 draws), the silent material_ready gate (true for both windows, both prepare calls true), blur production (both buffers cached-blur, never SHARP), buffer/texture identity (dragged and stationary elements bind the SAME Rc addresses and the SAME GL texture id 20 at 3440x1440 in one frame), the workspace/backdrop selector (ws_rect non-empty in 921 of 927 draws), blur strength (absent at 6 passes as at 1), overview-specificity (reproduces at zoom 1 in the normal view) and minification (tile_render_location upscales by zoom=1 there). Also established: the background xray buffer is empty at all times on this host because the wallpaper is place-within-backdrop, so all frost arrives via the backdrop path. Next probe: log the destination rect and XrayPos per element beside backdrop_rect and compare texels-sampled to pixels-drawn for the two elements.
