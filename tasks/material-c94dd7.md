---
id: material-c94dd7
title: Build a bounded settling capture and offline verdict
status: done
priority: 1
size: m
complexity: high
process: direct
owner: material-a1d7da
created: 2026-09-30T10:47:21Z
updated: 2026-10-01T07:52:17Z
started: 2026-09-30T12:09:27Z
completed: 2026-10-01T07:52:15Z
depends: [material-6e3bda]
parent: material-f86183
tags: [performance]
source: "docs/plans/2026-09-30-sustained-optic-settling.md#task-4"
model: claude-opus-5-5
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
- 2026-10-01T05:11:30Z (material-a1d7da): Task 4 driver generalized (uncommitted, in progress): manifest schema 2 declares per case the optic edge sequence, a state per edge interval (active at a cadence, static, settled, setup), journaled stimuli and pixel pairs; tools/optic_settling.py derives its own windows (last stimulus-free span for active/static, flush <=2 then zero events outside stimuli for settled, a quiet span >= hold) and aligns the CLOCK_MONOTONIC journal through the edges' real_ns. 27 cases over the nine families; TTY resume, unlock, idle inhibitor and output removal recorded unverified with the reason. Instrument facts found on the quiet host: Tracy GPU zones start 6.5-10.6 s into a trace; a frame's GPU zones are collected only when a later frame renders (the last frame before a quiet span shows no material draw); kitty's mouse_hide_wait (3 s) redraws; tracy-csvexport prints a notice, not a header, for a trace without messages; IPC pointer motions cost ~0.5 s, so drives run on absolute times.
- 2026-10-01T05:41:04Z (material-a1d7da): run: 37 min over four development subsets (est 25, headless); every one of the 23 nested cases passed in dev-20261001-6/7/8 after the fixes they found: absolute drive schedule, kitty mouse_hide_wait=0, pointer motion through setup, a final collect frame, the no-messages export, GPU P8 rest between cases (dev-20261001-5 was refused at the client-damage settle on P5). Readings: Aurora 4 Hz full / 2 Hz reduced, breathe+Aurora union ~12/s, pulse ~27/s; the actual 30 s default paused 30.0 s after the last motion; lower-threshold reload paused at the reload, raise and disable resumed there; screencopy drew held material with no redraw and no edge; DPMS power-on IPC drew once and stayed held
- 2026-10-01T05:55:44Z (material-a1d7da): run: 9 min (est 27, headless); build 1.5, six cases 7.5; aborted by the agent (TERM, clean teardown): gate-off could not align its new collect stimulus without an optic edge, so the full pilot could not pass; collect now only joins cases with edges (pilot-20261001-1; aurora-full, aurora-reduced, combined, amount-0, drift-0 passed)
- 2026-10-01T06:27:05Z (material-a1d7da): run: 26 min (est 27, headless); build 1.5, 23 cases 24.5; passed: first full pilot on 54d93893 (pre-review development evidence; Task 5 material-2ee11e still owes the code review before its acceptance pilot): all 23 nested cases passed, output-removal/tty-resume/unlock/idle-inhibitor unverified with their reasons, complete=false by design (pilot-20261001-2). Matrix started on the same binary gated on it
- 2026-10-01T06:28:34Z (material-a1d7da): run: 0.5 min (est 45, headless); refused by its own pilot gate before preflight capture: case configs embed their run's backdrop path, so raw config hashes never match across runs; the matrix gate now compares run_config_identity (run directory as $OUT) (matrix-20261001-1)
- 2026-10-01T07:40:30Z (material-a1d7da): run: 27 min (est 27, headless); passed: pilot on 067fc5e3, 23 passed, 4 unverified (pilot-20261001-3)
- 2026-10-01T07:40:30Z (material-a1d7da): run: 45 min (est 45, headless); failed on one case: matrix on 067fc5e3 gated on pilot-20261001-3; 26 of 27 run cases passed, including aurora-full-r2/r3 and aurora-reduced-r2/r3 and default-threshold; aurora-full (600 s hold) failed with 9 redraws at 604.4 s, alongside client commits: the focus-thief kitty's shared 'exec sleep 600' exited and its window closed inside the hold (fixture, not the optic: logical time held, 334606130807025 at pause and resume). Driver idles its clients with sleep infinity; rerunning only the 600 s case as a development subset against the same binary and pilot (matrix-20261001-2)
- 2026-10-01T07:52:15Z (material-a1d7da): run: 12 min (est 12, headless); passed: the 600 s case alone as a development matrix subset against the same binary and pilot, clients idling with sleep infinity (uncommitted driver change, committed next): one flush redraw, 601.9 s settled with zero redraws and material draws, 4 Hz before and after (matrix-dev-20261001-3)
- 2026-10-01T07:52:15Z (material-a1d7da): done
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T07:52:15Z (material-a1d7da): result: Task 4 implemented and exercised on the quiet host: pilot-20261001-3 passed (23/23 nested cases, 4 unverified lanes), matrix-20261001-2 passed 26/27 with the 600 s case failing on a fixture client exit, fixed and passed in matrix-dev-20261001-3. Pre-review evidence only: material-2ee11e owns the whole-change code review, the acceptance pilot and matrix on the reviewed binary, the owner's idle/resume clip and the evidence doc. Its runs need a quiet headless host (pilot ~27 min, matrix ~45 min).
