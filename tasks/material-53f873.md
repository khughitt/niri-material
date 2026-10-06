---
id: material-53f873
title: Responsive glass that settles completely
status: todo
priority: 1
size: l
complexity: high
lane: true
created: 2026-09-11T00:47:47Z
updated: 2026-10-06T22:10:56Z
depends: []
tags: [quick-add, dynamics, rendering]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
---

Make glass respond to movement and focus, then settle completely under the accepted settle contract. Next milestone: the reviewed design of a transient tilt with side walls (material-abc08c), built on material-be611b's height-field edge once its sheet is reviewed. The planar-tilt stills (material-89fb6b) were judged on 2026-10-06: one object, side walls first.

Goal: for the 3d glass to look its best it cannot be 100% static. Inject more movement:

- during drags and window moves: jostle / jitter / slowly translate or rotate / deform the material so light refracts through it differentially
- ease into rest after transitions (less CPU/GPU once settled)
- occasional idle micro-movement: small random dynamics roughly every ~300s, or on events/transitions
- experiment: generative dynamics; several parameterized animations/transforms, sampled and composed with noise

Related: material-6d4de5 (revisit drag/move/focus/flex/ripple; reparented here), material-36e968 (motion-stimulus sweep for measuring it), material-4bf8b8 (frost-on-idle), material-1c5a30 (organic light).

Source: mindful:thought:3f94e656b70f4e5585c1cb60c166e4da

## Notes

- 2026-09-12T19:24:28Z (materials-26.04): Complexity high: Dynamics behavior still needs design across interactive drag, settling, occasional motion, and optical response; the completed motion sweep does not settle those choices.
- 2026-10-02T23:08:32Z (materials-26.04): workstreams: now a lane (tag lane); scope unchanged; brief: docs/notes/2026-10-02-workstreams-brief.md
- 2026-10-02T23:08:32Z (materials-26.04): workstreams: adopted lighting ideas material-1c5a30 and material-f3e4e4 (time-varying light); material-77db8a joins when its claim ends
- 2026-10-02T23:47:23Z (materials-26.04): workstreams: retitled from 'Dynamics sprint: glass that is never fully static' to align with the accepted settle contract (material-f86183, material-0db905). First milestone: material-77db8a brought current with materials-26.04 and verified.
