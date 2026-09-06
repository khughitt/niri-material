---
id: material-6e7352
title: "Glass noise in a perceptual color space (HSV value, CIE lightness or chroma)"
status: idea
priority: 2
created: 2026-09-06T00:32:53Z
updated: 2026-09-06T00:32:53Z
depends: []
tags: [rendering, noise]
---

The current postprocess noise is per-pixel white noise added equally to RGB (postprocess.frag: color.rgb += (hash12(uv) - 0.5) * noise), which reads as grainy and coarse. Explore noise applied in HSV (value-only or hue jitter) or CIE Lab/Lch (lightness-only or chroma-only grain), and possibly a lower spatial frequency or blue-noise pattern so grain is fine rather than coarse. Each type is a shader selector plus a config enum; prism-d6b600 exposes it once the enum exists.
