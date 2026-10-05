---
id: material-dcb881
title: Screencast sample can take a frame before the damage it armed for
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-05T03:27:25Z
updated: 2026-10-05T09:10:26Z
depends: []
parent: material-2834d7
tags: [testing]
agent: claude-code/claude-opus-5-5
---

In the dedicated-lane screencast case, cast_sample arms the consumer, causes damage, then takes the first frame after arming; nothing ties that frame to the damage. 2026-10-05 dev check (hold-pilot/pilot-2, binary bin-257f847b): sample-1's frame came 390 ms after its request (passing runs ~30 ms), and the probe's print_line change landed one sample late: client-1 == client-2, client-3 differs, verdict invalid. An immediate rerun (pilot-3) passed. Make the sample wait for a frame that contains the damage (e.g. frame after the client's commit, or compare against the previous sample until changed within a bound) so a slow frame cannot shift the pattern.

## Notes

- 2026-10-05T03:27:25Z (disturber-hold): concerns: material-188aaa extension — the hold's first evidence run exposed a sampling race in the screencast instrument
- 2026-10-05T09:10:26Z (materials-26.04): correction: the concerns: note above was written before material-188aaa closed, so by the note rules it is a review finding of material-188aaa's live validation, not a concern against closed work; outcome measures should not count it.
