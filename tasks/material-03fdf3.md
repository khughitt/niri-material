---
id: material-03fdf3
title: Recover the B-spline grain's idle cost on every material draw
status: done
priority: 2
size: s
complexity: high
process: direct
owner: noise-followups
created: 2026-10-08T04:20:53Z
updated: 2026-10-16T11:44:54Z
started: 2026-10-08T09:47:10Z
completed: 2026-10-16T11:44:54Z
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
- 2026-10-08T10:21:54Z (noise-followups): parked (waiting on user, quiet; headless, 40 min): Quiet A/B (headless lane, desktop stopped) from .worktrees/noise-followups, W=$NIRI_MATERIAL_WORK_ROOT/noise-idle-variants, AB=$W/niri-tracy-bspline:$W/niri-tracy-hermite:$W/niri-tracy-unified. Pilot: CAPTURE_TASK=material-03fdf3 OUT=$NIRI_MATERIAL_WORK_ROOT/noise-idle-ab-pilot-$(date +%s) NOISE_LAYERS_COST_PILOT=1 NOISE_LAYERS_COST_AB=$AB NOISE_LAYERS_COST_AB_CASES=none docs/materials/scripts/noise-layers-cost.sh (build 5, 8 sub-runs 4). Then full: same with PILOT=0, OUT=...noise-idle-ab-full-..., AB_CASES='none one-fine-1 four-fine-8' (24 sub-runs, ~30). a = candidate (127bce2d, 72 regs), b1 = current B-spline, b2 = Hermite on HEAD, b3 = unified (96 regs). Then: land a if it beats b1 on none and is not dearer on one-fine-1/four-fine-8 (else unified if it is), write the Cost entry in docs/materials/2026-10-06-noise-layers-evidence.md, done, merge noise-followups. Rehearsal passed with CAPTURE_META stubbed; no real-preflight attempt yet.
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T10:42:54Z (noise-followups): resumed
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T10:53:53Z (noise-followups): run: 10 min (est 9, headless); build 3, load wait 1, preflight 1, sub-runs 9 (geometry + none a/b1/b2/b3 x2) 5; passed (clean release, settles 10.36-10.53 W, display on). Pilot none medians a 0.036352/0.035328, b1 0.040960/0.040960, b2 0.036864/0.037888, b3 0.037888/0.037888 ms. OUT noise-idle-ab-pilot-1792147386. Full next.
- 2026-10-16T11:01:27Z (noise-followups): run: 4 min (est 30, headless); build 0.5, load wait 0, preflight 1, sub-runs 4 (geometry, none a/b1/b2) 2; refused: settle before none-b3-1, gpu_power_w 17.115 vs baseline 10.39. Idle GPU floor stepped 10.4 -> 17.0 W (P8, 210/405 MHz, no clients, fan held 45% by gpu-fan-curve, which cannot reach nvidia-settings without a Wayland session) and stayed there; same 17 W floor as the 2026-10-07 runs. Completed none medians a 0.2877, b1 0.3246, b2 0.2918 ms (8x the pilot's 0.036: draw mode). OUT noise-idle-ab-full-1792148041. Rerunning with the floor stable at 17 W.
- 2026-10-16T11:08:33Z (noise-followups): run: 2 min (est 30, headless); build 0.5, preflight 1, sub-runs 2 (geometry, none-a-1) 1; refused: settle before none-b1-1, gpu_power_w 10.39 below baseline 17.125. The idle floor flips between ~10.4 and ~17 W after nested sessions (a 3-min passive sample with no load held 9.7-10.4 W), so a 24-sub-run A/B is unlikely to see one floor throughout; none-a-1 0.2877 ms. OUT noise-idle-ab-full-1792148490. Next: material-f386ae's sampled clocks, then decide.
- 2026-10-16T11:44:03Z (noise-followups): run: 21 min (est 30, headless); build 0.5, preflight 1, sub-runs 25 19; passed (clean release). none a 0.2867/0.2867, b1 0.3236/0.3246, b2 0.2908/0.2908, b3 0.2918/0.2918; one-fine-1 a 0.3589/0.3594, b1 0.3988/0.3983, b2 0.3676/0.3676, b3 0.3676/0.3676; four-fine-8 a 1.1448/1.1448, b1 1.2513/1.2513, b2 0.7342/0.7347, b3 1.3332/1.3332 ms. a beats b1 on none by 0.037 ms (11.5%) and is cheaper on one-fine-1 and four-fine-8 -> land a (127bce2d). OUT noise-idle-ab-full-1792149757.
- 2026-10-16T11:44:54Z (noise-followups): done
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T11:44:54Z (noise-followups): Row accumulator 127bce2d lands: same-session A/B noise-idle-ab-full-1792149757 none 0.2867 vs B-spline 0.3241 ms (-11.5%), one-fine-1 0.3592 vs 0.3986, four-fine-8 1.1448 vs 1.2513; Hermite 0.2908/0.3676/0.7345, unified 0.2918/0.3676/1.3332. Cost entry in the noise-layers evidence doc.
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
