---
id: material-4c455e
title: Bucketed full remainder and the 16-worker grant ceiling
status: done
priority: 0
size: s
complexity: low
process: direct
owner: fix/pre-commit-latency-2
created: 2026-10-07T10:57:28Z
updated: 2026-10-07T11:41:29Z
started: 2026-10-07T11:31:31Z
completed: 2026-10-07T11:41:28Z
depends: [material-8a5af7]
parent: material-5094f2
tags: [testing]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md
step: "Task 2: Bucketed full remainder and the 16-worker grant ceiling"
---

## Notes

- 2026-10-07T11:31:31Z (fix/pre-commit-latency-2): started
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-07T11:37:08Z (fix/pre-commit-latency-2): correctness: coordinator 22 tests passed after expected RED failures; full tooling passed 425 cases (skipped=2), 273.425 s with host-budget granting one worker under load 11.35; sequential native reference now running
- 2026-10-07T11:41:28Z (fix/pre-commit-latency-2): native comparison: full coordinator and sequential discovery both passed 425 cases with skipped=2; sequential 270.701 s; correctness run used the granted worker count rather than assuming an idle host
- 2026-10-07T11:41:28Z (fix/pre-commit-latency-2): done
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-07T11:41:28Z (fix/pre-commit-latency-2): full remainder runs in four module buckets ahead of lifecycle cases; granted budgets reach 16 workers
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
