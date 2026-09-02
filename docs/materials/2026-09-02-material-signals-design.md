# Material signals: design

**Status:** accepted 2026-09-02 on `feat/material-signals`; not yet
implemented. Compositor-first slice: signal model, IPC, envelope solver,
configuration, and the first glass responses. External sources (familiar,
shell integration, others) are follow-up tasks against the IPC contract
fixed here. Tracked by `material-a54d89`; follow-ups are the `signals`
tagged idea tasks that depend on it.

## Context

Native glass renders a window's material from static configuration plus
niri's own motion. Nothing in the material knows whether the window wants
attention, which project or agent it belongs to, or that something just
finished inside it. The only state-driven behavior available today is a hard
swap between named materials through `match is-urgent=true`, with no
interpolation and no per-window color.

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

The signal vocabulary is deliberately familiar-shaped, so the familiar bridge
is a one-to-one field mapping, while a native bell needs no familiar at all.

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
    expires:     Option<{ at: Instant, after: (Level, Motion) }>
    until_focus: bool              // focus demotes the slot to `after`,
                                   // or to (Quiet, Static) when unset
    written_at:  Instant
}
Impulse { kind: Ping | Done | Error, accent: Option<Color>, at: Instant }
```

**Fold** produces one `Signal` per window:

- `level` is the maximum across slots.
- `motion` comes from the slot that won on level; ties go to the most
  recently written slot.
- `accent` comes from the winning slot if it has one, otherwise from the
  first slot in write order that has one. A familiar window keeps its hue
  while a native bell is ringing.
- `tag` follows the same rule as `accent`.
- Impulses are the union of all sources, capped at four live entries; the
  oldest is evicted first.

**Decay.** A slot with `expires` set is replaced by its `after` pair at
`expires.at`, keeping `accent` and `tag`. Decay is pre-resolved by the
source so the compositor never has to know why a state ends. Each write
with an expiry registers a calloop `Timer` on the event loop, following the
focus-timestamp debounce precedent in `src/niri.rs`; on fire it applies the
decay, marks rules for recomputation, queues a redraw, and emits the event.
A quiet window therefore notices its own expiry without redrawing while it
waits. The fold also compares the clock, so a stale timer or a slot
rewritten before it fired is harmless.

**Native slot.** `is_urgent` becomes `Demand + Pulse` with `until_focus`.
A transition into urgent also enqueues a `Ping` impulse. Focus is not a
level: window rules already distinguish active windows, and focus must not
fight identity.

**Lifecycle.** Slots die with the window. Focus applies `until_focus`
demotion to every slot that asked for it. A source that wants to clear its
contribution sends the clear action; the compositor never garbage-collects
a live source's slot on its own.

## 2. IPC contract

Three actions and one event, shaped like the existing urgency actions in
`niri-ipc/src/lib.rs`.

```text
niri msg action set-window-signal --id 12 --source familiar \
    --accent '#e5a33c' --level demand --motion pulse --tag cats/ginger \
    --ttl-ms 30000 --after-level quiet --after-motion breathe --until-focus

niri msg action pulse-window-signal --id 12 --source familiar --kind done \
    [--accent '#e5a33c']

niri msg action clear-window-signal --id 12 --source familiar
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
  kind, and malformed color all return an error reply. Nothing is silently
  ignored.
- Colors are `#rrggbb` or `#rrggbbaa`; alpha is accepted and ignored so
  the familiar ramp can be forwarded verbatim.

The `Event` stream gains `WindowSignalChanged { id, signal }` carrying the
folded `Signal`, emitted whenever the fold result changes, including on
decay and focus demotion. The `Window` struct gains an optional `signal`
field with the same shape so `niri msg windows` shows it. Both are additive
and serialize as `null` when no slot exists.

The familiar bridge, when it lands in familiar's repo, maps `intent.json`
per session as: `color.base` to `--accent`; `urgency` `none` to `quiet`
(or `active` for `working`), `notice` to `notice`, `demand` to `demand`;
`motion` verbatim; `expiresAt`/`after` to `--ttl-ms` and the `after` flags;
transitions into `done` and `error` to a pulse of the same kind. Several
sessions in one window are aggregated by the bridge before writing a single
`familiar` slot. That mapping is out of scope here; the schema is chosen so
it is one-to-one.

## 3. Envelope solver and `SignalFrame`

Per frame, the tile turns the folded `Signal` into a `SignalFrame` of plain
scalars. Materials see only the frame.

```text
SignalFrame {
    accent:   Option<[f32; 3]>
    level:    f32              // 0 quiet, 1/3 active, 2/3 notice, 1 demand
    breath:   f32              // 0..1 oscillator; 0 when Static
    impulses: [ImpulseFrame; 4] // kind, envelope 0..1, accent
}
```

