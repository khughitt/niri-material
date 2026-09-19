# Bounded focus and attention ring motion: design

**Status:** draft for review; nothing implemented.

**Task:** `material-82323e`, piece of `material-a76720`. Wakes
`material-0e130e` (implementation) and `material-743692` (Prism contract).
Scoping brief: the ring follow-through brief of 2026-09-18 (git-excluded).

## Context

The within ring ([render-order design](2026-09-12-material-render-order-design.md),
merged as `d5b4188a`) lights the focused window's filament with a travelling
brightness, `focusGlow = focus · 0.7 · (0.55 + 0.45 · travel(angle, drift))`
in `src/render_helpers/shaders/material/main.frag`. `drift` is a phase on a
bucketed absolute clock (`signal::drift`, `DRIFT_PERIOD` = 10 s, `ring-drift-hz`
buckets per second), so a focused window whose slab band is in view redraws
`ring-drift-hz` times per second for as long as it stays focused. That is the
one perpetual redraw the focus ring causes; the crossfade on focus change is
already finite (`FocusCrossfade`, the `material-signal` animation entry).

Attention motion is separate: an IPC-set `motion` (`breathe`, `pulse`,
`flash`) on a window signal drives `signal::breath` on the same kind of
bucketed clock, and `Tile::tick_deadline` reports the next bucket boundary
through one timer per output. Both clocks are gated on the slab band being in
view (`slab_in_view`) and on the tile being rendered at all: deadlines are
reported from `Tile::render` on an `Output` target, so a tile on a hidden
workspace never ticks, and `Niri::redraw` skips rendering while monitors are
inactive, so a DPMS-off output stops ticking after its last armed timer
fires once. Nothing distinguishes an output the user is not looking at, and
nothing knows whether the user is at the keyboard: `Niri::notify_activity`
fires on input but only feeds the client idle-notify protocol.

Two facts shaped the decisions below. `ring-drift-hz` was never a speed: a lap
took 10 s at any setting, and the knob set only how many steps the lap was cut
into. And attention's phase is a function of the absolute clock, so motion can
stop and resume without stored state.

## Decisions

Settled with the owner during design:

- **Focus gain runs one lap of the travelling light, then rests.** The
  travel-then-settle shape keeps the ring's character; a brightness swell or
  a swell plus sweep were rejected as retiring the travel or doubling the
  mechanisms.
- **The lap ends on the pinned pattern.** The sweep takes the phase from 0 to
  2π with an ease-out and lands on the pattern `ring-drift-hz 0` produces
  today, so the settled look is continuous by construction and is the one
  `material-9306b5` tunes against. Fading to a uniform glow was rejected as a
  second eased quantity and a new shader uniform.
- **A lap always completes; a gain only starts one from rest.** Focus loss
  never sweeps; the crossfade out is the loss cue. Losing and regaining focus
  mid-lap restarts nothing. The one exception is a policy change that
  forbids the sweep (§1, cut rule), which cuts the lap to rest at once. No
  other phase jump is possible.
- **Visible means rendered with the slab band in view on any lit output.**
  The existing gate. A second monitor keeps animating attention; that is where
  an attention cue is useful. Focus-output-only and occlusion tests were
  rejected: the first silences the cue where it matters, the second has no
  evidence yet in a tiling layout.
- **Input idle is one compositor-wide threshold, `signal { idle-after-ms }`.**
  Default 30 000; `0` disables. Reusing `cursor hide-after-inactive-ms` or the
  client idle-notify threshold was rejected as the wrong policy or
  nondeterministic.
- **`ring-drift-hz` is retired; `ring-sweep-ms` replaces it; the sweep runs on
  the animation loop.** The crossfade already redraws every frame on focus
  gain (400 ms by default, the `material-signal` easing); the lap rides the
  same path and extends that per-frame window to its own duration, 1500 ms by
  default, once per focus gain and then never. No bucket timers and no
  deadline arm. A stale `ring-drift-hz` is a parse error naming the
  replacement, never a silent reinterpretation. Keeping the Hz as a bucket
  rate or as an opt-in perpetual mode was rejected: the first is visibly
  steppy over a short lap, the second leaves every existing Prism profile on
  perpetual redraw.
- **Reduced motion has no sweep.** `signal { motion reduced }`, `motion off`,
  and `animations { off }` all skip the lap; the crossfade remains. Attention
  keeps its existing reduced mapping (flash → pulse → breathe, off → static).

