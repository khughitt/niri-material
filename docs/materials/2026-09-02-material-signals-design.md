# Material signals: design

**Status:** implemented and verified 2026-09-04; accepted feature implementation
`663202b1`, merged into `materials-26.04` at `f9656128`, with the Arch recipe
pinned to that merge and
[acceptance evidence](2026-09-03-material-signals-smoke.md). Compositor-first
slice: signal model, IPC, envelope solver, configuration, and the first glass responses. External sources
(familiar, shell integration, others) are follow-up tasks against the IPC
contract fixed here. Tracked by `material-a54d89`; follow-ups are the
`signals` tagged idea tasks that depend on it.

**Historical rendering note:** the signal model, IPC, and motion contract
remain current. This document's original ring-placement statements are
superseded by the depth-ordered
[render-order design](../specs/2026-09-12-material-render-order-design.md)
and its [acceptance evidence](2026-09-12-material-render-order-evidence.md).

## Context

Before this slice, native glass rendered a window's material from static
configuration plus niri's own motion. It had no signal awareness: it could
not know whether the window wanted attention, which project or agent it
belonged to, or that something had just finished inside it. The only
state-driven behavior was a hard swap between named materials through `match
is-urgent=true`, with no interpolation and no per-window color.

Two external systems want to drive that appearance:

- `familiar` gives every coding-agent session a stable per-project hue and a
  state machine of six states (`idle`, `working`, `needs-input`,
  `needs-approval`, `error`, `done`). Its intent protocol already separates
  identity from state: identity owns color, state owns urgency and motion,
  and decay is pre-resolved as `{current, expiresAt, after}`. Its niri
  integration already joins agent sessions to niri window ids and publishes
  the join; nothing consumes it yet.
- niri itself tracks urgency from xdg-activation (`src/handlers/mod.rs`,
  `MappedWindow::set_urgent`), which is how kitty's bell reaches the
  compositor, and clears it on focus.

`prism` is the static parameter bus. It is global-scalar and daemonless by
design, with per-window addressing and events explicitly deferred. It is the
right owner for tuning constants and the wrong transport for per-window
state.

The renderer already has every hook a state-driven effect needs. Each tile
retains a `MaterialState` with a per-window seed, the material reads the
presentation clock into `mat_jelly_time`, and `InputFingerprint` plus
`JellyFingerprint` gate damage so an idle slab produces no redraws. The jelly
effect is an event-driven animation in all but name: spring residuals in,
quantized uniforms out. Signals reuse that pattern with a different driver.

## Decision

The compositor owns a material-agnostic **signal** per window. Sources write
facts about the window; the compositor folds them, animates them into a
frame of plain scalars, and hands that frame to whichever material the
window has. Materials decide how to draw it. Configuration, not sources,
decides which drawing a window gets.

Three layers, each with one owner:

```text
SOURCES                  TRANSPORT            COMPOSITOR                        MATERIAL
niri urgency/focus  ───  in-process   ──►  WindowSignals: one Slot per     ──►  glass responses:
familiar-niri       ───  niri msg     ──►    source, folded to one Signal        ring, rim orbit,
scripts, watchers   ───  niri msg     ──►  envelope solver ──► SignalFrame       sweep, flash, ripple
prism               ───  config       ──►  response selection (window rules)    (later: other types)
```

The signal vocabulary is deliberately familiar-shaped: the aggregate the
familiar bridge produces for a window is directly representable, with no
lossy translation, while a native bell needs no familiar at all.

## 1. Signal model

Each mapped window carries a `WindowSignals` store: a map from source name
to `Slot`, plus a short impulse queue. The native slot named `niri` is
implicit and derived from compositor state. Every other slot is written over
IPC.

```text
Slot {
    accent:      Option<Color>     // identity hue; state never repaints it
    level:       Quiet | Active | Notice | Demand
    motion:      Static | Breathe | Pulse | Flash
    tag:         Option<String>    // free-form, for config matching
    expires:     Option<{ at: Duration, after: (Level, Motion) }>
    until_focus: bool              // focus demotes the slot to `after`,
                                   // or to (Quiet, Static) when unset
    written_at:  Duration
}
Impulse {
    source: String                 // the slot it belongs to
    kind:   Ping | Done | Error
    accent: Option<Color>
    at:     Duration               // expires at `at + 1.5 s`
}
```

**Time.** Every signal timestamp, deadline, and oscillator phase uses the
unadjusted monotonic clock, `Clock::now_unadjusted`, stored as `Duration`
exactly like `focus_timestamp`. calloop deadlines are unadjusted, so slot
expiry, impulse expiry, and bucket timers agree with the values the solver
computes regardless of the animation slowdown. A timer is armed with
`Timer::from_duration(deadline.saturating_sub(now_unadjusted))`, so a
deadline already in the past fires immediately. Only the baseline crossfade
uses the adjusted `Clock::now`, because it is a niri `Animation` and is
meant to obey slowdown.

