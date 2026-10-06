---
id: material-76fd7e
title: Clear pyright's optional-typing errors in tools/capture_hold.py
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-06T15:12:20Z
updated: 2026-10-06T15:19:39Z
started: 2026-10-06T15:12:25Z
completed: 2026-10-06T15:19:39Z
depends: []
parent: material-2834d7
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

pyright reports 14 reportOptional* errors in tools/capture_hold.py (read_hold returning None flows into subscripts; hold/attempt dicts typed dict | None). Fix annotations or add explicit guards so pyright is clean on the file, tests keep passing. Pyright is not part of any gate; the 200-error census across the rest of tools/ is out of scope here.

## Notes

- 2026-10-06T15:12:25Z (materials-26.04): started
- 2026-10-06T15:18:53Z (material-76fd7e): review: impl round 1 — verdict: accept; findings: minor 2; reviewer: claude/claude-opus-5-5 (subagent); isinstance-first guard applied, python>=3.10 floor from TypeGuard accepted (tooling already targets builtin generics)
- 2026-10-06T15:19:39Z (material-76fd7e): done
- 2026-10-06T15:19:39Z (material-76fd7e): names_run is now a TypeGuard[dict] (isinstance-first body), so the callers that raise or return when the hold file does not name the run see a non-None dict; pyright is 0 errors on tools/capture_hold.py, 138 capture-hold/meta tests pass, full tooling 418 OK
