# Noise layers: verification evidence

**Result:** the nested-Weston smoke passes all six assertions of design
§7.2 against the `4a8b2072` baseline at `558bfa02` (quiet TTY run,
2026-10-06). Tracy costs are measured for all six cases. The first strict full
run found one pixel of single-node lightness glass one code off the baseline;
`558bfa02` fixes it and the rerun is byte-exact.

Owner's look (2026-10-07): the lattice shows at scale 8 — "scale-8 white grain reads blocky". Per §7.1 this reshapes `scale > 1` before the merge: `e5edd631` reconstructs scaled grain with a cubic B-spline over 4×4 lattice points instead of Hermite over 2×2.

**B-spline rerun (2026-10-07, evening, local time):** the smoke passes all six assertions again at
`e5edd631`, every identity still byte-exact against `4a8b2072`; the contact
sheet below is regenerated from that run, and a before/after sheet sets the
two lattices side by side. Four fine layers at scale 8 now cost 1.19 ms per
damaged frame (Hermite 0.65 ms). The B-spline shader also costs every
material draw about 0.034 ms (15 %) with noise off, 0.039 ms (13 %) with
one fine layer; same-session A/B runs
pin it to `noise.frag` (see Cost).

Owner's second look (2026-10-08, B-spline sheets): "scale 8 looks better"; merge now, with the idle cost left to material-03fdf3 on the owner's word.

## Pinned revisions and artifacts

Artifact directories are under `$NIRI_MATERIAL_WORK_ROOT`, the host-local
evidence root.

| Run | Source | Outcome |
| --- | --- | --- |
| `noise-layers-pilot-1791337568` | `4124e902` | every cell and assertion passed; release refused: `systemd-tmpfiles-clean.timer` fired at 21:57:01 |
| `noise-layers-full-1791338406` | `4124e902` | failed assertion 2: lightness glass vs baseline, one pixel |
| `noise-layers-full-soft-1791338868` | `4124e902` | that one identity recorded rather than asserted (scratch copy of the script); all else passed |
| `noise-layers-ident-1791340828` | `4124e902` + fix | glass identities only (white, fine, lightness): all AE 0 |
| `noise-layers-full2-1791341162` | `558bfa02` | **passed**, strict; clean release |
| `noise-layers-cost-pilot-1791339892` | `4124e902` | passed (3 damage steps per case) |
| `noise-layers-cost-full-1791340265` | `4124e902` + fix, uncommitted | passed; superseded (the shader was edited during its preflight) |
| `noise-layers-cost-full2-1791342346` | `558bfa02` | **passed**; clean release |
| `noise-layers-bspline-pilot-1791425925` | `372581d9` (`e5edd631` code) | passed, 23 sub-runs settled; clean release |
| `noise-layers-bspline-full-1791426646` | `372581d9` (`e5edd631` code) | **passed**, strict; 45 sub-runs settled; clean release |
| `noise-layers-bspline-cost-pilot-1791427673` | `372581d9` (`e5edd631` code) | passed (3 damage steps per case) |
| `noise-layers-bspline-cost-full-1791428043` | `372581d9` (`e5edd631` code) | **passed**; clean release |
| `noise-layers-pixels-pilot-1791520292` | `8e0f06c2` (merged branch) | pixels lane; failed assertion 2: fine glass vs the `4a8b2072` baseline, AE 8.3 |
| `noise-layers-pixels-pilot-1791542879` | `efe44dec` + the re-pin, uncommitted | pixels lane; **passed** against the `37de154e` baseline; clean release |

The two pixels-lane pilots (material-52dc6e) ran on the desktop in use, after
the noise merge `1ae20983`. Against `4a8b2072`, the merged branch's frame
differs by one code value in the bevel band and just outside the window, the
face ROI identical: glass edges (material-be611b) landed beside this work and
change the bevel by design. The smoke now pins the merge's first parent
`37de154e` (release niri
`23f7fe0f740bfd0a74e54bc46c2b70da08acc54dd928e19325592a39bd2732bf`), against
which every identity holds; the passing pilot's `metrics.txt` equals
`noise-layers-bspline-pilot-1791425925`'s line for line (material-a8232f).

