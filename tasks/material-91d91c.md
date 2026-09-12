---
id: material-91d91c
title: The lock
status: done
priority: 2
size: s
owner: material-bae9c9
created: 2026-09-12T02:42:50Z
updated: 2026-09-12T03:51:49Z
started: 2026-09-12T03:46:32Z
completed: 2026-09-12T03:48:36Z
depends: [material-1c21ea]
parent: material-bae9c9
tags: [performance, testing]
plan: docs/plans/2026-09-11-material-capture-protocol.md
step: "Task 3: The lock"
---

## Notes

- 2026-09-12T03:48:36Z (material-bae9c9): Implemented atomic, guarded capture ownership lock with validation and race coverage
- 2026-09-12T03:51:49Z (material-bae9c9): Review fix: invalid UTF-8 lock contents now remain untouched and map to CannotRun/read-acquire or false/release
