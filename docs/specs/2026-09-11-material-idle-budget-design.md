# Material idle GPU and power budget

**Status:** Task 1 fixture implemented, offline validated and reviewed on
experiment `results/idle-budget` at `8db5dc8`, with capture-protocol adoption
integrated at `9d7d43f`. Task 2 traces passed on 2026-09-25 (25 of 25; see the
[evidence](../materials/2026-09-11-idle-budget-evidence.md)). Power collection
(Task 3) remains pending, so the budget sections below are still proposed rules.

**Task:** `material-265eb0`, within `material-53f873`.
**Plan:** [execution plan](../plans/2026-09-11-material-idle-budget.md).

## Scope and existing evidence

Verify that existing material dynamics become quiescent after a finite stimulus,
then measure the board-power effect of enabling settled dynamics and intentional
low-rate animation. Do not implement idle micro-movement, redesign springs,
change Prism ranges, or introduce a new animation scheduler in this task.

At native `1985dd02`:

- `JellyFingerprint::quantize` in `src/render_helpers/material/mod.rs` pins its
  time component when activity is zero; `jelly_fingerprint_ignores_time_at_rest`
  already tests this. `MaterialState::advance_commit` only advances when pixel
  inputs change, with existing unchanged-input tests.
- `Tile::advance_animations` removes completed transitions;
  `are_transitions_ongoing` and the material tick deadline distinguish finite
  transitions from continuing optic/signal activity.
- Aurora schedules a deadline only when lit and moving. Its existing tests
  cover neutral, pinned, reduced-motion, and motion-off policies.
- The [hardware optic run](../materials/2026-09-11-material-hardware-evidence.md)
  observed zero quiet redraws and exact 4/2 Hz Aurora cadence, but whole-board
  variation prevented attributable power measurements.
- The [motion sweep](../materials/2026-09-11-jelly-motion-sweep.md) verified
  visible column-move response and exact pixel return after settling. Equal
  pixels alone do not prove absence of redraws or GPU work.
- The roughly 300-second micro-movement described by the sprint is proposed
  behavior. This preparation found no such scheduler in the rendering code;
  do not describe it as an existing source of continuous work.

## Two measurement lanes

**Trace lane:** reuse the headless NVIDIA Weston/niri host and matching-source
Tracy build with a retained experiment-only action span for trace-time
anchoring. This may run alongside the normal desktop because it measures the
owned compositor's zones. It cannot establish isolated board-power cost.

**Power lane:** run an uninstrumented niri directly on DRM in a dedicated local
session, without Weston, the normal desktop, a greeter, or other GPU clients.
The operator must arrange that session at execution time. The harness never
logs the user out, stops a display manager, kills unrelated clients, switches
VTs, changes GPU clocks, or changes persistent desktop configuration. A VT
switch alone is not evidence of GPU isolation.

Both lanes record actual GPU/driver identity, binary hashes, source commit and
source differences, display mode/scale/VRR, window geometry, config hashes, tool
versions, and owned process IDs. Resolve the display mode at preflight and hold
it unchanged for the run. Reject llvmpipe/software rendering and missing identity.
The initial fixture supports one NVIDIA GPU and one active output; reject other
layouts. Device-user enumeration must have complete read visibility and fail
explicitly if unavailable, without an interactive privilege prompt.
The earlier trace and power scenes have different output geometry; compare
cases within one lane, never their absolute costs across lanes.

For power, enumerate graphics and compute clients plus device users before
launch and throughout collection; record names and PIDs, not only utilization.
Permit only the recorded PIDs of the fixture's compositor, transparent kitty
clients, and owned wallpaper client. After mapping, the inventory must positively
list the compositor and both kitty PIDs; repeat this check throughout collection.
Use graphics/compute process reporting plus device-user enumeration (for example,
`nvidia-smi -q -d PIDS` and `fuser -v /dev/nvidia* /dev/dri/*`); compute-only
queries cannot establish graphics visibility. An empty or unsupported query,
missing owned PID, or incomplete device visibility invalidates the block.
Background GPU clients invalidate a block even if utilization looks low. The operator resolves contamination; the harness does not kill it.

## Scene and cases

Reuse the signal-free baseline and transparent probe/anchor from
niri-experiments `results/jelly-motion` (`f1b0741`). Keep the diagnostic backdrop,
ior 1.5, thickness 20, bevel 12, offsets (6,6), roughness/distortion/noise 0,
saturation 1, and backdrop blur false. Disable focus/accent responses and pin
ring drift. The anchor must stay transparent and stationary during observation.

