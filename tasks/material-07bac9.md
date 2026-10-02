---
id: material-07bac9
title: Establish terminal event transport and niri window attribution
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-29T22:06:56Z
updated: 2026-10-02T23:47:23Z
depends: []
parent: material-9b8bf9
tags: [signals, sources, needs-nested]
source: "docs/notes/2026-09-29-signal-sources-brief.md#terminal-transport"
agent: codex
---

Question: Can supported kitty and ghostty installations expose command start/exit and notification/progress events with a reliable niri window id, without a terminal-agnostic PTY proxy?
Where to start: docs/notes/2026-09-29-signal-sources-brief.md; docs/materials/2026-09-02-material-signals-design.md sections 1, 2 and 11; src/cli.rs; niri-ipc/src/lib.rs Window.pid; material-930c55 records the existing familiar window join. Check terminal versions and their official event/hook interfaces; record unavailable interfaces as unknown.
Bound: Read-only inspection plus one minimal opt-in transport demonstration in a nested compositor for a supported terminal. Demonstrate start, successful exit and failed exit in two windows, including two windows belonging to one terminal process if supported; record whether notification/progress events share the interface. No production watcher, PTY proxy, shell/terminal configuration rollout, progress IPC extension or rendering changes. Stop with a documented gap if no safe demonstration is possible.
Expected result: Record event coverage, window attribution, terminal restart/close behavior and a recommended transport/owning project on this task and in the brief. Show current IPC set-before-pulse and clearing behavior; separate completion from future numeric progress. Use the finding to scope implementation or request design rather than assuming identical terminal hooks.
Ideas it wakes: On completion, run tasks note on material-d277d0 and material-79d1de with the finding, in the same commit as this result.
