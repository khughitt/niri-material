# Follow-lag jelly stimulus for interactive drag

**Status:** accepted (spec round 6); follower implemented in `material-4354cf`;
capture preflight refused on host load; no clips captured, owner visual judgment
still required after a quiet-host retry.
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

A spring follower chases the pointer during `InteractiveMoveState::Moving`. The jelly receives its lag `L = follower − target`
as the motion residual, added to the move animations. The tile still renders at
the pointer.

1. **State.** The follower stores an anchor: the clock time `t₀` and the lag
   `L₀` and velocity `V₀` at that time. `L(t)` and `V(t)` are the closed-form
   solution of `L̈ = −k·L − c·L̇` from that anchor, using window-movement's
   spring parameters (§4.3). The follower supports every damping ratio, not
   only 1, because `oscillate` already has under-, critically and overdamped
   branches.
2. **Read lazily.** Every read (`lag()`, `is_settled()`) evaluates the solution
   at `clock.now()`. Nothing steps the follower in `advance_animations`. Frames
   render at the predicted presentation time, after `advance_animations` ran
   at the current time, and the move animations it adds to are read at that
   presentation time too. A stepped follower would lag them by part of a frame.
3. **Backwards time.** `Clock::now()` can step backwards. After a frame sets
   the presentation time and `clear()` runs, the next input event reads the
   earlier real time, and several outputs set different targets in one
   loop. The closed form is not evaluated at negative `dt`: its decaying
   exponentials grow backwards in time. A probe at damping ratio 10 and
   stiffness 1e8 overflowed to infinity on a 16 ms backwards read. Time is
   therefore clamped at the anchor. A read at `now < t₀` returns the anchor
   state `(L₀, V₀)`, and the anchor time never decreases (§3.4). Every read
   is a forward evaluation over `dt = max(now − t₀, 0)` and stays finite.
   The cost is at most a frame's presentation lead of extra decay already
   applied, which is invisible.
4. **Drive.** For each pointer delta `d` (logical, downscaled by the overview
   zoom), the follower re-anchors at `t = max(now, t₀)`: `t₀ = t`,
   `L₀ = L(t) − d` and `V₀ = V(t)`. A shift that arrives before the anchor
   applies its `d` at the anchor, so the anchor time is monotonic. The target moves by `d`, the follower
   does not, and its velocity carries across the event. Events at the same
   instant compose exactly, and events at different instants match the
   continuous integration of the delivered events.
5. **Residual.** `motion_residual() = animation_residual() + L`.
   `render_offset()` does not include `L`.
6. **Hold.** With no pointer motion, `L` decays to zero, and the residual,
   flex and ripple activity decay with it.
7. **Release.** The follower state lives on the tile and survives
   `interactive_move_end`. After release `L` keeps decaying from its value at
   the release instant, added to the release move animation. The window does
   not jump, and the lag term stays continuous. The release term itself still
   starts at `tile_render_loc − slot`, exactly as today (§4.4).
8. **Settle.** The follower is settled when `|L(now)| < 0.05` px and
   `|V(now)| < 1` px/s, and `advance_animations` then drops it. At the live jelly-flex of 0.0066 that is below 0.0004 px
   of flex. `are_animations_ongoing` tests that a follower is present, not
   `!is_settled(now)`. `advance_animations` runs at real time before the
   presentation-time read, so presence guarantees the one further frame that
   drops it, and frames are scheduled until it settles and no longer.
9. **Off.** Only `Moving` drives a follower. `Starting` (§4.5) and anything
   outside an interactive move never create or shift one. No follower is
   created when window-movement is `off` or uses an easing curve (§4.3).
   While the clock completes animations instantly
   (`should_complete_instantly()`, the same check `Animation::is_done`
   makes), `lag()` reads zero, `drag_follow` creates nothing, and
   `advance_animations` drops any live follower. A follower therefore
   completes with every other animation under `animations { off }` and
   `Op::CompleteAnimations`.
10. **Lifetime.** A follower ends only by settling or by a configuration
    change (§5, `update_config`). It ignores `stop_move_animations`, which
    resets position bookkeeping at lift and on a scrolling-to-floating move.
    The lag is a decaying stimulus, not a position. A tile grabbed again while
    its lag is still decaying keeps decaying through `Starting`, and `Moving`
    shifts the same follower, so flex never jumps to zero on a regrab.

