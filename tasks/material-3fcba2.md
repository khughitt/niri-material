---
id: material-3fcba2
title: "Glass noise layers: N stacked grain generators with gain, type, and seed scale each"
status: doing
priority: 2
size: m
complexity: mid
process: planned
owner: material-3fcba2
created: 2026-09-09T03:03:30Z
updated: 2026-10-07T10:11:31Z
started: 2026-10-06T09:08:29Z
depends: [material-cf32e5]
parent: material-3aa1f2
tags: [material, noise]
spec: docs/specs/2026-10-06-noise-layers-design.md
plan: docs/plans/2026-10-06-noise-layers.md
---

Prism goal prism-85f63a wants several noise devices stacked on one material. Extend the glass noise node to a small fixed number of layers (four, after the impulse fan-out precedent): packed vec4 uniforms for gain, type, and scale, an unrolled constant-bound loop in material.frag, and a per-layer seed scale or offset so identical types do not coincide. Single-node configs must render byte-identical. Record the cost of stacked fine layers (nine hashes per fragment per layer).

## Notes

- 2026-09-12T19:24:31Z (materials-26.04): Complexity mid: Four layers, packed uniforms, a bounded shader loop, and single-node pixel identity are specified; the existing noise optic localizes the work. Layer decoding, inheritance, seed separation, and lightness composition still require bounded implementation choices and capture verification.
- 2026-10-06T09:08:29Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T09:08:52Z (materials-26.04): Process planned: the body predates the noise site attribute (material-cf32e5) and the pipeline schema prism vendors. Open design choices: KDL shape for repeated noise nodes and their inherit/include merge, how a multi-instance parameter appears in pipeline.json's ParamSpec/stage ownership (prism-85f63a's flat-bus question), layer x site interaction ahead of material-829590, seed separation, and composition order of white/fine vs lightness layers.
- 2026-10-06T09:09:06Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T09:19:24Z (material-3fcba2): parked (waiting on user, review): Owner reviews .worktrees/material-3fcba2/docs/specs/2026-10-06-noise-layers-design.md; on acceptance agent drops material-829590 (folded in), notes prism-85f63a, then writes the plan with writing-plans
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T09:57:12Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T09:57:12Z (material-3fcba2): review: spec round 1 — verdict: revise; findings: P1 1, P2 1; reviewer: codex/gpt-6-astra
- 2026-10-06T09:57:58Z (material-3fcba2): parked (waiting on user, review): Owner reviews the revised .worktrees/material-3fcba2/docs/specs/2026-10-06-noise-layers-design.md (round 1 fixes: covariance norm, per-position check, narrowed collapse claim); on acceptance agent drops material-829590, notes prism-85f63a, writes the plan
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:01:47Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:01:47Z (material-3fcba2): review: spec round 2 — verdict: revise; findings: P2 1; reviewer: codex/gpt-6-astra
- 2026-10-06T10:17:30Z (material-3fcba2): parked (waiting on user, review): Owner reviews .worktrees/material-3fcba2/docs/plans/2026-10-06-noise-layers.md and picks an execution mode; then agent starts Task 1 (material-ae3a26)
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:26:50Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:26:50Z (material-3fcba2): review: plan round 1 — verdict: revise; findings: P1 1, P2 4; reviewer: codex
- 2026-10-06T10:26:50Z (material-3fcba2): Execution mode: native (owner, 2026-10-06), whole-branch review at the end.
- 2026-10-06T10:30:04Z (material-3fcba2): parked (waiting on user, review): Codex runs plan review round 2 on .worktrees/material-3fcba2/docs/plans/2026-10-06-noise-layers.md; on acceptance agent starts Task 1 (material-ae3a26) natively
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:41:38Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:41:38Z (material-3fcba2): review: plan round 2 — verdict: revise; findings: P1 1, P2 1; reviewer: codex
- 2026-10-06T10:42:39Z (material-3fcba2): parked (waiting on user, review): Codex runs plan review round 3 on .worktrees/material-3fcba2/docs/plans/2026-10-06-noise-layers.md; on acceptance agent starts Task 1 (material-ae3a26) natively
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:46:15Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:46:15Z (material-3fcba2): review: plan round 3 — verdict: revise; findings: P1 1, P2 1; reviewer: codex
- 2026-10-06T10:48:30Z (material-3fcba2): parked (waiting on user, review): Codex runs plan review round 4 on .worktrees/material-3fcba2/docs/plans/2026-10-06-noise-layers.md; on acceptance agent starts Task 1 (material-ae3a26) natively
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:59:55Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T10:59:55Z (material-3fcba2): review: plan round 4 — verdict: accept; findings: none; reviewer: codex
- 2026-10-06T10:59:55Z (material-3fcba2): parked (waiting on user, approval): Agent: execute the plan natively from Task 1 (tasks start material-ae3a26 in .worktrees/material-3fcba2); the first mutation demonstration (Task 2 Step 7a) also confirms nextest's real FAIL line format
  provenance: {"harness_session":"claude-code:931ef8b7-a925-41a6-983b-bb8509d7d7d0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T11:01:31Z (material-3fcba2): resumed
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:31:16Z (material-3fcba2): Executed inline: Task 1 65ca684e, Task 2 c066e7e7, Task 3 520c33b1; capture preparation daa55b22. Rust fast suite 504 passed, 1 skipped after Task 3; material docs/config 48 passed after prose edits; full tooling 407 passed, 2 optional skips; all coefficient/norm/order mutations caught. Baseline release built. Captures blocked by measured CPU/load/GPU activity; no sheet/evidence/costs or owner verdict yet. No merge or prism write performed.
- 2026-10-06T11:31:16Z (material-3fcba2): Ruling: checked actual host preflight before baseline release build to avoid unnecessary builds on a refused host; baseline subsequently built while preparing offline scripts; no rendering or capture assertion changed.
- 2026-10-06T11:31:16Z (material-3fcba2): parked (waiting on user, quiet; headless, 39 min): Agent: run material-a4d874's prepared smoke pilot (~6 min) and full (~15 min), write/attach evidence, then material-2f2af4's cost pilot (~8 min) and full (~10 min); baseline already built. Owner then judges contact sheet; agent runs gate, dispatches whole-branch review, integrates and handles prism schema refresh after its external-action approval. Prior headless preflight refused CPU 16.5%, load 4.92, GPU 34% at P5; no cells ran.
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:31:50Z (material-3fcba2): parked (waiting on user, quiet; headless, 39 min): Agent: run material-a4d874's prepared smoke pilot (~6 min) and full (~15 min), write/attach evidence, then material-2f2af4's cost pilot (~8 min) and full (~10 min); baseline already built. Owner then judges contact sheet; agent runs gate, dispatches whole-branch review, integrates, follows prism's repository instructions for the schema refresh, and closes the task. Prior headless preflight refused CPU 16.5%, load 4.92, GPU 34% at P5; no cells ran.
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-07T03:14:21Z (material-3fcba2): attached: 2026-10-06-noise-layers-sheet.png (1280028 bytes): Noise layers contact sheet: grain sizes, stacks, four-at-0.15 against one-at-0.3; 1:1 crops at 2x, controls bordered
- 2026-10-07T03:15:37Z (material-3fcba2): resumed
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:17:43Z (material-3fcba2): just gate passed at a1d0c330 (1.6 min): tooling 409 (2 optional skips), Rust 504 passed 1 ignored, doctests ok
- 2026-10-07T03:19:24Z (material-3fcba2): review: impl round 1 — verdict: accept; findings: Minor 4; reviewer: claude-code/claude-opus-5-5
- 2026-10-07T03:19:49Z (material-3fcba2): Review round 1 dispositions: 1 (spec claim of exact single-layer arithmetic) fixed in spec; 2 (seed coincidence understated) fixed in spec and material-config; 3 rejected: grain_deviation_holds_across_scales_and_cell_positions renders lone layers at scale 2/4/8 and would fail if the fast path ignored scale; 4 filed as material-49d816
- 2026-10-07T03:20:32Z (material-3fcba2): parked (waiting on user, review): Waiting on the owner's look (material-41d052); then agent merges into materials-26.04, refreshes prism's pipeline schema, closes; gate passed and review accepted at 299679cd
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T09:59:47Z (material-3fcba2): owner's look: lattice visible at scale 8 (white reads blocky); no deferral — reshape scale > 1 on this branch per plan Task 6
- 2026-10-07T10:06:58Z (material-3fcba2): review: spec round 3 — verdict: accept; findings: P3 4; reviewer: claude-code/claude-opus-5-5
- 2026-10-07T10:10:01Z (material-3fcba2): lattice reshaped to cubic B-spline over 4x4 points (spec §4 amended, round 3 accept); in-process: 28 noise tests pass, new lattice-visibility test measures 0.55-1.57 of mean (Hermite 0.10 at white scale 2); x2-sum and coefficient mutations caught. Smoke and cost captures must rerun on an idle host.
- 2026-10-07T10:11:20Z (material-3fcba2): attached: 2026-10-07-lattice-reshape-prototype.png (670249 bytes): Offline numpy prototype, not a capture: Hermite lattice (gold, what the owner saw) against the cubic B-spline reshape, white and fine at scale 2/4/8
- 2026-10-07T10:11:31Z (material-3fcba2): parked (waiting on user, quiet; headless, 45 min): Lattice reshaped to B-spline at e5edd631; material-41d052 holds the idle-host smoke and cost reruns, then the owner's second look, gate, review, merge, prism refresh
  provenance: {"harness_session":"claude-code:a7493e94-8545-442f-9657-208ac2609950","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
