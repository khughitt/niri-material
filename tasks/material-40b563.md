---
id: material-40b563
title: Cost captures with Tracy for the backdrop grain
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: material-cf32e5
created: 2026-10-05T14:01:38Z
updated: 2026-10-06T03:43:51Z
started: 2026-10-05T15:21:05Z
completed: 2026-10-06T03:43:51Z
depends: [material-2a1689]
parent: material-cf32e5
tags: []
model: claude-opus-5-5
agent: claude-code
plan: docs/plans/2026-10-05-noise-placement.md
step: "Task 6: Cost captures with Tracy"
---

Task 6 of docs/plans/2026-10-05-noise-placement.md.

## Notes

- 2026-10-05T15:21:05Z (material-cf32e5): started
  provenance: {"harness_session":"codex:01a10c84-1457-7892-bb5d-ed0227e1d702","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T15:42:03Z (material-cf32e5): Prepared all six Tracy cases, timestamped stimulus windows and owned-wallpaper cleanup; real-wrapper failure regression and CSV report test pass. Cost measurements remain unrun pending a quiet host.
- 2026-10-05T16:56:59Z (material-cf32e5): parked (waiting on agent, dependency): Agent after material-2a1689 passes, on the same quiet host: run noise-placement-cost.sh pilot (6 min), read all six verdicts, then full (6 min), analysis/evidence (4 min); pgrep swaybg before/after, record each run, restore display-dim before release. Append measured costs and renderer string and close this child in the evidence commit; then rerun gate, review changed evidence, integrate and complete material-e50969.
  provenance: {"harness_session":"codex:01a10c84-1457-7892-bb5d-ed0227e1d702","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T03:17:17Z (material-cf32e5): resumed
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T03:22:11Z (material-cf32e5): run: 3.2 min (est 6, headless); preflight+build 1, static+damage+drag backdrop 2.2; aborted: drag-backdrop 45 reload markers, expected 52. Cause: reload_marker used niri-tracy as the msg client, whose Tracy startup calibration costs 539 ms (plain niri msg 18 ms), so 100 ms drag steps ran at ~540 ms and the trace ended before the last 7. Switched the client to the plain binary; reload spans themselves are 0.07 ms. No swaybg left after.
- 2026-10-06T03:28:15Z (material-cf32e5): run: 5.6 min (est 6, headless); preflight+build 1, six cases 4.6; passed: pilot, all six cases complete, drag 50 steps in 5.5 s. Starting full.
- 2026-10-06T03:31:35Z (material-cf32e5): run: 2.4 min (est 6, headless); preflight+build 1, static+damage backdrop 1.4; aborted: damage-backdrop 16 reload markers, expected 21. Same cause: wall_count polled through niri-tracy msg (0.54 s per call), so 1 s damage steps took 1.6 s and 20 of them outlasted the 30 s trace; the 5-step pilot could not show it. All IPC now uses the plain niri client; sleep_until fails a step that overruns by more than half its period so a pilot catches a slow stimulus. Rerunning the pilot.
- 2026-10-06T03:37:26Z (material-cf32e5): run: 5.6 min (est 6, headless); preflight+build 1, six cases 4.6; passed: pilot with plain IPC client and overrun guard. Starting full.
- 2026-10-06T03:43:18Z (material-cf32e5): run: 5.6 min (est 6, headless); preflight+build 1, six cases 4.6; passed: full, all six cases complete (damage 20/20, drag 50/50 stimuli); no swaybg left.
- 2026-10-06T03:43:51Z (material-cf32e5): done
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T03:43:51Z (material-cf32e5): Tracy costs measured for all six cases (noise-cost-20261005-233726); appended to the evidence. IPC now uses the plain niri client (niri-tracy msg costs 539 ms) and stimulus steps fail on overrun.
  provenance: {"harness_session":"claude-code:51715bc9-2053-49ae-bc45-d479866d8d8b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
