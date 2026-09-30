# Sustained optic settling implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Status:** draft for owner review. No implementation or capture is authorized
by this document's creation. `material-0db905` delivers the reviewed spec and
plan only; the execution records belong under `material-f86183` and depend on
completion of that design task. Keep that idea's scoping decision for plan acceptance.

**Goal:** stop optional Aurora deadlines and uniform changes after input
inactivity, while preserving real client damage and phase-continuous resume.

**Architecture:** add a small pausable timeline to the existing shared `Clock`.
Optics receive logical time only; the registry suppresses paused deadlines and
maps the earliest logical deadline to real time. Existing activity edges update
that timeline and request a redraw only when attention or an optic is affected.

**Tech Stack:** existing Rust, std time/Rc/RefCell, Smithay/calloop, Tracy 0.13.1,
Bash, Python stdlib, Just and nextest. No new dependency, IPC request or config key.

**Spec:** [accepted design, round 4](../specs/2026-09-29-sustained-optic-settling-design.md).

## Global constraints

- `signal { idle-after-ms }`: default 30000 ms, 0 disables, maximum 3600000 ms.
  Aurora settles by default and shares this threshold with attention.
- Aurora retains its 600-second period and existing effective drift rate:
  full = configured rate, reduced = half rate, off/animations-off = zero.
  Amount 0 or drift 0 retains phase 0 and no deadline.
- Pause time, not the effective rate. Resume from held logical time with no
  catch-up, phase reset, or easing animation. Config/policy changes may change
  the bucket grid as specified in design §6.
- Production detector and timer calculations keep `get_monotonic_time()`.
  Pass each edge's timestamp explicitly to pause/resume. Never substitute cached
  or predicted `Clock::now_unadjusted()` for detector time.
- Virtual time belongs to timeline/layout tests using `Clock::with_time`.
  Niri fixtures use real time and real timers; never use `set_unadjusted` to
  freeze/advance across an activity edge. Add no clock-injection mechanism.
- Preserve finite focus beams, impulses, static level/accent, attention's
  absolute-clock resume, client updates, visibility gates and output semantics.
  Input idle is not signal Quiet, an idle inhibitor, or screencast activity.
- No optional real optic deadline and no changing optic fingerprint while
  settled. An affected edge permits the existing coalesced redraw and already
  armed one-shot; no repeating idle chain. Unaffected scenes retain no-op edges.
- All code and tests run in the eventual execution worktree. Rebase/merge the
  accepted documents onto current `materials-26.04` before implementation so its
  `test-one` recipe is available; resolve task notes by preserving both histories.
  Read `docs/materials/render-pipeline.md` before renderer edits.
- Each execution task runs its focused `just test-one`, then `just test-fast`,
  `tasks check`, and normal commit hooks. Regenerate `just upstream-report`
  after staging code changes. The full suite follows repository CI/push policy.
- Captures use an explicitly identified worktree binary, fresh artifact paths,
  preflight/settle gates and owned-process cleanup. Run the pilot through its
  verdict before a full matrix. Unavailable real-TTY or two-output lanes remain
  unverified. No host launcher changes, deployment or new power experiment.

## Review focus

- Predicted render time ahead of an idle edge, and outputs queried out of order:
  no rewind on hold, no query-order mutation while running (Task 1 tests).
- Hidden/no-output intervals and material replacement: the shared timeline
  records every activity edge even when no interested tile exists (Task 3 tests).
- A nonintegral rate or pause offset breaks bucket alignment: translated
  deadlines remain future instants; combined attention/Aurora wakes may be the
  union of both schedules (Task 2 tests, Tasks 4–5 captures).
- Video or backdrop updates while input remains idle: real damage advances the
  commit with held optics, then no optic cadence resumes (Task 2 tests, Task 5).
- Empty/truncated traces or missing markers look quiet: reject incomplete
  evidence, require live CPU/GPU controls, and distinguish missing lanes from
  passes (Task 4 analyzer tests, Task 5 verdict).

## File map and sequence

Paths in this document are source-root relative. The draft lives in
`.worktrees/material-0db905`; execution records the worktree it actually uses.

