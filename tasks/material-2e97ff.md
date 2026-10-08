---
id: material-2e97ff
title: Overview-correct view for material deadlines and signal visibility
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: overview-deadline
created: 2026-10-08T14:54:18Z
updated: 2026-10-08T15:45:44Z
started: 2026-10-08T15:21:43Z
completed: 2026-10-08T15:45:44Z
depends: []
parent: material-5d6b2c
tags: [rendering, bug, signals]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

Tile::tick_deadline tests slab_in_view against the output rect at the origin (src/niri.rs:4404-4406) while the tile location is view-relative workspace coordinates before overview zoom and geo.loc; signal_render_visible (tile.rs:1149-1157) uses the workspace view_size rect (scrolling.rs:399-410). In overview, columns outside the normal view are on screen (unbounded horizontal crop, monitor.rs:2199-2201), so by source a sustained optic or attention motion on such a column reports no deadline and holds, and its beam stops animating. Gate predates task ids (1a68bf01, a6a3ef18).

First: an in-process test (src/tests/signal.rs style) with an Aurora tile in a column outside the view, overview open, asserting a reported deadline; it must fail before the fix. Then make both predicates use one view mapped through overview zoom and workspace geo (workspace switch over-reports today; keep that harmless). Audit: docs/materials/2026-10-08-culling-damage-boundaries-audit.md section 2.

## Notes

- 2026-10-08T15:21:43Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:5cc92b05-3d27-4e09-8ba0-93cffe386590","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T15:21:56Z (overview-deadline): resumed
  provenance: {"harness_session":"claude-code:5cc92b05-3d27-4e09-8ba0-93cffe386590","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T15:40:43Z (overview-deadline): RED confirmed: both overview_column_outside_the_normal_view_* tests (src/tests/signal.rs) failed at their overview assertions before the fix; GREEN after. Fix in d9e43727: Monitor::workspace_screen_view (output mapped through zoom + geo.loc, ignores vertical card crop: superset, over-report only) feeds render via RenderCtx::signal_ticks (SignalTickSink) and update via Tile::update_render_elements screen_view (replaces visible: bool). Workspace switch is now exact rather than over-reporting.
- 2026-10-08T15:44:34Z (overview-deadline): review: impl round 1 — verdict: accept; findings: Minor 1; reviewer: claude-code/claude-opus-5-5
- 2026-10-08T15:45:44Z (overview-deadline): done
  provenance: {"harness_session":"claude-code:5cc92b05-3d27-4e09-8ba0-93cffe386590","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T15:45:44Z (overview-deadline): Overview-correct view: Monitor::workspace_screen_view maps the output through overview zoom and workspace geo; render deadlines (RenderCtx::signal_ticks view) and signal_render_visible (Tile screen_view) both test against it; interactive move maps through its own zoom. In-process tests failed before, pass after.
  provenance: {"harness_session":"claude-code:5cc92b05-3d27-4e09-8ba0-93cffe386590","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
