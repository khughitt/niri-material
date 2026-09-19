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
updated: 2026-09-19T01:52:57Z
started: 2026-09-19T00:56:02Z
depends: [material-92edaf]
parent: material-a76720
tags: [ring, dynamics]
source: docs/notes/2026-09-18-ring-next-steps-brief.md
agent: codex
spec: docs/specs/2026-09-18-ring-focus-motion-design.md
---

Produce a reviewed design, not implementation, for material-0e130e against the new within ring. Read the ring-next-steps brief, src/render_helpers/signal.rs tick_deadline/solve/envelope, src/layout/tile.rs tick_deadline and focus crossfade, src/window/signal.rs, and material-7afc31/material-f86183. Specify one finite focus-gain effect and static settled indication; sustained attention only when actually visible on an active output/workspace AND input-active, retaining static alert state otherwise. Decide duration, retrigger/focus-loss behavior, idle threshold, resume behavior and reduced/off motion. Reuse existing envelope/deadline machinery; distinguish existing slab-in-view gate from unproven occlusion/input-idle support. Define native configuration and minimum Prism controls, ownership of legacy driftHz and profile/reset semantics; do not expose every solver constant. Include deterministic finite-tail/deadline tests, hidden/idle/active-output matrix, user-reviewed clips and strict-cost versus pixel-only capture rules. Coordinate shared gates without making all visibility/prefilter research a prerequisite. Done: reviewed spec with implementable task decomposition and acceptance; add finding notes to material-0e130e and material-743692 in the same result commit. Design/plan review remains required before code.

## Notes

- 2026-09-19T00:56:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:98707379-59de-42cc-a5df-87b6c16dc4f5","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T01:32:52Z (material-82323e): Design drafted at docs/specs/2026-09-18-ring-focus-motion-design.md: one eased lap on focus gain (ring-sweep-ms, animation loop, ends on the pinned pattern), ring-drift-hz retired with a parse error, attention motion gated on a compositor-wide signal idle-after-ms; awaiting user review.
- 2026-09-19T01:34:45Z (material-82323e): parked (waiting on user, review): User reviews docs/specs/2026-09-18-ring-focus-motion-design.md; on approval run writing-plans against it and attach with --plan, then step children under material-0e130e
  provenance: {"harness_session":"claude-code:98707379-59de-42cc-a5df-87b6c16dc4f5","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-19T01:52:57Z (material-82323e): Review round 1 applied: staged install/apply/restart rollout with rollback (§5), sweep duration snapshotted at start plus a cut rule for policy changes mid-lap, idle timer armed in Niri::new, saturating phase for the presentation-time clock, §7 boundary and periodicity tests made approximate, crossfade cost corrected to 400 ms.
