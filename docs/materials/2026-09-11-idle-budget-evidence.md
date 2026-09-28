# Material idle budget: evidence

**Status:** complete. Quiescence and cadence were verified by traces (Task 2,
`material-4241c3`, 2026-09-25). Isolated board power was measured on DRM
(Task 3, `material-5f9dee`, 2026-09-27). Material at rest passes the idle
budget: jelly shows no resolvable board-power increase over plain glass at
the 1.0 W resolution target. A drifting aurora's cost is measured below. A
product watt allowance for it is a separate decision.

Design: [idle budget](../specs/2026-09-11-material-idle-budget-design.md).
Plan: [execution plan](../plans/2026-09-11-material-idle-budget.md).
Full method, per-window numbers and the artifact manifests are in
niri-experiments `results/idle-budget`, `docs/results/2026-09-11-idle-budget.md`.

## What is established

After finite material motion settles, niri does no rendering work. In every quiet
observation window (`[stimulus_end + 3 s, + 23 s)`, and one 600 s hold), across
animations off (A), jelly (B), a pinned aurora (P) and a drifting aurora with
`motion "off"` (O), after column moves and resizes:

- zero `Niri::redraw` zones and zero material GPU draws; no GPU zones at all;
- the screen returns to its pre-stimulus pixels exactly (decoded RGB equal);
- the only CPU activity is idle event-loop wakes at about 1.1 Hz (the 1 Hz
  frame-callback fallback timer), never a render.

A drifting aurora that keeps animating is paced, not free-running:
`drift-hz 4` redraws at exactly 4 Hz (80 per 20 s, median spacing 250.00 ms)
and `motion "reduced"` halves it to 2 Hz (40 per 20 s, 500.00 ms). Each redraw
costs one material draw of about 0.5 ms on the RTX 3070, about 2.0 ms of GPU
time per second at 4 Hz and 1.0–1.43 ms/s at 2 Hz. Clocks during the capture
were not sampled, so these costs are not tied to a known clock.

## Run

25 of 25 observations passed; the run's own analyzer and an independent
recomputation from the raw traces agree exactly.

| Case | Redraws / material draws in window | Pixels return |
| --- | --- | --- |
| A move ×3, A resize ×3 | 0 / 0 | yes |
| B move ×3, B resize ×3 | 0 / 0 | yes |
| P move ×3 | 0 / 0 | yes |
| O move ×3 | 0 / 0 | yes |
| B 600 s hold | 0 / 0 | yes |
| C (4 Hz) ×3 | 80 / 80 | animates by design |
| D (2 Hz) ×3 | 40 / 40 | animates by design |

Every trace also shows material GPU draws during its stimulus (5 to 136), so
the GPU channel was live before each zero-draw window.

`tools/capture-meta show` for the run (header):

```
run trace-20260925T220712  task material-265eb0  fixture idle-budget.sh  lane headless
started 2026-09-25T22:07:12-04:00

environment
  kernel 7.2.2-arch1-1
  cpu AMD Ryzen Threadripper 1950X 16-Core Processor (32 threads)
  gpu NVIDIA GeForce RTX 3070 615.71.09 /dev/dri/renderD128
  session tty
  kitty kitty 0.48.2 created by Kovid Goyal
  tracy 0.13.1
  weston weston 15.0.1

baseline
  cpu_busy_pct 1.1  load1 0.85  gpu_util_pct 0.0  gpu_power_w 10.83
  gpu_power_iqr_w 0.145  gpu_clock_mhz 210  gpu_pstates ['P8']

preflight quiet
sub-runs: 25 of 25 settled

provenance
  source fb8e9a959cdf8320e8deb6d89d5fb44f33b837da material-265eb0
  binary niri 227727d6a7764483cd50047d2188852e01735c08153a810bf312cb924c2e887b
```

The recorded source `fb8e9a95` is the worktree head at capture time. The
binary was built from `d91a6024` (`identity.json`, `source.tar` byte-identical
to its `git archive`), with `profile-with-tracy` and the experiment's
`IdleBudget::action` marker patch. The two commits differ only in docs and
task files, and `just test` passed 434 tests at `fb8e9a95`.

## Isolated board power

