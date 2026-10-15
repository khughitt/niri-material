---
id: material-ffd61f
title: Determine the smallest organic-light experiment using existing optics
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: research/material-ffd61f
created: 2026-10-06T18:41:37Z
updated: 2026-10-15T22:21:44Z
started: 2026-10-15T22:07:20Z
completed: 2026-10-15T22:15:57Z
depends: []
parent: material-53f873
tags: [lighting]
source: "docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility"
agent: codex
---

Question: Which single warm organic-light experiment can reuse existing optics, and what is missing for a cloudy/dim-to-sunlight transition?
Where to start: docs/notes/2026-09-29-material-dynamics-brief.md; material-1c5a30 and material-f3e4e4; src/render_helpers/material/optics/aurora.rs and mod.rs; src/render_helpers/shaders/material/aurora.frag; niri-config/src/material/mod.rs; src/layout/tile.rs::optic_frame/tick_deadline; docs/specs/2026-09-29-sustained-optic-settling-design.md; docs/materials/2026-09-11-idle-budget-evidence.md. Read related fireflies material-54bcac and ambient baseline material-f41c54 as context, preserving their separate goals.
Bound: Trace amount/color/phase and existing response controls end to end. Compare a static warm Aurora preset, one finite event-triggered light transition, and a weather cycle; recommend one smallest experiment and identify its exact missing behavior, if any. Specify trigger, duration, reduced/off motion, hidden/input-idle behavior, pilot scenes and a measured comparison against neutral and static controls. No new renderer, generic animation system, live desktop, capture run, or claim that existing power results price the proposed effect. Do not decide particle geometry or time-of-day transport on behalf of the related ideas.
Expected result: Record a supported feasibility recommendation and one bounded experiment proposal on this task and in the brief; distinguish a reusable preset from behavior requiring a reviewed design. Leave the chosen look and acceptable ongoing cost to owner judgment at that later artifact review.
Ideas it wakes: On completion, run tasks note on material-1c5a30 and material-f3e4e4 with the finding, before this result's branch merges.

## Notes

- 2026-10-15T22:07:20Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:77182cee-3286-44f3-9666-0bc89bdd1b7a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T22:07:40Z (research/material-ffd61f): resumed
  provenance: {"harness_session":"claude-code:77182cee-3286-44f3-9666-0bc89bdd1b7a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T22:15:04Z (research/material-ffd61f): finding: Aurora's amount and colors are per-definition constants (AuroraOptic::values reads ResolvedAurora); only phase moves (fixed ~5 px/s per 600 s lap; drift-hz and reduced motion change step and redraw rate, not speed); no response targets the Aurora stage; in-place edits cut (apply_resolved). A static warm Aurora preset is reusable now (config only); existing finite warm light is the sweep impulse (1.5 s band) and accent-tint (hue only, 400 ms).
- 2026-10-15T22:15:09Z (research/material-ffd61f): recommendation: one finite overcast-to-sun Aurora transition; endpoints are reusable presets, the only missing behavior is an envelope easing amount and colors from rendered to newly resolved values on an in-place definition change, ending exactly on them — needs a reviewed (planned) design. Trigger, 6 s duration, reduced/off, hidden, idle, pilot scenes S1–S5 and neutral/static controls are in docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility-material-ffd61f. Weather cycle deferred: clock + cover sequence + transport on top of the envelope. No design task filed: the next scope pass of material-1c5a30/material-f3e4e4 decides.
- 2026-10-15T22:15:57Z (research/material-ffd61f): done
  provenance: {"harness_session":"claude-code:77182cee-3286-44f3-9666-0bc89bdd1b7a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T22:15:57Z (research/material-ffd61f): Feasibility traced and one experiment proposed in the dynamics brief: finite overcast-to-sun Aurora transition (envelope on amount/colors on in-place definition change) needs a planned design; static warm preset reusable now; weather cycle deferred to the clock decision. Woke material-1c5a30 and material-f3e4e4 with finding notes and readmitted them to scope.
  provenance: {"harness_session":"claude-code:77182cee-3286-44f3-9666-0bc89bdd1b7a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T22:21:44Z (materials-26.04): review: impl round 1 — verdict: revise; findings: should-fix 3; reviewer: unverified (relayed by user)
