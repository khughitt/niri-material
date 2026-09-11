# Material idle GPU and power budget execution plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Status:** prepared for review; execution has not started. Stop here until the
user resumes execution. All unchecked steps describe future work.

**Goal:** verify quiescence after finite material motion and establish a
repeatability-bounded idle board-power comparison in an isolated session.

**Architecture:** reuse the archived headless/Tracy and motion fixtures for
behavioral checks, then run a separate uninstrumented DRM power experiment.
One small stdlib analyzer validates explicit observation windows, coverage,
case inventory, and repeated power comparisons. Renderer changes are conditional
on a reproduced defect and are not part of the initial implementation.

**Tech Stack:** existing niri, Weston, Tracy 0.13.1, nvidia-smi, kitty, ImageMagick,
Python stdlib, Bash, Just, and the task CLI. No new dependency or IPC interface.

**Spec:** [design and proposed acceptance rules](../specs/2026-09-11-material-idle-budget-design.md).

## Global constraints

- Native base `1985dd02`, branch/worktree `material-265eb0` /
  `.worktrees/material-265eb0`. Present all native worktree paths with that prefix.
- Preparation only: do not execute any step below until the user resumes.
- Read `docs/materials/render-pipeline.md` before any rendering investigation.
- Start the relevant execution child with `tasks start`; close it in the commit
  containing its result. Run `tasks check` before commits; use native Just gates.
- Experiments: create `results/idle-budget` from `results/jelly-motion` (`f1b0741`)
  in sibling `.worktrees/material-265eb0` only when execution starts. Archive and
  push that results branch; do not merge unrelated experiment result branches.
- Raw artifacts use fresh timestamped directories under
  `NIRI_MATERIAL_WORK_ROOT/material-265eb0/`; snapshot executable/source/config
  identities. Do not infer a binary's source from a checkout label.
- Quiet observation: zero redraws and material GPU draws after two seconds;
  window length 20 s, plus one 600-second quiet hold. C/D cadence 79–81 / 39–41
  redraws per 20 s. Heartbeat endpoint/internal-gap tolerance 1.5 s.
- Power: dedicated DRM session, no other GPU clients, uninstrumented binary;
  60 s warmup + 30 s observation, 1 Hz sampling, three blocks per comparison.
  Proposed resolution target 1.0 W; use the design's repeat-floor formula.
- Never stop the user's session/services or change clocks automatically. A VT
  switch is not isolation. Preserve valid partial evidence; keep blocked work open.
- No new motion behavior, Prism range changes, speculative scheduler, or blanket
  renderer refactor. No numeric feature watt limit is assumed.

## File map

Paths below are relative to the named repository; use the corresponding worktree
prefix when communicating commands to the user.

| Repository | Path | Responsibility |
| --- | --- | --- |
| experiments | `fixtures/idle-budget.sh` | source shared native helpers; own trace/DRM fixture lifecycle and cases |
| experiments | `fixtures/idle-budget.py` | sampling, explicit-window validation, power reduction, CLI |
| experiments | `fixtures/test_idle_budget.py` | small synthetic integrity/regression tests |
| experiments | `fixtures/idle-budget.just` | timed test/build/trace/power/analyze entry points |
| experiments | `fixtures/idle-budget-trace-marker.patch` | experiment-only action span for exact trace-time anchoring |
| experiments | `docs/results/2026-09-11-idle-budget.md` and adjacent `.sha256` | reproducibility, results, artifact hashes |
| native | `docs/materials/2026-09-11-idle-budget-evidence.md` | stable evidence and proposed-budget verdict |
| native | `docs/materials/material-config.md` | link measured limits once they exist |

Reuse `docs/materials/scripts/glass-optic-smoke-lib.sh` lifecycle, trace tools,
port reservation, exports, and config emission. Reuse the column-move setup in
experiments `fixtures/jelly-motion.sh` and the config-load completion pattern in
`fixtures/aurora-iridescence-installed.py`. Do not import either executable
Python file as a library: their top-level code runs actions. Copy only the small
needed stdlib routine. Keep the existing helper's 30-second capture default
unchanged; the new fixture owns an explicit-duration capture function for long
traces, reusing its owned `CAP_PID`, ready/wait, and cleanup functions.

### Task 1: Build and validate the bounded measurement fixture

**Interfaces:** `idle-budget.py analyze RUN` consumes `manifest.json`, CPU/GPU
exports, interval metadata, and optional power samples. `check_coverage(times,
start, end)` takes seconds on one monotonic axis. `power_comparison(sham,
blocks)` takes lists of four window medians reordered to ABBA. Results retain
all input medians, delta, floor, upper estimate, and integrity failures.

- [ ] Add failing synthetic tests before implementing the analyzer. Include
  missing/duplicate planned cases, empty/truncated/gapped intervals, negative or
  nonfinite samples, and repeat-noise cancellation. Representative assertions:

```python
with self.assertRaises(ValueError):
    check_coverage([], 10, 30)
with self.assertRaises(ValueError):
    check_coverage([10, 11, 29, 30], 10, 30)
check_coverage(list(range(10, 31)), 10, 30)
r = power_comparison([[20, 20, 20, 20]] * 3,
                     [[20, 22, 122, 120]] * 3)
self.assertEqual(r['delta_w'], 2)
self.assertGreaterEqual(r['floor_w'], 100)
self.assertFalse(r['resolved_increase'])
```

- [ ] Implement the minimum reducer and coverage validator, using the same
  formula in both tests and the documented protocol:

```python
from math import isfinite
from statistics import median

def block_delta(v):
    return (v[1] + v[2] - v[0] - v[3]) / 2

def power_comparison(sham, blocks):
    if len(sham) != 3 or len(blocks) != 3:
        raise ValueError('three complete blocks required')
    if any(len(v) != 4 or any(not isfinite(x) or x < 0 for x in v)
           for v in sham + blocks):
        raise ValueError('invalid window medians')
    ds = [block_delta(v) for v in blocks]
    floor = max([abs(block_delta(v)) for v in sham]
                + [max(ds) - min(ds)]
                + [abs(v[i] - v[j]) for v in sham + blocks
                   for i, j in [(0, 3), (1, 2)]])
    delta = median(ds)
    return dict(delta_w=delta, floor_w=floor,
                resolved_increase=delta > floor,
                precision_ok=floor <= 1.0,
                upper_w=max(0, delta + floor))
```

  `check_coverage` must reject nonfinite or non-increasing timestamps; retain
  only samples in `[start,end]`, require endpoint distances ≤1.5 s and internal
  gaps ≤1.5 s. Trace zone counting uses `[start,end)` to avoid double-counting
  a boundary. Power windows additionally require at least 29 samples in 30 s.
  Check manifest cases and repetitions against the declared matrix exactly.

- [ ] Create `fixtures/idle-budget-trace-marker.patch` with exactly one inserted
  line in `src/ipc/server.rs`, inside the `Request::Action` idle closure:

```rust
state.niri.advance_animations();
let _marker = tracy_client::span!("IdleBudget::action");
state.do_action(action, false);
let _ = tx.send_blocking(());
```

  Preserve the base commit and patch hash. Build from a fresh `git archive HEAD`
  extraction under per-machine `TRACE_SOURCE`, apply the patch there, and keep
  production source unchanged. Refuse a reused source directory. The build
  recipe takes `TRACE_SOURCE` and `CARGO_TARGET_DIR`; it records both base and
  patch identities beside the binary hash. Only the trace binary has this span.
- [ ] Add Just recipes, all through native `tools/tt`: `test`, `build-tracy`,
  `trace`, `power`, and `analyze directory`. `build-tracy` runs
  `cargo build --release --features profile-with-tracy` from `TRACE_SOURCE`
  with a per-machine `CARGO_TARGET_DIR` and an explicit `NIRI_BUILD_COMMIT`
  identifying the recorded base (the separate patch hash distinguishes this build). The power recipe accepts only the
  uninstrumented, separately identified binary.
- [ ] Implement the six configs from the design, validate all before launch,
  preserve input snapshots, and reject a used OUT. Implement duration-aware
  Tracy capture with `timeout(duration + 60)` and an explicit observation
  interval. Do not use `trace_end - 20` for the 600-second gate.
- [ ] In power mode, require dedicated-session preflight and complete GPU-client
  visibility before launch; record one fixed output mode. Own and wait for every
  sampler/capture child on success, failure, SIGINT, and SIGTERM. Do not change
  user config or session state. Implement the 1 Hz sampler with monotonic query
  start/return records and non-overlapping observation phases; reject gaps and
  contamination instead of silently dropping samples.
- [ ] Run `just --justfile fixtures/idle-budget.just test`, syntax/config checks,
  and a reviewer pass on interval alignment, zero-work false passes, power
  repeat floors, and cleanup. Include failure tests for truncated long coverage,
  absent GPU recording vs a valid empty observation window (with positive
  material draws during the preceding stimulus), and wrong binary identity.
  Commit fixture plus completed Task 1 record after checks. No power run yet.

### Task 2: Verify quiescence and bounded active cadence

**Consumes:** Task 1 fixture, source-identified Tracy binary with the retained
trace-only marker patch, pinned tools.
**Produces:** validated CPU/GPU traces, pixel-return evidence, and explicit
per-case verdicts; this task may finish even if isolated power is unavailable.

- [ ] Verify binary/source/GPU identity and record the inherited fingerprint,
  unchanged-commit, transition, and Aurora deadline tests. Use native Just tests;
  add a focused regression only if a behavior is missing or fails.
