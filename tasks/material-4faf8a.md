---
id: material-4faf8a
title: Resolve existing tilde-layout findings before checker rollout
status: doing
priority: 2
size: s
complexity: mid
process: direct
owner: fix/material-4faf8a
created: 2026-10-22T19:07:12Z
updated: 2026-10-29T20:48:22Z
started: 2026-10-29T20:26:32Z
depends: [ops-9fdfe3, ops-6e157a]
tags: []
source: ops-295f8d
agent: codex
---

Why: the final candidate from ops-295f8d reports existing home-relative machine-layout paths that the current checker misses. Resolve these before the stricter vendored copy arrives, so unrelated commits are not blocked. These are scanner findings to classify, not an instruction to discard historical documents.

Done: inspect each listed file in context. Replace accidental checkout/storage paths in shared docs, examples or fixtures with repository-relative paths, project-role references or portable URLs as appropriate. For intentional host configuration, use only the narrow existing layout_allowed mechanism where this repository is host configuration by design; do not exempt ordinary docs wholesale. Preserve historical evidence, attachments and every spec/plan review ledger; follow the task attachment CLI and retention rules instead of editing attached artifacts by hand. If a runtime-path correction needs an unsettled public contract, change that piece to planned before implementation. Run the project-declared checks and the reviewed candidate checker by explicit path, then record every listed file disposition and the clean verdict on this task. Do not repoint shared launchers, vendor the checker, publish, push or activate global guidance in this task.

Where: the current project guide/profile, listed tracked files, existing layout_allowed, and ops-295f8d for candidate branch/path and canonical report. The full sweep reached 36 registered repositories with no unavailable roots; counts here are capped scanner records, not an estimate of all occurrences. No source snippets are copied.

Reported files:
- docs/materials/scripts/noctalia-surfaces.sh: first reported line 320; 1 reported finding(s)
- docs/plans/2026-08-31-material-tasks-migration.md: first reported line 20; 6 reported finding(s)
- docs/plans/2026-09-02-material-noise-saturation.md: first reported line 37; 5 reported finding(s)
- docs/plans/2026-09-27-idle-budget-fail-fast.md: first reported line 26; 3 reported finding(s)
- docs/superpowers/plans/2026-08-29-material-backdrop-blur.md: first reported line 512; 1 reported finding(s)
- tasks/files/material-62347d/noctalia-hue-transition.png: first reported line 22451; 1 reported finding(s)

## Notes

- 2026-10-29T20:26:32Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:0abf3f61-62e6-4a09-a944-9063cc887bf3","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-29T20:26:50Z (fix/material-4faf8a): resumed
  provenance: {"harness_session":"claude-code:0abf3f61-62e6-4a09-a944-9063cc887bf3","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-29T20:35:05Z (fix/material-4faf8a): dispositions: docs/materials/scripts/noctalia-surfaces.sh:320 — reworded (decorative probe-terminal prompt text now "niri-material  materials-26.04"); docs/plans/2026-08-31-material-tasks-migration.md:20,21,36,108,109 — reworded (sibling checkout paths to <niri-material checkout>, <niri-experiments checkout>, <prism checkout> placeholders, matching existing repo usage); docs/plans/2026-09-02-material-noise-saturation.md:37,393,396-398 — reworded (prose to "Prism checkout", git -C commands to "<prism checkout>"); docs/plans/2026-09-27-idle-budget-fail-fast.md:26,106,1910 — reworded (<niri-experiments checkout> in prose and commands); docs/superpowers/plans/2026-08-29-material-backdrop-blur.md:512 — reworded (git -C "<dotfiles checkout>"); tasks/files/material-62347d/noctalia-hue-transition.png:22451 — allowed: exact-file layout_allowed entry in new root .ops-check.toml; the hit is coincidental bytes in compressed PNG data of an LFS task attachment, not a path, and attachments are never hand-edited
- 2026-10-29T20:46:14Z (fix/material-4faf8a): verification: candidate rc=0 at e534d632 (ops-295f8d worktree bin/ops-check, run from worktree top level; also rc=0 at a42b8360); vendored tools/ops-check v14 rc=0; just check rc=0 (hygiene, target-dir-check, nightly rustfmt, clippy, fast tooling 535 tests OK/40 skipped, identity/tasks/upstream-report/package-pin); pre-commit hook passed (fast tooling 535 OK/40 skipped; it regenerated docs/materials/upstream-divergence.md counts for the new root .ops-check.toml); just test full Rust baseline rc=0 (746 passed, 0 failed, 1 ignored; run on e534d632, which touches no Rust); git diff --check clean; no .tt.toml (policy absent); no focused test covers the noctalia-surfaces.sh probe text (string-literal change only)
- 2026-10-29T20:46:20Z (fix/material-4faf8a): parked (waiting on agent): controller: review fix/material-4faf8a at 25074015 in .worktrees/material-4faf8a, then integrate
  provenance: {"harness_session":"claude-code:0abf3f61-62e6-4a09-a944-9063cc887bf3","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-29T20:48:22Z (fix/material-4faf8a): resumed
  provenance: {"harness_session":"claude-code:0abf3f61-62e6-4a09-a944-9063cc887bf3","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