Pin both movement and resize springs to damping-ratio 1, stiffness 100,
epsilon .0001. Use the existing two-column layout at width proportion .4.
A movement stimulus moves the probe column right and back left; a resize
stimulus changes its column width to 55% and back to 40%. Wait 3 s from the
first action's IPC completion before sending the return action in both stimuli
(including case A); record both actions and their timing. Restore and verify
the original window geometry. Take geometry/pixel captures outside observation
windows only. The before image is the pre-stimulus settled capture; the after
image and any post-observation geometry restore/verify actions must wait until
the capture process has exited. Keep pointer, terminal contents, signals, config,
and focus stable during observation.

| Case | Settings | Trace expectation after settling |
| --- | --- | --- |
| A: quiet-disabled | flex 0, ripple 0, Aurora 0, animations off | zero redraws/draws |
| B: quiet-enabled | flex .02, ripple .5, Aurora 0, animations enabled | zero redraws/draws |
| P: pinned field | B plus Aurora .5, drift-hz 0 | zero redraws/draws |
| C: moving field | B plus Aurora .5, drift-hz 4 | 4 Hz |
| D: reduced field | C plus signal motion reduced | 2 Hz |
| O: motion off | C plus signal motion off | zero redraws/draws within 3 s, including native spring settling |

`signal motion off` controls optic/signal motion; it is not assumed to disable
native window-movement springs. C and D deliberately retain a live stimulus and
are not required to settle to zero. No screenshots, IPC polls, or config reloads
are allowed inside quiet trace windows except the pre-existing heartbeat.

## Proposed acceptance and coverage rules

These are reviewable engineering rules for this fixture, not established
universal budgets:

1. Quiet states A/B/P/O: zero `Niri::redraw` zones and zero material GPU draw
   zones during a fully covered 20-second observation window. Check A/B after
   both movement and resize; check P/O after movement. Three repetitions each.
   Start the window three seconds after the final stimulus. The pinned critical
   spring takes about 1.84 s at a 489 px displacement because its epsilon is an
   absolute pixel distance; 3 s allows the final render and resize client round
   trip. A later settle fails this proposed three-second fixture budget; do not
   move the observation window until it passes.
2. One additional B capture observes 600 quiet seconds starting 3 s after
   movement. This detects delayed wakeups in the current implementation; it does not test a
   future 300-second micro-movement feature. Require zero redraws/material draws.
3. C/D: count 79–81 / 39–41 redraws in each fully covered 20-second window,
   allowing one boundary frame; repeat three times. Record material draw count,
   summed GPU-zone time per second, and durations. Do not infer zero-cost optics
   from noisy relative timing or from a small isolated material-pass median.
4. Validate `Niri::refresh_idle_inhibit` coverage in the explicit observation
   interval: first/last heartbeat within 1.5 s of the interval ends and no
   internal gap over 1.5 s. Reject unordered/nonfinite timestamps, missing data,
   a truncated capture, and missing planned cases. A long trace must cover the
   whole 600-second interval, not merely its last 20 seconds. Require a positive
   material GPU draw during the preceding stimulus in the same trace, proving
   the GPU channel was recording; an entirely empty GPU export is not a valid
   zero-work result. Case A's animations-off jump may supply just one material
   draw; that satisfies the positive control without adding animation to A.
5. Require exact settled-pixel equality for A/B at the same final geometry.
   Keep the existing fingerprint/unchanged-commit tests as complementary
   evidence. Zero material GPU draws implies no material uniform uploads in
   that observation window; it does not claim zero process-wide CPU wakeups.
6. Power precision target: sham A/A blocks must have a noise floor at most
   **1.0 W** before attempting a resolved budget conclusion. This is a proposed
   instrument-resolution target, not a feature power allowance. If missed,
   report insufficient precision; do not call a noisy comparison a pass.
7. Settled B must have no resolved positive board-power increment over A.
   Report the measured upper bound at the achieved resolution. C/D costs are
   descriptive results used to choose a later product watt budget; this task
   does not invent a universal allowed wattage for visible animation.

