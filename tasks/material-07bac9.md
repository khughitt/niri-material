---
id: material-07bac9
title: Establish terminal event transport and niri window attribution
status: done
priority: 2
size: s
complexity: mid
process: direct
needs: [nested]
owner: material-07bac9
created: 2026-09-29T22:06:56Z
updated: 2026-10-11T09:31:04Z
started: 2026-10-11T09:09:37Z
completed: 2026-10-11T09:31:03Z
depends: []
parent: material-9b8bf9
tags: [signals, sources]
source: "docs/notes/2026-09-29-signal-sources-brief.md#terminal-transport"
agent: codex
---

Question: Can supported kitty and ghostty installations expose command start/exit and notification/progress events with a reliable niri window id, without a terminal-agnostic PTY proxy?
Where to start: docs/notes/2026-09-29-signal-sources-brief.md; docs/materials/2026-09-02-material-signals-design.md sections 1, 2 and 11; src/cli.rs; niri-ipc/src/lib.rs Window.pid; material-930c55 records the existing familiar window join. Check terminal versions and their official event/hook interfaces; record unavailable interfaces as unknown.
Bound: Read-only inspection plus one minimal opt-in transport demonstration in a nested compositor for a supported terminal. Demonstrate start, successful exit and failed exit in two windows, including two windows belonging to one terminal process if supported; record whether notification/progress events share the interface. No production watcher, PTY proxy, shell/terminal configuration rollout, progress IPC extension or rendering changes. Stop with a documented gap if no safe demonstration is possible.
Expected result: Record event coverage, window attribution, terminal restart/close behavior and a recommended transport/owning project on this task and in the brief. Show current IPC set-before-pulse and clearing behavior; separate completion from future numeric progress. Use the finding to scope implementation or request design rather than assuming identical terminal hooks.
Ideas it wakes: On completion, run tasks note on material-d277d0 and material-79d1de with the finding, in the same commit as this result.

## Notes

- 2026-10-11T09:09:37Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a12a31-1d83-7393-a62a-bfb6585d5fdb","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-11T09:13:05Z (material-07bac9): resumed
  provenance: {"harness_session":"codex:01a12a31-1d83-7393-a62a-bfb6585d5fdb","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-11T09:20:45Z (material-07bac9): First nested probe: kitty 0.49.2 emitted start and exit 0/1 in two OS windows with the same Wayland PID; unique app IDs routed Done/Error to only the intended niri window. Pulse-before-set refuses, TTL retains its source, clear removes it. Probe fixes before rerun: send OSC fixture from a file (send-text decodes backslashes); cache identity before on_close because kitty removes its OS-window record first. Installed niri binary 26.04 (99196eba), not a worktree build.
- 2026-10-11T09:26:14Z (material-07bac9): attached: terminal-transport-probe.md (19881 bytes): Nested kitty transport reproduction, successful assert-based verdict, and real command/notification/progress/close events; installed niri 99196eba.
- 2026-10-11T09:27:49Z (material-07bac9): verdict: bounded transport probe passed on installed niri 26.04 (99196eba; SHA-256 4b13e677894b228cf4dedee5e6bad91fa826d3ed9d5019ed11b1c400886dfab3). Two kitty OS windows in one PID routed Active while running and Done/Error on exit without sibling writes; observed separate OSC 9/99 notifications, OSC 9;4 progress, close callbacks, stale-ID rejection and clean restart. Production identity and bridge disconnect cleanup require design; Ghostty external hooks/window identity unverified. No production code or host config changes. Worktree baseline just test-fast selected zero packages; real probe supplied behavioral assertions. Unit stopped and host-load reported nothing left running.
- 2026-10-11T09:31:02Z (material-07bac9): review: impl round 1 — verdict: accept; findings: none; reviewer: codex
- 2026-10-11T09:31:02Z (material-07bac9): review disposition: accepted the bounded evidence. Declined production adapter/install, ordinary-window and pane policy, abrupt-death/disconnect injection, live Ghostty demo, and numeric-progress IPC/rendering as future design/implementation scope; each limitation is explicit in the brief and follow-up notes. No residual findings.
- 2026-10-11T09:31:03Z (material-07bac9): done
  provenance: {"harness_session":"codex:01a12a31-1d83-7393-a62a-bfb6585d5fdb","harness_session_source":"CODEX_THREAD_ID"}
- 2026-10-11T09:31:03Z (material-07bac9): Established kitty command transport and explicit per-window join in a nested demonstration; archived executable reproduction and verdict, documented Ghostty unknowns and lifecycle limits, and requested identity/cleanup design on both dependent ideas. Review accepted with no findings; probe assertions and docs checks passed; no host processes left running.
  provenance: {"harness_session":"codex:01a12a31-1d83-7393-a62a-bfb6585d5fdb","harness_session_source":"CODEX_THREAD_ID"}
