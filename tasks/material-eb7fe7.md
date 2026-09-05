---
id: material-eb7fe7
title: "Config: signal matches, `signal { motion }`, and the `material-signal` animation"
status: done
priority: 2
size: s
owner: feat/material-signals
created: 2026-09-02T14:26:51Z
updated: 2026-09-03T02:22:14Z
depends: [material-b0e938]
tags: [signals, plan-step]
plan: docs/plans/2026-09-02-material-signals.md
step: "Task 2: Config: signal matches, `signal { motion }`, and the `material-signal` animation"
---

Outcome: docs/plans/2026-09-02-material-signals.md step "Task 2: Config: signal matches, `signal { motion }`, and the `material-signal` animation" is implemented with its tests passing and committed as that task's final step. Acceptance evidence: the step's listed test command passes and the commit named in the step exists on feat/material-signals.

## Notes

- 2026-09-03T02:19:28Z (feat/material-signals): RED: cargo test -p niri-config signal_ fails because Config.signal, Animations.material_signal, Match signal regex fields, and SignalMotionPolicy are absent.
- 2026-09-03T02:22:14Z (feat/material-signals): feat(config): add signal matches, motion policy, and material-signal animation
