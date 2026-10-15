---
id: material-e88df7
title: Fix frost loss during interactive material-window drag
status: doing
priority: 3
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-01T14:35:48Z
updated: 2026-10-15T19:19:26Z
started: 2026-10-15T19:19:26Z
depends: [material-5d92cb]
parent: material-f8434f
tags: [migration, bug]
---

Why: a material window dragged by interactive move loses its frost from the moment the insert hint appears until the drag ends, in overview and the normal view. Established by material-5d92cb (2026-10-15, notes below; do not re-derive): a tiled drag animates the tile to INTERACTIVE_MOVE_ALPHA = 0.75 and composites it through the alpha offscreen, so a quarter of the sharp scene shows through the blurred glass. Sampling and the offscreen are correct; at alpha 1 the dragged glass is byte-identical to a floating drag. The owner chose (2026-10-15) to keep the glass opaque during the drag. Brief: docs/notes/2026-10-15-frost-drag-brief.md (alternative 1).

Done:
- A tile with a material (Tile::material() is Some, src/layout/tile.rs ~:2694) keeps alpha 1 for the whole interactive move: no drag-alpha animation and no alpha offscreen. The drag jelly remains the move cue. Windows without a material keep 0.75.
- Every drag-alpha site follows: the tiled move's start (src/layout/mod.rs, the animate_alpha to INTERACTIVE_MOVE_ALPHA after Starting becomes Moving), the floating/tiled target toggles (~:3245, ~:3258) and the drag end (~:4240). Keep the floating-tile invariant check (~:2464) passing.
- Regression check: port src/tests/frost_drag.rs from branch research/material-5d92cb (1c98e1cf) without its research hooks (FORCE_OPAQUE_DRAG, the mapping log). A tiled drag of a material window renders byte-identical to the same window dragged floating at the same place, and its face carries no checker energy; a window without a material still drags at 0.75. The test fails if the drag alpha returns.
- Docs: drop the 'Known limitation' paragraph on background_mapping (src/render_helpers/material/mod.rs ~:53-60), and correct the conclusion in docs/materials/2026-08-29-material-backdrop-blur-design.md (~:80, 'the divergence is entirely in the sampling geometry') with a pointer to the brief's result.
- just test-fast is green.

Obsolete from the 2026-09 acceptance: removing FROSTDBG instrumentation (none remains) and porting the 2026-09-01 probe patch (niri-experiments 96033b7, no longer applies). The frozen regression check replaces the live verification: the cause is a deterministic composite, which the frozen render measures directly.

## Notes

- 2026-09-01T23:35:58Z (materials-26.04): Deferred by owner: cosmetic, transient, judged acceptable in burn-in. Four probe rounds eliminated by measurement: the flag (blur=true on all 173 draws), the silent material_ready gate (true for both windows, both prepare calls true), blur production (both buffers cached-blur, never SHARP), buffer/texture identity (dragged and stationary elements bind the SAME Rc addresses and the SAME GL texture id 20 at 3440x1440 in one frame), the workspace/backdrop selector (ws_rect non-empty in 921 of 927 draws), blur strength (absent at 6 passes as at 1), overview-specificity (reproduces at zoom 1 in the normal view) and minification (tile_render_location upscales by zoom=1 there). Also established: the background xray buffer is empty at all times on this host because the wallpaper is place-within-backdrop, so all frost arrives via the backdrop path. Next probe: log the destination rect and XrayPos per element beside backdrop_rect and compare texels-sampled to pixels-drawn for the two elements.
- 2026-09-01T23:39:15Z (materials-26.04): Instrumentation preserved as niri-experiments fixtures/frost-drag-probe.patch (commit 96033b7, branch results/slice3, pushed). Verified to apply cleanly against materials-26.04 aa0a66be with git apply. The debug/overview-drag-frost worktree and branch are retired; resume by applying that patch.
- 2026-09-12T19:24:33Z (materials-26.04): Complexity high: Four probe rounds isolate per-element sampling geometry but do not identify the bad coordinate or scale. Resolving drag/workspace transforms and proving equivalent sampling remains subtle rendering investigation despite a potentially small fix.
- 2026-10-15T18:44:19Z (materials-26.04): scope: briefed; premise 'fault in sampling geometry' contested at 2dd3b9fa: no coordinate change since the probes and none visible in code, but a tiled drag alone draws through the alpha offscreen at INTERACTIVE_MOVE_ALPHA 0.75, letting the sharp scene through; probe patch (niri-experiments 96033b7) no longer applies and no FROSTDBG remains; returned to idea under new goal material-f8434f, waits on research material-5d92cb (frozen renders, no host); fix needs the owner's choice of dragged-glass look; brief: docs/notes/2026-10-15-frost-drag-brief.md
- 2026-10-15T18:46:15Z (materials-26.04): owner decision 2026-10-15: if the drag opacity is confirmed as the cause, keep the glass opaque during the drag (brief alternative 1). Implementation lean: material tiles skip INTERACTIVE_MOVE_ALPHA (alpha 1, no alpha offscreen), with the drag jelly as the move cue; content-only fade through the material pass is the rejected alternative unless the owner asks for it.
- 2026-10-15T19:05:16Z (materials-26.04): finding (material-5d92cb, 2026-10-15): confirmed, the drag opacity is the cause, not a sampling coordinate. A tiled drag composites the tile at 0.75, so a quarter of the sharp scene shows through: face energy 1.15 = 0.25 x bare checker 4.61, against 0.00 for a floating drag, a tiled drag forced to alpha 1, and every stationary window. Sampling and the alpha offscreen are correct (alpha-1 tiled drag byte-identical to a floating drag at the same place). Branch research/material-5d92cb 1c98e1cf; brief result section.
- 2026-10-15T19:05:55Z (materials-26.04): scope: scoped; research material-5d92cb confirmed the drag opacity (not sampling) and the owner chose opaque glass during drag; body rewritten as the fix (material tiles keep alpha 1 for the whole move, port frost_drag.rs as the regression check, drop the stale known-limitation comment and design-doc conclusion); FROSTDBG removal and probe-patch port dropped as obsolete, live verification replaced by the frozen check; s, low, direct; priority unchanged (P3)
- 2026-10-15T19:19:26Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:373349ad-abc0-4809-b4f1-5b4a032cb269","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
