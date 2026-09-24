---
id: material-930c55
title: "Source: familiar bridge in familiar/integrations/niri"
status: todo
priority: 2
size: xs
complexity: low
process: direct
created: 2026-09-02T12:09:35Z
updated: 2026-09-24T18:15:39Z
depends: [material-a54d89, fam-e7fa72]
tags: [signals, sources]
---

Cross-repo pointer: the familiar bridge that feeds niri's per-window material signals is implemented in familiar as fam-e7fa72 (familiar-niri watch pushes intent.json, joined to niri-windows.json, through set/pulse/clear-window-signal, one aggregated `familiar` slot per window; mapping from docs/materials/2026-09-02-material-signals-design.md section 2).

Why: with Prism's ring colorSource=familiar, every terminal ring rests on the same manual color because nothing sends set-window-signal (material-068639 made the Prism option say so).

Done when: fam-e7fa72 is done, and on this compositor `niri msg -j windows` shows familiar-sourced accents that differ across terminals hosting different projects, with the ring visibly taking each hue. Then revisit the colorSource disclosure material-068639 added in Prism (prism-b4d118 owns the Ring rows).

Where to look: fam-e7fa72; niri-ipc signal requests; Prism integrations/niri/render.js responseBlock.

## Notes

- 2026-09-22T15:37:52Z (materials-26.04): User-visible as of 2026-09-22: with Prism's ring colorSource set to familiar, every terminal shows the same resting ring color because nothing pushes set-window-signal. Filed as material-068639. This bridge is the fix for per-window hue; 068639 covers the interim honesty of the Prism option.
- 2026-09-24T18:15:39Z (materials-26.04): scope: scoped; implementation filed in familiar as fam-e7fa72 (direct, mid, m), this record now a pointer that closes on a live per-window hue check
