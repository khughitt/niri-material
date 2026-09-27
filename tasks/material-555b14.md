---
id: material-555b14
title: Triage the five unacknowledged upstream drift conflicts
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-27T10:26:42Z
updated: 2026-09-27T10:26:42Z
depends: []
tags: [upstream]
agent: claude-code/claude-opus-5-5
---

First end-to-end drift run (2026-09-27, local simulation of the CI checkout after material-7963b8) against upstream main 1f03391e reports unacknowledged conflicts in niri-config/src/lib.rs, src/layout/scrolling.rs, src/niri.rs, src/tests/client.rs, src/tests/mod.rs. For each: read the conflict (python3 tools/upstream-report --drift, then git merge-tree against upstream/main), decide whether it is a seam we accept and resolve by hand each rebase (acknowledge in docs/materials/upstream-conflicts.toml with a note) or fork work that should move out of the seam. Until then the weekly canary exits 1 and keeps its GitHub issue open, which is its intended signal.
