# Deferred glass signal responses

Scope pass: 2026-09-29. Handoff, not an approved design. Goal: `material-0a4093`.

## Problem

Give windows richer identity and state cues through five deferred responses:
fireflies (`material-54bcac`), idle frost (`material-4bf8b8`), progress fill
(`material-5d854f`), attenuation tint (`material-6f45a0`), and inactive client
desaturation (`material-987655`). Their original outcomes remain in their task
bodies. None is ready for rendering implementation merely because the signals
foundation shipped.

## Current behaviour and evidence

- The original [signals design](../materials/2026-09-02-material-signals-design.md)
  §11 explicitly deferred these responses. Foundation `material-a54d89` landed
  at `663202b1`. Current `niri-config/src/material/mod.rs` still accepts only
  `ring`/`none` for accent and `rim-orbit`/`ring-pulse`/`none` for attention.
- `Tile::signal_for_frame` in `src/layout/tile.rs` already crossfades accent
  color and presence. `SignalFrame` in `src/render_helpers/signal.rs` carries
  both, while `material/main.frag` applies attenuation before interior light.
  This gives tint an existing animation and rendering site.
- Input inactivity already suppresses sustained attention through
  `effective(..., input_active)`. `niri-config/src/signal.rs` defaults
  `idle-after-ms` to 30000; config support landed at `c0b2e151`. Quiet signal
  level and global input inactivity are distinct: the gate preserves level,
  accent and finite impulses. Frost still has no age, growth or wipe state.
- `Slot`, `SetSlot` and `Folded` in `src/window/signal.rs`, and signal requests
  and events in `niri-ipc/src/lib.rs`, have no job-progress value. The solver's
  impulse `progress` is elapsed animation age, not task completion.
- The [render pipeline](../materials/render-pipeline.md) and
  `src/render_helpers/shaders/material/main.frag` show that opaque client
  pixels return immediately. Saturation now acts on the sampled backdrop
  before attenuation (`e33aa968`); it cannot desaturate the client texture.
  `material-cad932` therefore does not cover `material-987655`.

## Constraints

Preserve default/zero-weight output, premultiplied compositing, cached
prefilter reuse and settled redraw gating. Fireflies must obey visibility,
input-activity and motion-policy gates, with no selected-effect cost at Quiet.
Frost must stop redrawing when fully grown. Client desaturation explicitly
changes the opaque-pixel contract and needs its own reviewed coverage rules.
Progress must define validation, source arbitration, expiry and clearing
before ring geometry. The current ring is an arc-length beam, not the
original signals design's historical ring placement.

Related open work: `material-1c5a30` covers broader organic lighting;
`material-f86183` covers wider inactivity/quiescence; `material-79d1de` needs
the future progress channel for OSC reports. These remain separate ideas;
this pass found no existing open research task answering the tint contract.

## Alternatives

1. **Start with attenuation tint (current lean).** Design one opt-in response
   using existing accent/presence animation and the attenuation stage; settle
   its weight and coexistence with ring identity first.
2. Start with frost or progress. Frost needs timing and wipe decisions;
   progress needs a signal/IPC extension and arbitration before any visual
   implementation. Both carry more unresolved state than tint.
3. Design all five together. This joins independent client-pixel, continuous
   data and procedural-light changes. Defer that bundle; a shared brief
   provides coordination without requiring a shared implementation.

## Unanswered questions

- **Tint:** replacement accent selector or independent weight alongside ring?
  What color space and missing-accent behavior? The tint design task proposes
  the contract; the owner reviews its design and visual acceptance captures.
- **Frost:** elapsed Quiet time, input inactivity, or both? Which growth/wipe
  geometry and reduced-motion behavior? A later frost design must settle
  these against `material-f86183`; reusing the global idle gate alone is
  insufficient.
- **Progress:** which source wins, and how are unknown/indeterminate progress,
  expiry and completion represented? A later progress design must cover the
  store, IPC/events and ring, coordinated with `material-79d1de`.
- **Fireflies:** particle density, depth and absent-accent hue behavior, with
  what measured GPU ceiling? A bounded visual/performance spike, coordinated
  with `material-1c5a30`, can answer; no measurement was run in this pass.
- **Desaturation:** Quiet or unfocused trigger, and should opaque client
  content, popups and nonmaterial windows change? The owner must judge the
  intended look before a design changes client-pixel coverage.

## Proposed decomposition

- `material-0a4093` groups the five ideas; all remain unclaimed `idea` records.
- `material-3bdffc` designs attenuation tint only, including config, color
  mixing, ring coexistence and default/zero-weight, crossfade and settled-cost
  checks. It wakes `material-6f45a0` with a finding note in the result commit.
  Spec and implementation-plan reviews precede rendering implementation.
- The other four ideas remain briefed with the questions above. No additional
  research/design tasks are filed until their respective direction is taken;
  existing lighting, inactivity and OSC ideas retain their sources and scope.