The B-spline runs were built from `e5edd631`'s code at `372581d9`, whose
working tree differed only in task records; the binaries' SHA-256s are the
pin: niri
`54e05f55b00cbc73840d5c92a851f858030e32e057872448fb6306668a84b5fb`, niri-tracy
`edca1d1adfbc14d5b2e57785495fe1dc824a7d6281043286ba6eef940d241a09`, the same
baseline. Binaries of the `558bfa02` runs: niri
`687ebab52188ad2e2da334354ccd1f92bdc11eaac48d0c1f22b4218d474b529e`, niri-tracy
`cb21ad039beaeae2e0fa9ce3e2e9249a81d65c45a9728b9d0db9bb91fcb801e4`, baseline
`4a8b2072` release `12920dea610789b1a75d478eb251c74a7391db850cefb0fb5acfc4f36bccc7e9`.
Every run is on the headless lane: desktop absent, quiet preflight (GPU 0 %
at P8, no clients), the 13 user timers held. Renderer: `OpenGL ES 3.2 NVIDIA
615.71.09` on an RTX 3070 (the nested parent's `weston.log`).

`tools/capture-meta show` for the passing full run, less its sub-run list
(all 45 settled):

```
run noise-layers-full2-1791341162  task material-a4d874  fixture glass-noise-layers-smoke.sh  lane headless
started 2026-10-06T22:46:02-04:00  host <capture host>
provenance
  source 558bfa02865cad44757645aee31bd4b09eaeac00 material-3fcba2
  binary niri 687ebab52188ad2e2da334354ccd1f92bdc11eaac48d0c1f22b4218d474b529e
  binary niri-tracy cb21ad039beaeae2e0fa9ce3e2e9249a81d65c45a9728b9d0db9bb91fcb801e4
  binary niri-baseline 12920dea610789b1a75d478eb251c74a7391db850cefb0fb5acfc4f36bccc7e9
  pilot=0
  baseline=4a8b2072
hold  desktop absent  13 items
  not held: dropbox.service (not_running)
hold end  restore complete (release)  scan clean
```

The pilot's, the same apart from source `4124e902` (dirty only in task
records), binary `29d7b42f…`, `pilot=1`, 23 sub-runs settled, and:

```
hold end  restore complete (release)  scan disturbed
  timer-fired systemd-tmpfiles-clean.service (system) at 2026-10-06T21:57:01-04:00
```

## The lightness identity finding

At `4124e902`, `one-lightness-glass` and `base-lightness-glass` differed at a
single pixel, (219, 578): green 121 against 120, the same in both captures of
each session. White and fine were byte-identical at all three sites, and so
was lightness at backdrop and film. The uniforms carry the same f32 values as
the baseline (slot 0 of each vec4), and the layered `noise_behind` computes
the same expression. So the difference is in how the driver compiles the
restructured lightness arithmetic (Oklab's cube roots behind a running value
and an `isLinear` select), not in the values.

`558bfa02` gives `noise_behind` a path for a lone slot-0 layer at scale 1
that runs the `4a8b2072` body verbatim. More layers, or any scale above 1,
take the layered path. The glass-only identity run and then the strict full
run measured AE 0 for every kind. In-process `noise` tests 27/27 and
`just test-fast` (504) passed.

## Nested smoke at `e5edd631` (B-spline lattice)

Run `noise-layers-bspline-full-1791426646`. Assertions 1–4 and 6 hold as at
`558bfa02` below: determinism 44 × AE 0; byte identity against the baseline AE
0 for every kind and site and for the inherited node; `scale=1` against
omitted AE 0 for all three kinds; quadrature ratio 1.4159; slots 1–3 at sd
0.08657/0.08682/0.08681 against slot 0's 0.08665, differences
0.1223/0.1227/0.1228. Scale 1 takes the per-pixel path, which `e5edd631`
leaves alone, so only assertion 5 moves:

