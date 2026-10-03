---
id: material-c1330b
title: Validate workspace signal folding from existing IPC events
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: material-c1330b
created: 2026-09-29T22:31:42Z
updated: 2026-10-03T00:43:35Z
started: 2026-10-03T00:33:02Z
completed: 2026-10-03T00:43:34Z
depends: []
parent: material-b5cbd6
tags: [signals, ipc]
source: "docs/notes/2026-09-29-signal-model-extensions-brief.md#workspace-replay"
model: claude-opus-5-5
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
- 2026-10-03T00:43:31Z (material-c1330b): replay: niri-ipc/tests/workspace_signal_replay.rs feeds compositor-shaped events through niri_ipc::state::EventStreamState (compositor mirror + one client) and folds max(Window.signal.level) per known workspace, None distinct from Quiet. Start: ws1{10,11} ws2{20} ws3{30 Demand}. Observed ws1/ws2/ws3: S0 connect none/none/Demand; S1 Quiet on 20 none/Quiet/Demand; S2 Notice on 10 Notice/Quiet/Demand; S3 Demand+TTL+Ping on 11 Demand/Quiet/Demand; S4 impulse expiry (same level) unchanged; S5 move 11->ws2 (one WindowOpenedOrChanged) Notice/Demand/Demand; S6 TTL demotes 11 to Active Notice/Active/Demand; S7 ws3 removed then 30->ws1: Notice/Active (30 unattributed) then Demand/Active; S8 drag 30 (ws None) Notice/Active, drop on ws2 Notice/Demand; S9 clear 10's last source none/Demand; S10 close 30 then 11 none/Active then none/Quiet; S11 reconnect after missed clear on 20 + Notice window 12 on ws1: stale none/Quiet, fresh snapshot none/none after WorkspacesChanged then Notice/none after WindowsChanged. Client fold equals compositor-mirror fold at every step.
- 2026-10-03T00:43:31Z (material-c1330b): ordering (from code at 6c62457f): refresh per dispatch (main.rs:270) runs refresh_signal_deadlines then ipc_refresh_layout = workspaces, windows, overview (niri.rs:844-846, ipc/server.rs:674-678); each event applied to the mirror then sent individually (server.rs:785-788, 897-900), no refresh boundary on the wire. Workspace change -> one full WindowOpenedOrChanged with the refresh-time fold and no WindowSignalChanged that refresh (server.rs:824-835, 617-620). signal_deadline_fired (niri.rs:2381-2393) sends no IPC; the following refresh emits WindowSignalChanged, and fold applies expired TTLs lazily (window/signal.rs:339-342). Close = bare WindowClosed after per-window events (server.rs:879-885). Drag reports workspace_id None (layout/mod.rs:1696-1700). New client gets replicate() = WorkspacesChanged, WindowsChanged, ... queued before joining the stream list with no await (server.rs:246-264, state.rs:99-108); slow clients dropped at 64 events (server.rs:42, 115-131).
- 2026-10-03T00:43:31Z (material-c1330b): gaps: no refresh boundary, so per-event folds expose intermediate states (workspace removal before the move off it leaves a Demand unattributed; reconnect shows all-none between WorkspacesChanged and WindowsChanged); refresh-atomic cross-workspace updates for two windows in one refresh are unknown, not shown by the replay or guaranteed by code; dragged windows belong to no workspace; impulse-only changes emit events without level change; cost/latency not measured.
- 2026-10-03T00:43:31Z (material-c1330b): recommendation: keep the workspace summary client-side, no compositor summary event now. Client must recompute from reducer state (not incremental counters), fold only known workspace ids, keep None distinct from Quiet, resync from a fresh snapshot on reconnect. Revisit a compositor summary only if a consumer needs refresh-atomic cross-workspace summaries, defined drag attribution, a centrally defined fold beyond level, or measured client cost it cannot bear. Unknown consumer requirements: which bar/overview and which fields (level vs accent/motion/impulses/source count); tolerance for intermediate states and the reconnect blank; drag attribution; Quiet vs no-signal rendering; merging native urgency (Workspace.is_urgent); per-event cost/latency budget.
- 2026-10-03T00:43:34Z (material-c1330b): done
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:43:34Z (material-c1330b): Committed replay test niri-ipc/tests/workspace_signal_replay.rs and the brief's Workspace replay section: a client can fold per-workspace max signal level from existing IPC events and snapshots; recommend keeping it client-side, gaps and unknown consumer requirements recorded
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
