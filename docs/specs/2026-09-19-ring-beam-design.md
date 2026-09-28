# The ring beam: a light inside the glass, tracing the face

**Status:** implemented on `material-2c3984` (`5610d666`, `98013739`, `0a12a7bd`, `0219599f`, `06b71bb1`); evidence in [2026-09-19-ring-beam-evidence.md](../materials/2026-09-19-ring-beam-evidence.md); constants accepted by the owner on the review sheets, 2026-09-21, defaults unchanged. Design of 2026-09-19, revised after two design reviews and the 2026-09-20 plan review (rendered-face geometry, timeout only before the first rendered frame, tests on returned dynamics).
Supersedes §1 (the pattern and the ease) of
[2026-09-18-ring-focus-motion-design.md](2026-09-18-ring-focus-motion-design.md)
and the `ring-inset` placement question of `material-9306b5`; keeps that
design's settled-focus gates (§2), attention gate (§3) and Prism migration
mechanism (§5). **Task:** `material-2c3984`, under `material-a76720`.

## Context

The bounded-motion design landed on `materials-26.04` (`597aba66`) and was
installed on 2026-09-19 together with the within ring
(`d5b4188a`), which the owner had not seen live before. The review of the
live result:

- The band reads as a flat metallic bevel, not glass with a light in it.
- The focus "pulse" is far too fast. The travelling brightness is
  `sin(2θ+φ)·sin(3θ−2φ)`: a five-lobe interference pattern with a
  three-cycles-per-lap component at every point, so a 1.5 s lap flickers
  each spot at ~2 Hz mean and ~6 Hz at the eased start. It is not one
  light moving.
- The terminal face floats above the ring. The band sits `ring-inset 5`
  from the slab's outer edge — on the 17 px chamfer, where the backdrop is
  refracted differently than under the flat face — so the face reads as a
  plate lying on a lit frame. The owner's terminals are fully transparent;
  the "body" is the glass face itself.

The intent, in the owner's words: something like a beam of coloured light
moving through a block of ice or glass — inside it, refracted and shaped by
the material — tracing the border of the window with a gap between the beam
and the edge of the glass, at a walking pace, then quiet.

## Decisions

Made in the brainstorm of 2026-09-19, with the alternatives at the end.

- **Beam on focus gain, then a quiet ring.** One beam runs the perimeter and
  the ring settles to a dim, even glow at zero redraws. Settled focus still
  costs nothing; the focused window still shows at rest.
- **Speed, not time.** `ring-beam-speed` in px/s replaces `ring-sweep-ms`; a
  lap takes `perimeter / speed`. A beam has a physical pace; a small pane
  must not crawl and a large one must not race.
- **A comet with a long tail.** A bright head of a few tens of pixels and a
  tail diffusing over a quarter of the perimeter: the head says beam, the
  tail says ice.
- **The gap is measured from the face.** `ring-gap` replaces `ring-inset`:
  distance inward from where the flat face begins. The beam is always under
  the face; the bevel may shrink to make room.
- **Arc-length parametrization.** Each fragment knows its distance along the
  beam line's perimeter, so the pace is constant in pixels around corners
  and sides alike.
- **One intensity knob.** `ring-glow` scales the whole focus light so the
  ratios judged on the sheets hold while the total is tuned; Prism exposes
  it beside speed and gap.
- **Focus gain always starts a beam; focus loss ends it.** The sweep's
  "only from rest" rule would blank the ring for 20 s after any focus
  change at this pace.
- **Under the face means through the window.** The beam shows through
  translucent window pixels and is hidden by opaque ones; there is no
  fallback band on the chamfer. One ring, one place.

## 1. Geometry and the shader

### 1.1 The beam line

`slabSurface` (prelude.frag) already derives the inner face as a rounded box:
`inner_half`, `inner_center`, per-corner radii `inner_r` (top-left,
top-right, bottom-right, bottom-left), jelly-moved and jelly-scaled. It
stores only the outer box in globals today; it will also store the inner one
(`g_face_half`, `g_face_center`, `g_face_r`).

The **beam line** is the face box shrunk by `ring-gap` on every side:
`half = g_face_half − gap`, `r_i = max(g_face_r_i − gap, 0)`. A face smaller
than `2 · gap` on either axis has no beam line and draws no beam (the
resting glow too); this is stated, not patched.

### 1.2 Arc position

