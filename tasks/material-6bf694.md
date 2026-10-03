---
id: material-6bf694
title: "Development check, pilot and matrix on the TTY"
status: done
priority: 1
size: m
complexity: mid
process: direct
owner: material-3acc86
created: 2026-10-02T21:02:02Z
updated: 2026-10-03T06:17:36Z
started: 2026-10-03T03:48:04Z
completed: 2026-10-03T06:17:36Z
depends: [material-a3ca2b]
parent: material-f7eb0b
tags: [performance]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-02-real-tty-settling-lane.md
step: "Task 7: Development check, pilot and matrix on the TTY"
---

## Notes

- 2026-10-03T01:51:09Z (material-3acc86): parked (waiting on user, quiet; headless, 30 min): From a TTY with the desktop stopped, in .worktrees/material-3acc86, following plan Task 7: display-dim if installed (1), build + snapshot niri-tracy at HEAD and wait for load1<1 (5), development check CASES='drm-aurora tty-resume screencast' per docs/materials/capture-host-setup.md (4), pilot (6), matrix with PILOT_DIR (12), restore dim, publish evidence, close material-f7eb0b and material-3acc86. Do not commit between snapshot and matrix. Watch the vt-away window margin under the 6 s cap. Needs the NOPASSWD /usr/bin/chvt rule and gst-plugin-pipewire (owner items).
  provenance: {"harness_session":"claude-code:bf767efc-bb55-40aa-933b-17ef6962442f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T02:05:13Z (material-3acc86): host: NOPASSWD /usr/bin/chvt rule installed and verified (sudo -n -l); gst-plugin-pipewire 1.6.9 installed — owner, 2026-10-02
- 2026-10-03T03:48:04Z (material-3acc86): started
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T05:28:24Z (material-3acc86): undimmed: display-dim not installed
- 2026-10-03T05:39:08Z (material-3acc86): run: 4 min (est 4, headless); dev check 4; failed: analyzer 'invalid cpu.csv time' — tracy-csvexport leaves zone names unquoted and the DRM path's MultiRenderer<'_, '_, '_> zones shift text into ns_since_start. Re-analysis of a copy with the fixed parser: development-passed (tty-resume edges [0,1,0], vt-out 3.2 s under the 6 s cap, screencast passed, vt-restore not-needed). OUT tty-dev-20261003-1, binary bin-54e30870
- 2026-10-03T05:39:09Z (material-3acc86): fix: zone exports rejoin a comma-bearing name's spilled fields (tools/optic_settling.py zone_rows); headless runs only shifted single-comma rows under unused names, so the analyzer's four zones (refresh_idle_inhibit, redraw, material draw, notify_activity) were read correctly there. Checked on 830,669 rows from 27 exports: 0 rows left malformed
- 2026-10-03T05:53:09Z (material-3acc86): run: 4 min (est 4, headless); dev check 4; passed: development-passed, drm-aurora/tty-resume/screencast passed, vt-restore not-needed. OUT tty-dev-20261003-2, binary bin-3125163e
- 2026-10-03T06:09:21Z (material-3acc86): run: 6 min (est 6, headless); pilot 6; passed: lane-passed, drm-aurora/tty-resume/unlock/screencast passed, vt-restore not-needed. OUT tty-pilot-20261003-1
- 2026-10-03T06:09:21Z (material-3acc86): run: 10 min (est 12, headless); matrix 10; passed: lane-passed, 8 passed incl tty-resume-r2/r3 and unlock-r2/r3, vt-restore not-needed. OUT tty-matrix-20261003-1
- 2026-10-03T06:17:36Z (material-3acc86): done
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T06:17:36Z (material-3acc86): dedicated lane: development, pilot and matrix lane-passed on DP-1 at 3125163e (tty-resume, unlock x3 each, screencast); analyzer parses unquoted comma zone names
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
