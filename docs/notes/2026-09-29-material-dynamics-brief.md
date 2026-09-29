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
  offset. Scrolling rendering combines it with column/view residuals;
  floating and interactive-move rendering pass the tile residual directly.
  `material_dynamics` still limits flex to one quarter of bevel depth.
  Missing drag-driven deformation is plausible, not a new capture finding.
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
  movement? `material-b3ce14` establishes the baseline and missing stimulus.
- Is the focus swap visually objectionable, and which parameters need
  continuity? `material-8e3b73` supplies a clip and state inventory; the owner
  judges the appearance before interpolation design is justified.
- Do the resulting behaviors need profile semantics beyond named responses,
  and where should those live? Revisit `material-9be53d` with both findings;
  Prism ownership remains unverified in this local pass.
- Which concrete window/workspace/session transition needs a hierarchy?
  A reproducible use case must answer before `material-e6036d` is unshelved.

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
