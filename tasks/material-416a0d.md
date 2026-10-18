---
id: material-416a0d
title: "Balance fast tooling buckets by measured case time, not case count"
status: todo
priority: 2
created: 2026-10-17T15:27:43Z
updated: 2026-10-18T22:48:03Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Fast tooling packs whole modules into 4 buckets by case count. On 2026-10-17 (529 cases, global hook excluded) the buckets finished at 5.5/19.5/18.7/5.0 s of case time; balancing the same modules by time gives ~12.3 s each, ~7 s off the fast pre-commit route. Needs a source of per-module durations (last run's timings, a committed weights file) and a staleness rule. Data and context: material-561927, docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md (Rejected alternatives).
