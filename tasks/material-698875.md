---
id: material-698875
title: Resize snapshot drops the material when a toplevel's buffer is single-pixel
status: done
priority: 3
size: s
complexity: mid
process: direct
owner: material-698875
created: 2026-10-01T09:54:50Z
updated: 2026-10-06T14:14:47Z
started: 2026-10-06T13:24:00Z
completed: 2026-10-06T13:32:58Z
depends: []
tags: [rendering, bug]
source: material-0e80c1
agent: claude-code/claude-opus-5-5
---

render_snapshot_from_surface_tree (src/render_helpers/surface.rs) bakes only imported textures, and smithay never imports wp_single_pixel_buffer buffers. A toplevel whose root buffer is single-pixel therefore gets an empty resize snapshot, RenderSnapshot::texture returns None, and Tile::render skips both the resize shader and the material for the whole resize (plain fallback). Found while building the matched ring pair (src/tests/ring_pair.rs), whose test client used single-pixel buffers. Real clients rarely do this for a root surface; a single-pixel subsurface would instead be missing from the snapshot. Scope: decide whether snapshots should bake single-pixel buffers as solid-colour elements.

## Notes

- 2026-10-01T10:00:23Z (materials-26.04): scope: todo P3, direct. Fix in render_snapshot_from_surface_tree: bake a single-pixel buffer as a solid-colour element at its view rect so the snapshot is never empty. Check: a ring_pair-style test whose client keeps single-pixel buffers renders the material mid-resize. Upstream has the same code path, so note it in upstream-divergence if it lands.
- 2026-10-06T13:24:00Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:d57c1528-57e5-424d-9b5a-e9635e5720d4","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T13:24:09Z (material-698875): resumed
  provenance: {"harness_session":"claude-code:d57c1528-57e5-424d-9b5a-e9635e5720d4","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T13:32:58Z (material-698875): done
  provenance: {"harness_session":"claude-code:d57c1528-57e5-424d-9b5a-e9635e5720d4","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T13:32:58Z (material-698875): Resize snapshots bake a single-pixel buffer as a solid-colour element at its view rect (BakedSurface in render_helpers/surface.rs), so a single-pixel toplevel or subsurface keeps the material mid-resize; ring_pair's single_pixel_window_keeps_the_material_mid_resize renders pixel-identical to the shm fixture; seam row added to upstream-divergence
  provenance: {"harness_session":"claude-code:d57c1528-57e5-424d-9b5a-e9635e5720d4","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T14:14:47Z (material-2f8ed6): correction: the single-pixel protocol is #[cfg(test)] in this fork and upstream (src/niri.rs, niri#619), so the fix only reaches the test client; no real toplevel or subsurface could hit the bug. Upstream PR #1463 makes the same change (see material-2f8ed6).
