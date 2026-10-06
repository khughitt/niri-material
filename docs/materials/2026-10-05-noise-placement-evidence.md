# Noise placement: verification evidence

**Result:** the nine in-process pixel/damage tests and affected Rust suites pass,
and the nested-Weston smoke passes all seven assertions against the baseline
binary (quiet TTY run, 2026-10-05); Tracy costs are measured for all six cases.

## Pinned revisions and artifacts

- Renderer and capture-integrity corrections through `4609c7bd`; implementation
  review round 2 accepted all four corrections with no remaining findings.
  Original implementation: config `b956182f`, shader/schema `fa8a447e`, grain
  pass `cd176b2a`, pixel/damage tests `8db25b43`.
- Baseline `b261ad1a`, release binary SHA-256 `932dd408244012476bad6fa71141bdf53e73c26c1f2d73a37a870524865cc20e`.
- Prepared binary snapshots: `$NIRI_NOISE_ARTIFACTS/noise-binaries-8db25b43/`.
  Corrected release SHA-256 `b380071c1f0b4ea2a413062b9babce896ec47e22cee589ecc6bbd3af610a0a14`;
  corrected Tracy SHA-256 `ee669cf4934bf6fd5cd224f5de646f9b39131c4deb5546fee0abc10c4c1b63c3`. Both builds passed.
  The quiet runs rebuilt both from `96571593` (no source, config or Cargo
  change since `4609c7bd`; the tree was dirty only in scripts, spec and task
  records) and record `edfa03ad…` (niri) and `0c189406…` (niri-tracy): niri
  embeds its git revision, so a rebuild at a new commit changes the hash.
- Offline simulation: `$NIRI_NOISE_ARTIFACTS/noise-simulation-20261005-112606/`;
  `sheet-white.png`, `sheet-fine.png`, the cell images and metrics.
- Refused readiness run:
  `$NIRI_NOISE_ARTIFACTS/noise-readiness-20261005-111743/capture.json`.

`NIRI_NOISE_ARTIFACTS` is the host-local directory holding these runs; set it
before executing the commands below. Artifact paths deliberately use that
variable rather than a machine's storage mount.

## In-process evidence

`just test-one -p niri noise_site`: 9/9 passed. All use the real surfaceless
GLES renderer and a frozen clock, with two transparent windows and a mapped
background layer. The tests cover omitted=glass, zero neutrality at every
site, film/glass within one code on the face for all three kinds,
backdrop/glass within two codes mean error, softening under three blur passes,
backdrop-to-glass removal, transparent texels, the plain window's blurred
background effect, and publication to the unchanged glass window.

The identity fixture rejects a change from 0.30 to 0.31 (56,538 changed bytes,
maximum delta 2 in the white case). Removing grain-change publication makes
both the invalidation table and real unchanged-window test fail. The native
GL seed orientation holds; no vertical seed correction was needed.

The plain-window coverage fixture uses one blur pass and retains the 0.5-code
presence threshold. At three passes its fine grain spread was only 0.026012
codes. The separate softening test still uses three passes.

Four additional low-level GPU regressions now cover incomplete framebuffer
failure, failed draws, sharp fallback held until invalidation, and context-owned
program reuse after buffer references drop / a shared renderer is replaced.
A fifth assertion proves a failed pass leaves no GL error for its fallback frame.

## Offline look model

A 32-pixel pilot exercised every cell and wrote both contact sheets before the
512-pixel run. The full run used amount 0.3, seed 11 and the synthesized
backdrop. Remaining backdrop grain relative to glass at p=0, L=0:

| Grain | p=0 | p=1 | p=2 | p=3 | p=3, L=2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| white | 1.0000 | 0.3171 | 0.1508 | 0.0829 | 0.0770 |
| fine | 1.0000 | 0.1902 | 0.0722 | 0.0456 | 0.0441 |

These rerun values agree with the rounded specification table (white roughly
32/15/8%, fine roughly 19/7/5% after one/two/three model passes).
**Owner's look: pending.**

## Earlier refused readiness run

`tools/capture-meta show` reports the following for the refused readiness run
(excerpt; the complete record is retained in `capture.json`):

```text
baseline
  seconds 20
  cpu_busy_pct 7.4
  load1 3.02
  gpu_util_pct 12.0
  gpu_pstates ['P8']
preflight refused
  load1 3.02 exceeds 2.0
  gpu_util_pct 12.0 exceeds 5.0
  compute clients present: browser GPU process
hold end  restore complete (preflight)  scan not-run
```

