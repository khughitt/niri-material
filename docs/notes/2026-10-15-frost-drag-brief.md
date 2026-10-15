# Frost loss during drag scoping brief

2026-10-15. Handoff for `material-e88df7` under goal `material-f8434f`, not an
approved design.

## Problem

When a material window is dragged by interactive move, its glass stops
looking frosted. The frost goes when the insert hint appears and returns when
the drag ends. It happens in overview and in the normal scrolling view, and
only to the dragged window: a stationary window beside it stays frosted. The
owner deferred it on 2026-09-01 as cosmetic and transient, acceptable in
burn-in.

## Current behaviour and evidence

- **Probe rounds (2026-09-01).** Four rounds ruled out, by measurement:
  - the blur flag, the `material_ready` gate and blur production;
  - texture identity: the dragged and stationary elements bind the same GL
    texture in one frame;
  - the workspace selector, blur strength, overview zoom and minification.

  They concluded that "the divergence is entirely in the sampling geometry"
  ([backdrop blur design](../materials/2026-08-29-material-backdrop-blur-design.md),
  near line 80).
- **Sampling path, unchanged since then** (read at `2dd3b9fa`):
  - `XrayPos::new` divides the position by zoom, and its readers multiply it
    back (`src/render_helpers/xray.rs:35-57`).
  - `background_mapping` maps the element rect into the backdrop
    (`src/render_helpers/material/mod.rs:53-96`).
  - `sampleBackdrop` reads by UV only and never the destination rect
    (`prelude.frag`).
  - The dragged tile gets an output-relative `XrayPos`
    (`render_interactive_move_for_output`, `src/layout/mod.rs:4907-4955`).

  No wrong coordinate is visible in the code.
- **What differs for the dragged tile.**
  - A tiled drag animates the tile's opacity to
    `INTERACTIVE_MOVE_ALPHA = 0.75` (`src/layout/mod.rs:102`, `:4081-4088`)
    and draws it through the alpha offscreen (`src/layout/tile.rs:2382-2407`).
  - Stationary tiles draw directly (`tile.rs:2409-2418`), and a floating drag
    stays at 1 (`mod.rs:2464-2470`).
  - At 0.75, a quarter of the sharp scene shows through the glass. A blurred
    backdrop has no high-frequency detail, so the detail the eye sees comes
    from that quarter, and the glass reads as clear.
  - The probes never looked at this path. The culling audit
    (`docs/materials/2026-10-08-culling-damage-boundaries-audit.md:180-182`)
    records that tiled drags use it.
- **Since the probes:**
  - Draw now goes through `render_prefiltered` (`4a87ceef`, 2026-09-01).
  - The drag follower adds jelly motion (`f8a890d3`, 2026-10-01).
  - Culling is skipped under the alpha animation (`797ab6ae`).

  None of these changes the coordinates. The probe patch (niri-experiments
  `96033b7`) no longer applies, and there is no `FROSTDBG` code left to
  remove.
- **Unexplained:** the earliest dragged frames logged `backdrop_rect.x` =
  0.994 with an empty `ws_rect`.

## Constraints

- The [render order](../materials/render-pipeline.md) holds.
- A stationary window's frost must not change.
- Drag feedback stays: today the opacity is the "being moved" cue, and the
  drag jelly follows the pointer.
- Frozen headless renders can drive interactive moves (`src/tests/floating.rs`
  calls `interactive_move_begin`), so testing the cause needs no host.

## Alternatives

These apply if the opacity is confirmed as the cause.

1. **Keep the glass opaque during the drag (chosen by the owner,
   2026-10-15).** The implementation lean is that material tiles skip the
   drag alpha: they are drawn at alpha 1 with no alpha offscreen, and the drag
   jelly remains the move cue. The rejected alternative fades only the window
   content, through a new content alpha in the material pass.
   - Gains: the frost is kept, and the glass reads as one object.
   - Costs: the drag cue changes for material windows only.
2. **Keep 0.75 and accept the see-through glass.** This is today's
   behaviour. The bug is dropped as by design.
3. **Composite the dragged glass over its own blurred backdrop.** The quarter
   that shows through would then be frosted too. This needs a second backdrop
   sample in the offscreen, a higher cost for a transient state.

## Unanswered questions

- **Is the opacity the cause?** `material-5d92cb` answers it with frozen
  renders: a tiled drag, a floating drag, and a tiled drag at alpha 1.
- **What should a dragged glass window look like?** Answered by the owner on
  2026-10-15: keep the glass opaque during the drag (alternative 1). This
  applies once `material-5d92cb` confirms the opacity as the cause.

## Proposed decomposition

- `material-f8434f`: the goal.
- `material-5d92cb`: the research (P3, direct, no host). It wakes
  `material-e88df7`.
- `material-e88df7` returns to idea and waits on `material-5d92cb`. If the
  research confirms the opacity, it is scoped directly as the fix in
  alternative 1. Case (c) of the research, a tiled drag at alpha 1, is the
  fix candidate.
