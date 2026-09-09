---
id: material-3ea602
title: Gate cannot run where the hardcoded cargo target-dir does not exist
status: done
priority: 2
size: s
owner: materials-26.04
created: 2026-09-08T16:02:23Z
updated: 2026-09-09T08:55:34Z
depends: []
tags: [robustness, multi-machine, gates]
---

Found on europa 2026-09-08. .cargo/config.toml is tracked and sets target-dir to /mnt/ssd3/niri-material/target, a mount that exists only on titan. On any other machine cargo fmt fails with 'failed to create directory ... Permission denied' before the gate runs, so no commit can be made there. packaging/arch/PKGBUILD already works around this by exporting CARGO_TARGET_DIR=target. Outcome: the target-dir choice does not prevent the repo being built, gated or committed on a second machine.

## Notes

- 2026-09-09T08:55:29Z (materials-26.04): The file is untracked and git-excluded; it reached the second machine through the synced tree, not through git. Fix is the same one node_modules and .worktrees already use: user.com.dropbox.ignored=1 on .cargo and on target, so each machine owns its own target-dir and no build output syncs. Verified in a worktree outside the synced tree, which is the fresh-machine case: no .cargo/config.toml, cargo fmt --all -- --check exits 0, target_directory resolves to the local target.
- 2026-09-09T08:55:34Z (materials-26.04): Marked .cargo and target Dropbox-ignored so the target-dir is per-machine, and documented both in AGENTS.md as artifacts that never leave their machine.
