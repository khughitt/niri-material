---
id: material-832caf
title: Revise the material performance guide after review
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: material-832caf
created: 2026-10-06T16:42:26Z
updated: 2026-10-06T16:44:17Z
started: 2026-10-06T16:42:26Z
completed: 2026-10-06T16:44:17Z
depends: []
parent: material-5d6b2c
tags: [docs, performance]
agent: claude-code/claude-opus-5-5
---

Review round 1 of docs/materials/performance.md (material-233295) found: per-call span medians labelled per-damage costs and the noise-slider 'drag' case misnamed; zero-draw claim not qualified for output damage from other elements; covered-window offscreen cost wrongly attributed to missing opaque regions (offscreen is prepared in Tile::render_inner before smithay's occlusion), also in the hidden-window attribution doc and material-7f6d0e; power mechanism and sub-resolution jelly deltas stated as established; equal whole-draw medians read as a per-stage bound; 'never freed' too absolute; evidence references not links.

## Notes

- 2026-10-06T16:42:26Z (materials-26.04): concerns: material-233295 defect — the guide misstated several measurements and one mechanism (review round 1)
- 2026-10-06T16:42:26Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:42:57Z (material-832caf): resumed
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:44:17Z (material-832caf): done
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:44:17Z (material-832caf): Revised docs/materials/performance.md per review round 1: per-damage totals (1.83–1.88 ms) separated from per-call span medians and the noise-amount case named; zero-draw claim scoped to quiet scenes with output-damage redraws noted; covered-window offscreen cost attributed to render_inner running before occlusion (dated correction in the hidden-window attribution doc, material-7f6d0e rescoped); power mechanism kept as the evidence's unproven hypothesis and jelly as unresolved; equal whole-draw medians no longer read as a stage bound; texture lifetime stated precisely; evidence references are now heading links, all verified.
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