The desktop and its applications were left running. The preflight's temporary
background-service holds were restored. No capture threshold was changed.

## Nested smoke

**PASS**, full matrix: `$NIRI_NOISE_ARTIFACTS/noise-site-20261005-225906/`
(17.9 min, headless lane, quiet preflight, every sub-run settled). The pilot
passed first at `noise-site-pilot-20261005-224851/`.

```text
run noise-site-20261005-225906  task material-cf32e5  fixture glass-noise-site-smoke.sh  lane headless
started 2026-10-05T22:59:07-04:00
  gpu NVIDIA GeForce RTX 3070 615.71.09 /dev/dri/renderD128   weston 15.0.1
baseline  cpu_busy_pct 1.2  load1 0.23  gpu_util_pct 0.0  gpu_pstates ['P8']  clients none
preflight quiet
provenance
  source 96571593 material-cf32e5 dirty (smoke script, spec and task records only)
  binary niri          edfa03ad76758b922d1a6538c3fcfa9694e5f9ca692023ca6adbc5ed3c5fc26f
  binary niri-baseline 932dd408244012476bad6fa71141bdf53e73c26c1f2d73a37a870524865cc20e (b261ad1a)
hold end  restore complete (release)  scan clean
```

| Assertion | Measured | Result |
| --- | --- | --- |
| 1. Every cell deterministic across two captures | AE 0 in every cell | pass |
| 2. Omitted site = explicit glass, every kind | AE 0 (white, fine, lightness) | pass |
| 3. Omitted site byte-identical to baseline binary | AE 0 (white, fine, lightness) | pass |
| 4. Sharp backdrop/glass face MAE within 2/255 | 0.0075 quantum (Q16) | pass |
| 5. Backdrop softens; glass stays within 5 % | see below | pass |
| 6. Film/glass face within 1/255, every kind | AE 0 at one-code fuzz | pass |
| 7. Ring band: film ≈ 1, glass from the transfer curve | film 0.9942; glass 0.4554 vs 0.4516 at `e_b` 0.924 | pass |

Grain `sd` (normalized, fine at amount 0.3), with the remaining fraction of the
glass site's grain beside §7.1's model prediction for fine grain:

| Cell | glass | backdrop | film | backdrop / glass | model (fine) |
| --- | ---: | ---: | ---: | ---: | ---: |
| blur off | 0.0867 | 0.0867 | 0.0867 | 1.000 | 1.000 |
| 1 pass | 0.0867 | 0.00357 | 0.0867 | 0.041 | 0.190 |
| 3 passes | 0.0867 | 0 | 0.0867 | 0 | 0.046 |
| roughness 0, sharp | 0.0849 | 0.0846 | — | 0.996 | 1.000 |
| roughness 0.5, sharp | 0.0849 | 0 | — | 0 | — |
| roughness 1, sharp | 0.0827 | 0 | — | 0 | — |
| roughness 0 / 0.5 / 1, blurred | 0.0849 / 0.0849 / 0.0827 | 0 / 0 / 0.000144 | — | ≤ 0.002 | — |

The real chain removes backdrop grain far faster than the model predicted:
one Kawase pass leaves about 4 % (under one code), three passes or any
roughness pyramid level leave none above the 8-bit floor. The blurred
roughness-1 cell's 0.000144 (grey `sd`, 0.037 codes) is not grain texture: only
blue differs, the backdrop cell one code higher on 44,066 of 80,000 face
pixels (mean +0.55 codes, blue-only `sd` 0.50 codes, red and green
identical), one-signed and near-DC (63 % of the grey `sd` survives an 8×
downsample, against 4.5 % of real grain). Its cause is not established:
the positive sign fits clipping of the grain at 0 on dark blue texels (24 %
of the plasma region's blue texels lie within the grain's ±38-code span of 0),
rounding in the 8-bit blur chain is not excluded, and the sharp pyramid at
roughness 1 shows exactly 0. (The 2026-10-05 runs' `grain-*.png` images are the
zero cell minus the site cell; `sd` is unaffected by the sign, and
material-0966ba since made `signed_diff` the site cell minus the zero cell.) Strict
"falls monotonically" cannot hold once grain reaches zero (the full run has
three 0 → 0 steps), so assertion 5 now reads "monotone down to the 8-bit
floor" (spec §7.2, half a code, the in-process presence threshold); in blue
alone this residue sits at that floor, which the grey metric does not show. In practice `site="backdrop"` is visible grain
only with `backdrop-blur false` and roughness 0, the sharp case §4 targets.
**Owner's look: pending.**

