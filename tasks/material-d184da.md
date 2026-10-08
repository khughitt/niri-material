---
id: material-d184da
title: In-process pixel tests for noise layers
status: done
priority: 2
complexity: mid
process: direct
owner: material-3fcba2
created: 2026-10-06T10:17:17Z
updated: 2026-10-06T11:18:15Z
started: 2026-10-06T11:14:57Z
completed: 2026-10-06T11:18:15Z
depends: []
parent: material-3fcba2
tags: []
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-06-noise-layers.md
step: "Task 3: In-process pixel tests"
---

## Notes

- 2026-10-06T11:14:57Z (material-3fcba2): started
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:16:21Z (material-3fcba2): mutation: independent-corner norm fails the per-position deviation assertion
- 2026-10-06T11:17:30Z (material-3fcba2): mutation: independent-corner norm fails fine scale 4: position (1, 1) has sd; reversed glass, film and backdrop application each fail a_sites_layers_apply_in_slot_order at the respective hook-ordered reference assertion
- 2026-10-06T11:18:15Z (material-3fcba2): done
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-06T11:18:15Z (material-3fcba2): In-process slot identities, per-slot seeding, quadrature, scale normalisation by cell position, backdrop scale damage
  provenance: {"harness_session":"codex:01a110dd-fa22-7dd1-aea1-b7dc927a8f5a","harness_session_source":"CODEX_SESSION_ID"}
