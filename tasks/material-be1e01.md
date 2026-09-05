---
id: material-be1e01
title: "Ring light: Smoke harness focused cases and captures"
status: doing
priority: 2
size: m
owner: design/ring-light
created: 2026-09-05T12:10:19Z
updated: 2026-09-05T15:59:38Z
depends: [material-6172a1]
parent: material-26dd8a
tags: [material, focus-ring]
plan: docs/plans/2026-09-05-ring-light-focus-response.md
step: "Task 6: Smoke harness focused cases and captures"
---

## Notes

- 2026-09-05T15:59:38Z (design/ring-light): smoke and captures done: focused-drift 15.0/s, reduced 7.5/s, static/anim-off/focus-none/other-focused zero, rest-confinement + accent-midfade + selectors pass; focus-none-toggle 23 and resize-flex face delta 104 miss their bounds for fixture reasons (no-material control costs 20; two instances 2-5 px apart mid-resize); DRM --prepare refused for both configs (source commit not in the fixture allowlist; pinned config makes the fixture worktree dirty), handoffs at NIRI_MATERIAL_WORK_ROOT/ring-light-drm-166cd3e6, awaiting operator VT2 run
