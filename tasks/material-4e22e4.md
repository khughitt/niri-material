---
id: material-4e22e4
title: test_a_screencast_run_reaches_its_crops fails under host load
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-17T11:58:10Z
updated: 2026-10-17T11:58:10Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
---

On 2026-10-17 at load average ~12, test_optic_settling.DriverCleanupTests.test_a_screencast_run_reaches_its_crops failed once in the full tooling suite with 'stimulus schedule failed at 44 s' (AssertionError: False is not true : case screencast). The same tree passed four full runs before, the test passes in isolation, and a rerun of the full suite passed. A timing bound in the screencast stimulus schedule appears not to tolerate a loaded host. Find the bound and make it load-independent or state its host requirement.
