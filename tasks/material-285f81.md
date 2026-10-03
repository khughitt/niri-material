---
id: material-285f81
title: "Settling review minors: layout unwrap, Niri-level redraw/timer assertions, flaky reload test"
status: done
priority: 3
size: s
complexity: mid
process: direct
owner: material-285f81
created: 2026-10-02T03:36:04Z
updated: 2026-10-02T16:20:59Z
started: 2026-10-02T16:06:16Z
completed: 2026-10-02T16:20:59Z
depends: []
parent: material-f86183
tags: [performance]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

Deferred Minor findings from the whole-change review of sustained optic settling (2026-10-02, final-review.md in the material-a1d7da SDD workspace): .unwrap() at src/layout/mod.rs:2820 panics if the two active/idle copies disagree; Niri-level tests never assert that edges queue a redraw or that the optic timer is dropped and re-armed (spec §7.5); a reload test is timing-dependent; one signal test pauses a clock the code never reads; Optic::next_change doc does not say it returns a logical deadline; hidden-tile tests assert only the shared clock, not a rendered tile.

## Notes

- 2026-10-02T16:06:16Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:06:25Z (material-285f81): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:06:31Z (material-285f81): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:20:59Z (material-285f81): done
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:20:59Z (material-285f81): Layout reads input_active from the shared clock (no second copy, no unwrap); Niri-level test asserts idle/resume edges queue a redraw and the optic timer drops when held and re-arms on resume, same-side reloads and repeated input queue nothing (mutation-checked); reload test raises to 10 s instead of racing 200 ms; signal test no longer pauses a clock the solver never reads; Optic::next_change documents its logical deadline; hidden-tile test asserts the tile's rendered optics hold, resume without catch-up, and move
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
