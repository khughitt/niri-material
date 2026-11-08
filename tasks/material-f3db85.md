---
id: material-f3db85
title: Skip the shell hue resolver and its timer when no layer has a surface remap
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-17T14:42:54Z
updated: 2026-10-17T15:13:33Z
depends: []
parent: material-55380c
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

Final review of material-62347d (deferred minors): update_shell_hue runs once per output redraw over every workspace, folding every signalled window's WindowSignals (allocating sources/impulses) just to read accent, and arm_shell_hue_timer re-inserts its calloop timer every output frame while a settle is pending, then wakes all outputs — for every user, including configs with no surface-remap. Gate the resolver and timer on any mapped layer (or the config) having a remap, add an accent-only accessor on WindowSignals, and re-arm only when the deadline changes.

## Notes

- 2026-10-17T15:13:33Z (materials-26.04): also from material-62347d's reviews: popup remap damage is one union rectangle, so opening or moving any popup damages the span between distant popups; the shell hue timer's callback queues a redraw on every output when only the settling workspace's output needs it
