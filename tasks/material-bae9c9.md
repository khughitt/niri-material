---
id: material-bae9c9
title: Consistent performance-capture protocol with provenance and environment metadata
status: todo
priority: 2
size: m
created: 2026-09-11T23:34:15Z
updated: 2026-09-11T23:34:15Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, testing]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Define one recipe for measuring the cost of each rendering element (pass, parameter, preset) so results are comparable across runs. Every capture stores the same metadata: provenance (commit, binary hash, preset/config, driver script) and environment (GPU, driver, clocks/power state, resolution, output count, background load). Requirements: runs need a solo-machine window with no other heavy work, and a CPU/GPU/memory baseline check before and between sub-runs to confirm the machine is idle. material-265eb0's hardware note already asks for an isolated workload; this task supplies the method it should use.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821
