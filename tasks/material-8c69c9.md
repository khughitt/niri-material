---
id: material-8c69c9
title: "build(arch): re-pin to the ring of light merge and install"
status: done
priority: 1
size: xs
owner: materials-26.04
created: 2026-09-05T20:56:45Z
updated: 2026-09-06T08:55:20Z
depends: []
tags: [packaging]
---

The installed niri is d47f675a (pre-merge); the ring of light focus response merged at f8bcb34c and its docs and closure land at 5dbe182d. Re-pin packaging/arch/PKGBUILD (pkgver, #commit=, NIRI_BUILD_COMMIT) to 5dbe182d as the earlier build(arch) commits did, build the package with CARGO_TARGET_DIR pinned, have the operator install it (sudo), and verify 'niri msg version' reports 5dbe182d. Until then the filament, ring-color, ring-drift-hz and light-ior keys are rejected by the installed parser, which blocks the Prism pieces.

## Notes

- 2026-09-05T21:30:57Z (materials-26.04): Pinned to 5dbe182d (pkgver 26.04.r278.g5dbe182d); package built clean, workspace tests pass, binary reports 5dbe182d. Awaiting operator sudo pacman -U.
- 2026-09-05T21:34:25Z (materials-26.04): Installed 26.04.r278.g5dbe182d; CLI reports 5dbe182d and validates focus/ring-color/ring-drift-hz/light-ior. 'niri msg version' still shows compositor d47f675a - needs a niri session restart.
- 2026-09-06T08:55:20Z (materials-26.04): Re-pinned packaging/arch/PKGBUILD to 5dbe182d (pkgver 26.04.r278.g5dbe182d, #commit=, NIRI_BUILD_COMMIT). Built clean with CARGO_TARGET_DIR pinned; workspace tests passed in check(). Operator installed; niri msg version reports 5dbe182d for both compositor and CLI. focus/ring-color/ring-drift-hz/light-ior now parse, unblocking the Prism pieces.
