---
id: material-032d10
title: capture-meta stamps run.finished when a guard or later preflight recovers a killed run
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-06T13:03:07Z
updated: 2026-10-06T13:03:08Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
---

Why: run.finished is written by the first release, which only an exit trap calls. A fixture killed outright (SIGKILL, OOM, a closed scope) never releases; its hold is restored by the guard unit, the next preflight's recover_stale, or capture-meta restore, and capture.json keeps no end time, which is the artifact-only case material-c44509 set out to close.

Done when restore_run, called by guard, next-preflight or hand, writes run.finished and run.duration_s when absent, with run.finished_by naming who stamped it (release otherwise), so a recovered end time (the moment recovery noticed, not the moment of death) is never mistaken for the fixture's own. Tests in tools/test_capture_meta.py LifecycleTests (guard after a kill mid-run, next-preflight recovery); spec §3 timing paragraph updated.

Where to look: tools/capture-meta restore_run's record() callback and finish_run.

## Notes

- 2026-10-06T13:03:07Z (materials-26.04): concerns: material-c44509 extension — killed runs recovered by the guard or a later preflight still get no end time
