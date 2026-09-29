---
id: material-d09741
title: Attribute hidden-window GPU work to clients or material rendering
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-09-29T21:44:30Z
updated: 2026-09-29T21:44:30Z
depends: []
parent: material-5d6b2c
tags: [performance]
source: docs/notes/2026-09-29-resource-aware-rendering-brief.md
agent: codex
---

Question: Do hidden or fully occluded material windows still cause compositor material draws, prefilter rebuilds, redraw scheduling or client GPU work beyond the existing visibility gates?
Where to start: docs/notes/2026-09-29-resource-aware-rendering-brief.md; src/layout/monitor.rs::update_render_elements and workspaces_with_render_geo; src/layout/tile.rs::render/tick_deadline; src/niri.rs::fill_xray_elements/send_frame_callbacks/send_frame_callbacks_on_fallback_timer; src/render_helpers/effect_buffer.rs; docs/materials/scripts/material-signals-smoke.sh.
Bound: Trace these paths and compare one fixed workload visible, on an inactive workspace, in a hidden tab, offscreen and behind an opaque covering window, plus an empty-workspace control. Include a second lit output and an overview/transition control so visible work is not misclassified. Start with one end-to-end capture pilot using tools/capture-meta, then the bounded matrix only when the lane passes. Attribute client work separately from compositor redraws, material draws and shared backdrop/prefilter rebuilds. No visibility algorithm, client suspension policy, always-on collector, exhaustive sweep or host launcher change.
Expected result: Record a per-case attribution table and a recommendation: existing gates suffice, or a reproducible gap with the responsible caller and a focused follow-up. Use existing Tracy draw/redraw zones; document any temporary instrumentation and positive draw controls. Host-wide GPU utilization alone is not evidence of compositor rendering. Update this task and the brief.
Ideas it wakes: On completion, run tasks note on material-7afc31 with the finding, in the same commit as this result.