An impulse always belongs to an existing slot. `pulse-window-signal` for a
source with no slot is an error; a source states its baseline with `set`
first, even if that baseline is `quiet`/`static`. Clearing a slot drops its
impulses. This keeps one lifecycle: `Window.signal` is `None` exactly when
the window has no slots, and `sources` always accounts for every impulse.

**Ordering.** Slots are ordered by ascending `written_at`, ties broken by
source name. "Earliest" below means first in that order.

**Fold** produces one `Signal` per window:

- `level` is the maximum across slots.
- `motion` comes from the slot that won on level; ties go to the latest
  written of the tied slots.
- `accent` comes from the winning slot if it has one, otherwise from the
  earliest slot that has one. A familiar window keeps its hue while a
  native bell is ringing.
- `tag` uses the same rule independently: the winning slot's tag if it has
  one, otherwise the earliest slot that has a tag. Accent and tag may
  therefore come from different slots.
- Impulses are the union across slots, at most four live entries. A fifth
  pulse evicts the oldest live impulse rather than being rejected: impulses
  are transient, and dropping the newest event in a burst would lose the
  most relevant one.
- `sources` lists every slot name, native slot included, in slot order.

**Decay.** A slot with `expires` set is replaced by its `after` pair at
`expires.at`, keeping `accent` and `tag` and clearing `until_focus`, so a
slot is never demoted twice. Decay is pre-resolved by the source so the
compositor never has to know why a state ends.

**Deadline timer.** Each window owns at most one calloop `Timer`, armed
for the earliest of its slot expiries and impulse expiries, following the
focus-timestamp debounce precedent in `src/niri.rs`. On fire it applies
elapsed decay, prunes expired impulses, marks the window for rule
recomputation, queues a redraw, and emits the event, so a quiet window
notices its own expiry without redrawing while it waits and event-stream
consumers never see a stale impulse. Rendering does not depend on this
timer: the fold is time-aware, so a read at `now` already sees elapsed
decay and omits expired impulses, and a stale or early timer is harmless.
Timers are reconciled from `State::refresh` through a per-window dirty
flag that every mutation sets, so IPC writes, urgency, focus demotion, and
timer-driven decay share one re-arm path; a window's timer is cancelled
when it unmaps.

**Native slot.** The `niri` slot exists if and only if `is_urgent` is
set. A transition into urgent creates it as `Demand + Pulse` and enqueues a
`Ping` impulse; urgency clearing, whether by focus or by the existing
unset action, removes the slot and its impulses. It never uses
`until_focus` or `expires`, so a window with no external sources returns
to `signal: null` the moment it is focused. Focus is not a level: window
rules already distinguish active windows, and focus must not fight
identity.

**Lifecycle.** Slots die with the window. Focus applies `until_focus`
demotion to every external slot that asked for it; demoted slots remain
until their source clears them. A source that wants to clear its
contribution sends the clear request; the compositor never garbage-collects
an external slot on its own.

**Bounds.** Signal state is written by unprivileged local clients, so it is
bounded: at most 16 external slots per window, source names at most 64
bytes, tags at most 256 bytes. A write past one of those bounds is rejected
with an error and changes nothing. The four-impulse cap is the exception
and evicts, as stated under the fold. Each window owns at most one
deadline timer; a rewrite replaces it, and a clear or window close removes
it.
`--ttl-ms` is at most 86 400 000 (24 hours); larger values are rejected so
the deadline is always a valid `Instant`.

## 2. IPC contract

Three requests and one event in `niri-ipc/src/lib.rs`. They are `Request`
variants, not `Action`s: `Action` is the keybind vocabulary, `do_action`
returns nothing, and the server always answers an action with
`Response::Handled`. Signal writes need a reply path, so they follow the
`PickColor` pattern instead: the server posts the mutation to the event
loop through `insert_idle`, waits on a bounded channel for a
`Result<(), String>`, and answers `Handled` or the error.

```text
niri msg set-window-signal --id 12 --source familiar \
    --accent '#e5a33c' --level demand --motion pulse --tag cats/ginger \
    --ttl-ms 30000 --after-level quiet --after-motion breathe --until-focus

niri msg pulse-window-signal --id 12 --source familiar --kind done \
    [--accent '#e5a33c']

niri msg clear-window-signal --id 12 --source familiar
```

Rules:

- `--source` is required, free-form, and case-sensitive. `niri` is reserved
  and rejected.
