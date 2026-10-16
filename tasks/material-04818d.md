---
id: material-04818d
title: Idle GPU floor flips between ~10.4 and ~17 W after nested runs and refuses long cost captures
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-16T11:49:16Z
updated: 2026-10-16T11:49:16Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

Seen 2026-10-16 (TTY, desktop stopped): after a nested niri session exits, the RTX 3070's idle P8 floor (210/405 MHz, no GPU clients, fan target 45%) settles at either ~10.4 W or ~17 W and can flip between sub-runs. capture-meta's settle gate (gpu_power_w within 1.5 W of the baseline) refused two noise A/B attempts on it (material-03fdf3: 10.39 -> 17.115 W, then 17.125 -> 10.39 W); a third passed. A 3-min passive sample with no load held 9.7-10.4 W. Separately, an fb0-blanked display gave an 8.9 W baseline and woke during the first sub-run (+1.4 W), refusing material-2283b8's first pilot. The 2026-10-07 runs sat at 17 W throughout and the 2026-10-03..06 runs at 10.4 W, so the two states are not new. Clocks do not distinguish them (material-f386ae).

What: find what selects the state (candidates: memory/voltage rail state after a P0 burst, PCIe ASPM, display link state, fan control left manual by gpu-fan-curve without a Wayland session), with nvidia-smi -q power/perf readings in both states. Then either make the capture prevent the flip, or let the settle gate accept the two known floors with a recorded reason. Draw medians in one capture were unaffected so far. Done when a 24-sub-run A/B is not refused by a floor flip, or the gate change is reviewed.
