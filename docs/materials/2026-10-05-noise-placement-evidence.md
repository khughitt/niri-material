# Noise placement: verification evidence

**Result:** the nine in-process pixel/damage tests and affected Rust suites pass.
Nested baseline smoke and Tracy costs have **not run**: quiet preflight refused
on 2026-10-05. This is a preparation record, not acceptance for merge.

## Pinned revisions and artifacts

- Renderer and capture-integrity corrections through `4609c7bd`; implementation
  review round 2 accepted all four corrections with no remaining findings.
  Original implementation: config `b956182f`, shader/schema `fa8a447e`, grain
  pass `cd176b2a`, pixel/damage tests `8db25b43`.
- Baseline `b261ad1a`, release binary SHA-256 `932dd408244012476bad6fa71141bdf53e73c26c1f2d73a37a870524865cc20e`.
- Prepared binary snapshots: `$NIRI_NOISE_ARTIFACTS/noise-binaries-8db25b43/`.
  Corrected release SHA-256 `b380071c1f0b4ea2a413062b9babce896ec47e22cee589ecc6bbd3af610a0a14`;
  corrected Tracy SHA-256 `ee669cf4934bf6fd5cd224f5de646f9b39131c4deb5546fee0abc10c4c1b63c3`. Both builds passed.
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

## Quiet-host readiness

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

## Nested smoke: pending

| Assertion | Status |
| --- | --- |
| Every cell deterministic across two captures | Pending quiet run |
| Omitted site = explicit glass, every kind | In-process pass; nested pending |
| Omitted site byte-identical to baseline binary | Pending quiet run |
| Sharp backdrop/glass face MAE within 2/255 | In-process pass; nested pending |
| Blur/roughness soften backdrop; glass/film spread stays stable | Three-pass in-process pass; matrix pending |
| Film/glass face within 1/255, every kind | In-process pass; nested pending |
| Ring-band film ratio near 1; glass ratio from transfer derivatives | Pending quiet run |

First run the pilot, which still exercises all seven assertions using fine
grain and the blurred roughness endpoints. Then run the full matrix with a
new OUT directory:

```bash
BASE_NIRI="$NIRI_NOISE_ARTIFACTS/noise-binaries-8db25b43/niri-baseline" \
OUT="$NIRI_NOISE_ARTIFACTS/noise-site-pilot-$(date +%Y%m%d-%H%M%S)" \
CAPTURE_TASK=material-cf32e5 NOISE_SITE_PILOT=1 \
bash docs/materials/scripts/glass-noise-site-smoke.sh

BASE_NIRI="$NIRI_NOISE_ARTIFACTS/noise-binaries-8db25b43/niri-baseline" \
OUT="$NIRI_NOISE_ARTIFACTS/noise-site-$(date +%Y%m%d-%H%M%S)" \
CAPTURE_TASK=material-cf32e5 \
bash docs/materials/scripts/glass-noise-site-smoke.sh
```

Append the run's metadata summary, binary hashes, metrics and measured assertion
values here. The ring fixture reports its grain-off encoded band level instead
of assuming that level.

## Tracy costs: pending

The script owns its wallpaper PIDs and traps EXIT, TERM and INT. An offline
regression proves failure status 23 still reaches the shared cleanup and that
its wallpaper process is killed and reaped. A synthetic CSV check proves the
report uses reload-span stimulus boundaries and counts both buffer passes.

```bash
OUT="$NIRI_NOISE_ARTIFACTS/noise-cost-pilot-$(date +%Y%m%d-%H%M%S)" \
CAPTURE_TASK=material-cf32e5 NOISE_COST_PILOT=1 \
bash docs/materials/scripts/noise-placement-cost.sh

OUT="$NIRI_NOISE_ARTIFACTS/noise-cost-$(date +%Y%m%d-%H%M%S)" \
CAPTURE_TASK=material-cf32e5 \
bash docs/materials/scripts/noise-placement-cost.sh
```

Both runs contain static/damage/drag cases at both sites, with simultaneous
sharp and blurred roughness windows in each workload. Distinct sharp/blurred
preparation spans validate both pyramids. Each wallpaper process must stay
alive and publish a layer; each recorded damage interval must show an actual
layer mapping, sharp damage and both preparations. Incomplete stimuli fail.
The pilot reduces
wallpaper changes from 20 to five. Reload spans mark the measured stimulus
window. Dragging uses completed, unique config paths and the supported explicit
reload action, preventing the watcher from duplicating the reload.

Report CPU cascade counts, GPU span medians and sample counts, the sum of
per-call medians, and the mean GPU total per actual change. The latter includes
all measured buffer passes, rather than assuming there is only one. A derived
60 Hz figure is the per-damage total times 60, labelled derived. No comparative
cost verdict is available until both measured pairs exist.

## Limitations and remaining acceptance

- The simulation's model equates one blur pass and one pyramid level; the real
  shader's Kawase offset and level mix differ.
- Baseline byte identity and the full nested matrix are unverified.
- The ring fixture's encoded band level and transfer ratio are unmeasured.
- Tracy costs and the actual nested renderer string remain unmeasured; the
  preflight's host GPU name is not a measured rendering-cost result.
- The first whole-branch review found four Important issues; scoped review
  round 2 accepted the GL error/lifetime and capture-integrity corrections.
- Latest host check still showed 16% GPU utilization, P3 and an active browser
  compute client; the quiet condition remains unmet.
- Corrected `just gate` passed: 409 tooling cases; Rust suites and doctests
  passed (491 niri tests, one existing ignored test, plus config/IPC/doctests).
  Merge, schema vendoring in prism and parent close remain capture-blocked.