- **Baseline crossfade.** A change in `level` or `accent` animates through
  niri's `Animation` type under a new `animations { material-signal { … } }`
  entry, so users tune it like every other niri animation and
  `animations { off }` disables it. Default `duration-ms 400`,
  `curve ease-out-cubic`. Accent crossfades in linear RGB.
- **Motion oscillator.** Breathe is a 4 s sine, Pulse a 1.2 s sine, Flash a
  0.5 s square with 50 ms soft edges. Phase is clock time plus the tile's
  existing per-window jelly seed, so windows never pulse in lockstep.
- **Impulse envelope.** 80 ms linear attack, then exponential decay with a
  350 ms time constant, dropped at 1.5 s. Kind selects only the response,
  not the shape; per-kind shape constants are a follow-up if a response
  needs them.
- **Reduced motion.** A top-level `signal { motion "full" | "reduced" |
  "off" }` block. `reduced` maps Flash to Pulse and Pulse to Breathe. `off`
  pins `breath` to zero and discards impulses, leaving the static accent
  and the level crossfade.

The solver is pure: `solve(signal, clock_now, seed, animation_state) ->
SignalFrame`, with no renderer dependency, following the `PrefilterState`
and `jelly_state` precedent so it is unit-testable without a GPU.

## 4. Redraw and damage contract

Signals follow the jelly discipline exactly.

- `Tile::are_transitions_ongoing` gains a signal term: true while the
  baseline crossfade runs, while the folded motion is not `Static`, or while
  any impulse is live. The tile therefore rides niri's existing
  `are_animations_ongoing` redraw loop; no new timer is introduced.
- `InputFingerprint` gains a `SignalFingerprint`: `level` and each accent
  channel quantized to 1/256, `breath` quantized to 1/32 of its period, each
  impulse envelope quantized to 1/128, and oscillator time pinned to a
  constant when motion is `Static` and no impulse is live.
- `MaterialState::advance_commit` is unchanged: the element commit advances
  only when the fingerprint differs, so a quiet window carrying only an
  accent ring costs zero redraws.

Expected redraw rates from the quantization: Breathe about 8 per second,
Pulse about 27, Flash bounded by the 50 ms edges, impulses about 128 over
1.5 s then nothing. A `Demand` window pulses until focused; that is the
intended behavior and the reason the reduced-motion switch exists.

## 5. Configuration contract

Response policy lives in two places that already exist. The material
definition says what each signal means for that material; the window rule
picks which response set a window gets.