- `set-window-signal` replaces the whole slot. A source never diffs; it
  states its current truth. Omitted fields take their defaults: no accent,
  `quiet`, `static`, no tag, no expiry, `until-focus` off.
- `--ttl-ms` requires at least one of `--after-level` and `--after-motion`;
  the other defaults to `quiet` or `static`. Either `after` flag without
  `--ttl-ms` is an error.
- Unknown window id, unknown source on `clear`, unknown level or motion or
  kind, malformed color, and any bound in section 1 all return an error
  reply, checked before mutation so a rejected request changes nothing.
  Nothing is silently ignored.
- Colors are `#rrggbb` or `#rrggbbaa`; alpha is accepted and ignored so
  the familiar ramp can be forwarded verbatim.
- `pulse-window-signal` requires an existing slot for `--source`;
  otherwise it is an error.

**Wire schema.** Defined in `niri-ipc` and serialized with the protocol's
existing convention: enum variants keep their Rust names in JSON, as
`Transform` and `WorkspaceReferenceArg` do, and derive `clap::ValueEnum`
so the CLI accepts them in kebab-case. Shown here in CLI spelling:

```text
enum Level       { quiet, active, notice, demand }
enum Motion      { static, breathe, pulse, flash }
enum ImpulseKind { ping, done, error }

struct Signal {
    level:    Level,
    motion:   Motion,
    accent:   Option<String>,      // normalized "#rrggbb", alpha stripped
    tag:      Option<String>,
    sources:  Vec<String>,         // slot order, see section 1
    impulses: Vec<Impulse>,        // live only, oldest first
}
struct Impulse {
    source:     String,
    kind:       ImpulseKind,
    accent:     Option<String>,
    at:         Timestamp,         // existing niri-ipc Timestamp, unadjusted monotonic
    expires_at: Timestamp,
}
```

Request payloads mirror the flags above: `SetWindowSignal { id: u64,
source: String, accent: Option<String>, level: Level, motion: Motion,
tag: Option<String>, ttl_ms: Option<u32>, after_level: Option<Level>,
after_motion: Option<Motion>, until_focus: bool }`,
`PulseWindowSignal { id, source, kind, accent }`, and
`ClearWindowSignal { id, source }`.

The three enums are shared between `niri-ipc` and the compositor. The
compositor's internal slot and fold types use `Duration` and the config
`Color` type; the IPC server converts them to the wire `Signal` when it
builds a `Window`, the same way it converts `focus_timestamp` to
`Timestamp` today. The server's cached `WindowsState` must apply
`WindowSignalChanged` in `niri-ipc/src/state.rs` so `Request::Windows`
returns the current signal and the server's diff does not re-emit the same
change.

The `Event` stream gains `WindowSignalChanged { id, signal: Option<Signal> }`
carrying the folded `Signal`, emitted whenever the fold result changes,
including on decay, focus demotion, impulse expiry, and the clear that
removes the last slot, which sends `None`. The `Window` struct gains the
same `signal: Option<Signal>` field so `niri msg windows` shows it. Both
are additive and serialize as `null` when no slot exists.

The familiar bridge, when it lands in familiar's repo, maps `intent.json`
per session as: `color.base` to `--accent`; `urgency` `none` to `quiet`
(or `active` for `working`), `notice` to `notice`, `demand` to `demand`;
`motion` verbatim; `expiresAt`/`after` to `--ttl-ms` and the `after` flags;
transitions into `done` and `error` to a pulse of the same kind. Familiar's
per-intent `motionPolicy` is applied by the bridge before sending: `off`
sends `static` and no pulses, `reduced` maps Flash to Pulse and Pulse to
Breathe. niri's global `signal { motion }` then applies on top, so the more
restrictive of the two wins and a familiar intent marked `off` never
animates. Several sessions in one window are aggregated by the bridge
before writing a single `familiar` slot. That mapping is out of scope here;
the schema is chosen so the resulting aggregate needs no lossy translation.

## 3. Effective signal, envelope solver, and `SignalFrame`

Per frame, the tile turns the folded `Signal` into a `SignalFrame` of plain
scalars in two stages. Materials see only the frame.

**Stage 1, effective signal (tile).** The tile applies the global motion
policy and the window's resolved response block to the folded signal:

```text
EffectiveSignal {
    accent:   Option<Color>
    level:    Level
    motion:   Motion            // after policy; Static when attention is `none`
    impulses: Vec<(selector: u8, accent: Option<Color>, at: Duration)>
                                // only impulses whose response is not `none`
}
```

