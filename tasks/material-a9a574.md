---
id: material-a9a574
title: "Cover floating, interactive-move, workspace-switch and CSD extents in the off-view cull tests"
status: doing
priority: 3
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-08T16:27:08Z
updated: 2026-10-08T16:28:11Z
started: 2026-10-08T16:28:11Z
depends: []
parent: material-5d6b2c
tags: [testing, performance]
agent: claude-code/claude-opus-5-5
---

The off-view cull (material-7afc31) is tested only for a scrolling column and the overview. Add in-process cases in src/tests/signal.rs: a floating window moved fully off the output, the interactive-move tile, and a mid-switch workspace, each asserting the material element is culled only when wholly off screen. Add a client window whose buffer extends past its xdg geometry (CSD shadow), so LayoutElement::buf_bbox decides a case: band off screen, shadow on screen -> not culled. The test client has no set_window_geometry yet; add it. Review findings on the tests' gaps, 2026-10-08.

## Notes

- 2026-10-08T16:27:08Z (materials-26.04): concerns: material-7afc31 extension — test coverage for placements and the buffer-extent arm the landed tests do not reach
- 2026-10-08T16:28:11Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:42335930-7fe4-4735-bd08-c7d1e56a5390","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
