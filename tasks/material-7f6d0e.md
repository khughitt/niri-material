---
id: material-7f6d0e
title: Narrowed damage and opaque regions for the material element
status: idea
priority: 2
created: 2026-10-06T15:07:38Z
updated: 2026-10-06T16:44:07Z
depends: []
parent: material-5d6b2c
tags: [performance, rendering]
agent: claude-code/claude-opus-5-5
---

MaterialRenderElement (src/render_helpers/material/mod.rs) keeps smithay's default damage_since and declares no opaque region. So any input change (a cursor blink in a terminal, a backdrop commit) redraws the whole slab, and elements behind its opaque parts are still drawn. Candidates: pass client damage through when only the window input changed; report the window's opaque regions minus the slab band. This does not cover hidden or covered tiles: their offscreen render happens in Tile::render_inner before smithay's occlusion pass, so culling them belongs to material-7afc31. Measure with the capture protocol before committing. Found while writing docs/materials/performance.md (material-233295); scope corrected by its review (material-832caf).

## Notes

- 2026-10-06T16:44:06Z (material-832caf): Review of material-233295 corrected this idea's scope: opaque regions cannot skip a covered tile's offscreen render (prepared in render_inner before occlusion); that part moves to material-7afc31.
