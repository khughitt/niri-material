---
id: material-0b6d06
title: Compose inherited postprocess in material rendering
status: done
priority: 2
size: s
owner: feat/material-cad932
created: 2026-09-02T11:20:52Z
updated: 2026-09-02T11:59:19Z
depends: []
tags: [rendering]
plan: docs/plans/2026-09-02-material-noise-saturation.md
step: "Task 1: Compose inherited postprocess in material rendering"
---

## Notes

- 2026-09-02T11:56:03Z (feat/material-cad932): Stable cargo fmt --all --check is red on the untouched foreign_toplevel file because nightly-only formatting options are unavailable; the plan now checks only the three changed Rust files with rustfmt.
- 2026-09-02T11:59:19Z (feat/material-cad932): Inherited global blur noise and saturation now compose in the material shader, opt-out remains neutral, and in-place edits invalidate the material commit.
