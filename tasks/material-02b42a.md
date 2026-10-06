---
id: material-02b42a
title: The reflection optic
status: done
priority: 2
size: m
complexity: mid
process: direct
owner: glass-edges
created: 2026-10-03T01:14:49Z
updated: 2026-10-03T10:57:32Z
started: 2026-10-03T10:46:31Z
completed: 2026-10-03T10:57:32Z
depends: [material-d63184]
parent: material-be611b
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-02-glass-edge-optics.md
step: "Task 7: The `reflection` optic"
---

## Notes

- 2026-10-03T10:46:31Z (glass-edges): started
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:56:37Z (glass-edges): reflection motion calibration: real 364 channels changed (0 clipped), mutated 0 (0 clipped); real < 2500 so 500 not kept; real >= 25*max(0,4), REFLECTION_MOTION_MIN = geometric mean of 364 and max(0,4) = 38; mutation fails with 'the reflection ignores the perturbation'
- 2026-10-03T10:57:32Z (glass-edges): done
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:57:32Z (glass-edges): reflection optic: config, renderer, GLSL, docs; neutral byte-identical, face untouched, bevel lit; follows the perturbed direction in motion over a patterned backdrop (mutation control fails)
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
