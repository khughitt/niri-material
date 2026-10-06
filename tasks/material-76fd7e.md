---
id: material-76fd7e
title: Clear pyright's optional-typing errors in tools/capture_hold.py
status: doing
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-06T15:12:20Z
updated: 2026-10-06T15:12:25Z
started: 2026-10-06T15:12:25Z
depends: []
parent: material-2834d7
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

pyright reports 14 reportOptional* errors in tools/capture_hold.py (read_hold returning None flows into subscripts; hold/attempt dicts typed dict | None). Fix annotations or add explicit guards so pyright is clean on the file, tests keep passing. Pyright is not part of any gate; the 200-error census across the rest of tools/ is out of scope here.

## Notes

- 2026-10-06T15:12:25Z (materials-26.04): started
