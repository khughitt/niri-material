---
id: material-a0cbb0
title: Lightweight per-frame render-cost counter exposed over IPC
status: idea
priority: 2
created: 2026-09-11T23:39:09Z
updated: 2026-09-29T21:43:51Z
depends: []
parent: material-5d6b2c
tags: [performance, ipc]
---

A cheap always-on counter (GPU time per material pass, redraw count, idle cadence) exported through the existing IPC/event stream, so cost can be observed continuously rather than only in Tracy sessions. Consumers: ops-71120e (cross-project observation), material-f86183 (verify quiescence), and the adaptive-glass dataset in prism-3e59b5, which could learn cost alongside taste. Keep the counter itself measurably free under the capture protocol.

## Notes

- 2026-09-29T21:43:51Z (materials-26.04): scope: briefed; existing Tracy redraw/material GPU zones cover experiments, while IPC has no cost payload; counter units, consumers and overhead budget remain unresolved; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
