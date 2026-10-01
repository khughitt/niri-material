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
  movement? Answered by `material-b3ce14`: see the drag baseline finding.
- Is the focus swap visually objectionable, and which parameters need
  continuity? `material-8e3b73` supplies the state and parameter inventory
  (see the focus swap finding) and a clip fixture; the owner judges the clip
  before interpolation design is justified.
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

| Phase | Scrolling peak flex | Floating peak flex |
| --- | ---: | ---: |
| Rubber band below the start threshold | 0 | — (no threshold) |
| Lift to the pointer | 1.46 px, settles in 240 ms | 0.18 px (first pointer step) |
| Drag at 40 px/frame | 0 | 0 |
| Hold | 0 | 0 |
| Release | 2.96 px (capped), settles in 288 ms | 0 (dropped in place) |
| Release below the threshold (cancel) | 0.75 px | — |
| Native column move (control) | 1.11 px, settles in 240 ms | — |

While a window follows the pointer, its tile carries no move animation, and
`animation_residual` excludes the grab offset. The jelly input is therefore
exactly zero during drag and hold, whatever the pointer speed. Ripple is
also off, because its activity gate reads the same residual. Glass only
flexes on the layout's own animations: lift and release for tiled windows,
and just the lift catch-up for floating ones. This is an absent stimulus, not
a capture failure. No pixel capture was run, because a zero residual renders
settled glass and the sweep already measured the shader's response to a
nonzero one.

The smallest missing contract is a follow-lag stimulus during an interactive
move. The tile still renders at the pointer, but a critically damped point
chases that location on the window-movement spring, and the jelly receives
the point's lag as the residual. It is velocity-derived, decays to zero on a
hold (finite settling and idle unaffected), and hands over to the release
`animate_move_from` without a jump if the release starts from the lagged
point. Native animation config and the existing jelly parameters express it;
no named profile or state machine is needed for this behaviour.

The owner still has to decide:

- whether a held drag should deform at all
- the gain: reuse jelly-flex or add a separate drag gain
- the spring: window-movement or a dedicated one
- whether a long tiled drop should keep saturating at the cap
- whether the rubber band should flex

## Focus swap finding (`material-8e3b73`)

The live Prism config selects `terminal-glass` for the active kitty or
ghostty and `terminal-glass-inactive` for the others, so every focus change
swaps two tiles' definitions. `src/tests/focus_swap.rs` drives that rule pair
through the real focus and window-rule path, pinned from `prism.kdl` of
2026-09-30, next to a same-definition control.

### State lifetime

A focus change sets `need_to_recompute_rules`; in `State::refresh`,
`refresh_window_rules` then calls `Tile::update_window`, whose
`refresh_material` hands the newly resolved material to `apply_resolved`. The
focus crossfade and beam start in the same cycle's `update_render_elements`,
reading the new material's response.

| What | On a name swap | Where it lives |
| --- | --- | --- |
| Glass and response values | cut to the new definition | `MaterialState.config` |
| Element `Id`, commit counter, offscreen buffer | replaced: one frame of full element damage and a fresh offscreen | `MaterialState` |
| Jelly seed | replaced: a new ripple pattern and aurora offset and phase | `MaterialState` |
| Column, tile and view motion, interactive-move offset | kept: the render position is identical across a mid-move swap | `Tile`, `Column`, `ScrollingSpace` |
| Focus and signal crossfades, focus beam | kept; the focus crossfade reverses from its current value | `Tile` |

The test pins these rows:

- **Same definition:** a focus toggle updates the state in place, with the
  same `Id` and seed.
- **Named response:** a focus-selected `response=` on one definition also
  updates in place.
- **Reversal within one refresh:** no swap happens.
- **Reversal across refreshes:** A to B to A mints a third state. The name
  returns, but the seed does not.

The old `material-5a5fff` note and the focus spike assumed a swap restarts
jelly residuals. It does not: motion lives on the layout and survives the
swap unchanged.

The seed is visible only where the shader reads it:

- the ripple, while the jelly is active (motion)
- the aurora field, at rest

The live pair has `aurora 0`, so at rest the live swap's visible change is the
glass values alone. During motion, the ripple pattern also jumps.

The focus beam is a separate hard cut: losing focus drops it, and gaining
focus starts a new one. Its gates read the response, which is identical in
both live definitions.

### Parameter inventory

The live pair differs in eight values. Everything else is equal, including
both responses: ior 1.28, light-ior 2.5, noise 0.03 white, jelly, bevel 12,
offsets 0, aurora 0 and iridescence 0.

| Parameter | Active | Inactive | Kind |
| --- | --- | --- | --- |
| thickness | 31.2 | 49.5 | length; also sets the jelly cap through `bevel_depth` |
| attenuation-color | `#152F30` | `#0C1314` | color |
| attenuation-distance | 16 | 28 | Beer-Lambert distance; a linear blend is not linear in transmittance |
| chromatic-aberration | 0.36 | 0.34 | continuous |
| distortion | 0 | 0.42 | continuous; the shader's `> 0` gate is continuous at 0 |
| distortion scale | 0.09 | 0.08 | noise frequency; blending it zooms the field rather than fading it |
| roughness | 0.24 | 0.03 | continuous |
| saturation | 0.95 | 0.8 | continuous |
| backdrop-blur | true | false | discrete gate: which backdrop texture is sampled |

