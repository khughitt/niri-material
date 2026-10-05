---
id: material-f19f9a
title: Disturber hold restore re-arms monotonic timers
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-05T03:27:25Z
updated: 2026-10-05T03:27:26Z
depends: []
parent: material-2834d7
tags: [performance]
agent: claude-code/claude-opus-5-5
---

capture-meta restores held timers with systemctl start, which re-arms monotonic triggers from the restore instant: on 2026-10-05 familiar-reap fired at once and atoms-recertify (OnActiveSec=5min, OnUnitInactiveSec=6h) moved from 23:37 to 23:11, so every held run adds an extra recertify 5 min after release. Outside the capture window, so it never disturbs a run; decide whether to accept it (document in capture-host-setup) or restore differently.

## Notes

- 2026-10-05T03:27:26Z (disturber-hold): concerns: material-188aaa defect — restore re-arms monotonic timers, so a held run triggers off-schedule timer runs after release
