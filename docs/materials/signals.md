# Material signals

Type: data model, guide; document types 1. The `response` blocks and the
`signal` policy are currently unversioned. How a window signal reaches the
glass: the per-window slots and their fold, the effective signal after the
motion policy and the input gate, the `response` fields that map it to the
ring, the tint, impulses and the tilt, the motion policy and its crossfade,
when a signal costs redraws, and how a client derives a per-workspace summary.
The requests that write signals and the `Window.signal` JSON are in
[IPC.md](../wiki/IPC.md#window-signals); how the ring, jelly and tilt move in
[motion.md](motion.md); the tint arithmetic in
[glass-optics.md](glass-optics.md#accent-tint); the `glass` parameters in
[material-config.md](material-config.md); the stage each input reaches in
[render-pipeline.md](render-pipeline.md).

## Effective signal

A window keeps one slot per source (`WindowSignals` in
`src/window/signal.rs`), ordered by when it was written. External sources
write through IPC. The native `niri` slot exists exactly while the window is
urgent: level `Demand`, motion `Pulse`, no accent, and its rise adds a `Ping`
impulse. A focus change clears urgency and the native slot with it; a focus
gain also demotes every `--until-focus` slot once.

`fold` turns the slots into the window's signal, or none when no slot exists:

- each slot whose TTL has passed counts as its after pair, even before its
  timer fires;
- the level is the highest level of any slot;
- the motion is that of the latest-written slot at that level;
- the accent and the tag are that slot's, or else the earliest-written
  slot's that has one, each chosen independently;
- impulses live for `IMPULSE_LIFETIME` (1.5 s); a window keeps at most four.

Each window has one timer at its next slot expiry or impulse end
(`next_deadline`); when it fires, `advance` demotes and drops what is due.

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
        accent "ring"
        accent-tint 0
        focus "ring-light"
        attention "rim-orbit"
        ping "ripple"
        done "sweep"
        error "flash"
        ring-beam-speed 300
        ring-beam-noise 0
        ring-beam-noise-hz 3
        ring-beam-decay 0
        ring-gap 8
        ring-glow 1.0
        ring-rest 1.0
        ring-accent 1.0
        ring-width 2.6
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
lights it on the focused window, in `ring-color`. On every focus gain one
beam of that light runs the perimeter of the band once, clockwise from the
top-left corner, at `ring-beam-speed` logical px/s: a bright head a few tens
of pixels wide with a tail diffusing over a quarter of the perimeter (at most
1200 px), fading in as it sets off and out as it returns to its start, the
tail draining behind it. A lap takes `perimeter / ring-beam-speed`; at the
default 300 px/s a full 1280×720 pane (perimeter about 3800 px) runs about
16 s, tail included, and `ring-beam-speed 900` runs it three times as
fast. The ring then settles to a dim, even resting glow and costs no redraws until the next focus gain (no
deadline, a constant fingerprint). `ring-beam-speed 0` shows the resting glow
without a beam; `signal { motion "reduced" }`, `motion "off"`, and
`animations { off }` skip the beam the same way. A focus change mid-run
restarts the beam on the newly focused window; focus loss ends it at once.
The beam's shape and when its run ends are in
[motion.md](motion.md#ring-beam).
`ring-glow` scales the whole focus light — head, tail, resting glow and
spill together — so their ratios hold while the total is tuned.
`ring-rest` scales only the resting glow, the ring that stays on a focused
window once the beam has passed: `0` leaves no resting ring while the beam
still runs on every focus gain, and above `1` the resting ring is brighter
than the beam's base. The comet rides on top of the resting level, so it
keeps its own brightness at any `ring-rest`.

`ring-beam-noise` makes the head's brightness wander as it travels, so the
comet reads as a living light rather than a lamp on a track. It is the
fraction either side of the head's own brightness that the wander reaches:
`0` is a steady head (the default), `0.3` a gentle breathing, `1.0` a head
that flares and nearly dies.
`ring-beam-noise-hz` is how fast it wanders, in wanders per second; either
value at `0` leaves the head steady.

The wander rides the head alone. The tail is a trail the head has already
left and stays smooth, and the resting glow is untouched — the quiet ring
still costs no redraws and keeps its constant fingerprint, so a wandering
head never puts a settled window back on a clock. It is seeded per window
and per run, so two panes that gain focus together do not flicker in step
and one pane's successive runs do not replay. The motion policies that skip
the beam skip the wander with it.

`ring-beam-decay` lets the comet die out on its way round: it is the
distance, in logical px along the band, over which head and tail darken
together until they are gone, falling as `(1 − distance / decay)²` so the
light eases out rather than cutting off. `ring-beam-decay 1500` at the
default speed is dark after 5 s, well before a full pane's lap closes. It is
a distance, not a time, so the comet's reach is the same on every window and
at every `ring-beam-speed`. The run ends where the comet goes dark, so a
short decay also stops the redraws sooner; the resting glow is unaffected.
`0`, the default, is no decay: the comet keeps its brightness for the whole
lap. A decay longer than the lap and its tail
still dims the comet along the way but never shortens the run. The motion
policies that skip the beam skip the decay with it.

`accepted_ring_look` in `src/tests/ring_look.rs` holds the owner's accepted
terminal glass and ring (`ring-beam-decay 4150`, speed 4350,
`ring-gap 6`, `ring-glow 1.2`). It renders that look through a real focus
gain, on a pane longer than the decay and on one shorter than it, from the
inactive material through the comet to rest. The renders are frozen and
byte-identical from run to run. Set `RING_LOOK_DUMP=<dir>` before and after
a change to ring or edge rendering, and compare the PNGs. The backdrop is
flat gray, so refraction of the backdrop does not show there.

`accent "ring"` lets a window signal light and tint the same band on any
window. Both together show the filament in the accent color. `ring-accent`
scales only the accent's own light on the band: at `0` a signal no longer
outlines its window, but the accent still colors the focus light, so the
comet and the resting ring run in the window's hue. The edge glint that
`attention "rim-orbit"` tints with the accent is separate; `attention "none"`
turns it off. The band sits
`ring-gap` px inward from the edge of the flat face (where the chamfer ends),
so it is always under the face and the bevel may shrink to make room; the
light is refracted through the glass at its remaining interior depth and
shows through translucent window pixels. On the chamfer the beam spills a
little light outward from the face edge, fading to the outer edge, so the
frame reads as lit by the beam. `ring-width` must be positive.

`accent-tint` tints the glass body toward the signal accent's hue,
independently of `accent`: `accent "none"` takes the accent off the band and
leaves the tint. The weight moves hue and saturation only. The flat face
transmits the same luminance as the untinted glass over a neutral backdrop,
so dark glass stays dark; a saturated backdrop shifts, brighter in the
channels the tint opens and darker in the others. The chamfer and the ring's
light pass the same tinted glass: under `accent "ring"` the band and the
focus light brighten with the tint, and under `accent "none"` a near-white
`ring-color` band dims.
Near-white glass has little room for hue at its own luminance, so the tint
barely shows on the default glass. Without an accent the glass is unchanged,
and the tint follows the accent's crossfade in and out. The arithmetic is
in [glass-optics.md](glass-optics.md#accent-tint).
On dark glass the recommended weight is `1`. The hue reads mainly through
the band and the focus light, which pass the tinted glass; the body keeps
its luminance, so under a translucent window it shifts only a few levels in
hue, and lower weights barely register. The default stays 0.

Two limits follow from the placement. An opaque window shows no ring: the
band lies wholly under the face and there is no fallback band on the
chamfer. A face narrower than `2 * ring-gap` on either axis has no beam
line, and the ring — accent band, beam and resting glow — is not drawn
on it. A `ring-sweep-ms`, `ring-inset`
or `ring-drift-hz` line is rejected with the replacement named. Set
`focus-ring { off }` (globally or in a window rule) for material windows so
the gradient ring does not draw a second ring; non-material windows keep
whatever ring the layout configures.

**Retired keys.** `ring-sweep-ms`, `ring-inset` and `ring-drift-hz` are
rejected by name with their replacement
(`ring-sweep-ms was replaced by ring-beam-speed`,
`ring-inset was replaced by ring-gap`); none is reinterpreted. Prism's
`prism migrate` maps `glass.ring.sweepMs` to `glass.ring.beamSpeed` (`0`
stays `0`, any positive value becomes the default 300). The filament is on
by default, so a material window with no `response` block shows the ring on
focus; `focus "none"` in the `default` response leaves a focused window
unlit. Two rings on screen are the layout's gradient ring under the
filament: turn it off with `focus-ring { off; }`.

### Focus tilt

`view-tilt` turns the pane briefly in depth when it gains focus: window
content and glass swing as one body about the pane's centre, across the
direction focus came from, and settle back to exactly face-on after
`view-tilt-settle` ms. The side the focus came from presses in first; one
small overshoot follows. While it turns, perspective engages with the angle
and the slab's rounded edge and side wall come into view. `view-distance`
is the camera's distance in multiples of the pane's extent along the swing:
smaller is stronger perspective, larger shows more wall. On a horizontal
swing the advancing wall shows on every moving frame when
`sin(view-tilt) > 1 / (2 · view-distance)`.

A swing starts only when the window's border and focus ring are off, its
background effect draws nothing (no xray or blur, `noise 0`, `saturation 1`;
Prism writes `blur false; noise 0; saturation 1`), it is not fullscreen or
opening, and `signal { motion "full"; }` with animations on. A pane
refocused mid-swing finishes the swing it has.
The policy cuts a running swing to rest on the next frame. At rest nothing
changes. How the swing moves and how the tilted slab is drawn is in
[motion.md](motion.md#rigid-tilt).

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

`reduced` changes sustained `flash` to `pulse`, `pulse` to `breathe`, and an
impulse `flash` response to `sweep`. `off` makes sustained motion static and
drops impulse effects, while retaining the static accent and level
crossfade. `material-signal` is the baseline signal crossfade; it defaults to
400 ms with `ease-out-cubic` and follows the normal animation configuration,
including `animations { off }`. The focus filament fades in and out with
`material-signal` too; `reduced`, `off`, and `animations { off }` skip its
focus-gain beam.

`idle-after-ms <int>` — sustained attention motion (`breathe`, `pulse`,
`flash`) settles to the static indication (level and accent lit, no pulse),
and Aurora holds its phase, once no input has arrived for this long. Attention
resumes on its absolute clock; Aurora resumes from its held phase. Default
30000; `0` disables both gates; at most 3600000, and the error
is `idle-after-ms must be at most 3600000`. What the gate holds and what it
leaves running is in [Redraw and idle budget](#redraw-and-idle-budget).


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
unlock count as activity; idle inhibitors and client damage do not. The
timer is armed at startup, so a compositor that never receives input still
goes idle. Once idle:

- sustained attention motion is `Static`: level and accent stay lit;
- the shared optic timeline holds (`OpticTimeline` in `src/activity.rs`), so
  an Aurora keeps its phase and requests no deadline even on an unfocused,
  signal-free window;
- impulses, crossfades, the focus beam and the tilt are not gated: they are
  finite.

On the next activity attention resumes on its absolute clock, in step with
windows that never stopped, and Aurora resumes from its held phase without
catching up. Either edge redraws only when some tile renders differently
(`Layout::set_input_active`). A config reload recomputes idleness from the
quiet time already elapsed. `src/tests/attention_idle.rs` covers these
edges.

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