`ResolvedResponse` exposes two material-agnostic methods for this:
`attention_is_none()` and `impulse_selector(kind, policy) -> Option<u8>`.
Selector values are ids owned by the material type. They are opaque to
stage 1 and to the solver, which only carry them; the material-specific
helper in section 6 is the one place that interprets them. Everything
downstream, including the transitions term,
timer candidates, and the fingerprint, is computed from the effective
signal, so state that draws nothing costs no recurring redraws. Each
accepted mutation and each deadline-timer firing still queues one
coalescible redraw request per output so the tile can re-evaluate; an
already queued frame absorbs it, and that is the whole cost. The stored slots,
impulse queue, and expiry timers are unaffected, because the event stream
reports stored state.

**Stage 2, solver (pure).** `solve(effective, now_unadjusted, seed,
crossfade) -> SignalFrame` and `next_boundary(motion, now_unadjusted) ->
Option<Duration>` live in `src/render_helpers/signal.rs` with no renderer
or config dependency, following the `PrefilterState` and `jelly_state`
precedent so they are unit-testable without a GPU.

```text
SignalFrame {
    accent:   Option<[f32; 3]>
    level:    f32              // 0 quiet, 1/3 active, 2/3 notice, 1 demand
    breath:   f32              // 0..1 oscillator; 0 when Static
    impulses: [ImpulseFrame; 4]
}
ImpulseFrame { selector: u8, envelope: f32, progress: f32, accent: Option<[f32; 3]> }
                              // envelope 0..1 for intensity; progress = age / 1.5 s, monotonic
```

- **Baseline crossfade.** A change in `level` or `accent` animates through
  niri's `Animation` type under a new `animations { material-signal { … } }`
  entry, so users tune it like every other niri animation and
  `animations { off }` disables it. Default `duration-ms 400`,
  `curve ease-out-cubic`. Accent crossfades in linear RGB.
- **Motion oscillator.** Breathe is a 4 s sine, Pulse a 1.2 s sine, Flash a
  0.5 s square with 50 ms soft edges. Breathe and Pulse phase is clock time
  plus the tile's existing per-window jelly seed, so windows never breathe
  in lockstep. Flash phase is clock time alone: it is an alarm, lockstep is
  acceptable, and a shared phase is what keeps its wakeup bound per output
  rather than per window.
- **Impulse envelope.** 80 ms linear attack, then exponential decay with a
  350 ms time constant, dropped at 1.5 s. `progress` runs from 0 to 1 over
  the same 1.5 s so a response can move monotonically while the envelope
  shapes intensity. Kind selects only the response, not the shape; per-kind
  shape constants are a follow-up if a response needs them.
- **Reduced motion.** A top-level `signal { motion "full" | "reduced" |
  "off" }` block, applied in stage 1. `reduced` maps the sustained motion
  Flash to Pulse and Pulse to Breathe, and maps the impulse response
  `flash` to `sweep`, so no chromatic jolt or strobe reaches the screen;
  `sweep` and `ripple` are single-pass and stay. `off` makes the effective
  motion Static and drops every impulse, leaving the static accent and the
  level crossfade.

## 4. Redraw and damage contract

Fingerprints bound damage, not compositor wakeups. While
`unfinished_animations_remain` is set, the tty backend requeues a redraw on
every vblank, so anything that reports itself as an ongoing transition runs
the render loop at refresh rate for as long as it lasts. The two kinds of
signal motion are therefore driven differently.

- **Transient motion rides the animation loop.** `Tile::are_transitions_ongoing`
  gains a signal term that is true while the baseline crossfade runs or any
  effective impulse from section 3 is live. Impulses last at most 1.5 s;
  the crossfade lasts its configured duration times the global animation
  slowdown, like every other niri animation. Refresh-rate wakeups for those spans match how
  open, move, and resize animations already behave.
- **Sustained motion uses one bucket timer per output.** Breathe, Pulse,
  and Flash do not set the transitions term. Breathe and Pulse boundaries
  are aligned to the unadjusted clock at `period / 32`; the per-window seed
  offsets the value inside a bucket, not the boundary, so every Breathe
  window on an output shares the same boundaries and ten windows cost the
  same wakeups as one. Flash is a square wave with a global phase, so its
  boundaries are only its edges and every Flash window shares them: each
  50 ms edge is sampled at four points, giving eight wakeups per 0.5 s
  period per output and none during the plateaus. Every oscillator
  exposes `next_boundary(now) -> Duration` from the pure solver.
