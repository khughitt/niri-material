---
id: material-561927
title: "Test latency over limit: hook-pre-commit-fast 54.226 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: materials-26.04
created: 2026-10-17T14:00:27Z
updated: 2026-10-17T15:16:53Z
started: 2026-10-17T15:16:53Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-17T14:00:24Z"
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-17T14:00:32Z (materials-26.04): breach: titan window 2026-10-10T14:00:24Z..2026-10-17T14:00:24Z: hook-pre-commit-fast median 54.226 s, limit 45 s, 28 runs on 6 days
- 2026-10-17T14:41:50Z (material-62347d): halt override: attempted material-591938 by 4e29fe84-ae01-4701-a4ac-da51f5fe6c77: close-out of the in-progress material-62347d (task records only: file Prism + rollout follow-ups, close the goal); no code, no effect on hook latency
- 2026-10-17T15:16:53Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
