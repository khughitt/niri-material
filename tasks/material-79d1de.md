---
id: material-79d1de
title: "Source: OSC 9;4 progress and OSC 9/99 notifications"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T22:07:57Z
depends: [material-a54d89]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: map terminal progress reports (OSC 9;4) to the progress channel once it exists, and desktop-notification escapes (OSC 9, OSC 99) to Notice level with a ping. Depends on the shell-integration watcher for the transport. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; notification state fits existing levels/impulses, but numeric job progress is absent; terminal transport research material-07bac9 precedes implementation; brief: docs/notes/2026-09-29-signal-sources-brief.md
