# Sustained optic settling after input inactivity

**Status:** accepted in spec review round 4; implementation plan accepted in
plan review round 2. Execution Tasks 1–4 are complete; headless pilot and matrix
passed, and the owner accepted the idle/resume clip on 2026-10-02.
The [acceptance evidence](../materials/2026-09-30-optic-settling-evidence.md)
records five unverified lifecycle cases and their follow-up tasks.
The design task itself implemented and captured nothing.
**Task:** `material-0db905`; wakes `material-f86183` after design and plan review.
**Baseline:** `1767db05` (the subsequent task-start commit changes only the task).

## 1. Intent and boundary

After extended input inactivity, optional sustained material motion must stop
requesting frames and changing its uniforms. Visible client damage must still
render. Static level/accent indicators, finite focus beams, impulses, and the
existing attention contract remain intact. Input inactivity is independent of
the signal's Quiet level, window focus, and client activity.

This task delivers the reviewed design and then an implementation plan for
review. It does not implement, capture, install, or change Prism. The
[resource-aware rendering brief](../notes/2026-09-29-resource-aware-rendering-brief.md)
sets this boundary. The broader idea's phrase “easing back” is resolved here as
phase-continuous resume at the configured cadence, without an extra easing
animation; the owner accepted this decision in round 2.
An easing animation would add a deadline and a visual contract to conceal a
normal bucket step in a 600-second field; phase continuity is sufficient here.

## 2. What the code establishes

- `InputActivity` in `src/activity.rs` owns the existing detector. Its startup,
  observe, poll, and reload paths already implement `signal { idle-after-ms }`:
  30000 ms by default, 0 disables, maximum 3600000 ms.
- `Niri::{notify_activity,arm_input_idle_timer,set_input_idle_threshold}` feed
  activity edges into `Layout::set_input_active`. That method currently returns
  whether any tile is `attention_gated`; this controls the edge redraw. It must
  also recognize an Aurora-only scene, including an unfocused, signal-free tile.
- The registry in `src/render_helpers/material/optics/mod.rs` contains saturation,
  noise, Aurora, and iridescence. Only Aurora overrides `next_change`.
  `AuroraOptic::rate` currently reads amount, motion policy, and animations off,
  but no input activity. Its 600-second field steps on `phase_on` buckets.
- `Tile::optic_frame` feeds both `material_dynamics` and `tick_deadline`.
  Optic uniforms join `InputFingerprint.optics`; withholding a deadline alone
  does not freeze the values when some other damage causes rendering.
- `Monitor::update_render_elements` clears tile visibility and updates rendered
  workspaces. `Tile::render` reports deadlines only through the render path;
  `tick_deadline` checks the slab band against the view. The output collector
  takes the earliest deadline and `Niri::arm_signal_timer` replaces its timer.
  Screen captures have no independent optic timer.

The [ring-motion design](2026-09-18-ring-focus-motion-design.md) §§2–3 and
the [render pipeline](../materials/render-pipeline.md) describe the contracts
being retained. The [idle-budget evidence](../materials/2026-09-11-idle-budget-evidence.md)
establishes finite-motion quiescence and active Aurora cadence, not the new
settling behavior. Its old cadence fixtures must explicitly disable the idle
gate or provide real input once this behavior lands.
The 2026-09-29 power re-run on `48ba40a1`, recorded in that evidence document
on `materials-26.04` by `material-39a46f`, measured Aurora above resting jelly
at +0.96 W for 4 Hz and +0.77 W for 2 Hz (upper estimates 1.37 W and 1.03 W).
These static-scene measurements motivate settling; they do not measure the
proposed implementation's savings.

## 3. Participation and configuration

**Default-on for Aurora, using the existing threshold; accepted by the owner in
round 2.** An Aurora with amount
greater than zero and a nonzero effective drift rate participates automatically.
No new configuration field, per-window override, second detector, or Prism key
is introduced. `idle-after-ms 0` disables input settling for both attention and
Aurora. This deliberately offers no independent “keep Aurora moving but settle
attention” switch.

Consequences to accept with this default: Aurora freezes after 30 seconds of
reading or watching without activity notifications, even while the client keeps
updating. Idle inhibitors and screencasts do not keep it moving: inhibitors feed
the idle-notifier protocol, not `InputActivity`. Despite its location under
`signal`, `idle-after-ms` now governs a material optic even on signal-free windows.
Client rendering and capture continue; only optional sustained optic time stops.
During video or other constant client damage, the glass effect redraws anyway:
freezing Aurora is primarily a visual change, with little expected saving.
The intended saving comes from static scenes where optic deadlines cause the draws.

