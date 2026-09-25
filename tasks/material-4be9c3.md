---
id: material-4be9c3
title: Quiet cases redraw once exactly 3.0 s after the last stimulus redraw
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-25T09:31:14Z
updated: 2026-09-25T09:31:14Z
depends: []
parent: material-4241c3
tags: [performance]
agent: claude-code/claude-opus-5-5
---

Idle-budget trace 2026-09-25 (TTY, desktop stopped; all 25 settle gates passed; evidence NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-20260925T043128): every quiet observation (A, B move and resize, P, O, and the 600 s B hold) has exactly one Niri::redraw and zero material draws, pixels equal; C (81/81) and D (41/41) pass. The redraw lands 3.00 s after the previous redraw (A-move-1: 17.783 -> 20.786 s; B-long-move-1: 18.706 -> 21.709 s; O-move-1: 18.755 -> 21.758 s), i.e. just inside the window that opens at stimulus_end + 3 s, and never recurs in 600 s. Case A has animations off, so it is not spring settling. The redraw is a full State::refresh_and_flush_clients -> redraw_queued_outputs -> Niri::redraw -> render pass (winit backend, nested in headless weston); no zone before it in the trace names the waker. Find what queues it (a 3 s timer in niri, smithay, the winit backend, or a client commit from the idle kitty/wallpaper clients) and whether it is a real wake or a fixture artifact. Per the spec (docs/specs/2026-09-11-material-idle-budget-design.md, Tracing 1): do not move the observation window; fix the source or record why it is outside the budget, then rerun the trace from a TTY (binary: NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target/release/niri, rebuilt if the fix is in niri).
