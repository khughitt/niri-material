---
id: material-f86183
title: "Inactivity settle mode: a genuinely quiescent low-power state"
status: doing
priority: 1
size: l
complexity: high
process: direct
owner: materials-26.04
created: 2026-09-11T23:34:15Z
updated: 2026-10-07T08:12:16Z
started: 2026-10-02T16:22:49Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, dynamics]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

After a period of mouse/keyboard inactivity, settle the glass into a static state: no per-frame uniform churn, no redraws without a stimulus, easing back on activity. Close to material-265eb0's settle state (which measures the idle budget) and material-4bf8b8 (frost-on-idle, a visible response to the same signal); this task is the input-inactivity trigger and the quiescence guarantee.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

2026-09-16 priority: extended keyboard/mouse inactivity must stop optional sustained material dynamics, including alert/attention ring animation; retain static state indicators. Sustained attention animation is eligible only while BOTH the window is actually visible on an active output/workspace and the user is active. Coordinate the visibility gate with material-7afc31; settle animation deadlines and uniform churn, not only the visible amplitude. Choose inactivity delay and resume behavior during design; do not invent them in the task.

## Notes

- 2026-09-16T12:00:25Z (materials-26.04): Raised P2 to P1 at user request: prioritize avoiding invisible-window and idle resource waste; distinguish observed client GPU use from unproven compositor rendering.
- 2026-09-29T21:43:50Z (materials-26.04): scope: briefed; 30 s input-idle attention gating and finite-motion quiescence are shipped; Aurora still ignores input activity, so wider optic settling needs reviewed design; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
- 2026-09-30T11:03:15Z (material-0db905): Accepted design from material-0db905: docs/specs/2026-09-29-sustained-optic-settling-design.md (spec round 4) and docs/plans/2026-09-30-sustained-optic-settling.md (plan round 2). Aurora settles by default on the existing 30000 ms input-idle threshold; 0 disables both Aurora and attention settling. Shared Clock logical time freezes phase and resumes without easing/catch-up. Optics see logical time only; the registry suppresses paused deadlines and translates the earliest logical deadline, checking frame/snapshot agreement. Preserve real client damage, finite effects, static indicators, attention semantics and existing visibility gates. Production edges retain explicit get_monotonic_time timestamps; virtual time stays in timeline/layout tests, Niri fixtures stay real-time. Verify actual unlock/power-on/inhibitor paths, transition Tracy markers and pilot-first captures; TTY resume remains a real-TTY check and screencast uses a real consumer. No watt saving is claimed from quiescence alone. Five sequential execution children start with material-a1d7da. User explicitly paused before execution; no implementation or captures started. On resume, scope this existing idea from the accepted artifacts and begin Task 1.
- 2026-09-30T11:12:56Z (material-0db905): scope: scoped; accepted spec and plan settle approach and verification; execute five sequential children, with hardware lanes gated on available host environment
- 2026-10-02T05:07:10Z (materials-26.04): parked (waiting on user, review): Owner: judge the settling clip (material-2ee11e); then agent finishes material-2ee11e in .worktrees/material-a1d7da and closes this goal once its children and follow-ups are settled.
  provenance: {"harness_session":"claude-code:2396c14b-dc41-43ce-a03a-8efc21e78f8a","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T08:19:52Z (material-a1d7da): Owner accepted the idle/resume clip on 2026-10-02 and authorized local integration. Implementation Tasks 1–4 and headless evidence are complete; material-2ee11e owns combined-tree validation and integration. This goal remains open for material-f7eb0b (real TTY/unlock), material-3acc86 (screencast), material-80caf4 (idle inhibitor), material-1af3c6 (two-output removal) and material-285f81 (deferred review minors).
- 2026-10-02T08:23:38Z (materials-26.04): parked (waiting on agent, dependency): Agent: complete material-285f81 review minors in a new .worktrees/ task checkout, then implement the remaining bounded lifecycle capture lanes in material-f7eb0b, material-3acc86, material-80caf4 and material-1af3c6; arrange real-TTY/two-output environments and pilot-first runs before closing this goal. The owner accepted the clip; material-2ee11e is done and the implementation is locally integrated.
  provenance: {"harness_session":"codex:01a0fba2-4318-7890-9b41-bbe036afaf06","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-02T16:22:49Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:22:49Z (materials-26.04): parked (waiting on agent, dependency): Agent: implement the remaining bounded lifecycle capture lanes in material-f7eb0b (TTY resume/unlock), material-3acc86 (screencast consumer), material-80caf4 (idle inhibitor) and material-1af3c6 (two outputs, one removed); arrange real-TTY/two-output environments and pilot-first runs before closing this goal. material-285f81 (review minors) landed in b6a0cf62.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T18:13:14Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T18:13:14Z (materials-26.04): parked (waiting on user, dependency): Owner: attach a second monitor to titan (or name a two-output host) so material-1af3c6 can run; then agent: finish 1af3c6's two-output lane and its pilot, confirm this goal's acceptance against the landed lifecycle lanes (f7eb0b TTY resume/unlock, 3acc86 screencast, 80caf4 idle inhibitor, all done), and close it.
  provenance: {"harness_session":"claude-code:e09fb767-0d8d-4b44-86a6-8d9ffc6849af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T08:12:15Z (materials-26.04): material-1af3c6 dropped on the owner's word (no second monitor); replaced by material-f5b371, a deterministic in-process two-output removal test. The real two-output capture stays unverified on hardware.
- 2026-10-07T08:12:15Z (materials-26.04): parked (waiting on agent): Agent: implement material-f5b371 (in-process two-output removal test, evidence doc update), then confirm this goal's acceptance against the landed lanes (f7eb0b, 3acc86, 80caf4) and close it
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
