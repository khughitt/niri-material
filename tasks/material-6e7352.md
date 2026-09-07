---
id: material-6e7352
title: "Glass noise type: white, fine, and Oklab lightness grain"
status: todo
priority: 2
size: m
created: 2026-09-06T00:32:53Z
updated: 2026-09-07T01:07:23Z
depends: []
tags: [rendering, noise]
spec: docs/specs/2026-09-06-material-glass-noise-type-design.md
---

Native piece of Prism goal prism-d6b600. The glass noise node gains an optional type property (noise <amount> type=white|fine|lightness). Omitted means white and renders byte-identical to today. fine is the fragment's hash minus the mean of its eight neighbours' hashes, scaled by sqrt(8/9) so its standard deviation matches white at the same amount: high-pass and bell-shaped, still achromatic in sRGB-encoded space. lightness applies the fine value to Oklab L so chroma and hue hold. One integer uniform, a Noise config struct mirroring Distortion, noise_type on ResolvedGlass and MaterialRenderConfig, commit-counter damage on a type change. GPU-free tests plus a per-type nested GLES matrix (variance, low-frequency ratio, tail size, Oklab chroma invariance, omitted-equals-white determinism), then desktop acceptance that decides whether lightness stays.

## Notes

- 2026-09-07T01:07:23Z (glass-noise-type): Scoped 2026-09-06 with Prism: shader-only selector on the noise node, no texture tile, no per-state type
