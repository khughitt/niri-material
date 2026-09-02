---
id: material-768b40
title: Cache and regenerate effect-buffer pyramids
status: done
priority: 2
size: m
owner: feat/material-c854bd
created: 2026-09-02T03:14:35Z
updated: 2026-09-02T03:28:00Z
depends: [material-2793ca]
tags: [rendering]
plan: docs/plans/2026-09-01-material-roughness.md
step: "Task 3: Cache and regenerate effect-buffer pyramids"
---

## Notes

- 2026-09-02T03:28:00Z (feat/material-c854bd): Extracted the existing downsample pass and added lazy sharp/blurred EffectBuffer pyramids with exact invalidation and once-per-dirty-period failure suppression.
