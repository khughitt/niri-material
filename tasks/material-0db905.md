---
id: material-0db905
title: Design sustained optic settling from the resource-aware rendering brief
status: doing
priority: 1
size: m
complexity: high
process: planned
owner: material-0db905
created: 2026-09-29T21:44:30Z
updated: 2026-09-30T10:37:04Z
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
- 2026-09-30T01:08:33Z (material-0db905): resumed
  provenance: {"harness_session":"codex:01a0ef6c-81be-7053-b1db-8781e2ef7949","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T01:08:33Z (material-0db905): review: spec round 1 — verdict: revise; findings: P2 3, P3 3; reviewer: claude-code/claude-fable-5-1
- 2026-09-30T01:11:43Z (material-0db905): Spec round 1 dispositions: P2-1 running samples are pure and only the idle hold uses the greatest recorded render sample; P2-2 activity interest probes the registry with an as-if-running snapshot, and all optics get the logical-time-only contract and paused real-time invariance checks; P2-3 chose the existing Clock carrier over additional constructor plumbing after checking upstream seams, and added seam-report regeneration. P3-1 separates IPC power-on from session-resume/unlock notifications; P3-2 states reading/video, inhibitor/screencast and shared signal-threshold consequences; P3-3 records the union-of-deadlines trade-off and adds a misaligned combined-motion capture control. Default-on and no extra resume easing remain proposed for owner acceptance.
- 2026-09-30T01:11:43Z (material-0db905): parked (waiting on user, review): User reviews the revised .worktrees/material-0db905/docs/specs/2026-09-29-sustained-optic-settling-design.md after spec round 1; agent records the next review and, once accepted, drafts the implementation plan for separate review. No implementation or captures in this task.
  provenance: {"harness_session":"codex:01a0ef6c-81be-7053-b1db-8781e2ef7949","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T09:59:37Z (material-0db905): resumed
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T09:59:55Z (material-0db905): Resume check: retained the staged spec round-1 revision. Relevant activity, clock, tile, layout and optic sources match the current main checkout. New evidence on main from material-39a46f: Aurora adds +0.96 W at 4 Hz (upper 1.37 W) and +0.77 W at 2 Hz (upper 1.03 W); resting jelly upper 0.19 W. Preserve that main task note when merging. The current main justfile now provides test-one, superseding the older branch recipe note in spec section 7. Owner spec review remains the next gate; no implementation or captures.
- 2026-09-30T09:59:55Z (material-0db905): parked (waiting on user, review): User reviews .worktrees/material-0db905/docs/specs/2026-09-29-sustained-optic-settling-design.md; after acceptance, agent records spec review round 2, updates the test-front-door note against the current main justfile, and drafts the implementation plan for separate review. Preserve both worktree review notes and the main power-evidence note when reconciling the task record.
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:10:29Z (material-0db905): resumed
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:10:29Z (material-0db905): review: spec round 2 — verdict: revise; findings: P2 1, P3 4; reviewer: claude-code/claude-opus-5-5
- 2026-09-30T10:12:53Z (material-0db905): Spec round 2 dispositions: P2-1 OpticFrame exposes only logical_now; entries return logical deadlines; the registry suppresses paused scheduling, converts the minimum to real time, and exposes next_logical_change for activity interest. P3-1 requires explicit edge times and one injected time source for detector, timer delays and render samples. P3-2 uses test-one then test-fast. P3-3 requires transition-only Tracy messages with edge timestamp, direction and held logical time. P3-4 requires a real TTY capture or an unverified result. Owner accepted default-on shared-threshold settling and phase-continuous resume without easing. Added constant-client-damage caveat and current power evidence. Spec links, placeholder checks, tasks check and diff checks passed; just test-fast selected zero packages for this docs-only change.
- 2026-09-30T10:12:53Z (material-0db905): parked (waiting on user, review): User reviews the round-2 revision at .worktrees/material-0db905/docs/specs/2026-09-29-sustained-optic-settling-design.md; after acceptance, agent records spec round 3 and drafts the implementation plan for separate review. Reconcile main power-evidence note with worktree review notes when merging. No implementation or captures in this task.
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:36:02Z (material-0db905): resumed
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:36:02Z (material-0db905): review: spec round 3 — verdict: revise; findings: P2 1; reviewer: human
- 2026-09-30T10:37:04Z (material-0db905): Spec round 3 disposition: P2 narrowed the time-domain invariant to existing mechanisms. Production detector and timer-delay reads retain get_monotonic_time; each edge forwards its exact event timestamp explicitly to layout/timeline pause or resume. Cached/predicted Clock samples are not detector timestamps. Timeline/layout tests use Clock::with_time and explicit virtual timestamps for zero-start and full-cycle coverage; Niri fixtures retain real time and may not freeze or advance set_unadjusted across an activity edge. No Niri clock injection or virtual timer mechanism is required. Verified against activity handlers and attention_idle fixtures; links, placeholders, tasks check and diff checks passed; just test-fast selected zero packages for this docs-only revision.
- 2026-09-30T10:37:04Z (material-0db905): parked (waiting on user, review): User reviews the round-3 revision at .worktrees/material-0db905/docs/specs/2026-09-29-sustained-optic-settling-design.md; after acceptance, agent records the next spec review and drafts the implementation plan for separate review. Preserve main power-evidence and worktree review notes when merging. No implementation or captures in this task.
  provenance: {"harness_session":"codex:01a0f1c0-c05d-71f1-8d5b-ed294adf0239","harness_session_source":"CODEX_SESSION_ID"}
