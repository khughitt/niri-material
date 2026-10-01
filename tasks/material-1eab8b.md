---
id: material-1eab8b
title: One-core regression test for the ring cap
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: material-a85a18
created: 2026-10-01T10:33:12Z
updated: 2026-10-01T11:09:34Z
started: 2026-10-01T11:05:13Z
completed: 2026-10-01T11:09:34Z
depends: []
parent: material-a85a18
tags: [rendering]
agent: claude-code/claude-opus-5-5
plan: docs/plans/2026-10-01-filament-shift-cap.md
step: "Task 1: One-core regression test for the ring cap"
---

## Notes

- 2026-10-01T11:05:13Z (material-a85a18): started
  provenance: {"harness_session":"claude-code:a67c31bc-8dd6-4172-ac6a-b8a3d5ec3118","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T11:08:22Z (material-a85a18): attached: ring-cap-sheet.png (24783 bytes): ring_cap_keeps_one_core, capped: bottom-right corners at the frozen mid-resize instant, 4x nearest; rows stock, binding, ior102, ior124, ior150; columns light-ior 1, 6, 12
- 2026-10-01T11:08:22Z (material-a85a18): attached: ior124-cap-light-ior-1-on.png (6198 bytes): ior 1.24 / thickness 43.3 / bevel 9 / ring-gap 8, light-ior 1, ring on, frozen mid-resize instant
- 2026-10-01T11:08:22Z (material-a85a18): attached: ior124-cap-light-ior-6-on.png (6399 bytes): ior 1.24 / thickness 43.3 / bevel 9 / ring-gap 8, light-ior 6, ring on, frozen mid-resize instant
- 2026-10-01T11:08:22Z (material-a85a18): attached: ior124-cap-light-ior-12-on.png (6399 bytes): ior 1.24 / thickness 43.3 / bevel 9 / ring-gap 8, light-ior 12, ring on, frozen mid-resize instant
- 2026-10-01T11:08:22Z (material-a85a18): attached: report.log (1485 bytes): ring_cap_keeps_one_core report (capped run)
- 2026-10-01T11:08:49Z (material-a85a18): attached: mutation.log (4318 bytes): ring_cap_keeps_one_core with the cap removed (float cap = 1e6), shader reverted after
- 2026-10-01T11:08:49Z (material-a85a18): mutation: cap removed -> ring_cap_keeps_one_core exit 100; second-maximum failures on binding (light-ior 1, 6, 12; left, right, top, bottom) and ior150 (light-ior 1, 6, 12; left, right, top, bottom); shader reverted
- 2026-10-01T11:09:34Z (material-a85a18): done
  provenance: {"harness_session":"claude-code:a67c31bc-8dd6-4172-ac6a-b8a3d5ec3118","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T11:09:34Z (material-a85a18): ring_cap_keeps_one_core guards the half-gap cap on five glass rows; one_core unit-tested; mutation gate passed (binding and ior150 second maxima); captures attached
  provenance: {"harness_session":"claude-code:a67c31bc-8dd6-4172-ac6a-b8a3d5ec3118","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
