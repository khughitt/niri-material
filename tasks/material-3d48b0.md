---
id: material-3d48b0
title: Pilot nested measurements with the desktop idle against the pinned TTY reference
status: todo
priority: 1
size: s
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T01:15:14Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §8.2 pilot 2: reference runs, binary cb21ad03… and config hashes pinned there; I_c = [min-w-q, max+w+q], q = 0.001024 ms. Rehearse offline first to compare config hashes; on a mismatch take three fresh TTY reference runs first. Two desktop-idle runs (park --needs idle), the first is the pilot. Pass admits desktop-idle in §5.3 and maps nested measurements to idle in the queue.
