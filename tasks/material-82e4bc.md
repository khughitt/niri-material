---
id: material-82e4bc
title: Establish safe culling and damage boundaries for material tiles
status: doing
priority: 1
size: m
complexity: high
process: direct
owner: materials-26.04
created: 2026-10-06T20:11:17Z
updated: 2026-10-08T14:43:59Z
started: 2026-10-08T14:43:59Z
depends: []
parent: material-5d6b2c
tags: [performance, rendering]
source: docs/notes/2026-09-29-resource-aware-rendering-brief.md
agent: codex
---

Question: Where can hidden material preparation be skipped safely, and what damage and opaque-region facts can the final material element truthfully expose? Treat pre-render culling and final-draw occlusion as separate boundaries.

Where to start: docs/notes/2026-09-29-resource-aware-rendering-brief.md; docs/materials/2026-09-30-hidden-window-attribution-evidence.md (including its 2026-10-06 correction); docs/materials/performance.md sections 1 and 6; src/layout/tile.rs::render, render_inner and tick_deadline; src/layout/scrolling.rs::render; src/layout/floating.rs::render; src/layout/monitor.rs::render_workspaces; src/render_helpers/material/mod.rs::InputFingerprint, advance_commit and Element; src/render_helpers/offscreen.rs; src/niri.rs::update_primary_scanout_output and screencopy paths.

Bound: One source audit and a proposed verification matrix. Trace every caller and render target at these two boundaries, including snapshots, overview/transitions, popups, opening/alpha/resize animations, rounded corners, jelly/slab offsets, fractional scale, opaque versus translucent covers and other lit outputs. Identify where visibility/coverage is available before material.offscreen.render and deadline collection, and whether skipping preparation leaves stale offscreen_data/frame-callback bookkeeping or breaks reveal damage. Identify how client damage history and trustworthy opacity could reach MaterialRenderElement while non-client input changes still force conservative damage. Reuse existing geometry/damage helpers. No renderer patch, new collector, fixture implementation or live/quiet capture in this task.

Expected result: Record a path-backed table of what is safe, conditional or unknown at each boundary, the smallest recommended next change (or why to retain current behavior), and explicit checks that would falsify each claim. Specify the smallest covered-sustained-optic capture and its visible/translucent/idle controls using docs/materials/scripts/hidden-window-attribution.sh; retain unmeasured scheduling, second-output and cost claims as unknown. Outline a pilot through verdict before any matrix and separate preparation from a later quiet capture task. Reuse material-31074f for quantitative cost evaluation; identify whether a reviewed design is needed before implementation. Update this task and the existing brief.

Ideas it wakes: On completion, run tasks note on material-7afc31 and material-7f6d0e with the boundary findings, in the same commit as this result.

## Notes

- 2026-10-08T14:43:59Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:a61b50cf-5bb1-48ac-96f6-2e5b5694ce13","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
