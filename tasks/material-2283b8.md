---
id: material-2283b8
title: GPU cost A/B on the capture lane
status: done
priority: 1
size: m
complexity: mid
process: direct
needs: [quiet, nested]
owner: material-abc08c
created: 2026-10-09T21:41:52Z
updated: 2026-10-16T12:25:27Z
started: 2026-10-10T11:24:14Z
completed: 2026-10-16T12:25:27Z
depends: [material-7e3447, material-abc08c]
parent: material-6901e0
tags: [rendering, camera]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-09-transient-rigid-tilt.md
step: "Task 14: GPU cost A/B on the capture lane"
---

Step 14 of the transient rigid tilt plan (docs/plans/2026-10-09-transient-rigid-tilt.md); spec docs/specs/2026-10-09-transient-rigid-tilt-design.md.

## Notes

- 2026-10-10T11:24:14Z (material-abc08c): started
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-10T11:40:38Z (material-abc08c): run: 13 min (est 12, headless); build 8 (release 3.4, tracy 4.4), load wait 5, preflight not reached; refused: await_load, load1 stayed at or above 1.0 for 5 min after the build (owner on the desktop); no capture_preflight, no sub-run started
- 2026-10-10T11:47:02Z (material-abc08c): review: impl round 1 — verdict: accept; findings: Minor 7; reviewer: claude-code
- 2026-10-10T11:47:02Z (material-abc08c): review: impl round 2 — verdict: accept; findings: none; reviewer: claude-code
- 2026-10-10T11:47:02Z (material-abc08c): parked (waiting on user, quiet; headless, 40 min): On a quiet host with the desktop stopped (headless lane), from .worktrees/material-abc08c: run the pilot, OUT=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/cost-pilot-$(date +%s) CAPTURE_TASK=material-2283b8 VIEW_TILT_COST_AB=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/niri-tracy-23039f67 VIEW_TILT_COST_PILOT=1 docs/materials/scripts/view-tilt-cost.sh (build ~1 warm/8 cold, load wait ≤5, preflight 1, 12 rest sub-runs + swing ~9), read metrics.txt so every gate ran; then the same with VIEW_TILT_COST_PILOT=0 (~20); record a run: note per attempt; judge per Task 14 Step 3 (PASS: fill the evidence Cost section and the performance.md §7 figures; INCONCLUSIVE: rerun; FAIL: Step 4 contingency). Last attempt (2026-10-10) refused at await_load: load1 stayed ≥ 1.0 for 5 min with the owner on the desktop; the per-gain swing deadlines have not run live yet
  provenance: {"harness_session":"claude-code:57713fd6-fddd-42ac-ae02-f19ef0c69b5d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T10:04:50Z (material-abc08c): resumed
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T10:18:50Z (material-abc08c): run: 12 min (est 20, headless); build 3, load wait 3, preflight 1, sub-runs 5 (tilt0 a1/b1/a2/b2, tilt10 a1/b1 done); refused: settle before tilt10-a2, gpu_power_w 10.685 vs baseline 8.945 (+1.5 limit). Cause: display blanked via fb0 for the baseline (8.9 W) and woke during tilt0-a1 (dpms On, fb0 blank 1), lifting the P8 floor to the usual ~10.3-10.4 W seen in Oct 3-6 TTY runs. Completed medians (pilot, 3 steps): tilt0 a 0.106/0.098 b 0.097/0.089 ms; tilt10 a1 0.104 b1 0.096. OUT cost-pilot-1792145107. Rerunning with the display left on.
- 2026-10-16T10:30:59Z (material-abc08c): run: 11 min (est 10, headless); build 0.5, load wait 2, preflight 1, sub-runs 12 + swing 7; failed: release disturbed by a console login on tty2 06:22:39-06:23:02 (getty@tty2 restart); every gate otherwise ran. Pilot medians (3 steps, display on): tilt0 a 0.106/0.104 b 0.088/0.096; tilt10 a 0.106/0.106 b 0.096/0.096; fine a 0.114/0.114 b 0.104/0.103 ms -> verdicts INCONCLUSIVE/FAIL/FAIL, a ~0.010 ms over b; swing 107 frames, median 0.624, max 1.595, extra 36.7 ms/gain. OUT cost-pilot-1792145940. Full run next.
- 2026-10-16T10:42:52Z (material-abc08c): run: 11 min (est 20, headless); build 0.5, load wait 1, preflight 1, sub-runs 12 + swing 8; passed (clean release). Verdicts: tilt0 a 0.7539 b 0.6569 bound 0.0066 INCONCLUSIVE (a1/a2 0.746/0.761); tilt10 a 0.7475 b 0.6561 bound 0.0066 FAIL; fine a 0.8166 b 0.7306 bound 0.0073 FAIL. Rest path +0.09 ms (~13%) over base, repeats tight -> Task 14 Step 4 contingency (second program variant). Swing 330 frames, median 1.425, max 1.713, extra 25.67 ms/gain. Draws ~7x the pilot's (0.66 vs 0.09 ms): the bimodal mode material-f386ae studies. OUT cost-full-1792146671.
- 2026-10-16T11:53:31Z (material-abc08c): Task 14 Step 4 contingency landed as f548b835 (perf(material): rigid tilt view path as a second program variant): rest program is the base 23039f67 shader text (stripped-source diff empty), view variant bound only when a swing runs; has_program requires both programs. Tests: material_source 1, the_view_uniforms_are_declared 1, hook/uniform/identity-pose 3, identity byte test 1, view_tilt 16, glslangValidator rest/view/grain ok, test-fast 630 passed 3 skipped. Rest identity rerun: 137 files byte-identical to the base dumps (contingency-20261016T0749/head). Note: nightly cargo fmt --all reformats 12 untouched files today; only own edits kept. Rerunning Steps 2-3.
- 2026-10-16T12:11:39Z (material-abc08c): run: 18 min (est 13, headless); build 3, load wait 4, preflight 1, sub-runs 12 + swing 10; passed (clean release). Contingency f548b835 pilot (3 steps, P0 mode): tilt0 a 0.0952/0.0952 b 0.0891/0.0963; tilt10 a 0.0957/0.0968 b 0.0947/0.0968; fine a 0.1050/0.0963 b 0.1050/0.0968 ms; all INCONCLUSIVE on repeat spread, a and b now overlap; swing 110 frames median 0.201 max 1.703 extra 38.6 ms/gain. OUT cost-pilot-1792151600. Full next.
- 2026-10-16T12:23:35Z (material-abc08c): run: 12 min (est 20, headless); build 0.5, load wait 1, preflight 1, sub-runs 12 + swing 9; passed (clean release). Contingency f548b835 full: tilt0 a 0.6577 b 0.6561 bound 0.0066 PASS; tilt10 a 0.6589 b 0.6587 bound 0.0066 PASS; fine a 0.7311 b 0.7311 bound 0.0073 PASS. Swing 330 frames, median 1.466, max 1.951, extra 32.36 ms/gain. OUT cost-full-1792152702.
- 2026-10-16T12:25:27Z (material-abc08c): done
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T12:25:27Z (material-abc08c): rest cost within bound after the Step 4 contingency (f548b835): tilt0 0.6577/0.6561, tilt10 0.6589/0.6587, fine 0.7311/0.7311 ms (bounds 0.0066-0.0073); before it the single program was +0.09 ms (FAIL). Swing 1.466 ms per frame median (max 1.951), 32.4 ms per gain. Cost section and performance §7 filled.
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
