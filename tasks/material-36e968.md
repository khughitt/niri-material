---
id: material-36e968
title: Motion-stimulus sweep for jelly-flex and jelly-ripple
status: todo
priority: 1
size: m
created: 2026-09-06T09:48:24Z
updated: 2026-09-11T00:52:15Z
depends: [material-37cec9]
parent: material-53f873
tags: [tooling, harness, dynamics]
---

material-37cec9's sweep captures a settled window with animations off, where jelly activity is 0 (material.rs:216 derives activity from motion residuals) and the shader gates ripple behind mat_jelly_activity > 0 (material.frag:398). Flex and ripple therefore render identically at every value, so the static sweep excludes them. Measuring them needs a reproducible motion stimulus (scripted resize or move via niri msg), a defined settle delay, and a capture phase pinned relative to the impulse - the timing determinism is its own verification problem. Note prism-5758d3 records that jelly-flex's range is blocked on material-6d4de5 deciding the renderer cap anyway.

## Notes

- 2026-09-11T00:52:15Z (materials-26.04): Promoted to the dynamics sprint's first child (material-53f873): the sweep is the only way to see whether jostle, ease-to-rest, or idle micro-movement changed anything. Do this before the rendering changes so each is judged by measurement, not eye. From mindful:thought:3f94e656b70f4e5585c1cb60c166e4da.
