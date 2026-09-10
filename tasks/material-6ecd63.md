---
id: material-6ecd63
title: "Parameter table generation, drift test, and the adding-an-optic guide"
status: dropped
priority: 2
size: m
created: 2026-09-10T09:33:23Z
updated: 2026-09-10T10:23:44Z
depends: []
parent: material-397fcb
tags: [material, docs]
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec section 6: ParamSpec per optic plus CORE_PARAMS, render_param_table between markers in material-config.md, the niri-config test that compares and rewrites under MATERIAL_DOCS_UPDATE=1; docs/materials/adding-an-optic.md with noise as the worked example; render-pipeline.md and README.md updates.

## Notes

- 2026-09-10T09:53:35Z (feat/material-397fcb): Review 2026-09-10: ParamSpec is typed (defaults from Resolved::default(), ranges from a Bounded trait on FloatOrInt/Milli/Positive); a second test drives the parser at min, max, and just outside; node strings remain the one hand-maintained fact
- 2026-09-10T10:23:44Z (feat/material-397fcb): superseded by the plan steps under material-397fcb (docs/plans/2026-09-10-material-optics.md)