Under a steady drag at velocity `v` with critical damping, the lag is a
sawtooth of ±`d/2` around `2v/ω`. It is largest just after each event and
smallest just before the next. At the baseline's 40 px/frame (2500 px/s, one
event per 16 ms frame) it runs from 157 to 197 px around 177 px. That gives
1.00–1.22 px of flex at the live jelly-flex of 0.0066 and 1.44–1.73 px at the
test's 0.01. The native column-move control peaks at 1.11 px, so a fast drag
bends the glass about as much as a native move. At realistic event rates
(8 ms and 4 ms events) the sawtooth narrows to ±10 px and ±5 px.

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

**Recommendation: leave the cap and the release trajectory unchanged.** A
tiled release still animates the window from `pointer − slot`, and the cap
still bounds the flex. Release flex itself can change, though. The residual
is the release term plus the live lag, and the lag points against the
pointer's last motion. When the drop slot lies behind the drag direction
(the user overshoots and the window springs back), the lag opposes the
release term. Flex at the release can then be lower than the baseline's,
cancelled, or briefly reversed. When the slot lies ahead, the lag adds and
the flex saturates sooner. Both follow from treating the follower as one
more motion stimulus, and the clip review (§7) judges them. Reshaping
release flex is the max-flex cap question on `material-6d4de5`. Rejected:
scaling or clamping the release term here.

### 4.5 Does the rubber band flex?

**Recommendation: no.** In `Starting` the tile only stretches toward the pointer by
a rubber-banded fraction (`limit 0.5`) under a small start threshold. It is
the resistance cue before the lift, and the lift then flexes the glass
through its own animation. A follower there would bend the glass before the
window commits to moving and would add a second stimulus on top of the lift.
Rejected: following the banded offset in `Starting`.

## 5. Components

- **`DragFollower`** (new, `src/layout/drag_follower.rs`): its only state is
  the anchor `(t₀, L₀, V₀)`, with `L₀` and `V₀` per axis, and the spring parameters. Its operations
  are `shift(now, d)`, `lag(now)`, `is_settled(now)` and `set_params(now, p)`,
  each taking a time so the type stays pure. Position and velocity come from
  the follower's own closed form for all three damping regimes, not from
  `Spring::value_at`. `Spring::oscillate` writes the overdamped branch as
  `e^(−βt) · cosh(ω₂t)`. At large `t` the exponential underflows to 0 and
  `cosh` overflows to infinity, and their product is NaN. A direct probe of
  damping ratio 10, stiffness 800 and a lag of −40 px gave infinity at
  2.5 s and NaN at 3 s. A NaN lag never compares as settled, so the tile
  would schedule frames forever. The follower writes the overdamped case as
  `C₁e^(λ₁t) + C₂e^(λ₂t)`, with `λ₁,₂ = −β ± ω₂` both negative. It writes
  the critically and underdamped cases in their usual bounded forms, with
  velocity as the analytic derivative of each. `Spring` is upstream code,
  so it stays unchanged here. Whether its own overdamped animations can
  reach the overflow is a separate question. A non-finite `L` or `V` is a bug: debug builds assert on it.
  The follower does not depend on the layout and is unit-tested on its
  own. The tile
  passes `Clock::now()`, so animation slowdown scales the follower like every
  other animation.
- **`Tile`**: holds `drag_follower: Option<DragFollower>`. `drag_follow(d)`
  creates the follower if needed and shifts it. `advance_animations` drops it
  once settled or under complete-instantly (§3.9), and never steps it.
  `update_config` drops it when window-movement becomes `off` or an easing
  curve. It re-anchors it with the new parameters when the spring changes,
  keeping `L` and `V` continuous. Complete-instantly is not checked there,
  because a reload calls `layout.update_config` before it sets the clock's
  complete-instantly flag. `motion_residual()` returns the residual and
  replaces `animation_residual()` at the six call sites, and
  `are_animations_ongoing` includes the follower. It is deliberately left
  out of `are_transitions_ongoing`, which also gates the pointer-focus
  refresh: the lag is cosmetic, and it must not delay pointer focus after a
  drop. The beam uses the same split.
- **`Layout::interactive_move_update`, `Moving` branch:** calls
  `move_.tile.drag_follow(delta.downscale(zoom))` with the delta it already
  receives.

Crossing outputs or workspaces during a drag changes `pointer_pos_within_output`
by an output offset. `delta` is the pointer's own `new − last` location in the
move grab, once per pointer frame, so the follower sees only motion the user
made. A pointer warp during a drag would feed its jump into the follower and
produce one transient. That is accepted, because warps during a grab are
rare. `L` is in workspace (unzoomed) units, so an animating overview zoom
rescales the on-screen lag with everything else. That is accepted too. A window unmapped mid-drag passes
its residual, lag included, to the unmap snapshot through the same
`motion_residual()`. The snapshot holds that value through the close
animation, as it already does for move residuals. Making snapshots decay is
out of scope.