- **Candidates come from the render pass, not the layout walk.**
  `update_render_elements` visits every tile in a visible workspace,
  including hidden tabs and columns scrolled out of view, so it cannot be
  the visibility source. Instead, when `Tile::render` runs for a tile whose
  effective motion is sustained and whose slab band intersects the output's
  view rect, the tile reports its next boundary into a per-output
  accumulator on the render context. The band test uses the tile rect
  inflated by `bevel` on every side, a superset of the exact slab, so a
  visible band never freezes when the tile rect alone leaves view. The
  view rect is set once per output pass, before the interactively moved
  tile renders, so every render path sees it. Hidden tabs never reach
  `Tile::render` and offscreen columns fail the intersection, so neither
  contributes.
  After the pass, the output arms a single calloop `Timer` for the earliest
  reported boundary, replacing the previous one, or removes it when nothing
  reported. The callback queues a redraw for that output; the next render
  pass re-arms. A hidden `Demand` window therefore schedules nothing until
  a workspace switch or the overview actually renders it. Wakeups per output
  are bounded by the union of the bucket grids in use: about 8 per second
  for Breathe alone, about 27 for Pulse, 16 for Flash.
- `InputFingerprint` gains a `SignalFingerprint` covering the complete
  effective frame plus the material helper's derived inputs: `level` and
  each accent channel quantized to 1/256, `breath` quantized to 1/32 of its
  period, and per impulse slot the selector, the envelope and progress each
  quantized to 1/128, and the accent channels at 1/256. Oscillator time is
  pinned to a constant when motion is `Static` and no impulse is live. A
  sweep whose band moves while its envelope stays in one bucket, or a slot
  whose selector changes, therefore always produces damage.
- `MaterialState::advance_commit` is unchanged: the element commit advances
  only when the fingerprint differs, so a quiet window carrying only an
  accent ring costs zero redraws and zero wakeups.

A `Demand` window pulses until focused; that is the intended behavior and
the reason the reduced-motion switch exists. The smoke test in section 9
measures wakeups per second, not just damage, so the timer path is verified
rather than assumed.

## 5. Configuration contract

Response policy lives in two places that already exist. The material
definition says what each signal means for that material; the window rule
picks which response set a window gets.

```kdl
material "terminal-glass" {
    glass { /* unchanged */ }
    response "default" {
        accent    "ring"          // "ring" | "none"
        attention "rim-orbit"     // "rim-orbit" | "ring-pulse" | "none"
        ping      "ripple"        // "ripple" | "flash" | "sweep" | "none"
        done      "sweep"
        error     "flash"
        ring-inset 6
        ring-width 2
    }
    response "loud" {
        attention "ring-pulse"
    }
}

window-rule {
    match app-id="^kitty$" signal-source="familiar" signal-tag="^cats/"
    material "terminal-glass" response="loud"
}

signal { motion "full" }

animations {
    material-signal { duration-ms 400; curve "ease-out-cubic" }
}
```

Rules:

- A material with no `response` block gets a built-in `default` equal to the
  first block above. If any block is present, one must be named `default`;
  its absence is a config error `material <name>: missing response
  "default"`. Duplicate response names are an error.
- Fields omitted from a named block inherit from that material's `default`
  block, so `"loud"` above only overrides `attention`.
- `response` blocks parse by the material's type. The glass response
  vocabulary is the one listed; a future material declares its own names
  and its own dispatcher branch, next to the `glass` child dispatcher the v1
  design already reserves.
- `ring-inset` and `ring-width` are logical pixels. `ring-inset` is measured
  inward from the slab's outer edge. `ring-inset + ring-width <= bevel` is
  validated on the resolved material, mirroring the existing
  `offset > bevel` rule, so the ring lies within the slab. The ring is
  subject to the compositing contract like every other slab term: it shows
  through the exterior band and through translucent window pixels, and is
  hidden under fully opaque window pixels. `material_frame` inflates the
  slab by `bevel - max(|offset|)` and then translates it by the offset, so
  the exterior band on each side is that inflation plus or minus the
  offset. With the defaults, bevel 12 and offsets 6, the band is 12 px on
  the right and bottom and 0 px on the top and left, so an opaque window
  shows the default ring on two sides only, consistent with how the offset
  slab already reads. Terminals with translucent backgrounds show the full
  ring. A full ring behind an opaque window needs
  `bevel >= 2 * max(|offset|) + ring-inset + ring-width`; the
  documentation states this rather than the validator enforcing it, because
  the two-sided look is a legitimate choice.
- `response=` on a window-rule material reference is validated in the same
  pass as material names (`validate_material_refs`), so an unknown response
  for that material is a whole-config error `material <name>: unknown
  response: <response>`. A rule without `response=` gets `default`.
- `signal-source` and `signal-tag` are regex matches added to `Match` next
  to `is-urgent`. `signal-source` matches when any slot on the window,
  including the native `niri` slot, has a source name matching the regex;
  a window is a familiar window whether or not familiar's slot won the
  fold. `signal-tag` matches against the folded `tag` from section 1. A
  window with no slot matches neither. A signal write, decay, or focus
  demotion sets
  `need_to_recompute_rules` the way `set_urgent` does, so `refresh_material`
  re-resolves the material and response without new plumbing. Matching on
  level is deferred; `is-urgent` covers the native case.