| Task | Files | Responsibility |
| --- | --- | --- |
| 1 | `src/activity.rs`, `src/animation/clock.rs` | pure timeline arithmetic, shared storage, clock tests |
| 2 | `src/render_helpers/material/optics/{mod,aurora,noise,saturation,iridescence}.rs`, `src/layout/tile.rs`, `src/render_helpers/material/mod.rs` | logical frames, registry deadlines, render recording, damage tests |
| 3 | `src/layout/mod.rs`, `src/layout/tests.rs`, `src/layout/tile.rs`, `src/niri.rs`, `src/tests/attention_idle.rs`, `src/tests/signal.rs` | edge propagation, interest, markers and lifecycle tests |
| 3 | `docs/materials/{material-config,render-pipeline,adding-an-optic}.md`, `docs/specs/2026-09-18-ring-focus-motion-design.md` | describe the shipped threshold and time contracts |
| 4 | new `docs/materials/scripts/optic-settling-smoke.sh`, new `tools/optic_settling.py`, new `tools/test_optic_settling.py`; existing Aurora smoke and shared smoke helper | bounded capture driver, offline verdict, explicit cadence controls |
| 5 | new `docs/materials/2026-09-30-optic-settling-evidence.md`; accepted spec/plan status and execution task notes | pilot, matrix, visual review, evidence and disposition |

Tasks 1–5 are sequential. Native execution is recommended: the renderer,
clock and activity interfaces are closely coupled; parallel implementation
would create overlapping edits. Use a whole-change review before integration.

Execution records, in order: `material-a1d7da`, `material-5bf79c`,
`material-6e3bda`, `material-c94dd7`, `material-2ee11e`. Their dependency chain
starts at `material-0db905`; none is ready while this plan awaits review.

### Task 1: Add the shared pausable optic timeline

**Files:** modify and test `src/activity.rs`, `src/animation/clock.rs`.

**Interfaces:** add these crate-local types/accessors. Existing animation
methods retain their behavior; clones share the added state.

```rust
// src/activity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpticTime {
    pub real_now: Duration,
    pub logical_now: Duration,
    pub running: bool,
    pub real_anchor: Duration,
    pub logical_anchor: Duration,
}

#[derive(Debug)]
pub(crate) struct OpticTimeline {
    real_anchor: Duration,
    logical_anchor: Duration,
    running: bool,
    rendered: Option<Duration>,
}
```

`OpticTimeline::new(now: Duration) -> Self` starts running with both anchors
equal to `now`, no render record. Add `sample(&self, now: Duration) -> OpticTime`,
`record_render(&mut self, logical_now: Duration)`, and
`set_active(&mut self, active: bool, now: Duration) -> Option<OpticTime>`.
The last returns the post-edge snapshot, or `None` for a repeated state.
On `Clock`, expose `optic_time(&self, now: Duration) -> OpticTime`,
`record_optic_render(&self, logical_now: Duration)`, and
`set_optic_active(&self, active: bool, now: Duration) -> Option<OpticTime>`.

- [ ] Add the failing pure timeline test in `activity.rs` (existing test module
  already imports `Duration`); run `just test-one -p niri optic_timeline`.

```rust
#[test]
fn optic_timeline_holds_and_resumes_without_catchup() {
    let ms = Duration::from_millis;
    let mut t = OpticTimeline::new(Duration::ZERO);
    assert_eq!(t.sample(ms(90)).logical_now, ms(90));
    assert_eq!(t.sample(ms(20)).logical_now, ms(20)); // query order is pure
    t.record_render(ms(110)); // predicted presentation beyond the event
    assert_eq!(t.set_active(false, ms(100)).unwrap().logical_now, ms(110));
    assert_eq!(t.sample(ms(1_300_000)).logical_now, ms(110));
    assert!(t.set_active(false, ms(1_300_000)).is_none());
    assert_eq!(t.set_active(true, ms(1_300_000)).unwrap().logical_now, ms(110));
    assert_eq!(t.sample(ms(1_300_040)).logical_now, ms(150));
    assert!(t.set_active(true, ms(1_300_040)).is_none());
}
```

