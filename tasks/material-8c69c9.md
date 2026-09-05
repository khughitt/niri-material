---
id: material-8c69c9
title: "build(arch): re-pin to the ring of light merge and install"
status: todo
priority: 1
size: xs
created: 2026-09-05T20:56:45Z
updated: 2026-09-05T20:56:45Z
depends: []
tags: [packaging]
---

The installed niri is d47f675a (pre-merge); the ring of light focus response merged at f8bcb34c and its docs and closure land at 5dbe182d. Re-pin packaging/arch/PKGBUILD (pkgver, #commit=, NIRI_BUILD_COMMIT) to 5dbe182d as the earlier build(arch) commits did, build the package with CARGO_TARGET_DIR pinned, have the operator install it (sudo), and verify 'niri msg version' reports 5dbe182d. Until then the filament, ring-color, ring-drift-hz and light-ior keys are rejected by the installed parser, which blocks the Prism pieces.