The other registered optics are already static. Focus beams and their brightness
noise, crossfades, jelly springs/ripples, and signal impulses remain on their
existing finite paths. Attention remains gated by `effective(..., input_active)`
and resumes on its existing absolute clock. No change to client frame callbacks,
signal expiry, idle inhibitors, or the callers of `notify_activity`. Those callers
include session resume and unlock as well as input (§6).

| Aurora configuration | While input active | While input idle |
| --- | --- | --- |
| amount > 0, drift > 0, motion full | configured bucket rate | held phase, no optic deadline |
| same, motion reduced | half bucket rate | held phase, no optic deadline |
| motion off or animations off | phase 0, no optic deadline | phase 0, no optic deadline |
| amount 0 or drift 0 | phase 0, no optic deadline | phase 0, no optic deadline |

“Effective rate” here retains `drift_rate`'s policy mapping. Input inactivity
does **not** make that rate zero for uniform evaluation: `phase_on(..., 0, ...)`
returns phase 0 and would replace the current image at every idle edge.

## 4. Phase and time

Use one small pausable optic timeline carried by the existing shared `Clock`
handle, derived from its unadjusted monotonic time. Niri, layout, and all its tiles
already share that handle, including hidden and interactive-move tiles. Keep the
timeline state in the shared inner allocation so cloning `Clock` shares it;
constructing a fresh `Clock` initializes a fresh timeline. It lives independently
of replaceable config/options and material objects.
Only activity edges pause or resume it; visibility and material selection do not.
It owns no timer and does not decide whether input is idle. Its pause, sample,
and render-record operations are separate from ordinary `Clock::now`,
`now_unadjusted`, rate, and complete-instantly behavior: existing animation callers
and attention continue to see their current clocks.

The clock stores a logical-time anchor and its real-time anchor while running,
or a held logical instant while paused. It also records the greatest logical
sample used to build material dynamics during the current active interval.
At startup the logical and real anchors are equal, so
before the first idle period Aurora has its current absolute-time phase and
cross-window bucket alignment. Sampling while running is:

```
logical_now = logical_anchor + saturating_sub(real_now, real_anchor)
```

Running sampling is a pure function of anchors and the supplied real time:
sampling a later time first does not change an earlier time's result. Material
rendering records its sample separately; scheduling and interest queries do not.
At the active-to-idle edge, hold the maximum of `logical(edge_time)` and the
greatest recorded render sample (if any). At resume,
set `logical_anchor` to the held instant and `real_anchor` to the event's time.
The exact resume instant therefore returns exactly the held phase. Subsequent
samples advance normally; there is no catch-up through the inactive interval,
reset to phase zero, replay of missed buckets, or additional easing deadline.
Reset the render record for the new active interval. Repeated edges with the
same state do nothing.

The hold-only high-water record matters because `Niri::redraw` temporarily samples
the predicted presentation time, which can be ahead of the next event-loop time.
An idle edge must not rewind past a logical instant already used for material
rendering. It does not clamp running samples or change the ordinary animation
clock. Tests cover backward raw samples, render recording, and two outputs sampled
out of order. Removing the record entirely is simpler but permits an idle-edge
step backward at a bucket boundary (one bucket is 1/2400 of the loop at 4 Hz
for the usual sub-bucket presentation-time skew); retain it to avoid that artifact.
The first frame after waking may show the normal advancement to its predicted
presentation instant, not elapsed idle time.

Rename `OpticFrame.now` to `logical_now`. This is the only time exposed to
`Optic::values` and `Optic::next_change`; real time, anchors, and running state
never reach an optic. `Optic::next_change` returns a logical deadline, regardless
of whether the timeline is paused. Aurora keeps using the existing helpers:
`phase_on(AURORA_PERIOD, effective_rate, logical_now, seed)` for its value and
`next_boundary_on(AURORA_PERIOD, effective_rate, logical_now)` for its deadline.

The registry's `optics::next_logical_change(glass, frame)` takes the minimum of
the entries' logical deadlines. The scheduler-facing `optics::next_change`
also takes the coherent timeline snapshot, containing real sample time,
logical sample time, running state, and running anchors. It returns `None`
while paused. While running, it takes the earliest logical deadline and
converts it once, in the registry:

