---
id: material-6ddc7e
title: Check narrow lifecycle paths statically
status: done
priority: 0
size: s
complexity: mid
process: direct
owner: fix/pre-commit-latency
created: 2026-10-04T20:39:13Z
updated: 2026-10-04T22:48:08Z
started: 2026-10-04T22:40:15Z
completed: 2026-10-04T22:46:33Z
depends: [material-37e5dc]
parent: material-cd7782
tags: [testing]
agent: codex
plan: docs/plans/2026-10-04-pre-commit-tooling-latency-plan.md
step: "Task 3: Check narrow lifecycle paths statically"
---

## Notes

- 2026-10-04T22:40:15Z (fix/pre-commit-latency): started
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T22:46:33Z (fix/pre-commit-latency): attached: material-task-3-evidence.md (4064 bytes): Deterministic arithmetic guard red/green and static dependency contracts
- 2026-10-04T22:46:33Z (fix/pre-commit-latency): done
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T22:46:33Z (fix/pre-commit-latency): Added one narrow justfile list, recursively sourced shell checks, selected class literal-path checks and transitive tools imports via AST. Arithmetic command substitution is rejected with file/line in entry and helpers; stubbed helper tool fallbacks stay outside the contract. Real static check and 14 fast-mode contracts pass; full 305-case suite passed in 28.464 s with two optional skips.
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T22:48:07Z (fix/pre-commit-latency): detached: material-task-3-evidence.md: Replace machine-specific traceback prefixes with repository-relative paths
- 2026-10-04T22:48:07Z (fix/pre-commit-latency): attached: material-task-3-evidence.md (3710 bytes): Static guard red/green receipts with portable traceback paths