- Reload semantics match materials: editing a response in place updates the
  retained `MaterialState`; changing the resolved material name swaps it.

## 6. Glass responses

Glass realizes a `SignalFrame` inside the existing `ProgramType::Material`
program. New uniforms:

```text
mat_sig_accent       vec4    // rgb, w = 1 when accent present
mat_sig_level        float
mat_sig_breath       float
mat_sig_light        vec3    // rim light direction xy, tint weight
mat_sig_impulse_env  vec4    // envelope per impulse slot
mat_sig_impulse_prog vec4    // progress per impulse slot
mat_sig_impulse_rgb  vec3[4]
mat_sig_impulse_resp ivec4   // response selector per impulse slot
mat_sig_response     ivec2   // accent, attention selectors
mat_sig_ring         vec2    // inset, width (logical px, like mat_area_size)
```

Impulse kinds never reach the shader. Stage 1 in section 3 has already
resolved each live impulse to a glass selector (`ripple`, `flash`,
`sweep`) and dropped `none`, so the shader only knows which effect to draw
at which envelope, progress, and color.

All glass-specific interpretation of selectors lives in one pure helper,
`glass_signal_inputs(frame, glass) -> GlassSignalInputs` in
`src/render_helpers/material.rs`, called from the tile's material branch.
It folds `ripple` slots into the jelly activity term, computes the
effective chromatic aberration and distortion from `flash` slots, derives
the sample count from those effective values, and rewrites consumed slots
to a zero-cost `none` selector. Its outputs feed both the uniforms and the
`SignalFingerprint`, so the tile's generic code and the solver never
branch on a selector value. A future material type supplies its own
helper. Response selection is a uniform-int branch, so one program serves
every combination and no permutation compile is needed.

- **`ring`.** A second rounded-box SDF band inside the slab at `ring-inset`,
  `ring-width` wide, added as an emissive term in the accent. Baseline
  brightness is `0.15 + 0.35 * level`; with no accent the band is skipped.
- **`rim-orbit`.** The Fresnel glint currently uses a fixed top-left light
  direction. This response sways `mat_sig_light.xy` around top-left by up
  to `level * π/2`, driven by `breath`, so its cost is covered entirely by
  the breath buckets, and mixes the specular toward the accent by a weight
  equal to `level`. Only `rim-orbit` moves or tints the light; `ring-pulse`
  and `none` keep today's fixed direction. At `Quiet` the light parks
  top-left and the output is bit-identical to today.
- **`ring-pulse`.** The ring's emissive term is multiplied by
  `1 + breath * level`.
- **`sweep`.** A specular band crossing the slab once along the top-left
  to bottom-right diagonal. Position is `progress`, so the band moves
  monotonically and never reverses; intensity is the envelope, so it
  brightens quickly and fades as it travels. Tinted by the impulse accent
  when present. Each sweep-selected slot draws its own
  band, so up to four bands can be in flight, each at its own position and
  accent.
- **`flash`.** Additive, because both defaults are zero and a multiplier
  would be inert on default glass. `glass_signal_inputs` takes `env = max`
  over flash-selected slots and derives the effective values: chromatic
  aberration `min(1, base + 0.5 * env)` and distortion
  `min(1, base + 0.25 * env)`, with `distortion-scale` unchanged. Those
  effective values are what the tile uploads as `mat_chromatic_aberration`
  and `mat_distortion` and what it feeds to `tap_count` for `mat_samples`,
  so the sample budget follows the flashed value and the shader needs no
  flash-specific path. Both are part of the fingerprint through the
  envelope.
- **`ripple`.** In `glass_signal_inputs`, ripple impulse envelopes are
  summed with the native jelly `activity` and clamped to the existing
  `[0, 1)` contract, so an event ripples the slab exactly as a resize does
  and simultaneous ripples saturate rather than overflow.

Fireflies, frost-on-idle, accent tint of attenuation color, and inactive
desaturation are not in this slice. They slot in as new response names
without touching sections 1 through 5.

## 7. Renderer ownership and data flow