An uninstrumented niri (`191ad747`, binary SHA-256 `282fdb10…`) ran directly on
DRM from a TTY with the desktop stopped. It drove DP-1 at 3440×1440@59.999,
scale 1, VRR off. The scene was a diagnostic backdrop and two transparent
kitty windows, and the only GPU clients present were niri and kitty. There
were 48 windows of 60 s warmup and 30 s observation at 1 Hz, run as a sham
A/A comparison and then A/B, B/C and B/D, each three ABBA/BAAB blocks. Every
window passed its settle check and the full GPU-client inventory.

| Comparison | Delta | Conservative floor | Upper estimate | Resolved increase |
| --- | ---: | ---: | ---: | --- |
| sham A/A | −0.025 W | 0.105 W | 0.08 W | no |
| plain → jelly at rest (A→B) | +0.015 W | 0.118 W | 0.13 W | **no** |
| jelly → aurora 4 Hz (B→C) | +0.845 W | 0.265 W | 1.11 W | yes |
| jelly → aurora 2 Hz (B→D) | +0.68 W | 0.30 W | 0.98 W | yes |

- **Precision.** The sham floor is 0.105 W and every combined floor is at most
  0.30 W, far inside the 1.0 W target.
- **Idle budget.** Resting jelly costs nothing resolvable. A is plain glass with
  animations off and B is jelly with the spring animations on, so this bounds
  the material and its animation machinery together, at rest.
- **Aurora.** A drifting aurora costs most of a watt, and halving its rate
  saves only about 20 %. Every sample stayed in P8 at 210 MHz with 0–4 % GPU
  utilization, so the cost is not clock ramping. It is consistent with each
  redraw holding the board out of deeper idle, which this run does not
  establish. Board power excludes CPU, monitor and total-system energy.

`tools/capture-meta show` for the power run (header):

```
run power-full-20260927T031335  task material-265eb0  fixture idle-budget.sh  lane dedicated
started 2026-09-27T03:13:46-04:00

environment
  kernel 7.2.2-arch1-1
  gpu NVIDIA GeForce RTX 3070 615.71.09 /dev/dri/renderD128
  session tty
  kitty kitty 0.49.1 created by Kovid Goyal

baseline
  cpu_busy_pct 3.6  load1 1.02  gpu_util_pct 0.0  gpu_power_w 10.61
  gpu_power_iqr_w 0.537  gpu_clock_mhz 210  gpu_pstates ['P8']

preflight quiet
sub-runs: 48 of 48 settled

provenance
  source b6bf6014f3fb974c0aae796e4e3f9b9fdfd4a543 material-265eb0 dirty
  binary niri 282fdb108d63c23e7c863f82d1c684fd965c8761f9e600bfd6a0bb817016b9aa
```

The recorded source is the worktree head at capture time. Its uncommitted
files were task notes. The binary was built from `191ad747` (`identity.json`,
`source.tar`).

## Findings along the way

- The first complete trace failed every quiet case on one redraw exactly 3.0 s
  after the stimulus (`material-4be9c3`). It was the test client: kitty hides
  the mouse cursor 3 s after a window slides under the pointer, and niri
  correctly redraws for the cursor change. The fixture now runs its kitty
  clients with `mouse_hide_wait=0`. This is a client behavior, not a niri
  wake; a real session with kitty under a stationary pointer will see that
  one redraw.
- Captures that re-check host quiet before every case run from a TTY with the
  desktop stopped; three desktop attempts each lost a settle check to a
  different transient (idle lock, a desktop GPU blip, build load).
- `material-b15ad7` now judges each case as it lands and offers a pilot for
  each lane. The live pilots ran on 2026-09-27 from a TTY with the desktop
  stopped, on binaries rebuilt from `4607d054`. Fail-fast, an interrupted
  trace, and the power pilot each behaved as designed. The power pilot's
  four sham windows had medians of 11.39 to 11.47 W. The trace pilot's
  cadence cases pass (C 81/81, D 41/41), and both quiet A cases fail on one
  redraw about 30 s after startup. That redraw comes from the attention
  gate's idle edge, which redraws every output unconditionally
  (`material-cd7deb`). The full trace waits on that fix.
- The power lane's first DRM pilots found three fixture defects that synthetic
  validation could not show. Seat managers holding the KMS node were counted
  as GPU clients, `fuser`'s split output was misparsed, and a settle could
  start inside our own compositor's teardown P-state transient. Each was fixed
  with a test before the full run (experiments `73fbbe0`, `7953684`,
  `127cf6b`).
