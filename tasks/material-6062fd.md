---
id: material-6062fd
title: "Glass optics: edges, depth and the ring inside the slab"
status: todo
priority: 1
lane: true
created: 2026-10-02T23:07:45Z
updated: 2026-10-03T16:48:07Z
depends: []
tags: [rendering]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
---

Give glass edges, content depth and the embedded ring a shared optical foundation. First milestone: implement the height-field bevel (material-be611b) and have the owner review its appearance.

The edge model (planar chamfer, global thickness, Schlick) underlies the opaque focused edge, the surface-drawn ring and content that floats above the glass. Done when its children land or are dropped; the height-field bevel (material-be611b) is the shared prerequisite. Host: pixel work on the headless fixture; owner judges sheets.

Ring embedding (material-4e3e9c) follows it; prepare its comparison scenes and acceptance criteria meanwhile.
