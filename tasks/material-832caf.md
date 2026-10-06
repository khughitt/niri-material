---
id: material-832caf
title: Revise the material performance guide after review
status: doing
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-06T16:42:26Z
updated: 2026-10-06T16:42:28Z
started: 2026-10-06T16:42:26Z
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
