# Material dynamics scoping brief

Updated 2026-10-15. Handoff for `material-53f873`, not an approved design.

## Problem

Give glass convincing movement and warm changing light while settling completely.
The original pass covers the five oldest eligible ideas in the dynamics lane:
`material-5a5fff`, `material-6d4de5`, `material-1c5a30`, `material-f3e4e4`
and `material-9be53d`. Completed research narrows the movement work; lighting
still needs a bounded experiment before implementation. The follow-up covers
finite transients (`material-d873bf`) together with compositional profiles
(`material-764d8c`) in the [adaptive brief](2026-09-29-adaptive-materials-brief.md).
The earlier lane ideas are related context, not reprocessed members.

## Current behaviour and evidence

### Drag baseline finding (`material-b3ce14`)

`material-4354cf` shipped drag follow-lag (`f8a890d3`); the owner accepted
its clips on 2026-10-02. `src/layout/tile.rs::motion_residual` now combines
native animation residuals and drag lag. `material_dynamics` still caps flex
at one quarter of bevel depth; `jelly_state` saturates it, while ripple
perturbs the normal before refraction. Cap, whole-slab shear and ripple
scaling remain separate questions. The [motion sweep](../materials/2026-09-11-jelly-motion-sweep.md)
measures response and settling, not a preferred deformation.

### Focus swap finding (`material-8e3b73`)

The owner kept hard focus cuts (`7f5ee9e2`, 2026-10-02).
`src/render_helpers/material/mod.rs::apply_resolved` updates one definition
in place and replaces state, seed and offscreen on a name swap; layout motion
survives. Named responses and `material-signal` crossfades already exist.
[Detailed drag and focus findings](2026-10-06-material-dynamics-findings.md)
retain the original traces, parameter inventory and capture provenance.

### Focus-loss boundary

`Tile::update_render_elements` clears `focus_beam` immediately on loss but
also starts `focus_crossfade` from the current focus value to zero. The
moving comet cuts; the resting focus light already fades. The original
transient note's whole-ring snap description therefore overstates the gap.
[The beam contract](../specs/2026-09-19-ring-beam-design.md) §2 and
`docs/materials/material-config.md` document the immediate beam cut;
`src/render_helpers/material/ring.rs` already implements finite tail/decay
and exact rest. A short loss drain is a candidate extension, not implemented
or visually accepted. The optical-only focus-gain view tilt (`material-77db8a`) was
rejected on 2026-10-06; a rigid tilt of window and glass is scoped in the
[transient depth brief](2026-10-06-transient-depth-brief.md).

### Organic light feasibility (`material-ffd61f`)

Traced on `c62df7cb`. Aurora (`src/render_helpers/material/optics/aurora.rs`,
`src/render_helpers/shaders/material/aurora.frag`) adds
`amount × 0.35 × mix(color a, color b, field) × coarse × att^0.2` to the
within stage. Amount and both colors are per-definition constants:
`AuroraOptic::values` reads `ResolvedAurora` directly. Only the phase moves,
one 600 s lap at about 5 px/s; `drift-hz` and reduced motion change the step
size and redraw rate, not the speed. Opaque client pixels bypass the glass, so
an opaque window shows the field only in its bevel band. No response or signal
targets the Aurora stage (`niri-config/src/material/pipeline.rs` lists none),
and an in-place definition edit cuts: `apply_resolved` replaces the config and
bumps. The registry suppresses optic deadlines while settled and holds
rendered values through idle damage (`63702813`).

Existing warm or finite light needs no code. A warm Aurora preset gives
dappled light at a constant overall brightness, settling and following the
motion policy as accepted. A `sweep` impulse with a warm accent is a 1.5 s
diagonal band. `accent-tint` warms the body's hue over `material-signal`
(400 ms), never its brightness. The last two are per-window signal channels.

| Candidate | Reuses | Missing |
| --- | --- | --- |
| Static warm preset | all of it, config only | nothing; answers "warm", not "breaks through" |
| One finite transition | Aurora, `Animation`, the in-place reload path | an envelope carrying amount and colors to new values |
| Weather cycle | the same optic | that envelope, a clock, a cover sequence, an idle rule, a transport |

A weather cycle also carries sustained cost, and a sun "poking through" in one
place needs new shader geometry.

**Recommendation:** one finite overcast-to-sun transition. Its endpoints are
reusable presets now. The transition is the one missing behavior and needs a
reviewed design before code. A weather cycle is a clock driving such
transitions, so it waits on this look and on the clock decision.

