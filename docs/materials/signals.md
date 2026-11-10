# Material signals

Type: data model, guide; document types 1. The `response` blocks and the
`signal` policy are currently unversioned. How a window signal reaches the
glass: the per-window slots and their fold, the effective signal after the
motion policy and the input gate, the `response` fields that map it to the
ring, the tint, impulses and the tilt, the motion policy and its crossfade,
when a signal costs redraws, and how a client derives a per-workspace summary.
Requests and the `Window.signal` JSON:
[IPC.md](../wiki/IPC.md#window-signals); the ring, jelly and tilt motion:
[motion.md](motion.md); the tint: [glass-optics.md](glass-optics.md#accent-tint);
`glass` parameters: [material-config.md](material-config.md); stages:
[render-pipeline.md](render-pipeline.md).

## Effective signal

A window keeps one slot per source (`WindowSignals` in
`src/window/signal.rs`), ordered by when it was written; external sources
write them through IPC. The native `niri` slot exists exactly while the
window is urgent: level `Demand`, motion `Pulse`, no accent; its rise adds a
`Ping` impulse. A focus change clears urgency and the native slot with it; a focus
gain also demotes every `--until-focus` slot once.

`fold` turns the slots into the window's signal, or none when no slot exists:

- each slot whose TTL has passed counts as its after pair, even before its
  timer fires;
- the level is the highest level of any slot;
- the motion is that of the latest-written slot at that level;
- the accent and the tag are that slot's, or else the earliest-written
  slot's that has one, each chosen independently;
- impulses live for `IMPULSE_LIFETIME` (1.5 s); a window keeps at most four.

One timer per window at its next expiry (`next_deadline`) runs `advance`.

`effective` in `src/render_helpers/signal.rs` then applies, in order: the
`signal { motion }` policy (`off` makes motion `Static`; `reduced` turns
`Flash` into `Pulse` and `Pulse` into `Breathe`), `attention "none"` (motion
`Static`), and the input gate (motion `Static` while input is idle; see
[Redraw and idle budget](#redraw-and-idle-budget)). Each impulse maps to its
`ping`, `done` or `error` response; `none` drops it, policy `off` drops all
of them, and `reduced` turns `flash` into `sweep`. Level and accent pass
through unchanged, so an idle or motion-off window keeps its static
indication.

On the glass the levels `Quiet`, `Active`, `Notice` and `Demand` are 0, 1/3,
2/3 and 1. A change of level or accent crossfades over
`animations { material-signal }`; an accent appearing or leaving fades its
presence while holding its color. Sustained motion is an oscillator on the
absolute monotonic clock: `Breathe` has a 4 s period and `Pulse` 1.2 s, both
a raised cosine stepped in 32 buckets per period and offset by a per-window
seed; `Flash` is a 500 ms square wave with 50 ms edges and one phase for
every window. An impulse rises linearly over 80 ms and then decays
exponentially (time constant 0.35 s) until its lifetime ends. A `ripple`
adds that envelope to the jelly's activity, and a `flash` raises
`chromatic-aberration` by half and `distortion` by a quarter of it (each
capped at 1); a `sweep` is drawn by the shader.

## Signal responses

Each material can map a window signal to glass effects with named `response`
blocks. A material with no response blocks gets this built-in `default`:

| Parameter | Values | Default |
| --- | --- | --- |
| `accent` | `ring`, `none` | `ring` |
| `accent-tint` | 0–1 | 0 (off) |
| `focus` | `ring-light`, `none` | `ring-light` |
| `attention` | `rim-orbit`, `ring-pulse`, `none` | `rim-orbit` |
| `ping` | `ripple`, `flash`, `sweep`, `none` | `ripple` |
| `done` | `ripple`, `flash`, `sweep`, `none` | `sweep` |
| `error` | `ripple`, `flash`, `sweep`, `none` | `flash` |
| `ring-beam-speed` | 0–5000 logical px/s | 300 |
| `ring-beam-noise` | 0–1 | 0 |
| `ring-beam-noise-hz` | 0–30 wanders per second | 3 |
| `ring-beam-decay` | 0–20000 logical px | 0 (no decay) |
| `view-tilt` | 0–20 degrees | 0 (off) |
| `view-tilt-settle` | 100–5000 ms | 900 |
| `view-distance` | 1.5–50 | 6 |
| `ring-gap` | 0–128 logical px | 8 logical px |
| `ring-glow` | 0–3 | 1.0 |
| `ring-rest` | 0–3 | 1.0 |
| `ring-accent` | 0–3 | 1.0 |
| `ring-width` | > 0, up to 128 logical px | 2.6 logical px |
| `ring-color` | `"#rrggbb"` | `#ccccff` |

When any `response` block is present, one must be named `default`. Named
responses inherit omitted fields from that block; the `default` block itself
inherits omitted fields from the built-in defaults.

```kdl
material "terminal-glass" {
    glass {
        bevel 16
    }

    response "default" {
        focus "ring-light"
        ring-beam-speed 900
        ring-gap 6
        ring-color "#ccccff"
    }

    response "loud" {
        attention "ring-pulse"
    }
}

window-rule {
    match app-id="^kitty$" signal-source="^familiar$" signal-tag="^cats/"
    material "terminal-glass" response="loud"
}
```

`response=` selects a named response on the referenced material; omitting it
selects `default`. `signal-source` and `signal-tag` are regex matches:
`signal-source` matches any source slot (including niri's native `niri`
slot), while `signal-tag` matches the window's folded signal tag. A window
with no signal matches neither.

The filament is one band shared by focus and signals. `focus "ring-light"`
lights it on the focused window, in `ring-color`: every focus gain runs one
beam of that light once round the band, and the ring then settles to a dim
resting glow that costs no redraws until the next gain. A focus change
mid-run restarts the beam on the newly focused window; focus loss ends it.
`signal { motion "reduced" }`, `motion "off"` and `animations { off }` skip
the beam, its wander and its decay, leaving the resting glow. The beam's
shape, its fades and when its run ends are in
[motion.md](motion.md#ring-beam). The fields:

- `ring-beam-speed`: the head's speed in logical px/s; a lap takes
  `perimeter / ring-beam-speed`. `0` shows the resting glow without a beam.
- `ring-glow`: scales the whole focus light (head, tail, resting glow and
  spill), so their ratios hold while the total is tuned.
- `ring-rest`: scales only the resting glow; `0` leaves no resting ring,
  above `1` it is brighter than the beam's base. The comet keeps its own
  brightness at any value.
- `ring-beam-noise`: how far the head's brightness wanders either side of
  its own level, as a fraction (`0` steady, `1.0` flares and nearly dies);
  `ring-beam-noise-hz`: wanders per second. Either at `0` leaves the head
  steady. The tail and the resting glow never wander, and the wander is
  seeded per window and per run.
- `ring-beam-decay`: the distance in logical px along the band over which
  head and tail darken to nothing; a distance, so the comet's reach is the
  same at every speed. A run ends where the comet goes dark, so a short
  decay also stops the redraws sooner. `0` is no decay.

`accent "ring"` lets a window signal light and tint the same band on any
window; with focus too, the filament shows in the accent color.
`ring-accent` scales only the accent's own light on the band: at `0` a
signal no longer outlines its window, but the accent still colors the focus
light. The edge glint that `attention "rim-orbit"` tints with the accent is
separate; `attention "none"` turns it off. The band sits `ring-gap` px
inward from the edge of the flat face, always under the face, and shows
through translucent window pixels; `ring-width` must be positive. So an
opaque window shows no ring, and a face narrower than `2 * ring-gap` on
either axis draws no ring at all. Set `focus-ring { off }` (globally or in
a window rule) for material windows so the layout's gradient ring does not
draw a second ring.

`accent-tint` tints the glass body toward the accent's hue, independently
of `accent` (`accent "none"` takes the accent off the band and leaves the
tint). It moves hue and saturation only: over a neutral backdrop the face
keeps its untinted luminance, so dark glass stays dark. The band and the
focus light pass the same tinted glass, so under `accent "ring"` they
brighten with the tint, and under `accent "none"` a near-white `ring-color`
band dims. Near-white glass has little room for hue, so the tint barely
shows on the default glass; on dark glass the recommended weight is `1`,
and the hue reads mainly through the band and the focus light. Without an
accent the glass is unchanged, and the tint follows the accent's
crossfade. The arithmetic is in
[glass-optics.md](glass-optics.md#accent-tint).

**Retired keys.** `ring-sweep-ms`, `ring-inset` and `ring-drift-hz` are
rejected by name with their replacement
(`ring-sweep-ms was replaced by ring-beam-speed`,
`ring-inset was replaced by ring-gap`); none is reinterpreted. Prism's
`prism migrate` maps `glass.ring.sweepMs` to `glass.ring.beamSpeed` (`0`
stays `0`; any other value becomes Prism's default for that key). The
filament is on by default, so a material window with no `response` block
shows the ring on focus; `focus "none"` in the `default` response leaves a
focused window unlit.

### Focus tilt

`view-tilt` is the peak angle in degrees of a brief turn in depth on focus
gain: window content and glass swing as one body, the side the focus came
from pressing in first, and settle exactly face-on after
`view-tilt-settle` ms. `view-distance` is the camera's distance in
multiples of the pane's extent along the swing: smaller is stronger
perspective, larger shows the side wall at smaller angles.

A swing starts only when the window's border and focus ring are off, its
background effect draws nothing (no xray or blur, `noise 0`, `saturation 1`;
Prism writes `blur false; noise 0; saturation 1`), it is not fullscreen or
opening, and `signal { motion "full"; }` with animations on. A pane
refocused mid-swing finishes the swing it has; a policy change cuts a
running swing to rest. How the swing moves and how the tilted slab is drawn
are in [motion.md](motion.md#rigid-tilt).

## Signal motion and animation

The top-level signal policy defaults to `full`:

```kdl
signal {
    motion "reduced" // full | reduced | off
    idle-after-ms 30000
}

animations {
    material-signal {
        duration-ms 400
        curve "ease-out-cubic"
    }
}
```

`reduced` and `off` map motion and impulses as
[Effective signal](#effective-signal) lists; `off` keeps the static accent
and the level crossfade. `material-signal` is the signal and focus
crossfade, 400 ms `ease-out-cubic` by default, following the normal
animation configuration including `animations { off }`; `reduced`, `off`
and `animations { off }` also skip the focus-gain beam.

`idle-after-ms <int>`: once no input has arrived for this long, sustained
attention motion settles to the static indication and Aurora holds its
phase ([Redraw and idle budget](#redraw-and-idle-budget)). Default 30000;
`0` disables both gates; at most 3600000, and the error is
`idle-after-ms must be at most 3600000`.

## Redraw and idle budget

A settled signal costs nothing. The rendered frame is quantized into a
`SignalFingerprint`; while it is unchanged the material is not re-rendered
and adds no damage. Static levels and accents, the resting ring and resting
jelly have no deadline.

Only sustained motion and moving optics put a window on a clock.
`Tile::tick_deadline` reports the earlier of the signal's next bucket
boundary (`next_boundary`: every 125 ms for `Breathe`, 37.5 ms for `Pulse`,
four samples on each 50 ms `Flash` edge) and the optics' next change (an
Aurora with `drift-hz` above 0), and only while the slab band, the tile
inflated by `bevel`, is in the output's view (`slab_in_view`). Tiles report
during rendering, so a hidden workspace or an output that is off reports
nothing. Each output keeps the earliest report (`SignalTicks`) and
`arm_signal_timer` arms one timer for it after each render. Finite motion,
the crossfades, live impulses, the focus beam and the tilt, runs on the
animation loop while the band is visible and stops when it ends.

The input gate is `signal { idle-after-ms }`. Input, a session resume and an
unlock count as activity; idle inhibitors and client damage do not; a
compositor that never receives input still goes idle. Once idle:

- sustained attention motion is `Static`: level and accent stay lit;
- the shared optic timeline holds (`OpticTimeline` in `src/activity.rs`), so
  an Aurora keeps its phase and requests no deadline even on an unfocused,
  signal-free window;
- impulses, crossfades, the focus beam and the tilt are not gated: they are
  finite.

On activity attention resumes on its absolute clock, in step with windows
that never stopped, and Aurora from its held phase without catching up.
Either edge redraws only when some tile renders differently
(`Layout::set_input_active`); a reload recomputes idleness from the quiet
time elapsed. `src/tests/attention_idle.rs` covers these edges.

## Workspace replay

niri sends no per-workspace signal summary. A client folds one from the
event stream: keep the reducer state (`niri_ipc::state::EventStreamState`)
and take the highest `Window.signal.level` over the windows whose
`workspace_id` names each known workspace. A workspace with no signalled
window has no level, which is not the same as `Quiet`. This holds because:

- each refresh sends workspace events before window events, and the
  compositor applies every event to its own copy of that state before
  sending it, so a client's fold equals the compositor's;
- a window whose workspace, floating state, title or app id changed is
  resent whole (`WindowOpenedOrChanged`, carrying its current signal);
  otherwise a changed signal is a `WindowSignalChanged`, including an
  impulse's start and end at an unchanged level;
- a TTL demotion arrives as `WindowSignalChanged` with the slot kept, the
  last cleared slot as `signal: null`, a close as a bare `WindowClosed`;
- a new client first receives `WorkspacesChanged`, then `WindowsChanged`.

Three gaps are expected. A window held in an interactive move has
`workspace_id: null`, and a window can briefly name a workspace removed
earlier in the same refresh: neither counts toward any workspace. Between a
snapshot's `WorkspacesChanged` and its `WindowsChanged` every workspace
folds to no level. A client more than 64 events behind is dropped and must
reconnect; the new snapshot reconciles whatever it missed.
`niri-ipc/tests/workspace_signal_replay.rs` replays these cases through the
reducer.
