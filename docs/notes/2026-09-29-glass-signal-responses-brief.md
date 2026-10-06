# Deferred glass signal responses

Scope passes: 2026-09-29 and 2026-10-06. Handoff, not an approved design.
Goal: `material-0a4093`.

## Problem

Give windows richer identity and state cues through five deferred responses:
fireflies (`material-54bcac`), idle frost (`material-4bf8b8`), progress fill
(`material-5d854f`), attenuation tint (`material-6f45a0`), and inactive client
desaturation (`material-987655`). Their original outcomes remain in their task
bodies. Tint has since shipped; the other four responses remain unresolved.
The follow-up pass covers desaturation together with workspace summaries
(`material-6cca0a`, now shelved) in the
[signal-model brief](2026-09-29-signal-model-extensions-brief.md).

## Current behaviour and evidence

- The original [signals design](../materials/2026-09-02-material-signals-design.md)
  §11 explicitly deferred these responses. Foundation `material-a54d89` landed
  at `663202b1`. The accent selector remains `ring`/`none`; independent
  `accent-tint` is now a response weight, default 0, in
  `niri-config/src/material/mod.rs`.
- `material-6f45a0` and its design task `material-3bdffc` are complete.
  The [accepted tint design](../specs/2026-10-03-accent-tint-design.md)
  uses the existing accent/presence crossfade and computes attenuation tint
  on the CPU (`src/render_helpers/material/mod.rs`). Neutral output and
  opaque client pixels remain unchanged. This does not supply client
  desaturation.
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
  `material-cad932` therefore does not cover `material-987655`. The current
  saturation optic (`src/render_helpers/material/optics/saturation.rs` and
  `src/render_helpers/shaders/material/saturation.frag`) owns this backdrop-only hook and returns
  unchanged color at 1; it is not a window-texture postprocess.

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
`material-d257d9` already designs the shared opt-in client-content boundary
for `material-7f5751` and desaturation. Keep the desaturation idea's source,
parent and trigger decision; do not create a second content-stage design.
`material-a7e72e` concerns film-site saturation of glass, not client pixels.

## Alternatives

1. **Reuse the content-stage design (current lean for desaturation).**
   `material-d257d9` settles opt-in client coverage, sampling/composition,
   neutral behavior and readability. It coordinates desaturation's own
   trigger without assuming Quiet and unfocused are equivalent. Retain
   current client pixels until written design and plan reviews are complete.
2. Start with frost or progress. Frost needs timing and wipe decisions;
   progress needs a signal/IPC extension and arbitration before any visual
   implementation. Both carry more unresolved state than tint.
3. Design all five together. This joins independent client-pixel, continuous
   data and procedural-light changes. Defer that bundle; a shared brief
   provides coordination without requiring a shared implementation.

## Unanswered questions

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
- **Desaturation:** Quiet, unfocused, or a separate opt-in trigger? Should
  opaque text, images, popups, nonmaterial windows and capture targets change?
  `material-d257d9` frames the alternatives and neutral/alpha checks; the
  owner reviews the design and text comparisons. Input idle, signal Quiet
  and focus are distinct; no trigger or cost ceiling is accepted here.

## Proposed decomposition

- `material-0a4093` retains the five responses; tint `material-6f45a0` and
  design `material-3bdffc` are done, while the other four remain ideas.
- Reuse `material-d257d9`: P2 / m / high / planned, under the existing optics
  lane. Its body already requires written spec/plan review, client sampling
  and alpha order, readability, popup/capture coverage, default opaque-pixel
  identity and a later bounded cost check. Its design result records a
  finding on `material-987655` and `material-7f5751` in the same commit and
  updates both this brief and the
  [optics brief](2026-10-06-glass-optics-brief.md).
- `material-987655` remains briefed; no parallel desaturation design or
  implementation task is needed. Other lighting, inactivity and OSC ideas
  retain their sources and scope; no new task was filed in this pass.
