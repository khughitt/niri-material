---
id: material-ffede2
title: Record the fast tooling commit route under its own tt target name
status: idea
priority: 2
created: 2026-10-08T10:09:29Z
updated: 2026-10-08T10:09:29Z
depends: []
tags: []
source: ops-6cff48
agent: claude-code/claude-opus-5-5
---

From ops-6cff48 (decided 2026-10-08 on ops' brief docs/notes/2026-10-08-tt-latency-follow-ups-brief.md): a hook route is its own target. hook-pre-commit currently records both the fast route (NIRI_TOOLING_FAST=1 ... --fast) and the full route under one target, so tt-latency judges one mixed median: at material-5094f2's diagnosis the routes ran 37.5 s and 106.6 s against one 45 s limit, and the fast route crept from its 35 s design target to the limit unseen.

Wanted: the hook records each route under its own target name (beliefs, ops and tasks already do this for their docs-only route as hook-pre-commit-docs), so each gets its own median. ops then judges it at the code-commit limit (45 s) and, once ops-6cff48 lands, reports creep against a tighter target material declares (35 s for the fast route). The name must be agreed with ops before the hook changes, since only named targets are judged.
