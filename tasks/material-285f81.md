---
id: material-285f81
title: "Settling review minors: layout unwrap, Niri-level redraw/timer assertions, flaky reload test"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-02T03:36:04Z
updated: 2026-10-02T03:36:04Z
depends: []
parent: material-f86183
tags: [performance]
agent: claude-code/claude-opus-5-5
---

Deferred Minor findings from the whole-change review of sustained optic settling (2026-10-02, final-review.md in the material-a1d7da SDD workspace): .unwrap() at src/layout/mod.rs:2820 panics if the two active/idle copies disagree; Niri-level tests never assert that edges queue a redraw or that the optic timer is dropped and re-armed (spec §7.5); a reload test is timing-dependent; one signal test pauses a clock the code never reads; Optic::next_change doc does not say it returns a logical deadline; hidden-tile tests assert only the shared clock, not a rendered tile.
