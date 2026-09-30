---
id: material-b58687
title: "upstream-report: adding a task file should not make the divergence report stale"
status: doing
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-28T08:21:42Z
updated: 2026-09-30T09:50:31Z
started: 2026-09-30T09:50:31Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

tools/upstream-report classifies every added path as class A and prints the total in docs/materials/upstream-divergence.md, so each commit that adds tasks/<id>.md (tasks add, feedback) fails the pre-commit --check until just upstream-report is rerun and staged. On 2026-09-27 this blocked four routine chore(tasks) commits and produced a conflict in the generated file on a --no-ff merge. Task records never exist upstream and cannot conflict on a rebase, so leave tasks/ out of the report (skip the prefix in the name-status and numstat reads, with a test). Consider docs/ the same way only if it proves just as noisy.

## Notes

- 2026-09-29T21:33:43Z (materials-26.04): Recurred during the 2026-09-29 scope pass: task records and a notes brief were the only staged changes; pre-commit passed format/clippy and 164 tooling tests, then upstream-report --check refused as stale. Regenerating the report is required to commit the pass.
- 2026-09-29T22:34:10Z (materials-26.04): Recurred during the deferred signal-model scope pass: pre-commit passed clippy and 164 tooling tests (2 skipped), then upstream-report --check refused the task/brief-only staged changes. Refreshing the generated inventory remains necessary; gate feedback filed as ops-50c1ae.
- 2026-09-30T09:50:31Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:5611586f-ab56-46ad-85c1-77404d632d38","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T09:50:31Z (materials-26.04): direct: filter tasks/ in inventory() beside SELF_PATHS so both diff reads stay joined; docs/ left counted — additions there are specs/plans that land with code commits, not routine chores
