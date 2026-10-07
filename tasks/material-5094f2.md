---
id: material-5094f2
title: "Test latency over limit: hook-pre-commit 45.425 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: fix/pre-commit-latency-2
created: 2026-10-07T03:05:29Z
updated: 2026-10-07T11:28:41Z
started: 2026-10-07T03:20:52Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-07T03:05:28Z"
spec: docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md
plan: docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-07T03:05:29Z (materials-26.04): breach: titan window 2026-09-30T03:05:28Z..2026-10-07T03:05:28Z: hook-pre-commit median 45.425 s, limit 45 s, 37 runs on 3 days
- 2026-10-07T03:20:52Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:25:53Z (materials-26.04): diagnosis: post-floor window (after 2026-10-05T10:16:51Z) splits by route: fast-tooling 40 runs median 37.5 s, full-tooling 7 runs median 106.6 s, legacy discover 4 runs ~242 s. Idle-host fast route ~35 s: fast tooling 24.1 s (409 cases, 2 children, 34.9 s of case time), clippy 6.4 s, ops-check 2.2 s, fmt 1.2 s, rest <1 s. Live check at 03:21Z reads ok (median 42.256 s, 39 runs) only because tonight's commits were fast-route: borderline, a recurrence of material-cd7782
- 2026-10-07T03:25:53Z (materials-26.04): probe: fast tooling at 2/4/6 children took 24.1/11.4/15.7 s (two runs each, idle host); at 4 and 6 the only failure is test_fast_preserves_native_inventory_and_module_fixtures pinning '2 children'; probe reverted
- 2026-10-07T03:26:03Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T03:26:52Z (fix/pre-commit-latency-2): spec drafted: docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast tooling 2 -> 4 children; full route unchanged; limit unchanged)
- 2026-10-07T03:28:41Z (fix/pre-commit-latency-2): filed ops-6cff48 (gap: no per-route latency target)
- 2026-10-07T03:28:41Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review .worktrees/material-5094f2/docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast tooling 2 -> 4 children). After acceptance, agent writes the plan, implements, runs three staged hook commits and tt-latency verify.
  provenance: {"harness_session":"claude-code:4acbe34b-b4ad-4dd2-a0a5-6d4fd4a4df22","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T09:59:42Z (material-3fcba2): halt override: attempted material-41d052 by a7493e94-8545-442f-9657-208ac2609950: owner resumed it in session with the sheet verdict; branch predates the halt
- 2026-10-07T10:01:24Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T10:01:46Z (fix/pre-commit-latency-2): spec amended: AGENTS.md Gates children count added to the change set
- 2026-10-07T10:01:46Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review .worktrees/material-5094f2/docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast tooling 2 -> 4 children). After acceptance, agent writes the plan, implements, runs three staged hook commits and tt-latency verify.
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T10:10:10Z (fix/pre-commit-latency-2): review: spec round 1 — verdict: revise; findings: P1 1, P2 1; reviewer: codex
- 2026-10-07T10:10:10Z (fix/pre-commit-latency-2): Spec review: preserve a warm full-route acceptance check before deferring that route: the preceding design required full median <=45 s and verified 40.752 s; the incident log reproduces 37 judged runs at 45.425 s, with 29 qualifying fast runs (37.618 s median), 4 qualifying full runs (89.7645 s median), and 4 legacy runs (242.398 s median). Separate cold/contended/legacy evidence before attributing recurrence to fast-suite growth. The proposed cap/assertion-only change also fails CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures: its fixture has only two modules, so observed PID count remains 2 versus expected 4. Confirmed through just test-one using a temporary copy; add four nonempty fixture modules and retain native inventory/module fixture assertions.
- 2026-10-07T10:15:30Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T10:30:51Z (fix/pre-commit-latency-2): spec round 2: full route measured warm at 46.3/46.6 s (check-full); cause is the single remainder process (26.7 s at 10-04, 38.8 s now). Design adds module-bucketed remainder (4 buckets, buckets first) and a 16 pool ceiling for granted budgets (unset stays 10): probe 22.7-23.5 s full tooling. P2: fast fixture grows to four modules; new full-route coordinator test
- 2026-10-07T10:30:51Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review round 2 of .worktrees/material-5094f2/docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md (fast 4 children; full remainder bucketed, pool 16). After acceptance, agent writes the plan, implements, measures check/check-full, runs the staged hook commits and tt-latency verify.
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T10:36:16Z (fix/pre-commit-latency-2): review: spec round 2 — verdict: accept; findings: P3 1; reviewer: codex
- 2026-10-07T10:36:16Z (fix/pre-commit-latency-2): Round 2 design accepted for planning: warm full-route regression is measured; shared four-bucket remainder partition and explicit-grant ceiling 16 address it while unset stays 10. Four nonempty fast fixture modules resolve P2, and independent fast/full acceptance prevents a fast-only mixed-median verdict from masking full regression. P3 clarification for spec line 127 and the plan: budget 1 means one child at a time; one remainder worker and each lifecycle case still launch as separate sequential processes. This does not block planning. Timing log confirms successful 424-case check-full runs near 46.2/46.5 s after a 52.9 s cold run. Review covers the proposed design; coordinator implementation and tests remain to be written.
- 2026-10-07T10:55:12Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T10:57:36Z (fix/pre-commit-latency-2): plan drafted: docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md (3 tasks: material-8a5af7, material-4c455e, material-bea903); spec line 127 clarified (budget 1 = one child at a time, lifecycle cases still separate sequential processes); real-hook acceptance uses private-index staged runs per route, as material-cd7782 did
- 2026-10-07T10:57:36Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review .worktrees/material-5094f2/docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md. After acceptance, agent executes Tasks 1-3 (material-8a5af7, -4c455e, -bea903).
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T11:05:36Z (fix/pre-commit-latency-2): review: plan round 1 — verdict: revise; findings: P1 1, P2 1; reviewer: codex
- 2026-10-07T11:05:36Z (fix/pre-commit-latency-2): Plan review P1 (Task 3 Step 3, lines 297-307): a private index does not isolate the working-tree report. tools/upstream-report --stage writes docs/materials/upstream-divergence.md; the staged newline changes its generated contents (confirmed with a private-index render-only probe), but removing the private index leaves that report changed against the real index. The next sample copies the old report index entry and --stage refuses unstaged report edits. Require a clean report, preserve its exact bytes, and restore it after every hook attempt, including failures/interruption; clean up the private index too. P2 (Task 3 Step 5, lines 330-337): tasks done material-5094f2 is attempted while material-bea903 is still open, so the CLI refuses. Close/stage each step record in its corresponding commit; close acceptance child before the parent, and put final whole-branch review/corrective rounds before final parent completion. Inline execution plus one fresh final reviewer is appropriate. Direct invocation of the real hook on staged code is accepted as timing evidence; make the global constraints say hook runs consistently. The jq query is valid against the current string command field. No repository code or report was changed.
- 2026-10-07T11:06:37Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T11:08:03Z (fix/pre-commit-latency-2): plan round 2: report save/restore per staged hook run (subshell trap on EXIT/INT/TERM, temp index removed), step children started and closed in their own commits, acceptance child then final branch review then parent; constraints say staged hook runs. Pilot of the helper: fast route rc=0 38.2 s (pre-remedy), working tree unchanged
- 2026-10-07T11:08:03Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review round 2 of .worktrees/material-5094f2/docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md. After acceptance, agent executes Tasks 1-3 inline (material-8a5af7, -4c455e, -bea903), then one fresh final reviewer.
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T11:12:15Z (fix/pre-commit-latency-2): review: plan round 2 — verdict: revise; findings: P1 1; reviewer: codex
- 2026-10-07T11:12:15Z (fix/pre-commit-latency-2): Round 2: original P1/P2 resolved. Ran the exact cleanup helper through just test-one with a temporary stub hook under zsh for success, hook failure, and process-group SIGINT: report/index preserved and temporary files removed in all three; no live hook/build was interrupted and no child remains. New P1 at Task 3 Steps 4/6 (lines 345-347, 362): non-dry-run verify records the obligation met before final review; tt-latency.verify then skips met pairs even when --after advances. A synthetic read-only probe with an old verified note, a newer after, and zero new runs returned exit 0, median null, runs null. Thus the proposed corrective rerun cannot verify the new remedy or refresh its floor. Use dry-run verification during acceptance and every corrective round, and perform the first recording verify only after final review/corrections, immediately before parent closure, with the latest remedy timestamp and current results. Filed tooling gap ops-38f668. Inline execution/fresh final reviewer remains appropriate.
- 2026-10-07T11:18:51Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T11:19:09Z (fix/pre-commit-latency-2): plan round 3: acceptance and corrective rounds use tt-latency verify --dry-run; the single recording verify runs after the final review passes, right before parent closure, with the latest remedy timestamp
- 2026-10-07T11:19:09Z (fix/pre-commit-latency-2): parked (waiting on user, review): Owner: review round 3 of .worktrees/material-5094f2/docs/plans/2026-10-07-pre-commit-latency-recurrence-plan.md. After acceptance, agent executes Tasks 1-3 inline (material-8a5af7, -4c455e, -bea903), then one fresh final reviewer, then the recording verify and parent closure.
  provenance: {"harness_session":"claude-code:b4fd4877-edf1-425e-b39a-e543fd97c021","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-07T11:22:23Z (fix/pre-commit-latency-2): review: plan round 3 — verdict: accept; findings: none; reviewer: codex
- 2026-10-07T11:22:23Z (fix/pre-commit-latency-2): Plan round 3 accepted for inline implementation. Global constraints and Task 3 Steps 4/6 use dry-run verification throughout acceptance/corrective rounds; Step 7 performs the first recording verification after final review passes, with the latest remedy timestamp, and closes the parent only on exit 0. The premature satisfaction issue is resolved. Prior report cleanup and child-before-parent fixes remain intact. tasks check exits 0; no additional runnable checks were needed for this sequencing-only revision. Implementer owns Tasks 1-3 and the fresh final branch review as agreed.
- 2026-10-07T11:28:41Z (fix/pre-commit-latency-2): resumed
  provenance: {"harness_session":"codex:01a1161e-2078-71f3-b1ac-74000dd2b0b4","harness_session_source":"CODEX_THREAD_ID"}
