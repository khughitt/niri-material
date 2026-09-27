---
id: material-603471
title: Upstream release watch and drift refresh
status: done
priority: 2
size: xs
complexity: low
process: direct
every: 14d
owner: materials-26.04
created: 2026-09-19T13:07:46Z
updated: 2026-09-27T11:31:02Z
started: 2026-09-27T11:30:36Z
completed: 2026-09-27T11:31:02Z
last_done: 2026-09-27T11:31:02Z
depends: []
tags: [upstream]
model: claude-opus-5-5
agent: "claude-code/claude-opus-5[1m]"
---

Fortnightly sweep, run in the main checkout (task-record maintenance and the drift doc only; no worktree needed unless a rebase is filed):

1. `git fetch upstream --tags` — never `--force` (docs/materials/upstream-divergence.md, rebase procedure step 1: forcing moves the local `v26.04` tag and breaks baseline resolution).
2. Compare `git tag -l 'v*' | sort -V | tail -1` against `tag` in docs/materials/upstream-baseline.toml. A newer release tag, or 90 days since the last rebase-log line (baseline established 2026-09-06, so quarterly falls due 2026-12-06), means file the rebase as a planned task following the rebase procedure in the divergence doc and link it here in a note. Do not rebase inside this task.
3. Otherwise `just upstream-report --drift`, read the conflict table, and commit the refreshed doc (`chore(upstream): refresh drift as of <date>`). Acknowledge nothing without a reason.
4. Check the weekly canary is alive: `gh run list -R khughitt/niri-material --workflow upstream-drift.yml --limit 2`. A failed or missing run is a finding to file, not something to absorb here (see material-7963b8 for the first one).

Close with `tasks done` naming the upstream tag checked, the upstream/main commit, and whether a rebase was filed.

## Notes

- 2026-09-27T11:30:36Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T11:31:02Z (materials-26.04): completed; next due 2026-10-11
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T11:31:02Z (materials-26.04): Checked upstream tag v26.04 (latest; matches the baseline) and upstream/main 1f03391e. No rebase filed: no new release, quarterly due 2026-12-06. Drift refreshed today in 67a4f053 (11 conflicting paths, all acknowledged); a rerun changed only the fork hash, so not recommitted. Canary alive: run 36315887493 passed.
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
