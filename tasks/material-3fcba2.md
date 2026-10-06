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
updated: 2026-10-06T10:42:39Z
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
