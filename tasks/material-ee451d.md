---
id: material-ee451d
title: "Ring motion evidence: redraw counts, clips, closing docs"
status: done
priority: 1
size: m
complexity: mid
process: direct
owner: material-82323e
created: 2026-09-19T02:49:04Z
updated: 2026-09-19T22:33:56Z
started: 2026-09-19T10:31:34Z
completed: 2026-09-19T22:33:56Z
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
- 2026-09-19T10:48:39Z (material-82323e): resumed
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T11:19:26Z (material-82323e): Smoke cases: all 30 pass at 4aad1587 (settled/reduced/anim-off/none/other-focused 0; sweep-toggle 338 total, 0 in the 9.4 s tail; idle-pulse 0; idle-resume 115 in 5 s then 0; dpms-off-pulse 0). Clips: capture-meta refused twice on desktop load (load1 5.43 then 6.65, GPU P0/P5, power IQR 5-8 W); evidence record written with both refusals.
- 2026-09-19T11:19:26Z (material-82323e): parked (waiting on user, quiet; idle, 10 min): On a quiet host, in .worktrees/material-82323e: CAPTURE_TASK=material-ee451d NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT docs/materials/scripts/ring-motion-clips.sh; then append the clip table (directory, frame count, capture.json hash) to docs/materials/2026-09-18-ring-focus-motion-evidence.md, update its status and the spec's, tasks done material-ee451d and material-0e130e
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T21:56:06Z (material-82323e): resumed
- 2026-09-19T22:33:03Z (material-82323e): Clips recorded: ring-motion-clips-079ed613/ring-clips-3278563-1789856906 (gain-from-rest 25, alt-tab-three 39, loss-mid-lap 24, idle-freeze 30, idle-resume 30 frames; preflight quiet, every settle P8; capture.json ebdeb4a6…). Fixture fixes on the way: skip the startup hotkey overlay; wait for the GPU P0→P8 step-down after each nested stop. Third preflight refusal was an orphaned bun test process (ops-38be00).
- 2026-09-19T22:33:56Z (material-82323e): resumed
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T22:33:56Z (material-82323e): took over a live claim held by session sid:3249807 (owner material-82323e, host titan, pid 3249807, worktree /mnt/ssd3/work/niri-material/.worktrees/material-82323e, since 2026-09-19T21:56:06Z, age 2270s, live)
- 2026-09-19T22:33:56Z (material-82323e): done
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T22:33:56Z (material-82323e): smoke cases pass on the headless host; clips recorded; status headers updated
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
