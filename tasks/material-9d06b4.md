---
id: material-9d06b4
title: "Tile wiring: crossfade, effective signal, fingerprint, transitions, and bucket timers"
status: done
priority: 2
size: m
owner: feat/material-signals
created: 2026-09-02T14:26:51Z
updated: 2026-09-03T04:08:43Z
depends: [material-0e32cc]
tags: [signals, plan-step]
plan: docs/plans/2026-09-02-material-signals.md
step: "Task 9: Tile wiring: crossfade, effective signal, fingerprint, transitions, and bucket timers"
---

Outcome: docs/plans/2026-09-02-material-signals.md step "Task 9: Tile wiring: crossfade, effective signal, fingerprint, transitions, and bucket timers" is implemented with its tests passing and committed as that task's final step. Acceptance evidence: the step's listed test command passes and the commit named in the step exists on feat/material-signals.

## Notes

- 2026-09-03T03:56:05Z (feat/material-signals): RED: cargo test --bin niri signal selects 0 tests; cargo test --lib signal fails on the missing Task 9 helpers, timer API, and output fields as expected.
- 2026-09-03T04:07:51Z (feat/material-signals): All-target verification found three niri-visual-tests RenderCtx constructors outside src; they now pass signal_ticks: None like every non-output-injected caller.
- 2026-09-03T04:08:43Z (feat/material-signals): feat(render): drive material signals from tiles with crossfade and bucket timers
