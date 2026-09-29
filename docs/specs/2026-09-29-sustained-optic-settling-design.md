# Sustained optic settling after input inactivity

**Status:** draft for owner review; no implementation or captures in this task.
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
animation; this is a proposed decision for owner review.

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

## 3. Participation and configuration

**Default-on for Aurora, using the existing threshold.** An Aurora with amount
greater than zero and a nonzero effective drift rate participates automatically.
No new configuration field, per-window override, second detector, or Prism key
is introduced. `idle-after-ms 0` disables input settling for both attention and
Aurora. This deliberately offers no independent “keep Aurora moving but settle
attention” switch.

The other registered optics are already static. Focus beams and their brightness
noise, crossfades, jelly springs/ripples, and signal impulses remain on their
existing finite paths. Attention remains gated by `effective(..., input_active)`
and resumes on its existing absolute clock. No change to client frame callbacks,
signal expiry, idle inhibitors, or what `notify_activity` counts as input.

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

Use one small shared pausable optic clock per `Layout`, derived from the existing
unadjusted monotonic clock. All its tiles hold the same handle, including tiles
on hidden workspaces, tiles moved between outputs, and the interactive-move tile.
It lives independently of replaceable config/options and material objects.
Only activity edges pause or resume it; visibility and material selection do not.
It owns no timer and does not decide whether input is idle.

The clock stores a logical-time anchor and its real-time anchor while running,
or a held logical instant while paused. It also retains the greatest logical
sample it has returned. At startup the logical and real anchors are equal, so
before the first idle period Aurora has its current absolute-time phase and
cross-window bucket alignment. Sampling while running is:

```
logical_now = max(last_sample,
                  logical_anchor + saturating_sub(real_now, real_anchor))
```

At the active-to-idle edge, sample once and hold that logical instant. At resume,
set `logical_anchor` to the held instant and `real_anchor` to the event's time.
The exact resume instant therefore returns exactly the held phase. Subsequent
samples advance normally; there is no catch-up through the inactive interval,
reset to phase zero, replay of missed buckets, or additional easing deadline.
Repeated edges with the same state do nothing.

The high-water sample matters because `Niri::redraw` temporarily samples the
predicted presentation time, which can be ahead of the next event-loop time.
An idle edge must not rewind a phase already sampled for rendering. The clock
is sampled only for optic work; it never clamps or changes the ordinary animation
clock. Tests cover backward raw samples and two outputs sampled out of order.
The first frame after waking may show the normal advancement to its predicted
presentation instant, not elapsed idle time.

`OpticFrame` keeps real `now` for the scheduler and adds a coherent snapshot of
optic logical time, running state, and the running anchors. Aurora evaluates
`phase_on(AURORA_PERIOD, effective_rate, logical_now, seed)`. While paused its
`next_change` is `None`. While running, calculate the next logical boundary with
the existing integer bucket helper and translate it back to real time:

```
real_deadline = real_anchor + (logical_boundary - logical_anchor)
```

It must be strictly later than the supplied real `now`. The snapshot and deadline
must use the same anchors and logical sample, including under predicted-time
skew. Passing a logical deadline directly to the output timer is incorrect once
the clock has paused. Retain integer-duration arithmetic and the helper's ceiling
division; do not introduce a floating-point accumulated phase.

An idle edge may advance from the previous displayed frame to the edge's normal
bucket once; that is the final active interval, not a phase reset. After that,
with unchanged config/seed, repeated optic values are identical. A config change
can deliberately change the image as described below.

## 5. Activity edges, rendering, and wake bounds

`InputActivity` remains the authority. Niri passes the same explicit monotonic
timestamp used for observe/poll/reload to the layout edge update. Update the
shared optic clock on **every** activity edge, before deciding whether any tile
needs a redraw. This must work even with no windows, no outputs, or no Aurora
currently configured. Do not discover idle history lazily at the next tile draw:
a hidden tile can miss several complete idle/resume cycles that way.

Broaden the current activity-interest query from attention alone to attention OR
an Aurora with nonzero effective rate, evaluated independently of the current
idle flag. The latter condition is needed on both edges; asking the paused
`next_change` whether a wake is necessary would always answer no. Keep attention's
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
| Global DPMS off | No material draws through the existing backend gate. Activity detection continues; the timeline pauses if the threshold expires. A programmatic power-on is not input. |
| DPMS wake by input | Existing input handling resumes the timeline; power-on redraw samples it. A programmatic power-on while idle draws held Aurora and arms no optic timer. |
| Tile creation, output hotplug, window move, capture | Reuse the layout's clock state. No clock reset, private missed-edge accounting, or capture-only wake timer. |

Visibility does not promise general occlusion detection. A completely covered
window that still reaches the current render path is the attribution work in
`material-d09741`. Do not broaden this design into a new visibility system.

## 7. Deterministic verification required of implementation

Use the existing Rust tests and `just` front door; no new testing framework.
The current justfile has no `test-one`, so use `just test-fast` for focused work
until that recipe is adopted, plus the required commit/push gates.