| Kind | Scale | sd | min class | max class | LF ratio | LF ratio, Hermite |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| white | 1 | 0.08662 | — | — | 0.2518 | 0.2518 |
| white | 2 | 0.08775 | 0.9997 | 1.0003 | 0.7152 | 0.5142 |
| white | 4 | 0.08681 | 0.9993 | 1.0010 | 0.8899 | 0.7026 |
| white | 8 | 0.08824 | 0.9936 | 1.0052 | 0.9738 | 0.9508 |
| fine | 1 | 0.08665 | — | — | 0.1250 | 0.1250 |
| fine | 2 | 0.08730 | 0.9999 | 1.0001 | 0.4740 | 0.2848 |
| fine | 4 | 0.08662 | 0.9981 | 1.0029 | 0.7756 | 0.5433 |
| fine | 8 | 0.08802 | 0.9915 | 1.0083 | 0.9518 | 0.9355 |

Every sd is within 1.9 % of its scale-1 value (limit 10 %) and every position
class within 0.9 % (limit 10 %; Hermite reached 1.7 %): the B-spline's exact
per-position normalisation shows as flatter classes. The low-frequency ratio
still rises at every step, and higher than Hermite's at each scale, as a
smoother reconstruction of the same lattice gives.

## Nested smoke (full run at `558bfa02`, Hermite lattice)

Assertion 1, determinism: every cell's two captures are identical (44
`*_determinism_ae=0`).

Assertion 2, byte identity against the baseline (AE):

| Kind | glass | backdrop | film |
| --- | ---: | ---: | ---: |
| white | 0 | 0 | 0 |
| fine | 0 | 0 | 0 |
| lightness | 0 | 0 | 0 |

No node, with `blur { noise 0.05 }` inheriting under `backdrop-blur`: 0.

Assertion 3, `scale=1` written against omitted: white 0, fine 0, lightness 0.

Assertion 4, independence: two fine layers at 0.2 over one, grain sd ratio
1.4159 (expected 1.4142 ± 5 %).

Assertion 5, normalisation by scale. Grain sd, the weakest and strongest
position class relative to the aggregate, and the low-frequency ratio:

| Kind | Scale | sd | min class | max class | LF ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| white | 1 | 0.08662 | — | — | 0.2518 |
| white | 2 | 0.08730 | 0.9997 | 1.0003 | 0.5142 |
| white | 4 | 0.08675 | 0.9972 | 1.0035 | 0.7026 |
| white | 8 | 0.08762 | 0.9921 | 1.0113 | 0.9508 |
| fine | 1 | 0.08665 | — | — | 0.1250 |
| fine | 2 | 0.08696 | 0.9998 | 1.0002 | 0.2848 |
| fine | 4 | 0.08678 | 0.9952 | 1.0057 | 0.5433 |
| fine | 8 | 0.08695 | 0.9828 | 1.0170 | 0.9355 |

Every sd is within 1.2 % of its scale-1 value (limit 10 %), every position
class within 1.7 % (limit 10 %), and the low-frequency ratio rises at every
step.

Assertion 6, slots: fine 0.3 alone in slot k after k zero-amount nodes.

| Slot | sd | sd of the difference from slot 0 |
| ---: | ---: | ---: |
| 0 | 0.08665 | — |
| 1 | 0.08657 | 0.1223 |
| 2 | 0.08682 | 0.1227 |
| 3 | 0.08681 | 0.1228 |

Each slot matches slot 0's sd within 0.2 %, and the differences are about
√2 times one layer's, as independent patterns give.

## Contact sheet

![Noise layers contact sheet](2026-10-06-noise-layers-sheet.png)

