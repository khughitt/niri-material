---
id: material-e2759c
title: capture-meta names the load when it refuses on it
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-09-21T21:04:07Z
updated: 2026-09-21T21:04:07Z
depends: []
tags: [capture]
source: ops-38be00
agent: claude-code/claude-opus-5
---

When the quiet preflight refuses on cpu_busy_pct or load1, run `host-load --json --section load -n 5` (ops's bin/host-load, on ~/.local/bin after `just install` there) and write its output beside the refusal in capture.json, so the artifact says which process carried the load, its age, whether it holds a terminal, and whether its scope is dead — instead of 'wait'. The tool absent on PATH is a CannotRun with the install hint, not a silent skip. Follow-up of ops-38be00, where the 2026-09-19 refusals (loads 5.43, 6.65, 2.12) were one stray bun from a mind6 TUI smoke run.
