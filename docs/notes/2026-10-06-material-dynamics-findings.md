# Material dynamics findings

Detailed evidence retained from the dynamics scoping brief during its
2026-10-06 refresh. These historical measurements describe their pinned
binaries; they are not measurements of current HEAD.

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
flex off. The owner accepted the drag and release appearance on 2026-10-02.

The follow-lag column comes from Task 2's recorded
`just test-one -p niri drag_dynamics -- --nocapture` run on `d9631ff8`, with
14/14 checks passing. Both columns use jelly-flex 0.01 and bevel 12 / thickness 20.
The settle times measure the last 16 ms sample at or above 1% of phase peak,
not the follower's lag/velocity removal threshold. Hold release was measured
after settling; moving-release continuity has separate deterministic checks.

### Follow-lag clips (`material-55f8a0`)

The 2026-10-02 run captured all five sequences from a TTY with the desktop
stopped, under `$NIRI_MATERIAL_WORK_ROOT/drag-lag-clips-7e80e25f/drag-lag-1388014-1790910915/`
(capture record `capture.json`). Clips: `scroll-fast` (39 frames),
`scroll-slow` (38), `float-fast` (39), `float-slow` (38) and `native` (40), each a
4 s burst at about 100 ms per frame with a GIF and contact sheet. IPC confirmed an
interactive move during the timed segment of all four drags
(`<sequence>: interactive move confirmed during the timed segment` in `clips.txt`).
Review page: <https://claude.ai/artifact/4jJWy7VAMfvHtUFzJxGcb8>.

At 125 Hz the fast segment lasts 192 ms, about two burst frames, and the slow one
480 ms, about five; the lift and release are the better-sampled phases. The first
full attempt on `03f249ec` failed at `float-fast`: the pointer starts at the
output centre, over the centred floating window, so `vdrag` received an enter at
map and ignored the leave the walk's first step produced. It then pressed over
kitty. `vdrag` now clears its entered state on `wl_pointer.leave` (`7e80e25f`).
The earlier 2026-10-01 pilot refused at preflight on host load and captured nothing.
GIF playback is approximate: every frame uses a fixed 80 ms delay while
`frames.txt` records varying request intervals. Judge timing from `frames.txt`,
not GIF duration; neither source measures display cadence.

The fixture pins jelly-flex 0.0066, bevel 12, thickness 20
and ripple off; runs `scroll-fast`, `scroll-slow`, `float-fast`, `float-slow` and
`native`; and verifies the drag enters an interactive move through IPC. The
owner accepted the published clips on 2026-10-02.

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

Verdict (2026-10-02): the owner judged `swap`, `same`, `seed`, `move` and
`beam` from the GIFs and step images and kept the hard cut.

### Recommendation

Keep the hard cut; the owner confirmed it on 2026-10-02. Revisit only if a
new clip shows a problem the seed jump does not already cover. The composition finding
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