## 6. Deterministic verification

In `src/layout/tests/drag_dynamics.rs`, on the existing pinned 16 ms clock. The
harness applies an update, advances 16 ms, then records, so a recorded drag
sample is the one just before the next event.

- **Drag flexes.** At 40 px/frame, scrolling and floating (step (40, 10))
  both reach a nonzero drag peak. Every recorded drag frame matches a
  reference `DragFollower` driven through the same events and clock times
  within 1e-9 px. That comparison needs no convergence, so the existing
  15-frame phases stay as they are. A follower that resets `V` on each
  event records about 485 px and fails the comparison.
- **Hold decays to zero.** After the drag stops, the residual falls below the
  settle threshold. The tile then drops its follower, and
  `are_animations_ongoing()` turns false within a bounded number of frames.
- **Release continuity.** A separate trace in each layout releases during
  motion, in the same frame as the last drag step. The existing traces hold
  for 30 frames first, which lets the follower settle. The test asserts a
  nonzero lag (above 10 px) just before `interactive_move_end`. At the
  zero-time read straight after the end, before any advance,
  `motion_residual()` minus the release move animation's residual equals
  `lag(now)` read just before the end. One frame later it equals that lag
  evaluated 16 ms on. In floating, where the drop is in place, the whole
  residual is the lag, and it decays to zero.
- **Opposing release.** A scrolling drag that moves away from the drop
  slot and releases in motion, so the lag opposes the release term, records
  release flex below the same release with the follower disabled. A drag
  toward the slot records release flex at least as large. Neither changes
  the window's release trajectory: `render_offset()` is identical with and
  without the follower.
- **Rate independence.** One pointer path is delivered once as one event per
  frame and once as four events per frame, staggered at 4 ms with
  `set_unadjusted`. The residual at each frame boundary matches a reference
  follower stepped through those exact times (within 1e-9), and matches a
  fine Euler integration of the same path (within 1e-3 px).
- **Backwards clock.** After the clock steps back, a read returns the anchor
  state exactly and leaves the anchor unchanged. A shift applies at the
  anchor time, and the anchor time never decreases. The boundary is
  continuous: a read at `t₀` and a read 1 ns after it agree within 1e-6 px.
- **Regrab.** A tile grabbed again while its lag decays keeps the same
  follower through `Starting` with a continuous lag, and `Moving` then
  shifts it. `stop_move_animations` leaves it in place.
- **Off and reload paths.** On a fresh tile, a window-movement `off` or easing
  config, or `Starting` alone, never creates a follower, and the rubber
  band's residual stays zero. Mid-drag, switching window-movement to `off`
  or easing drops the follower on the next `update_config`, and a spring
  parameter change keeps `L` continuous. With complete-instantly set,
  `Op::CompleteAnimations` drops a live follower and the lag reads zero.
- **Render untouched.** `render_offset()` and the tile's render location
  during drag are identical with and without the follower.
- **Replaced baseline assertions.** These assert the old zero stimulus and are
  rewritten to the checks above: the scrolling drag and hold peaks of zero
  (`drag_dynamics.rs`, around line 191), and floating's drag, hold and
  release peaks below 1e-9 (around lines 273–275). The table in the dynamics
  brief gets a column for the new run.

`DragFollower` unit tests cover the closed form against fine Euler
integration for damping ratios 0.6, 1 and 1.5, the velocity against a
finite difference of the position, the pre-anchor clamp, a shift that carries
velocity, the settle rule, high damping, and the steady drag. The
high-damping test (damping ratio 10, stiffness 800, lag −40 px) requires a
finite lag and velocity at every 16 ms sample out to 60 s, agreement with
fine Euler integration, and settling in bounded time. At damping ratio 10
and stiffness 1e8, reads 16 ms before the anchor return the finite anchor
state. The steady-drag test drives 50
events of 40 px, each followed by a 16 ms read, at critical damping. The
recorded lag must reach the discrete fixed point within 1e-6 px (156.79 px,
computed in the test by iterating the closed form). It lives here, not in
the layout test, because 2000 px of pointer travel would leave the
1280×720 test output.

## 7. Clip acceptance

The task's acceptance includes a nested drag, hold and release clip beside
the native column-move control, for the owner to judge. It runs on a headless
weston host, never the desktop session, and goes through the same preflight
and capture-metadata path as `ring-motion-clips.sh`. The driver is a move client:
it maps its own window, presses a virtual-pointer button on it and calls
`xdg_toplevel.move`. The pinned Smithay does not let virtual-keyboard modifiers
reach niri's Mod check (plan review round 1). The glass values are pinned in the script and recorded with the clips. The pilot runs one
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
