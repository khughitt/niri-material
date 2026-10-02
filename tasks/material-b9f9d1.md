---
id: material-b9f9d1
title: Isolate the pre-push gate from Git-local environment
status: doing
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-02T01:33:01Z
updated: 2026-10-02T01:33:02Z
started: 2026-10-02T01:33:01Z
depends: []
tags: []
source: ops-e6ec4e
agent: codex
---

Adopt the landed ops pre-push isolation block before the existing gate. Preserve keepalive, gate selection, custom steps, exact ref stdin and exit codes; Git LFS runs before isolation with its original environment and refs. Verify the exact hook with the shared behavioral regression, then project test-fast and required checks. Record the adoption commit on ops-e6ec4e.

## Notes

- 2026-10-02T01:33:01Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a0fa3a-dc49-7fe2-ab41-ffb86990bda9","harness_session_source":"CODEX_SESSION_ID"}
