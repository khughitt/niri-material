---
id: material-182b01
title: "Land the rustfmt and FreeBSD CI fix: main CI red since 2026-10-08"
status: done
priority: 1
size: xs
complexity: low
process: direct
owner: fix/ci-format-freebsd
created: 2026-10-11T14:55:19Z
updated: 2026-10-11T17:50:23Z
started: 2026-10-11T15:25:07Z
completed: 2026-10-11T17:10:55Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Every CI push run on materials-26.04 since at least 2026-10-08 (latest 38067275425) fails the rustfmt and freebsd jobs; all others pass. Branch fix/ci-format-freebsd (511f8b81 'fix(ci): align rustfmt checks and repair FreeBSD patch download', worktree .worktrees/fix-ci) carries an unmerged fix that no task tracks. Rebase it onto materials-26.04, verify both jobs go green, merge. Found by the material-603471 sweep on 2026-10-11.

## Notes

- 2026-10-11T15:25:07Z (materials-26.04): started
- 2026-10-11T15:26:52Z (fix/ci-format-freebsd): resumed
- 2026-10-11T15:26:52Z (fix/ci-format-freebsd): took over Unix session (owner materials-26.04, stale)
- 2026-10-11T17:10:55Z (fix/ci-format-freebsd): done
- 2026-10-11T17:10:55Z (fix/ci-format-freebsd): Rebased the unmerged CI fix onto materials-26.04 and extended it: gate and CI now share nightly rustfmt (import grouping), the FreeBSD VM moved to 15.1 to match the ports ABI, the pipewire patch download uses the GitHub mirror with -fL, and libdisplay-info-sys's stale <0.4.0 bound is patched in the VM. Run 38154090397 green, rustfmt and freebsd included.
- 2026-10-11T17:50:23Z (materials-26.04): review: impl round 1 — verdict: accept; findings: none; reviewer: codex
