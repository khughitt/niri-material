---
id: material-5dbf17
title: "Check a quiet run's inputs when it is parked, not when it runs"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-05T09:10:26Z
updated: 2026-10-05T09:10:26Z
depends: []
parent: material-2834d7
tags: [testing]
agent: claude-code/claude-opus-5-5
---

material-188aaa's quiet park named bin-3125163e, but the smoke driver accepts only a snapshot whose identity source_commit equals the worktree HEAD; the branch had moved 93 commits, so the first quiet attempt refused in 0.26 s and the session spent ~3 min building a snapshot at HEAD on the handed-over host. The driver failed early, but the error was in the park brief. Give the driver (or capture-meta) a check-only mode that runs its input checks (binary identity vs HEAD, sidecar hash, DRM_OUTPUT/DRM_MODE, tools) without capturing, and have a quiet park run it so a brief that cannot run is caught while the agent is still there. Re-check at resume, since commits after the park move HEAD.
