---
id: material-49d816
title: Pixel-test backdrop grain above scale 1
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-07T03:19:43Z
updated: 2026-10-07T03:19:43Z
depends: []
parent: material-3aa1f2
tags: [testing, noise]
agent: claude-code/claude-opus-5-5
---

Whole-branch review of material-3fcba2 (round 1, Minor): lattice normalisation, position classes and fractional scale are pixel-tested only at the glass site; noise_source at scale > 1 has only a damage test. A wrong backdrop scale wiring in GrainOptions::uniforms or grain.rs would leave backdrop grain at scale 1 with no failing render test. Add an in-process test rendering a backdrop layer at scale 4 and checking its low-frequency ratio rises over scale 1 and its deviation holds.