Fixture corrections made during the quiet run, none touching a renderer
bound: the film comparison stripped ImageMagick 7's normalized suffix (`0
(0)`) as the lib's `compare_metric` does; the ring band crop now follows the
face's `offset-x` (the ring's measured right peak sits at `PX + PW - 6.5`, the
earlier crop sampled the glow's falloff at level 0.6).

## Tracy costs

**Measured**, full run: `$NIRI_NOISE_ARTIFACTS/noise-cost-20261005-233726/`
(5.6 min, headless lane, quiet preflight: load1 0.24, GPU 0 % at P8, no
clients; every stimulus complete: damage 20/20, drag 50/50; no wallpaper
process left). Pilot: `noise-cost-pilot-20261005-233135/`. Tracy binary
`0c189406cd2f238bd7981a9a59743236205b47df1d7583620833ca7c54b17b66`, source
`96571593` plus the script corrections below. Renderer (the nested parent's
EGL device, from `weston.log`; niri's log at this level prints none):
`NVIDIA GeForce RTX 3070/PCIe/SSE2`, `OpenGL ES 3.2 NVIDIA 615.71.09`.

GPU span medians (ms) and sample counts, sharp and blurred roughness windows
open at once:

| Case | Grain::render | Blur::render | Prefilter::downsample | Material draw | per-call sum | mean total per stimulus | per damage / change | derived 60 Hz (ms/s) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| damage, backdrop | 0.0876 (40) | 0.456 (40) | 0.267 (80) | 0.210 (162) | 1.021 | 3.656 | 1.83 | 110 |
| damage, glass | — (0) | 0.452 (40) | 0.267 (80) | 0.272 (162) | 0.991 | 3.756 | 1.88 | 113 |
| drag, backdrop | 0.0364 (100) | 0.145 (100) | 0.0957 (200) | 0.0317 (104) | 0.309 | 1.416 | 1.42 | 85 |
| drag, glass | — (0) | — (0) | — (0) | 0.149 (104) | 0.149 | 0.285 | 0.285 | 17 |

Static cases: no grain, blur or prefilter work in the 20 s window at either
site; backdrop's first grain pass took 0.62 ms. Each wallpaper change damages
the backdrop twice, once at the old layer's removal (about 60 ms after the
reload) and once after the new layer's `Layer::mapped` (about 150 ms), so the
damage counts are twice the stimuli and the per-damage column halves the
per-stimulus total. A drag step is one change but runs the chain twice within
a millisecond of its reload: a grain option change updates both the
`xray.background` and `xray.backdrop` effect buffers (`src/niri.rs`), and the
per-change total covers both.

Reading: on backdrop damage the grain pass adds about 0.09 ms per pass, but
the per-stimulus totals cannot resolve it: backdrop's 3.66 ms is 0.1 ms *below*
glass's 3.76 ms, the opposite of the added work, so the totals vary by more
than the pass costs. A grain option change (the drag) costs `backdrop` a full chain
rerun, grain plus blur plus both pyramids, 1.4 ms per change against 0.29 ms
for `glass`, which only redraws its material. Derived 60 Hz figures are the
per-damage (or per-change) total times 60, labelled derived: an animated
backdrop under either site spends about 110 ms of GPU time per second, and a
continuously dragged backdrop amount slider about 85 ms.

Script corrections during the quiet run: the IPC client is the plain binary,
because `niri-tracy msg` pays Tracy's timer calibration (539 ms against 18
ms per call), which stretched drag steps to 540 ms and damage steps to 1.6 s
until the 30 s trace cut off their last stimuli; `sleep_until` now fails a
step that overruns by more than half its period, so a pilot shows a slow
stimulus.

## Limitations and remaining acceptance

- The simulation's model equates one blur pass and one pyramid level; the real
  shader's Kawase offset and level mix differ.
- One cost run per case on one host, so no spread is measured; the damage
  totals do not resolve the grain pass and no ranking between sites is claimed.
- The renderer string comes from the nested parent's log, not from niri's own.
- The first whole-branch review found four Important issues; scoped review
  round 2 accepted the GL error/lifetime and capture-integrity corrections.
- Corrected `just gate` passed: 409 tooling cases; Rust suites and doctests
  passed (491 niri tests, one existing ignored test, plus config/IPC/doctests).
  Merge, schema vendoring in prism and parent close were capture-blocked;
  the captures above clear that block.
