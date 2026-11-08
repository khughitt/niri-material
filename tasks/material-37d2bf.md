---
id: material-37d2bf
title: Fix CI nightly formatting and FreeBSD patch download
status: done
priority: 1
size: s
owner: fix/ci-format-freebsd
created: 2026-09-11T15:28:10Z
updated: 2026-09-11T15:33:31Z
started: 2026-09-11T15:28:25Z
completed: 2026-09-11T15:33:31Z
depends: []
tags: [ci]
source: "https://github.com/khughitt/niri-material/actions/runs/34587756149"
---

Reproduce rustfmt and FreeBSD failures, align local formatting checks with CI, and repair the pinned PipeWire patch download.

## Notes

- 2026-09-11T15:31:12Z (fix/ci-format-freebsd): Stable rustfmt silently ignores nightly import settings; nightly reproduced all 14 CI diffs. FreeBSD cgit returned a 162-byte HTTP 429 HTML page, matching the CI download size; the same pinned patch from the official GitHub mirror applies cleanly to locked PipeWire crates.
- 2026-09-11T15:33:31Z (fix/ci-format-freebsd): Corrected all nightly formatting differences, aligned local gates with nightly, and repaired the pinned FreeBSD patch download with HTTP failure handling. just gate and actionlint passed; updated workflow download and patch application verified locally. Full FreeBSD VM build awaits CI rerun.
