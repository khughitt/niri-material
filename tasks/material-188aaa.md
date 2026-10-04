---
id: material-188aaa
title: Hold host disturbers for the length of a quiet capture
status: doing
priority: 2
size: s
complexity: mid
process: planned
needs: [quiet]
owner: disturber-hold
created: 2026-09-24T20:09:46Z
updated: 2026-10-04T09:13:05Z
started: 2026-10-04T09:02:22Z
depends: []
parent: material-2834d7
tags: [performance]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-04-capture-disturber-hold-design.md
---

Quiet-host captures are brittle to scheduled host activity that the readiness preflight cannot see, because it samples load only before the run. Evidence 2026-09-24 (material-cd0e1d): wali-rotate.timer (every 15 min) rotated the wallpaper at 16:06:58, which ran Prism's apply, reloaded the desktop niri, and turned glass back on through the active profile; in the same second the view-tilt smoke's nested probe kitty never mapped and the smoke failed after 13 minutes. Earlier the same day the desktop's own redraws failed the GPU P8/IQR gate until the monitors were powered off, and a monitor wake at ~05:08 would have perturbed a capture had one been running.

Scope: a capture-meta (or fixture-lib) hold that, for the run's duration, pauses a declared list of disturbers (wali-rotate.timer; monitor power via niri power-off-monitors; others found by auditing user timers), records each in capture.json with how to restore it, and restores on exit/TERM/INT; plus a post-run check that flags any disturber that fired mid-run (journal scan over the run window) so a result is never silently taken under one. Related: prism-6d1d72 (the glass toggle does not survive rotation), ops-2da76d (estimate vs actual).

## Notes

- 2026-09-24T20:56:53Z (materials-26.04): Second disturber, 2026-09-24 16:54:05: the desktop idle lock raised the GPU's P8 power floor from 9.9 W to ~11.5 W mid-run, and capture-meta's settle gate refused material-4241c3 at case 12 of 24 (22 min lost). The hold should fix the lock state for the whole run (lock first, or inhibit idle), not just the monitors.
- 2026-10-04T02:09:33Z (materials-26.04): Disturber found 2026-10-03 (material-124f1f sheet-2): dropbox.service crash-loops whenever the desktop is down (override DISPLAY=:0; xwayland-satellite panics with no compositor), restarting every ~14 s; one restart put a P5 sample in a per-cell settle window and refused the run. Add it to the held list alongside wali-rotate.timer; familiar-reap.timer (every minute) is worth checking too.
- 2026-10-04T09:02:22Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:580ebea0-d2ea-4477-a438-6499e193b927","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T09:05:00Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:580ebea0-d2ea-4477-a438-6499e193b927","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T09:07:05Z (disturber-hold): Spec drafted: docs/specs/2026-10-04-capture-disturber-hold-design.md (branch disturber-hold). Holds all active user timers, a per-host service list (dropbox.service), desktop caffeine + monitors; journal scan at release; host hold file for kill recovery.
- 2026-10-04T09:10:13Z (disturber-hold): review: spec round 1 — verdict: revise; findings: Critical 1, Important 5, Minor 6; reviewer: claude-code/claude-opus-5-5
- 2026-10-04T09:13:05Z (disturber-hold): review: spec round 2 — verdict: revise; findings: Critical 1, Important 1, Minor 4; reviewer: claude-code/claude-opus-5-5
