# Jelly flex and ripple: motion sweep evidence

**Status:** native task `material-36e968` completed. All eight parameter cases
passed integrity checks on the installed NVIDIA renderer. No production
renderer or Prism range changes were needed.

The static sweep renders settled windows, where jelly activity is zero. Its
motion companion runs a reproducible column move and measures flex/ripple
while native animation residuals are nonzero.

## Scene and measurement

Installed niri `26.04 (7526af1d)`, binary SHA-256
`38b10fbb412c4e07c1f14e12ba59c77697fd15489bef6fc6580bef35e32b56a8`,
on NVIDIA RTX 3070 / driver 610.57.04. Source checkout `2afd0fb2` has no
rendering/config/IPC/Cargo diff from that installed source. CPU: AMD Ryzen
Threadripper 1950X. Both nested niri and Weston report the NVIDIA renderer.

A 1280×720 headless Weston hosts a transparent 456×640 kitty and a transparent
anchor over the archived diagnostic backdrop. Glass has ior 1.5, thickness 20,
bevel 12, offsets (6,6), and neutral blur, distortion, grain, and response.
Only one jelly parameter changes; the other remains zero. The settled optical
response box is 468×652 at (40,40).

The probe's column moves right, settles for two seconds, then moves left with
a critically damped spring (stiffness 100, epsilon .0001). Two bursts per value
request captures at 0, 25, 50, 75, 100, 150, 200, 300, 400, 600, 800, 1100, and
2200 ms. Monotonic-clock records bracket screenshot IPC requests; they are not
presentation timestamps. Actual pane progress supplies the alignment coordinate.

The companion reuses the niri-experiments parity analyzer's directional bevel
shear for flex and face residual variation for ripple, with its translation
correction and face inset. Each comparison aligns four bursts on their common
10-pixel progress grid. Signal is the mean of two variant-minus-baseline deltas;
the repeat floor is the maximum of their separation and both settings' repeat
ranges. This bound is descriptive, not statistical confidence. Ripple's absolute
clock phase is not pinned, so dynamic pixels are not claimed to be identical.

## Results

All eight cases passed integrity checks: 208 burst captures (16 bursts × 13
frames), eight settled captures, and one wallpaper-only capture. First
IPC-return offsets ranged from 34.274 to 43.255 ms; first measured displacement
was 488–489 pixels. Every burst progressed monotonically to zero and every
final/settled capture was pixel-identical to the neutral settled image.

| Parameter | Value | Change from zero | Repeat floor | Neighbor change | Neighbor floor | Neighbor resolved |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| jelly-flex | 0 | 0.000000 | 0.014120 | — | — | — |
| jelly-flex | 0.004 | 0.091181 | 0.060414 | 0.091181 | 0.060414 | Yes |
| jelly-flex | 0.01 | 0.198983 | 0.039717 | 0.107802 | 0.060414 | Yes |
| jelly-flex | 0.02 | 0.253729 | 0.044295 | 0.054746 | 0.044295 | Yes |
| jelly-ripple | 0 | 0.000000 | 0.000000 | — | — | — |
| jelly-ripple | 0.06 | 0.001403 | 0.000139 | 0.001403 | 0.000139 | Yes |
| jelly-ripple | 0.25 | 0.005581 | 0.000160 | 0.004178 | 0.000160 | Yes |
| jelly-ripple | 0.5 | 0.008583 | 0.000091 | 0.003002 | 0.000251 | Yes |

Every nonzero setting and every neighboring step resolved an increase in its
own metric above the observed repeat floor. Flex and ripple use different
metrics, so their numeric magnitudes cannot be compared to one another. Flex gain per unit
parameter decreases across these sampled intervals; this does not establish
a universal saturation point or justify changing Prism's exposed range.

This measures native column-movement animation in the pinned scene. It does
not measure interactive pointer dragging, resize, other glass parameters or
wallpapers, physical-panel perceptibility, or GPU/power cost. The dynamics
redesign remains `material-6d4de5`; broader power budgeting is `material-265eb0`.

## Reproduction and artifacts

The sibling niri-experiments branch `results/jelly-motion`, commit `f1b0741`, contains
`fixtures/jelly-motion.just`, capture and analysis scripts, tests, and
`docs/results/2026-09-11-jelly-motion-sweep.md` with repeatable commands and the
artifact manifest. Set `MATERIAL_ROOT` to a matching native checkout and
`NIRI_MATERIAL_WORK_ROOT` to per-machine storage, then use its Just `test`,
`capture`, and `analyze` recipes. `KEY` and ascending, zero-based `VALUES` can
select a narrower sweep. All configurations validate before launching Weston;
the fixture refuses a reused output directory and pins the installed binary.

Accepted artifacts are under
`/mnt/ssd3/niri-material/material-36e968/sweep-20260911T100050`.
They include source/binary identity, machine metadata, requested/observed timing,
configs, pixels, raw motion series, comparison results, and source snapshots.
Missing planned cases, late/sparse motion, nonmonotonic progress, and failure to
return to identical settled pixels are errors. A flat/unresolved curve remains
a supported measurement outcome.

All 485 retained artifact hashes were verified. The experiment manifest SHA-256
is `a08d1988c7be71aa02ce5670b948caa327c53e9714b6cc7749ad2370f0754d4c`.
The existing analyzer suite passed 55 tests; three new integrity tests cover
coverage, repeat-noise handling, and complete planned-case inventory.
