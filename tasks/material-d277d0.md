---
id: material-d277d0
title: "Source: OSC 133 shell-integration watcher"
status: idea
priority: 2
size: m
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T22:07:57Z
depends: [material-a54d89]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: a small watcher that turns kitty and ghostty shell-integration marks (OSC 133 command start, end, exit status) into done and error impulses and an Active level while a command runs, with no familiar involved. Decide where it lives (kitty watcher kitten, ghostty hook, or a terminal-agnostic pty shim) and how it finds the niri window id. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; existing set/pulse/clear IPC covers command state; research material-07bac9 establishes terminal event delivery and reliable window mapping; brief: docs/notes/2026-09-29-signal-sources-brief.md
