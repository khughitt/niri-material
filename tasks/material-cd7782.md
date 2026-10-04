---
id: material-cd7782
title: "Test latency over limit: hook-pre-commit 45.305 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: fix/pre-commit-latency
created: 2026-10-04T15:00:02Z
updated: 2026-10-04T17:03:52Z
started: 2026-10-04T16:48:06Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-04T15:00:01Z"
spec: docs/specs/2026-10-04-pre-commit-tooling-latency-design.md
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-04T15:00:02Z (materials-26.04): breach: titan window 2026-09-27T15:00:01Z..2026-10-04T15:00:01Z: hook-pre-commit median 45.305 s, limit 45 s, 140 runs on 8 days
- 2026-10-04T16:48:06Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T16:48:43Z (fix/pre-commit-latency): resumed
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T16:56:58Z (fix/pre-commit-latency): diagnosis: profiled Python tooling through just test-fast: 287 tests passed (2 skipped), 237.311 s; test_optic_settling 186.357 s and test_vt_lib 23.181 s dominate. Non-Rust consistency commands total 2.223 s. Recent successful hooks grew from Sep 30 median 27.806 s to Oct 4 median 246.593 s. A temporary sleep-acceleration probe reduced the slowest screencast test from 46.755 s to 13.623 s but changed the dead-consumer failure from exited to never became ready, so indiscriminate time compression is rejected; no repository implementation changed.
- 2026-10-04T17:01:10Z (fix/pre-commit-latency): probe correction: the first fast-tier timing probe patched separately imported modules and therefore still ran driver tests; stopped it with SIGINT. The next probe exposed a missing repository root on the scratch script import path and failed with 2 import errors. The corrected probe discovers the canonical modules, asserts the full 287-test inventory, and excludes exactly 32 process-lifecycle cases; only this corrected result will inform the design.
- 2026-10-04T17:02:48Z (fix/pre-commit-latency): design: recommend an explicit fast tooling tier for commit checks, with 32 process lifecycle cases retained at their real timings in full validation and a new CI tooling job. Corrected temporary tier probe: full 287-case inventory, 32 intended skips plus 2 existing skips, 26.700 s, exit 0. Written spec awaits owner review; no implementation has started.
- 2026-10-04T17:03:51Z (fix/pre-commit-latency): spec self-review: checked scope, tier defaults, full-CI coverage, host-budget independence, preservation of real timing assertions, and incident verification. The complete post-remedy hook is explicitly unverified; plan and implementation remain gated on owner review.
- 2026-10-04T17:03:51Z (fix/pre-commit-latency): parked (waiting on user, review): Owner: review .worktrees/pre-commit-latency/docs/specs/2026-10-04-pre-commit-tooling-latency-design.md and accept or revise the fast/full tooling split. After acceptance, agent writes and submits the implementation plan in this worktree; implementation follows the separate plan review.
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
