---
id: material-9fb9ec
title: Design terminal command-source identity and lifecycle from the transport brief
status: todo
priority: 2
size: s
complexity: high
process: planned
created: 2026-10-11T09:56:24Z
updated: 2026-10-11T09:56:43Z
depends: [material-53da57]
parent: material-9b8bf9
tags: [signals, sources, design]
source: "docs/notes/2026-09-29-signal-sources-brief.md#terminal-identity-lifecycle-design"
agent: codex
---

Why: material-07bac9 proved kitty command transport, but unique app IDs alter window-rule matches, PID ancestry collapses shared-process windows, and TTL only demotes a slot. Ghostty transport evidence comes from material-53da57. These decisions must be settled before implementing material-d277d0 or reusing the join for material-79d1de.
Outcome: A reviewed design spec and implementation plan for the smallest terminal command-source adapter over existing niri set/pulse/clear IPC. Choose a supported per-window identity and launch policy that preserves existing rules or explicitly declares the migration; reject ambiguous attribution. Define pane aggregation, source naming/ownership, Active-to-Done/Error transitions, finite activity leases for long-running commands, clear/reconciliation after bridge death or IPC loss, and cached-ID invalidation on compositor restart. State the supported Ghostty route or an explicit limitation from the prerequisite probe. Preserve independent familiar accents and configuration-owned appearance.
Where to look: docs/notes/2026-09-29-signal-sources-brief.md; material-07bac9 attachment; material-53da57; docs/materials/2026-09-02-material-signals-design.md sections 1, 2 and 11; src/window/signal.rs; src/cli.rs; niri-ipc/src/lib.rs; familiar integrations/niri/window.js for the observed shared-PID applicability limit.
Bound: Design and plan review, not implementation or deployment. Prefer an adapter owned in material with opt-in wiring in dots; no familiar dependency, PTY proxy, generic watcher framework, numeric-progress schema or rendering changes. Notification transport may reuse identity but requires its separate callback.
Done: User-reviewed spec and plan name the identity/launch policy, per-pane fold, ownership and lifecycle cleanup; define checks for concurrent commands, ambiguous joins, normal close/restart, abrupt terminal/bridge death, disconnect/reconnect and compositor restart; link the artifacts and leave an executable task handoff for material-d277d0. Record the shared identity/cleanup decisions on material-79d1de.

## Notes

- 2026-10-11T09:56:43Z (materials-26.04): concerns: material-07bac9 extension — turn the proven transport into a reviewed identity, aggregation and disconnect-cleanup contract before production adapters.
