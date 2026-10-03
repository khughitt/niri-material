---
id: material-6f45a0
title: "Glass response: accent tint of attenuation color"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-10-03T13:04:09Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, rendering]
---

Outcome: an identity response that mixes attenuation-color toward the signal accent by a configurable weight, so glass reads tinted with the project hue at rest. Acceptance: response name in the glass vocabulary, crossfades with the accent animation, bit-identical output at weight 0. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; filed material-3bdffc to design opt-in attenuation tint using existing accent/presence crossfade with neutral-path checks; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
- 2026-10-03T11:20:58Z (material-3bdffc): design finding (material-3bdffc): reviewed spec docs/specs/2026-10-03-accent-tint-design.md (accepted round 3) and plan docs/plans/2026-10-03-accent-tint.md (accepted round 3). Decisions: new response field accent-tint (0..1, default 0, inherited); hue-only tint at preserved face-transmittance luminance (T-space, p_f = thickness/attenuation-distance, gamut pull to gray within [0.001^p_f, 1]); weight = accent-tint x presence; tint chromaticity interpolates between crossfade endpoints (black<->colored continuous); computed in f64 on the CPU in glass_signal_inputs, no shader change; neutral path returns the configured color bitwise; fingerprint gains the quantized attenuation color. Density promise holds on neutral backdrops only; ring-colored band dims 0.47-0.87x at w=1. Native execution through children material-01ddd7..material-76a30f.
- 2026-10-03T13:04:02Z (material-3bdffc): concerns: none — Prism exposure filed as prism-2b9a40 (idea)
- 2026-10-03T13:04:09Z (material-3bdffc): correction: the preceding 'concerns:' line is not a concern record (no closed work fell short); it only records that Prism exposure was filed as prism-2b9a40
