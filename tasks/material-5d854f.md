---
id: material-5d854f
title: "Glass response: progress fill on the ring"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-02T12:09:35Z
depends: [material-a54d89]
tags: [signals, rendering, design]
---

Outcome: the ring fills proportionally to a progress channel. Requires a continuous progress value in the signal model, which the v1 slice excludes; design that extension first (a per-slot progress: Option<f32> is the likely shape). Acceptance: design note, then response name in the glass vocabulary. Source: docs/materials/2026-09-02-material-signals-design.md section 11.
