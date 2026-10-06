---
id: material-f19f9a
title: Disturber hold restore re-arms monotonic timers
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-05T03:27:25Z
updated: 2026-10-06T14:13:30Z
started: 2026-10-06T14:06:37Z
completed: 2026-10-06T14:09:33Z
depends: []
parent: material-2834d7
tags: [performance]
agent: claude-code/claude-opus-5-5
---

capture-meta restores held timers with systemctl start, which re-arms monotonic triggers from the restore instant: on 2026-10-05 familiar-reap fired at once and atoms-recertify (OnActiveSec=5min, OnUnitInactiveSec=6h) moved from 23:37 to 23:11, so every held run adds an extra recertify 5 min after release. Outside the capture window, so it never disturbs a run; decide whether to accept it (document in capture-host-setup) or restore differently.

## Notes

- 2026-10-05T03:27:26Z (disturber-hold): concerns: material-188aaa defect — restore re-arms monotonic timers, so a held run triggers off-schedule timer runs after release
- 2026-10-05T09:10:26Z (materials-26.04): correction: the concerns: note above was written before material-188aaa closed, so by the note rules it is a review finding of material-188aaa's live validation, not a concern against closed work; outcome measures should not count it.
- 2026-10-06T14:06:37Z (materials-26.04): started
- 2026-10-06T14:09:22Z (material-f19f9a): decision: accept and document — restore is not schedule-preserving (systemctl start re-anchors OnActiveSec; no writable next-elapse, masking the services would fail the journal scan); spec §3.1 corrected and capture-host-setup records the behaviour
- 2026-10-06T14:09:33Z (material-f19f9a): done
- 2026-10-06T14:09:33Z (material-f19f9a): decided: accept and document the re-arm — spec §3.1 now states restore is not schedule-preserving (OnActiveSec re-anchors at restore, elapsed OnBootSec fires at once, OnUnit*Sec/OnCalendar keep their anchors, no writable next-elapse exists) and capture-host-setup records the behaviour with the 2026-10-05 observation
- 2026-10-06T14:13:30Z (material-f19f9a): review: impl round 1 — verdict: accept; findings: minor 2; reviewer: claude/claude-opus-5-5 (subagent)
