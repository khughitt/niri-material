---
id: material-4a64bb
title: Glass shader responses
status: done
priority: 2
size: s
owner: feat/material-signals
created: 2026-09-02T14:26:51Z
updated: 2026-09-03T05:01:25Z
depends: [material-9d06b4]
tags: [signals, plan-step]
plan: docs/plans/2026-09-02-material-signals.md
step: "Task 10: Glass shader responses"
---

Outcome: docs/plans/2026-09-02-material-signals.md step "Task 10: Glass shader responses" is implemented with its tests passing and committed as that task's final step. Acceptance evidence: the step's listed test command passes and the commit named in the step exists on feat/material-signals.

## Notes

- 2026-09-03T04:39:52Z (feat/material-signals): Smoke brief needed two execution fixes: separate the nested runtime directory from the Weston socket path and terminate inline KDL child nodes; visual mode then passed.
- 2026-09-03T04:41:06Z (feat/material-signals): feat(material): render ring, rim orbit, and sweep signal responses
- 2026-09-03T05:01:25Z (feat/material-signals): Fix round 1: bounded Niri and Weston shutdown, host-socket leak detection, and lock-guarded free Tracy port selection; nested visual mode passed with five fresh screenshots.
