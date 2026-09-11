---
id: material-300b87
title: Accept aurora and iridescence on the installed hardware renderer
status: done
priority: 2
size: m
owner: material-300b87
created: 2026-09-11T08:00:32Z
updated: 2026-09-11T09:07:42Z
depends: [material-f0fc7b]
tags: [rendering, performance]
---

Native plan evidence used llvmpipe. On the restarted installed niri 7526af1d or newer, capture Aurora and Rainbow with CPU metadata and GPU/driver identity; check edge-weighted iridescence and face colors, aurora pinned and stepped behavior, both focus states, reduced motion and off. Measure warmed median material-pass overhead and idle redraw/power against plain glass on the hardware renderer. Compare the plan acceptance budgets, record repeatable commands and captures under docs/materials/, and file concrete regressions if budgets fail. Prism controls and profiles are landing via prism-763054 and prism-08c1de. Coordinate broader idle/power budgets with material-265eb0 and cost reporting with prism-d54be4; do not duplicate their general scope.

## Notes

- 2026-09-11T08:36:09Z (material-300b87): Hardware preflight selects RTX3070/NVIDIA610.57.04 in headless Weston and installed niri7526af1d. Both-focus visual run passes neutral, pinned/off and moving gates; chroma chamfer622.168 vs face98.9579 Q16, aurora face RMSE5918.03. Review found shared count_last20 false-passes empty/truncated traces; regression tests now cover this and helper validates the idle-inhibit heartbeat. Repeated GPU/whole-board power run in progress; source has no render-code diff from installed7526af1d. Plan has no numeric GPU/power threshold, so report measured deltas and variance.
- 2026-09-11T09:07:42Z (material-300b87): Recorded installed7526af1d RTX3070 pixel/focus/preset checks, 15 warmed GPU traces and 18 idle traces (0/4/2 Hz), plus four running-session captures with config/focus restored. Relative GPU and whole-board power deltas remain unresolved from clock/desktop variance; broader isolation stays material-265eb0. Fixed false-zero acceptance of incomplete traces with regression tests. Fixtures/evidence manifest pushed as niri-experiments e6522d0; native evidence in docs/materials/2026-09-11-material-hardware-evidence.md.