```
real_deadline = real_anchor + (logical_boundary - logical_anchor)
```

The mapping preserves deadline order while running. The result must be strictly
later than the snapshot's real sample time. Build the frame's `logical_now` from
that same snapshot, including under predicted-time skew. Passing a logical
deadline directly to the output timer is incorrect once the clock has paused.
Retain integer-duration arithmetic and the helper's ceiling division; do not
introduce a floating-point accumulated phase. Document the logical deadline
contract on `OpticFrame` and `Optic`; test all entries plus the registry's
pause suppression and conversion (§7). A future animated optic cannot omit
pause handling because that handling belongs to the registry.

An idle edge may advance from the previous displayed frame to the edge's normal
bucket once; that is the final active interval, not a phase reset. After that,
with unchanged config/seed, repeated optic values are identical. A config change
can deliberately change the image as described below.

After a pause, Aurora boundaries move off the absolute clock grid. Attention
retains its absolute grid, so the output's combined wakeups can be the union of
both schedules rather than the faster schedule alone. For example, 4 Hz Aurora
and 8 Hz breathe can require up to 12 bucket wakes per second before presentation
coalescing, instead of 8 when aligned. Accept this active-time trade-off for phase
continuity and unchanged attention semantics; do not claim wake coalescing or
net energy savings without measuring. Include one deliberately misaligned combined
case in the pilot and full matrix (§8). Re-aligning Aurora or pausing attention
would change the selected phase or attention contract and is outside this design.

## 5. Activity edges, rendering, and wake bounds

`InputActivity` remains the authority. Production keeps `get_monotonic_time()`
for detector startup, observe, poll, reload, and timer-delay calculation.
At each edge, Niri passes the same timestamp used for observe/poll/reload
explicitly to the layout and timeline pause/resume operations; those operations
never fetch time themselves. Do not use `clock.now_unadjusted()` as the detector's
event timestamp: it may contain a cached or predicted presentation time from
an earlier redraw in the same event-loop pass. Rendering retains that existing
prediction behavior, with the hold record handling its skew as described in §4.

Edge timestamps and render samples must share a time domain. Timeline and layout
tests use `Clock::with_time` and explicit virtual times for pause, resume, and
sampling, including zero-start and full-cycle cases. Niri-level fixture tests
keep real time, following `src/tests/attention_idle.rs`: calloop timers and the
detector remain on the real clock. These tests must not use `set_unadjusted`
to freeze or advance time across an activity edge. This prevents a real event
anchor meeting a virtual render sample and `saturating_sub` hiding the mismatch.
No Niri time-source injection or virtual calloop timer mechanism is introduced.

Update the shared optic clock on **every** activity edge, before deciding whether any tile
needs a redraw. This must work even with no windows, no outputs, or no Aurora
currently configured. Do not discover idle history lazily at the next tile draw:
a hidden tile can miss several complete idle/resume cycles that way.

Broaden the current activity-interest query from attention alone to attention OR
`optics::next_logical_change(glass, &frame).is_some()`. Use the registry,
not an Aurora-specific test in layout. The probe uses the ordinary logical frame
with its material, motion policy, animation switch, seed, and logical sample;
only the presence of a logical deadline is consumed. No hypothetical running
snapshot is needed. The probe neither resumes the shared clock nor records a
render sample.
This query is independent of the idle flag on both edges; asking the actually
paused `next_change` whether a wake is necessary would always answer no. Keep attention's
existing predicate and the existing all-output redraw coalescing. No change is
required for an edge when all materials are static/neutral/policy-off and no
attention signal is affected.

For an affected scene, the idle edge queues at most one redraw request per
output through the existing coalescing path. Rendering gathers no Aurora deadline
and the existing timer replacement removes its arm. A callback already dispatched
at the edge may coalesce with that redraw; a skipped/DPMS-off render may leave at
most the previously armed one-shot callback. Neither may start a repeating idle
chain. Resume similarly requests one redraw per output and reconstructs future
bucket deadlines. Repeated active input does not queue optic edge redraws.

Acceptance is zero optional optic deadlines and constant optic fingerprints
throughout the settled interval, with zero recurring material redraws in an
otherwise static scene. A bounded transition flush is measured separately.
No-op edges in a scene without affected motion must retain the existing zero
redraw behavior that fixed `material-cd7deb`.

Real client or backdrop damage still changes the ordinary fingerprint and draws
the current content with held Aurora uniforms. Finite effects may render and
finish while input remains idle. This is not a promise of zero GPU work for an
animated client, animated wallpaper, cursor, or another live compositor effect.

