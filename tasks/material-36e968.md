---
id: material-36e968
title: Motion-stimulus sweep for jelly-flex and jelly-ripple
status: idea
priority: 2
created: 2026-09-06T09:48:24Z
updated: 2026-09-06T09:49:21Z
depends: [material-37cec9]
tags: [tooling, harness]
---

material-37cec9's sweep captures a settled window with animations off, where jelly activity is 0 (material.rs:216 derives activity from motion residuals) and the shader gates ripple behind mat_jelly_activity > 0 (material.frag:398). Flex and ripple therefore render identically at every value, so the static sweep excludes them. Measuring them needs a reproducible motion stimulus (scripted resize or move via niri msg), a defined settle delay, and a capture phase pinned relative to the impulse - the timing determinism is its own verification problem. Note prism-5758d3 records that jelly-flex's range is blocked on material-6d4de5 deciding the renderer cap anyway.
