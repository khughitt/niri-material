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
material draw about 0.034 ms (15 %) with noise off; same-session A/B runs
pin it to `noise.frag` (see Cost).

Owner's second look (B-spline sheets): pending.

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

Every case runs 13–15 % above the Hermite run, `none` included. The grain
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
every material draw about 0.034 ms (15 %), with noise or without.** The
rebuild row rules out build drift. Streaming the lattice through a few
vec4s instead of `g[16]`/`h[36]`/`r[24]` changes nothing, so it is not those
arrays. Looping the layers (two inlined copies of the scaled path instead
of eight) halves the idle cost but makes an active layer dearer, so neither
is kept; the shader stays at `e5edd631`. The likeliest cause is the larger
scaled path inlined eight times into the material program weighing on its
register allocation, but no ISA or register count was read. Recovering the
idle cost is filed as its own task. Settled glass renders no frames, so the
cost lands on damaged frames only.

With that, four fine layers at scale 8 add 0.923 ms over `none` against
Hermite's 0.424 ms; the ratio 2.18 is close to the hash ratio 144/64 = 2.25,
since both runs' `none` carries its own build's idle cost. The draws are
bimodal as before: split at 0.6 × the case median, four fine at scale 8
has 16 early draws at 0.140 ms and 68 later at 1.188 ms.

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
mechanism is not measured: a GPU clock change between the start-up burst
and the 1 Hz cadence fits the constant ratio, but no clock trace was taken.
The noise placement run's damage-case material draw (0.210 ms,
`2026-10-05-noise-placement-evidence.md`) sits in the later mode.

Settled glass renders no frames, so these are costs per damaged frame. A
superseded full run (`noise-layers-cost-full-1791340265`, same shader
uncommitted) measured within 1 % of these.

## Limitations

- One run per configuration on one host; no spread is measured.
- The identity fix pins the baseline's arithmetic for one layer only. A
  stacked config has no baseline to match, and its lightness layers can
  still differ by a code from an algebraically equal single-layer form.
- The cost cases measure fine grain only; white is one hash per layer at
  scale 1 and sixteen above it, and was not timed.
