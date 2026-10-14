---
id: material-2d0360
title: Explore continuous frosted-glass strength
status: idea
priority: 2
created: 2026-10-12T10:50:30Z
updated: 2026-10-14T10:43:33Z
depends: []
tags: []
source: "mindful:thought:fc0a7db0d7bbae0edfa27a6a310d25d3"
agent: codex
---

Explore making frosted glass approximately continuous rather than binary. During scoping, identify which current binary behavior the capture means and compare existing roughness controls and focus-state material selection before choosing an implementation. Preserve this distinction from focus-swap interpolation (material-5a5fff), whose earlier review kept hard focus cuts.

Established at main d9914514: glass roughness is already numeric (0..1) in niri-config/src/material/mod.rs. PrefilterState::selection in src/render_helpers/effect_buffer.rs blends adjacent levels; roughness is multiplied by clamp(2 * ior - 2, 0, 1), so at IOR 1 it has no softening effect. backdrop-blur is a separate boolean choosing the source. docs/materials/2026-09-02-material-roughness-smoke.md records distinct roughness 0, 0.08 and 0.5 outputs. The original capture says only "make frosted glass ~continuous (now: binary)" and does not identify which switch or situation appears binary.

## Open questions

- Owner: does "binary" mean the static range of frost strengths, a focus/material transition, or disappearance during movement? Supply the affected control or situation. Current recommendation: first check static strength with the existing roughness control; do not assume a request to reverse the accepted hard focus cuts. If movement is the issue, material-e88df7 already tracks frost loss during interactive drag; if focus transitions are intended, explicitly re-scope material-5a5fff with the new observation.

When answered, add a finding with tasks note material-2d0360 --no-reviewed "<answer>" and re-scope against that concrete behavior.
