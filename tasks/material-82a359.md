---
id: material-82a359
title: IPC tile_pos_in_workspace_view is null for every window
status: todo
priority: 2
size: s
created: 2026-09-06T11:29:32Z
updated: 2026-09-06T11:29:32Z
depends: []
tags: [ipc, bug]
---

niri msg -j windows reports tile_pos_in_workspace_view as null for all 11 windows in a live session, including ordinary tiled windows that have a valid pos_in_scrolling_layout, and also in a nested headless session. The niri-ipc doc comment describes it as 'Tile position within the current view of the workspace' and the same 'workspace view' as gradients' relative-to, with optionality framed as applying to some window types. As it stands the field is unusable for its stated purpose. It is also a silent trap for consumers: jq turns null[0] + 0 into 0.0, so deriving a screen position from it yields a plausible 0,0 rather than an error - that is exactly how material-37cec9's geometry derivation broke before it was replaced by measuring the window rect from a capture. Decide whether the field should be populated or the documentation corrected, and check whether niri upstream behaves the same.
