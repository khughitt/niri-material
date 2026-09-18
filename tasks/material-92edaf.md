---
id: material-92edaf
title: "Within hook: ring and aurora"
status: todo
priority: 2
size: m
complexity: high
process: direct
created: 2026-09-06T00:32:54Z
updated: 2026-09-18T00:17:56Z
depends: [material-a1d4bf, material-f8b6e9]
parent: material-5b3107
tags: [rendering, ring]
spec: docs/specs/2026-09-12-material-render-order-design.md
plan: docs/plans/2026-09-12-material-render-order.md
step: "Task 2: Within hook: ring and aurora"
---

Implement the within hook for ring and aurora as Task 2 of the render-order plan: move interior light ahead of surface terms, remove the ring's bevel mask, shorten the lookup path to 0.2 thickness, scatter the band with roughness (mat_scatter, integral-conserving widening), drop the ring-inset + ring-width <= bevel validation so the ring may sit on the face, refract aurora at its landing point, replace rest face-zero assertions with reach bounds and record motion under the existing drift. Motion policy (one-shot pulse, gating) belongs to material-0e130e, scoped after this lands. Defaults do not change; a stock-inset tuning follow-up is filed if the captures want it.

## Notes

- 2026-09-12T20:26:18Z (material-5b3107): parked (waiting on user, review): Review the render-order plan, then complete Task 1 before starting Task 2.
- 2026-09-12T20:36:44Z (material-5b3107): parked (waiting on agent, dependency): Plan approved; wait for Task 1 implementation and its capture evidence before starting within.
- 2026-09-17T23:48:34Z (material-5b3107): Scoped from the 2026-09-17 amendment: a default-glass capture showed the ring as a plated bezel (hard inner edge from the mask, no normal dependence, untouched by roughness or tint, confined by the inset validation). Spec and plan Task 2 amended with scatter, face placement and motion deferral; process direct once the amendment is reviewed.
- 2026-09-17T23:48:53Z (material-5b3107): parked (waiting on user, review): Review the 2026-09-17 spec and plan amendment for Task 2 (ring scatter, face placement, motion deferral); after approval, wait for Task 1 capture acceptance, then start Task 2 in .worktrees/material-5b3107
  provenance: {"harness_session":"claude-code:01d95010-526a-434c-8bc8-ef0c0d05f772","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-18T00:01:37Z (material-5b3107): Plan corrections applied 2026-09-17: config acceptance test pins offset-x/offset-y 0 (inherited 6 trips the offset check at bevel 4); reach gains --scatter (0.5 for the roughness fixture, else its gate is 37 not 56); mat_scatter needs the prelude declaration and a _1f entry in material_uniform_names(), asserted by the assembly check.
- 2026-09-18T00:04:09Z (material-5b3107): User approved the amended spec/plan and all three review corrections; requested subagent-driven-development. Task 1 acceptance remains the prerequisite; resuming it before Task 2.
- 2026-09-18T00:17:56Z (material-5b3107): parked (waiting on agent, dependency): Amendment and three review corrections approved; wait for material-f8b6e9 candidate capture acceptance and commit, then implement Task 2 via SDD in .worktrees/material-5b3107.
  provenance: {"harness_session":"codex:01a0b1cd-b9d7-71f1-85bb-a5aa09a40df4","harness_session_source":"CODEX_SESSION_ID"}
