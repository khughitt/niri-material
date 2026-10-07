---
id: material-5094f2
title: "Test latency over limit: hook-pre-commit 45.425 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: fix/pre-commit-latency-2
created: 2026-10-07T03:05:29Z
updated: 2026-10-07T09:59:42Z
started: 2026-10-07T03:20:52Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-07T03:05:28Z"
spec: docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-07T03:05:29Z (materials-26.04): breach: titan window 2026-09-30T03:05:28Z..2026-10-07T03:05:28Z: hook-pre-commit median 45.425 s, limit 45 s, 37 runs on 3 days
- 2026-10-07T03:20:52Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:25:53Z (materials-26.04): diagnosis: post-floor window (after 2026-10-05T10:16:51Z) splits by route: fast-tooling 40 runs median 37.5 s, full-tooling 7 runs median 106.6 s, legacy discover 4 runs ~242 s. Idle-host fast route ~35 s: fast tooling 24.1 s (409 cases, 2 children, 34.9 s of case time), clippy 6.4 s, ops-check 2.2 s, fmt 1.2 s, rest <1 s. Live check at 03:21Z reads ok (median 42.256 s, 39 runs) only because tonight's commits were fast-route: borderline, a recurrence of material-cd7782
- 2026-10-07T03:25:53Z (materials-26.04): probe: fast tooling at 2/4/6 children took 24.1/11.4/15.7 s (two runs each, idle host); at 4 and 6 the only failure is test_fast_preserves_native_inventory_and_module_fixtures pinning '2 children'; probe reverted
- 2026-10-07T03:26:03Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:26:52Z (fix/pre-commit-latency-2): spec drafted: docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast tooling 2 -> 4 children; full route unchanged; limit unchanged)
- 2026-10-07T03:28:41Z (fix/pre-commit-latency-2): filed ops-6cff48 (gap: no per-route latency target)
- 2026-10-07T03:28:41Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review .worktrees/material-5094f2/docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast tooling 2 -> 4 children). After acceptance, agent writes the plan, implements, runs three staged hook commits and tt-latency verify.
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T09:59:42Z (material-3fcba2): halt override: attempted material-41d052 by a7493e94-8545-442f-9657-208ac2609950: owner resumed it in session with the sheet verdict; branch predates the halt
