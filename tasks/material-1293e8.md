---
id: material-1293e8
title: Add optional glass noise and saturation parameters
status: done
priority: 2
size: s
created: 2026-09-05T09:05:28Z
updated: 2026-09-05T11:30:22Z
depends: []
tags: [material, rendering, design]
spec: docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md
plan: docs/plans/2026-09-05-material-glass-noise-saturation-params.md
---

Outcome: glass { noise; saturation } are optional material parameters (noise 0-1, saturation 0-3). A written value renders regardless of backdrop-blur and blur { off }; omission keeps the inheritance from the global blur block that 7c702e58 introduced. Only resolve_material's derivation changes; the shader and damage path already consume the pair. Evidence: config parse and resolve_material unit tests, workspace cargo test, and a nested GLES smoke on the headless Weston unit recorded as docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md. Native half of Prism goal prism-63dd45.

## Notes

- 2026-09-05T11:09:42Z (glass-noise-saturation): glass { noise; saturation } land as optional parameters; written values always apply; nested GLES evidence passes
- 2026-09-05T11:30:22Z (glass-noise-saturation): final review fixes: reachable pre-change pin 56aed303, smoke label and trap tidy-ups
