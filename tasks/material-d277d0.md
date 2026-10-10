---
id: material-d277d0
title: "Source: OSC 133 shell-integration watcher"
status: idea
priority: 2
size: m
created: 2026-09-02T12:09:35Z
updated: 2026-10-10T14:05:12Z
depends: [material-a54d89, material-07bac9]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: kitty and ghostty shell-integration marks (OSC 133 command start, end, exit status) drive done and error impulses, and an Active level while a command runs, on the terminal's window, with no familiar involved. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

The set/pulse/clear IPC already exists (`src/cli.rs`, `niri-ipc/src/lib.rs`), so the open work is event delivery from the terminal and the join to a niri window id. `Window.pid` is optional and does not map a command to a unique window. material-07bac9 is the probe that answers both; the brief (docs/notes/2026-09-29-signal-sources-brief.md) favours terminal or shell hooks over the existing IPC and defers a shared watcher framework or pty proxy, so the pty-shim option is out unless 07bac9 shows hooks cannot work.

Done when a command started and finished in kitty and ghostty produces the configured impulse on that window, and restart, window-close and lost-connection clear the slot (brief, Constraints). material-79d1de takes its transport from the same finding.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; existing set/pulse/clear IPC covers command state; research material-07bac9 establishes terminal event delivery and reliable window mapping; brief: docs/notes/2026-09-29-signal-sources-brief.md
- 2026-10-08T17:18:09Z (materials-26.04): curate: refined; body names the existing IPC, the open transport and window-join work, and the brief's deferral of a pty shim; added depends material-07bac9, the probe that wakes it. Process left unassessed: direct if 07bac9 names one hook for both terminals, planned if they diverge
- 2026-10-10T14:05:12Z (materials-26.04): scope: briefed; no change: readmitted only by the 2026-10-08 curate refinement, which matches the brief; material-07bac9 (todo) is still the unanswered transport and window-join question that wakes it; brief: docs/notes/2026-09-29-signal-sources-brief.md