## 1. The focus sweep

**Phase.** `signal::sweep_phase(started, now, duration) -> f32` is pure:
with `t = now.saturating_sub(started)`, the phase is `2π · ease(t / duration)`
for `t < duration` and `0` otherwise, where `ease(x) = 1 − (1 − x)³`. The
phase approaches `2π` as `t` approaches `duration` and is `0` from `duration`
on; `travel` is 2π-periodic in its phase, so the lap's end and rest are the
same pattern. The subtraction saturates because `Niri::redraw` sets the
unadjusted clock to the predicted presentation time and `clear()` re-fetches
real time on the next iteration, so a sweep started during a redraw can be
sampled at a `now` before its `started`; that sample is `t = 0`, phase `0`,
which is rest, so nothing jumps. `Animation::value_at` makes the same choice.
The per-window seed does not enter: the settled pattern is phase 0 for every
window today (`phase_on` returns 0 at rate 0) and stays so.

**State.** `Tile` gains `focus_sweep: Option<FocusSweep>` with
`started: Duration` (unadjusted clock) and `duration: Duration`, and loses
`focus_drift_hz`. The duration is snapshotted from the response when the
sweep starts, so a config reload that changes `ring-sweep-ms` mid-lap does
not move the phase; the new value applies to the next lap. `FrameInputs.drift_hz`
becomes `FrameInputs.sweep: f32` (the phase, computed by the tile);
`SignalFrame.drift` becomes `SignalFrame.sweep` and still lands in
`mat_sig_focus.y`. `solve` no longer computes a phase. The fingerprint's
`drift_q` becomes `sweep_q` at the same 1/1024 rad quantization: it changes
every frame during the lap and is constant at rest.

**Start rule.** In `Tile::update_render_elements`, when `is_active` turns on
for a tile whose response has `focus ring-light`, `ring-sweep-ms > 0`, the
policy is `full` and animations are on, and `focus_sweep` is `None` or
finished (`now ≥ started + duration`), set `focus_sweep` with `started = now`
and the response's duration. Any other transition leaves it alone. A finished
sweep is cleared in `advance_animations` like a finished crossfade.

**Cut rule.** A lap in progress is cut to rest (`focus_sweep = None`) when
the sweep is no longer permitted: the policy becomes `reduced` or `off`,
`animations { off }` is set, or the response no longer has `focus ring-light`.
The cut happens where the tile absorbs the config change (`refresh_material`
and the options update), not in the start rule. The phase jumps to 0, which
is what niri's `animations { off }` does to every other animation, and it is
the policy the user just asked for.

**Redraw.** `are_transitions_ongoing` reports an unfinished sweep under the
existing `signal_render_visible` guard, exactly as the focus crossfade is
reported. A sweep on a tile whose band is out of view drives no redraws; its
phase is still a function of time, so when the band scrolls into view the
ring shows whatever the lap has reached, usually rest.

**Removed.** `DRIFT_PERIOD`, `signal::drift`, `drift_next_boundary`,
`drift_rate`'s ring caller, the `drift_hz` argument of `signal::tick_deadline`,
`ResolvedResponse::ring_drift_hz`, and the tests that assert drift buckets.
`phase_on`, `next_boundary_on`, and `drift_rate` stay: the aurora optic uses
them for its own `drift-hz`, which this design does not touch.

## 2. Settled focus and the scheduler

The design's acceptance is stronger than "amplitude reaches zero":

- `signal::tick_deadline(eff, in_view, now)` has no focus arm. A settled
  focused tile with a static signal reports no deadline, so the per-output
  timer is never armed on its account.
- `SignalFingerprint::quantize` of a settled focused frame is constant across
  frames, so the offscreen is not re-rendered and no damage is produced.
- `are_transitions_ongoing` is false for a settled focused tile with no other
  transition, so the animation loop does not keep the output awake.

Each of the three is a unit test; §7 adds the redraw-count evidence.

## 3. Attention gating: visibility and activity

**Visibility** is unchanged: `slab_in_view` and the render path. §7 records
the hidden-workspace, out-of-view, and DPMS-off cases as tests so the existing
gate is evidence rather than assumption.

**Activity.** `Niri` gains `last_activity: Duration`, `input_idle: bool`, and
`idle_timer: Option<RegistrationToken>`.

