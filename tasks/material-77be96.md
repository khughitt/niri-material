---
id: material-77be96
title: Shared cargo target dir serves stale niri-config artifacts across worktrees
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: material-77be96
created: 2026-09-23T19:47:41Z
updated: 2026-10-03T00:51:21Z
started: 2026-10-03T00:33:02Z
completed: 2026-10-03T00:38:36Z
depends: []
parent: material-2834d7
tags: [tooling]
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

Worktrees share one build.target-dir. After a build of another branch or commit (a baseline worktree, another task's worktree), a build in this worktree can fail with 'no field <x> on ResolvedResponse' for fields present in its own niri-config source. cargo clean -p niri-config -p niri clears it, at the cost of an 18 GiB rebuild. Hit three times on 2026-09-23: twice in the focus-view-tilt smoke (after the 1c978f75 baseline build) and once committing on material-265eb0. Find why cargo reuses the other tree's niri-config (fingerprint or metadata collision between path packages at different roots?) and fix it, or give each worktree its own target dir from just setup. If the cause is the shared-target-dir convention itself, the fix belongs in ops's worktree setup instead.

## Notes

- 2026-09-23T19:50:04Z (materials-26.04): Fourth hit 2026-09-23: the main checkout's pre-commit (materials-26.04) failed with no field ring_beam_speed/idle_after right after a material-265eb0 build; cargo clean -p niri-config -p niri fixed it. So a rebuild of any tree can break the others.
- 2026-09-25T15:28:37Z (materials-26.04): Recurred 2026-09-25 (material-43f88c): a main-checkout commit failed clippy with no field ring_beam_speed / idle_after because another worktree's niri-config build in the shared target dir looked fresh; touching niri-config/src and niri-ipc/src cleared it.
- 2026-10-02T23:47:23Z (materials-26.04): workstreams: moved to the instruments lane at P1; the shared target dir threatens parallel lanes. Main checkout, glass-edges and material-77db8a all set target-dir to the same path (checked 2026-10-02); the configuration persists, the cause of the reported failures is not yet established. Investigate before increasing concurrent builds.
- 2026-10-03T00:33:02Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:33:16Z (materials-26.04): Scoped at start: process direct — the question is bounded (why cargo reuses another tree's workspace-member artifacts) and the fix is either a setup change or a documented convention change; reproduce on a minimal two-root workspace before touching setup.
- 2026-10-03T00:35:51Z (material-77be96): resumed
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:38:36Z (material-77be96): Cause reproduced 2026-10-02 on cargo 1.99 with a two-copy workspace and one CARGO_TARGET_DIR: workspace crates get the same fingerprint dir in both copies (metadata hashes the package path relative to the workspace root), and freshness compares source mtimes with the last build. After copy b built, copy a (older sources) reported Finished with no compile and ran b's binary; touching only a's app then failed with missing field against b's cfg. So the shared target dir itself is the cause, and the silent case (another tree's binary, no error) is worse than the reported compile errors. -Z checksum-freshness would fix it but is still unstable; build.build-dir does not separate workspace crates.
- 2026-10-03T00:38:36Z (material-77be96): Fix: worktrees build into their own target/ (already local storage, no .cargo/config.toml); main keeps /mnt/ssd3/niri-material/target. tools/target-dir-check compares the target dir cargo metadata resolves in every live worktree: a linked worktree that shares fails with the fix named, main lists the sharers on stderr and passes. Runs from new just setup and first in check_cmd. Existing worktrees that share (fix-ci, material-5b3107, material-5b3107-baseline, material-3acc86, material-77db8a, material-80caf4) were left alone: each fails its own check once it carries this change, and moving is deleting its .cargo/config.toml plus one full build. Evidence captured in a sharing worktree could have run another tree's build only when none of its own sources were newer than that build.
- 2026-10-03T00:38:36Z (material-77be96): done
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:38:36Z (material-77be96): tools/target-dir-check with tests, wired into just setup and check_cmd; AGENTS.md: one target dir per checkout, worktrees build into their own target/
  provenance: {"harness_session":"claude-code:f80dd8d7-6772-40fc-bb37-21b14ef04885","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:47:50Z (material-77be96): review: impl round 1 — verdict: revise; findings: Important 4, Minor 7; reviewer: claude-code/claude-opus-5-5
- 2026-10-03T00:51:21Z (material-77be96): Review round 1 fixes: the check now compares cargo's target_directory and build_directory (a shared build.build-dir collides the same way), skips with a note any other checkout that is missing (locked worktrees are never prunable), bare, or whose manifest cargo cannot read, exits 2 on its own failures, and its tests run with an empty CARGO_HOME. Corrections to the earlier notes: a build.build-dir with {workspace-path-hash} does separate intermediates (not the final binaries in a shared target dir); staleness is per crate, so a binary built while sharing can mix another tree's crates with its own, compiling silently when the APIs match; glass-edges also shared. Moved off the shared dir on 2026-10-02 (config replaced by a comment-only .cargo/config.toml, kept because older branches' AGENTS.md makes ops-check require the file): fix-ci, material-5b3107, material-5b3107-baseline, material-77db8a, material-80caf4, glass-edges. material-3acc86 still shares: a live session holds it, so it moves when that session merges this change and its check refuses.
