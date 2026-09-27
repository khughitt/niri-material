---
id: material-188aaa
title: Hold host disturbers for the length of a quiet capture
status: todo
priority: 2
size: s
complexity: mid
process: planned
created: 2026-09-24T20:09:46Z
updated: 2026-09-24T20:56:53Z
depends: []
tags: [performance]
agent: claude-code/claude-opus-5-5
---

Quiet-host captures are brittle to scheduled host activity that the readiness preflight cannot see, because it samples load only before the run. Evidence 2026-09-24 (material-cd0e1d): wali-rotate.timer (every 15 min) rotated the wallpaper at 16:06:58, which ran Prism's apply, reloaded the desktop niri, and turned glass back on through the active profile; in the same second the view-tilt smoke's nested probe kitty never mapped and the smoke failed after 13 minutes. Earlier the same day the desktop's own redraws failed the GPU P8/IQR gate until the monitors were powered off, and a monitor wake at ~05:08 would have perturbed a capture had one been running.

Scope: a capture-meta (or fixture-lib) hold that, for the run's duration, pauses a declared list of disturbers (wali-rotate.timer; monitor power via niri power-off-monitors; others found by auditing user timers), records each in capture.json with how to restore it, and restores on exit/TERM/INT; plus a post-run check that flags any disturber that fired mid-run (journal scan over the run window) so a result is never silently taken under one. Related: prism-6d1d72 (the glass toggle does not survive rotation), ops-2da76d (estimate vs actual).

## Notes

- 2026-09-24T20:56:53Z (materials-26.04): Second disturber, 2026-09-24 16:54:05: the desktop idle lock raised the GPU's P8 power floor from 9.9 W to ~11.5 W mid-run, and capture-meta's settle gate refused material-4241c3 at case 12 of 24 (22 min lost). The hold should fix the lock state for the whole run (lock first, or inhibit idle), not just the monitors.
