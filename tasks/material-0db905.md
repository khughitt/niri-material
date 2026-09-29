---
id: material-0db905
title: Design sustained optic settling from the resource-aware rendering brief
status: doing
priority: 1
size: m
complexity: high
process: planned
owner: materials-26.04
created: 2026-09-29T21:44:30Z
updated: 2026-09-29T23:14:14Z
started: 2026-09-29T23:08:21Z
depends: []
parent: material-5d6b2c
tags: [performance]
source: docs/notes/2026-09-29-resource-aware-rendering-brief.md
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Why: Attention motion already uses signal { idle-after-ms } (default 30000; 0 disables), but Tile::optic_frame and AuroraOptic::rate do not read input activity. Finite-motion quiescence has passed the idle-budget trace; the remaining outcome is stopping optional sustained optic motion during extended input inactivity.
Where to start: docs/notes/2026-09-29-resource-aware-rendering-brief.md; docs/specs/2026-09-18-ring-focus-motion-design.md sections 2-3 and 8; src/activity.rs; src/niri.rs activity handlers; src/layout/tile.rs::optic_frame/tick_deadline/material_dynamics; src/render_helpers/material/optics/mod.rs::OpticFrame and aurora.rs::rate.
Bound: Write a reviewed design, then an implementation plan for review; this task does not implement or capture it. Reuse the existing activity threshold and visibility gates. Settle which sustained optics participate, opt-in/default behavior, phase freezing and resume behavior, interaction with motion full/reduced/off and animations off, and how reload/hidden workspaces/DPMS/other lit outputs behave. Preserve static level/accent indicators, finite focus beams and impulses, ordinary client updates, and the existing attention contract. Do not introduce a second input-idle detector or conflate signal Quiet with input inactivity.
Done: The design specifies no optional optic deadline or changing optic fingerprint while settled, continued visible client rendering on real damage, and finite wake behavior; includes deterministic deadline/fingerprint checks and a pilot-first capture matrix with positive controls. Document a freeze/resume strategy that avoids an unexplained phase jump.
Ideas it wakes: On completion, run tasks note on material-f86183 with the reviewed decisions, in the same commit as this result.

## Notes

- 2026-09-29T23:08:21Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a0ef6c-81be-7053-b1db-8781e2ef7949","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-29T23:12:49Z (material-0db905): Design proposal: default Aurora settling on the existing idle threshold; shared paused optic timeline preserves phase across idle/resume and hidden-workspace cycles; broaden the attention-only activity redraw predicate. Draft is pending owner review; no implementation or captures.
- 2026-09-29T23:13:26Z (material-0db905): Spec self-review: checked the scoped requirements against the draft and current source; explicit real/logical deadline conversion, predicted-time monotonicity, Aurora-only resume interest, preserved attention semantics, bounded edge wakes, and pilot positive controls are covered. Local links, placeholder scan, git diff --check, tasks check and docs-only just test-fast passed.
- 2026-09-29T23:13:26Z (material-0db905): parked (waiting on user, review): User reviews .worktrees/material-0db905/docs/specs/2026-09-29-sustained-optic-settling-design.md; agent records the review and revises as needed, then drafts the implementation plan for separate review. This task implements and captures nothing.
  provenance: {"harness_session":"codex:01a0ef6c-81be-7053-b1db-8781e2ef7949","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-29T23:14:14Z (material-0db905): Commit hook initially refused missing optional machine-local Cargo paths in the fresh worktree (ops-3b4a6e filed). Hydrated ignored Cargo config and target link using the existing local build cache; no repository source or host launcher changed.
