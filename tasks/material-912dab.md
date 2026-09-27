---
id: material-912dab
title: "Task 6: Evidence — smoke, sheets, acceptance, docs, pin"
status: done
priority: 2
size: m
complexity: mid
process: direct
owner: material-2c3984
created: 2026-09-20T01:53:29Z
updated: 2026-09-21T09:27:31Z
started: 2026-09-20T10:38:50Z
completed: 2026-09-21T09:27:31Z
depends: [material-19f0e9]
parent: material-2c3984
tags: [rendering, signals]
agent: claude-code
plan: docs/plans/2026-09-19-ring-beam.md
step: "Task 6: Evidence — smoke, sheets, acceptance, docs, pin"
---

## Notes

- 2026-09-20T10:38:50Z (material-2c3984): started
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-20T11:12:19Z (material-2c3984): smoke cases: 28/28 pass on 0219599f (run signals-11660-1789901391); beam-run 146 redraws during the 3 s run, 0 in the 12 s after; focused-settled and beam-off 0; first attempt failed demand-pulse-focused (7 redraws) on a kitty.conf write reloading the nested kitties, rerun clean
- 2026-09-20T11:45:00Z (material-2c3984): sheets not recorded: capture preflight refused three runs (GPU util 9.5 %, P5 samples, power IQR 3.3 W) and seven launch windows over ~20 min never saw a quiet GPU; ring-motion-clips.sh scratch_build local-expansion bug fixed; release binary fcb57713 built
- 2026-09-20T11:52:21Z (material-2c3984): Smoke 28/28 on 0219599f (beam-run 146 during / 0 after); evidence draft + §7 docs at 7a6708a4. Sheets not recorded: clip preflight refused (GPU util 9.5 %, P5 samples) with the desktop in use.
- 2026-09-20T11:52:21Z (material-2c3984): parked (waiting on user, quiet; idle, 40 min): From the worktree: CAPTURE_TASK=material-912dab NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material setsid nohup docs/materials/scripts/ring-motion-clips.sh > /mnt/ssd3/niri-material/ring-beam-clips.log 2>&1 & — then fill the evidence doc's Review sheets section and hand the sheets/corner crops to the owner (beam-run vs gap16, spill vs nospill, plateau vs splash, 300 vs 900, bevel0); after acceptance: Step 5 close commit, Step 6 finish/pin/rollout
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T08:34:23Z (material-2c3984): resumed
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T09:00:56Z (material-2c3984): sheets recorded: ring-motion-clips-35e736b6/ring-clips-2977602-1789979677, 6/6, preflight quiet (GPU 0 % P8, tty session); smoke rerun on 35e736b6 aborted at idle-resume 99 (bound 100-166): same 136-redraw burst as the 0219599f trace, but the case's window is anchored to the trace end and this capture ended 1 s sooner; beam-run passed before the abort; rerunning
- 2026-09-21T09:21:24Z (material-2c3984): smoke rerun 28/28 on 35e736b6 (signals-3039250-1789981256): beam-run 152 during / 0 after, idle-resume 108; evidence doc filled: sheets, head trace, review composites, both smokes, aborted-run analyses; idle-resume windowing filed as material-85f0c0
- 2026-09-21T09:22:05Z (material-2c3984): parked (waiting on user, review): Owner judges the sheets: gifs/sheets under ring-motion-clips-35e736b6/ring-clips-2977602-1789979677/ and composites under ring-motion-clips-35e736b6/review/ (gap 8 vs 16, spill vs nospill vs bevel0, plateau vs splash, 300 vs 900); record accepted constants in the evidence doc's last section (any change: ring.rs + shader + niri-config + Prism defaults together, just test, re-record). Then Step 5: tasks done material-912dab, material-9306b5, material-2c3984; spec status date; just test && just check; close commit. Then Step 6: finishing-a-development-branch, package pin, rollout order (makepkg -si, then merge prism-1514d3, prism migrate/apply).
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T09:26:40Z (material-2c3984): resumed
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T09:27:31Z (material-2c3984): done
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T09:27:31Z (material-2c3984): smoke 28/28 on 35e736b6; six sheets recorded on a quiet host; constants accepted by the owner 2026-09-21, defaults unchanged; docs updated
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
