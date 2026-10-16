---
id: material-f386ae
title: Record GPU clocks in cost captures to explain the bimodal draw times
status: done
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet]
owner: material-f386ae
created: 2026-10-08T09:13:39Z
updated: 2026-10-16T11:57:19Z
started: 2026-10-11T10:34:37Z
completed: 2026-10-16T11:46:10Z
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
- 2026-10-11T10:34:37Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a12a6b-bf2c-7863-89fb-f5d210c421f0","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-11T10:35:21Z (material-f386ae): resumed
  provenance: {"harness_session":"codex:01a12a6b-bf2c-7863-89fb-f5d210c421f0","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-11T10:37:06Z (material-f386ae): Scope: shared Tracy capture lifecycle owns an opt-in 100 ms NVIDIA clock sampler; noise-layers-cost gains one-case selection and a wall-time bracket around its first reload marker. Keep unsampled captures unchanged because polling may perturb GPU clocks. Tests cover sample/query output, failures and reaping on normal/aborted capture, plus clock-to-draw alignment and missing samples. Live verdict still needs a quiet headless pilot and full case.
- 2026-10-11T10:47:31Z (material-f386ae): review: impl round 1 — verdict: revise; findings: Important 1; reviewer: codex
- 2026-10-11T10:55:42Z (material-f386ae): Review round 1 final detail: Important 1 confirmed by real initial EventStream replay (anchor ended 70 ms before requested completion); Minor 2. Ruling: fix stale initial ConfigLoaded before capture. minor (deferred): invalid CAPTURE_GPU_CLOCKS validation occurs after runtime-directory creation. minor (deferred): independently launched Tracy in optic-settling can inherit a sampling identity claim without calling capture_bg; the documented clock experiment uses noise-layers-cost, which owns the sampler.
- 2026-10-11T10:58:08Z (material-f386ae): review: impl round 2 — verdict: accept; findings: none; reviewer: codex
- 2026-10-11T10:58:08Z (material-f386ae): Ruling: preparation accepted after one corrective round (stale initial ConfigLoaded fixed, regression RED→GREEN); two minor (deferred) findings remain in round 1 detail. Quiet hardware pilot/full and unsampled control own the remaining attribution verdict; task stays doing until that evidence is recorded.
- 2026-10-11T10:59:52Z (material-f386ae): parked (waiting on user, quiet; headless, 25 min): Agent resumes from .worktrees/material-f386ae on a quiet TTY host with the desktop stopped. Check display-dim status (restore stale record), set 0.25. Run sampled one-case pilot: CAPTURE_TASK=material-f386ae CAPTURE_GPU_CLOCKS=1 OUT=$NIRI_MATERIAL_WORK_ROOT/gpu-clocks-pilot-$(date +%s) NOISE_LAYERS_COST_PILOT=1 docs/materials/scripts/noise-layers-cost.sh none (release builds ~8 cold, load wait <=5, preflight/calibration/capture ~3). Read metrics.txt, capture.json, none.gpu-clocks.csv and none.draw-clocks.csv before full. Then sampled full with PILOT=0/fresh OUT (~3), then unsampled full CAPTURE_GPU_CLOCKS=0/fresh OUT (~3) as polling control, one run at a time. Record run: note for every attempt. Agent analyzes early/later draw modes against UTC graphics-clock/pstate samples and retained wall-time bounds (~3), fills Hardware verdict in docs/materials/2026-10-06-noise-layers-evidence.md, recommends later-mode/clock-lock/startup exclusion or names timer-query/prefilter next candidate, restores display-dim, then done/commit/merge. Offline one-case report and checksum rehearsal passed; no live host/preflight attempt yet. No host pointers changed.
  provenance: {"harness_session":"codex:01a12a6b-bf2c-7863-89fb-f5d210c421f0","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-16T11:08:14Z (material-f386ae): resumed
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T11:18:16Z (material-f386ae): run: 9 min (est 13, headless); build 5 (cold), load wait 2, preflight/capture 2; passed. Sampled pilot (none, 3 steps): all 16 draws at P0 1815/7001 MHz, median 0.0410 ms; GPU fell P0 -> P5 390 -> P8 315 -> P8 210 MHz within ~2 s after the burst; max sample gap 104 ms, anchor width 4.7 ms. OUT gpu-clocks-pilot-1792148904.
- 2026-10-16T11:20:27Z (material-f386ae): run: 2 min (est 3, headless); preflight/capture 2; passed. Sampled full (none, 20 steps): 80 draws, median 0.3246; split lines up with clocks: 16 draws at P0 1815 MHz median 0.0410, 6 in the P5/P8 descent 0.27-0.33, 62 at P8 210 MHz median 0.3256 (ratio 7.9). OUT gpu-clocks-full-1792149496.
- 2026-10-16T11:22:34Z (material-f386ae): run: 2 min (est 3, headless); preflight/capture 2; passed. Unsampled control (CAPTURE_GPU_CLOCKS=0): 80 draws, median 0.324608 ms, identical to the sampled run: polling does not perturb. OUT gpu-clocks-control-1792149627. Analysis and evidence doc next.
- 2026-10-16T11:46:10Z (material-f386ae): done
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T11:46:10Z (material-f386ae): Mode split is the GPU clock: sampled full run gpu-clocks-full-1792149496 has 16 draws at P0 1815 MHz (0.0410 ms) then 62 at P8 210 MHz (0.3256 ms, ratio 7.9); unsampled control identical (0.324608). Recommend reporting the later mode selected by clock state, not by count; clock locking rejected. Hardware verdict in the noise-layers evidence doc.
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T11:55:57Z (material-f386ae): review: impl round 1 — verdict: revise; findings: Important 1, Minor 10; reviewer: claude-code
- 2026-10-16T11:57:19Z (material-f386ae): Round 1 fixes: verdict rewritten (draw cost tracks the graphics clock, ~70k cycles/draw across a 17x memory-clock range; P5 tag is sampling resolution); in-window draw counts throughout (pilot 12, P0 12); control claim narrowed to one timer step with descent effect not ruled out; P0 exit described as observed. Minors 4-10 deferred to material-d2e552.
