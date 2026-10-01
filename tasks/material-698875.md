---
id: material-698875
title: Resize snapshot drops the material when a toplevel's buffer is single-pixel
status: idea
priority: 2
created: 2026-10-01T09:54:50Z
updated: 2026-10-01T09:54:50Z
depends: []
tags: []
source: material-0e80c1
agent: claude-code/claude-opus-5-5
---

render_snapshot_from_surface_tree (src/render_helpers/surface.rs) bakes only imported textures, and smithay never imports wp_single_pixel_buffer buffers. A toplevel whose root buffer is single-pixel therefore gets an empty resize snapshot, RenderSnapshot::texture returns None, and Tile::render skips both the resize shader and the material for the whole resize (plain fallback). Found while building the matched ring pair (src/tests/ring_pair.rs), whose test client used single-pixel buffers. Real clients rarely do this for a root surface; a single-pixel subsurface would instead be missing from the snapshot. Scope: decide whether snapshots should bake single-pixel buffers as solid-colour elements.