| Concern | Location |
|---|---|
| `WindowSignals`, `Slot`, `Impulse`, fold, decay, focus demotion | new `src/window/signal.rs`, stored on `MappedWindow` beside `is_urgent` |
| IPC types (`Level`, `Motion`, `ImpulseKind`, `Signal`, requests, event) | `niri-ipc/src/lib.rs` |
| Event-stream state reduction for `WindowSignalChanged` | `niri-ipc/src/state.rs` (`WindowsState::apply`) |
| CLI subcommands and flag parsing | `src/cli.rs`, `src/ipc/client.rs` |
| Request handling with result channel, `WindowSignalChanged` diffing | `src/ipc/server.rs` next to `PickColor` and `WindowUrgencyChanged`; mutation entry points on `Niri` in `src/niri.rs` returning `Result` |
| Per-window deadline timer, dirty-flag reconciliation, unmap cancellation | `src/niri.rs` (`refresh_signal_deadlines` from `State::refresh`), tokens keyed by window id; cancel called from the unmap paths in `src/handlers/compositor.rs` and `src/handlers/xdg_shell.rs` |
| Per-output oscillator bucket timer | `RenderCtx` in `src/render_helpers/mod.rs` gains a shared next-boundary accumulator with the output view rect; `Niri::render` injects and resets it for output targets; `Tile::render` reports into it; `Niri::redraw` arms or removes the timer after a completed pass, token stored per output |
| Solver, `SignalFrame`, `SignalFingerprint`, oscillator constants | new `src/render_helpers/signal.rs`; pure, no GL |
| Stage 1 effective signal (motion policy, response selection), crossfade `Animation`, transitions term, per-frame wiring | `src/layout/tile.rs`, both `render_inner` material sites and `are_transitions_ongoing` |
| `ResolvedResponse::attention_is_none` and `impulse_selector` | `niri-config/src/material.rs`, per material type |
| Response config, validation, resolved response | `niri-config/src/material.rs` (`Response`, `ResolvedResponse`), `niri-config/src/window_rule.rs` (`Match`, `MaterialRef`), `niri-config/src/lib.rs` (`validate_material_refs`) |
| `signal { motion }` and `animations { material-signal }` | `niri-config/src/lib.rs`, `niri-config/src/animations.rs` |
| `glass_signal_inputs` helper: ripple to activity, flash to effective aberration, distortion, and sample count | `src/render_helpers/material.rs`, pure, tested without a GPU |
| Uniform upload and response selectors | `src/render_helpers/material.rs` (`MaterialRenderElement`) |
| Shader responses | `src/render_helpers/shaders/material.frag` |

The three enums are defined once in `niri-ipc` and reused by the
compositor. The wire `Signal` is built from the internal fold at the IPC
boundary, so there is one conversion, in the server, and it is tested.

## 8. Failure behavior

- Config errors listed in section 5 are whole-config errors, consistent with
  unknown material names today.
- IPC errors are error replies; the slot map and timers are unchanged on any rejected
  request.
- A response name that a material's type does not define is rejected at
  parse time, never at render time, so the renderer has no fallback path.
- A future material type that does not implement signals receives the frame
  and ignores it; the interface requires no material to respond.

## 9. Verification

- Unit tests, `src/tests/material.rs` and a new `src/tests/signal.rs`:
  fold precedence, slot ordering, independent accent and tag selection,
  and the `sources` list; decay, `until_focus` demotion, timer replacement
  on rewrite, and impulse expiry pruning; every bound in section 1
  including the TTL cap; fifth-pulse eviction; stage 1 effective signal
  under each policy and with `none` responses; sweep progress monotonic
  across the envelope; `SignalFingerprint` differs when progress or a
  selector changes while the envelope bucket is constant;
  `glass_signal_inputs` for ripple summation, flash max, and sample count; native slot creation and
  removal with urgency; simultaneous ripple clamping and flash max with
  the derived sample count; bucket alignment across differently seeded
  windows and Flash edge-only boundaries; `WindowsState::apply` for
  `WindowSignalChanged` in `niri-ipc/src/state.rs`; the internal to wire
  conversion; oscillator and envelope shapes at fixed clock values; impulse
  kind to selector resolution; fingerprint pinned to a constant at rest and
  changing under motion; config parse and validation for every error in
  section 5, including `response=` reference checking and the ring rule.
- IPC round trip on a nested instance: set, pulse, clear through
  `niri msg`, asserting the `WindowSignalChanged` events including the
  `null` on final clear, the impulse-expiry event, and the `signal` field
  in `niri msg windows`, plus every rejection case returning an error reply
  with state unchanged, including pulse before set.
