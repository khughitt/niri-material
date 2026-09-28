---
id: material-204be4
title: Live pilots (quiet host; the person runs them)
status: done
priority: 2
size: s
complexity: low
process: direct
owner: material-b15ad7
created: 2026-09-27T13:19:02Z
updated: 2026-09-28T03:27:54Z
started: 2026-09-27T14:13:46Z
completed: 2026-09-28T03:27:54Z
depends: []
parent: material-b15ad7
tags: [performance]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-09-27-idle-budget-fail-fast.md
step: "Task 8: Live pilots (quiet host; the person runs them)"
---

## Notes

- 2026-09-27T14:13:46Z (material-b15ad7): started
  provenance: {"harness_session":"codex:01a0e32b-8d4c-7d40-9694-4c3b6a75bb15","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-27T14:14:55Z (material-b15ad7): done
  provenance: {"harness_session":"codex:01a0e32b-8d4c-7d40-9694-4c3b6a75bb15","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-27T14:14:55Z (material-b15ad7): niri-experiments 40452bd: partial analysis, stop and teardown reporting, descriptive pilot block; 45 fixture tests pass
  provenance: {"harness_session":"codex:01a0e32b-8d4c-7d40-9694-4c3b6a75bb15","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-27T14:29:01Z (material-b15ad7): resumed
  provenance: {"harness_session":"codex:01a0e32b-8d4c-7d40-9694-4c3b6a75bb15","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-27T14:29:01Z (material-b15ad7): parked (waiting on user, quiet; headless, 35 min): User runs the three supervised trace, interrupted-trace, and power pilots from a headless TTY using docs/plans/2026-09-27-idle-budget-fail-fast.md Task 8; agent then checks verdicts, stop record, checksums, files run notes, and closes this child and material-b15ad7
  provenance: {"harness_session":"codex:01a0e32b-8d4c-7d40-9694-4c3b6a75bb15","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-27T14:29:13Z (material-b15ad7): Correction: this is Task 8. Its earlier done status and completion note were assigned to the wrong child; no live pilots have run. It is parked for the user on a quiet headless host.
- 2026-09-27T14:50:28Z (material-b15ad7): Offline implementation and fresh review are complete at niri-experiments d7593b7 (67/67 fixture tests). Run the supervised trace, interrupted trace, and power pilots from the plan on the quiet host; validate their artifacts before closing this task.
- 2026-09-28T02:38:55Z (material-b15ad7): resumed
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T02:40:50Z (material-b15ad7): run: 1 min (est 12, headless); validate 1; refused: trace-target niri (source d91a6024, pre-beam branch) rejects ring-beam-speed that the checkout's glass-optic-smoke-lib now emits since the 2026-09-27 merge; rebuilding both binaries from HEAD into trace-target-/power-target-<T>
- 2026-09-28T02:56:27Z (material-b15ad7): run: 3 min (est 12, headless); build 3 (both binaries from 4607d054, trace-target-/power-target-20260927T224045), trace 2.5; failed: A-move-1 1 redraw at 30.405 s trace time, 0 material draws, pixels equal; fail-fast stop.json first-failure, not_run 3, SHA256SUMS ok (pilot-trace-20260927T224948)
- 2026-09-28T02:56:27Z (material-b15ad7): run: 3 min (est 12, headless); trace 2.5; failed: reproduced exactly, A-move-1 redraw at 30.405 s again (pilot-trace-20260927T225321). The 2026-09-25 trace on d91a6024 had 0 redraws in A; suspect the HEAD material tick deadline (ring beam / signal work). Running --inventory to exercise all four pilot cases
- 2026-09-28T03:13:59Z (material-b15ad7): run: 8 min (est 6, headless); trace 8; aborted: the interrupt was never delivered. The agent's harness started the supervisor with & from a non-interactive shell, so INT was ignored at entry and its trap could not install; the run completed as a second inventory pilot (pilot-trace-int-20260927T230524: same A 1-redraw FAILs, C 81, D 41 pass). Rerunning with env --default-signal=INT, which matches a TTY Ctrl-C to the supervisor alone
- 2026-09-28T03:19:04Z (material-b15ad7): run: 8 min (est 12, headless); trace 8; failed: pilot 1 as --pilot --inventory (fail-fast would stop at A-move-1): four verdicts, analysis pilot true, complete true, no stop.json, SHA256SUMS ok, no settle refusal; A-move-1 and A-resize-1 FAIL with 1 redraw at 30.4 s (material-cd7deb), C-move-1 81/81 and D-move-1 41/41 pass (pilot-trace-inv-20260927T225621)
- 2026-09-28T03:19:04Z (material-b15ad7): run: 5 min (est 6, headless); trace 4.5, teardown after INT 0.2; passed: pilot 2 interrupted after the second verdict, supervisor exit 130, stop.json kind signal message INT exit 130 during C-move-1, analysis covers A-move-1 and A-resize-1 with not_run C and D, SHA256SUMS ok, no leftover processes (pilot-trace-int-20260927T231359)
- 2026-09-28T03:27:54Z (material-b15ad7): run: 8 min (est 12, headless); preflight 0.5, power 7.5; passed: pilot 3 on DRM DP-1 3440x1440@59.999 scale 1, four sham block 1 verdicts (medians 11.385, 11.395, 11.465, 11.460 W), analysis pilot true, complete true, pilot_block present (descriptive, block delta 0.0075 W), no stop.json, SHA256SUMS ok, preflight passed with seat managers pinned to pid 1 and systemd-logind MainPID 980 (pilot-power-20260927T231857)
- 2026-09-28T03:27:54Z (material-b15ad7): done
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-28T03:27:54Z (material-b15ad7): Live pilots on 2026-09-27 from a TTY with binaries rebuilt from 4607d054: all four pass criteria met. Trace pilot (inventory) has four verdicts, is complete, and has no stop record; interrupted pilot recorded a signal stop with INT and exit 130; power pilot passed under the pinned seat exemption; no settle refusals. The quiet A cases fail on a niri idle-edge redraw, filed as material-cd7deb
  provenance: {"harness_session":"claude-code:1462ad85-ed02-4290-a860-8f7187b484a9","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