- [ ] Implement running sample as `logical_anchor + now.saturating_sub(real_anchor)`;
  paused sample returns `logical_anchor`. Recording while running takes the
  maximum rendered logical time; recording while paused changes nothing.
  `set_active(false, now)` holds `max(sample(now).logical_now, rendered)`;
  `set_active(true, now)` preserves held logical time and sets `real_anchor = now`.
  Reset `rendered` for the new active interval. Repeated states return before
  changing anchors. These are operations on explicit timestamps only.
- [ ] Store `OpticTimeline` inside `AdjustableClock`, initialized with the same
  `time` already obtained in `AdjustableClock::new`. Access it through the
  existing `Rc<RefCell<_>>`; do not add a constructor parameter or change
  `LazyClock`, `Clock::now`, rate adjustment, or complete-instantly semantics.
  Each accessor delegates through the existing inner borrow, for example:

```rust
pub(crate) fn optic_time(&self, now: Duration) -> OpticTime {
    self.inner.borrow().optic_timeline.sample(now)
}
```

- [ ] Add clock tests: two clones share a pause; a new `Clock::with_time` does
  not; a sample without `record_optic_render` cannot raise the hold; a second
  active interval resets the render maximum. In one test set animation rate
  to 0.5, sample ordinary animation time before/after an optic pause, and compare
  it with an otherwise identical clock that was never paused. Toggle
  complete-instantly and prove the optic timeline still uses explicit raw time.
- [ ] Run `just test-one -p niri optic_timeline` and
  `just test-one -p niri animation::clock`; then fast tests and commit gates.
  Commit: `feat(material): add a shared pausable optic timeline`.

### Task 2: Route optic values and deadlines through logical time

**Files:** all five optic Rust files in the map, `src/layout/tile.rs`, tests in
`src/render_helpers/material/mod.rs`.

**Interfaces:** consume Task 1's clock accessors and `OpticTime`. Rename
`OpticFrame.now` to `logical_now`; `Optic::next_change` returns logical time.
Add `optics::next_logical_change(glass: &ResolvedGlass, frame: &OpticFrame<'_>)
-> Option<Duration>`. Change registry `next_change` to take a third argument
`time: &OpticTime`; this is the sole logical-to-real deadline boundary.

- [ ] Add a registry test using an Aurora amount 0.5, full motion, drift 4 Hz,
  nonzero seed and real/logical anchors 10 s/100 ms. With real time 10 s,
  logical time is 100 ms, logical deadline is 250 ms and real deadline is
  10.150 s. Pause the same timeline and require no registry deadline while
  `next_logical_change` still reports 250 ms. This test must fail before the
  registry gains the new contract. Run `just test-one -p niri optic_settling`.
- [ ] Move the existing registry minimum into `next_logical_change`; implement:

```rust
pub(crate) fn next_change(
    glass: &ResolvedGlass,
    frame: &OpticFrame<'_>,
    time: &OpticTime,
) -> Option<Duration> {
    if !time.running {
        return None;
    }
    next_logical_change(glass, frame)
        .map(|boundary| time.real_anchor + (boundary - time.logical_anchor))
}
```

  Keep frame/snapshot construction together so their logical samples agree.
  Rename `ctx.now` in Aurora's two helper calls and every `OpticFrame` literal
  in the other optics' tests. Do not expose anchors or running state to entries.
- [ ] In `Tile::optic_frame`, interpret the time argument as logical time and
  name it accordingly. In `material_dynamics`, sample once from the raw tile
  clock, record only this render sample, and pass its logical time to optics:

```rust
let time = self.clock.optic_time(now);
self.clock.record_optic_render(time.logical_now);
let optics = optics::values(glass, &self.optic_frame(material, time.logical_now));
```

  In `tick_deadline`, sample using its supplied `now`, build the frame from that
  sample, and pass both to registry `next_change`. Leave the attention deadline
  on real `now`. Scheduling and interest must never record a render sample.
