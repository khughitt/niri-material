---
id: material-07a816
title: Test + CI iteration cost audit
status: doing
priority: 2
size: m
complexity: mid
owner: materials-26.04
created: 2026-09-04T21:44:54Z
updated: 2026-09-28T08:35:07Z
depends: [ops-31f038]
tags: [testing]
---

Piece of ops-65837b (the cross-project audit in the ops hub). 1. Measure: full-suite wall time, and roughly how often agent full-suite runs fail here. 2. Add a fast or affected-only test target for the inner loop and point AGENTS.md at it; keep the full suite for commit and CI. 3. Use a quiet reporter so test output does not flood agent context. 4. Fix suite hygiene: sleeps, real network, unshared fixtures. Record the before and after numbers in a note on this task.

## Notes

- 2026-09-05T02:38:40Z (materials-26.04): design: ops docs/specs/2026-09-04-test-ci-audit-design.md; follow §5: (1) justfile + vendored tools/tt, route existing hooks, CI, and documented test commands through it, verify a line lands under each agent; (2) after a week of runs, add a note reading 'baseline <date>: <tt-report --project numbers>'; (3) gates to §4.6, AGENTS.md line, hygiene; (4) close with before/after numbers
- 2026-09-05T08:25:01Z (audit/front-door): step 1 in worktree .worktrees/audit-front-door (branch audit/front-door). Findings: the design's 'material checked-in hook scripts' do not exist; only git-lfs hooks in .git/hooks, so .githooks/ carries the template hooks plus the LFS hooks that core.hooksPath would bypass. No AGENTS.md existed; one is added with the just recipes. Stable rustfmt passes --check here (the unstable options only warn), so check_cmd uses stable fmt. cargo-nextest installed user-locally via cargo install for the fast target; tools/test-affected selects packages changed against HEAD plus dependents from cargo metadata, excludes niri-visual-tests as CI does, and on an empty selection prints a nextest-shaped Summary line so tt records tests: 0. CI test jobs route through just ci-test / ci-test-release via extractions/setup-just@v4.
- 2026-09-05T08:30:49Z (audit/front-door): step 1 verified: just test-fast recorded once under claude (shared log), once under codex (codex workspace-write sandbox cannot write the shared log or the main checkout, so the line landed in the worktree .tt/runs.jsonl and tt-report harvested it), once by hand with the agent vars unset; agent and session fields correct in all three. Numbers on this host: full suite cold in a fresh worktree 461s / 371 tests; warm 16.7s; test-fast after a niri-config edit 15.7s / 366 tests (niri + niri-config via nextest); test-fast on a tooling-only change 1.6s / tests 0; just check cold 71s (clippy dominates; stable rustfmt prints 32 unstable-option warnings per run; hygiene for step 3). One warm-suite line 'error in client communication' printed by a passing test. Affected set is nearly always niri + niri-config because niri is the root and depends on both libs; the empty case (docs, tasks, tooling) is where the fast target saves time.
- 2026-09-07T00:03:24Z (materials-26.04): Hygiene observed 2026-09-06 while committing through the hook: cargo clippy --all --all-targets prints 13 warnings for the niri lib test target (unused import MergeWith at src/layout/tests.rs:3, needless_late_init in src/layout/mod.rs, field_reassign_with_default in render_helpers/material.rs and signal.rs, for_kv_map in two protocols, an unused gtk prelude in niri-visual-tests). They are pre-existing and do not fail check, but they add about 150 lines of noise to every hook run; candidates for step 3.
- 2026-09-12T19:24:32Z (materials-26.04): Complexity mid: The linked ops audit design fixes the workflow and measurement contract; justfile, timing wrapper, hooks, and affected selector already exist. Remaining baseline interpretation, evidence-led hygiene, guidance, and before/after reporting are bounded investigation.
- 2026-09-28T08:34:21Z (materials-26.04): baseline 2026-09-28 (tt-report --project material --since 2026-09-05 --until 2026-09-24; 19 active days, before step 3): test 97 runs median 19.5s p90 93.6s fail 0.05, 74 min total; test-fast 281 runs median 3.3s p90 62.4s fail 0.39 empty 14, 201 min total; check 131 runs median 15.8s p90 37.0s fail 0.24, 38 min total; hook-pre-commit 363 runs median 19.0s p90 33.2s fail 0.24, 125 min total; hook-pre-push 28 runs median 44.2s p90 93.0s fail 0.07, 27 min total; front-door total 7.72 h (24 min per active day), ad-hoc targets 19 min; fast/full by agents 2.90; bypasses 94
- 2026-09-28T08:35:07Z (materials-26.04): Baseline window 2026-09-05..09-24 closes the day before ops host-budget worker sizing (09-25), the first timing change after step 1; no step-3 change had landed. Front-door total counts wrapper seconds of test, test-fast, check and both hooks. Next: step 3 (gates to ops design §4.6, the AGENTS.md inner-loop line, hygiene the numbers point at), then an after-window read with tt-report --since/--until.
