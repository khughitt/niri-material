---
id: material-a54d89
title: "Implement material signals: model, IPC, solver, config, glass responses"
status: todo
priority: 1
size: l
created: 2026-09-02T12:08:58Z
updated: 2026-09-02T12:48:26Z
depends: []
tags: [signals, rendering, ipc]
---

Outcome: the compositor-first slice of docs/materials/2026-09-02-material-signals-design.md lands on materials-26.04: WindowSignals fold, decay, and impulse expiry; set/pulse/clear-window-signal IPC requests with a result channel plus WindowSignalChanged; the pure envelope solver with SignalFingerprint damage gating and per-output bucket timers for sustained motion; response blocks and response= references with signal-source/signal-tag matches; signal { motion } and the material-signal animation; and the glass ring, rim-orbit, ring-pulse, sweep, flash, and ripple responses. Acceptance evidence: section 9 of the spec (unit tests, nested IPC round trip, headless GLES smoke with zero-redraw-at-rest, bucket-rate wakeup, hidden-window, and multi-window checks, Tracy cost, DRM check) and the section 10 documentation updates.

## Notes

- 2026-09-02T12:27:59Z (feat/material-signals): Review pass 2026-09-02: signal writes are IPC Requests with a result channel, not Actions; sustained motion uses bucket timers rather than the animation loop; impulse kinds resolve to shader selectors on the CPU; flash is additive; ring geometry documented against the offset slab; state bounds and event Option<Signal> added. See spec sections 1, 2, 4, 5, 6.
