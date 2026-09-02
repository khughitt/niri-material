---
id: material-cad932
title: Compose global noise and saturation with glass
status: doing
priority: 2
size: s
owner: feat/material-cad932
created: 2026-09-01T14:36:13Z
updated: 2026-09-02T11:07:18Z
depends: []
tags: [migration, design, rendering]
spec: docs/specs/2026-09-02-material-noise-saturation-design.md
---

Outcome: glass with effective backdrop-blur inherits the global blur block's noise and saturation in the existing material pass, applied after optics and before slab coverage/window compositing; opt-out and blur off remain neutral, per-window background-effect overrides stay independent, and Prism gains no duplicate controls. Acceptance evidence: preserve the current-build offset/saturation measurements; unit-test the effective policy and commit advancement; verify full Rust and Prism suites; and capture grayscale saturation, observable noise, untouched opaque pixels, blur-off/opt-out equivalence, and default-off pre-change equivalence on nested GLES. Sources: docs/specs/2026-09-02-material-noise-saturation-design.md, docs/materials/2026-08-29-material-backdrop-blur-design.md, src/render_helpers/background_effect.rs, src/render_helpers/shaders/postprocess.frag, src/render_helpers/material.rs, and src/render_helpers/shaders/material.frag.

## Notes

- 2026-09-02T10:32:14Z (feat/material-cad932): Current v26.04-162-gd9c912d1 nested GL capture disproves the suspected independent blur bugs: at passes 1, offset 1 to 8 changed the probe ROI (normalized RMSE 0.113418; grayscale SD 0.239053 to 0.171272), and saturation 0 produced exact grayscale samples (79/79/79 and 191/191/191).
- 2026-09-02T10:41:36Z (feat/material-cad932): Control surface decision: glass inherits the global blur block's noise and saturation; do not add glass-specific duplicates.
- 2026-09-02T10:42:30Z (feat/material-cad932): Composition order decision: in material.frag, apply saturation then screen-space noise to completed glass color after transmission and specular, before slab coverage and window compositing; opaque window pixels remain untouched.
- 2026-09-02T10:43:40Z (feat/material-cad932): Boundary decision: window-rule background-effect noise/saturation remain independent; glass reads only global blur values, so Prism can keep per-window noise 0 and saturation 1 to suppress the redundant pass.
- 2026-09-02T10:46:17Z (feat/material-cad932): Approach selected: apply inherited global noise/saturation in the material shader; reject cached-texture processing (shared-cache variants and wrong optical order) and a separate background-effect pass (duplicate work and competing composition).
- 2026-09-02T10:49:14Z (feat/material-cad932): Architecture approved: effective backdrop-blur gates neutral versus inherited values; render state carries noise/saturation with commit tracking; material.frag applies them after optics and before coverage/window composite; no EffectBuffer, cache, grammar, or background-effect changes.
- 2026-09-02T10:49:51Z (feat/material-cad932): Prism scope decision: keep ownership unchanged and add no global blur controls; update backdropBlur description/tests to document inherited global blur strength, saturation, and noise while generated KDL remains unchanged.
- 2026-09-02T10:50:37Z (feat/material-cad932): Compatibility approved: default/opt-out unchanged; opted-in glass intentionally gains global defaults; blur off neutralizes all three effects; no new parsing, allocation, failure path, or compatibility layer; Prism KDL remains identical.
- 2026-09-02T11:07:18Z (feat/material-cad932): Review amendments incorporated: named MaterialRenderConfig wrapper and pair-aware in-place commit update; Prism gains an explicit description assertion; the stale backdrop-blur follow-up now records current-build evidence; shader design uses a distinct hash seed and neutral branches for byte identity.
