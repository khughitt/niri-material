---
id: material-ef025b
title: Cover vertical and shrinking resizes in the deterministic ring-motion check
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-04T14:46:48Z
updated: 2026-10-04T14:46:55Z
depends: []
parent: material-2834d7
tags: [harness, testing]
source: material-22d78f
agent: codex
---

Extend src/tests/ring_pair.rs::ring_tracks_face_during_resize beyond its horizontal 640 -> 840 widening fixture. Add deterministic vertical widening and horizontal/vertical shrinking cases at a frozen midpoint. Derive each axis's signed face displacement independently from the known resize residual, bevel-depth cap and slabSurface inner-half scaling; use the existing flex-zero ring-gap control for sampled expected centroids. Check both edges on the resizing axis and unchanged perpendicular edges for stock and binding glass with the motion core pinned at gap 8. Verify a disabled vertical resize or reversed resize sign is detected by the check. Reuse the existing fixture and render helpers; no production changes or new capture framework. Keep chamfer/gap-2 motion, corners, fractional scale and moving-beam coverage outside this task. Run the focused test through just test-one, then just test-fast, and update the measurement brief's coverage limits.

## Notes

- 2026-10-04T14:46:55Z (materials-26.04): concerns: material-22d78f extension — cover vertical resize transport and shrinking residual signs using the established frozen-clock motion check.
