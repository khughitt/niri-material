---
id: material-22d78f
title: Deterministic mid-flex probe for filament face confinement
status: idea
priority: 2
created: 2026-09-05T16:12:46Z
updated: 2026-09-05T16:12:46Z
depends: []
tags: []
---

resize-flex in focus-ring-light.sh compares two nested hosts mid-resize; independent clocks skew 2-5 px so the 2-level face bound cannot be measured. Needs a way to capture both filament-on and filament-off at identical animation progress (clock stepping or a single-host toggle).