`2026-10-06-noise-layers-sheet.png`: 200 × 200 px crops at 1:1 from each
face, shown at 2× without smoothing, rows decoded on the sheet, with the
controls (today's single layer at scale 1) bordered. It is regenerated from
the B-spline run: `scripts/noise-layers-sheet.py grid
<run dir> e5edd631 <out>`. The Hermite version is in git history at
`a1d0c330`.

![Scaled grain before and after the B-spline lattice](2026-10-07-noise-layers-lattice-sheet.png)

`2026-10-07-noise-layers-lattice-sheet.png`: the scale 2/4/8 white and fine
tiles of the Hermite run (`noise-layers-full2-1791341162`, bordered as the
control) above the same tiles of the B-spline run, ids A2–D8;
`scripts/noise-layers-sheet.py lattice <before dir> 558bfa02 <after dir>
e5edd631 <out>`. The run's own
downscaled montage is `contact-sheet.png` in the run directory. In it
per-pixel grain averages away, so it is not the sheet to judge.

## Cost

### B-spline lattice (`e5edd631`)

Full run `noise-layers-bspline-cost-full-1791428043`, the same cases and
cadence as the Hermite run below:

| Case | Material draw median (ms) | Grain::render median (ms) | ns/px over `none` | Hermite draw median (ms) | Hashes per fragment |
| --- | ---: | ---: | ---: | ---: | ---: |
| none | 0.2632 (80) | — | — | 0.2294 | 0 |
| one fine, scale 1 | 0.3400 (80) | — | 0.263 | 0.3011 | 9 |
| four fine, scale 1 | 0.5734 (80) | — | 1.063 | 0.5043 | 36 |
| four fine, scale 8 | 1.1868 (80) | — | 3.165 | 0.6533 | 144 |
| backdrop fine, scale 1 | 0.2703 (80) | 0.0885 (40) | 0.025 | 0.2345 | 9 per backdrop texel |
| backdrop fine, scale 8 | 0.2678 (80) | 0.0938 (40) | 0.016 | 0.2324 | 36 per backdrop texel |

Hash counts are spec §4's at `e5edd631`: 9 per fine layer at scale 1, 36
above it (one 6×6 block).

Every case without a scaled material layer (all but four fine, scale 8)
runs 13–15 % above the Hermite run, `none` included. The grain
pass (`Grain::render`, the effect program, a far smaller program that
inlines the same scaled path four times) moved only 0.5 % at scale 1, so
the shift is not the host. Same-session A/B runs settle
it (`NOISE_LAYERS_COST_AB`: this tree's `niri-tracy` against a kept one,
alternating a, b, a, b; material draw medians in ms):

| Run | a | `none` a / b | one fine, scale 1, a / b |
| --- | --- | ---: | ---: |
| `noise-layers-ab-full-1791429717` | `e5edd631` | 0.2627, 0.2632 / 0.2294, 0.2294 | 0.3400, 0.3400 / 0.3011, 0.3011 |
| `noise-layers-ab-hermite-rebuild-1791431057` | `558bfa02`'s `noise.frag` built today | 0.2294 ×2 / 0.2294 ×2 | 0.3011, 0.3011 / 0.3021, 0.3021 |
| `noise-layers-ab-stream-1791430389` | `e5edd631`, scaled path without arrays | 0.2652, 0.2662 / 0.2294 ×2 | 0.3400, 0.3410 / 0.3016, 0.3011 |
| `noise-layers-ab-loop-1791432546` | that, glass and film layers in a slot loop | 0.2488 ×2 / 0.2294 ×2 | 0.3594, 0.3604 / 0.3021 ×2 |

b is always `558bfa02`'s kept `niri-tracy`. **The B-spline `noise.frag` costs
every material draw: 0.034 ms (15 %) with noise off, 0.039 ms (13 %) with
one fine layer.** The
rebuild row rules out build drift. Streaming the lattice through a few
vec4s instead of `g[16]`/`h[36]`/`r[24]` changes nothing, so it is not those
arrays. Looping the layers (two inlined copies of the scaled path instead
of eight) cuts the idle cost by about 40 % (to 0.019 ms) but makes an
active layer dearer, so neither
is kept; the shader stays at `e5edd631`. The likeliest cause is the larger
scaled path inlined eight times into the material program weighing on its
register allocation; the register counts below confirm it. Recovering the
idle cost is material-03fdf3, which carries both variants as patches and
lands the row accumulator below. Settled glass renders no frames, so the
cost lands on damaged frames only.

With that, four fine layers at scale 8 add 0.923 ms over `none` against
Hermite's 0.424 ms; the ratio 2.18 is close to the hash ratio 144/64 = 2.25,
since both runs' `none` carries its own build's idle cost. The draws are
bimodal as before: split at 0.6 × the case median, four fine at scale 8
has 16 early draws at 0.140 ms and 68 later at 1.188 ms.

### Row accumulator (`127bce2d`, material-03fdf3)

NVIDIA's shader disk cache holds the compiled SASS, and `nvdisasm` reads its
register count (the `regs.sh` attachment on material-03fdf3). On the
material program at the branch head: Hermite 96 registers, the B-spline
`noise.frag` of `e5edd631` 128, the streamed and slot-loop variants 96. On
`e5edd631`'s own source the counts (Hermite 64, B-spline 96) order the same
way as the A/B timings above, with no spills anywhere: registers set most of
the idle cost, code size the rest.

`127bce2d` folds each of the six hash rows into one `vec4` accumulator with
per-row B-spline weights, one code path for white and fine: 72 registers.
Its pixels differ from the current B-spline by at most one code (4 to 67 of
5.2 M bytes per case over eight scaled cases). The trade is that a scaled
white layer hashes 36 points instead of 16. A second candidate, a rolling
window shared by white and fine ("unified", bit-identical to `e5edd631`),
compiles to 96 registers.

Same-session A/B `noise-idle-ab-full-1792149757` (2026-10-16, TTY with the
desktop stopped, clean capture verdict): a is `127bce2d`; b1 the current
B-spline, b2 Hermite and b3 unified, each a `niri-tracy` built at `d89a8988`'s
tree with that `noise.frag`. Material draw medians in ms, two rounds each:

| Case | a (row accumulator) | b1 (B-spline) | b2 (Hermite) | b3 (unified) |
| --- | ---: | ---: | ---: | ---: |
| none | 0.2867, 0.2867 | 0.3236, 0.3246 | 0.2908, 0.2908 | 0.2918, 0.2918 |
| one fine, scale 1 | 0.3589, 0.3594 | 0.3988, 0.3983 | 0.3676, 0.3676 | 0.3676, 0.3676 |
| four fine, scale 8 | 1.1448, 1.1448 | 1.2513, 1.2513 | 0.7342, 0.7347 | 1.3332, 1.3332 |

**The row accumulator removes the B-spline's idle cost and then some:**
0.037 ms (11.5 %) below the current B-spline with noise off, 0.004 ms below
Hermite, and cheaper than the B-spline with one fine layer (0.040 ms) and with
four scaled layers (0.107 ms). Hermite stays cheapest at scale 8, because it
hashes 16 points there against the B-spline's 36. `127bce2d` lands.

Absolute medians run higher than the 2026-10-07 runs (Hermite `none` 0.2908
against 0.2294), so only the same-session columns compare. All medians sit in
the later mode, at the P8 idle clock (the clock attribution experiment,
material-f386ae).
Two earlier attempts the same night were refused by the settle gate when the
GPU's idle floor moved between about 10.4 and 17 W between sub-runs.

### Hermite lattice (`558bfa02`)

Full run `noise-layers-cost-full2-1791342346`: 20 wallpaper replacements per
case at 1 Hz, each damaging the backdrop twice, so 40 grain passes and about
80 material draws per case. The window face is 291,840 px; the glass also
covers the slab band outside it, so the per-pixel figures are upper bounds.

| Case | Material draw median (ms) | Grain::render median (ms) | ns/px over `none` | Hashes per fragment |
| --- | ---: | ---: | ---: | ---: |
| none | 0.2294 (80) | — | — | 0 |
| one fine, scale 1 | 0.3011 (80) | — | 0.246 | 9 |
| four fine, scale 1 | 0.5043 (80) | — | 0.942 | 36 |
| four fine, scale 8 | 0.6533 (80) | — | 1.453 | 64 |
| backdrop fine, scale 1 | 0.2345 (80) | 0.0881 (40) | 0.018 | 9 per backdrop texel |
| backdrop fine, scale 8 | 0.2324 (80) | 0.0911 (40) | 0.011 | 16 per backdrop texel |

Hash counts are those of the Hermite spec: 9 per fine layer at scale 1, 16 above it.

The material draw times (all 83–84 samples in each trace) are bimodal in
every case. The first 11–16 draws
take about 0.03 ms and the rest about 0.23 ms, a constant ratio near 8
across cases:

| Case | first draws: n, median (ms) | later draws: n, median (ms) |
| --- | ---: | ---: |
| none | 11, 0.0297 | 72, 0.2299 |
| one fine, scale 1 | 16, 0.0389 | 68, 0.3021 |
| four fine, scale 1 | 16, 0.0614 | 68, 0.5059 |
| four fine, scale 8 | 16, 0.0788 | 68, 0.6533 |

The pilot's three steps per case saw only the first mode (none 0.030, four
fine at scale 8 0.079 ms), so its absolute figures are not comparable with
the full run's; the ratio between cases agrees (2.6 against 2.8). The
mechanism is the GPU clock: the early draws run in the start-up P0 burst and
the later ones at the P8 idle clock (the clock attribution experiment below).
The noise placement run's damage-case material draw (0.210 ms,
`2026-10-05-noise-placement-evidence.md`) sits in the later mode.

