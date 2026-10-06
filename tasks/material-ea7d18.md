---
id: material-ea7d18
title: "Adopt niri#1463's single-pixel snapshot handling once it merges upstream"
status: todo
priority: 4
size: xs
complexity: low
process: direct
defer: 2026-12-05
created: 2026-10-06T14:25:00Z
updated: 2026-10-06T14:25:02Z
depends: [material-4233bd]
tags: [rendering, upstream]
source: "https://github.com/niri-wm/niri/pull/1463"
agent: claude-code/claude-opus-5-5
---

Check whether niri#1463 (Implement wp_single_pixel_buffer_manager_v1 in render helpers) has merged. If it has, it arrives with the next upstream rebase (docs/materials/upstream-divergence.md, Rebase procedure); confirm the material still renders mid-resize for a single-pixel window then, e.g. by restoring the single_pixel_window_keeps_the_material_mid_resize check from a8235dff. If it is still open or closed unmerged, defer again or drop with the reason.

## Notes

- 2026-10-06T14:25:00Z (materials-26.04): concerns: material-698875 change — upstream's #1463 replaces our backed-out fix
