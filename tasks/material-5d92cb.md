---
id: material-5d92cb
title: "Determine whether the drag opacity, not a sampling coordinate, removes the frost from a dragged glass window"
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-15T18:43:28Z
updated: 2026-10-15T18:43:28Z
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