## 6. Lifecycle and visibility

| Event | Result |
| --- | --- |
| Startup, no input | Existing timer pauses the optic clock after the threshold; Aurora settles even if no input was ever received. |
| Threshold reload: lower below elapsed quiet time | Pause now, on the existing recomputed idle edge; do not rewind to the historical threshold. |
| Threshold reload: raise above elapsed quiet time, or set 0 | Resume from held time; 0 also cancels the detector timer as today. |
| Threshold reload stays on the same side | Preserve anchors and phase history; only the detector's next check changes. |
| Color/amount/material/focus-split reload | Apply configuration through ordinary damage. Shared time survives material replacement. A newly enabled Aurora while idle is static at the held timeline and its own seed. |
| Change drift rate or full/reduced policy | Re-evaluate the same logical instant on the new bucket grid. A grid-sized phase change is an explicit configuration result, not an idle-resume jump. |
| Set motion off, animations off, drift 0, or amount 0 | Preserve today's phase-0/no-deadline behavior. Re-enabling evaluates the shared timeline; it does not restore a private phase from before that explicit policy/config change. |
| Hidden workspace/tab or slab out of view | Existing visibility gates schedule no frames for it. Logical time advances while input is active and freezes while idle, even while hidden. Revealing it renders the current logical phase, without replay. |
| Overview or workspace transition | Every actually rendered slab uses the same timeline and existing view gates, regardless of keyboard focus. The transition itself remains finite and independent. |
| One output removed/disabled; another lit | Do not pause the shared clock because an output disappears. The lit output retains active cadence or idle holding according to global input state. |
| Global DPMS off | No material draws through the existing backend gate. Activity detection continues; the timeline pauses if the threshold expires. |
| Power-on action without input | `Action::PowerOnMonitors` in `src/input/mod.rs`, invoked through IPC, calls `activate_monitors` without `notify_activity`. While idle it draws held Aurora and arms no optic timer. |
| DPMS wake by input | Existing input handling calls `notify_activity` and resumes the timeline; power-on redraw samples it. |
| TTY backend session resume | `src/backend/tty.rs` calls `notify_activity` on session activation, even without an input event. An idle timeline resumes and can settle again after the threshold. |
| Session unlock | `SessionLockHandler::unlock` in `src/handlers/mod.rs` activates monitors and calls `notify_activity`. It resumes an idle timeline without requiring a separate input event. |
| Tile creation, output hotplug, window move, capture | Reuse the existing shared `Clock` and its timeline. No clock reset, private missed-edge accounting, or capture-only wake timer. |

Visibility does not promise general occlusion detection. A completely covered
window that still reaches the current render path is the attribution work in
`material-d09741`. Do not broaden this design into a new visibility system.

## 7. Deterministic verification required of implementation

Use the existing Rust tests and `just` front door; no new testing framework.
Use `just test-one -p niri <filter>` for focused work, then `just test-fast`,
plus the required commit/push gates. `test-one` is present on `materials-26.04`;
the eventual implementation must include that updated test front door.

1. **Clock and deadline translation.** Pause at a nonzero, non-period-aligned
   phase, hold across multiple 600-second periods, and resume. Assert exact held
   values, no idle deadline, and the first changed value at the translated future
   boundary (unchanged one nanosecond before). Cover repeated cycles, repeated
   same-state calls, nonintegral valid drift rates, reduced motion, a bucket/period
   boundary, and predicted timestamps ahead of or behind the edge timestamp.
   Assert that out-of-order running queries produce the same values for the same
   timestamps, while the idle hold retains the greatest recorded render sample.
   Queries alone must not alter the hold record or the ordinary animation clock.
   Timeline and layout tests drive pause, resume, and samples with explicit
   virtual timestamps on `Clock::with_time`; include a zero-start clock and a
   full active/idle/resume cycle.
2. **Participation and uniform identity.** Iterate over every entry in `OPTICS`,
   with non-neutral configurations and a nonzero seed. Hold logical time fixed
   while varying real sample time across buckets and periods in paused timeline
   snapshots; build each frame from its snapshot and compare every entry's
   complete uniform vector. Require no scheduler-facing registry deadline;
   individual animated entries may still return logical deadlines. Exercise
   every row in §3 and verify registry-derived interest through
   `next_logical_change`, without mutating shared state. Check the earliest
   logical deadline maps to the earliest real deadline while running.
   Compare complete vectors, not just amplitude.
   Static/neutral cases retain constant values and no deadlines. Test rate/config
   reloads separately from input resume.
