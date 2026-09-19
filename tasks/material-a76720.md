---
id: material-a76720
title: "Ring follow-through: appearance, bounded motion, and Prism controls"
status: todo
priority: 1
size: m
complexity: high
process: planned
created: 2026-09-19T00:25:37Z
updated: 2026-09-19T02:49:37Z
depends: [prism-d8ee06, prism-0ea68f, prism-71b7d1, material-0e130e, material-9306b5, prism-eff23a]
tags: [ring, prism]
source: docs/notes/2026-09-18-ring-next-steps-brief.md
agent: codex
---

Coordinate ring follow-through against depth-order commits e33aa968/e79b226b/e013edc0, which are not yet integrated or installed. Keep appearance tuning, motion policy and Prism wiring distinct. Reuse material-9306b5 from the material-5b3107 branch; preserve material-0e130e under its resource-aware parent. Done when the reviewed motion policy is delivered, the chosen appearance is accepted, and approved controls round-trip through Prism into both native materials with validation and scheduling evidence. No cap redesign, general animation framework, or deployment authorization is implied. Handoff: docs/notes/2026-09-18-ring-next-steps-brief.md.

## Notes

- 2026-09-19T00:50:09Z (materials-26.04): After material-5b3107 integrates: tasks edit material-9306b5 --parent material-a76720, tasks dep material-a76720 --on material-9306b5, and close material-5b3107 from prime's closeout. The scoping brief (docs/notes/2026-09-18-ring-next-steps-brief.md) is git-excluded; these follow-ups live here so they survive without it.