### Clock attribution experiment (material-f386ae)

Preparation adds opt-in `CAPTURE_GPU_CLOCKS=1` sampling to the shared Tracy
capture helper. It launches `nvidia-smi` before Tracy, queries UTC timestamp,
graphics and memory clocks, pstate and power every 100 ms, and reaps the
sampler when capture finishes or aborts. Sampling is optional because polling
may affect power state; existing unsampled captures remain available.

`noise-layers-cost.sh none` runs one case. With sampling enabled it subscribes
to the private compositor's event stream, consumes the initial config state,
then brackets the first requested config reload until its `ConfigLoaded`
event. This matters because the action reply precedes the asynchronous reload.
The corresponding CPU zone supplies a trace-time anchor. The report keeps a
wall-time interval for the first material draw and every subsequent draw,
rather than assuming Tracy's program-relative origin is a wall clock.

Each case retains `<case>.gpu-clocks.csv`, sampler stderr in
`<case>.gpu-clocks.log`, `<case>.clock-anchor.txt`, and
`<case>.draw-clocks.csv` beside its Tracy and exported CPU/GPU files. Joined
rows show the last sampled state preceding the draw's earliest possible wall
time, with the sample timestamp and both draw-time bounds. They are nearby
observations, not proof of instantaneous clocks. The report refuses an anchor
wider than 100 ms, missing draw coverage, or sampling gaps above 250 ms.
The sampler runs on this host's single GPU; interleaved multi-GPU rows fail
the strictly increasing timestamp check.

