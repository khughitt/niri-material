---
id: material-cad932
title: Decide how noise and saturation compose with glass
status: idea
priority: 2
size: s
created: 2026-09-01T14:36:13Z
updated: 2026-09-01T14:36:13Z
depends: []
tags: [migration, design, rendering]
---

Outcome: settle whether and where global blur noise and saturation should apply to glass transmission, with enough evidence to scope implementation or reject it. Acceptance evidence: confirm on a current build whether blur.offset at one pass is observable and whether saturation 0 reaches grayscale; trace the existing background-effect postprocess and material shader paths; record the chosen control surface, compositing order, defaults, and compatibility impact before code changes. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md, src/render_helpers/background_effect.rs, and src/render_helpers/shaders/postprocess.frag. Uncertainty: the follow-up is real, but current evidence does not establish whether the observations are material integration gaps, independent blur bugs, or intended behavior.
