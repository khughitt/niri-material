---
id: material-5d92cb
title: "Determine whether the drag opacity, not a sampling coordinate, removes the frost from a dragged glass window"
status: done
priority: 3
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-15T18:43:28Z
updated: 2026-10-15T19:06:18Z
started: 2026-10-15T18:50:03Z
completed: 2026-10-15T19:06:18Z
depends: []
parent: material-f8434f
tags: [rendering, bug]
agent: claude-code/claude-opus-5-5
---

Question: Is the frost loss during a tiled interactive drag (material-e88df7) caused by the tile's drag opacity, INTERACTIVE_MOVE_ALPHA = 0.75 drawn through the alpha offscreen, which lets 25 % of the sharp scene show through the glass, rather than by a wrong backdrop coordinate?
Where to start: docs/notes/2026-10-15-frost-drag-brief.md; src/layout/mod.rs:102 (the constant), :4081-4088 (tiled move animates 1 to 0.75), :2464-2470 (floating moves stay at 1), :4907-4955 (render_interactive_move_for_output); src/layout/tile.rs:2382-2418 (alpha offscreen versus the direct path); src/render_helpers/material/mod.rs:53-96 (background_mapping and its known-limitation comment); prelude.frag sampleBackdrop (UV only, never dst). Frozen headless renders work in src/tests (ring_pair.rs, glass_edge.rs), and src/tests/floating.rs:445 already calls interactive_move_begin.
Bound: frozen-pixel renders only, no host and no fix. Over a high-detail backdrop with backdrop-blur on, compare the glass face of a stationary window with (a) a tiled drag, (b) a floating drag (alpha 1) and (c) a tiled drag with the drag alpha forced to 1 on an unmerged branch; measure high-frequency energy in the face. Also log backdrop_rect for the dragged element to settle the probe's early backdrop_rect.x = 0.994 with an empty ws_rect.
Expected result: record which case loses frost and the measurements on this task and in the brief. If opacity is the cause, list the look options for the owner (brief, Alternatives) with a recommendation; if not, name the next probe. The 2026-09-01 probe patch (niri-experiments 96033b7) no longer applies to HEAD; port only what the bound needs.
Ideas it wakes: on completion, run tasks note material-e88df7 --no-reviewed "<finding>" before this result's branch merges.

## Notes

- 2026-10-15T18:46:25Z (materials-26.04): owner decision 2026-10-15: if the drag opacity is confirmed as the cause, keep the glass opaque during the drag (brief alternative 1). Implementation lean: material tiles skip INTERACTIVE_MOVE_ALPHA (alpha 1, no alpha offscreen), with the drag jelly as the move cue; content-only fade through the material pass is the rejected alternative unless the owner asks for it. Case (c), the tiled drag at alpha 1, is therefore also the fix candidate: report whether it keeps the frost equal to a stationary window's.
- 2026-10-15T18:50:03Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:077e797e-8917-4b97-b869-2d1f82fdabfc","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T19:03:37Z (materials-26.04): attached: frost-drag-evidence.png (60557 bytes)
- 2026-10-15T19:04:13Z (materials-26.04): result 2026-10-15: confirmed, the drag opacity is the cause. Frozen headless renders (branch research/material-5d92cb, 1c98e1cf, base c13f470f), two frosted windows over a 16 px checker placed within the backdrop, frost measured as mean |luma - 3x3 mean| inside the face: stationary 0.00 in every case; tiled drag at alpha 0.75 1.15 (bare checker there 4.61, 0.25 x 4.61 = 1.15); floating drag (alpha 1, direct) 0.00; tiled drag forced to alpha 1 (still through the alpha offscreen) 0.00. Pixelwise, tiled = 0.75 x tiled-at-alpha-1 + 0.25 x bare wallpaper to worst 0.5, mean 0.33 of 8 bits; tiled-at-alpha-1 vs floating at the same place is byte-identical. So sampling is right and the alpha offscreen is harmless; only the 0.75 composite lets the sharp scene through. The dragged element's backdrop_rect equals its position over the backdrop size in every frame, including the first (geo 649,191 then 860,380 on 1280x720); the probe's early backdrop_rect.x = 0.994 did not reproduce on a single output and stays unexplained (multi-output or the drag's start point on the host are untested). Reproduce: in .worktrees/material-5d92cb, FROST_DRAG_LOG=1 FROST_DRAG_DUMP=<dir> just test-one -p niri --no-capture drag_opacity_versus_sampling_geometry
- 2026-10-15T19:06:18Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:077e797e-8917-4b97-b869-2d1f82fdabfc","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T19:06:18Z (materials-26.04): Confirmed: the tiled drag's 0.75 opacity, not a sampling coordinate, removes the frost (face energy 1.15 = 0.25 x checker 4.61 vs 0.00 at alpha 1; alpha-1 tiled drag byte-identical to a floating drag). Code on unmerged research/material-5d92cb (1c98e1cf); evidence attached; brief result section; material-e88df7 woken and scoped as the fix.
  provenance: {"harness_session":"claude-code:077e797e-8917-4b97-b869-2d1f82fdabfc","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