The offline one-case rehearsal reached its report and checksum verdict with
host/build operations substituted. Tests exercise actual capture cleanup on
normal completion, trace failure, sampler failure and TERM, plus delayed reload
completion after the initial event burst. A single real NVIDIA query confirmed
the timestamp and units, including `TZ=UTC`; it was not a cost measurement.

**Hardware verdict: the split is the GPU clock.** Three runs of `none` on
2026-10-16, TTY with the desktop stopped, all with clean capture verdicts.
Draw counts are those inside the stimulus window (from the first reload
marker), as `metrics.txt` reports them:

| Run | Steps | Sampling | Draws | Median (ms) |
| --- | ---: | --- | ---: | ---: |
| `gpu-clocks-pilot-1792148904` | 3 | on | 12 | 0.0410 |
| `gpu-clocks-full-1792149496` | 20 | on | 80 | 0.3246 |
| `gpu-clocks-control-1792149627` | 20 | off | 80 | 0.3246 |

The sampled full run's joined rows (anchor width 4.5 ms, largest sample gap
104 ms) put each in-window draw in one clock state:

| State (graphics / memory MHz) | Draws | Median (ms) | Graphics cycles per draw | Seconds after the window opens |
| --- | ---: | ---: | ---: | --- |
| P0, 1815 / 7001 | 12 | 0.0404 | ~73k | 0.00–2.16 |
| P5, 345 / 810 | 2 | 0.2662 | ~92k | 2.98–3.06 |
| P8, 270 / 405 | 2 | 0.2698 | ~73k | 3.11–3.15 |
| P8, 225 / 405 | 2 | 0.3251 | ~73k | 3.98–4.07 |
| P8, 210 / 405 | 62 | 0.3256 | ~68k | from 4.11 |

The fast draws are exactly the ones that ran while the GPU was still in P0.
P0 was already present when sampling began, about a second before the first
draw (2.2 s before the window opens; the cooldown before the sub-run had left
the GPU in P8). It left P0 about 2.5 s into the window, read P5 for one
0.5 s reading, and P8 from about 3.1 s, although the cadence was unchanged:
the driver's hold after the compositor's start-up expiring fits, but is not
measured. Every later draw ran at the P8 idle clock. The ratio, 0.3256 /
0.0404 = 8.1, is the constant ratio near 8 seen in every earlier case.

