---
id: material-9f9ca2
title: Verify concurrency stability and close latency evidence
status: done
priority: 0
size: m
complexity: mid
process: direct
owner: fix/pre-commit-latency
created: 2026-10-04T20:39:13Z
updated: 2026-10-05T10:26:30Z
started: 2026-10-04T23:16:58Z
completed: 2026-10-05T10:16:51Z
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
- 2026-10-05T09:42:05Z (fix/pre-commit-latency): Main advanced during implementation with capture-hold tooling, new tests and lifecycle script changes. Integrate committed main changes into this isolated worktree before final acceptance; retain the completed 320-case host set as earlier evidence. Main uncommitted task files remain untouched.
- 2026-10-05T10:15:51Z (fix/pre-commit-latency): attached: material-verified-latency-evidence.md (1065802 bytes): Verified merged398-case parity/stability/cancellation, all stages, three qualifying real hooks per route and actual six-run verifier success.
- 2026-10-05T10:16:51Z (fix/pre-commit-latency): done
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T10:16:51Z (fix/pre-commit-latency): Implemented and reviewed parallel fast validation; merged current capture-hold tooling.398-case native/fast parity,10 isolated locked and5 consecutive full greens per environment, actual driver cancellation clean. Three qualifying whole hooks per route: fast median24.818s/full40.752s. Actual tt-latency verify exit0 on titan: hook-pre-commit median32.447s, limit45s,6 successful uncontended unwidened runs; every obligation met. Evidence attached; temporary verifier registry only, no shared pointers changed.
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T10:26:29Z (fix/pre-commit-latency): attached: material-integration-verification-evidence.md (130320 bytes): Retained controller HEAD-race failure during redundant completion check; diagnosed source identity mismatch, stable-HEAD focused and full398-case reruns green without source/bound changes.
- 2026-10-05T10:26:29Z (fix/pre-commit-latency): Post-acceptance integration bookkeeping: controller started redundant completion check before metadata-only reconciliation merge finished, causing binary-source/HEAD mismatch in preflight fixture. Retained failed receipt; stable-HEAD focused case and full398-case SDD completion verification passed. No source or timing assertion changed; all acceptance remains on the same runtime code.
