---
id: material-e88df7
title: Fix frost loss during interactive material-window drag
status: doing
priority: 2
size: m
owner: debug/overview-drag-frost
created: 2026-09-01T14:35:48Z
updated: 2026-09-01T14:35:48Z
depends: []
tags: [migration, bug]
---

Outcome: frosted backdrop sampling remains visually stable while a material window is interactively dragged between workspaces in overview and normal scrolling. Acceptance evidence: remove temporary FROSTDBG instrumentation; add the smallest runnable regression check for the corrected sampling geometry; pass the source-neutral Rust gates; and record a focused live verification showing the dragged and stationary elements retain equivalent frost without fallback, sharp-texture selection, or sampling-coordinate divergence. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md and the preserved debug/overview-drag-frost five-file diff. Uncertainty: four probe rounds isolate the fault to sampling geometry passed to two elements reading identical blurred textures, but the exact bad coordinate or scale is not yet identified.
