# Material idle budget: evidence

**Status:** quiescence and cadence verified (Task 2, `material-4241c3`,
2026-09-25). Board-power acceptance is still pending Task 3, which needs the
operator-arranged dedicated DRM session. This page makes no power claim and
proposes no watt limit yet.

Design: [idle budget](../specs/2026-09-11-material-idle-budget-design.md).
Plan: [execution plan](../plans/2026-09-11-material-idle-budget.md).
Full method, per-draw numbers and the artifact manifest are in niri-experiments
`results/idle-budget`, `docs/results/2026-09-11-idle-budget.md` (`f0c13ac`).

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
- The fixture judges only at the end of a run; per-case analysis and a pilot
  mode are `material-b15ad7`.
