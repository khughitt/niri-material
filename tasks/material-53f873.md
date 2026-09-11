---
id: material-53f873
title: "Dynamics sprint: glass that is never fully static"
status: todo
priority: 1
size: l
created: 2026-09-11T00:47:47Z
updated: 2026-09-11T00:47:47Z
depends: []
tags: [quick-add, dynamics, rendering]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
---

Goal: for the 3d glass to look its best it cannot be 100% static. Inject more movement:

- during drags and window moves: jostle / jitter / slowly translate or rotate / deform the material so light refracts through it differentially
- ease into rest after transitions (less CPU/GPU once settled)
- occasional idle micro-movement: small random dynamics roughly every ~300s, or on events/transitions
- experiment: generative dynamics; several parameterized animations/transforms, sampled and composed with noise

Related: material-6d4de5 (revisit drag/move/focus/flex/ripple; reparented here), material-36e968 (motion-stimulus sweep for measuring it), material-4bf8b8 (frost-on-idle), material-1c5a30 (organic light).

Source: mindful:thought:3f94e656b70f4e5585c1cb60c166e4da
