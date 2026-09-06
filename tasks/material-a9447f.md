---
id: material-a9447f
title: Upstream divergence document and sync strategy
status: done
priority: 2
size: m
owner: docs/upstream-divergence
created: 2026-09-06T11:09:40Z
updated: 2026-09-06T17:51:12Z
depends: []
tags: [docs, upstream, tooling]
spec: docs/specs/2026-09-06-upstream-divergence-design.md
plan: docs/plans/2026-09-06-upstream-divergence.md
---

Permanent-fork posture: upstream only the seams. Produce docs/materials/upstream-divergence.md (prose + generated inventory), tools/upstream-report, a just recipe, and a new scheduled workflow that reports drift against upstream/main. Baseline is pinned by tree hash: patched-26.04~2^{tree} == v26.04^{tree} == 7b010d1b.

## Notes

- 2026-09-06T11:13:23Z (docs/upstream-divergence): Design approved and committed. Upstream is 85 commits past v26.04 (cached tip 3439d4ef, 2026-08-21) with 6 conflicting paths; the 2798/2713 figures ancestry produces are artifacts of the rewritten history.
- 2026-09-06T11:21:21Z (docs/upstream-divergence): Corrected after review: status is awaiting review, not approved. ci.yml already runs fmt (line 233) and clippy (line 218), so the CI gap is only tooling tests + tasks check + the new freshness check; merges 855ac7af and f8bcb34c carry real resolutions that flattening must audit and reproduce; baseline record pins release tree + tag_commit + patched_commit + carried patch-ids and resolves by SHA; acknowledgments are path-only (merge-tree supplies no hunk counts); GitHub default branch must move, not just origin/HEAD.
- 2026-09-06T11:31:54Z (docs/upstream-divergence): Second review round: baseline validation compared commit identity (patched~n == tag_commit), which fails here — patched-26.04~2 is 8ed0da44, v26.04 is aece2b0c, identical tree 7b010d1b. Now compares trees on both sides. Patch retirement uses patch-id --stable as evidence then reads the target tree. Unattributed-conflict fixture corrected to directory-rename (file/directory conflicts do produce entries). Class B is no longer claimed to be the only conflict source; class D retires on target-tag presence.
- 2026-09-06T17:51:12Z (docs/upstream-divergence): took over a live claim held by session 4eff9541-9fa7-47fd-b090-592d47acdd28 (owner docs/upstream-divergence, host titan, pid 739748, worktree /mnt/ssd/Dropbox/niri-material/.worktrees/upstream-divergence, since 2026-09-06T11:09:45Z, age 24087s, live)
- 2026-09-06T17:51:12Z (docs/upstream-divergence): upstream divergence document, generated inventory, drift canary, and rebase procedure landed