Across every field of `ResolvedGlass` and `ResolvedResponse`:

- **Continuous values:**
  - ior, light-ior, thickness, attenuation-distance, chromatic-aberration
  - distortion, anisotropic-blur, roughness
  - iridescence, aurora amount, jelly-flex, jelly-ripple
  - ring-gap, ring-width, ring-glow, ring-beam-noise
- **Geometry-bearing values:** bevel and offset-x/y. They move the slab
  and its view test.
- **Rates and frequencies:** blending them moves phase, not appearance.
  - distortion scale, aurora drift-hz
  - ring-beam-speed, ring-beam-noise-hz, ring-beam-decay
- **Colors:** attenuation-color, the aurora pair and ring-color.
- **Discrete gates:**
  - backdrop-blur
  - noise type
  - the written-or-inherited state of noise and saturation, whose
    inherited value follows backdrop-blur
- **Response selectors:** focus (`ring-light`/`none`), accent, attention,
  ping, done and error.
- **Per-state random value:** the seed.

A missing material, the transient between a reload's layout update and its
rule recompute, drops the state and restores a fresh one; there is no value
to blend from. A reload that keeps the name updates in place, and a rename
swaps.

### Clip

`docs/materials/scripts/focus-swap-clips.sh` records six sequences on a
two-pane headless scene: `swap`, `same`, `seed`, `reversal`, `move` and
`beam`. The script header describes each one. Each sequence produces a
screenshot burst, a GIF, per-pane frame-to-frame RMSE and the largest
step's before and after frames.

The swap sequences turn the beam off. This separates the glass cut and the
seed replacement from the focus beam, which only the `beam` sequence shows.

Recorded 2026-10-01 from a TTY with the desktop stopped, at `d1cd7685`
(release binary SHA-256 `7d2a1b44…e824e`), every sequence `settled` under
`tools/capture-meta`: run `focus-swap-3831771-1790827068` under
`$NIRI_MATERIAL_WORK_ROOT/focus-swap-clips-d1cd7685/`. The burst samples at
about 3.2 frames a second (0.31 s apart), not the ten the script header
assumed, so a step is resolved to one 0.31 s interval. Focus changes about
0.52 s into each burst. Per-pane RMSE against the previous frame, on the
two 640 px halves (left / right):

| Sequence | Step at the change | Next frame | Then |
| --- | --- | --- | --- |
| `swap` | 0.033 / 0.033 | 0.0007 / 0.0009 | 0 |
| `same` (control) | 0.017 / 0.017 | 0.0005 / 0.0005 | 0 |
| `seed` | 0.029 / 0.027 | 0.0008 / 0.0008, then 0.0001 | 0 |
| `reversal` (back after 120 ms) | 0.033 / 0.033, twice | 0.0002 / 0.0002 | 0 |
| `move` (focus 100 ms into a column move) | 0.22 / 0.22, twice | 0.0002 / 0.0002 | 0 |
| `beam` (live response) | 0.033 / 0.033 | 0.0008 / 0.0010 | 0 |

Reading:

- The swap is a one-interval cut, with a small residue from the 400 ms
  ring-light crossfade in the next frame. Half of the step is present in the
  same-definition control too: kitty's focused cursor and the ring light
  change on any focus change. The definition swap roughly doubles it.
- `seed` isolates the replaced `MaterialState`: two definitions identical
  but for their names still step by 0.029, and the pinned Aurora field
  visibly jumps to a new pattern (`seed-step.png`). Seed replacement is the
  largest part of the swap's own contribution.
- A reversal inside 120 ms produces two full cuts, not a partial one.
- During a column move, the motion dominates (0.22) and the cut is not
  separable at this sample rate.
- `beam` matches `swap`: at 0.31 s spacing the burst does not catch the beam
  pass, so it is not evidence about the beam.

Visual acceptance is pending: the owner judges `swap`, `same`, `seed`,
`move` and `beam` from the GIFs and step images.

### Recommendation

Keep the hard cut until the owner judges the clip. The composition finding
matters for `material-764d8c` and `material-9be53d`: focus can select a
definition or a named response, and these behave differently.

- **A definition:** a hard cut with a fresh state.
- **A named response:** updated in place, but responses hold only ring
  and signal values.
- **Signal accents:** folded and crossfaded independently of both.

So focus-dependent glass requires two definitions today.

An interpolation design would first have to settle:

- identity keyed on the tile rather than the definition name
- a seed that survives the swap
- the interpolation space for colors and attenuation
- what a blend does with backdrop-blur and the other gates: switch at one
  end, or sample both and crossfade
- rate parameters
- reversal from the current blended value
- a reload or missing material mid-blend
- whether it shares the 400 ms `material-signal` timing with the
  ring-light crossfade
- finite settling

None of these is chosen here.

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
