---
id: material-77be96
title: Shared cargo target dir serves stale niri-config artifacts across worktrees
status: idea
priority: 1
created: 2026-09-23T19:47:41Z
updated: 2026-10-02T23:47:24Z
depends: []
parent: material-2834d7
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

Worktrees share one build.target-dir. After a build of another branch or commit (a baseline worktree, another task's worktree), a build in this worktree can fail with 'no field <x> on ResolvedResponse' for fields present in its own niri-config source. cargo clean -p niri-config -p niri clears it, at the cost of an 18 GiB rebuild. Hit three times on 2026-09-23: twice in the focus-view-tilt smoke (after the 1c978f75 baseline build) and once committing on material-265eb0. Find why cargo reuses the other tree's niri-config (fingerprint or metadata collision between path packages at different roots?) and fix it, or give each worktree its own target dir from just setup. If the cause is the shared-target-dir convention itself, the fix belongs in ops's worktree setup instead.

## Notes

- 2026-09-23T19:50:04Z (materials-26.04): Fourth hit 2026-09-23: the main checkout's pre-commit (materials-26.04) failed with no field ring_beam_speed/idle_after right after a material-265eb0 build; cargo clean -p niri-config -p niri fixed it. So a rebuild of any tree can break the others.
- 2026-09-25T15:28:37Z (materials-26.04): Recurred 2026-09-25 (material-43f88c): a main-checkout commit failed clippy with no field ring_beam_speed / idle_after because another worktree's niri-config build in the shared target dir looked fresh; touching niri-config/src and niri-ipc/src cleared it.
- 2026-10-02T23:47:23Z (materials-26.04): workstreams: moved to the instruments lane at P1; the shared target dir threatens parallel lanes. Main checkout, glass-edges and material-77db8a all set target-dir to the same path (checked 2026-10-02); the configuration persists, the cause of the reported failures is not yet established. Investigate before increasing concurrent builds.
