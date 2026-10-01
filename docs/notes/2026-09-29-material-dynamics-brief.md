# Material dynamics scoping brief

## Problem

Make glass respond coherently to movement and focus, then establish whether
named animation profiles or hierarchical state add anything the existing
mechanisms cannot express. This pass covers `material-6d4de5`,
`material-5a5fff`, `material-9be53d` and `material-e6036d`. Reuse the dynamics
goal `material-53f873`; this brief is not an approved implementation design.

## Current behaviour and evidence

- The [motion sweep](../materials/2026-09-11-jelly-motion-sweep.md), on
  `7526af1d`, resolved all sampled neighboring flex/ripple values during
  native column movement and returned to identical settled pixels. It did
  not test pointer dragging, physical-panel perceptibility or suitable gains.
- `src/layout/tile.rs::animation_residual` excludes the interactive grab
  offset. `material-4354cf` adds `motion_residual`, combining move animations
  with drag-follower lag for rendering and unmap snapshots. Scrolling also
  includes column/view residuals. `material_dynamics` still limits flex to
  one quarter of bevel depth. Deterministic drag/hold/release traces are
  recorded below; headless capture and owner judgment remain outstanding.
- `src/render_helpers/material/mod.rs::apply_resolved` updates parameters
  in place for the same definition name and replaces `MaterialState` on a
  name change. Replacement changes the offscreen buffer, element identity
  and seed. The old `material-5a5fff` note incorrectly attributes layout
  residual reset to this swap: movement animations live on `Tile`.
- Named response blocks already exist in `niri-config/src/material/mod.rs`.
  Tile focus/signal crossfades use native `Animation` and the configurable
  `material-signal` animation. These crossfades do not interpolate glass
  definitions. The [focus treatment spike](../materials/2026-09-04-focus-glass-spike.md)
  established seamless transparent terminals, not that the hard cut is
  objectionable today.

Both referenced mindful thoughts (`3f94e656b70f4e5585c1cb60c166e4da` and
`5778c060e57d47dd808e20347223cfd5`) were unavailable through the local CLI.
Their task bodies and notes remain intact as the available intent.

## Constraints

Preserve the [render order](../materials/render-pipeline.md), motion policy,
finite settling, visibility gates and input-idle behavior. The idle-budget
trace on `721b8df7` passed 25/25 observations, including a quiescent 600 s
hold; current-binary power verification remains `material-39a46f`.
Do not duplicate `material-0db905`'s sustained-optic settling design or
`material-c1330b`'s workspace signal replay. Their questions remain open.
The source's occasional micro-movement proposal does not authorize
unconditional periodic redraws. Visual suitability requires owner judgment;
new design and implementation plans retain their review gates.

## Alternatives

1. **Current lean: reuse native animations and named responses.** Establish
   drag/release and focus-swap baselines, then design only a demonstrated gap.
2. Add a focused deformation input or glass-parameter transition. This
   requires explicit stimulus, reversal, discrete-value and settling rules;
   raw pointer displacement is not a velocity signal.
3. Introduce named animation profiles and nested state machines now. Defer
   this until concrete behavior establishes missing composition or state
   semantics; no such requirement is demonstrated by these records.

## Unanswered questions

- What does drag/hold/release do in each layout, compared with native
  movement? Answered by `material-b3ce14`: see the drag baseline finding.
- Is the focus swap visually objectionable, and which parameters need
  continuity? `material-8e3b73` supplies a clip and state inventory; the owner
  judges the appearance before interpolation design is justified.
- Do the resulting behaviors need profile semantics beyond named responses,
  and where should those live? Revisit `material-9be53d` with both findings;
  Prism ownership remains unverified in this local pass.
- Which concrete window/workspace/session transition needs a hierarchy?
  A reproducible use case must answer before `material-e6036d` is unshelved.

## Drag baseline finding (`material-b3ce14`)

`src/layout/tests/drag_dynamics.rs` drives the layout on a pinned clock at
16 ms frames with default animations (window-movement spring, matching the
live config) and records the motion residual the renderer passes to
`jelly_state` for each frame, next to a native column-move control. Flex is
reported at jelly-flex 0.01 on the motion sweep's bevel 12 / thickness 20,
where the cap is 3 px; the live 0.0066 scales magnitudes, not phases. The run
is deterministic, so repeats do not vary; `-- --nocapture` prints the traces.

