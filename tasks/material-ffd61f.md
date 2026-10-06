---
id: material-ffd61f
title: Determine the smallest organic-light experiment using existing optics
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-06T18:41:37Z
updated: 2026-10-06T18:41:37Z
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
Ideas it wakes: On completion, run tasks note on material-1c5a30 and material-f3e4e4 with the finding, in the same commit as this result.
