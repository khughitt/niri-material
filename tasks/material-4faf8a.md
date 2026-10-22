---
id: material-4faf8a
title: Resolve existing tilde-layout findings before checker rollout
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-22T19:07:12Z
updated: 2026-10-22T19:07:12Z
depends: []
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
