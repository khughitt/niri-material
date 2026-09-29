---
id: material-5d854f
title: "Glass response: progress fill on the ring"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T21:32:24Z
depends: [material-a54d89]
parent: material-0a4093
tags: [signals, rendering, design]
---

Outcome: the ring fills proportionally to a progress channel. Requires a continuous progress value in the signal model, which the v1 slice excludes; design that extension first (a per-slot progress: Option<f32> is the likely shape). Acceptance: design note, then response name in the glass vocabulary. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; confirmed job progress is absent from slots and IPC; source folding/expiry and current ring geometry require design before rendering; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
