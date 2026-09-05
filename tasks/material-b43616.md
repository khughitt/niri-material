---
id: material-b43616
title: Verification evidence
status: done
priority: 2
size: l
owner: feat/material-signals
created: 2026-09-02T14:26:51Z
updated: 2026-09-05T01:40:56Z
depends: [material-2ecd18]
tags: [signals, plan-step]
plan: docs/plans/2026-09-02-material-signals.md
step: "Task 12: Verification evidence"
---

Outcome: docs/plans/2026-09-02-material-signals.md step "Task 12: Verification evidence" is implemented with its tests passing and committed as that task's final step. Acceptance evidence: the step's listed test command passes and the commit named in the step exists on feat/material-signals.

## Notes

- 2026-09-03T11:39:08Z (feat/material-signals): The experimental inactive shader path was later reverted because its paired GPU measurements did not pass. Balanced capture order then passed three trials with a pooled 4.41% default-path regression. The verified `38d506f2` Arch package is built and staged; task remains open for interactive installation and physical DRM acceptance.
- 2026-09-03T23:46:00Z (feat/material-signals): Verification evidence recorded in docs/materials/2026-09-03-material-signals-smoke.md
- 2026-09-04T00:47:30Z (feat/material-signals): Reopened after final review found invisible signal transitions contributing to render scheduling; corrected source and package acceptance are in progress.
- 2026-09-05T01:40:56Z (feat/material-signals): Corrected source 663202b1 and package recipe aaae42a4 passed final package, visual, and physical acceptance.
