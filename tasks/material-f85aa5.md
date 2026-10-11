---
id: material-f85aa5
title: Remove the unused MergeWith import in src/layout/tests.rs
status: done
priority: 4
size: xs
complexity: low
process: direct
owner: material-ffede2
created: 2026-10-08T09:13:39Z
updated: 2026-10-11T15:08:41Z
started: 2026-10-11T15:04:21Z
completed: 2026-10-11T15:08:41Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Every test build warns: unused import MergeWith in src/layout/tests.rs:3 (since c8a72507, material-285f81). Drop it.

## Notes

- 2026-10-11T15:04:21Z (material-ffede2): started
- 2026-10-11T15:08:41Z (material-ffede2): done
- 2026-10-11T15:08:41Z (material-ffede2): Removed the unused MergeWith import from src/layout/tests.rs; layout::tests pass (135 run, 412 skipped); committed through the fast commit route
