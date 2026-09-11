# Aurora and iridescence: NVIDIA hardware evidence

**Status:** hardware pixel, cadence, and installed-session integration checks passed.
GPU timing and board power are recorded; relative overhead remains unresolved.
Task `material-300b87` follows the earlier llvmpipe evidence with the installed
binary on an RTX 3070. No production renderer changes are part of this task.

## Environment and identity

| Input | Value |
| --- | --- |
| Installed CLI and running compositor | niri 26.04 (`7526af1d`) |
| Installed binary SHA-256 | `38b10fbb412c4e07c1f14e12ba59c77697fd15489bef6fc6580bef35e32b56a8` |
| Tracy binary SHA-256 | `ed2116ce35afbbb73be2c1b0d23a3b59fa007db8135d9b9ec21a6b3df54e102d` |
| Source checkout | `2ad7fc3c`; no rendering/config/IPC/Cargo diff from `7526af1d` |
| GPU and driver | NVIDIA RTX 3070, 610.57.04, `/dev/dri/renderD128` |
| CPU | AMD Ryzen Threadripper 1950X, 16 cores / 32 threads |
| Kernel | Linux 7.2.2-arch1-1 |
| Controlled host | Weston 15.0.1, headless GL, 1280×720 |
| Test surface | Transparent kitty, 456×640, flat warm backdrop |

Both Weston and nested niri explicitly reported `NVIDIA GeForce RTX 3070`.
The fixture rejects a software renderer. Captures use a copy of the installed
binary; timing uses a matching-source release build with Tracy instrumentation.
The source and binary identities are separate facts, both retained in the run.

## Pixels and focus states

| Check | Focused | Unfocused |
| --- | ---: | ---: |
| Plain vs iridescence 0, AE | 0 | 0 |
| Plain vs aurora 0, AE | 0 | 0 |
| Pinned / motion-off, AE after 2.5 s | 0 / 0 | 0 / 0 |
| Moving / reduced within-bucket pair | Identical | Identical |
| Moving / reduced later bucket | Changed | Changed |
| Iridescence chamfer Oklab a/b RMSE, Q16 | 622.168 | 622.168 |
| Iridescence face Oklab a/b RMSE, Q16 | 98.9579 | 98.9579 |
| Aurora face RMSE, Q16 | 5918.03 | 5918.03 |
| Plain / lit face mean | 0.464767 / 0.540244 | 0.464767 / 0.540244 |

The shader's edge weighting is measurable: iridescence changes chamfer chroma
about 6.3 times as much as face chroma. Aurora raises face brightness and has
soft green/violet structure in the captured preset. The quiet response disables
focus/accent animation and geometry motion, so these comparisons isolate the
optics. Window IPC records verify which state was focused for every capture.

## Material-pass timing

Each cell is a median of 14 warmed `MaterialRenderElement::draw` GPU events.
The samples are the first 14 events whose program timestamps lie in 20–28 s;
three runs rotate case order. The overall statistic is the median of those
three medians, following the original optic smoke's convention.

| Case | Run 1, µs | Run 2, µs | Run 3, µs | Median, µs |
| --- | ---: | ---: | ---: | ---: |
| Plain | 9.728 | 12.288 | 7.168 | 9.728 |
| Iridescence 0 | 13.312 | 11.264 | 13.312 | 13.312 |
| Iridescence 0.8 | 5.120 | 6.144 | 13.312 | 6.144 |
| Aurora 0 | 9.216 | 12.288 | 14.848 | 12.288 |
| Aurora 0.5 at 4 Hz | 9.216 | 9.728 | 12.800 | 9.728 |

**Relative optic overhead is not resolved by these runs.** GPU clocks and memory
states varied, and even the zero-strength controls differed substantially from
plain glass. The raw median deltas (iridescence −36.8%, aurora 0%) therefore do
not establish a speedup or zero cost. The observed draw medians were 5.120–14.848 µs
for this small, roughness-zero scene. They are not a whole-frame or larger-window
budget. The original plan requires cost recording and pixel/redraw gates; it
sets no numeric GPU or power acceptance threshold.

