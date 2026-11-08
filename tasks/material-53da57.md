---
id: material-53da57
title: Demonstrate Ghostty command transport and niri window attribution
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [nested]
created: 2026-10-11T09:56:04Z
updated: 2026-10-11T09:56:23Z
depends: []
parent: material-9b8bf9
tags: [signals, sources]
source: "docs/notes/2026-09-29-signal-sources-brief.md#ghostty-transport-follow-up"
agent: codex
---

Question: Can the installed Ghostty deliver command start, successful exit and failed exit to a niri writer with a unique window identity, using supported terminal or opt-in shell hooks?
Where to start: docs/notes/2026-09-29-signal-sources-brief.md (2026-10-11 transport finding); material-07bac9 and its saved nested probe; installed Ghostty shell integration and official CLI/configuration/Linux activation documentation; src/cli.rs and niri-ipc/src/lib.rs.
Bound: Inspect supported interfaces and run one minimal opt-in demonstration in a private nested compositor. Exercise two OS windows, including a shared terminal process if supported; record absent or ambiguous identity explicitly. Check whether notification/progress events can reach that same transport. No production watcher, PTY proxy, host configuration rollout, numeric-progress IPC or rendering changes. If no safe supported route exists, finish with a documented gap and recommendation.
Expected result: Record event coverage, exact window join and launch constraints, normal close/restart behavior, interface stability, and the smallest recommended Ghostty adapter or an explicit supported-scope restriction on this task and in the existing brief. Preserve window-rule behavior or document any required identity change. Archive an assert-based reproduction and clean up all probe processes.
Ideas it wakes: On completion, note material-d277d0 and material-79d1de with the finding before merging; the terminal identity/lifecycle design consumes this evidence.

## Notes

- 2026-10-11T09:56:23Z (materials-26.04): concerns: material-07bac9 extension — establish the live Ghostty route left explicitly unknown by the completed kitty transport probe.
