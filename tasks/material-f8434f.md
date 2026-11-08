---
id: material-f8434f
title: Dragged glass windows keep their frost
status: done
priority: 2
created: 2026-10-15T18:43:23Z
updated: 2026-10-15T19:31:54Z
completed: 2026-10-15T19:31:54Z
depends: []
tags: [rendering, bug]
source: docs/notes/2026-10-15-frost-drag-brief.md
agent: claude-code/claude-opus-5-5
---

Why: a material window dragged by interactive move loses its frost from the moment the insert hint appears until the drag ends, in overview and in the normal view (material-e88df7). Four probe rounds blamed sampling geometry; a code reading on 2026-10-15 points instead at the dragged tile's 0.75 opacity. Handoff: docs/notes/2026-10-15-frost-drag-brief.md.

Done when the cause is established and the owner's chosen look for a dragged glass window is implemented or the bug is dropped.

## Notes

- 2026-10-15T18:46:20Z (materials-26.04): owner decision 2026-10-15: if the drag opacity is confirmed as the cause, keep the glass opaque during the drag (brief alternative 1). Implementation lean: material tiles skip INTERACTIVE_MOVE_ALPHA (alpha 1, no alpha offscreen), with the drag jelly as the move cue; content-only fade through the material pass is the rejected alternative unless the owner asks for it.
- 2026-10-15T19:31:54Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:373349ad-abc0-4809-b4f1-5b4a032cb269","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T19:31:54Z (materials-26.04): Met: cause established by material-5d92cb (the tiled drag's 0.75 opacity through the alpha offscreen, not sampling geometry); the owner's chosen look (opaque glass during drag, brief alternative 1) implemented by material-e88df7 at c805ab6e with a frozen-render regression check.
  provenance: {"harness_session":"claude-code:373349ad-abc0-4809-b4f1-5b4a032cb269","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