- [ ] Add exact deadline/phase assertions at the boundary and 1 ns before it;
  repeat with drift 2.3, reduced motion, a period boundary and several pauses.
  For every registry entry, configure non-neutral values, hold logical time
  fixed and vary paused real samples across multiple periods: compare entire
  uniform vectors. Test amount/drift zero, motion off and animations off.
  Assert registry minimum/conversion agrees with the minimum of individually
  mapped logical deadlines. Direct optic deadlines remain logical while paused.
- [ ] Extend `an_unfocused_signal_free_aurora_tile_reports_its_next_bucket`
  using its existing `material_tile`, `frame_for`, and `render_dynamics` helpers.
  Require held dynamics across time advances, a `None` deadline while paused,
  and unchanged slab visibility rejection. Resume with the explicit timestamp
  and require the translated boundary. Keep all these tests virtual-time only.
- [ ] Extend the existing `optic_values_are_damage` test using its `fingerprint`
  and `MaterialState::advance_commit` helpers. Supply uniforms from the paused
  registry, not fabricated different values. The core assertions are:

```rust
let first = state.advance_commit(RenderTarget::Output, held.clone());
assert_eq!(state.advance_commit(RenderTarget::Output, held.clone()), first);
let mut client_changed = held.clone();
client_changed.window = commit_after(2);
assert_ne!(state.advance_commit(RenderTarget::Output, client_changed), first);
```

  Build `held` with the existing `fingerprint(1, 1, &background_id, &backdrop_id)`
  and set its `optics` to the sampled vector. Repeat for background/backdrop
  commits and ordinary config replacement; verify held optics stay equal.
- [ ] Run focused `optic_settling`, `aurora`, `optic_values_are_damage`, and
  `unfocused_signal_free_aurora` filters through `just test-one -p niri`;
  run fast tests and commit gates. Commit:
  `feat(material): freeze optic values and translate registry deadlines`.

### Task 3: Connect activity edges and preserve lifecycle behavior

**Files:** activity/layout/Niri tests and product docs listed in the file map.

**Interfaces:** change `Layout::set_input_active` to
`(&mut self, active: bool, now: Duration) -> bool`; return value remains redraw
interest. Add `Tile::activity_gated(&self) -> bool` as the OR of existing
`attention_gated()` and registry `next_logical_change(glass, &frame).is_some()` for the
tile's current material. Keep attention's predicate intact.

- [ ] Add a virtual layout test starting with `Clock::with_time(Duration::ZERO)`
  and no outputs/windows. Its executable assertions are:

```rust
let clock = Clock::with_time(Duration::ZERO);
let mut layout: Layout<TestWindow> = Layout::with_options(clock.clone(), Options::default());
assert!(!layout.set_input_active(false, Duration::from_secs(1)));
assert_eq!(clock.optic_time(Duration::from_secs(100)).logical_now, Duration::from_secs(1));
assert!(!layout.set_input_active(true, Duration::from_secs(100)));
assert_eq!(clock.optic_time(Duration::from_secs(101)).logical_now, Duration::from_secs(2));
```

  Run `just test-one -p niri optic_settling` and confirm failure before wiring.
- [ ] After the existing same-state early return, update the shared timeline
  before scanning any tile. Feed the same supplied timestamp to the clock.
  Reuse the current workspaces-plus-interactive-move iteration and change its
  predicate to `Tile::activity_gated`; do not restrict propagation to visibility.
  The tile predicate uses its current logical frame without changing timeline
  state. A paused Aurora remains interesting through `next_logical_change`.
- [ ] Emit the transition-only Tracy message at this shared edge integration
  point, after `set_optic_active` returns a snapshot. Use a comma-free payload
  so the retained CSV exporter can preserve it without quoting ambiguity:

```rust
if let Some(client) = tracy_client::Client::running() {
    client.message(&format!(
        "OpticTimeline active={} real_ns={} logical_ns={}",
        u8::from(active), now.as_nanos(), time.logical_now.as_nanos(),
    ), 0);
}
```

  Here `time` is the returned `OpticTime`. Same-state calls return before this
  block. No per-frame message, trace-triggered redraw or new timer is added.
