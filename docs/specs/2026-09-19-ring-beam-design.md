# The ring beam: a light inside the glass, tracing the face

**Status:** design, 2026-09-19; awaiting the owner's review before planning.
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
face_radii, gap) -> f64`, uses the same formula on the un-jellied face and
gives the tile its run length (§2). The two agree exactly at rest; under
jelly motion the shader's `P` differs by the resize, which changes only
when the run ends, never where the head is.

### 1.3 The comet

Uniforms: `mat_sig_focus = vec3(focus, head, env)` — `focus` as today,
`head` the head's arc position in px (unbounded during a run, see §2),
`env` the head envelope 0–1. `mat_sig_ring = vec3(gap, width, glow)`.

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
dHead = min(|head − s|, |head − s − P|)          // s = 0 and s = P coincide
headTerm = env · exp(−dHead² / (2 σ²))
behind = head − s                               // no wrap: the beam enters at s = 0
tailTerm = behind > 0 && behind < L ? BEAM_TAIL_START · exp(−behind / L) · (1 − exp(−behind / σ)) : 0
beam = BEAM_BASE · glow · (headTerm + tailTerm + BEAM_REST)
```

`head` runs from `0` to `P + L` over a run (§2); while `head > P` the tail
is what remains, leaving at the start point. The `(1 − exp(−behind/σ))`
factor lets the tail grow out of the head instead of stepping on. At rest
`env = 0`, `head` is irrelevant and `beam = BEAM_BASE · glow · BEAM_REST`.

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

`FocusSweep` becomes `FocusBeam { started: Duration, speed: f64, run: Duration }`
on the tile, on the unadjusted clock as today.

- **Start.** On every focus gain with `focus "ring-light"`, `motion "full"`,
  animations on, and `ring-beam-speed > 0`: `run = (P + L) / speed`, with
  `P = beam_perimeter(...)` from the tile's current face and `L` the tail
  length of §1.3. A running beam is replaced. A beam starts nowhere else.
- **Frame.** `head = speed · elapsed` px. `env = smoothstep(0, FADE, t) ·
  (1 − smoothstep(P/speed − FADE, P/speed, t))` with `FADE = 300 ms`: the
  head fades in as it sets off and fades out as it returns to the start
  point; the tail follows by construction and clears by `run`.
- **End.** `elapsed ≥ run` drops the beam; the ring holds the resting glow.
  The three settled gates of the motion design hold unchanged: no
  `tick_deadline` arm, a constant fingerprint, `are_transitions_ongoing`
  false. Focus loss drops the beam at once (the ring is dark without
  focus, as today).
- **Skips.** `motion "reduced"`, `motion "off"`, `animations { off }`,
  `ring-beam-speed 0`, and the fixtures' `should_complete_instantly` skip
  the beam: focus gain shows the resting glow immediately.
- **Cut.** As today: `focus` ceasing to be `ring-light`, or the motion
  policy ceasing to allow it, drops a running beam. A reload of
  `ring-beam-speed`, `ring-gap` or `ring-glow` does not cut it; speed and run
  are snapshotted, gap and glow apply on the next frame.
- **Scheduling.** The beam runs on the animation loop through
  `are_transitions_ongoing`, as the sweep did. No bucket deadline.
- **Idle gate.** Untouched. The beam is focus, not attention.

Cost, stated: one focus gain on a 6000 px terminal at 300 px/s is about
20 s on the animation loop for that tile (≈1200 material redraws at 60 Hz),
where the sweep was ~56. It is bounded per focus change; the idle-budget
goal's concern was sustained motion, which this design does not add.

Fingerprint: `head` quantized to 0.5 px, `env` to 1/256.

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
| `mat_sig_focus` vec3, `mat_sig_ring` vec3 | `src/render_helpers/material/mod.rs` | §1.3 |
| `FrameInputs { beam_head, beam_env }`, fingerprint | `src/render_helpers/signal.rs` | §2 |
| `FocusBeam`, start/end/skip/cut, uniforms | `src/layout/tile.rs` | §2 |
| Prism defs, sink, migration | prism `defs/glass.yaml`, `integrations/niri/render.js`, `manifest.yaml`, `resources/profiles/*` | §4 |

Data flow per frame: tile computes `head`, `env` from `FocusBeam` and the
clock → `FrameInputs` → `SignalFrame` → uniforms `mat_sig_focus`,
`mat_sig_ring` → shader evaluates `arcPosition`, comet, spill at each
landing point.

## 6. Verification

Compositor unit tests:

- `ring.rs`: `beam_perimeter` of a box with four equal radii equals
  `4(hx+hy) − (2 − π/2)·4r`; with zero radii `4(hx+hy)`; with a gap larger
  than a radius the radius floors at 0; a face narrower than `2·gap` yields
  `0`.
- Config: defaults and bounds of the three keys; `ring-sweep-ms` and
  `ring-inset` rejected with their replacement messages; inheritance through
  `with_overrides`.
- Tile: focus gain starts a beam with `run = (P + L)/speed`; a gain during a
  run restarts it; loss drops it; `head` and `env` follow the clock (`env`
  0 at `t = 0`, 1 mid-run, 0 at `P/speed`); after `run` the three settled
  gates hold; the skip set shows the resting glow with no beam; a speed
  reload mid-run does not change `run`; a `focus "none"` reload cuts.
- Signal: fingerprint changes with `head` by 0.5 px and not below;
  `settled_focus_fingerprints_to_a_constant` holds with `env = 0`.
- Uniforms: `mat_sig_focus` carries `(focus, head, env)`, `mat_sig_ring`
  carries `(gap, width, glow)`.

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
with `BEAM_SPILL` at 0 for comparison. The owner's judgment of head, tail,
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
