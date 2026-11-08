---
id: material-e1ef98
title: capture.json records the fixture's exit status at release
status: idea
priority: 2
reviewed: true
reviewed_at: 2026-10-20T21:44:22Z
created: 2026-10-06T13:03:07Z
updated: 2026-10-20T21:44:22Z
depends: [material-232940]
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
---

Why: the run: note's outcome (passed, failed, refused, hung, aborted) must still come from the person or the agent's transcript for a detached or TTY-started run. capture.json now has its duration (material-c44509) but not whether the fixture passed. Every fixture's exit trap holds rc when it calls release; release --exit-status "$rc" would record it as run.exit_status. The record then distinguishes a pass from anything else, though not failed from aborted; scoping decides whether that is enough to be worth the change across every fixture's trap.

## Current scope (2026-10-20)

release still has no exit-status argument. finish_run stamps only the first release; summarize releases before log checks/PASS, and cleanup can fail writing checksums afterward. The newer capture-meta judge checks evidence gates, not final fixture exit. Reuse material-232940 for the smallest truthful outcome boundary, after material-282edc inventories the external lifecycle; preserve unknown outcomes for killed/recovered runs.

Read [ref:brief] for the existing alternatives, constraints and bounded handoff.

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-capture-lifecycle-brief.md — existing scope handoff and follow-up boundaries

## Notes

- 2026-10-06T13:03:07Z (materials-26.04): concerns: material-c44509 extension — the record has the run's length but not its outcome
- 2026-10-06T19:17:33Z (materials-26.04): scope: briefed; early release, repeated-release immutability and checksum work after release prevent treating release success as fixture exit; design final outcome recording separately from hold restoration; brief: docs/notes/2026-10-06-capture-lifecycle-brief.md
- 2026-10-20T21:44:22Z (materials-26.04): scope: briefed; revalidated against 52c14d4d; reused material-232940 and recorded the missing dependency; release still has no exit-status argument. finish_run stamps only the first release; summarize releases before log checks/PASS, and cleanup can fail writing checksums afterward. The newer capture-meta judge checks evidence gates, not final fixture exit. Reuse material-232940 for the smallest truthful outcome boundary, after material-282edc inventories the external lifecycle; preserve unknown outcomes for killed/recovered runs.; brief: docs/notes/2026-10-06-capture-lifecycle-brief.md
