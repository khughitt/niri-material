---
id: material-a54d89
title: "Implement material signals: model, IPC, solver, config, glass responses"
status: todo
priority: 1
size: l
created: 2026-09-02T12:08:58Z
updated: 2026-09-02T12:09:08Z
depends: []
tags: [signals, rendering, ipc]
---

Outcome: the compositor-first slice of docs/materials/2026-09-02-material-signals-design.md lands on materials-26.04: WindowSignals fold and decay, set/pulse/clear-window-signal actions plus WindowSignalChanged, the pure envelope solver with SignalFingerprint damage gating, response blocks and response= references with signal-source/signal-tag matches, signal { motion } and the material-signal animation, and the glass ring, rim-orbit, ring-pulse, sweep, flash, and ripple responses. Acceptance evidence: section 9 of the spec (unit tests, nested IPC round trip, headless GLES smoke with zero-redraw-at-rest and quantized-rate checks, Tracy cost, DRM check) and the section 10 documentation updates.
