# Follow-lag jelly stimulus for interactive drag

**Status:** draft, awaiting owner review (spec round 1).
**Task:** `material-4354cf`, under the dynamics goal `material-53f873`; extends
`material-b3ce14`.
**Baseline:** `b172940e` (the task-start commits after it change only the task).

## 1. Intent and boundary

The glass should respond to the pointer during a drag, and not only to the
layout's lift and release animations. `material-b3ce14` measured the jelly input
at exactly zero during drag and hold, in both layouts, at any pointer speed
([dynamics brief](../notes/2026-09-29-material-dynamics-brief.md), "Drag
baseline finding"). The cause is an absent stimulus. The renderer and shader
are not at fault: while a window follows the pointer, its tile carries no move
animation, and `Tile::animation_residual` leaves out the grab offset.

This design adds that stimulus and settles the owner decisions the brief left
open. The tile keeps rendering at the pointer. Only the jelly sees a virtual
lag. The design does not change the jelly math, the max-flex cap, whole-slab
shear or ripple levers (`material-6d4de5` keeps those), render order, or how a
window moves on screen.

## 2. What the code establishes

- `jelly_state` (`src/render_helpers/material/mod.rs`) is a pure function of
  the residual it receives. Flex is `tanh`-saturated `jelly-flex × residual`,
  and the ripple gate's `activity` is the residual normalized by half the tile
  diagonal. It holds no state, so any discontinuity in the residual appears
  unchanged as a discontinuity in flex.
- The residual is "render position minus target". During a native move the
  tile trails its target, so the residual points against the motion.
- `Tile::render_offset` is the move animations plus `interactive_move_offset`.
  `Tile::animation_residual` is the move animations alone. Six call sites pass
  `animation_residual()` to rendering or to the unmap snapshot:
  `layout/mod.rs` (the moving tile's render and snapshot),
  `scrolling.rs` (two), `floating.rs` and `workspace.rs`.
- `Layout::interactive_move_update` receives the pointer `delta` in global
  logical coordinates. The `Starting` branch downscales it by the overview
  zoom, and the `Moving` branch recomputes the tile's location from
  `pointer_pos_within_output`.
- Lift (`Starting → Moving`) and release both call `Tile::animate_move_from`.
  The release starts from the tile's current render location (at the pointer)
  toward its slot.
- **`animate_move_from` cannot carry a follower.** On every retarget it
  restarts the spring from rest (`anim.restarted(1., 0., 0.)`, and
  `Animation::restarted` passes the old, not the current, initial velocity for
  springs). If each pointer event fed a move animation, the steady-state lag
  would grow with the event rate. At 8 ms events and 2500 px/s it reaches
  about 900 px, against 177 px for a true follower. Velocity must survive
  pointer events.
- The live window-movement animation is a spring with damping ratio 1,
  stiffness 800 and epsilon 0.0001 (ω = √800 ≈ 28.3 s⁻¹).

## 3. Contract

A critically damped follower chases the pointer during
`InteractiveMoveState::Moving`. The jelly receives its lag `L = follower − target`
as the motion residual, added to the move animations. The tile still renders at
the pointer.

1. **Drive.** For each pointer delta `d` (logical, downscaled by the overview
   zoom), the target moves by `d` and the follower does not move, so `L −= d`.
   The follower's velocity is unchanged.
2. **Integrate.** `L` and its velocity `V` follow
   `L̈ = −k·L − c·L̇`. The spring parameters are those of window-movement
   (§4.3), stepped in closed form from the tile's `Clock`. The step is
   independent of event rate: advancing to `now` and then applying `d` gives
   the same state for one event of `2d` as for two events of `d` at the same
   instant.
3. **Residual.** `motion_residual() = animation_residual() + L`.
   `render_offset()` does not include `L`.
4. **Hold.** With no pointer motion, `L` decays to zero, and the residual,
   flex and ripple activity decay with it.
5. **Release.** The follower state lives on the tile and survives
   `interactive_move_end`. After release `L` keeps decaying from its value at
   the release instant, added to the release move animation. The window does
   not jump, and the lag term stays continuous. The release term itself still
   starts at `tile_render_loc − slot`, exactly as today (§4.4).
6. **Settle.** The tile drops the follower when `|L| < 0.05` px and
   `|V| < 1` px/s. At the live jelly-flex of 0.0066 that is below 0.0004 px
   of flex. Until then the follower counts in `are_animations_ongoing`, so
   frames are scheduled until it settles and no longer.
7. **Off.** No follower exists when window-movement is `off` or uses an easing
   curve (§4.3), during `Starting` (§4.5), or outside an interactive move.

The steady lag at velocity `v` is `2v/ω`: 177 px at the baseline's
40 px/frame (2500 px/s). Read just after a pointer event, the lag also carries
part of that event's step, up to `d`: 187 px at one event per 16 ms frame. That
gives 1.1–1.2 px of flex at the live jelly-flex of 0.0066 and 1.6–1.7 px at
the test's 0.01. The native column-move control peaks
at 1.11 px, so a fast drag bends the glass about as much as a native move.

### Why the tile does not lag

The task's candidate contract started the release `animate_move_from` from
the lagged point. That moves the window on screen by `L` at release: up to
177 px at the baseline speed. Rendering the tile at the follower instead
would make the window trail the pointer during every drag. Either way the
change would alter how the window moves, which this task must not do. Keeping
`L` as a residual-only term and letting it decay on the tile through the
release gives the jelly a continuous lag and leaves window motion exactly as
it is.

## 4. Owner decisions

Each decision gives a recommendation and the alternative rejected. The review
of this spec settles them.

### 4.1 Does a held drag deform?

**Recommendation: no.** The flex decays to zero on a hold within the follower's
settling time: about 0.22 s, to 1% of peak, after a 2500 px/s drag. A permanent deformation while held
would need a non-decaying stimulus. It would hold frames (or freeze a bent
slab) for as long as the button is down, and would break finite settling and
the idle budget. Rejected: a held "carried" sag.

### 4.2 Gain

**Recommendation: reuse `jelly-flex`.** The lag already scales with velocity, and
its magnitude at a brisk drag matches the native move (§3). One gain keeps
one material response. Rejected: a separate `drag-flex`. If review of the clip
shows drag reading too strong or too weak against moves, it can be added
later without changing the follower.

### 4.3 Spring

**Recommendation: window-movement's spring parameters.** The drag then feels
like the layout's own motion, and Prism's existing window-movement setting
tunes both. When window-movement is configured with an easing curve, there is
no follower. An easing curve has no velocity state to carry across retargets,
and restarting it per event reproduces the event-rate defect of §2. `off`
disables it as it disables every move animation. Rejected: a dedicated
`drag-lag` spring config. It adds a knob before anyone has needed it.

### 4.4 Long tiled drops saturating at the cap

**Recommendation: unchanged.** A tiled release still starts from `pointer − slot`
and saturates at the cap, as in the baseline (2.96 px). The follower adds at
most its lag at the release instant, so it neither creates nor removes that
saturation. Reshaping release flex is the max-flex cap question on
`material-6d4de5`. Rejected: scaling the release term here.

### 4.5 Does the rubber band flex?

**Recommendation: no.** In `Starting` the tile only stretches toward the pointer by
a rubber-banded fraction (`limit 0.5`) under a small start threshold. It is
the resistance cue before the lift, and the lift then flexes the glass
through its own animation. A follower there would bend the glass before the
window commits to moving and would add a second stimulus on top of the lift.
Rejected: following the banded offset in `Starting`.

## 5. Components

- **`DragFollower`** (new, `src/layout/drag_follower.rs`): `L` and `V` per axis,
  the spring parameters, and the `Clock` time of its last step. Its operations
  are `step_to(now)`, `shift(d)`, `lag()` and `is_settled()`. It uses the
  closed-form `Spring` solution (`from = L`, `to = 0`,
  `initial_velocity = V`), and the velocity at `dt` comes from the same
  solution. It does not depend on the layout and is unit-tested on its own.
  The clock time comes from `Clock::now()`, so animation slowdown scales the
  follower like every other animation.
- **`Tile`**: holds `drag_follower: Option<DragFollower>`. `drag_follow(d)`
  creates the follower if needed and shifts it. `advance_animations` steps
  it and drops it once settled. `motion_residual()` returns the residual and
  replaces `animation_residual()` at the six call sites, and
  `are_animations_ongoing` includes the follower. It is deliberately left
  out of `are_transitions_ongoing`, which also gates the pointer-focus
  refresh: the lag is cosmetic, and it must not delay pointer focus after a
  drop. The beam uses the same split.
- **`Layout::interactive_move_update`, `Moving` branch:** calls
  `move_.tile.drag_follow(delta.downscale(zoom))` with the delta it already
  receives.

Crossing outputs or workspaces during a drag changes `pointer_pos_within_output`
by an output offset, but `delta` is the real pointer motion. The follower
therefore sees only motion the user made. A window unmapped mid-drag passes
its residual, lag included, to the unmap snapshot through the same
`motion_residual()`.

## 6. Deterministic verification

In `src/layout/tests/drag_dynamics.rs`, on the existing pinned 16 ms clock:

- **Drag flexes.** At 40 px/frame, scrolling and floating both reach a nonzero
  drag peak. The steady lag lies between `2v/ω` and `2v/ω + d`, where `d`
  is the per-frame step. This replaces the
  baseline's assertion that the drag peak is zero.
- **Hold decays to zero.** After the drag stops, the residual falls below the
  settle threshold. The tile then drops its follower, and
  `are_animations_ongoing()` turns false within a bounded number of frames.
- **Release continuity.** At the release frame, `motion_residual()` minus the
  release move animation equals the lag stepped one frame from its
  pre-release value. In floating, where the drop is in place, the whole
  residual is that lag and decays to zero.
- **Rate independence.** The same pointer path, delivered as one event per
  frame and as four events per frame, gives the same residual at each frame
  boundary (within 1e-9).
- **Off paths.** No follower under window-movement `off` or an easing curve,
  none during `Starting`, and the rubber band's residual stays zero.
- **Render untouched.** `render_offset()` and the tile's render location
  during drag are identical with and without the follower.

`DragFollower` unit tests cover the closed form against fine Euler
integration, critically damped decay, a shift with velocity carried, and
the settle rule.

## 7. Clip acceptance

The task's acceptance includes a nested drag, hold and release clip beside
the native column-move control, for the owner to judge. It runs on a headless
weston host, never the desktop session, and goes through the same preflight
and capture-metadata path as `ring-motion-clips.sh`. A scripted drag needs
a held button. `wlrctl` cannot hold one, so the plan supplies a small
`zwlr_virtual_pointer_v1` driver, which niri already serves: Mod+button down,
motion at a fixed cadence and speed, a hold, then release. The glass values
are pinned in the script and recorded with the clips. The pilot runs one
scrolling drag before the full set: scrolling and floating, a slow and a
fast speed, each with a hold, plus the native column-move control.

The verdict is the owner's: whether the drag reads as glass responding to the
hand, and whether release still reads as one motion. The deterministic checks
in §6 hold regardless.

## 8. Alternatives

- **Feed `animate_move_from` per event:** rejected. It makes the lag depend
  on the event rate, and the window trails the pointer (§2, §3).
- **Velocity-derived residual without a spring** (`L = −τ·v`, smoothed): it
  needs its own smoothing and settle rules, and it has no natural handover
  at release. The follower is the same idea with dynamics that already
  match the layout's.
- **Add the lag in the shader:** rejected. The shader sees one frame of
  uniforms and has no velocity history, and it would duplicate state the
  layout owns.
