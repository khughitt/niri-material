---
id: material-e1ef98
title: capture.json records the fixture's exit status at release
status: idea
priority: 2
created: 2026-10-06T13:03:07Z
updated: 2026-10-06T13:03:08Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
---

Why: the run: note's outcome (passed, failed, refused, hung, aborted) must still come from the person or the agent's transcript for a detached or TTY-started run. capture.json now has its duration (material-c44509) but not whether the fixture passed. Every fixture's exit trap holds rc when it calls release; release --exit-status "$rc" would record it as run.exit_status. The record then distinguishes a pass from anything else, though not failed from aborted; scoping decides whether that is enough to be worth the change across every fixture's trap.

## Notes

- 2026-10-06T13:03:07Z (materials-26.04): concerns: material-c44509 extension — the record has the run's length but not its outcome