1. **Clock and deadline translation.** Pause at a nonzero, non-period-aligned
   phase, hold across multiple 600-second periods, and resume. Assert exact held
   values, no idle deadline, and the first changed value at the translated future
   boundary (unchanged one nanosecond before). Cover repeated cycles, repeated
   same-state calls, nonintegral valid drift rates, reduced motion, a bucket/period
   boundary, and predicted timestamps ahead of or behind the edge timestamp.
2. **Participation and uniform identity.** Exercise every row in §3 with a
   nonzero seed. Compare the complete optic uniform vector, not just amplitude.
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
   paths plus Aurora interest. Check no-op scenes, one-shot replacement, stale
   edge callback, repeated input, and wake rearming. An idle scene cannot repeatedly
   rearm a past/logical deadline.
6. **Preservation.** Attention still resumes using its absolute clock and retains
   its level/accent behavior. Finite beam/impulse tests pass while input is idle;
   a signal-free Aurora must not require a signal cache. Existing view-bound and
   finite-animation tests retain their semantics.

## 8. Pilot-first capture acceptance

Extend the existing idle-budget/signals fixtures and their capture protocol in
the eventual implementation work. No capture is run or claimed by this document.
Use an explicit worktree binary path or the fixture's binary override. Record
source and binary identity, configuration, thresholds, input/idle boundaries,
observation windows, Tracy redraw/material-draw counts, decoded pixels, verdict,
and cleanup. Use lane preflight and `tasks quiet` for runs requiring an idle host.

Before a full matrix, run the smallest end-to-end pilot: one cycle per check
below, with short explicit thresholds (for example 5 seconds) and short hold
windows. It must reach trace export, pixel comparison, verdict, and cleanup;
inspect its results before starting any longer run. Do not treat host GPU
utilization or a blank/missing trace as evidence of compositor quiescence.

| Check family | Cases and positive controls | Required verdict |
| --- | --- | --- |
| Active → idle → input resume | Lit Aurora at 4 Hz/full and 2 Hz/reduced; unfocused and no signal. Record active draws before each quiet window and a real virtual-pointer input afterward. | Expected active cadence, zero redraw/material draws after transition flush, cadence returns without catching up idle phase. |
| Gate/policy controls | `idle-after-ms 0`; amount 0; drift 0; motion off; animations off; startup without input. | Gate-disabled positive control keeps moving; each static control stays quiet; ordinary input/damage proves the trace channel is live. |
| Client damage while settled | Change a known client region while idle; hold backdrop/static geometry fixed. Also exercise real backdrop damage. | Updated content appears, optic phase stays held, no optic cadence starts. Observe a quiet window afterward. |
| Finite effects and attention | Programmatic finite impulse/focus gain while idle, then no stimulus; attention active/idle/resume control. | Finite effect draws and ends, static level/accent persists, attention retains its prior resume contract. |
| Visibility | Hidden workspace, hidden tab, offscreen column; reveal by IPC while still idle; overview and transition. | No invisible-only cadence; revealed idle Aurora is held; active visible control animates. Account separately for transition frames. |
| Outputs/DPMS | Global DPMS off/on without input and with input; disable/remove one output while another remains lit. | No off-output material draws; lit output respects global activity; power-on alone does not resume an idle field. |
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

- **Chosen: shared paused logical time, existing detector and scheduler.**
  Preserves phase and window alignment without a timer per tile. Constructor
  plumbing follows the existing `Clock` route through layout/monitor/workspace
  into tiles; keep the small clock implementation with activity timing in
  `src/activity.rs`, not a general animation framework.
- **Rate zero or deadline suppression alone.** Smaller patches, but the former
  resets the image and the latter changes it on client damage and jumps on resume.
- **Per-tile last-rendered phase and custom resume easing.** Can freeze precisely
  the last shown bucket, but adds per-tile history, missed-edge/lifecycle handling,
  and an extra animation contract. Not needed for a slow 600-second field.
- **Opt-in settling or a second idle threshold.** Adds configuration and leaves
  existing drifting Aurora installations continuously rendering by default.
  The existing threshold is the selected policy and escape hatch.

After owner review of this spec, write the implementation plan under
`docs/plans/`. It must cover clock plumbing, the shared optic-frame/deadline path,
activity-interest integration, deterministic verification, capture fixture changes,
and the evidence/review gates. Implementation is later work, not an execution
phase of `material-0db905`; do not close that later work through this design task.

When implementation lands, update `material-config.md` (Aurora and signal motion),
`render-pipeline.md`, `adding-an-optic.md`, and the attention design's scope/status
references so they describe the new shared threshold and Aurora time contract.
Retain attention's distinct resume rule. Existing capture cadence controls must
state their idle-gate configuration. No parser migration or Prism rollout is needed.
At this task's completion, note the reviewed design and plan decisions on
`material-f86183` in the same commit as the result.
