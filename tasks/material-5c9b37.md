---
id: material-5c9b37
title: Pin module_buckets dropping empty buckets with a coordinator test
status: todo
priority: 4
size: xs
complexity: low
process: direct
created: 2026-10-08T02:16:30Z
updated: 2026-10-08T02:16:30Z
depends: []
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Final review of material-5094f2 (minor, deferred): tools/tooling_tests.py module_buckets filters empty buckets, but no test fails if the filter goes; the plan's Review Focus claims a two-module full-route test exercises it and none exists. Add a unit test (one module, count=4 -> exactly one bucket) and correct the plan wording.