- [ ] Execute A/B after both move and resize, P/O after move, and C/D after
  settling: eight scenarios × three repetitions = 24 short observations. The
  owning compositor must receive no screenshot/config/input traffic inside
  observation windows. Observe `[stimulus_end + 2 s, stimulus_end + 22 s)`.
- [ ] Start capture after mapping clients; issue exactly the reset/final
  stimulus actions, then no further action until observation ends. Require
  exactly two `IdleBudget::action` marker zones from Task 1's trace binary;
  anchor `stimulus_end` to the end of the second zone using
  `ns_since_start + exec_time_ns`. Retain the corresponding controller action
  sequence. Missing/extra markers reject the observation. Do not treat
  controller IPC return time as a Tracy timestamp. The trace-only marker
  neither changes rendering behavior nor schedules another frame.
- [ ] Require at least one material GPU draw before the quiet interval in
  every trace. This positive control proves GPU recording is live before a
  zero-draw conclusion. Reject an entirely empty GPU channel.
- [ ] Run one 600-second quiet B hold after movement, with full heartbeat
  coverage and zero redraw/material-draw zones. Capture clean settled images
  before/after the interval and compare decoded pixels at AE 0.
- [ ] Analyze all intervals and expected case inventory. Report active GPU-zone
  sums/durations alongside cadence, with DVFS limits. If any behavioral gate
  fails, retain evidence and file a concrete regression under this task; do not
  automatically widen the settle delay or rewrite the renderer.
- [ ] Independently review raw intervals, absence-of-work coverage, and source
  attribution. Archive valid traces/results and mark Task 2 done in that commit.
  Record that board-power acceptance is still pending Task 3.

### Task 3: Measure isolated power and publish the budget verdict

**Consumes:** validated fixture/trace results; operator-provided dedicated DRM
session. **Produces:** isolated repeated comparisons, precision/idle verdict,
raw archive, and stable native documentation.

- [ ] Present the concrete fixture, runtime estimate (about 72 minutes plus
  setup), output mode, client inventory, and restoration/exit behavior. Let the
  operator arrange a session with the desktop and other GPU clients closed.
  Do not issue logout, display-manager-stop, or VT-switch commands for them.
  If the session is not available, park this child waiting on the user.
- [ ] From that session, run `just --justfile fixtures/idle-budget.just power`
  with `MATERIAL_ROOT`, `NIRI_MATERIAL_WORK_ROOT`, an identified uninstrumented
  binary, and a fresh OUT. Use a private config and launch directly on DRM:

```bash
env -u DISPLAY -u WAYLAND_DISPLAY -u NIRI_SOCKET \
  dbus-run-session -- "$NIRI_BINARY" -c "$OUT/config.kdl"
```

  The generated config starts the fixture controller inside that compositor.
  Confirm the DRM backend in its log; never accept a nested fallback.
  Reject any incomplete GPU-client query or other graphics/compute client.
- [ ] Collect sham A/A, then A/B, B/C, B/D. For each, execute three blocks in
  ABBA, BAAB, ABBA order. Each window is 60 s warmup + 30 s observation at 1 Hz.
  Save all 48 windows, output/process inventory, clocks/P-state/temperature,
  raw power samples, and config-load completion events outside observation.
- [ ] Analyze the sham floor before interpreting feature comparisons. A floor
  over 1.0 W, missing coverage, or external GPU work means insufficient precision
  or invalid evidence. Preserve the run and leave this child open; no repeated
  shared-desktop substitution and no selecting only favorable windows.
- [ ] Report all window medians, block deltas, repeat floor, and upper estimate.
  B must have no resolved positive idle increment over A at ≤1.0 W resolution.
  Report C/D costs descriptively; a product watt allowance remains a separate
  decision. Link any proven regression to the responsible task.
- [ ] Write experiment and native evidence, SHA-256 manifests, exact commands,
  source/binary identity, machine/display metadata, and all exclusions. Verify
  hashes and compare report values independently with raw samples.
- [ ] Update this design/plan status and native material-config link only when
  results exist. Close Task 3 and then `material-265eb0` only if required evidence
  and verdicts are complete. `tasks check`, native `just check`/push gate,
  conventional commits, archive push, native merge, and owned-worktree cleanup
  (after `tt-report`) finish execution. Keep unrelated worktrees untouched.

## Preparation handoff

The design's scope, matrix, isolation boundary, proposed precision target,
coverage rules, artifact locations, and failure handling map to Tasks 1–3.
No experimental code or measurement is included in the preparation commit.
Execution children: `material-ec6229` (fixture), `material-4241c3` (traces), and
`material-5f9dee` (isolated power), in dependency order.
Review the two-second settle budget, 1.0 W resolution target, and dedicated
session requirement before resuming. The execution task records remain open.