`arcPosition(vec2 q, out float P)` returns the distance `s ∈ [0, P)` along
the beam line of the point of the line nearest `q`, and the perimeter

```
P = 4 · (half.x + half.y) − (2 − π/2) · Σ r_i
```

The origin is where the top edge meets the top-left arc; `s` increases
clockwise: top edge, top-right arc, right edge, bottom-right arc, bottom
edge, bottom-left arc, left edge, top-left arc. Straight runs measure `s` by
the coordinate along the side; arcs by the angle around the corner centre
times the radius. The function is a projection, so it is valid for any `q`,
including chamfer points (used by the spill in §1.4).

Its Rust twin, `render_helpers::material::ring::beam_perimeter(face_half,
face_radii, gap) -> f64`, uses the same formula and decides, every frame,
whether the run has ended (§2). It is given the **same effective face** the
shader draws: the tile already computes the jelly resize it sends as
`mat_jelly_resize`, and the face half-extents handed to `beam_perimeter`
are scaled by it exactly as `slabSurface` scales `inner_half`
(`half · (1 + resize / max(2 · half_ext, 1))`), with the radii refit the
same way. The slab size and chamfer come from the rendered `MaterialFrame`,
and the corner radii are the fitted values passed to the element; neither
is reconstructed from the target window size. Timing and rendering therefore
see one perimeter at every frame,
jelly included; there is no approximation to bound. A unit test feeds the
Rust twin a jelly-scaled face and checks it against the closed form on the
scaled extents.

### 1.3 The comet

Uniforms: `mat_sig_focus = vec4(focus, head, env, decay)` — `focus` as
today, `head` the head's arc position in px (unbounded during a run, see
§2), `env` the head cutoff 0–1 (exactly `0` once the head has completed its
lap), `decay` the shared brightness of head and tail 0–1 (`1` throughout a
plateau run). `mat_sig_ring = vec3(gap, width, glow)`. Both are registered
as `_2f` in `src/render_helpers/shaders/mod.rs` today and become `_4f` and
`_3f`.

Constants, named at the top of the ring block so the sheets can move them
in one line:

| Constant | Value | Meaning |
| --- | --- | --- |
| `BEAM_HEAD_SIGMA` | 20 px | Gaussian half-width of the head |
| `BEAM_TAIL_START` | 0.6 | Tail brightness just behind the head |
| `BEAM_TAIL_FRACTION` / `BEAM_TAIL_MAX` | 0.25 / 1200 px | Tail length `L = min(P · fraction, max)` |
| `BEAM_REST` | 0.2 | Resting glow |
| `BEAM_SPILL` | 0.25 | Edge spill onto the chamfer (§1.4) |
| `BEAM_BASE` | 0.7 | Focus light scale (today's `0.7`) |

For a landing point with arc position `s`:

```
dHead = |mod(head − s + P/2, P) − P/2|            // circular distance: s = 0 and s = P coincide
headTerm = env · exp(−dHead² / (2 σ²))
behind = head − s                                 // no wrap: the beam enters at s = 0
tailTerm = (behind > 0 && behind < L)
    ? BEAM_TAIL_START · exp(−behind / L) · (1 − exp(−behind / σ)) · (1 − behind / L)²
    : 0
beam = BEAM_BASE · glow · (decay · (headTerm + tailTerm) + BEAM_REST)
```

The whole tail expression lives inside the conditional. Evaluated outside
it, `exp(−behind/σ)` at rest (`head = 0`, `s = 2000`, `σ = 20`) is
`exp(100)`, which overflows float32 to infinity, and infinity times a zero
taper is NaN, not zero. Inside the branch `behind > 0`, so every factor is
finite and at most `1`.

The circular distance is symmetric across the seam: at `head = 20`, `σ = 20`
the head lights `s = 0` and `s = P − 20` alike (`0.607` and `0.135`), and on
its return at `head = P − 20` it lights `s = 20` the same way. The `(1 − behind/L)²` taper takes
the tail to exactly `0` at `behind = L` with zero slope, so the tail
diffuses into the resting glow instead of ending in a moving step (without
it the cut would sit at `0.6/e ≈ 0.22`, brighter than the rest). The
`(1 − exp(−behind/σ))` factor lets the tail grow out of the head instead of
stepping on. `head` runs from `0` to `P + L` over a run (§2); while `head >
P` the tail is what remains, leaving at the start point.

**Rest is `head = 0, env = 0, decay = 0`.** Those are the uniform values
the tile sends whenever no beam is running, and they define the resting
state exactly: `env = 0` removes the head, `head = 0` makes `behind = −s ≤
0` for every fragment so the tail branch is not taken, and `decay = 0`
would remove both anyway; `beam = BEAM_BASE · glow · BEAM_REST`, finite in
float32 at every `s`. The tail-drain phase (`head ∈ (P, P + L]`, `env = 0`,
`decay > 0`) is *not* rest: `head` still moves and the fingerprint tracks
it (§2).

`focusGlow` becomes `beam`; `travel`, `drift` and the `sin·sin` pattern go.
The accent glow (`accentGlow`, familiar signals, `mat_sig_level`,
`ring-pulse`) is unchanged and rides the same band at its new position; the
fast rhythm stays the attention vocabulary.

### 1.4 Where the light lands, and the spill

The light path is unchanged: `lightShift` through `light-ior` at 20 % of
thickness, capped at half the gap, split per channel by aberration;
`filamentBand`'s Gaussian core and scattered halo, widened by roughness;
`att^0.2` tinting with the attenuation colour. `filamentBand` measures its
distance from the **face edge** inward (`d = −sdRoundedBox(q − g_face_center,
g_face_half, g_face_r) − gap`) instead of from the outer edge. Under the
flat face the structural normal is straight, so the shift there comes only
from `distortion` noise and jelly ripple: the beam bends where the glass is
uneven.

**Edge spill.** Light inside ice leaks at its edge. On chamfer fragments
(`slabChamfer > 0`, outside the face): take `s = arcPosition(q)`, evaluate
`beam` there without `BEAM_REST`, and add
`color · BEAM_SPILL · beam · (1 − t) · att^0.2` where `t` is the fragment's
position across the chamfer from the face edge (0) to the outer edge (1).
The frame is lit by the beam, which is the one term in this design aimed at
the face floating above the frame. The sheets decide whether it stays and at
what constant.

Compositing is unchanged: `color = win + (1 − win.a) · glassed`, with the
early return on opaque window pixels.

## 2. Motion policy

`FocusSweep` becomes `FocusBeam { started, speed, rendered, done }` on the
tile, on the unadjusted clock as today. Only the start instant and the
speed are snapshotted. The two flags start false: the first geometry
evaluation sets `rendered`, and the frame that sees the tail clear sets
`done`. Geometry and envelope values are derived per frame.

- **Start.** On every focus gain with `focus "ring-light"`, `motion "full"`,
  animations on, and `ring-beam-speed > 0`. A running beam is replaced. A
  beam starts nowhere else.
- **Frame.** With `P = beam_perimeter(...)` from the tile's effective face
  *as it is this frame* (jelly included, §1.2) and `L = min(P ·
  BEAM_TAIL_FRACTION, BEAM_TAIL_MAX)`: `head = speed · elapsed` px. Two
  envelope factors, both from the clock:
  - `env`, the **head amplitude**: `smoothstep(0, FADE, t) · (1 −
    smoothstep(P/speed − FADE, P/speed, t)) · gain(t)`, with `FADE = 300 ms`
    capped at half the lap (so on a lap shorter than both fades the head
    fades out only after it has faded in), and exactly `0` for
    `t ≥ P/speed`. The head fades in as it sets off and
    out as it returns to the start point, and can never begin a second lap
    during the drain, whatever the other factor does. `gain` is the head's
    brightness wander, `1` unless asked for (addendum below).
  - `decay`, the **shared brightness** of head and tail: `1` throughout a
    plateau run (the default), times the distance decay when one is asked
    for (addendum below). The tail follows the head by construction.
- **Geometry changes.** Because `P` and `L` are read every frame, a window
  that grows mid-run keeps the beam going until the head has covered the
  *new* perimeter and the tail has cleared it; one that shrinks ends the run
  as soon as the tail has cleared the smaller perimeter, with no redraws
  spent on a lap that no longer exists. A `ring-gap` reload changes `P` the
  same way. The head is never repositioned: it stays at `speed · elapsed`,
  so the seam moves under it by the size change and the beam keeps its pace.
  One consequence: on a grow during the drain the head, still counting, is
  back inside the longer lap, so `env` returns to 1 and the head visibly
  reappears until it completes the new perimeter.
  Jelly motion is the same case: both sides read the jelly-scaled face
  (§1.2), so the spring never fades the head early or ends the run before
  the shader's tail has cleared.
- **End.** `head ≥ P + L` — or `head ≥ D` under a distance decay `D`
  (addendum below) — evaluated against this frame's geometry, marks the
  beam done and sends the rest uniforms `head = 0, env = 0, decay = 0` (§1.3);
  `advance_animations` then removes it without consulting an old perimeter.
  The ring holds the resting glow. The three settled gates of the motion design hold
  unchanged: no `tick_deadline` arm, a constant fingerprint,
  `are_animations_ongoing` false. Focus loss drops the beam at once (the
  ring is dark without focus, as today).
- **Never-rendered timeout.** `BEAM_MAX_RUN = 120 s` expires a beam only
  while `rendered` is false. Its first geometry evaluation permanently
  disables this backstop for that run. A rendered beam at `P = 6000` and
  `speed = 50` lasts 144 s, including the entire 24 s tail after 120 s.
  Hiding a previously rendered tile does not reset the flag or arm a new
  timeout: visibility suppresses redraws, and its next rendered frame
  decides completion from elapsed time and current geometry.
- **Envelope shape under review.** The plateau above is the default. A
  second shape, *splash*, is carried as a named alternative for the sheets:
  head and tail start at full brightness and decay together over the run,
  like a ripple leaving the centre of a splash. It changes only the shared
  factor: `decay = (1 − t / run)²` with `run = (P + L) / speed`, and the
  head's fade-in shortens to `100 ms` so the launch is at its brightest;
  the head cutoff `env` is unchanged, so the head still ends its lap at the
  seam and the tail still drains behind it. One constant selects the shape;
  the sheets decide which ships. Both end at rest the same way.
- **Head wander (addendum, 2026-09-22, material-9704b0).** `gain(t)` gives
  the head's brightness a life of its own, so the comet reads as a living
  light rather than a lamp on a track. Two response keys drive it:
  `ring-beam-noise`, the fraction either side of 1 the gain reaches, and
  `ring-beam-noise-hz`, its rate. `gain = 1 + intensity · (2 · w − 1)`,
  floored at 0, where `w ∈ [0, 1]` is one octave of value noise — hashed
  lattice points one unit apart on `t · hz`, smoothstep between them — so
  the head wanders rather than flickers. Either key at `0` gives `gain = 1`
  and a bit-identical beam. The seed mixes the pane's jelly seed with the
  run's start instant, so two panes focused together do not flicker in step
  and one pane's successive runs do not replay.

  It rides the **head alone**, by multiplying `env`: the tail is a trail the
  head has already left, and re-wandering it would make the whole comet
  breathe rather than its light flicker. Because it multiplies `env` it is
  carried by the existing `mat_sig_focus.z` — no new uniform, and the
  shader formula is unchanged. Two properties are preserved by
  construction: `env` is still exactly `0` from the lap's end on, so the
  drain and the rest uniforms are untouched; and the wander exists only
  while a beam runs, so the settled ring keeps its constant fingerprint and
  its zero redraws. The skips below skip it with the beam.

- **Distance decay (addendum, 2026-09-28, material-338d21).** The response
  key `ring-beam-decay` is a distance `D` in px along the beam line: the
  shared `decay` falls as `(1 − head / D)²`, clamped at `0`, so head and
  tail darken together and are exactly dark once the head has travelled
  `D`. The slope is zero there, so the light eases out. A distance rather
  than a half-life makes the comet's reach a property of the beam, the same
  on every window and at every speed. `D = 0` (the default) is no decay and
  a bit-identical beam; it composes with the splash shape by multiplication.
  The run ends at `min(P + L, D)`: when the comet goes dark before the lap
  and its tail are done, that frame ends the run and the redraws stop there
  instead of drawing nothing until `P + L`. `decay` rides the existing
  `mat_sig_focus.w`, so the shader is unchanged, and the resting glow sits
  outside it, so the settled ring keeps its constant fingerprint. A decay
  inside the lap also makes the seam fade moot: the comet is already dark
  when the head reaches the seam.

- **Skips.** `motion "reduced"`, `motion "off"`, `animations { off }`,
  `ring-beam-speed 0`, and the fixtures' `should_complete_instantly` skip
  the beam: focus gain shows the resting glow immediately.
- **Cut.** As today: `focus` ceasing to be `ring-light`, or the motion
  policy ceasing to allow it, drops a running beam. A reload of
  `ring-beam-speed`, `ring-gap`, `ring-glow` or `ring-beam-decay` does not
  cut it; the speed is snapshotted, gap (through `P`), glow and decay apply
  on the next frame (a decay reloaded shorter than the head's distance ends
  the run on that frame).
- **Scheduling.** The beam runs on the animation loop through
  `are_animations_ongoing`; it is not a layout transition, so
  `are_transitions_ongoing` — which also gates pointer-focus refresh in
  `Niri::refresh_pointer_contents` — does not report it. No bucket deadline.
- **Idle gate.** Untouched. The beam is focus, not attention.

Cost, stated: one focus gain on a 6000 px terminal at 300 px/s is
`(6000 + 1200) / 300 = 24 s` on the animation loop for that tile, tail
included (≈1440 material redraws at 60 Hz), where the sweep was ~56. It is
bounded per focus change; the idle-budget goal's concern was sustained
motion, which this design does not add. The speed is the time knob:
`ring-beam-speed 600` halves it, and the Prism slider is there so the
default is chosen by eye, not by argument — a faster, subtler pass may
suit the circuit better than the slow breathing the old pattern wanted.

Fingerprint: `head` quantized to 0.5 px, `env` and `decay` to 1/256.
During the tail drain `env = 0` and `head` still advances, so the
fingerprint keeps changing and the tile keeps redrawing until the run ends;
at rest `head = 0, env = 0, decay = 0` and the fingerprint is constant.

## 3. Native configuration contract

In `response`:

| Key | Type | Default | Range | Replaces |
| --- | --- | --- | --- | --- |
| `ring-beam-speed` | float px/s | 300 | 0–5000 | `ring-sweep-ms` |
| `ring-gap` | float px | 8 | 0–128 | `ring-inset` |
| `ring-glow` | float | 1.0 | 0–3 | new |
| `ring-width` | float px | 2.6 | > 0, ≤ 128 | unchanged |
| `ring-color` | color | unchanged | | |

A `ring-sweep-ms` line is rejected with `ring-sweep-ms was replaced by
ring-beam-speed; see material-config.md`; a `ring-inset` line with
`ring-inset was replaced by ring-gap; see material-config.md`. Neither is
reinterpreted. `ResolvedResponse` gains `ring_beam_speed`, `ring_gap`,
`ring_glow` and loses `ring_sweep`, `ring_inset`; `with_overrides` maps
them.

The bevel constraint of material-config.md (`bevel >= 2·max(|offset|) +
ring-inset + ring-width` for an opaque window) is retired with the outer-edge
inset: under this design an opaque window shows no ring, and the doc says
so. A face narrower than `2 · ring-gap` draws no beam.

## 4. Prism contract and rollout

Three Ring definitions land in one Prism task, orders continuing the group
after `glass.ring.color` (520):

| Key | Type | Range | Default / neutral | Order | Emits |
| --- | --- | --- | --- | --- | --- |
| `glass.ring.beamSpeed` | int | 0–5000, step 50 | 300 | 530 | `ring-beam-speed` |
| `glass.ring.gap` | int | 0–128, step 1 | 8 | 540 | `ring-gap` |
| `glass.ring.glow` | float | 0–3, step 0.1 | 1 | 550 | `ring-glow` |

`glass.ring.beamSpeed` declares `replaces: glass.ring.sweepMs`; `prism
migrate` converts through the existing rule (`0 → 0`, any positive → the new
default `300`, an existing `beamSpeed` kept and reported). `gap` and `glow`
are new: `ring-inset` was never a Prism control (`prism-d8ee06` planned it
and is subsumed here). `render.js::responseBlock` emits the three keys in
every response block and never `ring-sweep-ms`; the manifest binds them at
`liveness: reload`; the starter profiles and every suite follow. `doctor`
names the pending migration. The panel needs nothing: its sliders come from
`describe --json`.

**Rollout** is the same pair-move as the sweep's, for the same reason (each
build rejects the other's keys): install the new niri; merge Prism to
`main`, `prism migrate`, `prism apply` (the running compositor rejects the
reload and holds its config until the restart); restart the session.
Rollback: the migration backup over the config dir, the previous niri
package, the previous Prism, apply, restart.

## 5. Ownership and data flow

| Piece | Owner | Change |
| --- | --- | --- |
| Keys, defaults, rejections | `niri-config/src/material/mod.rs` | §3 |
| Face globals, `arcPosition`, `filamentBand` from the face edge | `src/render_helpers/shaders/material/prelude.frag` | §1.1, §1.2, §1.4 |
| Comet, spill, `focusGlow` | `src/render_helpers/shaders/material/main.frag` ring block | §1.3, §1.4 |
| `beam_perimeter`, comet constants shared with the shader | `src/render_helpers/material/ring.rs` (new) | §1.2 |
| `mat_sig_focus` vec4, `mat_sig_ring` vec3 | `src/render_helpers/material/mod.rs` (values), `src/render_helpers/shaders/mod.rs:72-73` (registration, `_2f` → `_4f` / `_3f`) | §1.3 |
| `FrameInputs { beam_head, beam_env, beam_decay }`, fingerprint | `src/render_helpers/signal.rs` | §2 |
| `FocusBeam`, start/end/skip/cut, uniforms | `src/layout/tile.rs` | §2 |
| Prism defs, sink, migration | prism `defs/glass.yaml`, `integrations/niri/render.js`, `manifest.yaml`, `resources/profiles/*` | §4 |

Data flow per frame: tile computes `P` (effective face, jelly included),
`head`, `env`, `decay` from `FocusBeam` and the clock → `FrameInputs` →
`SignalFrame` → uniforms `mat_sig_focus`, `mat_sig_ring` → shader evaluates
`arcPosition`, comet, spill at each landing point.

## 6. Verification

Compositor unit tests:

- `ring.rs`: `beam_perimeter` of a box with four equal radii equals
  `4(hx+hy) − (2 − π/2)·4r`; with zero radii `4(hx+hy)`; with a gap larger
  than a radius the radius floors at 0; a face narrower than `2·gap` yields
  `0`; a jelly-scaled face (resize `+8` on a `200 × 100` face) equals the
  closed form on the scaled extents, i.e. what the shader's `slabSurface`
  produces for the same `mat_jelly_resize`.
- Config: defaults and bounds of the three keys; `ring-sweep-ms` and
  `ring-inset` rejected with their replacement messages; inheritance through
  `with_overrides`.
- Tile: focus gain starts a beam; a gain during a run restarts it; loss
  drops it and sends `head = 0, env = 0, decay = 0`; `head`, `env` and
  `decay` follow the clock (`env` 0 at `t = 0`, 1 mid-run, exactly 0 for
  every `t ≥ P/speed` including the whole drain; `decay` 1 throughout a
  plateau run and `(1 − t/run)²` under splash, both still `> 0` during the
  drain); the run ends at `head ≥ P + L` and the three settled gates then
  hold; the skip set shows the
  resting glow with no beam; a speed reload mid-run keeps the old speed; a
  `focus "none"` reload cuts.
- Tile, timeout: a never-rendered beam expires at 120 s; a rendered
  `P = 6000`, `speed = 50` plateau beam remains active at 121 s and ends
  on the geometry evaluation at 144 s. Hiding it after its first frame
  does not re-arm the timeout. Observe the returned `material_dynamics`
  uniforms and fingerprint: the tail advances after 120 s, then rest is
  constant. Cached `FrameInputs.beam` stays `REST` and is not the output
  under test.
- Tile, geometry changes: a resize from `P = 4000` to `6000` during the
  head phase keeps the beam running until `head ≥ 6000 + L₆₀₀₀`; the same
  resize during the tail phase likewise; a shrink from `6000` to `4000`
  during the head phase ends the run at `4000 + L₄₀₀₀` (earlier than the
  old geometry would have); a `ring-gap` reload mid-run changes the end the
  same way; `head` is `speed · elapsed` throughout, never repositioned.
- Comet numerics (Rust twin of the shader formulas, so the shader's
  constants are tested where they are defined): the circular distance at
  `head = 20`, `σ = 20`, `P = 4000` gives `0.607` at `s = 0` and `0.135` at
  `s = P − 20` (launch) and, at `head = P − 20`, `0.135` at `s = 20`
  (return); the tail is `0` at `behind = L` and its slope there is `0`; the
  tail is `0` at `behind = 0`; rest (`head = 0, env = 0, decay = 0`) yields
  exactly `BEAM_BASE · glow · BEAM_REST` at every `s`, **evaluated in
  `f32`** at `s = 2000, σ = 20` and at `s = P − 1` — finite, never NaN; under
  splash, a head `> P` with `decay > 0` contributes no head term anywhere
  (`env = 0` gates it), only its tail.
- Signal: fingerprint changes with `head` by 0.5 px and not below, and keeps
  changing through the tail drain (`env = 0`, `head` advancing);
  `settled_focus_fingerprints_to_a_constant` holds at rest (`head = 0,
  env = 0, decay = 0`) — the settled tests and the drain tests are distinct
  cases.
- Uniforms: `mat_sig_focus` carries `(focus, head, env, decay)`,
  `mat_sig_ring` carries `(gap, width, glow)`.

Signals smoke (`material-signals-smoke.sh cases`): `focused-settled` stays
`0`; the sweep cases become `beam-run` — on a known 1280×720 pane at
`ring-beam-speed 1200` (`P ≈ 3800`, run ≈ 4 s) the redraws during the run
approximate `run · frame rate` and the tail window after it is `0`;
`beam-off` (`ring-beam-speed 0`) is `0` throughout; the reduced / anim-off /
focus-none cases keep their zeros.

Sheets, the gate that matters (`ring-motion-clips.sh`, capture protocol):
a `beam` sequence on a 1280×720 pane with `bevel 10; ring-gap 8` — one
focus gain sampled at 1 fps for the run plus 4× crops of a corner at rest,
mid-pass, and as the tail clears; a second pass at `ring-gap 16`; a third
with `BEAM_SPILL` at 0; a fourth with the *splash* envelope (§2); and one
at `ring-beam-speed 900` for pace, all for comparison. The owner's judgment of head, tail,
glow, spill and gap is recorded in the evidence doc and becomes the
shipped defaults; the tunables (`ring-beam-speed`, `ring-gap`, `ring-glow`)
cover the rest.

Prism: the render tests carry the three keys in both materials, default and
override, and no `ring-sweep-ms`; the migration tests of the sweep spec
re-pointed at `beamSpeed`, including the collision case; the doctor hint.

## 7. Documentation updates when implementation lands

- `docs/materials/material-config.md`: the ring paragraph (beam, speed,
  gap from the face, glow, resting glow, spill; opaque windows show no ring;
  faces narrower than `2·gap` show no beam); the key table; the upgrade
  note naming both rejections and the Prism migration.
- `docs/materials/render-pipeline.md`: stage 5 row (arc-length beam, face
  globals, spill on the chamfer), the key map.
- `docs/materials/2026-09-02-material-signals-design.md`: the focus
  response paragraph.
- `docs/specs/2026-09-18-ring-focus-motion-design.md`: status line noting
  §1 superseded by this design.
- `material-9306b5`: closed as subsumed when the sheets are accepted.

## 8. Decomposition

1. **Config** — `niri-config` keys, rejections, resolved fields, tests.
2. **Beam** — face globals, `arcPosition`, `ring.rs`, comet and spill,
   `FocusBeam` and the tile rules, uniforms, unit tests; smoke fixtures.
3. **Prism** — definitions, sink, migration, tests (in the Prism repo, one
   plan task; depends on 1).
4. **Evidence** — smoke `cases`, the beam sheets on the headless lane, the
   owner's acceptance of the constants, docs of §7, package pin.

## Alternatives rejected

- **Beam circling while focused.** Most literal, but sustained animation on
  the focused window for the whole session — what the idle-budget goal and
  the bounded-motion design set out to remove.
- **Beam then dark.** Cheapest and most dramatic; focus would be visible
  only in the moment it changes.
- **Time per lap (`ring-sweep-ms`).** A fixed lap time makes the beam race
  on large panes and crawl on small ones; the owner's 9–18 s on a 6000 px
  terminal is 55 px/s on a 1000 px floater.
- **Angular parametrization with one lobe.** The smallest change, but pixel
  speed varies with the aspect ratio — ~2× on a 1700×1300 pane, worse on
  wide ones.
- **A separate beam element.** Duplicates the slab geometry and the
  refraction path the band already has.
- **Keeping the outer-edge inset with a larger default.** Couples the ring's
  place to `paneLip` by hand; any thicker lip pushes the beam back onto the
  chamfer.
- **A fallback band on the chamfer for opaque windows.** Two rings in two
  places; the owner's material windows are translucent, and the doc states
  the limit instead.

## Non-goals

- Exposing the comet constants (head width, tail length, rest, spill) as
  configuration; they are named constants until the sheets ask otherwise.
- Any change to attention motion, impulses, the idle gate, or the aurora.
- A general animation framework or a change to the animation loop.
- Continuous focus motion of any kind.