```kdl
material "terminal-glass" {
    glass { /* unchanged */ }
    response "default" {
        accent    ring          // ring | none
        attention rim-orbit     // rim-orbit | ring-pulse | none
        ping      ripple        // ripple | flash | sweep | none
        done      sweep
        error     flash
        ring-inset 6
        ring-width 2
    }
    response "loud" {
        attention ring-pulse
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
  `offset > bevel` rule, so the ring always lies in the bevel band and stays
  visible behind an opaque window.
- `response=` on a window-rule material reference is validated in the same
  pass as material names (`validate_material_refs`), so an unknown response
  for that material is a whole-config error `material <name>: unknown
  response: <response>`. A rule without `response=` gets `default`.
- `signal-source` and `signal-tag` are regex matches on the folded
  `Signal`, added to `Match` next to `is-urgent`. A window with no slot
  matches neither. A signal write, decay, or focus demotion sets
  `need_to_recompute_rules` the way `set_urgent` does, so `refresh_material`
  re-resolves the material and response without new plumbing. Matching on
  level is deferred; `is-urgent` covers the native case.
- Reload semantics match materials: editing a response in place updates the
  retained `MaterialState`; changing the resolved material name swaps it.

## 6. Glass responses

Glass realizes a `SignalFrame` inside the existing `ProgramType::Material`
program. New uniforms:

```text
mat_sig_accent      vec4   // rgb, w = 1 when accent present
mat_sig_level       float
mat_sig_breath      float
mat_sig_light       vec3   // rim light direction xy, tint weight
mat_sig_impulse_env vec4   // envelope per impulse slot
mat_sig_impulse_rgb vec3[4]
mat_sig_response    ivec4  // accent, attention, ping, done selectors
mat_sig_response2   ivec2  // error selector, spare
mat_sig_ring        vec2   // inset, width (physical pixels)
```

Response selection is a uniform-int branch, so one program serves every
combination and no permutation compile is needed.

- **`ring`.** A second rounded-box SDF band inside the slab at `ring-inset`,
  `ring-width` wide, added as an emissive term in the accent. Baseline
  brightness is `0.15 + 0.35 * level`; with no accent the band is skipped.
- **`rim-orbit`.** The Fresnel glint currently uses a fixed top-left light
  direction. This response rotates `mat_sig_light.xy` around the perimeter
  at a rate that follows `level` and an excursion that follows `breath`,
  and mixes the specular toward the accent by the tint weight. At `Quiet`
  the light parks top-left and the output is bit-identical to today.
- **`ring-pulse`.** The ring's emissive term is multiplied by
  `1 + breath * level`.
- **`sweep`.** One specular band crossing the slab along the top-left to
  bottom-right diagonal, position driven by the impulse envelope, tinted by
  the impulse accent when present.
- **`flash`.** Chromatic aberration and distortion each scaled by
  `1 + 3 * envelope`, then back.
- **`ripple`.** The impulse envelope is added into the existing jelly
  `activity` input, so an event ripples the slab exactly as a resize does.

Fireflies, frost-on-idle, accent tint of attenuation color, and inactive
desaturation are not in this slice. They slot in as new response names
without touching sections 1 through 5.

## 7. Renderer ownership and data flow

| Concern | Location |
|---|---|
| `WindowSignals`, `Slot`, `Impulse`, fold, decay, focus demotion | new `src/window/signal.rs`, stored on `MappedWindow` beside `is_urgent` |
| IPC types (`Level`, `Motion`, `ImpulseKind`, `Signal`, actions, event) | `niri-ipc/src/lib.rs` |
| Action handling and `WindowSignalChanged` diffing | `src/input/mod.rs` next to the urgency actions; `src/ipc/server.rs` next to `WindowUrgencyChanged` |
| Solver, `SignalFrame`, `SignalFingerprint`, oscillator constants | new `src/render_helpers/signal.rs`; pure, no GL |
| Per-frame wiring, crossfade `Animation`, transitions term | `src/layout/tile.rs`, both `render_inner` material sites and `are_transitions_ongoing` |
| Response config, validation, resolved response | `niri-config/src/material.rs` (`Response`, `ResolvedResponse`), `niri-config/src/window_rule.rs` (`Match`, `MaterialRef`), `niri-config/src/lib.rs` (`validate_material_refs`) |
| `signal { motion }` and `animations { material-signal }` | `niri-config/src/lib.rs`, `niri-config/src/animations.rs` |
| Uniform upload and response selectors | `src/render_helpers/material.rs` (`MaterialRenderElement`) |
| Shader responses | `src/render_helpers/shaders/material.frag` |

The `Signal` type is defined once in `niri-ipc` and reused by the compositor
so the event stream and the internal fold cannot drift.

## 8. Failure behavior

- Config errors listed in section 5 are whole-config errors, consistent with
  unknown material names today.
- IPC errors are error replies; the slot map and timers are unchanged on any rejected
  request.
- A response name that a material's type does not define is rejected at
  parse time, never at render time, so the renderer has no fallback path.
- A material assigned to a window whose type does not implement signals at
  all (none exist yet) receives the frame and ignores it; the interface
  requires no material to respond.

## 9. Verification

- Unit tests, `src/tests/material.rs` and a new `src/tests/signal.rs`:
  fold precedence and accent inheritance; decay and `until_focus`
  demotion; oscillator and envelope shapes at fixed clock values; fingerprint
  pinned to a constant at rest and changing under motion; config parse and
  validation for every error in section 5, including `response=` reference
  checking and the ring-band rule.
- IPC round trip on a nested instance: set, pulse, clear through
  `niri msg`, asserting the `WindowSignalChanged` events and the `signal`
  field in `niri msg windows`, plus every rejection case.
- Nested GLES smoke on the headless verification unit, mirroring
  `2026-09-02-material-roughness-smoke.md`: a quiet window with an accent
  ring produces zero material redraws; a `Demand + Pulse` window redraws at
  the quantized rate and stops on focus; a `done` pulse ends and damage
  stops; `signal { motion "off" }` leaves only the crossfade. Tracy capture
  for per-frame cost of ring plus rim orbit against the roughness baseline.
- Physical DRM check on the daily-driver build before acceptance, because
  this is the first material effect that runs on unfocused windows for long
  stretches.

## 10. Documentation updates when implementation lands

- `docs/materials/material-config.md`: `response` block, glass response
  vocabulary, `ring-inset`/`ring-width`, `signal-source`/`signal-tag`
  matches, `response=` on material references, `signal { motion }`, and the
  `material-signal` animation.
- `docs/wiki/IPC.md` or the fork's IPC notes: the three actions, the event,
  and the `signal` field.
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
