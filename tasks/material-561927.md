---
id: material-561927
title: "Test latency over limit: hook-pre-commit-fast 54.226 s against 45 s"
status: todo
priority: 0
size: m
complexity: high
process: planned
created: 2026-10-17T14:00:27Z
updated: 2026-10-17T14:00:32Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-17T14:00:24Z"
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-17T14:00:32Z (materials-26.04): breach: titan window 2026-10-10T14:00:24Z..2026-10-17T14:00:24Z: hook-pre-commit-fast median 54.226 s, limit 45 s, 28 runs on 6 days
