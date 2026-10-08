---
id: material-03fdf3
title: Recover the B-spline grain's idle cost on every material draw
status: todo
priority: 2
size: s
complexity: high
process: direct
created: 2026-10-08T04:20:53Z
updated: 2026-10-08T04:26:21Z
depends: [material-3fcba2]
parent: material-5d6b2c
tags: [performance, noise]
agent: claude-code/claude-opus-5-5
---

e5edd631's B-spline noise.frag costs every material draw ~0.034 ms (+15%) with noise off: same-session A/B against 558bfa02's kept niri-tracy (noise-layers-ab-full-1791429717: none 0.263 vs 0.229 ms). A rebuild of the Hermite shader matches the kept binary, so it is the shader, not build drift. Streaming the lattice without arrays: no change. A slot loop (2 inlined copies instead of 8): idle 0.249 but one active layer 0.360 vs 0.340 unrolled. Likely the scaled path inlined 8x into the material program raising its register allocation; read the NVIDIA register count/ISA (or test a version that keeps the scaled path out of the unrolled call sites) before more blind A/B runs. Measure with docs/materials/scripts/noise-layers-cost.sh NOISE_LAYERS_COST_AB=<kept niri-tracy> (~10 min, headless, quiet). Evidence: docs/materials/2026-10-06-noise-layers-evidence.md, Cost.

## Notes

- 2026-10-08T04:26:19Z (material-3fcba2): review: impl round 2 note — experiment variants attached (streamed lattice, slot loop) so they can be rerun
- 2026-10-08T04:26:19Z (material-3fcba2): attached: noise-frag-streamed.diff (4143 bytes): A/B variant 'scaled path without arrays' (noise-layers-ab-stream-1791430389): patch against e5edd631's noise.frag; no idle-cost change
- 2026-10-08T04:26:19Z (material-3fcba2): attached: noise-frag-streamed-slot-loop.diff (7116 bytes): A/B variant 'slot loop' (noise-layers-ab-loop-1791432546): patch against e5edd631's noise.frag (streamed lattice plus glass/film layers in a loop); idle 0.249 ms, one active layer 0.360 ms
