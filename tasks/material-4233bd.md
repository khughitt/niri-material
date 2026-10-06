---
id: material-4233bd
title: "Back out the fork-only single-pixel snapshot change until niri#1463 lands"
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-06T14:25:00Z
updated: 2026-10-06T14:25:01Z
depends: []
tags: [rendering, upstream]
source: material-698875
agent: claude-code/claude-opus-5-5
---

material-698875 baked single-pixel buffers into resize snapshots, but the protocol is #[cfg(test)] in niri and upstream PR #1463 makes the same change to render_snapshot_from_surface_tree. Keep that code identical to upstream instead: revert the src/ and test changes from a8235dff (surface.rs, layout/mod.rs, tests/client.rs, tests/ring_pair.rs), drop the single-pixel row from docs/materials/upstream-divergence.md and regenerate its seam inventory. The task records in that commit stay.

## Notes

- 2026-10-06T14:25:00Z (materials-26.04): concerns: material-698875 change — keep render_snapshot_from_surface_tree identical to upstream rather than carrying our own fix
