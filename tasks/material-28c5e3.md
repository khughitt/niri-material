---
id: material-28c5e3
title: "ring-motion-clips.sh: the mid-pass corner crop should follow the head"
status: doing
priority: 3
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-21T09:39:49Z
updated: 2026-09-30T09:54:03Z
started: 2026-09-30T09:54:03Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5
---

docs/materials/scripts/ring-motion-clips.sh crops the focused pane's top-left corner at the frame nearest 2 s as 'mid-pass'. The head starts at the top-left and at 300 px/s is at the top-right corner by 2 s, so the crop holds the tail's end over the resting ring, not the comet (evidence doc, 'The 2 s corner crop'). Crop the corner the head is nearest at that instant (arc position = speed × t on the perimeter), or the top-right at 2 s; keep rest and tail-clear as they are. The owner composites of 2026-09-21 were made by hand for this reason.

## Notes

- 2026-09-30T09:54:03Z (materials-26.04): started
