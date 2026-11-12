# Material performance guide

Type: guide; document types 1. Where the GPU, CPU, memory and power go when
glass is drawn, which parameters are expensive and why, and the levers that
reduce cost. Each measured number states its basis: what was measured, how,
on which host and when. The reference host is an AMD Ryzen Threadripper 1950X
with an NVIDIA RTX 3070; nested runs put niri under a headless Weston on that
GPU unless a run is marked llvmpipe. Claims marked *code* are read from the
source and not measured. The pass order is in
[the render pipeline](render-pipeline.md); read it first. How runs are
recorded and gated is in [captures.md](captures.md).

Two facts frame everything else:

- **Damage decides whether anything runs.** A quiet scene does no material
  work: Tracy traces of nested niri on the reference host (2026-09-25)
  showed 0 redraws and 0 material draws in every 20 s quiet window after
  column moves and resizes, with animations off, jelly, a pinned aurora and a
  drifting aurora under `motion "off"`, and over one 600 s hold.
  A window redraws when its own inputs change or when other output damage
  overlaps it (section 1).
- **On a discrete GPU, frame time is not the cost that matters; wakeups and
  power are.** On an RTX 3070 one material draw takes microseconds to a
  fraction of a millisecond, and a whole backdrop damage under 2 ms
  (sections 2 and 3). A drifting aurora at 4 Hz added close to 1 W of board
  power at idle (section 4). Integrated GPUs are unmeasured.

## 1. Three tiers of work

| Tier | Work | Runs when | Scales with | Shared by |
| --- | --- | --- | --- | --- |
| Backdrop chain | Sharp render of the Background layer, backdrop grain, dual-Kawase blur, roughness pyramids (`effect_buffer.rs`, `grain.rs`, `blur.rs`) | The Background layer is damaged, or the blur or grain options change | Output pixels | Every material window on that output and render target |
| Window | Window body into the tile offscreen; CPU `material_dynamics` and the input fingerprint (`tile.rs`, `material/mod.rs`) | Each frame the tile renders, while building render elements; the offscreen draws only on client damage | Window count | — |
| Fragment | The material shader over the damaged part of the element | The fingerprint changed (client commit, backdrop commit, a moving optic, jelly or signal input), or accumulated output damage intersects the element, such as an overlapping window moving | Transparent window area plus the bevel band, × per-fragment samples | — |