- [ ] Update all three Niri edge paths. In `notify_activity`, the timer's
  `poll` closure, and `set_input_idle_threshold`, capture a local
  `let now = get_monotonic_time();` once and pass it to both detector operation
  and `layout.set_input_active(active, now)`. Preserve timer token replacement,
  startup, delay calculation and redraw coalescing. Do not change TTY/unlock or
  power-on callers: their existing notification distinction is the contract.
- [ ] Extend virtual layout/tile tests using the existing `TestWindow` and
  operation helpers for hidden workspace/tab, interactive move, no outputs,
  output remove/re-add and new tile while idle. Exercise several missed render
  cycles, material replacement and full/reduced/off switches; all tiles retain
  the same shared time modulo their seed. Assert an Aurora-only, unfocused,
  signal-free tile returns interest on both edges; static/policy-off scenes
  with no attention return false. Repeated edges change neither clock nor interest.
- [ ] Extend `src/tests/attention_idle.rs` with an Aurora material fixture.
  Reuse its real sleeps, dispatch, virtual pointer and timer-token assertions:
  startup settles; pointer resumes; threshold reload lowers/raises/disables;
  the gate re-engages. Keep `an_idle_edge_that_changes_no_tile_queues_no_redraw`
  unchanged and passing. Use output redraw states/counters already in that file.
  Do not freeze the fixture clock. Run `just test-one -p niri attention_idle`.
- [ ] Use real-time Niri fixture checks for IPC power-on while idle versus
  `notify_activity` on resume/unlock; use virtual layout tests to pin held versus
  resumed deadlines. Actual backend activation/unlock delivery remains a Task 5
  capture requirement. Exercise inhibitor/screencast state without an input
  notification and require the timeline to remain paused. Run existing signal
  tests for level/accent, attention and finite impulses; add idle cases for
  finite beam/impulse completion, without virtual time spanning a Niri edge.
- [ ] Update the three material guides and attention design: shared 30 s
  threshold and 0 escape hatch, logical-only optic frame/deadline contract,
  registry pause/conversion, reading/video caveat and distinct attention resume.
  Mark future behavior as shipped only in this implementation commit.
- [ ] Run focused activity/layout/signal/beam tests through `just test-one`,
  fast tests and commit gates. Commit:
  `feat(material): settle Aurora on existing input activity edges`.

### Task 4: Build a bounded settling capture and offline verdict

**Files:** create the driver/analyzer/test files in the file map; modify
`docs/materials/scripts/glass-aurora-smoke.sh` and, only for explicit cadence
configuration, `docs/materials/scripts/glass-optic-smoke-lib.sh`.

**Interfaces:** driver modes `prepare`, `pilot`, `matrix`, with
`--lane headless|dedicated` (default headless); environment
`OUT`, `NIRI_BIN`, `CAPTURE_TASK`, `NIRI_MATERIAL_WORK_ROOT`, and `PILOT_DIR`
for matrix mode. `prepare` validates
configs and writes the planned inventory without launching a compositor.
Runtime snapshots the explicit `NIRI_BIN`; require a provenance-matched
`profile-with-tracy` binary. Analyzer entry point:
`python3 tools/optic_settling.py RUN_DIR`, writing `analysis.json` and returning
nonzero for a failed required case or invalid evidence. Its importable
`analyze_run(run: Path) -> dict` supplies the same verdict to offline tests.
Missing hardware lanes are recorded as `unverified` and prevent a complete
verdict, while a complete pilot can pass the subset declared for its lane.

- [ ] Reuse native smoke helpers for config generation, nested launch, screenshots,
  ROIs, Tracy port/tool selection, preflight and identity. Read the existing
  `niri-experiments` branch `results/idle-budget` fixture/analyzer for its
  duration-aware capture, successful-action journal and cleanup patterns;
  keep changes in this repository. Do not import executable experiment scripts
  as libraries or run their default matrix.
- [ ] Write one focused stdlib unittest module for the analyzer before its
  implementation. Define `parse_edges(rows) -> list[dict]` accepting
  `csv.DictReader` message rows and rejecting malformed/duplicate/invalid
  transitions. Define `check_window(cpu_ns, gpu_ns, start_ns, end_ns,
  max_redraws, max_material_draws) -> None` raising `ValueError` on violations.
  Concrete assertions include:

