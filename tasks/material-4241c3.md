---
id: material-4241c3
title: Verify material quiescence and bounded animation cadence
status: done
priority: 2
size: m
complexity: mid
process: direct
owner: material-265eb0
created: 2026-09-11T10:27:54Z
updated: 2026-09-26T03:14:16Z
started: 2026-09-12T09:57:45Z
completed: 2026-09-26T03:14:16Z
depends: [material-ec6229]
parent: material-265eb0
tags: [performance]
model: "claude-opus-5-5[1m]"
spec: docs/specs/2026-09-11-material-idle-budget-design.md
plan: docs/plans/2026-09-11-material-idle-budget.md
step: "Task 2: Verify quiescence and bounded active cadence"
---

After execution resumes and the fixture is validated, collect 24 short trace observations and one 600 s quiet hold. Verify exact quiet redraw/material-draw zeros, three-second settling and inter-stimulus wait, heartbeat coverage, pixel return, and 4/2 Hz Aurora cadence. Match the complete startup/setup/stimulus IPC journal to trace markers and retain independently reviewable raw evidence. No board-power claim from the shared desktop.

## Notes

- 2026-09-11T10:30:13Z (material-265eb0): parked (waiting on user): Prepared only; await user resume and completed fixture task before trace collection.
- 2026-09-12T09:59:44Z (material-265eb0): User resumed trace execution on 2026-09-12. Integrated capture protocol into materials-26.04 at a2b9c58c, experiment results/idle-budget at 9d7d43f, and native execution worktree via 8e09f91b. Readiness preflight precedes expensive build/capture; no threshold overrides or user-session changes.
- 2026-09-12T10:01:44Z (material-265eb0): Readiness on 2026-09-12T06:00:38-04:00 sampled 20 seconds after integration tests finished: exit 1; CPU 25.5% > 10%, load1 12.89 > 2.0, GPU 34.5% > 5%, mixed P5/P8, power IQR 5.64 W > 1.0 W. Graphics clients: Xwayland, firefox, kitty, niri, noctalia, qs; no compute clients. Release exited 0. Retained capture.json and logs under NIRI_MATERIAL_WORK_ROOT/material-265eb0/readiness-20260912-15msz4ks. No release build or compositor capture started.
- 2026-09-12T10:01:44Z (material-265eb0): Merged verification passed: 434 Rust/unit/doc tests, 126 tooling tests, 22 idle-budget fixture tests.
- 2026-09-12T10:01:44Z (material-265eb0): parked (waiting on user, environment): Resume the authorized trace build and approximately 50-minute capture after a real headless readiness preflight passes on a quiet desktop. Keep default thresholds; retained refusal record readiness-20260912-15msz4ks explains current load. Power still requires the operator-arranged dedicated session.
- 2026-09-23T19:40:37Z (material-265eb0): resumed
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T19:40:37Z (material-265eb0): parked (waiting on user, quiet; idle, 50 min): Resume the authorized trace build and approximately 50-minute capture after a real headless readiness preflight passes on a quiet desktop. Keep default thresholds; retained refusal record readiness-20260912-15msz4ks explains current load. Power still requires the operator-arranged dedicated session.
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T20:25:57Z (material-265eb0): resumed
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T20:56:53Z (material-265eb0): 2026-09-24 attempts (rotation timer stopped, monitors off): build-tracy 16:28:03-16:30:57 (2 min 54 s). Trace 1 refused at preflight 16:32:56 on load1 3.22 (90 s settle after a 32-core build is too short; wait for load1 < 1.0). Trace 2 16:34:12-16:56:03 settled 11 of 24 cases (~2 min each), then refused before B-resize-3: gpu_power_w 11.585 vs baseline 9.9 (+1.5 W limit) after the desktop idle lock engaged at 16:54:05 and raised the P8 power floor ~1.6 W (util 0). Trace 3 started ~16:58 from the locked state so its baseline matches. Export bound: d91a6024 here, 38b13a2 in experiments.
- 2026-09-24T21:13:21Z (material-265eb0): Trace 3 16:56:49-17:10:58 (locked state, baseline 10.055 W P8 util 0) settled 11 of 24 again, then refused before B-move-2: gpu_util_pct median 5.0 vs baseline 0.0 (+3.0 limit), power 10.0 W P8; clients only the desktop's own (Xwayland, kitty, niri, noctalia, qs). A 60 s nvidia-smi pmon afterwards showed zero use: intermittent desktop GPU blips. With 24 one-shot 10 s settle windows the run needs a desktop that never draws; three evening attempts each failed on a different transient. Host changes restored at 17:15 (glass unset, wali-rotate.timer started).
- 2026-09-24T21:13:21Z (material-265eb0): parked (waiting on user, decision): Decide how the 24 settle gates survive desktop blips, then rerun trace only (binary built: NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target/release/niri; runner waits for load1 < 1.0, powers monitors off). Options: (a, recommended) run from a TTY with the desktop session stopped (--needs headless), no protocol change; (b) let the fixture retry a refused settle a bounded number of times, each refusal kept in capture.json, a change to the reviewed Task 1 fixture. ~60 min once started.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T23:01:35Z (material-265eb0): resumed
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T23:01:35Z (material-265eb0): User decision 2026-09-24: next attempt runs from a TTY with the desktop session stopped (option a); no fixture or protocol change.
- 2026-09-24T23:01:35Z (material-265eb0): parked (waiting on user, quiet; headless, 60 min): From a TTY with the desktop session stopped, in .worktrees/material-265eb0: rerun the trace only (binary built at NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-target/release/niri): wait for load1 < 1.0, then OUT=$NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-<ts> NIRI_BIN=<that binary> MATERIAL_ROOT=$PWD just --justfile <experiments>/.worktrees/material-265eb0/fixtures/idle-budget.just trace (the scratch runner 4241c3-trace.sh did exactly this; drop its power-off-monitors). Expect 24 settled cases + 600 s hold, then analysis.json; then the plan's Task 2 review and commit.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-25T08:31:23Z (material-265eb0): resumed
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-25T08:31:23Z (material-265eb0): TTY run 2026-09-25: desktop stopped (niri.service inactive, no noctalia); wali-rotate.timer stopped for the run (restore: systemctl --user start wali-rotate.timer).
- 2026-09-25T09:31:22Z (material-265eb0): TTY trace 2026-09-25 04:31:29-05:29:49 (58 min 20 s against the 60 min estimate): capture valid, all 25 settle gates passed, analysis ran. Behavioral gates: C and D cadence pass (81 and 41 redraws with draws); every quiet observation (A, B, P, O, 600 s hold) fails on one redraw 3.00 s after the last stimulus redraw, zero material draws, pixels equal. Filed material-4be9c3 per the plan's rule; evidence kept at NIRI_MATERIAL_WORK_ROOT/material-265eb0/trace-20260925T043128. wali-rotate.timer restored at 05:3x.
- 2026-09-25T09:31:22Z (material-265eb0): parked (waiting on agent, dependency): After material-4be9c3 lands (the trailing 3.0 s redraw fixed or accounted for), rerun the trace from a TTY with the desktop stopped (runner as in the 2026-09-25 note; rebuild the trace binary first if the fix is in niri), then the plan's Task 2 review and commit.
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-26T02:06:00Z (material-265eb0): material-4be9c3 done (fixture fix niri-experiments 0f7d00f, pilots pass one case of every kind). Full TTY trace rerun started 2026-09-25 evening on the same trace binary; wali-rotate.timer stopped (restore: systemctl --user start wali-rotate.timer).
- 2026-09-26T03:14:16Z (material-265eb0): done
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-26T03:14:16Z (material-265eb0): Task 2 complete: TTY trace 2026-09-25 22:07-23:05 (58 min 23 s), 25/25 observations pass (A/B/P/O and the 600 s hold 0 redraws / 0 material draws / pixels equal; C 80/80, D 40/40), preflight quiet, 25/25 settled; independent raw recomputation matches analysis.json exactly. Evidence: docs/materials/2026-09-11-idle-budget-evidence.md; experiments results/idle-budget f0c13ac (pushed), fixture fix 0f7d00f. Board-power acceptance still pending Task 3 (material-5f9dee).
  provenance: {"harness_session":"claude-code:29e26027-f18b-4d4f-9742-5dc4a9c688ec","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
