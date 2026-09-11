---
id: material-265eb0
title: Idle GPU and power budget for a never-static glass
status: todo
priority: 2
size: m
created: 2026-09-11T00:52:15Z
updated: 2026-09-11T00:52:15Z
depends: []
parent: material-53f873
tags: [quick-add, dynamics, performance]
source: "mindful:thought:3f94e656b70f4e5585c1cb60c166e4da"
---

Idle micro-movement (~every 300s) and eased settling mean the compositor never fully stops redrawing. Measure idle draw rate and power with dynamics on vs. off, and define a settle state in which the shader is genuinely quiescent (no per-frame uniform churn, no redraw without a stimulus). The sprint's ease-to-rest item assumes this; nothing enforces it yet. Related: prism-ed6be0 (intermittent slow material draws), prism-d54be4 (per-parameter GPU cost estimates).

Source: mindful:thought:3f94e656b70f4e5585c1cb60c166e4da
