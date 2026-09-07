---
id: material-6e7352
title: "Glass noise type: white, fine, and Oklab lightness grain"
status: done
priority: 2
size: m
created: 2026-09-06T00:32:53Z
updated: 2026-09-07T09:49:57Z
depends: []
tags: [rendering, noise]
spec: docs/specs/2026-09-06-material-glass-noise-type-design.md
plan: docs/plans/2026-09-06-material-glass-noise-type.md
---

Native piece of Prism goal prism-d6b600. The glass noise node gains an optional quoted string property (`noise <amount> type="white"|"fine"|"lightness"`). Omitted means white and renders byte-identical to the pinned pre-change build. `fine` is the fragment hash minus the mean of its eight neighbours, scaled by sqrt(8/9) so its standard deviation matches white at the same amount; it is high-pass, bell-shaped, and achromatic in sRGB-encoded space. `lightness` applies the same fine value to Oklab L so chroma movement stays within the measured one-code tolerance except for documented gamut clipping and quantization. One float shader uniform reads `ResolvedGlass.noise_type`; the existing resolved material config and commit-counter inequality carry and damage type changes. GPU-free tests and the six-capture nested GLES matrix pass. Desktop acceptance remains pending and decides whether `lightness` stays.

## Notes

- 2026-09-07T01:07:23Z (glass-noise-type): Scoped 2026-09-06 with Prism: shader-only selector on the noise node, no texture tile, no per-state type
- 2026-09-07T09:49:57Z (glass-noise-type): Implemented quoted white/fine/lightness noise types; GPU-free gates and six-capture GLES evidence pass, with desktop lightness acceptance explicitly pending.
