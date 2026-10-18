---
id: material-e91ae9
title: "Source: per-pid audio activity from PipeWire"
status: idea
priority: 2
size: s
reviewed: true
created: 2026-09-02T12:09:35Z
updated: 2026-10-18T23:00:10Z
depends: [material-a54d89, material-9cdccf]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: a watcher that maps active PipeWire output streams to windows by pid and writes an Active level with Breathe while sound plays, so the window that is talking is findable. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

Established (scope passes 2026-09-29 and 2026-10-18): Active with Breathe fits the current signal slots; no model change is needed. `Window.pid` (niri-ipc/src/lib.rs) is the PID that opened the Wayland connection: optional (portal windows have none) and shared by every window of one process. material-07bac9 showed a PID join collapsing two windows of one process. A stream-to-window join therefore needs evidence before a watcher or clearing policy is chosen.

Waiting on: material-9cdccf, the PipeWire attribution probe. Its finding decides whether audio is attributed per window (a unique match with ambiguity rejected, the rule material-9fb9ec designs for terminal sources) or not attempted for some stream classes. Brief: docs/notes/2026-09-29-signal-sources-brief.md.

## Notes

- 2026-09-29T22:07:58Z (materials-26.04): scope: briefed; audio Active/Breathe fits current slots; Wayland connection PID can be missing or shared, so stream attribution and teardown need evidence; brief: docs/notes/2026-09-29-signal-sources-brief.md
