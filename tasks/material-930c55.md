---
id: material-930c55
title: "Source: familiar bridge in familiar/integrations/niri"
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-02T12:09:35Z
updated: 2026-09-24T18:42:48Z
started: 2026-09-24T18:42:48Z
completed: 2026-09-24T18:42:48Z
depends: [material-a54d89, fam-e7fa72]
tags: [signals, sources]
model: "claude-opus-5-5[1m]"
---

Cross-repo pointer: the familiar bridge that feeds niri's per-window material signals is implemented in familiar as fam-e7fa72 (familiar-niri watch pushes intent.json, joined to niri-windows.json, through set/pulse/clear-window-signal, one aggregated `familiar` slot per window; mapping from docs/materials/2026-09-02-material-signals-design.md section 2).

Why: with Prism's ring colorSource=familiar, every terminal ring rests on the same manual color because nothing sends set-window-signal (material-068639 made the Prism option say so).

Done when: fam-e7fa72 is done, and on this compositor `niri msg -j windows` shows familiar-sourced accents that differ across terminals hosting different projects, with the ring visibly taking each hue. Then revisit the colorSource disclosure material-068639 added in Prism (prism-b4d118 owns the Ring rows).

Where to look: fam-e7fa72; niri-ipc signal requests; Prism integrations/niri/render.js responseBlock.

## Notes

- 2026-09-22T15:37:52Z (materials-26.04): User-visible as of 2026-09-22: with Prism's ring colorSource set to familiar, every terminal shows the same resting ring color because nothing pushes set-window-signal. Filed as material-068639. This bridge is the fix for per-window hue; 068639 covers the interim honesty of the Prism option.
- 2026-09-24T18:15:39Z (materials-26.04): scope: scoped; implementation filed in familiar as fam-e7fa72 (direct, mid, m), this record now a pointer that closes on a live per-window hue check
- 2026-09-24T18:21:57Z (materials-26.04): fam-e7fa72 landed on familiar main 188c602 (not pushed). One-shot live sync from its worktree lit windows 65/66/67 as #c464bc tuxedo active/pulse, #d2a956 maine-coon quiet/breathe, #5990cf seal-point active/pulse; pulse and ttl commands accepted by niri; slots cleared afterwards. The running familiar-niri watch (started by niri at login) still runs the old code until restarted.
- 2026-09-24T18:21:57Z (materials-26.04): parked (waiting on user, review): Restart familiar-niri watch so it runs familiar 188c602, then confirm each terminal ring takes its familiar hue with Prism's colorSource=familiar; close this and revisit the 068639 disclosure in Prism
  provenance: {"harness_session":"claude-code:043fdd81-3649-4354-a9c1-0d8726289ef8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T18:42:48Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:043fdd81-3649-4354-a9c1-0d8726289ef8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T18:42:48Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:043fdd81-3649-4354-a9c1-0d8726289ef8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-24T18:42:48Z (materials-26.04): User confirmed 2026-09-24 after restarting familiar-niri watch on familiar 188c602: each terminal ring takes its session's familiar hue (5 windows live, distinct accents and levels). Prism disclosure cleanup filed as prism-492ce8
  provenance: {"harness_session":"claude-code:043fdd81-3649-4354-a9c1-0d8726289ef8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
