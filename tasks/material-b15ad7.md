---
id: material-b15ad7
title: "Idle-budget fixture: analyze each case as it lands, fail fast, and offer a one-case pilot"
status: todo
priority: 2
size: s
complexity: mid
process: planned
created: 2026-09-25T09:50:48Z
updated: 2026-09-26T03:14:16Z
depends: []
parent: material-265eb0
tags: [performance]
agent: claude-code/claude-opus-5-5
---

The 2026-09-25 TTY trace spent 58 min to report a failure that every quiet case showed, starting with the first (~2 min in). The same trailing 3 s redraw (material-4be9c3) was already in the retained traces of the two refused desktop attempts on 2026-09-24 (A-move-1 and B-move-1 in trace-20260924T163317 and trace-20260924T165644), but the fixture analyzes only after all 24 cases, so a refused or aborted run never judges the cases it completed. Scope: (1) run the per-observation behavioral gate right after each case's export and stop at the first failure (keeping evidence), unless a full-inventory flag asks for the complete matrix; (2) on a settle refusal or abort, analyze the completed cases before exiting; (3) a pilot mode that runs one quiet case (A-move-1) and one cadence case (C-move-1), about 5 min, to run before committing an hour. Changes the reviewed Task 1 fixture, so planned.

## Notes

- 2026-09-26T03:14:16Z (material-265eb0): From the 2026-09-25 Task 2 review: the analyzer never requires the trace to reach the window end (the heartbeat check tolerates a trace ending up to 1.5 s early); add an explicit trace-end >= window-end gate with the per-case analysis. The watcher that judged cases as they landed during the passing run is a working model: judge each case with trace_observation right after its GPU export.