## Redraw and power method

Idle cases are each sampled three times in rotated order. The probe is unfocused
and signal-free. Counts cover the final 20 s of 30 s Tracy captures, with the
one-second idle-inhibit heartbeat checked for capture coverage. Board power and
clocks are sampled at 1 Hz with `nvidia-smi`; each run reports its final-20-second
median. The desktop remains active on the same GPU, so these are whole-board
readings, not process power. Broader isolation and power budgeting remain with
`material-265eb0`; Prism cost reporting remains `prism-d54be4`.

## Idle redraws and board power

All three rounds gave the same redraw count. Plain glass, static iridescence,
pinned aurora, and motion-off remained idle; 4 Hz aurora drew 80 times in 20 s,
and reduced motion drew 40 times.

| Case | Redraws per 20 s, each round | Run 1, W | Run 2, W | Run 3, W | Median, W | Delta vs plain, W |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plain | 0 | 20.610 | 19.850 | 21.275 | 20.610 | +0.000 |
| Iridescence | 0 | 20.330 | 15.615 | 21.270 | 20.330 | -0.280 |
| Pinned aurora | 0 | 20.295 | 22.290 | 21.390 | 21.390 | +0.780 |
| Aurora 4 Hz | 80 | 13.720 | 21.060 | 23.975 | 21.060 | +0.450 |
| Reduced motion | 40 | 21.850 | 20.950 | 21.700 | 21.700 | +1.090 |
| Motion off | 0 | 21.885 | 28.960 | 21.625 | 21.885 | +1.275 |

**These power deltas are not attributable optic costs.** Individual run medians
span 13.720–28.960 W across the cases, much more than their aggregate differences.
The active desktop shares the GPU and clock states vary. Isolation and broader
power budgeting remain in `material-265eb0`; Prism reporting is `prism-d54be4`.

## Running compositor integration

The installed `7526af1d` session captured Aurora and Rainbow in both focus states
on DP-1 at 3440×1440, 59.999 Hz, scale 1, VRR disabled. Temporary, renamed copies
of the shipped presets applied only to fixture windows in a dedicated workspace.
The captures show Aurora's face field and Rainbow's chamfer tint on the real
wallpaper. They are qualitative integration evidence; desktop notifications
make whole-output pixel differences unsuitable here. They do not substitute for
an operator's physical-panel aesthetic verdict.

The fixture waited for successful config-load events, closed its windows,
restored the original config and focused window/workspace, and verified that
the original config file hash was unchanged. A subsequent `niri validate` passed
and no fixture windows remained. No Prism profile files were changed.

## Reproduction and retained evidence

The sibling niri-experiments branch `results/aurora-iridescence`, commit
`e6522d0`, holds the fixture, its Just front door, independent analyzer, and
experiment record at `docs/results/2026-09-11-aurora-iridescence-hardware.md`.
Its adjacent `.sha256` manifest covers all 367 retained artifacts; every hash
was verified after collection. Manifest SHA-256:
`54227878ead8873fbb19ca8c2fd8497909f1eae86a96e4b059db77b3ebc1c11b`. The native shared helper is
`docs/materials/scripts/glass-optic-smoke-lib.sh`. Run directories are retained
under `/mnt/ssd3/niri-material/material-300b87`:

- `visual-20260911T042915`: installed-binary captures, configs, geometry, renderer
  and machine metadata, and independent pixel analysis.
- `trace-20260911T043433`: all 33 traces, CPU/GPU exports, power/clock samples,
  binaries, and copies of the fixture and shared helper used in the run.
- `installed-20260911T045655`: four running-session captures, output/window IPC
  records, temporary configs, and the restoration record.

The native helper was corrected during this task: the old `count_last20` treated
an empty or truncated CSV as a successful zero-redraw result. Regression tests
reproduce that false pass and now reject empty, truncated, and stalled traces.
The helper also waits for its capture child during cleanup. Current captures
must establish coverage before any idle conclusion is accepted.
