---
id: material-6bf694
title: "Development check, pilot and matrix on the TTY"
status: todo
priority: 1
size: m
complexity: mid
process: direct
created: 2026-10-02T21:02:02Z
updated: 2026-10-03T02:05:13Z
depends: [material-a3ca2b]
parent: material-f7eb0b
tags: [performance]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-02-real-tty-settling-lane.md
step: "Task 7: Development check, pilot and matrix on the TTY"
---

## Notes

- 2026-10-03T01:51:09Z (material-3acc86): parked (waiting on user, quiet; headless, 30 min): From a TTY with the desktop stopped, in .worktrees/material-3acc86, following plan Task 7: display-dim if installed (1), build + snapshot niri-tracy at HEAD and wait for load1<1 (5), development check CASES='drm-aurora tty-resume screencast' per docs/materials/capture-host-setup.md (4), pilot (6), matrix with PILOT_DIR (12), restore dim, publish evidence, close material-f7eb0b and material-3acc86. Do not commit between snapshot and matrix. Watch the vt-away window margin under the 6 s cap. Needs the NOPASSWD /usr/bin/chvt rule and gst-plugin-pipewire (owner items).
  provenance: {"harness_session":"claude-code:bf767efc-bb55-40aa-933b-17ef6962442f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T02:05:13Z (material-3acc86): host: NOPASSWD /usr/bin/chvt rule installed and verified (sudo -n -l); gst-plugin-pipewire 1.6.9 installed — owner, 2026-10-02
