---
id: material-c94dd7
title: Build a bounded settling capture and offline verdict
status: doing
priority: 1
size: m
complexity: high
process: direct
owner: material-a1d7da
created: 2026-09-30T10:47:21Z
updated: 2026-10-01T04:39:11Z
started: 2026-09-30T12:09:27Z
depends: [material-6e3bda]
parent: material-f86183
tags: [performance]
source: "docs/plans/2026-09-30-sustained-optic-settling.md#task-4"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
plan: docs/plans/2026-09-30-sustained-optic-settling.md
step: "Task 4: Build a bounded settling capture and offline verdict"
---

Implement the pilot-first driver and stdlib analyzer with marker-aligned windows, positive controls, bounded cleanup and explicit cadence controls. No live capture in this step. Execute only after plan acceptance and completion of material-0db905; the design task itself implements and captures nothing.

## Notes

- 2026-09-30T12:09:27Z (material-a1d7da): started
  provenance: {"harness_session":"codex:01a0f202-290f-70e2-b161-8b244605074f","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T12:43:08Z (material-a1d7da): run: headless development pilot preflight refused at load1 4.31 > 2.0; no compositor or trace started; artifact /mnt/ssd3/niri-material/optic-settling/pilot-dev-20260930-1; continue offline fixture work, then queue live pilot for tasks quiet.
- 2026-09-30T12:48:28Z (material-a1d7da): parked (waiting on agent, quiet; headless, 10 min): On the quiet host, resume this task and run the 18-second headless active-idle pilot with the identified binary; inspect marker/CSV timing and cleanup, then implement the remaining headless check families before matrix capture.
  provenance: {"harness_session":"codex:01a0f202-290f-70e2-b161-8b244605074f","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T04:15:51Z (material-a1d7da): resumed
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T04:15:51Z (material-a1d7da): run: 8 min (est 1, headless); load wait 0.5, preflight 0.5, probe wait 0.2, hung cleanup 7; failed: on HEAD e4f3337a with a fresh Tracy build (the 02fc43c0 sidecar no longer matched HEAD), the probe kitty never mapped: the driver set WAYLAND_DISPLAY to the IPC socket's basename, so kitty blocked connecting, and on_exit's unbounded wait sat on it (TERM blocked) until killed by hand; capture lock released, no trace (pilot-dev-20261001-1)
- 2026-10-01T04:23:52Z (material-a1d7da): run: 4 min (est 2, headless); build 1.5, load wait 0.5, preflight 0.5, capture+export+analysis 1; failed (driver, not the code under test): end to end with clean teardown, analysis invalid 'missing active GPU before control'. CPU shows the intended settle: 4 Hz redraws to the pause marker at 5.40 s, one edge-flush redraw at 6.47 s, none until resume at 20.26 s, then 4 Hz. Tracy recorded GPU zones only from 6.47 s (d09741's visible case: from 10.6 s while CPU redraws start at 0.4 s), so a pause at 5 s has no GPU control; and the hold [pause+1, pause+6] would have held the 1.07 s flush. Driver now keeps input active to 12 s, holds from pause+2 s for the declared hold (pilot-dev-20261001-2)
- 2026-10-01T04:39:10Z (material-a1d7da): run: 1.5 min (est 2, headless); build 1.5 (before), preflight 0.5, capture+export+analysis 1; failed: extra pause/resume pair at 5.42/6.03 s: trace time counts from the compositor's start and setup outlasted the 5 s threshold; 540a0240 sends pointer motion from launch (pilot-dev-20261001-3)
- 2026-10-01T04:39:10Z (material-a1d7da): run: 1.5 min (est 2, headless); preflight 0.5, capture+export+analysis 1; passed for active-idle-resume (run verdict invalid only on 'missing case combined-motion', by design until the other families exist): 4 Hz redraws to the pause marker at 27.00 s, one edge-flush redraw/draw at the marker, zero redraws and material draws to the resume at 36.49 s, 4 Hz after; GPU zones from 7.8 s so both active controls hold GPU draws; held screenshots byte-equal; no leftover processes, lock released (pilot-dev-20261001-4, binary 540a0240)
