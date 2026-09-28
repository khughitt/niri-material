---
id: material-cd7deb
title: Attention idle edge redraws every output even when no tile's signal changes
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-28T03:05:12Z
updated: 2026-09-28T03:05:12Z
depends: []
parent: material-53f873
tags: [performance, signals, bug]
agent: claude-code/claude-opus-5-5
---

Idle-budget trace pilots 2026-09-27 (TTY, desktop stopped; binary built from 4607d054 at NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target-20260927T224045): every quiet A observation has exactly one Niri::redraw at 30.405 s trace time, 0 material draws, pixels equal (pilot-trace-20260927T224948, pilot-trace-20260927T225321, pilot-trace-inv-20260927T225621: A-move-1 and A-resize-1 fail, C 81/81 and D 41/41 pass). The 2026-09-25 trace on d91a6024, which predates the ring-focus work, had 0 redraws in A. Cause: the attention gate (docs/specs/2026-09-18-ring-focus-motion-design.md §3). Niri::arm_input_idle_timer's callback, on Poll::Idle, calls layout.set_input_active(false) and then queue_redraw_all() unconditionally. signal.idle_after defaults to 30 s, counted from InputActivity::new at startup, and the nested run gets no input events, so the timer fires about 30 s in and redraws every output though no tile's effective motion changes (A has no sustained signal). On a real desktop the same repaint happens 30 s after every last input. Fix: queue a redraw only where the gate changes something, for example set_input_active returning whether any tile's effective signal had sustained motion, or queueing per affected output; the idle→active edge in notify_activity has the same shape. Add a test in src/tests/attention_idle.rs: an idle edge with no sustained signal queues no redraw. Then rebuild the trace binary (build-tracy) and rerun the idle-budget pilot: A-move-1 and A-resize-1 at 0 redraws.