- **Trigger:** an in-place change to the resolved Aurora amount or colors of a
  same-named material (config reload, Prism apply). Name swaps keep the
  accepted hard cut (`7f5ee9e2`), `drift-hz` changes cut, and an opening window
  renders resolved values. A change mid-transition starts from the rendered
  values, as `crossfade_origin` does. A change with amount 0 at both ends is
  invisible and starts nothing. Rejected: input resume (it needs dimming
  at idle, reopening the accepted held phase), focus gain (it competes with the
  ring light, `material-ccda38` and `material-abc08c`), and signal accents or
  impulses (a per-window identity channel; ambient ownership is
  `material-f41c54`'s).
- **Duration (D):** 6 s from the last trigger on niri's `Animation`
  (`cubic-bezier 0.42 0 0.58 1`), one `animations` entry and no profile
  system. The review compares 3, 6 and 12 s as timed clips. "The end" below
  is D after the last trigger: D in most scenes, 1.5 D in S4.
- **Reduced/off motion:** a luminance and hue fade is not motion. Like the
  signal crossfade, it fades under `motion "reduced"` and `"off"`;
  `animations { off }` cuts to the target. The drift keeps its existing table.
- **Hidden:** it runs on the shared clock and requests frames only while
  `signal_render_visible`, as the signal crossfade does. A reveal shows the
  current value, with no replay.
- **Input idle:** finite on the ordinary clock, like crossfades, so it ends
  after the threshold and then requests nothing; the drift phase holds as
  accepted. The alternative for the clock question is the optic timeline,
  which holds a change landing during idle until input returns.
- **Pilot scenes:** headless `src/tests` renders, no live desktop. S1: one
  translucent window (`TRANSLUCENT`) with the accepted terminal glass, frames
  at 0, D/4, D/2, 3D/4, D and D + 1 s. S2: a focused and an unfocused window in
  lockstep beside the resting ring light. S3: an opaque client, bevel band
  only. S4: a reverse change at D/2. S5: a window on another workspace during
  the change, revealed after D. S6: the change lands while input is already
  idle. S7: the idle threshold passes at D/2. S8 and S9: S1 under
  `motion "reduced"` and `"off"`. S6–S9 use `drift-hz 4` at both ends, so the
  drift's hold and policy are observable beside the fade.
- **Controls and checks:** neutral is `aurora 0` (the accepted look). The
  overcast and sun controls are the two pilot presets below, rendered with
  no transition. S1–S5 and the controls use `drift-hz 0`, so the transition
  is the only time-varying term. Deterministic checks:
  - Endpoints (S1–S5): the frame before the first trigger equals the overcast
    control, and the frame at the end equals the sun control byte for byte;
    S4's equals the overcast control. S6–S9 drift, so they compare Aurora
    amount and color uniforms instead of bytes.
  - Frame requests (every scene, S6 and S7 included): unchanged pixels do not
    prove quiescence, since crossfades request frames through
    `are_animations_ongoing`, not optic deadlines. That must be true on every
    visible frame before the end, false from the end on, and false while S5's
    slab is out of view. A fade whose frames came from optic deadlines would
    stop requesting them at S6's and S7's idle edge and fail here.
  - Settling (every scene but S8): from the end on, no material deadline arms
    and the fingerprint is constant. S8 keeps input active, so its drift goes
    on stepping at half rate, as accepted.
  - Idle: S6's and S7's Aurora amount and color uniforms equal S1's at every
    sampled time, and they reach the sun preset's at the end with no input
    event. The drift phase holds from the idle edge on. A fade on the paused
    optic timeline would stay at its start in S6 and fail.
  - Motion policy: S8's and S9's amount and color uniforms equal S1's at every
    sampled time. The drift steps at half rate in S8 and stays at phase 0 in
    S9. `animations { off }` reaches the sun control in the first frame and
    requests no frames.
  - Neutral: amount 0 at both ends starts nothing and requests no frames.
- **Review media:** S1's stills judge appearance only. With one easing curve,
  frames at the same fractions of D are identical for every D, so pacing is
  judged on timed clips instead. Each clip renders the deterministic harness
  at a fixed 25 frames per second of clock time and encodes at 25 fps (an
  exact 4 cs GIF delay), so playback runs in real time. There is one clip
  per D (3, 6 and 12 s), each with 2 s held before and after. The live clip
  scripts' screenshot bursts, about three a second, are too coarse for this.
- **Measured cost:** a later `quiet` run, piloted first and following the cost
  lane's method, measures material draws and draw time over D against the
  static control over the same window. It also checks that settled idle power
  matches the static control within the sham floor. Whether to bucket the
  envelope as `drift-hz` does is a design choice with a measured answer.
- **Pilot values, not a chosen look:** overcast
  `aurora 0.12 { drift-hz 0; color "#8f98a3"; color "#c9c1b0"; }`, sun
  `aurora 0.45 { drift-hz 0; color "#ffb347"; color "#ffe0a8"; }`.

Each transition costs D seconds of material draws, so ongoing cost is that
times the event rate, which the transport sets. The
[power re-run](../materials/2026-09-11-idle-budget-evidence.md#re-run-on-48ba40a1)
measured +0.96 W (4 Hz) and +0.77 W (2 Hz) for a drifting Aurora in its pinned
scene, which does not price this. The owner judges the look and the acceptable
ongoing cost at the design's sheet review. Particles, mouse interaction and
time-of-day transport stay with `material-54bcac`, `material-1c5a30` and
`material-f41c54`.

All three original mindful thoughts
were recovered with their canonical `thought:` IDs on 2026-10-06; their lighting,
weather and animation suggestions agree with the captured task bodies. The earlier
unavailable-context report used bare IDs after incorrectly removing `thought:`.
Source lookup guidance is reported as P1 feedback `tasks-bde218`.

## Constraints

Preserve the [render order](../materials/render-pipeline.md), opaque client
pixels, finite settling, visibility and reduced/off motion behavior, and the
[accepted sustained-optic idle contract](../specs/2026-09-29-sustained-optic-settling-design.md).
`material-39a46f`'s power verification and `material-0db905`'s settling
design are complete; do not reopen those questions. Fireflies
`material-54bcac` and ambient transport `material-f41c54` keep their existing
goals. No host capture is needed
for the new feasibility task; later measured runs declare `quiet` and run a pilot first.

## Alternatives

1. **Current lean: reuse existing animations, responses and optics.** Keep
   the accepted focus cut and drag response; examine one warm-light experiment.
   For loss, compare the existing beam cut/light crossfade against one short
   drain before choosing new behavior; avoid a generic profile/state system.
2. Design a targeted deformation or light envelope only after a demonstrated
   gap establishes the required look, timing and checks.
3. Build named animation profiles or a general state machine now. Shelve
   that approach until a wanted behavior exceeds current mechanisms.

## Unanswered questions

- **Deformation:** which current cap, shear or ripple appearance is unacceptable?
  The owner supplies a concrete example before `material-6d4de5` becomes a design task.
- **Lighting:** answered by `material-ffd61f` above: a finite overcast-to-sun
  Aurora transition, whose only missing behavior is the envelope. The owner
  later judges the look and acceptable ongoing cost.
- **Clock:** should weather follow activity, real time or an event, and what
  should input inactivity freeze? An event is the definition change itself.
  Real time must either hold at idle or reopen the settling contract. Activity
  fits the optic timeline, holding during idle and resuming on input. No weather
  clock or ambient-source ownership is chosen here.
- **Focus loss:** does a finite beam drain improve focus readability over
  the current crossfade? What happens after an already-finished beam, rapid
  refocus, a material swap, hide/reveal or reduced/off motion? `material-ccda38`
  frames one candidate and its checks; the owner reviews the design and look.
  It coordinates with the pending focus-gain tilt without assuming acceptance.

## Proposed decomposition

- `material-5a5fff`: propose drop, supported by `material-8e3b73` and
  `7f5ee9e2`; the idea remains open pending disposition.
- `material-6d4de5`: briefed, limited to the remaining deformation questions;
  its original captured request is retained below the updated body.
- `material-1c5a30` and `material-f3e4e4`: `material-ffd61f` traced current
  optics and proposed the overcast-to-sun transition above, with no rendering
  change or capture run. Both ideas carry its finding note. Their next scope pass
  decides whether to file the transition's design task (`planned`).
- `material-9be53d`: shelved until a reproducible wanted behavior and acceptance
  check demonstrate a gap in existing animations, responses or optic clocks.
- `material-d873bf`: briefed; `material-ccda38` (P2 / s / high / planned,
  owner need) designs one finite focus-loss cue against the current baseline.
  Written spec and plan reviews precede code. Default behavior, termination,
  refocus, policy, visibility and neutral-client checks must be explicit;
  keeping current behavior remains an acceptable verdict. Completion updates
  this brief and wakes the original idea with a finding note before the result's branch merges.
  Open/move/resize glints remain in the idea's preserved body, outside this
  first design. No host capture or rendering implementation ran in this pass.
- Reuse the existing lane; no new goal or animation subsystem. Historical
  baseline tasks `material-b3ce14` and `material-8e3b73` are complete.
  Previously shelved `material-e6036d` remains related context, untouched.
