---
id: material-e2759c
title: capture-meta names the load when it refuses on it
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-21T21:04:07Z
updated: 2026-09-27T12:10:42Z
started: 2026-09-27T12:06:36Z
completed: 2026-09-27T12:10:42Z
depends: []
tags: [capture]
source: ops-38be00
agent: claude-code/claude-opus-5
---

When the quiet preflight refuses on cpu_busy_pct or load1, run `host-load --json --section load -n 5` (ops's bin/host-load, on ~/.local/bin after `just install` there) and write its output beside the refusal in capture.json, so the artifact says which process carried the load, its age, whether it holds a terminal, and whether its scope is dead — instead of 'wait'. The tool absent on PATH is a CannotRun with the install hint, not a silent skip. Follow-up of ops-38be00, where the 2026-09-19 refusals (loads 5.43, 6.65, 2.12) were one stray bun from a mind6 TUI smoke run.

## Notes

- 2026-09-27T12:06:36Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:968962ae-5823-4e8c-b3dd-890b43741440","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:10:42Z (material-e2759c): done
  provenance: {"harness_session":"claude-code:968962ae-5823-4e8c-b3dd-890b43741440","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:10:42Z (material-e2759c): A preflight refused on cpu_busy_pct or load1 runs host-load --json --section load -n 5, stores the report as preflight.host_load, and names each process (cpu share, age, tty, dead scope) in the refusal and in show. host-load absent on PATH is exit 2 with the install hint before sampling; a host-load failure after a refusal is exit 2 naming the refusal. Verified live: a forced load1 refusal named the top five with host-load's own process dropped.
  provenance: {"harness_session":"claude-code:968962ae-5823-4e8c-b3dd-890b43741440","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