- `Niri::new` initializes `last_activity` to the current monotonic time,
  `input_idle` to false, and arms the timer when the threshold is nonzero, so
  a compositor that receives no input after startup goes idle at the
  threshold like any other quiet period. Without this, a window set to
  breathe at login would animate until the first keypress.
- `notify_activity` records `last_activity`. If `input_idle` was set it clears
  it and calls `queue_redraw_all`, so every output redraws once and re-arms
  its attention timers from the absolute clock. If no idle timer is armed and
  the threshold is nonzero, it arms one for `last_activity + threshold`.
- The timer callback compares `now − last_activity` with the threshold: at or
  past it, set `input_idle`, drop the token, and `queue_redraw_all`; otherwise
  re-arm for `last_activity + threshold`. Input during the wait therefore
  costs no timer churn; `notify_activity` is already deduplicated per loop
  iteration.
- A config reload that changes the threshold recomputes the state, not just
  the timer: `input_idle = threshold > 0 && now − last_activity ≥ threshold`.
  If that differs from the stored flag, store it and `queue_redraw_all`.
  Then cancel any timer and arm one at `last_activity + threshold` when the
  new threshold is nonzero and the compositor is not idle. Raising the
  threshold past the elapsed quiet time therefore wakes attention on the
  spot; lowering it under the elapsed time freezes on the spot; `0` clears
  and cancels.

**Effect.** `Layout` gains `set_input_active(bool)`; the value follows
`is_active` through the `update_render_elements` chain (monitor, workspace,
scrolling, floating) to `Tile::update_render_elements`, which passes it to
`signal::effective(folded, policy, response, input_active)`. When
`input_active` is false the effective `motion` is `Static` after the policy
mapping. Level, accent, and impulses are untouched: an idle window keeps its
static alert indication, a `ping` while idle still ripples once (finite), and
the focus sweep is not gated (finite; a programmatic focus change while idle
costs one lap).

**Resume.** Because `breath` and its boundaries are functions of the absolute
clock, resuming needs no stored phase: the redraw after `input_idle` clears
computes the current bucket and reports the next boundary. Motion resumes in
step with windows that never stopped.

**Effective-signal matrix.** For a window with sustained motion set:

| rendered | band in view | input active | motion | deadline |
| --- | --- | --- | --- | --- |
| no (hidden workspace, DPMS off) | — | — | — | none |
| yes | no | — | static frame | none |
| yes | yes | no | `Static` | none |
| yes | yes | yes | as set | next bucket |

## 4. Native configuration contract

In `response` blocks (`niri-config/src/material/mod.rs`):

- `ring-sweep-ms <int>`: duration of the focus-gain lap in milliseconds.
  Default `1500`; `0` disables the sweep; the upper bound is `10000`, and the
  error reads `ring-sweep-ms must be at most 10000`. Resolves to
  `ResolvedResponse::ring_sweep: Duration`.
- `ring-drift-hz` is no longer a field. The parser rejects it with
  `ring-drift-hz was replaced by ring-sweep-ms; see material-config.md`.
  Response inheritance (`response "still" { … }`) carries `ring-sweep-ms`
  like the other ring fields.

In the `signal` block (`niri-config/src/signal.rs`):

- `idle-after-ms <int>`: input idle threshold. Default `30000`; `0` disables
  the gate; upper bound `3600000` with `idle-after-ms must be at most 3600000`.

No other solver constant is exposed. `ATTACK`, `DECAY_TAU_SECS`,
`IMPULSE_LIFETIME`, and the oscillator periods stay fixed; `material-743692`
records that they remain unexposed until a design asks for them.

## 5. Prism contract

Prism owns `glass.ring.driftHz` in `defs/glass.yaml` and emits `ring-drift-hz`
from `integrations/niri/render.js::responseBlock` into both native materials.
Under this design:

- `glass.ring.driftHz` is replaced by `glass.ring.sweepMs` (integer, `0` to
  `10000`, default `1500`), in the same profile and reset slots; the sink
  emits `ring-sweep-ms` in every response block it writes.
