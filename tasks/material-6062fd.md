---
id: material-6062fd
title: "Glass optics: edges, depth and the ring inside the slab"
status: todo
priority: 1
created: 2026-10-02T23:07:45Z
updated: 2026-10-02T23:47:23Z
depends: []
tags: [lane, rendering]
source: docs/notes/2026-10-02-workstreams-brief.md
agent: claude-code/claude-opus-5-5
---

Lane goal (see docs/notes/2026-10-02-workstreams-brief.md). Why: the edge model (planar chamfer, global thickness, Schlick) underlies the opaque focused edge, the surface-drawn ring and content that floats above the glass. Done when its children land or are dropped; the height-field bevel (material-be611b) is the shared prerequisite. Host: pixel work on the headless fixture; owner judges sheets.
First milestone: the height-field bevel (material-be611b) implemented and its appearance reviewed. Ring embedding (material-4e3e9c) follows it; prepare its comparison scenes and acceptance criteria meanwhile.