```python
rows = [
    {'MessageName': 'OpticTimeline active=0 real_ns=100 logical_ns=110', 'total_ns': '10'},
    {'MessageName': 'OpticTimeline active=1 real_ns=900 logical_ns=110', 'total_ns': '810'},
]
edges = parse_edges(rows)
self.assertEqual([e['logical_ns'] for e in edges], [110, 110])
check_window([], [], 20, 800, 0, 0)
with self.assertRaises(ValueError):
    check_window([30], [], 20, 800, 0, 0)
with self.assertRaises(ValueError):
    parse_edges(rows[:1] + rows[:1])
```

  Add run-level rejection tests for absent pause/resume, missing CSV headers,
  failed export, no active CPU/GPU positive control, missing/duplicate cases,
  short observation coverage, a third edge-flush redraw on one output, and a
  missing real-TTY lane falsely reported as verified. Run:
  `just --set one_cmd 'python3 -m unittest' test-one tools.test_optic_settling`.
- [ ] Implement the stdlib reducer using `csv`, integer nanoseconds and explicit
  `[start,end)` intervals. Sort zone events by start time: exports group zones,
  not global time. Retained Tracy supports `--messages` (`MessageName,total_ns`),
  `--unwrap` (including `name`, `ns_since_start`, `exec_time_ns`) and `--gpu`. Export all three
  with the helper's bounded `csvexport` function. Parse exact header names from
  the retained tool; no positional guessing. Match only `OpticTimeline` messages;
  ignore unrelated messages, but fail if required edges are absent. Use message
  `total_ns` for trace alignment; payload `real_ns` is not the trace time origin.
- [ ] Inventory all nine design §8 check families in `manifest.json` with lane,
  config, stimuli, repetitions, observation duration and required verdict. Record
  capture/trace endpoints and the existing `Niri::refresh_idle_inhibit` heartbeat
  used by idle-budget: endpoint distances and internal gaps must be at most 1.5 s.
  Require a complete interval and live active-draw controls.
  Zero zone rows alone never establishes coverage. Verify held logical time at
  pause/resume, cadence before/after, positive decoded pixel changes on real
  client/backdrop damage and unchanged Aurora-only crops while held.
- [ ] Keep transition accounting separate: in a static single-output case,
  count all redraws after the pause marker until resume. At most two are the
  edge flush; after the last allowed flush draw, the declared hold interval
  must contain zero redraws/material draws. Do not discard an arbitrary settle
  delay to hide a third recurring redraw. For multiple outputs, account per
  output or report the topology and total bound; finite stimuli have separately
  journaled intervals. Screenshots happen outside quiet observation windows.
- [ ] Add `pilot` with one short cycle per available check family, 5 s threshold
  and 5 s hold; the default-threshold case uses 30 s. Include a deliberately
  misaligned Aurora+breathe resume. `matrix` repeats 4/2 Hz cycles three times,
  covers the actual 30 s default and includes one 600 s held interval. Gate a
  matrix run on the passed manifest in `PILOT_DIR`, from the same binary and
  case-config identities (declared repetition/hold-duration changes are allowed).
  Hardware-only TTY/two-output cases require their own lane pilot; do not claim
  them from nested Weston. No automatic session stop or VT switch.
  Headless mode uses the existing nested launcher. Dedicated mode requires an
  operator-provided real TTY with the desktop stopped, uses dedicated preflight,
  and launches the identified compositor on that session; it must refuse an
  inherited Wayland/X11 display. Record output topology and expose the session
  resume/unlock steps for the operator to perform. Do not script a host switch.
- [ ] Own every PID, bound capture/export/waits, and install EXIT/INT/TERM cleanup
  that stops and waits for compositor, client, Weston and capture children before
  analysis/checksums and lock release. Reject reused OUT. Exercise prepare and
  cleanup with subprocess stubs in the same unittest module, including TERM
  during capture/export and a failed preflight. Preserve partial evidence.
