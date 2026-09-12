---
id: material-aea722
title: Finish manifest and its test are locale-dependent
status: done
priority: 2
size: xs
complexity: low
owner: materials-26.04
created: 2026-09-12T15:41:32Z
updated: 2026-09-12T15:43:37Z
started: 2026-09-12T15:41:58Z
completed: 2026-09-12T15:43:37Z
depends: []
tags: [testing]
---

test_finish_hashes_every_file_but_the_manifest normalizes SHA256SUMS with the host locale's sort but asserts C-collation order; on en_US.UTF-8 that reorders ./c.tracy after ./capture.json. finish() writes the manifest with the same locale-dependent sort -z, so its line order is not reproducible across machines. Force LC_ALL=C in both.

## Notes

- 2026-09-12T15:43:37Z (materials-26.04): finish writes SHA256SUMS with LC_ALL=C sort -z, and the optic smoke test normalizes with LC_ALL=C sort, so manifest line order no longer varies with the host locale