- Persisted settings live in three stores that Prism validates in full on
  every resolve (`resolveLayered` rejects an unknown key in base, in every
  profile, and in every wallpaper context, active or not), so a stored
  `driftHz` blocks every `apply` under the new definitions and a stored
  `sweepMs` blocks every `apply` under the old ones. The migration is an
  explicit command, `prism migrate`, driven by the definition: the
  `glass.ring.sweepMs` entry declares `replaces: glass.ring.driftHz`. The
  command walks base and every context file, and for each occurrence of a
  replaced key removes it and writes the replacing key at its default (the
  old value is a bucket rate and carries no meaning for a duration). Before
  rewriting anything it copies every file it will touch into a timestamped
  directory under Prism's state dir, then reports each file and key changed
  and the backup location. `doctor` reports a pending migration (an orphan
  key that a definition replaces) and names the command. The old key never
  reaches generated config, so `niri validate` accepts the new Prism's output
  against the new build and rejects the old Prism's output with the parse
  error above.
- `idle-after-ms` is not a Prism control: Prism does not own the `signal`
  block. Placement (`ring-inset`, `ring-width`) stays in `prism-d8ee06`;
  `light-ior` stays in `prism-0ea68f`.

**Rollout.** The two sides reject each other's config: the old compositor
rejects `ring-sweep-ms`, the new one rejects `ring-drift-hz`. Prism's `apply`
validates with the *installed* binary (`niri validate`) and reloads the
*running* one (`niri msg action load-config-file`), so installing the new
build alone leaves a running compositor that cannot take the new output, and
restarting alone brings up a compositor that cannot take the old file. The
sequence, in this order:

1. Install the new niri build. The running compositor is untouched;
   `niri validate` now speaks the new contract.
2. Update Prism, run `prism migrate`, and apply. Validation passes against
   the installed binary and the file is written. The reload request succeeds as a request, but the
   running (old) compositor rejects the file, keeps its in-memory config, and
   shows its reload-error notification. From here until step 3 the file on
   disk is ahead of the running compositor and every further apply reports
   the same; this is expected and bounded to the restart.
3. Restart the compositor (log out and in). The new build loads the new file.

Soft failure: a new build restarted on an unmigrated file does not crash. It
runs on the default config with the error notification (`main.rs`
`config_errored`), so no material renders until step 2 is done; running
step 2 then recovers the running compositor through its reload, with no
second restart.

Rollback: restore the migration backup over base and the contexts first
(the previous Prism rejects `sweepMs` in every layer, so without this step
its `apply` fails before reaching niri), then reinstall the previous build
and the previous Prism, apply (the old sink emits `ring-drift-hz`; the
running new compositor rejects it and holds its config, as in step 2), and
restart. The pair must move together in both directions; there is no
version that accepts both keys, by decision.

## 6. Ownership and data flow

- `niri-config`: parse and validate `ring-sweep-ms` and `idle-after-ms`;
  reject `ring-drift-hz`.
- `src/render_helpers/signal.rs`: `sweep_phase`; `effective` takes
  `input_active`; `tick_deadline` loses its drift arm; `FrameInputs` and
  `SignalFrame` rename the phase field.
- `src/layout/tile.rs`: `focus_sweep` start, cut, phase, clearing, and the
  transition report; `signal_for_frame` computes the phase.
- `src/layout/*.rs`: thread `input_active` beside `is_active`.
- `src/niri.rs`: activity timestamp and idle timer, initialized in
  `Niri::new`; `set_input_active`.
- `main.frag`: unchanged; the phase still arrives in `mat_sig_focus.y`.

## 7. Verification

Deterministic, in `just test`:

- `sweep_phase`: 0 at start, monotonic on `[0, duration)`, within `1e-3` of
  `2π` one nanosecond before `duration`, exactly `0` at and after `duration`,
  and `0` for `now < started` (the saturating case). `travel(a, φ)` within
  `1e-5` of `travel(a, φ + 2π)` for sampled angles and phases.
- Start rule: gain from rest starts; gain mid-lap does not restart; loss does
  not start; `ring-sweep-ms 0`, `reduced`, `off`, and `animations off` never
  start. The started sweep carries the response's duration at that moment:
  changing `ring-sweep-ms` mid-lap leaves the phase where it was.
- Cut rule: a policy change to `reduced` or `off`, `animations off`, or a
  response without `focus ring-light` mid-lap clears the sweep and the next
  frame reads phase 0.
- Settled focus (§2): no deadline, constant fingerprint, no transition.
- `effective` with `input_active = false`: motion `Static`, level and accent
  and impulses unchanged.
- Idle timer: activity inside the threshold re-arms without flipping; past
  the threshold flips and queues one redraw; activity clears and queues one
  redraw; threshold `0` never flips; a fresh `Niri` with no input flips at
  the threshold. A reload that raises the threshold past the elapsed quiet
  time clears idle and redraws; one that lowers it under the elapsed time
  sets idle and redraws; one that keeps the state on the same side changes
  nothing. The idle logic is a pure function of
  `(last_activity, now, threshold)` so it is testable without an event loop.
