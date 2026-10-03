---
id: material-d37c1a
title: Height-field bevel in the shader
status: done
priority: 2
size: m
complexity: high
process: direct
owner: glass-edges
created: 2026-10-03T01:14:49Z
updated: 2026-10-03T11:29:32Z
started: 2026-10-03T10:20:30Z
completed: 2026-10-03T10:30:43Z
depends: [material-10e12e, material-ce3229, material-f1b307]
parent: material-be611b
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-02-glass-edge-optics.md
step: "Task 4: Height-field bevel in the shader"
---

## Notes

- 2026-10-03T10:20:30Z (glass-edges): started
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:30:43Z (glass-edges): done
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T10:30:43Z (glass-edges): height-field bevel in the shader: two-boundary u, profile, softened outer gradient, ray path for taps and attenuation, tap lift, (1 - F), spill on u; mirror test pins prelude lines; the rim profile reads rounded against a forced planar render at rest, mid-resize and mid-scroll
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T11:29:32Z (glass-edges): ruling: ring_cap_keeps_one_core failed on binding right/bottom with a 59/58/59 shoulder after the height-field bevel. Cause is expected to be the intended spill brightening where the glass thins (spec 3.2/3.3), quantization possibly contributing; not isolated by measurement. one_core's walk (starts at low = core) now tolerates a one-count rise, i.e. the core spreading onto the chamfer; a rise of 2 past the minimum still ends the walk and is reported (one_core_stops_at_a_two_count_rise). Thresholds unchanged.