*Code:* the material element does not narrow its own damage
(`MaterialRenderElement` keeps smithay's default `damage_since`), so a change
to any of its inputs, such as a cursor blink in a terminal or a wallpaper
change behind it, redraws the whole slab. Opaque window pixels still return
after one texture read (step 0 of
[the per-fragment program](render-pipeline.md#3-inside-the-material-shader-per-fragment)).

## 2. The backdrop chain: memory and caching

Each output owns an `Xray` with a `background` and a `backdrop`
`EffectBuffer` for each of three render targets (output, screencast, screen
capture), all `Abgr8888` at full output size (*code*: `xray.rs`,
`effect_buffer.rs`). Writing *F* for one full-output frame (14 MiB at
2560×1440), one buffer holds up to:

| Texture | Size | Exists when |
| --- | --- | --- |
| Sharp | 1 F | A consumer prepared the buffer |
| Grain | 1 F | Any material places noise at `site="backdrop"` |
| Blur chain | `passes + 1` textures, halving; 1.33 F at 3 passes | A material with effective `backdrop-blur` drew |
| Sharp pyramid | ~0.33 F, levels to 1×1 (11 at 1440p) | `roughness > 0` and `ior > 1` on the sharp source |
| Blurred pyramid | ~0.33 F | The same, on the blurred source |

A material samples both buffers, so a normal desktop with blur and roughness
holds about 2 × 2.7 F ≈ 75 MiB per 1440p output; screencast and capture
targets each add their own set when they render. Textures are allocated
lazily. Disabling a feature alone keeps its textures allocated; they are
replaced on an output size or renderer context change, and lowering blur
`passes` drops the surplus levels (*code*: `effect_buffer.rs`, `prepare`;
`blur.rs`).

Invalidation (*code*: `effect_buffer.rs`, `cleared_by`):

- Sharp damage or a grain-option change clears grain, blur and both
  pyramids. The sharp render itself is damage-tracked, but any damage reruns
  the whole downstream chain at full size, and a dirty pyramid rebuilds every
  level.
- A blur-option change clears only the blurred result and its pyramid.
- Roughness and `ior` only select levels; they invalidate nothing. One
  pyramid build serves every window whatever roughness each picks.

Measured with Tracy GPU spans on the reference host, nested at 1280×720
from a TTY with the desktop stopped, a sharp and a blurred transparent window
with roughness 0.5, wallpaper changes and noise-amount drags as stimuli
(2026-10-05): one backdrop damage, chain and both material draws together,
totals 1.83–1.88 ms; a wallpaper change damages twice. The GPU span medians inside
it are per call, and one damage runs several calls: `Blur::render`
0.45 ms, `Prefilter::downsample` 0.27 ms, `Grain::render` 0.09 ms. Static
scenes did no chain work. A change to the noise amount at
`site="backdrop"` reruns the chain on both buffers, 1.42 ms per change,
against 0.29 ms at `site="glass"`, which only redraws the material. In
software rendering (Tracy on Mesa llvmpipe, 2026-09-02) one wallpaper change
caused one pyramid rebuild per source, then reuse: no further rebuild in the
10 s before the trace ended.
Blur `offset` changes the spread, not the cost; passes beyond about three
add little, because the full and half levels dominate (*code*).

## 3. The material draw, per fragment

Only pixels where the window is transparent, or in the slab band outside it,
do glass work. A terminal with a fully transparent background is glass over
its whole area; a window with an opaque surface pays only on its bevel band.

Texture reads per fragment (*code*: `main.frag`, `prelude.frag`,
`material/mod.rs::tap_count`):

| Case | Reads | What sets it |
| --- | --- | --- |
| Opaque window pixel | 1 | Early return |
| Default glass | 2–3 | One tap; backdrop read only where the background is translucent or outside the workspace |
| Smeared glass | taps × 1–4 | `tap_count = clamp(ceil(8 · max(anisotropic-blur, chromatic-aberration)), 2, 8)`, or 1 when both are 0 |
| Aberration | × 3 per tap | One full background sample per channel |
| Fractional roughness level | × 2 per buffer | Low and high pyramid level mixed |
| Worst case | 97 | Aberration 1, fractional roughness, translucent background |

A flash impulse adds to aberration for its duration, so it can briefly leave
the single-tap path.

Arithmetic, by trigger (*code*):

- `distortion > 0`: 12 simplex evaluations per pixel, every frame the glass
  draws. `jelly-ripple`: 4 more, only while a spring runs.
- Aurora: 3 simplex, a refraction and a `pow`. Ring: band distances, a few
  `exp` and `pow`, an `atan`; it runs whenever a focus or accent response is
  set, at rest too, over the whole glass. Iridescence and saturation are a
  few operations.
- Glass-site noise: `white` 1 hash, `fine` 9, `lightness` fine plus an Oklab
  round trip. `site="backdrop"` moves grain out of the fragment shader into
  the cached chain of section 2.
- Neutral optics are not compiled out; each branches on a uniform. The
  program is assembled with every optic (`shaders/mod.rs`).

Measured:

- RTX 3070, 456×640 window, roughness 0: 5–15 µs per whole material draw
  (`MaterialRenderElement::draw` GPU spans, medians of 14 warmed draws in
  each of three runs, nested at 1280×720, 2026-09-11). Plain glass and
  `aurora 0.5` at 4 Hz both had a 9.728 µs median; GPU clock variation left
  the optics' relative overhead unresolved.
- Whole-draw medians with the behind stage (noise and saturation) or the
  within stage (ring and aurora) active equalled the neutral baseline: every
  median was 0.020 ms (20,480 ns), +0.0 %, in Tracy matrices of nine rotated
  sub-runs, all at P8, nested from a TTY with the desktop stopped
  (2026-09-18). No difference was resolved, and the stages were not timed on
  their own.
- RTX 3070, the two-window scene of section 2 (the same 2026-10-05 run):
  material draw medians 0.21–0.27 ms per call on backdrop damage,
  0.03–0.15 ms per call on a noise amount change.
- llvmpipe, relative only (Tracy draw medians of 14 events between 20 and
  28 s, nested under a 1280×720 headless Weston on Mesa llvmpipe,
  2026-09-02): roughness 0.08 +8.7 %; adding `anisotropic-blur 1` and
  `chromatic-aberration 1` +508 %. Aurora +28.9 % (llvmpipe, a 456×640
  window, 2026-09-10): `aurora 0.5` at 4 Hz cost 5.552 ms per material draw
  against 4.308 ms plain. Software timings rank shader cost; they do not
  predict GPU cost.

## 4. Redraws: what keeps the compositor awake

| Source | Redraw rate | Settles | Basis |
| --- | --- | --- | --- |
| Aurora drift | `drift-hz` while input is active | One flush redraw after `idle-after-ms` (default 30 s), then 0 until input | Measured, a |
| Attention (breathe, pulse, flash) | One timer per output; 8–27/s; ten windows cost as one | To static after idle | Measured, b |
| Ring beam | Refresh rate for `(perimeter + tail) / ring-beam-speed`, or `ring-beam-decay / ring-beam-speed`: about 16 s for a 1280×720 pane at the default 300 px/s | 0 once the run ends; settled focus costs nothing | *code* for the run, c |
| Rigid tilt | Refresh rate for `view-tilt-settle` (900 ms default) after a focus gain, while the resting slab or the swing's footprint is in view; runs on the unadjusted clock, so `animations.slowdown` does not stretch it | 0 once the run ends | *code*: `tile.rs` |
| Focus and signal crossfades, impulses | Refresh rate for 0.4–1.5 s | 0 | *code*: `tile.rs` |
| Jelly | No clock; rides move and resize animations | Resting jelly: 0 | Measured, d |

- a. Tracy redraw counts, nested at 1280×720 from a TTY with the desktop
  stopped (2026-10-03): 18 redraws per 4.5 s active at 4 Hz and 9 under
  `motion "reduced"`, one edge-flush redraw, then zero redraws and material
  draws until resume. Before input settling existed, llvmpipe redraw counts
  over 20 s (2026-09-10) were 80 at `drift-hz 4`, 40 under reduced motion
  and 0 when pinned or with motion off.
- b. Tracy `Niri::redraw` counts over 20 s, nested (2026-09-04): breathe
  8.0/s, flash 15.9/s, pulse 26.6/s; ten breathe or ten flash windows
  counted the same as one.
- c. The run length is `ring.rs`'s `run_length` at the default speed.
  Tracy redraw counts, nested on llvmpipe (2026-09-21): 0 redraws in the
  12 s after a beam run, and 0 for a focused window whose beam had finished.
- d. The 2026-09-25 quiet windows above, including the 600 s jelly hold.

Board power was measured with an uninstrumented niri on DRM from a TTY with
the desktop stopped: DP-1 at 3440×1440@59.999, scale 1, two transparent
kitty windows, the board sampled at 1 Hz over 48 windows of 60 s warmup and
30 s observation in ABBA/BAAB blocks, on 2026-09-27 and again on 2026-09-29
on a later build. Every sample was in P8 at 210 MHz, and the idle baseline was
about 10.7 W (10.61 W and 10.845 W). Resting jelly showed no resolvable
increase over plain glass in either run (+0.015 W, upper estimate 0.13 W;
then +0.005 W, upper estimate 0.19 W). A drifting aurora over resting jelly
added +0.85–0.96 W at 4 Hz (upper estimates 1.11 W and 1.37 W) and
+0.68–0.77 W at 2 Hz (upper estimates 0.98 W and 1.03 W), so halving the
rate saved only about a fifth. The clocks did not ramp; redraws holding the
board out of deeper idle are a possible cause that these runs do not
establish. Board power excludes CPU, monitor and total-system energy.

## 5. Focused and unfocused glass

The split is two material definitions chosen by `is-active` window rules
([material configuration](material-config.md); Prism writes the second from
`glass.inactive.*` when `glass.focusSplit` is set). Costs that follow
(*code*):

- A focus change swaps definitions, so both windows get a new
  `MaterialState`: a new offscreen and a new element id, which is full damage
  once.
- If the two definitions differ in `backdrop-blur` and both use roughness,
  the output keeps and rebuilds both the sharp and the blurred pyramid.
- The unfocused definition is the natural place for concessions, since most
  glass on screen is unfocused.

## 6. Hidden and covered windows

Measured with Tracy on nested niri from a TTY with the desktop stopped, a
hidden probe in an inactive workspace, a hidden tab, an offscreen column and
under an opaque cover (2026-10-01): hidden windows issue no material draws,
and a hidden client drops from 60.2 fps to 1.0–1.2 fps (about 1 Hz), but each
of its commits still queues an output redraw, 20 in 20 s. A window in an
offscreen column or under an opaque cover still re-renders its offscreen on
those redraws: `Tile::render_inner` renders it while building the tile's
elements, before smithay's damage and occlusion pass decides what to draw
(*code*). Only culling the tile before `render_inner` avoids that work. A
sustained optic under an opaque cover is unmeasured; *code*:
`Tile::tick_deadline` stops scheduling only for windows out of view, not
covered ones. A window under an opaque *glass* window still draws and gets
full-rate frame callbacks, because no material element declares an opaque
region (*code*, unmeasured). A cull would have to come before `Tile::render`
builds the tile's elements, so that the effect-buffer `prepare`, the offscreen
render, `material_dynamics` and the deadline report are skipped together.

## 7. Parameters by cost

| Parameter | Cost | Cheap setting |
| --- | --- | --- |
| `chromatic-aberration` | Taps up to 8, × 3 channels | 0 |
| `anisotropic-blur` | Taps up to 8 | 0; frost through `backdrop-blur` and `roughness` instead |
| `roughness` | Pyramid build per backdrop damage, memory, × 2 reads when fractional | 0, or `ior` 1 |
| `backdrop-blur` | Blur chain per backdrop damage, 1.33 F per buffer | Same value across definitions, so one pyramid kind |
| `noise site="backdrop"` | 1 F per buffer and a full-frame pass per damage; one setting every material must share | `glass` or `film`; backdrop grain is erased by 3 blur passes (none left in the 2026-10-05 nested smoke) |
| `noise type=` | white < fine < lightness | white |
| `distortion` | 12 simplex per pixel per draw | 0 |
| `jelly-ripple` | 4 simplex per pixel during motion | Cheap at rest |
| `aurora` | Redraws at `drift-hz`; close to 1 W in the measured setup | `drift-hz 0`, `aurora 0`, or `signal { motion "off" }`; a shorter `idle-after-ms` |
| `ring-beam-speed`, `ring-beam-decay` | Refresh-rate run per focus gain | Faster speed or shorter decay; speed 0 or `signal { motion "off" }` removes the run |
| `view-tilt` | During a swing only: the footprint quad (the bounding box of the projected slab and window, merged with the resting area and cropped to the view; its size grows with `view-tilt` and `thickness` and shrinks as `view-distance` grows) and the traced solid on the bevel band and wall, for `view-tilt-settle` per focus gain; about 1.47 ms per material draw (median; 1.95 ms maximum) and 32 ms of GPU time per gain at `view-tilt 10`, against 0.66 ms at rest (Tracy, nested from a TTY with the desktop stopped, 330 draws within 0.9 s of six focus gains at P8, RTX 3070, 2026-10-16); at rest: nothing, since the view path is a second program bound only while a swing runs (an A/B against the build without it, within 0.002 ms) | 0 |
| Window transparency, `bevel`, `offset-*` | The area that runs the shader | Opaque content, a narrower band |
| `reflection` | One prefiltered background read per bevel fragment, two when the prefilter mix is fractional | 0 |
| `bevel-profile` above 1 | A few operations per bevel fragment (the height field and its normal) | 1, a planar chamfer |
| `saturation`, `iridescence`, `edge-highlight`, `attenuation-*`, ring at rest | A few operations | — |

## 8. Open levers

Not built yet:

- Culling invisible and covered windows before `Tile::render_inner`, which
  skips their offscreen renders and their scheduling.
- Narrowed damage for `MaterialRenderElement`, so a cursor blink does not
  redraw the whole slab, and opaque regions, so elements behind its opaque
  parts are skipped.
- Cost-tiered unfocused glass.
- Pyramid rebuilds only up to the highest requested level; freeing chain
  textures when a feature is turned off; building xray elements only when
  something consumes them (FIXME in `niri.rs`, `update_xray_render_elements`).
- A per-frame cost counter over IPC and a per-element cost model.

## 9. Measuring

Use the [capture protocol](captures.md):
every run writes `capture.json` through `scripts/capture/capture-meta`, which records
provenance and the environment and refuses a busy host, so two runs are
comparable. Power and idle runs need the host to themselves
([capture host setup](capture-host-setup.md); parked under `tasks quiet`).
Tracy zones: `Blur::render`, `Prefilter::downsample`, `Grain::render`, and
`MaterialRenderElement::draw` on the GPU. These are per-call spans; sum them
per stimulus before calling a figure a cost. GL timer medians on NVIDIA step
in 1024 ns, so a difference on a small window can read as +0.0 %; size the
scene so the effect clears the step.
