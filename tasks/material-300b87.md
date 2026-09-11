---
id: material-300b87
title: Accept aurora and iridescence on the installed hardware renderer
status: todo
priority: 2
size: m
created: 2026-09-11T08:00:32Z
updated: 2026-09-11T08:00:32Z
depends: [material-f0fc7b]
tags: [rendering, performance]
---

Native plan evidence used llvmpipe. On the restarted installed niri 7526af1d or newer, capture Aurora and Rainbow with CPU metadata and GPU/driver identity; check edge-weighted iridescence and face colors, aurora pinned and stepped behavior, both focus states, reduced motion and off. Measure warmed median material-pass overhead and idle redraw/power against plain glass on the hardware renderer. Compare the plan acceptance budgets, record repeatable commands and captures under docs/materials/, and file concrete regressions if budgets fail. Prism controls and profiles are landing via prism-763054 and prism-08c1de. Coordinate broader idle/power budgets with material-265eb0 and cost reporting with prism-d54be4; do not duplicate their general scope.
