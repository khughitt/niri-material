---
id: material-c1330b
title: Validate workspace signal folding from existing IPC events
status: doing
priority: 2
size: s
complexity: mid
process: direct
owner: material-c1330b
created: 2026-09-29T22:31:42Z
updated: 2026-10-03T00:35:50Z
started: 2026-10-03T00:33:02Z
depends: []
parent: material-b5cbd6
tags: [signals, ipc]
source: "docs/notes/2026-09-29-signal-model-extensions-brief.md#workspace-replay"
agent: codex
---

Question: Can a client derive a correct per-workspace maximum signal level from existing niri IPC snapshots and events, without a new compositor summary event?
Where to start: docs/notes/2026-09-29-signal-model-extensions-brief.md; niri-ipc/src/lib.rs (Window.signal, Window.workspace_id and Workspace); niri-ipc/src/state.rs (WindowsState::apply and replication); src/ipc/server.rs::ipc_refresh_windows; src/niri.rs::signal_deadline_fired.
Bound: Trace snapshot/event ordering and replay one sequence covering no signal versus Quiet, competing windows, a workspace move, TTL demotion, final-source clear, window close and reconnect. Use an isolated replay and the existing IPC reducer; no bar integration, daemon, new wire API or live host capture. Treat maximum level as a candidate summary, not an approved accent/motion/impulse fold.
Expected result: Record the replay inputs and observed workspace levels, any state gap, and a client-side-versus-compositor recommendation on this task and in the brief. List consumer requirements still unknown; do not infer atomic cross-workspace snapshots or acceptable performance from API shape alone.
Ideas it wakes: On completion, run tasks note material-6cca0a with the finding, in the same commit as this result.

## Notes

- 2026-10-03T00:33:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:35:50Z (material-c1330b): resumed
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