| Phase | Scrolling baseline peak flex | Floating baseline peak flex | Follow-lag (scrolling / floating) |
| --- | ---: | ---: | --- |
| Rubber band below the start threshold | 0 | — (no threshold) | 0 / — (unchanged) |
| Lift to the pointer | 1.46 px, settles in 240 ms | 0.18 px (first pointer step) | 1.4554 px, 240 ms / 0.1845 px, 224 ms |
| Drag at 40 px/frame | 0 | 0 | 1.4705 px / 1.4705 px |
| Hold | 0 | 0 | both below 1% of drag peak after 208 ms |
| Release | 2.96 px (capped), settles in 288 ms | 0 (dropped in place) | 2.9622 px, 288 ms / zero after settled hold |
| Release below the threshold (cancel) | 0.75 px | — | unchanged (no follower created) |
| Native column move (control) | 1.11 px, settles in 240 ms | — | 1.1054 px, 240 ms / — (unchanged) |

In the baseline, while a window follows the pointer, its tile carries no move
animation, and `animation_residual` excludes the grab offset. The baseline jelly
input is therefore
exactly zero during drag and hold, whatever the pointer speed. Ripple is
also off, because its activity gate reads the same residual. Glass only
flexes on the layout's own animations: lift and release for tiled windows,
and just the lift catch-up for floating ones. This is an absent stimulus, not
a capture failure. No pixel capture was run, because a zero residual renders
settled glass and the sweep already measured the shader's response to a
nonzero one.

`material-4354cf` now adds the follow-lag stimulus. The tile still renders at
the pointer, while a spring follower contributes lag only to the jelly residual.
The lag decays on a hold and survives release as an added decaying residual;
release position and its existing animation remain unchanged. The
[accepted design](../specs/2026-10-01-drag-follow-lag-design.md) reuses jelly-flex
and window-movement's spring, keeps the existing flex cap, and leaves rubber-band
flex off. Owner judgment of drag and release appearance remains outstanding.

The follow-lag column comes from Task 2's recorded
`just test-one -p niri drag_dynamics -- --nocapture` run on `d9631ff8`, with
14/14 checks passing. Both columns use jelly-flex 0.01 and bevel 12 / thickness 20.
The settle times measure the last 16 ms sample at or above 1% of phase peak,
not the follower's lag/velocity removal threshold. Hold release was measured
after settling; moving-release continuity has separate deterministic checks.

### Follow-lag clips (`material-55f8a0`)

The stdlib move client and `docs/materials/scripts/drag-lag-clips.sh` fixture are
implemented. The 2026-10-01 `scroll-fast` pilot attempt refused at preflight
before launching Weston: CPU busy 46.4% (limit 10%), load1 23.33 (limit 2),
GPU utilization 12% (limit 5%), non-idle GPU states, power IQR 8.475 W
(limit 1 W), and a BitwigStudio compute client. The attempt took 26.491 s;
the release build had passed in 6 min 2 s. Evidence is under
`$NIRI_MATERIAL_WORK_ROOT/drag-lag-clips-81ec2a1f/drag-lag-1752426-1790865687/capture.json`.
No clip, frame, interactive-move confirmation, or review page exists. The pilot
and full five-sequence capture need a quiet-host retry; the fixture does not
relax thresholds or stop competing applications.

The fixture pins jelly-flex 0.0066, bevel 12, thickness 20
and ripple off; runs `scroll-fast`, `scroll-slow`, `float-fast`, `float-slow` and
`native`; and verifies the drag enters an interactive move through IPC. Owner
judgment follows a successful headless pilot and full capture. Budget 11 min
for the quiet-host retry: build 6.5, preflight 0.5, pilot 1, full run 3. The agent
runs the pilot through its verdict before attempting the full set.

## Proposed decomposition

- `material-b3ce14` (drag baseline): P1, small, mid complexity, direct
  research; wakes `material-6d4de5` and `material-9be53d`.
- `material-8e3b73` (focus swap): P2, small, mid complexity, direct
  research; wakes `material-5a5fff` and `material-9be53d`.
- Both follow-ups belong to `material-53f873`, record findings here and
  add a finding note to every waiting idea in the same result commit.
  No new capture was run during scoping.
- The three waiting ideas remain briefed. `material-e6036d` is shelved
  until an explicit transition cannot be expressed with current signals,
  responses and animations, with an acceptance check identifying the gap.
  Previously unparented members now share the existing dynamics goal;
  its body and all original sources are preserved.
