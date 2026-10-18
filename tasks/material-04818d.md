---
id: material-04818d
title: Idle GPU floor flips between ~10.4 and ~17 W after nested runs and refuses long cost captures
status: doing
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet]
owner: materials-26.04
created: 2026-10-16T11:49:16Z
updated: 2026-10-18T03:08:02Z
started: 2026-10-16T12:34:35Z
depends: []
tags: [rendering]
agent: claude-code/claude-opus-5-5
---

Seen 2026-10-16 (TTY, desktop stopped): after a nested niri session exits, the RTX 3070's idle P8 floor (210/405 MHz, no GPU clients, fan target 45%) settles at either ~10.4 W or ~17 W and can flip between sub-runs. capture-meta's settle gate (gpu_power_w within 1.5 W of the baseline) refused two noise A/B attempts on it (material-03fdf3: 10.39 -> 17.115 W, then 17.125 -> 10.39 W); a third passed. A 3-min passive sample with no load held 9.7-10.4 W. Separately, an fb0-blanked display gave an 8.9 W baseline and woke during the first sub-run (+1.4 W), refusing material-2283b8's first pilot. The 2026-10-07 runs sat at 17 W throughout and the 2026-10-03..06 runs at 10.4 W, so the two states are not new. Clocks do not distinguish them (material-f386ae).

What: find what selects the state (candidates: memory/voltage rail state after a P0 burst, PCIe ASPM, display link state, fan control left manual by gpu-fan-curve without a Wayland session), with nvidia-smi -q power/perf readings in both states. Then either make the capture prevent the flip, or let the settle gate accept the two known floors with a recorded reason. Draw medians in one capture were unaffected so far. Done when a 24-sub-run A/B is not refused by a floor flip, or the gate change is reviewed.

## Notes

- 2026-10-16T12:34:35Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T12:34:40Z (materials-26.04): parked (waiting on user, quiet; headless, 30 min): On a quiet TTY host (desktop stopped): from the main checkout, dump nvidia-smi -q (full) at rest, then run noise-layers-cost.sh none (NOISE_LAYERS_COST_PILOT=1, CAPTURE_TASK=material-04818d, fresh OUT; build ~3, run ~3) repeatedly, sampling power.draw between runs, until the floor lands on ~17 W; dump nvidia-smi -q again and diff the two (power/perf/clocks event reasons, PCIe, fan, display). Then propose prevention or a reviewed settle-gate change. Earlier: the 2026-10-16 session saw both floors and two refused A/B attempts.
  provenance: {"harness_session":"claude-code:45aab597-9132-43ae-af7a-ff9fd44c6248","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T11:40:09Z (capture-gates-evidence): The settle gate change its done-criterion allows is reviewed and implemented: docs/specs/2026-10-17-capture-evidence-gates-design.md (client-free headless power steps are annotated). The floor's cause stays this task's question.
- 2026-10-17T13:09:05Z (materials-26.04): Since material-d2e552 (eadc3835), noise-layers-cost.sh samples GPU clocks by default: each case now keeps <case>.gpu-clocks.csv (100 ms power.draw/pstate/clocks through the capture) and metrics.txt reports idle_clock_power_median_w, which shows the floor (≈10.4 vs ≈17 W) the case ran on. CAPTURE_GPU_CLOCKS=0 restores the unsampled condition.
- 2026-10-18T03:08:02Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:5449fcbc-f1d5-4050-872d-855f55483679","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
