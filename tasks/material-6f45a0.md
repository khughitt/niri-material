---
id: material-6f45a0
title: "Glass response: accent tint of attenuation color"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T21:32:24Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, rendering]
---

Outcome: an identity response that mixes attenuation-color toward the signal accent by a configurable weight, so glass reads tinted with the project hue at rest. Acceptance: response name in the glass vocabulary, crossfades with the accent animation, bit-identical output at weight 0. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; filed material-3bdffc to design opt-in attenuation tint using existing accent/presence crossfade with neutral-path checks; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