Each trace observation uses a fresh compositor process and its first Tracy capture.
The non-ondemand `profile-with-tracy` build buffers events from process start,
including setup IPC before capture connects. Record every IPC action from launch
through capture completion in one serial controller journal, including helper
spawn/focus/width actions. Count only successful IPC calls; record validation
rejections separately because they produce no marker, and abort on an unknown
call outcome. Match the complete ordered marker list to the successful entries
by count and ordinal; their last two actions must be the declared outward/return
stimulus pair, with the pinned 3 s wait. Reject missing/extra markers or later
actions, and anchor `stimulus_end` on the last marker's end in Tracy time. Do not
assume a connection discards setup events or use host IPC time as trace time.
For the GPU positive control, count draws from the outward marker's start to
observation start; GPU work need not overlap the short CPU action span itself.

The current mechanisms may already satisfy all behavioral gates. Only a
reproduced regression justifies a renderer fix, with a separately scoped task
and regression test. Do not preemptively rewrite scheduling or caching.

## Isolated power protocol

Use the uninstrumented, source-identified binary. Hold the physical output mode,
scale, refresh, VRR policy, scene geometry, and sampling overhead constant. Leave
DVFS enabled; forcing high clocks would change the idle-power question. Record
P-state, graphics/memory clocks, utilization, temperature, and power alongside
client inventory. A stable clock state is evidence to record, not proof that
all confounders have disappeared.

Each window lasts 90 s: 60 s warmup after config application and any stimulus,
then 30 s observation. Schedule 1 Hz queries on absolute monotonic deadlines
`t0 + k * 1 s`, not a one-second sleep after each query. Record missed deadlines
and skip elapsed ticks rather than issuing catch-up bursts. Record monotonic
request/return times around each query; use return times for interval assignment
and coverage; reject failed/unsupported/nonfinite samples, non-increasing
times, gaps over 1.5 s, or endpoint coverage worse than 1.5 s. Require at least
29 observations per 30-second interval. Verify unchanged outputs and client
inventory across each block. No captures or unrelated work run concurrently. Board power excludes CPU,
monitor, and total-system energy.

Use three ABBA blocks for each comparison, alternating the second block to
BAAB. First run a sham A/A comparison; then A/B, B/C, and B/D. Warmup is excluded
from every window median. Budget about **72 minutes** for 48 windows, plus
preflight and any interruptions. Refuse to overwrite prior outputs. Do not
rerun selectively to obtain a desired sign.

For window medians `(a1, b1, b2, a2)`, a block delta is
`(b1 + b2 - a1 - a2) / 2`. Reorder BAAB labels before that calculation. Report
all medians and deltas; overall delta is their median. Report these floors:

- `sham_floor_w`: maximum of absolute sham block deltas and all within-sham
  A or B repeat differences. Require this ≤1.0 W before interpreting comparisons.
- `abba_floor_w`: maximum of absolute sham block deltas and the range of
  comparison block deltas (the floor after linear drift cancellation).
- `raw_repeat_floor_w`: maximum within-block A or B repeat difference across
  both sham and comparison blocks.
- `floor_w`: maximum of `sham_floor_w`, `abba_floor_w`, and
  `raw_repeat_floor_w`; retain this conservative floor as the ≤1.0 W gate.

The raw repeat floor deliberately keeps sensitivity to drift that ABBA/BAAB
cancels from the delta: a same-case change over 1 W across a block fails precision
even if drift cancellation is exact. Report both floors to distinguish raw repeat
variation from residual block/sham noise; they diagnose sensitivity, not prove a
DVFS or drift cause. These are descriptive bounds, not confidence intervals.
A positive delta is resolved only when it exceeds `floor_w`; a conservative upper
estimate is `max(0, delta + floor_w)`. Report sham and combined precision verdicts
separately; neither a small delta nor a small cancelled floor overrides either gate.

If isolation or precision cannot be achieved, keep the power execution task
open with the exact blocker. Preserve valid trace results independently. Do
not repeat the earlier shared-desktop power comparison and label it isolated.

## Artifacts and execution boundary

New experiment fixtures/results belong on niri-experiments branch
`results/idle-budget`, based on `results/jelly-motion`. Stable evidence and the
budget contract belong in native `docs/materials/`. Raw traces, CSV, binaries,
configs, inventory, logs, interval boundaries, and SHA-256 manifests stay on
per-machine storage under `NIRI_MATERIAL_WORK_ROOT/material-265eb0/`.

Fixture implementation and offline validation (Task 1) are complete. The user
resumed Task 2 on 2026-09-12; trace collection must pass the capture protocol's
headless preflight and per-sub-run settle checks. The isolated-session operator
checkpoint still precedes the power run. Logout and VT switches remain outside
the harness's authority. The parent closes only after its execution tasks and valid power
conclusion are complete, or its scope is explicitly revised by the user.
