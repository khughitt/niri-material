---
id: material-a9447f
title: Upstream divergence document and sync strategy
status: doing
priority: 2
size: m
owner: docs/upstream-divergence
created: 2026-09-06T11:09:40Z
updated: 2026-09-06T11:13:23Z
depends: []
tags: [docs, upstream, tooling]
spec: docs/specs/2026-09-06-upstream-divergence-design.md
---

Permanent-fork posture: upstream only the seams. Produce docs/materials/upstream-divergence.md (prose + generated inventory), tools/upstream-report, a just recipe, and a new scheduled workflow that reports drift against upstream/main. Baseline is pinned by tree hash: patched-26.04~2^{tree} == v26.04^{tree} == 7b010d1b.

## Notes

- 2026-09-06T11:13:23Z (docs/upstream-divergence): Design approved and committed. Upstream is 85 commits past v26.04 (cached tip 3439d4ef, 2026-08-21) with 6 conflicting paths; the 2798/2713 figures ancestry produces are artifacts of the rewritten history.
