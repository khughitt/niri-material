---
id: material-f86183
title: "Inactivity settle mode: a genuinely quiescent low-power state"
status: idea
priority: 1
created: 2026-09-11T23:34:15Z
updated: 2026-09-16T12:00:25Z
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
