---
id: material-233295
title: Concise performance guide for the material pipeline
status: todo
priority: 2
size: s
complexity: low
created: 2026-09-11T23:34:15Z
updated: 2026-09-12T19:24:28Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, docs]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Write a short guide to where GPU, CPU, and memory go in the material: per-pass costs in the render chain, prefilter pyramids and their cache reuse, per-window vs per-frame work, focused vs unfocused split, which parameters are expensive and why, and the known levers for reducing cost. Keep it concise and reference the evidence docs under docs/materials/ rather than repeating them.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-12T04:47:03Z (material-bae9c9): When writing the performance guide, point readers to docs/specs/2026-09-11-material-capture-protocol-design.md for comparable capture provenance and environment metadata.
- 2026-09-12T04:53:15Z (material-bae9c9): The future performance guide should reference docs/specs/2026-09-11-material-capture-protocol-design.md for comparable capture provenance and quietness checks.
- 2026-09-12T19:24:28Z (materials-26.04): Complexity low: The deliverable is a concise synthesis of existing render-pipeline and evidence docs, with the capture-protocol reference identified in the notes; verify each cost claim against those sources and current code.
