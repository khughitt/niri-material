---
id: material-ffede2
title: Record the fast tooling commit route under its own tt target name
status: todo
priority: 2
size: xs
complexity: low
process: direct
created: 2026-10-08T10:09:29Z
updated: 2026-10-10T14:12:52Z
depends: []
tags: [testing]
source: ops-6cff48
agent: claude-code/claude-opus-5-5
---

From ops-6cff48 (decided 2026-10-08 on ops' brief docs/notes/2026-10-08-tt-latency-follow-ups-brief.md): a hook route is its own target. hook-pre-commit currently records both the fast route (NIRI_TOOLING_FAST=1 ... --fast) and the full route under one target, so tt-latency judges one mixed median: at material-5094f2's diagnosis the routes ran 37.5 s and 106.6 s against one 45 s limit, and the fast route crept from its 35 s design target to the limit unseen.

Why: each route needs its own median. The decided shape (ops brief, routes option (a)) is that the hook records its fast route under its own name and ops' latency.toml gains a row for it; the code-commit limit is 45 s by kind of wait. Name: `hook-pre-commit-fast`, after the existing `hook-pre-push-fast` target. The full route keeps `hook-pre-commit`, so its existing limit row and incident history carry over. Evidence (ops runs.jsonl, material, exit 0): since 2026-10-08 the fast route's median is 32.0 s over 58 runs and the full route's 34.1 s over 7, so neither breaches 45 s on the split.

Done when:
- ops' latency.toml `[limits]` has `"hook-pre-commit-fast" = 45`, landed in ops first (mirror ops-95074e: the spec §4 row and its config test), or the new target goes unjudged ("targets outside [limits] are never judged").
- justfile `hook-pre-commit` (the fast recipe, line ~85) records `{{tt}} hook-pre-commit-fast`; `hook-pre-commit-full` still records `hook-pre-commit`. Recipe names and the .githooks/pre-commit dispatch stay as they are.
- tools/test_gates.py `test_recipe_composition_modes_counts_and_shared_hook_target` asserts the two targets (rename it: they are no longer shared); AGENTS.md's Gates line "Both code routes record the same `hook-pre-commit` target" says which route records which.
- One real commit on each code route shows the expected target in runs.jsonl, and `tt-latency check` lists material's `hook-pre-commit-fast` row.

Out of scope: the tighter 35 s declared target for the fast route waits on ops-6cff48 (todo), which defines where a project declares it.

Where to look: justfile lines 84-93; .githooks/pre-commit; tools/test_gates.py ~230-265; docs/specs/2026-10-04-pre-commit-tooling-latency-design.md (it chose the shared target; add a superseded-by line); ops latency.toml and docs/specs/2026-09-29-test-latency-escalation-design.md §4.

## Notes

- 2026-10-10T14:12:51Z (materials-26.04): scope: scoped; the 2026-10-08 decision fixes the shape (fast route as its own target plus an ops limit row at the 45 s code-commit limit); named hook-pre-commit-fast after hook-pre-push-fast; medians since 2026-10-08 (fast 32.0 s, full 34.1 s) breach nothing on the split; todo P2 xs/low/direct
