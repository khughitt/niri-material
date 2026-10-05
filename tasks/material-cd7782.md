---
id: material-cd7782
title: "Test latency over limit: hook-pre-commit 45.305 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: materials-26.04
created: 2026-10-04T15:00:02Z
updated: 2026-10-05T09:36:47Z
started: 2026-10-04T16:48:06Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-04T15:00:01Z"
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-04T15:00:02Z (materials-26.04): breach: titan window 2026-09-27T15:00:01Z..2026-10-04T15:00:01Z: hook-pre-commit median 45.305 s, limit 45 s, 140 runs on 8 days
- 2026-10-04T16:48:06Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a107ab-fa58-7e10-ab82-9b6b7e57845b","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T08:54:13Z (materials-26.04): halt override: attempted material-511efe by 36e424bd-8c56-41e9-a164-005ea47ccb82: doc-only quiet-run protocol fix found by the quiet session; independent of the pre-commit latency incident
- 2026-10-05T09:36:47Z (material-a00785): halt override: attempted material-a00785 by 23285ff0-fcf8-4e52-93eb-5ccb34db3e4c: plan document and task records only, no renderer code this session; independent of the pre-commit latency incident material-cd7782
