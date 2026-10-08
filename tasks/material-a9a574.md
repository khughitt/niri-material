---
id: material-a9a574
title: "Cover floating, interactive-move, workspace-switch and CSD extents in the off-view cull tests"
status: done
priority: 3
size: s
complexity: low
process: direct
owner: cull-tests
created: 2026-10-08T16:27:08Z
updated: 2026-10-08T16:37:34Z
started: 2026-10-08T16:28:11Z
completed: 2026-10-08T16:37:33Z
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
- 2026-10-08T16:28:17Z (cull-tests): resumed
  provenance: {"harness_session":"claude-code:42335930-7fe4-4735-bd08-c7d1e56a5390","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T16:37:33Z (cull-tests): scope: a floating window cannot leave the output in the normal view (placement keeps 10-75 px on screen), so the floating case tests the edge clamp stays drawn; the off-output floating case is the interactive move. A tiled drag holds a 0.75 alpha animation and never culls; its material id is nested in the alpha offscreen, so it is not asserted.
- 2026-10-08T16:37:33Z (cull-tests): done
  provenance: {"harness_session":"claude-code:42335930-7fe4-4735-bd08-c7d1e56a5390","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T16:37:33Z (cull-tests): Four in-process cull tests in src/tests/signal.rs: floating window at the edge clamp draws; floating interactive move culls off the output and redraws on return (screen capture always draws); mid workspace switch draws both workspaces and keeps an off-view column culled; a CSD shadow reaching into view keeps an off-view column drawn (new test-client set_geometry). RED-checked with the cull off and with the buffer arm removed.
  provenance: {"harness_session":"claude-code:42335930-7fe4-4735-bd08-c7d1e56a5390","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
