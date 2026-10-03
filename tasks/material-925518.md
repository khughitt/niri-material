---
id: material-925518
title: "capture-meta: an llvmpipe lane that doesn't gate on the NVIDIA GPU"
status: idea
priority: 2
created: 2026-09-23T19:47:41Z
updated: 2026-10-02T23:07:56Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
---

The preflight and settle checks gate on NVIDIA utilisation, P-state and power spread even when the fixture pins Mesa llvmpipe and checks the renderer on every launch, as glass-view-tilt-smoke.sh does. On 2026-09-23 it refused 8 attempts over 60 min on desktop GPU activity (utilisation 12-45% against a 5% limit; P-state not steady at P8) while CPU and load were within limits, so the view-tilt identity and cost runs (material-cd0e1d) could not start. gpu_pstate is a string threshold, so --threshold cannot relax it. Consider a lane, or a renderer-aware mode, that keeps the CPU, load and memory gates, drops the GPU ones once the renderer is verified to be llvmpipe, and records that choice in capture.json. This changes the measurement protocol, so it needs its own review.