- Nested GLES smoke on the headless verification unit, mirroring
  `2026-09-02-material-roughness-smoke.md`: a quiet window with an accent
  ring produces zero material redraws and zero redraw wakeups; a
  `Demand + Pulse` window wakes at the bucket rate, not the refresh rate,
  and stops on focus; the same window on an inactive workspace, as a
  hidden tab, or in a column scrolled out of view produces no recurring
  redraws; ten
  seeded Breathe windows on one output wake at the same rate as one; a
  sustained Flash window wakes only at its edges and ten Flash windows on
  one output wake at the same rate as one; a window with live impulses
  under `signal { motion "off" }` or with every impulse response set to
  `none` produces no recurring redraws, only the coalescible request per
  pulse and per deadline firing; a `Demand + Pulse` window under `signal {
  motion "off" }` arms no timer; a `Demand + Flash` window under
  `reduced` wakes at Pulse bucket boundaries, not Flash edges; a window
  whose response has `attention none` arms no timer; wakeup rates are
  unchanged under a non-unit animation slowdown; a `done` pulse ends and damage stops; `flash` is
  visible on default glass; `signal { motion "off" }` leaves only the
  crossfade. Wakeups are measured from the Tracy redraw frames. Per-draw
  GPU cost is measured on a material-specific GPU zone with an identical
  fixture on this build and on the branch's base commit: the unsignaled
  default path must stay within 10% of the base commit, and the ring plus
  rim-orbit cost is reported against the default path without a threshold.
- Physical DRM check on the daily-driver build before acceptance, because
  this is the first material effect that runs on unfocused windows for long
  stretches.

## 10. Documentation updates when implementation lands

- `docs/materials/material-config.md`: `response` block, glass response
  vocabulary, `ring-inset`/`ring-width`, `signal-source`/`signal-tag`
  matches, `response=` on material references, `signal { motion }`, and the
  `material-signal` animation.
- `docs/wiki/IPC.md` or the fork's IPC notes: the three requests, the
  event, and the `signal` field.
- `docs/materials/README.md`: this design and its evidence entries.
- This document's status header.

## 11. Follow-up tasks

Registered as idea tasks tagged `signals`, each depending on
`material-a54d89` and carrying the notes from the design discussion.

Glass responses:
- fireflies: procedural points drifting inside the slab thickness, refracted
  and accent-tinted, densest near the edges; the "dancing" attention state
  for windows with no identity.
- frost-on-idle: roughness rises slowly at `Quiet`, clears with a wipe on
  focus or `Active`.
- accent tint: mix `attenuation-color` toward the accent as an identity
  response.
- inactive desaturation: post-process on the window texture, not the glass,
  so focus reads even when the material is quiet.
- progress fill on the ring, driven by a future progress channel.

Sources:
- familiar bridge in familiar's `integrations/niri`, using the mapping in
  section 2.
- OSC 133 shell-integration watcher: command exit status as `done` or
  `error` impulses, no familiar required.
- OSC 9;4 progress and OSC 9/99 notifications.
- privacy indicators from `is-window-cast-target` and microphone state.
- per-pid audio activity from PipeWire.
- ambient baseline from time of day or scheme tone.

Prism:
- ring geometry, oscillator periods, and impulse constants as ordinary glass
  params in `defs/glass.yaml`, rendered into the `response "default"` block
  by the niri sink.

Deferred design questions:
- matching window rules on signal level.
- per-workspace signal summaries in the event stream for bars.
- per-kind impulse envelope shapes.

## Alternatives rejected

### Sources choose the effect

The signal could carry an effect name. That couples every source to each
material's vocabulary and puts rendering policy in familiar's theme files.
Compositor configuration owns policy instead; sources state facts.

### Prism as the transport

Prism's file bus is global-scalar and write-path-only. Per-window addressing
and a daemon fast path are explicitly deferred in its design, and adding
them for this consumer would reintroduce the external-client-chasing-IPC
model the fork exists to replace. Prism keeps the static tuning role.

### Signal writes as `Action`s

Making the three writes `Action` variants would let keybinds send them and
would reuse `validate_action`, but `do_action` has no return value and the
server always replies `Handled`, so unknown ids and bound violations could
not be reported. Nothing needs a keybind to write a signal; requests with a
result channel keep the error contract.

### Sustained motion on the animation loop

Reporting Breathe or Pulse as an ongoing transition is the least code, but
`unfinished_animations_remain` requeues every vblank, so a single
`Demand` window would hold the output at refresh rate indefinitely while the
fingerprint discarded most of those frames. Bucket timers wake the
compositor only when the uniform changes.

### Shader-side envelopes

Passing raw timestamps and solving envelopes in GLSL would keep the CPU
uninvolved but would defeat damage gating: the fingerprint could not know
when the envelope is done. CPU-solved, quantized frames keep the existing
zero-cost-at-rest guarantee.

### A separate `signal-rule` block

A new rule block would need its own matching, its own recompute trigger,
and a runtime fallback when a selected response does not exist on the
window's material. Putting `response=` on the window-rule material
reference reuses window-rule recomputation and validates at parse time.

## Non-goals

- Any change to familiar, prism, kitty, or ghostty in this slice.
- Signals on layer surfaces or per-workspace signals.
- Continuous channels beyond the four levels and four motions.
- Per-source rate limiting; sources are expected to write at state-change
  rate, and the fingerprint bounds redraw cost regardless.
- A second material type.
