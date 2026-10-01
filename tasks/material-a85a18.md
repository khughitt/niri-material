---
id: material-a85a18
title: Revisit the filament shift cap versus light-ior for dense glass
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: material-a85a18
created: 2026-09-05T17:14:14Z
updated: 2026-10-01T10:25:47Z
started: 2026-10-01T10:02:43Z
depends: []
parent: material-49871a
tags: [rendering]
spec: docs/specs/2026-10-01-filament-shift-cap-design.md
---

The render-order change shortened the filament's remaining path from 0.6 to
0.2 of thickness. The half-`ring-inset` cap remains a high-shift safety limit,
but no longer saturates stock glass or Prism's `ior 1.24` at every
`light-ior` value: at `light-ior 1` those examples are roughly 1.16 px and
1.54 px, respectively, against the default 2.5 px cap. Revisit the cap only
where a dense or high-`light-ior` setting actually reaches it. Decide whether
the cap, a bevel-relative depth, or an auto light-ior is the intended model,
with captures at ior 1.02, 1.24, and 1.5. Take one capture at ior 1.24 /
bevel 9 once the deterministic flex probe (material-22d78f) exists so the
owner's own glass has evidence.

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; current shader caps shared shift at half ring-gap with a 0.2-thickness path; old numeric examples need recomputation before any cap redesign; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-01T09:55:04Z (material-0e80c1): material-0e80c1 result: formula table rederived in the brief. The cap binds at Prism active 1.28/31.2/10 gap 2 even at light-ior 1 (1.24 vs 1.0 px), and ring_pair.rs measures light-ior 1 vs 6 there as byte-identical (0 px). Stock 1.5/20/12 gap 8 stays under the cap (1.16/2.28 vs 4); light-ior 1 vs 6 changes 10,861 px by at most 2 levels. ior 1.24/thickness 43.3/bevel 9/gap 8 binds from light-ior 6 (4.09 vs 4). The shift acts only on tilted chamfer normals, never on the band core on the flat face, so dense glass changes the halo tail, not the band position. The 1.24/bevel 9 capture is a new Glass row in ring_pair.rs. Brief: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings
- 2026-10-01T10:00:23Z (materials-26.04): scope: todo, planned. The decision is the cap model (keep the half-gap cap, a bevel-relative depth, or an auto light-ior), and it is not settled: on the owner's live Prism glass (1.28/31.2/10, gap 2) the cap binds even at light-ior 1, so light-ior 1 and 6 render byte-identically and prism.kdl's light-ior 6 has no visible effect. Spec first, for owner review. Evidence: src/tests/ring_pair.rs renders frozen matched pairs; add Glass rows for ior 1.02/1.24 (bevel 9)/1.5 and use RING_PAIR_DUMP PNGs as the owner's capture sheet. Formula table and measured binding: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings
- 2026-10-01T10:02:43Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:fd411147-f51a-4dff-8930-c6c5a1318ace","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T10:02:50Z (material-a85a18): resumed
  provenance: {"harness_session":"claude-code:fd411147-f51a-4dff-8930-c6c5a1318ace","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T10:08:42Z (material-a85a18): attached: cap-vs-nocap-sheet.png (37556 bytes): Pilot: bottom-right corner crops at a frozen mid-resize instant (ring_pair), 4x nearest. Rows stock 1.5/20/12 gap 8, binding 1.28/31.2/10 gap 2, dense124 1.24/43.3/9 gap 8, dense150 1.5/80/12 gap 5; columns current half-gap cap vs cap removed at light-ior 1/6/12
- 2026-10-01T10:08:42Z (material-a85a18): pilot: removing the cap (throwaway shader edit) makes binding, dense124 and dense150 show a second, ghost copy of the ring core on the chamfer with a seam at the face edge and a stippled corner; binding li1 vs li6 then differ by max 45. With the cap, binding and dense150 render li1/6/12 byte-identically, dense124 only li12. The cap is load-bearing; the open choice is what light-ior means inside it. Sheet attached.
- 2026-10-01T10:17:48Z (material-a85a18): decision (owner, 2026-10-01): keep the hard half-gap cap; the second core on the chamfer is the defect; light-ior inert where the cap binds is accepted. Spec: docs/specs/2026-10-01-filament-shift-cap-design.md (regression test for one core per edge, docs, Prism idea)
- 2026-10-01T10:19:19Z (material-a85a18): parked (waiting on user, review): Owner reviews docs/specs/2026-10-01-filament-shift-cap-design.md in .worktrees/material-a85a18; on approval the agent writes the implementation plan (writing-plans) there
  provenance: {"harness_session":"claude-code:fd411147-f51a-4dff-8930-c6c5a1318ace","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T10:22:46Z (material-a85a18): resumed
  provenance: {"harness_session":"codex:01a0f6fb-670e-7b72-88db-56a76f8642ea","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T10:23:10Z (material-a85a18): review: spec round 1 — verdict: revise; findings: P2 2, P3 1; reviewer: codex
- 2026-10-01T10:23:11Z (material-a85a18): Spec review details: P2 at lines 112-119: selecting the global profile maximum can select the exterior ghost and skip the real core; outward samples [82,60,20,83,50,0] pass despite two strong peaks, and all-zero profiles also pass. Require a positive face-core anchor before scanning outward. P2 at lines 61-75 and 138-144: qualify flat-face, inert-light-ior and tilt claims to unperturbed normals and zero chromatic aberration; main.frag perturbs normals before lightShift, and uncapped channel offsets retain light-ior dependence. P3 at lines 47-49 and 83: bevel-relative shifts are 0.3976 and 0.9910 px at light-ior 1 and 6, so the knob is not inert; the 1.02/80/12 gap-5 binding threshold is 10.1346, not 10.4. Review used shader/fixture inspection, attached pilot sheet and standalone arithmetic; no rendering suite or shader mutation was run.
- 2026-10-01T10:23:11Z (material-a85a18): parked (waiting on agent): Review assignment complete. Hand back to the design author: revise the spec in .worktrees/material-a85a18 to anchor the test on a positive face core, qualify fixture-only optical claims, and correct the numeric statements; then submit the revised spec for review before writing the implementation plan.
  provenance: {"harness_session":"codex:01a0f6fb-670e-7b72-88db-56a76f8642ea","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T10:24:32Z (material-a85a18): resumed
  provenance: {"harness_session":"claude-code:fd411147-f51a-4dff-8930-c6c5a1318ace","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T10:25:47Z (material-a85a18): spec revision 2 for review round 1: check anchored on a positive face core at face_inset + ring-gap (+-2 px) before the outward scan; optical claims scoped to the fixture's unperturbed normals and zero aberration (cap bounds the shared shift only; live glass has ca 0.36 and ripple 0.23, so light-ior still moves the uncapped aberration offsets, blue ~0.27 px at li 1 vs 0.10 at 6); bevel-relative depth 0.40->0.99 px (live, not inert); 1.02/80/12 threshold 10.1
