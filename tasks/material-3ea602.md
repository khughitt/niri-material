---
id: material-3ea602
title: Gate cannot run where the hardcoded cargo target-dir does not exist
status: todo
priority: 2
size: s
created: 2026-09-08T16:02:23Z
updated: 2026-09-08T16:02:23Z
depends: []
tags: [robustness, multi-machine, gates]
---

Found on europa 2026-09-08. .cargo/config.toml is tracked and sets target-dir to /mnt/ssd3/niri-material/target, a mount that exists only on titan. On any other machine cargo fmt fails with 'failed to create directory ... Permission denied' before the gate runs, so no commit can be made there. packaging/arch/PKGBUILD already works around this by exporting CARGO_TARGET_DIR=target. Outcome: the target-dir choice does not prevent the repo being built, gated or committed on a second machine.