The draw cost tracks the graphics clock alone: about 70k graphics cycles per
draw from P0 to P8, across a 17-fold range of memory clock. The driver
refreshes its clock and power readings only every 0.5 s, although they are
polled every 100 ms, so a draw up to 0.5 s before a reported change can carry
the previous state. The two draws tagged P5 cost what the 270 MHz draws cost,
and the two tagged 225 MHz cost the 210 MHz rate: their tags are that
resolution, not evidence about the memory clock.

The unsampled control's steady P8 median agrees with the sampled run's to
one step of the GPU timer (durations are multiples of 1024 ns, about 0.3 %
here), and both runs have 12 fast draws in the window. So 10 Hz polling of
`nvidia-smi` does not hold the GPU out of P8. One pair of runs cannot rule out
an effect on the descent itself: the control's six descent draws spread
0.186–0.296 ms against the sampled run's 0.264–0.328. The three-step pilot
ends inside the P0 burst, which is why pilots see only the first mode.

Recommendation: **report the later mode, and select it by clock state rather
than by count.** The later mode is the shader's cost at the clock an idle
desktop actually runs at when a damage arrives about once a second, which is
the condition these captures model. It is not a cost at full clock. Locking
clocks (`nvidia-smi -lgc`, root) is rejected: it would measure a state the
desktop never reaches at this cadence and needs a privileged host change in
every capture. Dropping the first N draws by count is fragile, because the
burst length depends on start-up timing (11 to 16 draws in the earlier
traces, 12 in the window here). Cost reports should exclude draws whose
joined row is not P8 (allowing for the 0.5 s reading lag above), and pilots should not be compared
with full runs in absolute terms. Medians of earlier full runs
(about 80 draws, at most 16 early ones) are already the later mode and stand.

**The report now selects them (material-d2e552).** `noise-layers-cost.sh`
samples clocks by default; `CAPTURE_GPU_CLOCKS=0` keeps the unsampled
condition earlier references were measured in. A draw counts as idle-clock when
every reading from its joined row through the first reading 0.5 s past its
latest wall time is P8 at the trace's lowest P8 graphics clock. That rule keeps
no draw from the descent and gives up the two draws whose 225 MHz tags were
lag. `<case>.draw-clocks.csv` marks each draw `in_window` and `idle_clock`.
`metrics.txt` adds, per case, `idle_graphics_mhz`, the in-window idle-clock
draws' `draw_idle_clock_count` and `draw_idle_clock_median_ms`, and their
`idle_clock_power_median_w`, which tells the 10.4 W floor from the 17 W one
below. A sampled full run refuses a case with fewer idle-clock draws than
damage steps, and its deltas over `none` add an `_idle_clock_` set that
refuses cases idling at different graphics clocks. `draw_median_ms` still
covers every in-window draw. Rerun offline on the sampled full run above, the
report selects 62 draws, median 0.3256 ms at 10.84 W; on the pilot, none.

A separate observation from the same session: after nested runs end, the
idle P8 floor (210 / 405 MHz, no GPU clients) settled at either about 10.4 W
or about 17 W, and could flip between runs. The clocks do not distinguish the
two states. capture-meta's settle gate refused two noise A/B attempts on such
a flip (material-03fdf3). Draw medians within one capture are unaffected so
far, but long A/B captures are exposed to it.

Settled glass renders no frames, so these are costs per damaged frame. A
superseded full run (`noise-layers-cost-full-1791340265`, same shader
uncommitted) measured within 1 % of these.

## Limitations

- One run per configuration on one host; no spread is measured, except the
  A/B rows: two rounds per binary each, within 0.001 ms.
- The identity fix pins the baseline's arithmetic for one layer only. A
  stacked config has no baseline to match, and its lightness layers can
  still differ by a code from an algebraically equal single-layer form.
- The cost cases measure fine grain only; white is one hash per layer at
  scale 1 and sixteen above it, and was not timed.
