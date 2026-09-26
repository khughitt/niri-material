---
id: material-4be9c3
title: Quiet cases redraw once exactly 3.0 s after the last stimulus redraw
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: material-265eb0
created: 2026-09-25T09:31:14Z
updated: 2026-09-26T02:06:00Z
started: 2026-09-26T01:43:05Z
completed: 2026-09-26T02:06:00Z
depends: []
parent: material-4241c3
tags: [performance]
model: "claude-opus-5-5[1m]"
agent: claude-code/claude-opus-5-5
---

Idle-budget trace 2026-09-25 (TTY, desktop stopped; all 25 settle gates passed; evidence NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-20260925T043128): every quiet observation (A, B move and resize, P, O, and the 600 s B hold) has exactly one Niri::redraw and zero material draws, pixels equal; C (81/81) and D (41/41) pass. The redraw lands 3.00 s after the previous redraw (A-move-1: 17.783 -> 20.786 s; B-long-move-1: 18.706 -> 21.709 s; O-move-1: 18.755 -> 21.758 s), i.e. just inside the window that opens at stimulus_end + 3 s, and never recurs in 600 s. Case A has animations off, so it is not spring settling. The redraw is a full State::refresh_and_flush_clients -> redraw_queued_outputs -> Niri::redraw -> render pass (winit backend, nested in headless weston); no zone before it in the trace names the waker. Find what queues it (a 3 s timer in niri, smithay, the winit backend, or a client commit from the idle kitty/wallpaper clients) and whether it is a real wake or a fixture artifact. Per the spec (docs/specs/2026-09-11-material-idle-budget-design.md, Tracing 1): do not move the observation window; fix the source or record why it is outside the budget, then rerun the trace from a TTY (binary: NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target/release/niri, rebuilt if the fix is in niri).

## Notes

- 2026-09-26T01:43:05Z (material-265eb0): started
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-26T01:45:02Z (material-265eb0): Pilots 2026-09-25 evening (TTY, desktop stopped): wali-rotate.timer stopped for the runs (restore: systemctl --user start wali-rotate.timer).
- 2026-09-26T02:04:23Z (material-265eb0): Root cause: the test client, not niri. kitty 0.48.2's mouse_hide_wait defaults to 3.0 s and the fixture ran kitty --config NONE. The column moves slide a kitty window under niri's pointer; kitty answers wl_pointer.enter with set_shape and exactly 3.003 s later sends wl_pointer.set_cursor(nil) (WAYLAND_DEBUG, pilot-default-20260925T214503: enter 01:46:03.235 -> set_cursor(nil) 01:46:06.238), which is a real cursor change niri must redraw. Pilot 1 (A-move-1, unchanged fixture, 2 min 26 s) reproduced the FAIL (redraw at +3.023 s). Pilot 2 (kitty -o mouse_hide_wait=0 only, A-move-1 + C-move-1): A 0 redraws PASS, C 80 PASS, no set_cursor(nil). Fix in the fixture next to its cursor_blink_interval=0: niri-experiments 0f7d00f (with a test; 23/23 fixture tests pass with MATERIAL_ROOT set). No niri change, so the trace binary stands.
- 2026-09-26T02:06:00Z (material-265eb0): done
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-26T02:06:00Z (material-265eb0): Fixed in the fixture: niri-experiments 0f7d00f runs the kitty clients with mouse_hide_wait=0 (kitty's 3 s cursor hide was the trailing redraw; no niri change). Pilot 3 on the committed fixture 2026-09-25 21:53-22:05 (pilot-fixed-20260925T215340): A-resize-1, B-move-1, B-resize-1, P-move-1, O-move-1 0 redraws / 0 material draws / pixels equal; D-move-1 40/40; C-move-1 80/80 in pilot 2. Full trace rerun is material-4241c3's.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
