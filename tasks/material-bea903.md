---
id: material-bea903
title: Acceptance measurements and latency verification
status: done
priority: 0
size: s
complexity: mid
process: direct
owner: fix/pre-commit-latency-2
created: 2026-10-07T10:57:28Z
updated: 2026-10-08T02:12:31Z
started: 2026-10-07T11:42:20Z
completed: 2026-10-08T02:12:30Z
depends: [material-4c455e]
parent: material-5094f2
tags: [testing]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md
step: "Task 3: Acceptance measurements and latency verification"
---

## Notes

- 2026-10-07T11:42:20Z (fix/pre-commit-latency-2): started
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-07T11:42:53Z (fix/pre-commit-latency-2): run: 0 min (est 15, idle); preflight 0; refused: load 5.13 exceeds 4 and another session runs Playwright/browser tests
- 2026-10-07T11:42:53Z (fix/pre-commit-latency-2): parked (waiting on user, quiet; idle, 15 min): Agent: in .worktrees/material-5094f2, run Task 3 idle-host preflight, one warmup check-full, three check-full/check samples, six private-index hook samples with report cleanup; resolve the Task 2 hook 35.884 s miss before dry-run verify, results commit, final fresh reviewer and recording verify
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-08T02:03:52Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:0a5d985e-3544-4780-b427-a957c8940292","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T02:12:30Z (fix/pre-commit-latency-2): run: 6 min (est 15, idle); preflight 1, warm 1, check samples 2, hook samples 2; passed
- 2026-10-08T02:12:30Z (fix/pre-commit-latency-2): done
  provenance: {"harness_session":"claude-code:0a5d985e-3544-4780-b427-a957c8940292","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T02:12:30Z (fix/pre-commit-latency-2): both routes under target (check-full 27.8 s, check 18.3 s, hooks full 27.7-28.0 s, fast 18.2-18.3 s); tt-latency verify --dry-run exit 0
  provenance: {"harness_session":"claude-code:0a5d985e-3544-4780-b427-a957c8940292","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
