---
id: material-37cec9
title: Parameter sweep script for the smoke harness
status: todo
priority: 2
size: s
created: 2026-09-06T00:38:33Z
updated: 2026-09-06T09:38:46Z
depends: []
tags: [tooling, harness]
spec: docs/specs/2026-09-06-glass-parameter-sweep-design.md
---

A script under docs/materials/scripts that renders one glass parameter at N values across its range in the headless harness (as glass-noise-saturation-smoke.sh does for fixed values) and reports the perceptual delta between neighboring captures (an AE or Lab RMSE metric, as the noise/saturation evidence already computes). Output is a table of value versus delta so the region where the effect stops changing is visible. Prism's slider range task prism-5758d3 consumes it to choose bounds and scales; refraction (prism-8e8a18) is the first parameter to run.
