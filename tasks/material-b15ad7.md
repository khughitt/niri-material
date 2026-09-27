---
id: material-b15ad7
title: "Idle-budget fixture: analyze each case as it lands, fail fast, and offer a one-case pilot"
status: doing
priority: 2
size: s
complexity: mid
process: planned
owner: materials-26.04
created: 2026-09-25T09:50:48Z
updated: 2026-09-27T13:54:21Z
started: 2026-09-27T12:25:25Z
depends: []
parent: material-53f873
tags: [performance]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-09-27-idle-budget-fail-fast-design.md
plan: docs/plans/2026-09-27-idle-budget-fail-fast.md
---

The 2026-09-25 TTY trace spent 58 min to report a failure that every quiet case showed, starting with the first (~2 min in). The same trailing 3 s redraw (material-4be9c3) was already in the retained traces of the two refused desktop attempts on 2026-09-24 (A-move-1 and B-move-1 in trace-20260924T163317 and trace-20260924T165644), but the fixture analyzes only after all 24 cases, so a refused or aborted run never judges the cases it completed. Scope: (1) run the per-observation behavioral gate right after each case's export and stop at the first failure (keeping evidence), unless a full-inventory flag asks for the complete matrix; (2) on a settle refusal or abort, analyze the completed cases before exiting; (3) a pilot mode that runs one quiet case (A-move-1) and one cadence case (C-move-1), about 5 min, to run before committing an hour. Changes the reviewed Task 1 fixture, so planned.

## Notes

- 2026-09-26T03:14:16Z (material-265eb0): From the 2026-09-25 Task 2 review: the analyzer never requires the trace to reach the window end (the heartbeat check tolerates a trace ending up to 1.5 s early); add an explicit trace-end >= window-end gate with the per-case analysis. The watcher that judged cases as they landed during the passing run is a working model: judge each case with trace_observation right after its GPU export.
- 2026-09-27T08:53:57Z (material-265eb0): From the 2026-09-27 Task 3 review (power run valid): the seat-manager exemption (experiments 73fbbe0) keys on the command name, so a root 'systemd --user' or a root process that renamed itself would also be exempt, and it does not require our niri to be running. Pin it to pid 1 and systemd-logind's MainPID (systemctl show -p MainPID systemd-logind), recorded in the inventory. Also: the power pilot runner used this evening (sham block 1 only, per-window power_observation) is the model for the power lane's pilot mode.
- 2026-09-27T12:25:25Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:5fc9e346-b9ce-4a5a-be56-3ea1c4973d50","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T13:19:24Z (material-b15ad7): Spec review 1 (5 findings) addressed in 65075362: cleanup reordered (analysis before native_cleanup, which exits; subshell rejected since it cannot wait on the parent's children), stop.json from fail/signal/ERR with first-writer-wins, explicit compositor pid through inventory/interval.json, execution driven by manifest plan rows, output baseline = first verdict with a summary
- 2026-09-27T13:19:24Z (material-b15ad7): Spec review 2 (1 finding) addressed in 2e82d7e0: weston reaped before the checksum, OUT sealed before native_cleanup, ERR trap removed in cleanup; spec approved for planning. Plan docs/plans/2026-09-27-idle-budget-fail-fast.md with 8 step children
- 2026-09-27T13:36:38Z (material-b15ad7): Plan review 1 (4 findings): retried waits via reap/reap_group with a second-signal-during-reap test; fixture cleanup no longer calls the lib's cleanup (lingering socket skipped runtime-dir removal and lock release); verdict marking opt-in in synthetic runs; live runs through fixtures/idle-budget-supervise.sh. Found while prototyping it: tt SIGKILLs its child on Ctrl-C (ops-8fe6c9), so the supervisor signals only the fixture shell; spec cleanup order and live section amended to match
- 2026-09-27T13:46:07Z (material-b15ad7): Plan review 2 (2 findings): reap_group KILLs a group still alive 5 s after TERM and waits again; a group surviving KILL sets UNSEALABLE (no analysis, no sums, lock kept, non-zero exit); writer stubs publish readiness after installing their TERM handler and starters wait for it. Both prototyped in scratch: TERM-ignoring descendant killed and sums held; 12/12 immediate stops logged shutdown
- 2026-09-27T13:54:21Z (material-b15ad7): Plan review 3 (2 findings): teardown no longer blocks in wait; running/members_running poll /proc state and reap our zombies, terminate applies TERM-5s-KILL-5s to single processes and whole groups, leader included (prototyped: deaf leader and deaf single both KILLed, zombies reaped). A survivor of KILL now keeps the fixture alive holding the capture lock until it is gone, since capture-meta reclaims a dead owner's lock; the test attempts a real second acquire_lock
