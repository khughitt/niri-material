---
id: material-d2b55c
title: Route commits and wire guarded full CI
status: done
priority: 0
size: m
complexity: mid
process: direct
owner: fix/pre-commit-latency
created: 2026-10-04T20:39:13Z
updated: 2026-10-04T23:13:50Z
started: 2026-10-04T23:02:58Z
completed: 2026-10-04T23:13:49Z
depends: [material-e75a92]
parent: material-cd7782
tags: [testing]
agent: codex
plan: docs/plans/2026-10-04-pre-commit-tooling-latency-plan.md
step: "Task 5: Route commits and wire guarded full CI"
---

## Notes

- 2026-10-04T23:02:58Z (fix/pre-commit-latency): started
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T23:13:49Z (fix/pre-commit-latency): attached: material-task-5-evidence.md (988 bytes): Narrow fail-closed routing, exact recipes and real standalone tooling CI
- 2026-10-04T23:13:49Z (fix/pre-commit-latency): done
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T23:13:49Z (fix/pre-commit-latency): Narrow full-first routing and explicit fast/full/static commands are wired with shared checks. Both code hooks record hook-pre-commit; gate/other pushes run full tooling once, Rust CI stays Rust-only, docs check paths without discovery. Twelve gate controls, actionlint and affected Rust front door pass; full host 313/two skips and actual clean Ubuntu ci-tooling-test without ops 313/five skips are green. Stability and latency acceptance remain Task 6.
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
