---
id: material-555b14
title: Triage the five unacknowledged upstream drift conflicts
status: doing
priority: 2
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-27T10:26:42Z
updated: 2026-09-27T11:14:49Z
started: 2026-09-27T11:11:28Z
depends: []
tags: [upstream]
agent: claude-code/claude-opus-5-5
---

First end-to-end drift run (2026-09-27, local simulation of the CI checkout after material-7963b8) against upstream main 1f03391e reports unacknowledged conflicts in niri-config/src/lib.rs, src/layout/scrolling.rs, src/niri.rs, src/tests/client.rs, src/tests/mod.rs. For each: read the conflict (python3 tools/upstream-report --drift, then git merge-tree against upstream/main), decide whether it is a seam we accept and resolve by hand each rebase (acknowledge in docs/materials/upstream-conflicts.toml with a note) or fork work that should move out of the seam. Until then the weekly canary exits 1 and keeps its GitHub issue open, which is its intended signal.

## Notes

- 2026-09-27T11:04:53Z (materials-26.04): CI canary now files these as khughitt/niri-material#1 (run 36314529632); close that issue once all five are acknowledged or resolved.
- 2026-09-27T11:11:28Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T11:14:09Z (materials-26.04): All five acknowledged with per-path reasons (merged 67a4f053); none is a behavioural disagreement: each is both sides editing the same lines. Local --drift against upstream/main 1f03391e exits 0. Remaining: push, rerun the canary, close issue #1.
- 2026-09-27T11:14:49Z (materials-26.04): parked (waiting on user, approval): User approves pushing materials-26.04 (2 commits ahead); then gh workflow run upstream-drift.yml -R khughitt/niri-material --ref materials-26.04, confirm Report drift is skipped (drift exit 0), close issue #1 with a comment naming 67a4f053, tasks done, and remove .worktrees/material-555b14.
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
