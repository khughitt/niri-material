---
id: material-f8434f
title: Dragged glass windows keep their frost
status: todo
priority: 2
created: 2026-10-15T18:43:23Z
updated: 2026-10-15T18:43:23Z
depends: []
tags: [rendering, bug]
source: docs/notes/2026-10-15-frost-drag-brief.md
agent: claude-code/claude-opus-5-5
---

Why: a material window dragged by interactive move loses its frost from the moment the insert hint appears until the drag ends, in overview and in the normal view (material-e88df7). Four probe rounds blamed sampling geometry; a code reading on 2026-10-15 points instead at the dragged tile's 0.75 opacity. Handoff: docs/notes/2026-10-15-frost-drag-brief.md.

Done when the cause is established and the owner's chosen look for a dragged glass window is implemented or the bug is dropped.
