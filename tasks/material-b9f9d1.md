---
id: material-b9f9d1
title: Isolate the pre-push gate from Git-local environment
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: fix/pre-push-rollout
created: 2026-10-02T01:33:01Z
updated: 2026-10-02T01:46:11Z
started: 2026-10-02T01:33:01Z
completed: 2026-10-02T01:46:11Z
depends: []
tags: []
source: ops-e6ec4e
agent: codex
---

Adopt the landed ops pre-push isolation block before the existing gate. Preserve keepalive, gate selection, custom steps, exact ref stdin and exit codes; Git LFS runs before isolation with its original environment and refs. Verify the exact hook with the shared behavioral regression, then project test-fast and required checks. Record the adoption commit on ops-e6ec4e.

## Notes

- 2026-10-02T01:33:01Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a0fa3a-dc49-7fe2-ab41-ffb86990bda9","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-02T01:36:14Z (fix/pre-push-rollout): resumed
  provenance: {"harness_session":"codex:01a0fa3a-dc49-7fe2-ab41-ffb86990bda9","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-02T01:42:46Z (fix/pre-push-rollout): review: impl round 1 — verdict: accept; findings: none; reviewer: codex/GPT-6
- 2026-10-02T01:46:10Z (fix/pre-push-rollout): Focused tooling checks: 6 passed. Python tooling test-fast/check: 176 tests passed, 2 skipped; full just check including Rust formatting and clippy passed. Hydrated known per-machine cargo config/target in the worktree.
- 2026-10-02T01:46:11Z (fix/pre-push-rollout): done
  provenance: {"harness_session":"codex:01a0fa3a-dc49-7fe2-ab41-ffb86990bda9","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-02T01:46:11Z (fix/pre-push-rollout): Adopted pre-push Git-local environment isolation; preserved existing gates, keepalive, refs and exit status; shared behavioral regression and project checks pass.
  provenance: {"harness_session":"codex:01a0fa3a-dc49-7fe2-ab41-ffb86990bda9","harness_session_source":"CODEX_SESSION_ID"}
