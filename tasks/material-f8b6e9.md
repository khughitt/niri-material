---
id: material-f8b6e9
title: "Behind hook: noise and saturation"
status: done
priority: 2
size: m
complexity: high
process: direct
owner: material-5b3107
created: 2026-09-06T00:32:53Z
updated: 2026-09-18T12:24:08Z
started: 2026-09-12T20:31:50Z
completed: 2026-09-18T12:24:08Z
depends: [material-a1d4bf]
parent: material-5b3107
tags: [rendering, noise]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
step: "Task 1: Behind hook: noise and saturation"
---

Implement the reviewed behind hook for noise and saturation as Task 1 of the render-order plan. Preserve signed sRGB formulas, neutral branches, inheritance and opaque bypass; isolate additive-light effects with quantization-aware comparisons. Transmitted chamfer grain is accepted without a mask. Record face/bevel grain and frame cost and update stage/spec docs in the same implementation commit. Plan approved for inline sequential execution; capture readiness and predecessor completion still gate implementation.

## Notes

- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review the two-step render-order implementation plan before starting Task 1.
- 2026-09-12T20:35:43Z (material-5b3107): Execution preflight refused: CPU 16.6% >10%, load1 5.53 >2, GPU 26% >5%, P3/P5/P8, power IQR 7.463 W >1 W, compute client BitwigStudio. Retained render-order-readiness.pCbsOC. This is NOT the old-build additive regression failure. Only offline grain/additive metric preparation is complete; no shader changes or baseline build.
- 2026-09-12T20:35:43Z (material-5b3107): parked (waiting on user, environment): On a quiet host, rerun default capture preflight, snapshot baseline 522a09fe binaries, finish the smoke and retain its actual additive failure against the old shader before production edits. Offline metric CLI and synthetic tests are prepared.
- 2026-09-12T21:06:29Z (material-5b3107): Retry after user closed Bitwig window: preflight render-order-readiness.03bDro still refused (CPU 18.3%, load 7.31, GPU 41%, P5, BitwigStudio compute client). Independent nvidia-smi query confirms live BitwigStudio PID 2467300 using 293 MiB; ps confirms its audio engine PID 2467825. No process terminated; no shader edits or regression capture.
- 2026-09-12T21:06:29Z (material-5b3107): parked (waiting on user, environment): BitwigStudio PID 2467300 and its audio engine remain active. Resume after the app fully exits and default preflight passes; then obtain the old-build additive failure before shader edits.
- 2026-09-12T21:10:25Z (material-5b3107): Confirmed Bitwig exited: no compute clients. Retry ZVFSho refused (CPU 7.4%, load 5.06, GPU 32%, variable P-states/power). After settling, TFm260 passes CPU 6.3%, load 1.91, power IQR 0.537 W; remaining refusals are GPU 23.5% >5% and P5/P8 instead of P8 throughout. Separate pmon sampling observed niri/Noctalia GPU activity. No threshold override or shader changes.
- 2026-09-12T21:10:25Z (material-5b3107): parked (waiting on user, environment): Bitwig is gone; wait for desktop GPU activity to settle below default thresholds (latest TFm260: 23.5%, P5/P8). Then resume baseline and actual old-shader additive capture before implementation.
- 2026-09-16T12:00:24Z (material-5b3107): 2026-09-16 empty-workspace retry passed unchanged preflight: readiness SxYENG, GPU 3%, P8 throughout, CPU 2.3%, load 0.53. Resuming reviewed plan; prior busy result was 37% GPU with a Kitty process observed, not proof of hidden material draws.
- 2026-09-16T12:09:34Z (material-5b3107): Pinned release/Tracy baseline 522a09fe built and hashed in render-order-baseline-522a09fe. New additive smoke MODE=red prepared; 0QXIZU refused on CPU 31.1%, load 21.52 while GPU passed at 5%, P8. Actual additive RED still outstanding. just test intentionally fails only updated assembly ordering (338 passed, 1 failed); just check passes including 128 tooling tests. Uncommitted preparation retained in .worktrees/material-5b3107; no production shader changes.
- 2026-09-16T12:09:34Z (material-5b3107): parked (waiting on user, quiet; idle, 90 min): On an idle host rerun PHASE=behind MODE=red with pinned baseline binaries and a fresh OUT through just test-fast; retain measured additive exit 1 before shader edits, then finish Task 1 matrix and commit for review.
- 2026-09-17T02:31:36Z (material-5b3107): Retry aloE6t with one mapped focused Kitty: CPU 2.0%, load 0.99 now pass; GPU 35.5%, P5/P8 still refuse. Separate five-sample pmon observed niri and Noctalia GPU activity (also Kitty 3% once); this does not isolate a material cause. No capture or shader edit; refusal and process diagnostic retained.
- 2026-09-17T02:31:36Z (material-5b3107): parked (waiting on user, quiet; idle, 90 min): Retry prepared MODE=red additive capture on a quiet desktop using pinned baseline snapshots; latest aloE6t passes CPU/load but GPU remains 35.5% with niri/Noctalia activity. Retain actual measured additive failure before shader edits.
- 2026-09-17T02:41:44Z (material-5b3107): Baseline milestone reached: completed render-order-behind-red.064YZ3 against pinned 522a09fe. All four repeated image pairs identical; actual additive metric exits 1 with 153383 failures / 239988 valid channels, 234141 informative, 12 clipped. User explicitly authorized threshold relaxation; external retained wrapper waives GPU quietness for pixel-only evidence, retaining CPU/load/memory/client/lock checks. No production shader edits or performance claims. Evidence doc and artifact manifest updated.
- 2026-09-17T02:41:44Z (material-5b3107): parked (waiting on agent, session): Baseline captured with real additive RED. Continue approved Task 1: signed transfer helpers, behind hooks and registry order, then candidate acceptance matrix and commit for review. Pixel-only GPU waiver and baseline command are retained in render-order-behind-red.064YZ3.
- 2026-09-17T07:00:16Z (material-5b3107): Behind implementation now in working tree: signed transfer bases, linear-boundary saturation/noise after averaged taps, both registries and params reordered; pipeline/optic/config/spec docs and old formula fixtures updated. Final just test passes (339 niri, 89 config, integration/IPC/docs); just check passes with 128 tooling tests. User requested pause before next solo CPU/GPU period. No candidate GLES capture or timing run; uncommitted until acceptance. Initial aurora additive smoke ready; remaining Task 1 matrix/cost pending.
- 2026-09-17T07:00:16Z (material-5b3107): parked (waiting on user, quiet; idle, 15 min): User prepares quiet CPU/GPU window; then run documented MODE=verify candidate additive smoke against retained baseline, followed by remaining Task 1 matrix and timing before commit. First build/capture pass about 10–15 minutes; no capture started.
- 2026-09-18T00:03:15Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T00:15:42Z (material-5b3107): 2026-09-17 resumed unchanged preflight yREof0 refused: CPU 10.5%, load 7.3, GPU 20%, P3/P5/P8, power IQR 1.937 W, BitwigStudio compute client; no retry/waiver/process change. Offline focused smoke now prepares neutral/grain/additive/dense/signed-transfer/opaque/cost checks; 14 focused tests, bash syntax, tasks check and diff check pass. Candidate GLES, separate legacy formula/signal smokes and cost remain unrun.
- 2026-09-18T00:15:42Z (material-5b3107): parked (waiting on user, quiet; idle, 60 min): On an idle host, rerun the unchanged readiness gate, then execute the prepared PHASE=behind MODE=verify smoke plus separate noise-type, noise/saturation and signal done/error smokes; record candidate GLES/cost evidence before closing or committing Task 1.
- 2026-09-18T00:25:48Z (material-5b3107): Offline review round 1 fixed: dark/near-dark cases now gate signed grain above one code plus non-clipped pixels; saturation 3 gates analytic clamped RGB within one code; each Tracy trace checks shader log before GPU median. Synthetic/stub regressions added; hardware acceptance remains pending.
- 2026-09-18T00:29:57Z (material-5b3107): Offline review round 2 fixed normalized/quantum mismatch: informative grain now compares normalized SD to explicit 1/255; shared ImageMagick quantum-unit one_code helper unchanged. Regression uses real helper and passes after failing pre-fix.
- 2026-09-18T00:31:00Z (material-5b3107): Offline review fixes added informative dark-noise gates, analytic saturation-3 RGB, per-trace log checks and corrected normalized SD threshold. Final controller verification: 16 focused tests pass via just; diff check clean; tasks check no errors. Hardware acceptance/full gates remain pending; no commit or Task 2 edits.
- 2026-09-18T00:45:48Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T00:56:19Z (material-5b3107): Empty-workspace readiness iCx8Zn passed (GPU 3%, P8). First candidate OUT oFYXCN refused its in-run preflight at GPU 8%. One authorized settled retry 1mK7AE passed preflight, built candidate hashes 72ae9ca8/d7485d0d, calibrated geometry, then refused before neutral capture on +2.49 W power drift and P5/P8. No acceptance metric or cost; awaiting user decision on pixel-only waiver with strict cost separation.
- 2026-09-18T00:56:51Z (material-5b3107): parked (waiting on user, decision): Candidates built in render-order-behind-candidate.1mK7AE; neutral-baseline settle refused on +2.49W power drift and P5/P8 after empty-workspace preflight passed. Await user decision on extending baseline GPU waiver to candidate pixels only, with timing kept strict; split pixel/cost execution before any waiver.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T01:07:24Z (material-5b3107): parked (waiting on user, decision): User decides whether to authorize a pixel-only GPU quietness waiver. If approved, add the documented minimal pixels/cost scope split and explicit retained-binary inputs, run pixel scope under the waiver, and keep cost strict.
- 2026-09-18T01:21:55Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T01:21:55Z (material-5b3107): User explicitly approved GPU quietness waiver for candidate pixel checks only; timing remains strict. Resume minimal pixel/cost split, retained binary reuse, pixel matrix and strict cost attempts.
- 2026-09-18T02:05:25Z (material-5b3107): Pixel-only candidate matrix tyHcpH PASS under explicitly approved GPU-only waiver; noise-type rQuR6J, noise/saturation SnjVx7 and signal visual PASS. Strict cost bJ3Ghy passed preflight but refused before first trace on P5/P8, so no timing accepted. Signal cases passed 21 behavioral cases then impulse-none failed 10 redraws vs max 6; trace has expected six pulse-adjacent plus unexplained late cluster of four. Task 1 remains open/uncommitted; Task 2 untouched.
- 2026-09-18T02:08:34Z (material-5b3107): parked (waiting on user, quiet; idle, 60 min): Candidate pixels/formula/visual smokes and full software gates pass; independent code review clear. Resolve signal impulse-none late four redraws (10 vs max6; retained cases trace), then obtain strict baseline/neutral/active cost on stable P8 host. Pixel waiver stays pixel-only. No commit until both acceptance gaps close.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T09:14:48Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a09717-78d8-7ce2-bee2-b5502cbe3649","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T09:27:10Z (material-5b3107): 2026-09-18: strict cost NiPgCC passed preflight and four traces, then refused before active-2 on P5/P8. Signal resume against retained candidate passed impulse-none twice (6 redraws, zero afterward), done-pulse (93 burst/0 after), and slowdown (533 vs534 control). Old late cluster correlates with 90 client surface commits, absent in both new quiet tails; specific client trigger unproven. Fresh 30s-cooldown cost retry 4gBlTh refused at preflight solely on P5/P8 despite 0% median GPU. No thresholds relaxed; strict cost is the only remaining acceptance gap.
- 2026-09-18T09:27:10Z (material-5b3107): parked (waiting on user, quiet; headless, 20 min): Prepare a more isolated desktop/GPU session for strict three-round cost matrix using retained binaries; pixels and signal checks pass. Latest cooldown retry has 0% GPU but P5/P8, so do not repeat ordinary busy-host retries or silently relax timing. Commit only after cost acceptance.
  provenance: {"harness_session":"codex:01a09717-78d8-7ce2-bee2-b5502cbe3649","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T09:37:53Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T09:40:59Z (material-5b3107): parked (waiting on user, quiet; headless, 20 min): Verified newer signal resume artifacts: unchanged assertions pass; only strict cost remains. User-arranged desktop-free GPU interval needed, then retained-binary three-round cost matrix. Existing headless Weston is not host isolation. No desktop shutdown authorized; do not repeat ordinary retries or relax timing.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T11:53:17Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T12:04:56Z (material-5b3107): parked (waiting on user, quiet; headless, 20 min): Bounded30s timestamped cooldown before each render-order cost case implemented and reviewed;19 focused tests pass. Rerun previous TTY retained-binary cost command with fresh OUT during desktop-free interval. Full strict settle unchanged; stop on refusal. No accepted full cost matrix/commit yet.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T12:19:58Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T12:24:08Z (material-5b3107): done
- 2026-09-18T12:24:08Z (material-5b3107): Behind hook implemented; colour-space, additive-light and capture checks recorded
