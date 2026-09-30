---
id: material-0db905
title: Design sustained optic settling from the resource-aware rendering brief
status: doing
priority: 1
size: m
complexity: high
process: planned
owner: materials-26.04
created: 2026-09-29T21:44:30Z
updated: 2026-09-30T09:10:59Z
started: 2026-09-29T23:08:21Z
depends: []
parent: material-5d6b2c
tags: [performance]
source: docs/notes/2026-09-29-resource-aware-rendering-brief.md
agent: codex
---

Why: Attention motion already uses signal { idle-after-ms } (default 30000; 0 disables), but Tile::optic_frame and AuroraOptic::rate do not read input activity. Finite-motion quiescence has passed the idle-budget trace; the remaining outcome is stopping optional sustained optic motion during extended input inactivity.
Where to start: docs/notes/2026-09-29-resource-aware-rendering-brief.md; docs/specs/2026-09-18-ring-focus-motion-design.md sections 2-3 and 8; src/activity.rs; src/niri.rs activity handlers; src/layout/tile.rs::optic_frame/tick_deadline/material_dynamics; src/render_helpers/material/optics/mod.rs::OpticFrame and aurora.rs::rate.
Bound: Write a reviewed design, then an implementation plan for review; this task does not implement or capture it. Reuse the existing activity threshold and visibility gates. Settle which sustained optics participate, opt-in/default behavior, phase freezing and resume behavior, interaction with motion full/reduced/off and animations off, and how reload/hidden workspaces/DPMS/other lit outputs behave. Preserve static level/accent indicators, finite focus beams and impulses, ordinary client updates, and the existing attention contract. Do not introduce a second input-idle detector or conflate signal Quiet with input inactivity.
Done: The design specifies no optional optic deadline or changing optic fingerprint while settled, continued visible client rendering on real damage, and finite wake behavior; includes deterministic deadline/fingerprint checks and a pilot-first capture matrix with positive controls. Document a freeze/resume strategy that avoids an unexplained phase jump.
Ideas it wakes: On completion, run tasks note on material-f86183 with the reviewed decisions, in the same commit as this result.

## Notes

- 2026-09-29T23:08:21Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a0ef6c-81be-7053-b1db-8781e2ef7949","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T09:10:59Z (materials-26.04): Input from material-39a46f (power re-run on 48ba40a1, 2026-09-29): aurora drift costs +0.96 W at 4 Hz (upper 1.37 W) and +0.77 W at 2 Hz (upper 1.03 W) over resting jelly. The 4 Hz upper estimate now exceeds 1.0 W because the run-to-run spread widened; resting jelly still costs nothing resolvable (upper 0.19 W). See docs/materials/2026-09-11-idle-budget-evidence.md, Re-run on 48ba40a1.
