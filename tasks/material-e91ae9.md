---
id: material-e91ae9
title: "Source: per-pid audio activity from PipeWire"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T22:07:58Z
depends: [material-a54d89]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: a watcher that maps active PipeWire output streams to windows by pid and writes an Active level with Breathe while sound plays, so the window that is talking is findable. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:07:58Z (materials-26.04): scope: briefed; audio Active/Breathe fits current slots; Wayland connection PID can be missing or shared, so stream attribution and teardown need evidence; brief: docs/notes/2026-09-29-signal-sources-brief.md