- Config: both new fields parse, default, bound, and inherit; `ring-drift-hz`
  errors with the replacement message.
- Prism `test/niri-render.test.js`: `ring-sweep-ms` in both materials,
  default and override, and no `ring-drift-hz` anywhere in generated output.
  A migration test: base, a profile, and a wallpaper context each holding
  `driftHz` come out holding `sweepMs` at its default, the backup holds the
  originals byte for byte, the report names all three, and a second run
  changes nothing. `doctor` names the command while a replaced key remains.

Headless (`docs/materials/scripts/material-signals-smoke.sh cases`, on the
headless verification host, never the desktop session), Tracy redraw counts
per steady state:

- focused, settled: zero redraws at rest;
- focused, sustained attention, input active: bucket-rate redraws;
- same, after the idle threshold (driven by a synthetic quiet period): zero;
- same, after synthetic input: bucket-rate again, phase in step;
- attention window on a hidden workspace, and with the output DPMS off: zero.

User-reviewed clips, captured under the capture protocol with strict cost
timing (the GPU waiver covers pixels only): focus gain from rest, rapid
alt-tab across three windows, focus loss mid-lap, and idle freeze then
resume of a breathing window. The owner judges the sweep duration and
ease from these; `1500` is the starting value, not a finding.

## 8. Documentation updates when implementation lands

- `docs/materials/material-config.md`: replace the `ring-drift-hz` row and
  prose with `ring-sweep-ms`; give `aurora drift-hz` its own sentence instead
  of "the same rule as `ring-drift-hz`"; add `signal idle-after-ms`.
- `docs/materials/render-pipeline.md`: the focus ring's redraw contract
  (finite lap on the animation loop; no bucket clock).
- `docs/materials/2026-09-02-material-signals-design.md` §4 and §5: note the
  activity gate on sustained motion, with a pointer here.
- `2026-09-05-ring-light-focus-response-design.md` status header: drift
  superseded by this design.
- The smoke scripts that pass `ring-drift-hz` in fixture configs.
- The rollout sequence and rollback (§5) beside the package-pin rollout
  notes, so the next install follows it.

## 9. Decomposition

Implementation is `material-0e130e`, planned, after this spec and its plan
are reviewed. The plan's steps, in order:

1. Config: `ring-sweep-ms`, `idle-after-ms`, `ring-drift-hz` rejection, with
   tests and `material-config.md`.
2. Sweep: `sweep_phase`, tile state and start rule, phase in the frame,
   removal of the drift clock and deadline arm, settled-focus tests.
3. Activity gate: `Niri` timer and flag, `Layout::set_input_active`,
   `effective(input_active)`, tests.
4. Evidence: smoke `cases` runs on the headless host, clips for review,
   the remaining §8 documents.

Prism: one task under `material-743692`'s coordination replacing `driftHz`
with `sweepMs` and migrating profiles (§5), dependent on step 1 landing and
following the §5 rollout order when applied live. `material-743692` itself closes when that
task and `prism-d8ee06` have landed and validated against both materials.

## Alternatives rejected

- **Tune the perpetual drift.** Appearance only; does not settle.
- **Brightness swell via an impulse slot.** Finite and cheap, but retires the
  travelling light and would compete with client impulses for the four slots.
- **Bucketed sweep on the signal timer.** Keeps `ring-drift-hz` parsing, but
  a 1.5 s ease-out at 15 steps per second is visibly stepped, and the knob's
  meaning changes anyway.
- **Perpetual drift as an opt-in.** Every stored Prism profile holds `15`, so
  the settled behavior would reach only users who find the knob.
- **Attention only on the focused output.** Removes the cue exactly where it
  is wanted.

## Non-goals

- Cap redesign (`material-a85a18`), deterministic mid-flex geometry
  (`material-22d78f`), and a general animation or oscillator framework.
- Occlusion tests, per-output attention, or client-visible idle semantics.
- Exposing impulse or oscillator constants.
- The aurora optic's own drift clock and the wider idle budget
  (`material-265eb0`), visibility (`material-7afc31`), and settle-mode
  (`material-f86183`) work; this design coordinates with them by sharing the
  activity flag, not by absorbing them.
