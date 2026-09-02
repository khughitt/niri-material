---
id: material-743692
title: "Prism: signal tuning parameters"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-02T12:09:35Z
depends: [material-a54d89]
tags: [signals, prism]
---

Outcome: ring-inset, ring-width, oscillator periods, and impulse constants exposed as ordinary glass params in prism defs/glass.yaml and rendered into the response "default" block by the niri sink. Requires the native config to accept those constants first; today the oscillator and envelope constants are fixed in the solver. Source: docs/materials/2026-09-02-material-signals-design.md section 11.
