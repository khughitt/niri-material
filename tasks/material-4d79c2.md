---
id: material-4d79c2
title: Correct capture CPU accounting and malformed record handling
status: done
priority: 2
size: xs
owner: material-bae9c9
created: 2026-09-12T05:10:42Z
updated: 2026-09-12T05:15:37Z
started: 2026-09-12T05:10:46Z
completed: 2026-09-12T05:15:37Z
depends: []
parent: material-bae9c9
tags: [testing]
source: final-review-important-findings
---

## Notes

- 2026-09-12T05:15:37Z (material-bae9c9): Corrected guest CPU accounting and normalized malformed schema-1 consumption for show, settle, and release without mutating records or foreign locks.
- 2026-09-12T05:15:37Z (material-bae9c9): capture-meta counts CPU counters through steal, validates consumed schema-1 shapes, and covers the three malformed CLI paths
