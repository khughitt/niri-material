---
id: material-82323e
title: Design bounded focus and attention ring motion from the brief
status: doing
priority: 1
size: s
complexity: high
process: planned
owner: materials-26.04
created: 2026-09-19T00:26:01Z
updated: 2026-09-19T00:56:02Z
started: 2026-09-19T00:56:02Z
depends: [material-92edaf]
parent: material-a76720
tags: [ring, dynamics]
source: docs/notes/2026-09-18-ring-next-steps-brief.md
agent: codex
---

Produce a reviewed design, not implementation, for material-0e130e against the new within ring. Read the ring-next-steps brief, src/render_helpers/signal.rs tick_deadline/solve/envelope, src/layout/tile.rs tick_deadline and focus crossfade, src/window/signal.rs, and material-7afc31/material-f86183. Specify one finite focus-gain effect and static settled indication; sustained attention only when actually visible on an active output/workspace AND input-active, retaining static alert state otherwise. Decide duration, retrigger/focus-loss behavior, idle threshold, resume behavior and reduced/off motion. Reuse existing envelope/deadline machinery; distinguish existing slab-in-view gate from unproven occlusion/input-idle support. Define native configuration and minimum Prism controls, ownership of legacy driftHz and profile/reset semantics; do not expose every solver constant. Include deterministic finite-tail/deadline tests, hidden/idle/active-output matrix, user-reviewed clips and strict-cost versus pixel-only capture rules. Coordinate shared gates without making all visibility/prefilter research a prerequisite. Done: reviewed spec with implementable task decomposition and acceptance; add finding notes to material-0e130e and material-743692 in the same result commit. Design/plan review remains required before code.

## Notes

- 2026-09-19T00:56:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:98707379-59de-42cc-a5df-87b6c16dc4f5","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
