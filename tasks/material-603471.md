---
id: material-603471
title: Upstream release watch and drift refresh
status: todo
priority: 2
size: xs
complexity: low
process: direct
every: 14d
created: 2026-09-19T13:07:46Z
updated: 2026-09-19T13:07:46Z
depends: []
tags: [upstream]
agent: "claude-code/claude-opus-5[1m]"
---

Fortnightly sweep, run in the main checkout (task-record maintenance and the drift doc only; no worktree needed unless a rebase is filed):

1. `git fetch upstream --tags` — never `--force` (docs/materials/upstream-divergence.md, rebase procedure step 1: forcing moves the local `v26.04` tag and breaks baseline resolution).
2. Compare `git tag -l 'v*' | sort -V | tail -1` against `tag` in docs/materials/upstream-baseline.toml. A newer release tag, or 90 days since the last rebase-log line (baseline established 2026-09-06, so quarterly falls due 2026-12-06), means file the rebase as a planned task following the rebase procedure in the divergence doc and link it here in a note. Do not rebase inside this task.
3. Otherwise `just upstream-report --drift`, read the conflict table, and commit the refreshed doc (`chore(upstream): refresh drift as of <date>`). Acknowledge nothing without a reason.
4. Check the weekly canary is alive: `gh run list -R khughitt/niri-material --workflow upstream-drift.yml --limit 2`. A failed or missing run is a finding to file, not something to absorb here (see material-7963b8 for the first one).

Close with `tasks done` naming the upstream tag checked, the upstream/main commit, and whether a rebase was filed.
