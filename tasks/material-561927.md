---
id: material-561927
title: "Test latency over limit: hook-pre-commit-fast 54.226 s against 45 s"
status: doing
priority: 0
size: m
complexity: high
process: planned
owner: fix/material-561927
created: 2026-10-17T14:00:27Z
updated: 2026-10-17T16:23:16Z
started: 2026-10-17T15:16:53Z
depends: []
tags: [halt, test-latency, testing]
source: "tt-latency:titan:2026-10-17T14:00:24Z"
spec: docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md
---

Filed by tt-latency on titan: the median of successful, uncontended, unwidened runs over the trailing window is over the limit in latency.toml (ops). The material project is halted while this task is open: tasks start refuses new lower-priority work there. Each pair in a `breach:` note below is an obligation on the host it names. Fix the suite, then run `tt-latency verify <this id> --after <remedy timestamp>` on each host named; the task closes when verify exits 0, and the tasks done message carries its output.

Process: planned

## Notes

- 2026-10-17T14:00:32Z (materials-26.04): breach: titan window 2026-10-10T14:00:24Z..2026-10-17T14:00:24Z: hook-pre-commit-fast median 54.226 s, limit 45 s, 28 runs on 6 days
- 2026-10-17T14:41:50Z (material-62347d): halt override: attempted material-591938 by 4e29fe84-ae01-4701-a4ac-da51f5fe6c77: close-out of the in-progress material-62347d (task records only: file Prism + rollout follow-ups, close the goal); no code, no effect on hook latency
- 2026-10-17T15:16:53Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T15:25:06Z (fix/material-561927): resumed
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T15:25:13Z (fix/material-561927): diagnosis: global pre-commit hook hook.identity-check (dotfiles 6b73551, 2026-10-17 06:26) runs git-identity-check (~110 ms: python startup, ops-profile/tt imports, 9 subprocesses) on every fixture git commit; fixture commit 0.128 s vs 0.010 s with hook disabled. Fast tooling 42.0 s with hook vs 23.3 s with a hook-free global config (529 cases, 4 children); test_upstream_report alone 26.1 s vs 7.3 s. Non-tooling check steps ~14 s (clippy 7.6, ops-check 4.1). Fast hook went 34.6 s (01:36Z, 487 cases) -> 54-67 s (11:06Z on, same 487 cases); the hook-pre-commit full route stays 33 s (more children).
- 2026-10-17T15:27:50Z (fix/material-561927): filed ops-3dc696 (friction: git-identity-check ~110 ms per call in unjudged repos); bucket time-balancing follow-up filed as an idea (see spec Rejected alternatives)
- 2026-10-17T15:28:04Z (fix/material-561927): parked (waiting on user, review): Owner: review .worktrees/material-561927/docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md (hermetic git helper for tooling fixtures: GIT_* scrub + GIT_CONFIG_GLOBAL=/dev/null + GIT_CONFIG_NOSYSTEM=1, in coordinator workers and every git-using module). After acceptance, agent writes the plan, implements, runs three staged fast-route hook runs and tt-latency verify.
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T16:14:42Z (fix/material-561927): resumed
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T16:14:48Z (fix/material-561927): review: spec round 1 — verdict: revise; findings: Must fix 2, Should fix 1; reviewer: unverified (relayed by user)
- 2026-10-17T16:14:54Z (fix/material-561927): Spec review round 1: (1) acceptance must be post-merge: branch acceptance with --dry-run, merge, three qualifying post-merge runs, recording verify from the registered checkout (tt-latency refuses with an active worktree claim or newer worktree record); (2) add test_optic_settling (git at 578/626/843) to the isolation inventory with a focused poisoned-env test; (3) marker test needs a control commit proving the hook fires, plus a global core.hooksPath fixture for older git
- 2026-10-17T16:15:57Z (fix/material-561927): spec round 2: two-stage verification (branch --dry-run; merge = remedy timestamp; release claim; three qualifying post-merge runs; recording verify from the registered checkout); test_optic_settling added to isolation with inventory import test and a focused poisoned-GIT_DIR run; suppression tests use a control commit for both config-hook and global core.hooksPath forms
- 2026-10-17T16:16:36Z (fix/material-561927): spec round 2 correction: test_optic_settling's git lookups read the checkout's own HEAD (Path(__file__).parents[1]), not a fixture; focused-run test asserts the checkout HEAD wins over a decoy GIT_DIR
- 2026-10-17T16:16:42Z (fix/material-561927): parked (waiting on user, review): Owner: review round 2 of .worktrees/material-561927/docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md (post-merge two-stage verification; test_optic_settling in the inventory; control commits for config-hook and core.hooksPath forms). After acceptance, agent writes the plan, implements, branch acceptance with --dry-run, merge, post-merge runs and recording verify from the registered checkout.
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T16:23:16Z (fix/material-561927): resumed
  provenance: {"harness_session":"claude-code:1de14f0d-c1b6-47e6-a778-7ce6a1cc8ae1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