- [ ] Make existing Aurora cadence smokes explicitly set `idle-after-ms 0`
  while preserving full/reduced/off variants and exactly one `signal` block.
  The external idle-budget configs already have their own gate policy: record
  it when reused, and require updating any old default-dependent fixture before
  treating its cadence verdict as current. Do not globally disable the new
  gate for the settling driver.
- [ ] Run tooling-focused checks and the tooling fast front door:
  `just --set fast_cmd 'python3 -m unittest discover -s tools 2>&1' test-fast`.
  Run `bash -n` on modified/new shell scripts, then commit gates. Commit:
  `test(material): add bounded optic settling capture checks`.

### Task 5: Run the pilot and matrix, review evidence and integrate

**Files:** evidence document in the map; spec/plan statuses and execution notes.
**Interfaces:** Task 4's driver modes and `analysis.json`; native capture protocol.

- [ ] Complete code review and required checks before captures. Build and snapshot
  the explicit execution source with `--release --features profile-with-tracy`,
  recording source commit, binary hash, features and config hashes. Resolve the
  Cargo target directory using existing build helpers; do not assume `target/`
  names the correct shared binary. Run `prepare` with that identified binary.
- [ ] Run the headless pilot through export, analysis, pixel comparison and
  cleanup. From the execution worktree, the new driver's contract is:

```sh
OUT="$NIRI_MATERIAL_WORK_ROOT/optic-settling/pilot-$(date +%s)" \
  CAPTURE_TASK="$SETTLING_CAPTURE_TASK" NIRI_BIN="$SETTLING_BINARY" \
  docs/materials/scripts/optic-settling-smoke.sh pilot --lane headless
```

  `SETTLING_CAPTURE_TASK` is this execution record's id; `SETTLING_BINARY` is
  the identified snapshot. Never use a shared launcher. Inspect `analysis.json`,
  message CSV, transition counts, liveness controls and cleanup before proceeding.
  On a load refusal, retain output and park with `--reason quiet`, the observed
  pilot duration plus the estimated matrix duration, and the required host lane.
- [ ] Run `matrix --lane headless` with a new OUT and `PILOT_DIR` pointing to
  the passed pilot only after pilot acceptance. Run the real-TTY
  and two-output lane pilots/matrices when the user supplies those environments;
  otherwise leave them unverified and their acceptance work open. A TTY backend
  resume and an unlock are different stimuli from IPC `power-on-monitors`.
  Input wake must resume; IPC power-on while idle must keep the field held.
  Record a `run:` note for each attempt, including refusal/abort/hang.
- [ ] Publish the nine-family verdict table with artifact hashes, source/binary
  identity, durations, topology and links to every missing/failed case. Include
  a phase-continuous idle/resume clip for owner judgment. Compare active 4/2 Hz
  controls with idle holds, and report the actual combined-motion wake union.
  Distinguish transition flush, finite stimuli, real client damage and idle-only
  draws. Do not infer a watt saving from zero draws; cite existing power evidence
  only as motivation, and state the negligible expected video-case saving.
- [ ] Obtain the visual judgment, fix any reproduced defects with focused tests,
  and re-run only affected checks plus required gates. Keep missing lanes and
  unresolved visual findings open. Once all required evidence is accepted, update
  spec/plan status, the broader settling task, and the resource-aware brief;
  complete execution records in their result commits. This does not retroactively
  turn the design task into an implementation task. Commit:
  `docs(material): record sustained optic settling evidence`.
- [ ] Use the repository finishing workflow for integration. This plan does not
  authorize publishing a PR or changing the installed compositor. Preserve the
  execution worktree while any host pointer refers to it; harvest timings before
  removing it.

## Plan review and handoff

Coverage: design §§3–4 → Tasks 1–2; §5 → Task 3; §6 → Task 3 tests and Task 5
hardware evidence; §7 → Tasks 1–3; §8 → Tasks 4–5; §9 → Task 3 docs and Task 5
follow-through. All five review-focus cases have an owning test/capture step.

After plan acceptance, finish `material-0db905` with a note on
`material-f86183` recording the accepted design and plan. Scope that existing
idea as the execution parent; its child tasks remain blocked until the design
task is done. Do not execute any implementation step during this planning task.
