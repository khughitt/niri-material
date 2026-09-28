---
id: material-b58687
title: "upstream-report: adding a task file should not make the divergence report stale"
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-09-28T08:21:42Z
updated: 2026-09-28T08:21:42Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

tools/upstream-report classifies every added path as class A and prints the total in docs/materials/upstream-divergence.md, so each commit that adds tasks/<id>.md (tasks add, feedback) fails the pre-commit --check until just upstream-report is rerun and staged. On 2026-09-27 this blocked four routine chore(tasks) commits and produced a conflict in the generated file on a --no-ff merge. Task records never exist upstream and cannot conflict on a rebase, so leave tasks/ out of the report (skip the prefix in the name-status and numstat reads, with a test). Consider docs/ the same way only if it proves just as noisy.
