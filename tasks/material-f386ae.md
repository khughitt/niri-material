---
id: material-f386ae
title: Record GPU clocks in cost captures to explain the bimodal draw times
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-08T09:13:39Z
updated: 2026-10-10T13:46:21Z
depends: []
parent: material-2834d7
tags: [performance, harness, capture]
agent: claude-code/claude-opus-5-5
---

Why: every cost capture's material draws split into two modes: the first 11–16 draws run at about 1/8 of the later ones, a constant ratio across cases (docs/materials/2026-10-06-noise-layers-evidence.md, Cost; the lattice rerun in 1d9e8ddb shows the same split, 16 draws at 0.140 ms against 68 at 1.188 ms). A GPU clock change between the start-up burst and the 1 Hz damage cadence fits the ratio but was never measured, so it is unknown whether the reported medians (the later mode) measure the shader or the idle clock. This also bears on material-3d48b0: its desktop-idle versus TTY interval check compares the same noise-layers-cost medians, and a desktop that holds the GPU at a different clock would move them for reasons that have nothing to do with settles.

What: sample the GPU at 10 Hz or faster through the whole Tracy capture of one noise-layers-cost case (`nvidia-smi --query-gpu=timestamp,clocks.gr,clocks.mem,pstate,power.draw --format=csv -lms 100` beside tracy-capture, started and reaped by the capture function in docs/materials/scripts/glass-optic-smoke-lib.sh, ~line 489), with a wall-clock anchor for the first material draw. capture-meta's GpuReader (tools/capture-meta, QUERY) already reads clocks.gr and pstate, but only during preflight and settles, never while the capture runs.

Done when: the evidence doc's Cost section states whether the mode split lines up with a clocks.gr/pstate transition, with the trace kept beside the run, and recommends one of: report the later mode as is, lock clocks for cost captures, or drop the early draws. If the split does not line up with any clock change, record that and name the next candidate (timer-query start-up, prefilter warm-up). Pilot one case through the full capture first; with the sampler wired in, the full run can share material-3d48b0's quiet session.

## Notes

- 2026-10-10T13:46:18Z (materials-26.04): scope: scoped; set todo P2 s/mid/direct, need quiet, parent material-2834d7; body names the sampler, where it hooks in, the done check, and the bearing on material-3d48b0's interval comparison