3. **Tile and material integration.** An unfocused, signal-free Aurora tile
   wakes on an idle-to-active edge; an Aurora-only scene settles. Tile deadline
   agrees with uniform changes. `InputFingerprint.optics` and material commit
   remain unchanged across idle samples with identical inputs; client/background
   commit changes still advance material damage with identical held optics.
4. **Hidden-state propagation.** A tile omitted from render updates through
   several activity cycles later samples the same clock as a continuously visible
   tile, modulo its seed. Include an interactive-move tile, a new tile created
   while idle, config replacement, no-output state, and output removal/re-add.
5. **Scheduler edges.** Exercise the existing activity timer startup and reload
   paths plus registry-derived optic interest. Niri-level fixtures use real time
   and real calloop timers as in `src/tests/attention_idle.rs`; no test may use
   `set_unadjusted` across an activity edge. Explicit virtual-time coverage
   belongs to the timeline and layout tests in item 1.
   Check no-op scenes, one-shot
   replacement, stale edge callback, repeated input, and wake rearming. An idle
   scene cannot repeatedly rearm a past/logical deadline.
6. **Preservation.** Attention still resumes using its absolute clock and retains
   its level/accent behavior. Finite beam/impulse tests pass while input is idle;
   a signal-free Aurora must not require a signal cache. Existing view-bound and
   finite-animation tests retain their semantics. Exercise the power-on action,
   session-resume activity notification, and unlock as distinct paths. Verify that
   idle inhibition and screencast activity alone do not resume the optic timeline.

## 8. Pilot-first capture acceptance

Extend the existing idle-budget/signals fixtures and their capture protocol in
the eventual implementation work. No capture is run or claimed by this document.
Use an explicit worktree binary path or the fixture's binary override. Record
source and binary identity, configuration, thresholds, input/idle boundaries,
observation windows, Tracy redraw/material-draw counts, decoded pixels, verdict,
and cleanup. Use lane preflight and `tasks quiet` for runs requiring an idle host.

Add a Tracy message at each actual optic pause/resume transition, carrying
the edge's monotonic timestamp, direction, and held logical time (the resume
anchor on resume). Emit it at the shared transition point so input, timer,
reload, session-resume, and unlock edges are all covered, including when no
tile needs a redraw. Repeated same-state calls emit nothing. Use these markers
to align observation windows and measure the bounded edge flush; the configured
threshold and `Niri::notify_activity` span alone cannot identify an idle edge.
This is transition-only instrumentation, not a per-frame stream.

Before a full matrix, run the smallest end-to-end pilot: one cycle per check
below, with short explicit thresholds (for example 5 seconds) and short hold
windows. It must reach trace export, pixel comparison, verdict, and cleanup;
inspect its results before starting any longer run. Do not treat host GPU
utilization or a blank/missing trace as evidence of compositor quiescence.

| Check family | Cases and positive controls | Required verdict |
| --- | --- | --- |
| Active → idle → input resume | Lit Aurora at 4 Hz/full and 2 Hz/reduced; unfocused and no signal. Record active draws before each quiet window and a real virtual-pointer input afterward. | Expected active cadence, zero redraw/material draws after transition flush, cadence returns without catching up idle phase. |
| Combined sustained motion | Aurora plus breathe on one output; first aligned, then resume after an idle interval that offsets Aurora from attention's grid. | Record the union of deadlines, redraws and material draws; bounded by the summed bucket schedules with presentation coalescing, not assumed to remain at the faster rate. Both settle again. |
| Gate/policy controls | `idle-after-ms 0`; amount 0; drift 0; motion off; animations off; startup without input. | Gate-disabled positive control keeps moving; each static control stays quiet; ordinary input/damage proves the trace channel is live. |
| Client damage while settled | Change a known client region while idle; hold backdrop/static geometry fixed. Also exercise real backdrop damage. | Updated content appears, optic phase stays held, no optic cadence starts. Observe a quiet window afterward. |
| Finite effects and attention | Programmatic finite impulse/focus gain while idle, then no stimulus; attention active/idle/resume control. | Finite effect draws and ends, static level/accent persists, attention retains its prior resume contract. |
| Visibility | Hidden workspace, hidden tab, offscreen column; reveal by IPC while still idle; overview and transition. | No invisible-only cadence; revealed idle Aurora is held; active visible control animates. Account separately for transition frames. |
| Outputs/DPMS | Global DPMS off, then IPC `power-on-monitors` without input; separately wake with input. Disable/remove one output while another remains lit. | No off-output material draws; lit output respects global activity; only the power-on action without input leaves an idle field held. |
| Session activation/unlock | Exercise the TTY session-resume path on a real TTY and the session-lock unlock path separately from the power-on action. Nested/headless-weston runs cannot cover TTY resume; mark that case unverified unless the real TTY lane runs. Include an idle inhibitor and screencast-only control. | Resume/unlock notifications wake the timeline; inhibitor and screencast alone do not. Each resumed case settles again without further notifications. Deterministic §7 tests cover the logic even when the TTY capture is unavailable. |
| Reload and clock continuity | Raise/lower/disable threshold; unrelated reload; rate/policy/material changes while held. | §6 behavior; unrelated reload has no phase reset; config changes are distinguished from activity-driven changes. |

