---
id: material-7f6d0e
title: Opaque regions and narrowed damage for the material element
status: idea
priority: 2
created: 2026-10-06T15:07:38Z
updated: 2026-10-06T15:07:38Z
depends: []
parent: material-5d6b2c
tags: [performance, rendering]
agent: claude-code/claude-opus-5-5
---

MaterialRenderElement (src/render_helpers/material/mod.rs) overrides neither opaque_regions nor damage_since: any input change (a cursor blink in a terminal, a backdrop commit) redraws the whole slab, opaque window pixels still pay a read and a branch, and a window under an opaque cover still re-renders its offscreen (2026-09-30-hidden-window-attribution-evidence.md, Attribution). Candidate: report the window's opaque regions minus the slab band, and pass client damage through when only the window input changed. Measure with the capture protocol before committing. Found while writing docs/materials/performance.md (material-233295).
