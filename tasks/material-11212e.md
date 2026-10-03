---
id: material-11212e
title: "ring-motion-clips.sh: shell nits from the ring-beam Task 4 review"
status: todo
priority: 4
size: s
complexity: low
process: direct
created: 2026-09-21T09:39:49Z
updated: 2026-10-02T23:07:56Z
depends: []
parent: material-2834d7
tags: [tooling]
agent: claude-code/claude-opus-5
---

Deferred minors on docs/materials/scripts/ring-motion-clips.sh from the 2026-09-19 ring-beam Task 4 review, one pass: quote the $(awk …) expansions; replace A && B || C with if/else where C must not run on B's failure; the scratch builds copy HEAD into a fresh tree and so build without the worktree's .cargo config (rustflags/target-dir) — copy or reference .cargo, or document the difference; the capture preflight runs before the release and scratch builds, so the builds' load is not in the baseline — move preflight after the builds or settle again before the first launch; the focus-toggle subshell swallows wlrctl failures — surface a non-zero exit.
