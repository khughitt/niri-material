---
id: material-9f9ca2
title: Verify concurrency stability and close latency evidence
status: doing
priority: 0
size: m
complexity: mid
process: direct
owner: fix/pre-commit-latency
created: 2026-10-04T20:39:13Z
updated: 2026-10-05T09:28:59Z
started: 2026-10-04T23:16:58Z
depends: [material-d2b55c]
parent: material-cd7782
tags: [testing]
agent: codex
plan: docs/plans/2026-10-04-pre-commit-tooling-latency-plan.md
step: "Task 6: Verify concurrency stability and close latency evidence"
---

## Notes

- 2026-10-04T23:16:58Z (fix/pre-commit-latency): started
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T23:30:16Z (fix/pre-commit-latency): Final review found three Important gaps; reproduced each through just test-one, corrected cancellation during active cleanup, native expected-failure outcomes and package initializer traversal. Acceptance runs follow scoped re-review.
- 2026-10-04T23:47:45Z (fix/pre-commit-latency): Both environments completed pilots, ten isolated locked cases, five consecutive 317-case full greens and actual-driver cancellation with every recorded PID gone. Final fast pilot 36.179 s missed 35 s target; profiled fixture marker process overhead, replaced markers with native shell built-ins without changing assertions. Gate suite 12 tests remains green; remeasure and rerun acceptance on final revision. Fresh-review tooling feedback filed as flows-4887f2.
- 2026-10-05T00:21:38Z (fix/pre-commit-latency): Final implementation 877b084a passed ten isolated locked cases, five consecutive full greens per environment and actual driver cancellation. Actual hooks: full median 39.293 s, fast 37.238 s; verifier dry-run exit 0 with 5 qualifying runs, median 37.202 s, zero widened, one contended original sample. Keep open: fast 35 s target and third qualifying full hook are unmet. Disposable two-process fast prototype passes all 317 IDs/37 host skips, initial tooling 17.428 s and hook recipe 22.296 s. Proposed spec/Task 6 plan amendment needs owner review before runtime changes.
- 2026-10-05T00:24:36Z (fix/pre-commit-latency): parked (waiting on user, review): Owner: review the parallel-fast execution amendment in .worktrees/pre-commit-latency/docs/specs/2026-10-04-pre-commit-tooling-latency-design.md and its Task 6 steps in .worktrees/pre-commit-latency/docs/plans/2026-10-04-pre-commit-tooling-latency-plan.md. After acceptance, agent implements the two-worker fast route, runs scoped review and repeats final-revision stability and actual qualifying hooks before closure. No host pointers changed; no owned test processes remain.
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T09:06:54Z (fix/pre-commit-latency): resumed
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T09:28:58Z (fix/pre-commit-latency): attached: material-fast-amendment-evidence.md (284086 bytes): Parallel-fast RED/GREEN contracts, exact native/fast 320-ID parity on both environments, cancellation/native outcome controls and guarded Ubuntu CI success.
- 2026-10-05T09:28:58Z (fix/pre-commit-latency): Approved fast amendment implemented: explicit --fast, two module buckets capped by strict NEXTEST_TEST_THREADS, private explicit worker modes, full-only CI and preserved native outcomes/cancellation. Host and Ubuntu fast/native IDs and skips match exactly (320 cases, 37/40 fast skips); full validation passes (2/5 optional skips). Scoped review and final-revision stability/actual-hook acceptance remain.
