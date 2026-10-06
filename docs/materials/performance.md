# Material performance guide

Where the GPU, CPU, memory and power go when glass is drawn, which parameters
are expensive and why, and the levers that reduce cost. This is a map, not
the evidence: every measured number below links to the section that measured
it, and claims marked *code* are read from the source and not yet measured.
The pass order is in [the render pipeline](render-pipeline.md); read it first.

Two facts frame everything else:

- **Damage decides whether anything runs.** A quiet scene does no material
  work: the measured quiet windows show 0 redraws and 0 material draws
  ([idle budget, *What is established*](2026-09-11-idle-budget-evidence.md#what-is-established)).
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
| Fragment | `material.frag` over the damaged part of the element | The fingerprint changed (client commit, backdrop commit, a moving optic, jelly or signal input), or accumulated output damage intersects the element, such as an overlapping window moving | Transparent window area plus the bevel band, × per-fragment samples | — |

*Code:* the material element does not narrow its own damage
(`MaterialRenderElement` keeps smithay's default `damage_since`), so a change
to any of its inputs, such as a cursor blink in a terminal or a wallpaper
change behind it, redraws the whole slab. Opaque window pixels still return
after one texture read ([render pipeline §3](render-pipeline.md#3-inside-materialfrag-per-fragment),
step 0).

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

Measured, RTX 3070, nested headless Weston, a sharp and a blurred
transparent window with roughness 0.5
([noise placement, *Tracy costs*](2026-10-05-noise-placement-evidence.md#tracy-costs)):
one backdrop damage, chain and both material draws together, totals
1.83–1.88 ms; a wallpaper change damages twice. The GPU span medians inside
it are per call, and one damage runs several calls: `Blur::render`
0.45 ms, `Prefilter::downsample` 0.27 ms, `Grain::render` 0.09 ms. Static
scenes did no chain work. A change to the noise amount at
`site="backdrop"` reruns the chain on both buffers, 1.42 ms per change,
against 0.29 ms at `site="glass"`, which only redraws the material. In
software rendering one wallpaper change caused one pyramid rebuild per
source, then reuse
([roughness smoke, *Tracy acceptance*](2026-09-02-material-roughness-smoke.md#tracy-acceptance)).
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

- RTX 3070, 456×640 window: 5–15 µs per whole material draw
  ([hardware evidence, *Material-pass timing*](2026-09-11-material-hardware-evidence.md#material-pass-timing)).
  Whole-draw medians with the behind stage (noise and saturation) or the
  within stage (ring and aurora) active equalled the neutral baseline; no
  difference was resolved in these captures, and the stages were not timed
  on their own
  ([render order, *Strict TTY cost matrix accepted*](2026-09-12-material-render-order-evidence.md#strict-tty-cost-matrix-accepted-2026-09-18)).
- RTX 3070, the two-window scene of section 2: material draw medians
  0.21–0.27 ms per call on backdrop damage, 0.03–0.15 ms per call on a noise
  amount change
  ([noise placement, *Tracy costs*](2026-10-05-noise-placement-evidence.md#tracy-costs)).
- llvmpipe, 1280×720, relative only: roughness 0.08 +8.7 %; adding
  `anisotropic-blur 1` and `chromatic-aberration 1` +508 %
  ([roughness smoke, *Tracy acceptance*](2026-09-02-material-roughness-smoke.md#tracy-acceptance)).
  Aurora +28.9 %
  ([aurora evidence, *Metrics*](2026-09-10-material-aurora-evidence.md#metrics)).
  Software timings rank shader cost; they do not predict GPU cost.

## 4. Redraws: what keeps the compositor awake

| Source | Redraw rate | Settles | Evidence |
| --- | --- | --- | --- |
| Aurora drift | `drift-hz` while input is active | One flush redraw after `idle-after-ms` (default 30 s), then 0 until input | [optic settling, *Verdict*](2026-09-30-optic-settling-evidence.md#verdict) |
| Attention (breathe, pulse, flash) | One timer per output; 8–27/s; ten windows cost as one | To static after idle | [signals smoke, *GLES redraw behavior*](2026-09-03-material-signals-smoke.md#gles-redraw-behavior) |
| Ring beam | Refresh rate for `(perimeter + tail) / ring-beam-speed`, or `ring-beam-decay / ring-beam-speed`: about 16 s for a 1280×720 pane at the default 300 px/s | 0 once the run ends; settled focus costs nothing | [ring beam, *Redraw counts*](2026-09-19-ring-beam-evidence.md#redraw-counts-2026-09-21-and-2026-09-20) |
| Focus and signal crossfades, impulses | Refresh rate for 0.4–1.5 s | 0 | *code*: `tile.rs` |
| Jelly | No clock; rides move and resize animations | Resting jelly: 0 | [idle budget, *What is established*](2026-09-11-idle-budget-evidence.md#what-is-established) |

Board power, one RTX 3070 experiment: DRM at 3440×1440, the GPU in P8
throughout, idle baseline about 10.7 W
([idle budget, *Isolated board power*](2026-09-11-idle-budget-evidence.md#isolated-board-power)
and [*Re-run*](2026-09-11-idle-budget-evidence.md#re-run-on-48ba40a1)).
Resting jelly showed no resolvable increase over plain glass in either run.
A drifting aurora added +0.85–0.96 W at 4 Hz and +0.68–0.77 W at 2 Hz, so
halving the rate saved only about a fifth. The clocks did not ramp; the
evidence names redraws holding the board out of deeper idle as a possible
cause and does not establish it.

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
  glass on screen is unfocused (idea `material-a91346`).

## 6. Hidden and covered windows

Measured
([hidden-window attribution, *Attribution*](2026-09-30-hidden-window-attribution-evidence.md#attribution)):
hidden windows issue no material draws, and a hidden client drops to about
1 Hz, but each of its commits still queues an output redraw. A window in an
offscreen column or under an opaque cover still re-renders its offscreen on
those redraws: `Tile::render_inner` renders it while building the tile's
elements, before smithay's damage and occlusion pass decides what to draw
(*code*). Only culling the tile before `render_inner` avoids that work. A
sustained optic under an opaque cover is unmeasured; *code*:
`Tile::tick_deadline` stops scheduling only for windows out of view, not
covered ones.

## 7. Parameters by cost

| Parameter | Cost | Cheap setting |
| --- | --- | --- |
| `chromatic-aberration` | Taps up to 8, × 3 channels | 0 |
| `anisotropic-blur` | Taps up to 8 | 0; frost through `backdrop-blur` and `roughness` instead |
| `roughness` | Pyramid build per backdrop damage, memory, × 2 reads when fractional | 0, or `ior` 1 |
| `backdrop-blur` | Blur chain per backdrop damage, 1.33 F per buffer | Same value across definitions, so one pyramid kind |
| `noise site="backdrop"` | 1 F per buffer and a full-frame pass per damage; one setting every material must share | `glass` or `film`; backdrop grain is erased by 3 blur passes |
| `noise type=` | white < fine < lightness | white |
| `distortion` | 12 simplex per pixel per draw | 0 |
| `jelly-ripple` | 4 simplex per pixel during motion | Cheap at rest |
| `aurora` | Redraws at `drift-hz`; close to 1 W in the measured setup | `drift-hz 0`, `aurora 0`, or `signal { motion "off" }`; a shorter `idle-after-ms` |
| `ring-beam-speed`, `ring-beam-decay` | Refresh-rate run per focus gain | Faster speed or shorter decay; speed 0 or `signal { motion "off" }` removes the run |
| Window transparency, `bevel`, `offset-*` | The area that runs the shader | Opaque content, a narrower band |
| `saturation`, `iridescence`, `attenuation-*`, ring at rest | A few operations | — |

## 8. Open levers

Not built yet:

- Culling invisible and covered windows before `Tile::render_inner`, which
  skips their offscreen renders and their scheduling
  (`material-7afc31`).
- Narrowed damage for `MaterialRenderElement`, so a cursor blink does not
  redraw the whole slab, and opaque regions, so elements behind its opaque
  parts are skipped (`material-7f6d0e`).
- Cost-tiered unfocused glass (`material-a91346`).
- Pyramid rebuilds only up to the highest requested level; freeing chain
  textures when a feature is turned off; building xray elements only when
  something consumes them (FIXME in `niri.rs`, `update_xray_render_elements`).
- A per-frame cost counter over IPC (`material-a0cbb0`) and a per-element
  cost model (`material-31074f`).

## 9. Measuring

Use the
[capture protocol](../specs/2026-09-11-material-capture-protocol-design.md):
every run writes `capture.json` through `tools/capture-meta`, which records
provenance and the environment and refuses a busy host, so two runs are
comparable. Power and idle runs need the host to themselves
([capture host setup](capture-host-setup.md); parked under `tasks quiet`).
Tracy zones: `Blur::render`, `Prefilter::downsample`, `Grain::render`, and
`MaterialRenderElement::draw` on the GPU. These are per-call spans; sum them
per stimulus before calling a figure a cost. GL timer medians on NVIDIA step
in 1024 ns, so a difference on a small window can read as +0.0 %; size the
scene so the effect clears the step.
