---
id: material-ee451d
title: "Ring motion evidence: redraw counts, clips, closing docs"
status: doing
priority: 1
size: m
complexity: mid
process: direct
owner: material-82323e
created: 2026-09-19T02:49:04Z
updated: 2026-09-19T10:39:42Z
started: 2026-09-19T10:31:34Z
depends: [material-63e077]
parent: material-0e130e
tags: [ring, testing]
agent: claude-code/claude-opus-5
plan: docs/plans/2026-09-18-ring-focus-motion.md
step: "Task 4: Evidence — redraw counts, clips, and closing docs"
---

## Notes

- 2026-09-19T10:31:34Z (material-82323e): started
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T10:39:04Z (material-82323e): Steps 1-2 done: base fixtures carry signal { idle-after-ms 0 } (extra signal lines fold into the one block; every fixture is validated by the binary), sweep-toggle/idle-pulse/idle-resume/dpms-off-pulse cases added; ring-motion-clips.sh written for the four review clips under capture-meta. Dry run wrote and validated every fixture and stopped at the wlrctl gate: wlrctl (AUR, yay -S wlrctl) is not installed on this host.
- 2026-09-19T10:39:42Z (material-82323e): parked (waiting on user, environment): Install wlrctl (AUR: yay -S wlrctl), then in .worktrees/material-82323e: NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT docs/materials/scripts/material-signals-smoke.sh cases 2>&1 | tee $NIRI_MATERIAL_WORK_ROOT/ring-motion-cases.log; then CAPTURE_TASK=material-ee451d docs/materials/scripts/ring-motion-clips.sh; then write docs/materials/2026-09-18-ring-focus-motion-evidence.md, set the two status headers, tasks done material-ee451d and material-0e130e
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
