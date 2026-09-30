---
id: material-f86183
title: "Inactivity settle mode: a genuinely quiescent low-power state"
status: idea
priority: 1
created: 2026-09-11T23:34:15Z
updated: 2026-09-30T11:03:15Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, dynamics]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

After a period of mouse/keyboard inactivity, settle the glass into a static state: no per-frame uniform churn, no redraws without a stimulus, easing back on activity. Close to material-265eb0's settle state (which measures the idle budget) and material-4bf8b8 (frost-on-idle, a visible response to the same signal); this task is the input-inactivity trigger and the quiescence guarantee.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

2026-09-16 priority: extended keyboard/mouse inactivity must stop optional sustained material dynamics, including alert/attention ring animation; retain static state indicators. Sustained attention animation is eligible only while BOTH the window is actually visible on an active output/workspace and the user is active. Coordinate the visibility gate with material-7afc31; settle animation deadlines and uniform churn, not only the visible amplitude. Choose inactivity delay and resume behavior during design; do not invent them in the task.

## Notes

- 2026-09-16T12:00:25Z (materials-26.04): Raised P2 to P1 at user request: prioritize avoiding invisible-window and idle resource waste; distinguish observed client GPU use from unproven compositor rendering.
- 2026-09-29T21:43:50Z (materials-26.04): scope: briefed; 30 s input-idle attention gating and finite-motion quiescence are shipped; Aurora still ignores input activity, so wider optic settling needs reviewed design; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
- 2026-09-30T11:03:15Z (material-0db905): Accepted design from material-0db905: docs/specs/2026-09-29-sustained-optic-settling-design.md (spec round 4) and docs/plans/2026-09-30-sustained-optic-settling.md (plan round 2). Aurora settles by default on the existing 30000 ms input-idle threshold; 0 disables both Aurora and attention settling. Shared Clock logical time freezes phase and resumes without easing/catch-up. Optics see logical time only; the registry suppresses paused deadlines and translates the earliest logical deadline, checking frame/snapshot agreement. Preserve real client damage, finite effects, static indicators, attention semantics and existing visibility gates. Production edges retain explicit get_monotonic_time timestamps; virtual time stays in timeline/layout tests, Niri fixtures stay real-time. Verify actual unlock/power-on/inhibitor paths, transition Tracy markers and pilot-first captures; TTY resume remains a real-TTY check and screencast uses a real consumer. No watt saving is claimed from quiescence alone. Five sequential execution children start with material-a1d7da. User explicitly paused before execution; no implementation or captures started. On resume, scope this existing idea from the accepted artifacts and begin Task 1.
