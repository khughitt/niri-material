---
id: material-03fdf3
title: Recover the B-spline grain's idle cost on every material draw
status: doing
priority: 2
size: s
complexity: high
process: direct
owner: noise-followups
created: 2026-10-08T04:20:53Z
updated: 2026-10-08T10:21:13Z
started: 2026-10-08T09:47:10Z
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
- 2026-10-08T09:47:10Z (noise-followups): started
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T10:19:58Z (noise-followups): Cause found statically: NVIDIA's shader disk cache holds the SASS (zstd frame, NVuc container; /opt/cuda/bin/nvdisasm -b SM86 reads its REG section). Material fragment program on HEAD (RTX 3070, 615.71.09): Hermite 96 regs / 13192 instr; B-spline (e5edd631 noise.frag) 128 / 16864; streamed 96 / 17032; slot loop 96 / 10552. On e5edd631's own source: Hermite 64, B-spline 96, streamed 96, loop 96 (10104 instr) - matching the A/B timings (0.229 / 0.263 / 0.265 / 0.249): registers set most of the idle cost, code size the rest. No spills anywhere.
- 2026-10-08T10:19:58Z (noise-followups): Candidate committed (127bce2d): noiseValue folds each of the six hash rows into one vec4 accumulator with per-row B-spline weights (noiseRow), one code path for white and fine. HEAD: 72 regs / 15424 instr (below Hermite's 96). Also measured 'unified' (rolling-window stream, white and fine shared): 96 / 12512, bit-identical to e5edd631. Candidate pixels vs current B-spline over 8 scaled cases (glass, film, backdrop, white/fine/lightness, a 4-layer stack): max 1 code on 4-67 of 5.2M bytes per case. Trade: a scaled white layer hashes 36 points instead of 16. noise_ tests 26/26, test-fast 534/534.
- 2026-10-08T10:19:58Z (noise-followups): Rehearsal (desktop up, CAPTURE_META=/bin/true, pilot, AB_CASES=none) of the multi-binary A/B passed end to end; medians not evidence (early-mode draws, unsettled host) but ordered as the register counts predict: candidate 0.036, Hermite 0.037, unified 0.038, B-spline 0.041 ms. Comparison binaries in $NIRI_MATERIAL_WORK_ROOT/noise-idle-variants (each niri-tracy built at d89a8988's tree with that noise-<v>.frag): bspline d6f6639d2ac3e1a2579d1978c165e57dd2cc7c4bdb7c1662895098c2a931e93c, hermite baf41bc6d31d07e3d70814d219feea78ce3aa4e3cc03ca68ed3d95a151e3ab91, unified 661ffc352b2dd868a4e3fa1f47d8b2267e9c3cc472ab3f1d1677014a6f7fa4ac.
- 2026-10-08T10:19:58Z (noise-followups): attached: regs.sh (1863 bytes): Register/instruction count of the material program for a given noise.frag: swaps it into a base source, compiles through the ignored probe test with NVIDIA's disk cache on, decompresses the SASS and reads it with nvdisasm. Paths inside are this session's; set P/WT before reuse.
- 2026-10-08T10:19:58Z (noise-followups): attached: program_probe.rs (1569 bytes): The ignored probe test regs.sh drives (src/tests/program_probe.rs + 'mod program_probe;' in src/tests/mod.rs): compiles PROBE_SRC (or material_source()) headlessly and writes glGetProgramBinary to PROBE_OUT.
- 2026-10-08T10:19:58Z (noise-followups): attached: swap.py (240 bytes): regs.sh helper: replaces the noise optic section of a material source.
- 2026-10-08T10:21:12Z (noise-followups): detached: regs.sh: replaced by a version without host paths
- 2026-10-08T10:21:12Z (noise-followups): attached: regs.sh (2084 bytes): Register/instruction count of the material program for a given noise.frag: swaps it into BASE, compiles through the ignored probe test (program_probe.rs) in WT with NVIDIA's disk cache on, decompresses the SASS and reads it with nvdisasm.
