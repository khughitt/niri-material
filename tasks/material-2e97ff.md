---
id: material-2e97ff
title: Overview-correct view for material deadlines and signal visibility
status: doing
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-08T14:54:18Z
updated: 2026-10-08T15:21:43Z
started: 2026-10-08T15:21:43Z
depends: []
parent: material-5d6b2c
tags: [rendering, bug, signals]
agent: claude-code/claude-opus-5-5
---

Tile::tick_deadline tests slab_in_view against the output rect at the origin (src/niri.rs:4404-4406) while the tile location is view-relative workspace coordinates before overview zoom and geo.loc; signal_render_visible (tile.rs:1149-1157) uses the workspace view_size rect (scrolling.rs:399-410). In overview, columns outside the normal view are on screen (unbounded horizontal crop, monitor.rs:2199-2201), so by source a sustained optic or attention motion on such a column reports no deadline and holds, and its beam stops animating. Gate predates task ids (1a68bf01, a6a3ef18).

First: an in-process test (src/tests/signal.rs style) with an Aurora tile in a column outside the view, overview open, asserting a reported deadline; it must fail before the fix. Then make both predicates use one view mapped through overview zoom and workspace geo (workspace switch over-reports today; keep that harmless). Audit: docs/materials/2026-10-08-culling-damage-boundaries-audit.md section 2.

## Notes

- 2026-10-08T15:21:43Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:5cc92b05-3d27-4e09-8ba0-93cffe386590","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
