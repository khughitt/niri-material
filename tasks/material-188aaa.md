---
id: material-188aaa
title: Hold host disturbers for the length of a quiet capture
status: done
priority: 2
size: s
complexity: mid
process: planned
needs: [quiet]
owner: disturber-hold
created: 2026-09-24T20:09:46Z
updated: 2026-10-05T03:27:26Z
started: 2026-10-04T09:02:22Z
completed: 2026-10-05T03:27:26Z
depends: []
parent: material-2834d7
tags: [performance]
model: claude-opus-5-5
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
- 2026-10-04T12:39:27Z (disturber-hold): resumed
  provenance: {"harness_session":"codex:01a106eb-14e5-7b61-9f6b-e9b21b9511ca","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-04T13:16:05Z (disturber-hold): Quiet-run snapshot: $NIRI_MATERIAL_WORK_ROOT/optic-settling/bin-3125163e/niri-tracy; identity source 3125163e2ca0ee68e845e154f5d96ea41807a61e, SHA256 verified 91b9a0c02b50b3961f925689b0cdf300b3f38d85ea396bdb89f1b5461ba09287. The plan mtime cutoff excludes valid retained snapshots because the spec checkout is newer.
- 2026-10-04T13:29:33Z (disturber-hold): host: wrote ~/.config/niri-material/capture-hold (dropbox.service); remove to stop holding services
- 2026-10-04T13:42:55Z (disturber-hold): review: impl round 1 — verdict: revise; findings: Important 3 | none; reviewer: codex/gpt-6-astra
- 2026-10-04T13:53:41Z (disturber-hold): review: impl round 2 — verdict: revise; findings: Important 1; reviewer: codex/gpt-6-astra
- 2026-10-04T13:56:00Z (disturber-hold): review: impl round 3 — verdict: accept; findings: none; reviewer: codex/gpt-6-astra
- 2026-10-04T13:59:15Z (disturber-hold): Implementation corrective rounds: colliding lock identity, unreadable wake evidence, recovery manifest refresh, and unchanged completed manifests each reproduced RED then GREEN. Full tooling fast gate: 364 tests in 236.093 s, 2 skipped, PASS. Scoped impl round 3 accepted with no findings. Live host validation remains required before closing the parent.
- 2026-10-04T14:03:33Z (disturber-hold): parked (waiting on user, quiet; headless, 25 min): Owner supplies a quiet TTY session with the desktop stopped; agent resumes material-188aaa in .worktrees/disturber-hold and runs these phases sequentially (25 min total): display-dim status, restore any stale dim record, then display-dim set 0.25 when available; T=tools/capture-meta and W=$NIRI_MATERIAL_WORK_ROOT/hold-pilot, with fresh run directories. (1) round trip, 2 min: $T preflight $W/rt-1 --lane dedicated --task material-188aaa --fixture hold-pilot --owner-pid $$; confirm active user timers stopped and dropbox held; $T release $W/rt-1 must exit 0 and show clean. (2) positive control, 2 min: same preflight for $W/pc-1; systemd-run --user --on-active=30s true; sleep 45; release must exit 1 and name timer-added and timer-fired. (3) kill recovery, 1 min: launch a recorded bash child that passes its own $$ to preflight for $W/kill-1, then sleeps 600; after preflight succeeds kill -9 that recorded child, reap it, and within 5 s confirm timers/services restored and hold_end.restore complete with by=guard. Pass T and the run directory as bash positional arguments, not unexported shell variables. (4) dedicated evidence pilot, 20 min: NIRI_BIN=$NIRI_MATERIAL_WORK_ROOT/optic-settling/bin-3125163e/niri-tracy (SHA256 verified 91b9a0c02b50b3961f925689b0cdf300b3f38d85ea396bdb89f1b5461ba09287); CASES='drm-aurora tty-resume screencast', CAPTURE_TASK=material-188aaa, DRM_OUTPUT=DP-1, DRM_MODE=3440x1440@59.999, fresh OUT under $W; docs/materials/scripts/optic-settling-smoke.sh pilot --lane dedicated. A restricted development pilot never reports a passed full pilot; inspect its per-case results and finished capture.json/SHA256SUMS. Pilot before any full run, one run at a time; write a run: note for every attempt, analyze partial evidence before retrying, restore display-dim after the last attempt. Earlier hazard: Dropbox crash-loop is now held by ~/.config/niri-material/capture-hold. Desktop caffeine/monitor wake behavior remains adapter-tested until a separately authorized desktop lane.
  provenance: {"harness_session":"codex:01a106eb-14e5-7b61-9f6b-e9b21b9511ca","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T03:05:22Z (disturber-hold): resumed
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T03:09:06Z (disturber-hold): run: 1 min (est 2, headless); round trip 1; passed: preflight held 13 user timers + dropbox.service, release exit 0, scan clean, restore complete by=release undone=14
- 2026-10-05T03:09:06Z (disturber-hold): run: 2 min (est 2, headless); positive control 2; passed: release exit 1 naming timer-added and timer-fired for the systemd-run transient timer, restore complete
- 2026-10-05T03:09:06Z (disturber-hold): run: 1 min (est 1, headless); kill recovery 1; passed: kill -9 of the owner child, guard restored all 14 items in 0.79 s, hold_end.restore complete by=guard, timers and dropbox back
- 2026-10-05T03:09:06Z (disturber-hold): Observation: restoring with 'systemctl start' re-arms monotonic timers from the restore instant: familiar-reap fired at once, and atoms-recertify (OnActiveSec=5min) moved its next run from 23:37 to 23:11. Harmless to the capture window, but every held run triggers an extra recertify 5 min after release.
- 2026-10-05T03:09:41Z (disturber-hold): run: 0 min (est 20, headless); identity check 0; refused: binary source differs from worktree HEAD — queued bin-3125163e is 93 commits behind disturber-hold HEAD 257f847b (Rust changes); rebuilding a profile-with-tracy snapshot at HEAD per the real-tty plan step 2
- 2026-10-05T03:21:40Z (disturber-hold): run: 5 min (est 20, headless); load wait 3, development pilot 5; failed: verdict invalid — screencast client-1.rgb == client-2.rgb, client-3 differs (probe change landed one sample late; sample-1 frame 390 ms after request vs ~30 ms in the 2026-10-03 passes at 3125163e). Hold itself clean: scan clean, restore complete by=release undone=14, vt-restore not-needed, SHA256SUMS written. Binary bin-257f847b built at HEAD (sha256 54cbcbe9…). Rerunning once to separate flake from the accent-tint changes since 3125163e.
- 2026-10-05T03:26:55Z (disturber-hold): run: 5 min (est 20, headless); load wait 0, development pilot 5; passed: verdict development-passed (drm-aurora, tty-resume edges [0,1,0], screencast client-1≠client-2=client-3, samples ~30 ms after request), hold scan clean, restore complete by=release, vt-restore not-needed, SHA256SUMS verifies. Undimmed: display-dim not installed.
- 2026-10-05T03:27:26Z (disturber-hold): done
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T03:27:26Z (disturber-hold): Live validation passed on the quiet TTY host: round trip clean (13 timers + dropbox held and restored), positive control exit 1 with timer-added/timer-fired, kill recovery by guard in 0.79 s, dedicated-lane development check development-passed with the hold in place (bin-257f847b built at HEAD; first attempt hit a screencast sampling flake, filed material-dcb881). Restore re-arms monotonic timers, filed material-f19f9a.
  provenance: {"harness_session":"claude-code:36e424bd-8c56-41e9-a164-005ea47ccb82","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
