---
id: material-774c4b
title: Refresh deployed package pin after Prism syntax change
status: done
priority: 1
size: xs
owner: materials-26.04
created: 2026-09-02T22:47:02Z
updated: 2026-09-05T01:35:41Z
depends: []
tags: [packaging]
---

## Notes

- 2026-09-02T22:47:10Z (materials-26.04): Prism emits roughness, but PKGBUILD and installed niri remain pinned to pre-parser commit 52f74f10.
- 2026-09-02T23:06:07Z (materials-26.04): Clean build exposed local .cargo target-dir leakage; PKGBUILD now pins CARGO_TARGET_DIR=target so package() consumes its own artifacts.
- 2026-09-02T23:22:02Z (materials-26.04): Archive built and inspected; installation awaits operator sudo because no cached credential is available.
- 2026-09-05T01:35:41Z (materials-26.04): PKGBUILD refreshed to 88756be6 (parser support for Prism roughness/noise) with CARGO_TARGET_DIR pinned; the installed niri 663202b1 (feat/material-signals, r215) is a descendant of that pin, so deployed parser support is verified without installing this archive
