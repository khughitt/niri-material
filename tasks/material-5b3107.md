---
id: material-5b3107
title: "Render pass order: what the glass slab refracts"
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: material-5b3107
created: 2026-09-06T00:38:34Z
updated: 2026-09-18T12:24:08Z
started: 2026-09-12T19:55:05Z
depends: []
tags: [rendering, design]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
---

Several open questions are one architectural question: in what order the passes run (backdrop blur, glass refraction and bevel, ring of light, noise and saturation postprocess) and therefore what the slab refracts and what lands on top of it. Noise on the glass edges (material-f8b6e9) and the ring embedded inside the glass (material-92edaf) both hinge on it. Document the current order first, then decide the target order once for both.

## Notes

- 2026-09-12T19:24:33Z (materials-26.04): Complexity high: The current pass order is documented in render-pipeline.md, but the target order for edge noise and an embedded ring remains an architectural decision shared by the two open exploratory children.
- 2026-09-12T19:58:07Z (material-5b3107): Drafted depth-ordered hooks for review. Source review found whole-bevel zero grain, a strict two-width ring cutoff, and unchanged rendered brightness are not guaranteed; spec proposes measurable replacements and defines depth from back to front.
- 2026-09-12T19:58:07Z (material-5b3107): parked (waiting on user, review): Review the written spec, especially corrected grain/reach gates and lookup-depth convention; then write the implementation plan.
- 2026-09-12T20:18:34Z (material-5b3107): Spec review accepted for planning, including transmitted chamfer grain and no mask. Preparing one sequential two-step plan using the existing children.
- 2026-09-12T20:26:18Z (material-5b3107): Plan self-review derived 37/61/61 px one-code rest bounds, specified quantization-aware additive comparisons, and corrected the cap-binding threshold claim. Reused both existing children sequentially; no implementation or captures run.
- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review docs/plans/2026-09-12-material-render-order.md; then execute the two existing children sequentially with executing-plans.
- 2026-09-12T20:35:43Z (material-5b3107): Approved plan execution began inline. Task 1 is environment-blocked before shader edits by retained capture preflight render-order-readiness.pCbsOC; offline metrics prepared. Task 2 remains dependent on Task 1.
- 2026-09-12T20:35:43Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 after the capture host is quiet; preserve the required old-build additive failure and later wide-core positive face-strip gate.
- 2026-09-12T21:06:29Z (material-5b3107): Second capture readiness run 03bDro refused; independent process/GPU queries confirm BitwigStudio remains running after its window was closed. Task 1 remains gated before shader changes.
- 2026-09-12T21:06:29Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 when Bitwig has fully exited and the capture host passes default quietness checks.
- 2026-09-12T21:10:25Z (material-5b3107): Bitwig closure verified; after a settling interval CPU, load and power variance pass. Task 1 remains blocked only on GPU utilization/P-state in readiness TFm260.
- 2026-09-12T21:10:25Z (material-5b3107): parked (waiting on user, environment): Resume Task 1 when desktop GPU activity passes the unchanged preflight thresholds; no regression capture has run.
- 2026-09-16T12:09:34Z (material-5b3107): 2026-09-16: visibility/input-idle priorities and one-shot focus task committed separately on main as f4a4302a. Empty-workspace readiness passed, baseline snapshots built, later additive preflight refused on CPU load. Task 1 prepared but not implemented; actual old-build additive failure still required.
- 2026-09-16T12:09:34Z (material-5b3107): parked (waiting on user, quiet; idle, 90 min): Resume material-f8b6e9 in .worktrees/material-5b3107 on a quiet host; its baseline binaries and initial additive fixture are prepared, with the intentional assembly RED test left uncommitted.
- 2026-09-17T02:41:44Z (material-5b3107): Old-build baseline obtained in 064YZ3: measured additive failure and deterministic repeats, using user-authorized pixel-only GPU quietness waiver. Task 1 is no longer waiting for the baseline; production implementation and candidate verification remain.
- 2026-09-17T02:41:44Z (material-5b3107): parked (waiting on agent, session): Continue material-f8b6e9 from completed baseline 064YZ3; implement and verify behind hooks before Task 2. No new permission is needed for the approved implementation.
- 2026-09-17T07:00:16Z (material-5b3107): Task 1 code and docs implemented locally; offline test/check gates pass. Paused as requested before candidate hardware capture. Task 1 remains open/uncommitted pending acceptance; Task 2 untouched.
- 2026-09-17T07:00:16Z (material-5b3107): parked (waiting on user, quiet; idle, 15 min): User prepares next quiet capture window; resume material-f8b6e9 candidate verification in .worktrees/material-5b3107. Offline checks pass, but do not close or commit the child before its capture acceptance.
- 2026-09-17T23:48:34Z (material-5b3107): 2026-09-17: spec and plan amended for Task 2 (ring scatter, face placement, motion deferred to material-0e130e); amendment awaits user review before Task 2 starts. Task 1 capture acceptance still pending the quiet window.
- 2026-09-18T00:04:09Z (material-5b3107): Resumed via subagent-driven-development in existing worktree. Corrected Task 2 amendment approved. Task 1 acceptance delegated before Task 2 implementation.
- 2026-09-18T00:17:56Z (material-5b3107): parked (waiting on user, quiet; idle, 60 min): User cannot idle host this session. Finish Task 1 capture acceptance on an idle host using prepared smoke, then implement approved Task 2 via SDD; progress ledger retained in .worktrees/material-5b3107.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T00:45:48Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T00:56:51Z (material-5b3107): parked (waiting on user, decision): Task 1 candidate binaries built; strict capture settle remains unstable. Await pixel-only GPU waiver decision; timing stays strict and Task 2 stays dependent on Task 1 acceptance.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T01:21:55Z (material-5b3107): resumed
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T02:05:25Z (material-5b3107): Task 1 candidate pixels and formula regressions pass; strict frame-cost remains refused and signal impulse-none gate remains failed/unexplained. No Task 1 commit or Task 2 implementation. Full artifact/status details in Task 1 report and render-order evidence.
- 2026-09-18T02:08:34Z (material-5b3107): parked (waiting on user, quiet; idle, 60 min): Task 1 pixel evidence and code review pass, but strict cost has no trace and signal impulse-none failed. Resume Task 1 from retained artifacts; Task 2 remains dependent and untouched.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T09:27:10Z (material-5b3107): Task 1 pixel and signal acceptance now pass; strict cost still blocked solely on sampled P-state transitions. Four partial timing traces retained; cooldown retry refused at 0% GPU. No completion commit; Task 2 untouched.
- 2026-09-18T09:27:10Z (material-5b3107): parked (waiting on user, quiet; headless, 20 min): Prepare isolated GPU session, then finish material-f8b6e9 strict cost and commit for review before Task 2.
  provenance: {"harness_session":"codex:01a09717-78d8-7ce2-bee2-b5502cbe3649","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-18T12:24:08Z (material-5b3107): Task 1 acceptance and closeout are recorded in the Task 1 commit; Task 2 remains unimplemented and is next only after that commit.
