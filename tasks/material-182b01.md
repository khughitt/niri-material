---
id: material-182b01
title: "Land the rustfmt and FreeBSD CI fix: main CI red since 2026-10-08"
status: doing
priority: 1
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-11T14:55:19Z
updated: 2026-10-11T15:25:07Z
started: 2026-10-11T15:25:07Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Every CI push run on materials-26.04 since at least 2026-10-08 (latest 38067275425) fails the rustfmt and freebsd jobs; all others pass. Branch fix/ci-format-freebsd (511f8b81 'fix(ci): align rustfmt checks and repair FreeBSD patch download', worktree .worktrees/fix-ci) carries an unmerged fix that no task tracks. Rebase it onto materials-26.04, verify both jobs go green, merge. Found by the material-603471 sweep on 2026-10-11.

## Notes

- 2026-10-11T15:25:07Z (materials-26.04): started
