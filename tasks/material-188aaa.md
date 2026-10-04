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
updated: 2026-10-04T12:37:20Z
started: 2026-10-04T09:02:22Z
depends: []
parent: material-2834d7
tags: [performance]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-04-capture-disturber-hold-design.md
plan: docs/plans/2026-10-04-capture-disturber-hold.md
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
- 2026-10-04T09:13:36Z (disturber-hold): review: spec round 3 — verdict: accept; findings: Minor 1; reviewer: claude-code/claude-opus-5-5
- 2026-10-04T09:13:36Z (disturber-hold): Implementation note from spec review: recovery takes guarded() only after acquire_lock returns (flock is per open file; nesting in-process deadlocks).
- 2026-10-04T09:13:39Z (disturber-hold): parked (waiting on user, review): Owner reviews docs/specs/2026-10-04-capture-disturber-hold-design.md (branch disturber-hold, .worktrees/disturber-hold); on approval write the plan, implement, and stage tonight's TTY quiet run (round trip, positive control, kill recovery, optic-settling dedicated pilot)
  provenance: {"harness_session":"claude-code:580ebea0-d2ea-4477-a438-6499e193b927","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T09:50:10Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:580ebea0-d2ea-4477-a438-6499e193b927","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T09:50:10Z (disturber-hold): review: spec round 4 — verdict: revise; findings: P1 3, P2 1; reviewer: human
- 2026-10-04T09:52:06Z (disturber-hold): Round 4 applied (b2d65182): guard started+confirmed before first host change; guard watches connected connectors' sysfs dpms, wake = monitor-woke; DPMS-off verified at hold or preflight fails; service activation = Starting or unpaired Started/Finished/failed; release runs before SHA256SUMS in glass-optic-smoke-lib finish and optic-settling exit, second release idempotent.
- 2026-10-04T09:54:10Z (disturber-hold): review: spec round 5 — verdict: revise; findings: Important 3, Minor 1; reviewer: claude-code/claude-opus-5-5
- 2026-10-04T09:54:48Z (disturber-hold): review: spec round 6 — verdict: accept; findings: Minor 2; reviewer: claude-code/claude-opus-5-5
- 2026-10-04T09:54:48Z (disturber-hold): Plan items from spec round 6: hold_end rewrite on restore-failed retry needs its own update path (write_section is write-once); finish writes SHA256SUMS before failing on a nonzero release.
- 2026-10-04T09:54:51Z (disturber-hold): parked (waiting on user, review): Owner reviews docs/specs/2026-10-04-capture-disturber-hold-design.md (branch disturber-hold, .worktrees/disturber-hold; agent review accepted round 6). Optional 5 s check: power-off-monitors then read /sys/class/drm/card1-DP-1/enabled. On approval: plan, implement, stage tonight's TTY run.
  provenance: {"harness_session":"claude-code:580ebea0-d2ea-4477-a438-6499e193b927","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:25:56Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:25:59Z (disturber-hold): Owner check 2026-10-04: after niri power-off-monitors, /sys/class/drm/card1-DP-1/dpms reads Off (round-5 claim that dpms does not follow was wrong for connected connectors). Spec watches dpms again.
- 2026-10-04T10:26:02Z (disturber-hold): parked (waiting on user, review): Owner reviews docs/specs/2026-10-04-capture-disturber-hold-design.md (branch disturber-hold); on approval write the plan, implement, stage tonight's TTY run
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:37:21Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:37:21Z (disturber-hold): review: spec round 7 — verdict: revise; findings: P1 2, P2 2; reviewer: human
- 2026-10-04T10:38:13Z (disturber-hold): Round 7 applied (b4873bd6+): USER_INVOCATION_ID for user-manager entries; preflight rollback failure keeps hold file/guard/lock; hold_end split into write-once scan (until_us fixed at first restore) and updatable restore with attempts; guard/hand/next-preflight complete a failed record.
- 2026-10-04T10:38:16Z (disturber-hold): parked (waiting on user, review): Owner reviews docs/specs/2026-10-04-capture-disturber-hold-design.md after round 7 fixes (branch disturber-hold); on approval write the plan, implement, stage tonight's TTY run
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:51:14Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T10:51:14Z (disturber-hold): review: spec round 8 — verdict: accept; findings: minor 1; reviewer: human
- 2026-10-04T11:03:04Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T11:03:16Z (disturber-hold): parked (waiting on user, review): Owner reviews docs/plans/2026-10-04-capture-disturber-hold.md (branch disturber-hold) and picks subagent-driven or native execution; then implement steps material-810dec..f52898 and stage tonight's TTY run
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T11:28:49Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T11:28:49Z (disturber-hold): review: plan round 1 — verdict: revise; findings: P1 3, P2 4; reviewer: human
- 2026-10-04T11:31:24Z (disturber-hold): Plan round 1 applied: restore_transaction (record before hold file removal, atomic save_record), apply refuses restoring/dead-owner holds under lock, lit_connectors fails closed, guard polls under lock + scan filters wakes to window, smoke-lib SHA256SUMS written in cleanup after last release, explicit sys.path in test_capture_hold, command timeouts and attempt-limit wording.
- 2026-10-04T11:31:27Z (disturber-hold): parked (waiting on user, review): Owner re-reviews docs/plans/2026-10-04-capture-disturber-hold.md after round-1 fixes (branch disturber-hold); then native execution of material-810dec..f52898 and a whole-branch review
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:20:36Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:20:36Z (disturber-hold): review: plan round 2 — verdict: revise; findings: P1 3; reviewer: human
- 2026-10-04T12:22:30Z (disturber-hold): Plan round 2 applied: hold file items never shrink (held tracks remainder) so a retried scan sees the same input; release of a complete record finishes leftovers (finalize, guard stop, lock) without touching the record; record_lock (flock on run dir) serializes every capture.json read-modify-write, order guarded() then record_lock.
- 2026-10-04T12:22:33Z (disturber-hold): parked (waiting on user, review): Owner re-reviews docs/plans/2026-10-04-capture-disturber-hold.md after round-2 fixes (branch disturber-hold); then native execution of material-810dec..f52898 and a whole-branch review
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:29:24Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:29:24Z (disturber-hold): review: plan round 3 — verdict: revise; findings: P1 1; reviewer: human
- 2026-10-04T12:29:36Z (disturber-hold): Plan round 3 applied: hold_end.cleanup {guard, owner_pid} written with the first restore record; finish_leftovers uses it; cleanup-interruption test also starts from a record with only run.
- 2026-10-04T12:29:39Z (disturber-hold): parked (waiting on user, review): Owner re-reviews docs/plans/2026-10-04-capture-disturber-hold.md after round-3 fix (branch disturber-hold); then native execution of material-810dec..f52898 and a whole-branch review
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:37:17Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-04T12:37:17Z (disturber-hold): review: plan round 4 — verdict: accept; findings: none; reviewer: human
- 2026-10-04T12:37:20Z (disturber-hold): parked (waiting on user): Plan approved (dae52058). On the owner's go: native execution in .worktrees/disturber-hold of steps material-810dec, 4c12cd, 1c38d5, e04d51, 616b5d, 46bebd, f52898 in order, then a fresh whole-branch review, then stage tonight's TTY quiet run from Task 7's runbook
  provenance: {"harness_session":"claude-code:7afe7f97-b3a8-4bf7-9228-6886337cb0ba","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
