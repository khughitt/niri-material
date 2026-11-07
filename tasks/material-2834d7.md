---
id: material-2834d7
title: "Evidence instruments: every check runs on the least host it needs"
status: todo
priority: 1
lane: true
created: 2026-10-02T23:07:45Z
updated: 2026-11-07T22:01:10Z
depends: []
tags: [harness]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
origin: mixed
---

Quiet-host captures block whole efforts, while most visual checks need only deterministic pixels. First milestone: shared frozen-time helpers (material-3a17b8) and one visual check migrated onto them (material-22d78f).

Quiet-host captures block whole efforts; most visual checks need only deterministic pixels, not an idle GPU. Done when every evidence check declares its host class (none, headless, quiet), pixel checks run in frozen-clock in-process fixtures (src/tests/ring_pair.rs pattern), and the remaining quiet lanes survive host disturbers. Children: measurement (material-49871a) and capture-lane tooling.

Frozen pixels settle visual behaviour only; timing, power and real lifecycle keep their own checks.

## Notes

- 2026-11-07T22:01:10Z (materials-26.04): origin backfill (tasks-0dfdf9): stage 1 unavailable (created 2026-10-02 before tasks-053e89); traced via claude-code:83185704-f448-42a6-ac20-8857595bdb59 L9 user prompt asking to organize material into parallel core goals combined with agent 2026-10-02-workstreams-brief.md -> origin=mixed
