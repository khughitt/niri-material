---
id: material-82a359
title: IPC tile_pos_in_workspace_view is null for every window
status: done
priority: 2
size: s
created: 2026-09-06T11:29:32Z
updated: 2026-09-06T22:57:39Z
depends: []
tags: [ipc, bug]
---

niri msg -j windows reports tile_pos_in_workspace_view as null for all 11 windows in a live session, including ordinary tiled windows that have a valid pos_in_scrolling_layout, and also in a nested headless session. The niri-ipc doc comment describes it as 'Tile position within the current view of the workspace' and the same 'workspace view' as gradients' relative-to, with optionality framed as applying to some window types. As it stands the field is unusable for its stated purpose. It is also a silent trap for consumers: jq turns null[0] + 0 into 0.0, so deriving a screen position from it yields a plausible 0,0 rather than an error - that is exactly how material-37cec9's geometry derivation broke before it was replaced by measuring the window rect from a capture. Decide whether the field should be populated or the documentation corrected, and check whether niri upstream behaves the same.

## Notes

- 2026-09-06T22:54:39Z (fix/ipc-tile-pos-doc): Root cause: ScrollingSpace::tiles_with_ipc_layouts never fills tile_pos_in_workspace_view; only FloatingSpace does. Upstream (#1265) behaves identically and its maintainer declined per-tile filling in #2381 because a resize would fan out layout events to every tile to the right; #4369 (populate) is open unreviewed, #4147 (Workspace::scrolling_view_pos) is already carried by this fork. Decision: keep upstream's contract, correct the niri-ipc doc to say floating-only and point at scrolling_view_pos, pin it with a layout test, correct the stale plan-doc derivation.
- 2026-09-06T22:57:39Z (fix/ipc-tile-pos-doc): Kept upstream's contract: tile_pos_in_workspace_view stays floating-only. Documented that in niri-ipc with a pointer to Workspace::scrolling_view_pos (the carried #4147 field), pinned it with layout::tests::ipc_tile_pos_in_workspace_view_is_set_for_floating_only, and corrected the v1 preflight plan's step-5 derivation. Upstream: same behaviour, issue #2381 open, #4369 (populate) and #4147 (view pos) open.
