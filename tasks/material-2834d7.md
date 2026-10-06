---
id: material-2834d7
title: "Evidence instruments: every check runs on the least host it needs"
status: todo
priority: 1
lane: true
created: 2026-10-02T23:07:45Z
updated: 2026-10-03T16:48:07Z
depends: []
tags: [harness]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
---

Quiet-host captures block whole efforts, while most visual checks need only deterministic pixels. First milestone: shared frozen-time helpers (material-3a17b8) and one visual check migrated onto them (material-22d78f).

Quiet-host captures block whole efforts; most visual checks need only deterministic pixels, not an idle GPU. Done when every evidence check declares its host class (none, headless, quiet), pixel checks run in frozen-clock in-process fixtures (src/tests/ring_pair.rs pattern), and the remaining quiet lanes survive host disturbers. Children: measurement (material-49871a) and capture-lane tooling.

Frozen pixels settle visual behaviour only; timing, power and real lifecycle keep their own checks.
