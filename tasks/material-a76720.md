---
id: material-a76720
title: "Ring follow-through: appearance, bounded motion, and Prism controls"
status: done
priority: 1
size: m
complexity: high
process: planned
created: 2026-09-19T00:25:37Z
updated: 2026-09-22T13:45:28Z
completed: 2026-09-22T13:45:28Z
depends: [prism-d8ee06, prism-0ea68f, prism-71b7d1, material-0e130e, material-9306b5, prism-eff23a]
tags: [ring, prism]
source: docs/notes/2026-09-18-ring-next-steps-brief.md
agent: codex
---

Coordinate ring follow-through against depth-order commits e33aa968/e79b226b/e013edc0, which are not yet integrated or installed. Keep appearance tuning, motion policy and Prism wiring distinct. Reuse material-9306b5 from the material-5b3107 branch; preserve material-0e130e under its resource-aware parent. Done when the reviewed motion policy is delivered, the chosen appearance is accepted, and approved controls round-trip through Prism into both native materials with validation and scheduling evidence. No cap redesign, general animation framework, or deployment authorization is implied. Handoff: docs/notes/2026-09-18-ring-next-steps-brief.md.

## Notes

- 2026-09-19T00:50:09Z (materials-26.04): After material-5b3107 integrates: tasks edit material-9306b5 --parent material-a76720, tasks dep material-a76720 --on material-9306b5, and close material-5b3107 from prime's closeout. The scoping brief (docs/notes/2026-09-18-ring-next-steps-brief.md) is git-excluded; these follow-ups live here so they survive without it.
- 2026-09-20T01:02:49Z (materials-26.04): Owner review 2026-09-19 of the live sweep (ring-sweep-ms 1500): the focus 'pulse' is far too fast. Cause: travel = sin(2θ+φ)·sin(3θ−2φ) has a 3-cycles-per-lap component at any point, so a 1.5 s lap flickers each spot ~2 Hz mean and ~6 Hz at the eased start. Owner wants a glowing light moving within the glass (one lobe, once per lap; ≤2 Hz) and the fast rhythm reserved for bells/attention. Next: one design under this goal covering the travel pattern, the band's glass reading, and body/band depth (material-9306b5 as the placement half). Duration alone (sweepMs 4500 ≈ 2 Hz initial) is a stopgap, not the fix.
- 2026-09-22T13:45:28Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:86218c99-e333-49f8-965c-e0e30c526e78","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-22T13:45:28Z (materials-26.04): Reviewed motion policy delivered (material-82323e), the ring beam accepted on the 2026-09-21 sheets and shipped (material-2c3984, installed 649c731b), and the approved controls round-trip through Prism into both native materials: beamSpeed/gap/glow/light-ior emitted, geometry validated against the installed build, applied live 2026-09-22
  provenance: {"harness_session":"claude-code:86218c99-e333-49f8-965c-e0e30c526e78","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
