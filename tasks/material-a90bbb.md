---
id: material-a90bbb
title: Confirm the upstream drift canary recovers after its 2026-10-05 tooling-test failure
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-11T14:55:19Z
updated: 2026-10-11T14:55:19Z
depends: []
tags: [upstream]
agent: claude-code/claude-opus-5-5
---

Scheduled run 37326837429 (2026-10-05, at 81165c50) failed its Tooling tests step: test_gates errors (TypeError in focused-recipe tests) and route-name assertions (hook-pre-push vs hook-pre-push-fast, hook-pre-commit vs hook-pre-commit-docs). The canary's exact command (python3 -m unittest discover -s tools, NIRI_TOOLING_FAST unset) passes on 9fb19a20 (466 tests, 2 skips), so the failure was a test/route mismatch at that commit, since fixed. Check the next scheduled run (2026-10-12) passes; if it fails, attribute it. Found by material-603471.
