---
id: material-9306b5
title: Tune stock ring inset against the refracted within ring
status: idea
priority: 2
created: 2026-09-18T23:33:13Z
updated: 2026-09-19T00:56:02Z
depends: [material-92edaf]
parent: material-a76720
tags: [rendering]
source: "2026-09-12-material-render-order:stock-inset-captures"
agent: codex
---

The render-order acceptance captures support revisiting stock placement: within-pinned-on.png (ring-inset 5, bevel 12) reads as a luminous rim with an inward halo, not a clearly separated embedded band. within-face-on.png (inset 20) demonstrates available face placement but is an acceptance extreme, not a proposed default. Compare retained images under $NIRI_MATERIAL_WORK_ROOT/render-order-within-pixels.eS4KzB/run and scope a small visual tuning comparison after material-92edaf. Defaults remain unchanged in render-order work; choose the final inset with user visual judgment, not from reach bounds. Motion remains separately deferred to material-0e130e.
