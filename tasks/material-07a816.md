---
id: material-07a816
title: Test + CI iteration cost audit
status: todo
priority: 2
size: m
created: 2026-09-04T21:44:54Z
updated: 2026-09-04T21:44:54Z
depends: []
tags: [testing]
---

Piece of ops-65837b (the cross-project audit in the ops hub). 1. Measure: full-suite wall time, and roughly how often agent full-suite runs fail here. 2. Add a fast or affected-only test target for the inner loop and point AGENTS.md at it; keep the full suite for commit and CI. 3. Use a quiet reporter so test output does not flood agent context. 4. Fix suite hygiene: sleeps, real network, unshared fixtures. Record the before and after numbers in a note on this task.