Count transition activity separately, with at most the queued redraw and an
already-dispatched one-shot per output; do not hide a repeating timer in a generous
settle exclusion. Quiet observation begins after that bounded flush and any
deliberate finite stimulus has ended. Screenshot requests themselves are stimuli:
take endpoint pictures outside the zero-redraw observation window. Full matrix:
repeat the active/idle/resume cases three times, test the actual 30-second default,
and include one 600-second held interval.

Phase verification combines deterministic uniform/deadline tests with decoded
Aurora-only crops and an owner-reviewed idle/resume clip. Full-frame pixel equality
is required only for unchanged static scenes, not for the client-damage or finite
effect cases. A two-output check records its actual topology; an unavailable lane
is unverified, not a pass. Isolated power measurements remain separate evidence;
zero compositor draws does not establish a particular watt saving.

## 9. Alternatives and follow-through

- **Chosen: shared paused logical time on the existing `Clock` handle.**
  Preserves phase and Aurora bucket alignment across windows without a timer per tile or
  another constructor argument. Keep the small timeline arithmetic in the
  fork-only `src/activity.rs`, with storage/access on `Clock` and no dependence
  there on material configuration. This adds a seam in `src/animation/clock.rs`,
  unchanged against the pinned baseline at review time, and gives a general clock
  one specialized timeline. Its narrow additive accessors leave existing methods
  intact; tests must prove pause does not affect finite animation or real time.
- **Rejected: a separate shared handle through layout constructors.** Cleaner
  separation from `Clock`, but expands plumbing through floating, monitor,
  scrolling, workspace, and tile code. The current
  [divergence report](../materials/upstream-divergence.md) lists all five as
  conflicting with upstream. One new localized clock seam is preferable to
  deepening that distributed constructor diff; layout and tile still need their
  actual activity/render integration edits whichever carrier is chosen.
- **Rate zero or deadline suppression alone.** Smaller patches, but the former
  resets the image and the latter changes it on client damage and jumps on resume.
- **Per-tile last-rendered phase and custom resume easing.** Can freeze precisely
  the last shown bucket, but adds per-tile history, missed-edge/lifecycle handling,
  and an extra animation contract. Not needed for a slow 600-second field.
- **Opt-in settling or a second idle threshold.** Adds configuration and leaves
  existing drifting Aurora installations continuously rendering by default.
  The existing threshold is the selected policy and escape hatch.

After owner review of this spec, write the implementation plan under
`docs/plans/`. It must cover the shared `Clock` timeline, the optic-frame/deadline path,
activity-interest integration, deterministic verification, capture fixture changes,
and the evidence/review gates. Implementation is later work, not an execution
phase of `material-0db905`; do not close that later work through this design task.

When implementation lands, update `material-config.md` (Aurora and signal motion),
`render-pipeline.md`, `adding-an-optic.md`, and the attention design's scope/status
references so they describe the new shared threshold and Aurora time contract.
The optic guide must specify logical-only frame time and logical deadlines;
pause suppression and conversion to real deadlines are registry responsibilities.
Retain attention's distinct resume rule. Existing capture cadence controls must
state their idle-gate configuration. Regenerate and stage the seam inventory with
`just upstream-report` after staging the implementation changes; `just check`
runs `upstream-report --check`. No parser migration or Prism rollout is needed.
At this task's completion, note the reviewed design and plan decisions on
`material-f86183` in the same commit as the result.
