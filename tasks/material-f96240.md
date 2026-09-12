---
id: material-f96240
title: `identity`
status: done
priority: 2
size: s
owner: material-bae9c9
created: 2026-09-12T02:42:50Z
updated: 2026-09-12T04:06:18Z
started: 2026-09-12T04:00:25Z
completed: 2026-09-12T04:02:40Z
depends: [material-1cabb0]
parent: material-bae9c9
tags: [performance, testing]
plan: docs/plans/2026-09-11-material-capture-protocol.md
step: "Task 5: `identity`"
---

## Notes

- 2026-09-12T04:02:40Z (material-bae9c9): Implemented identity provenance with SHA-256 binary/input hashes and dirty source fingerprints, including NUL-safe untracked paths.
- 2026-09-12T04:06:18Z (material-bae9c9): Review fix: fingerprint untracked symlink payloads, including retargets with equal-content referents and broken links.
