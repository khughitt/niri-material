# Bounded focus and attention ring motion: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The focused window's ring runs one eased lap of its travelling light on focus gain and then costs nothing; sustained attention motion stops while the user is idle and resumes in step on input.

**Architecture:** The sweep is a per-tile start instant plus a snapshotted duration, sampled by a pure phase function on the animation loop (the path the focus crossfade already uses), replacing the bucketed drift clock and its deadline arm. Input activity is one compositor-wide state on `Niri` (`src/activity.rs`, pure and unit-tested) that reaches `signal::effective` through the `update_render_elements` chain and forces sustained motion to `Static`. Config retires `ring-drift-hz` with a named parse error, adds `ring-sweep-ms` and `signal { idle-after-ms }`.

**Tech Stack:** Rust (niri fork), `knuffel` KDL config, `calloop` timers, GLES shader untouched; bash smoke fixture with Tracy redraw counts.

**Spec:** `docs/specs/2026-09-18-ring-focus-motion-design.md` — the plan argues from it; read both.

**Task tracker:** `material-0e130e` carries this plan; each `### Task N:` heading is a step child. `tasks start <child>` before a task, `tasks done <child> "<what landed>"` in the task's final commit, `tasks check` before every commit. Tests run through `just` (`just test`, `just check`), never `cargo` directly. Every commit runs the pre-commit gate (`just check`), which includes `python3 tools/upstream-report --check`: when a commit adds or removes a tracked file, run `python3 tools/upstream-report` and stage `docs/materials/upstream-divergence.md` with it.

## Global Constraints

- `ring-sweep-ms`: integer, default `1500`, `0` disables, maximum `10000`, error `ring-sweep-ms must be at most 10000` (spec §4).
- `ring-drift-hz` in any response block is a parse error: `ring-drift-hz was replaced by ring-sweep-ms; see material-config.md` (spec §4).
- `signal { idle-after-ms }`: integer, default `30000`, `0` disables, maximum `3600000`, error `idle-after-ms must be at most 3600000` (spec §4).
- Sweep phase: `t = now.saturating_sub(started)`; `2π · (1 − (1 − t/D)³)` for `t < D`, exactly `0` at and after `D` (spec §1).
- Sweep starts only from rest, only under `motion full` with animations on and `focus ring-light`; it is cut to rest when any of those stops holding (spec §1).
- Settled focus reports no deadline, a constant fingerprint, and no transition (spec §2).
- Idle forces effective `motion` to `Static`; level, accent, and impulses are untouched; the sweep is not gated on idle (spec §3).
- The shader (`src/render_helpers/shaders/material/main.frag`) is not modified; the phase still arrives in `mat_sig_focus.y` (spec §6).
- The aurora optic's `drift-hz` and its use of `phase_on`, `next_boundary_on`, `drift_rate` are out of scope and must keep working (spec §1 Removed, Non-goals).
- No `Claude-Session`, co-author, or AI attribution trailers on commits.

---

## File map

| File | Responsibility after this plan |
| --- | --- |
| `niri-config/src/material/mod.rs` | `Response.ring_sweep_ms`, `ResolvedResponse.ring_sweep: Duration`, rejection of `ring-drift-hz` in `validate` |
| `niri-config/src/signal.rs` | `Signal.idle_after: Duration`, `SignalPart.idle_after_ms` |
| `niri-config/src/lib.rs` | `signal` merge arm validates the bound; tests |
| `src/render_helpers/signal.rs` | `sweep_phase`; `effective(.., input_active)`; `tick_deadline(eff, in_view, now)`; `FrameInputs.sweep`, `SignalFrame.sweep`, `sweep_q`; drift clock removed |
| `src/render_helpers/material/mod.rs` | reads `frame.sweep` into `mat_sig_focus.y` |
| `src/layout/tile.rs` | `FocusSweep` state, start and cut rules, phase into `FrameInputs`, transition report, `input_active` parameter |
| `src/layout/{mod,monitor,workspace,scrolling,floating}.rs` | thread `input_active` beside `is_active`; `Layout::set_input_active` |
| `src/activity.rs` (new) | `InputActivity`: pure idle state machine |
| `src/niri.rs` | owns `InputActivity` and its timer; `notify_activity` feeds it; reload recomputes |
| `docs/materials/material-config.md`, `render-pipeline.md`, `2026-09-02-material-signals-design.md`, `docs/specs/2026-09-05-ring-light-focus-response-design.md` | documentation per spec §8 |
| `docs/materials/scripts/material-signals-smoke.sh`, `glass-optic-smoke-lib.sh` | fixtures on `ring-sweep-ms`; new steady cases |

---

### Task 1: Configuration contract

**Files:**
- Modify: `niri-config/src/material/mod.rs:264-338` (Response, ResolvedResponse, with_overrides), `:745-755` (validate)
- Modify: `niri-config/src/signal.rs:24-45`
- Modify: `niri-config/src/lib.rs:222` (`"signal"` merge arm), tests near `:979` and `:855`
- Modify: `docs/materials/material-config.md:185-189, 239, 261, 282-300`
- Test: `niri-config/src/lib.rs` (`mod tests`)

**Interfaces:**
- Produces: `ResolvedResponse { ring_sweep: Duration, .. }`; `Signal { motion: SignalMotionPolicy, idle_after: Duration }`. `ResolvedResponse.ring_drift_hz` stays for one task, always at its default, so the workspace keeps compiling; Task 2 removes it with the drift clock.
- Tasks 2–3 read `response.ring_sweep` and `config.signal.idle_after`.

- [ ] **Step 1: Write the failing config tests**

In `niri-config/src/lib.rs` `mod tests`, replace `ring_drift_hz_is_zero_or_at_least_one` (line 979) with:

```rust
    #[test]
    fn ring_drift_hz_is_rejected_with_its_replacement() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-drift-hz 15; }; }"#,
        )]);
        assert!(
            err.contains("ring-drift-hz was replaced by ring-sweep-ms; see material-config.md"),
            "{err}"
        );
    }

    #[test]
    fn ring_sweep_ms_defaults_bounds_and_inherits() {
        let parsed = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" {}; }"#,
        )])
        .unwrap();
        assert_eq!(
            parsed.materials[0].resolve().response(None).ring_sweep,
            Duration::from_millis(1500)
        );

        let parsed = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-sweep-ms 0; }; response "still" {}; }"#,
        )])
        .unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!(m.response(None).ring_sweep, Duration::ZERO);
        assert_eq!(m.response(Some("still")).ring_sweep, Duration::ZERO, "inherits");

        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-sweep-ms 10001; }; }"#,
        )]);
        assert!(err.contains("ring-sweep-ms must be at most 10000"), "{err}");
    }

    #[test]
    fn signal_idle_after_ms_defaults_and_bounds() {
        let parsed = parse_files(&[("config.kdl", "")]).unwrap();
        assert_eq!(parsed.signal.idle_after, Duration::from_millis(30_000));

        let parsed = parse_files(&[("config.kdl", "signal { idle-after-ms 0\n}")]).unwrap();
        assert_eq!(parsed.signal.idle_after, Duration::ZERO);

        let err = parse_files_err(&[("config.kdl", "signal { idle-after-ms 3600001\n}")]);
        assert!(err.contains("idle-after-ms must be at most 3600000"), "{err}");
    }
```

Add `use std::time::Duration;` to the test module if it is not already imported there (check the top of `mod tests`).

- [ ] **Step 2: Run them to verify they fail**

Run: `just test -p niri-config ring_ 2>&1 | tail -20` — if `just test` does not take cargo args, run `just test 2>&1 | grep -E "ring_|idle_after|error\[" | head`.
Expected: compile errors on `ring_sweep` / `idle_after` (fields do not exist).

- [ ] **Step 3: Implement the response fields**

In `niri-config/src/material/mod.rs`:

```rust
// Response: replace the ring_drift_hz field
    /// Retired: decoded only so the error can name its replacement.
    #[knuffel(child, unwrap(argument))]
    pub ring_drift_hz: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub ring_sweep_ms: Option<u32>,

// ResolvedResponse: keep `ring_drift_hz: f64` for now with this doc, and add:
    /// Retired with the drift clock (Task 2); never set by config.
    pub ring_drift_hz: f64,
    /// Duration of the focus-gain lap; zero disables the sweep.
    pub ring_sweep: Duration,

// Default: keep `ring_drift_hz: 15.,` and add
            ring_sweep: Duration::from_millis(1500),

// with_overrides: replace the ring_drift_hz line with both
            ring_drift_hz: base.ring_drift_hz,
            ring_sweep: response
                .ring_sweep_ms
                .map_or(base.ring_sweep, |ms| Duration::from_millis(u64::from(ms))),
```

Add `use std::time::Duration;` at the top of the file if absent.

In `validate`, replace the `ring-drift-hz must be 0 or at least 1` block (and its comment) with:

```rust
        if self
            .responses
            .iter()
            .any(|response| response.ring_drift_hz.is_some())
        {
            return Err(String::from(
                "ring-drift-hz was replaced by ring-sweep-ms; see material-config.md",
            ));
        }
        if self
            .responses
            .iter()
            .any(|response| response.ring_sweep_ms.is_some_and(|ms| ms > 10_000))
        {
            return Err(String::from("ring-sweep-ms must be at most 10000"));
        }
```

- [ ] **Step 4: Implement the signal field**

In `niri-config/src/signal.rs`:

```rust
use std::time::Duration;

/// The top-level `signal { }` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub motion: SignalMotionPolicy,
    /// Input-idle threshold for sustained attention motion; zero disables the gate.
    pub idle_after: Duration,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            motion: SignalMotionPolicy::default(),
            idle_after: Duration::from_millis(30_000),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalPart {
    #[knuffel(child, unwrap(argument, str))]
    pub motion: Option<SignalMotionPolicy>,
    #[knuffel(child, unwrap(argument))]
    pub idle_after_ms: Option<u32>,
}

impl SignalPart {
    pub fn validate(&self) -> Result<(), String> {
        if self.idle_after_ms.is_some_and(|ms| ms > 3_600_000) {
            return Err(String::from("idle-after-ms must be at most 3600000"));
        }
        Ok(())
    }
}

impl MergeWith<SignalPart> for Signal {
    fn merge_with(&mut self, part: &SignalPart) {
        if let Some(motion) = part.motion {
            self.motion = motion;
        }
        if let Some(ms) = part.idle_after_ms {
            self.idle_after = Duration::from_millis(u64::from(ms));
        }
    }
}
```

In `niri-config/src/lib.rs`, replace `"signal" => m_merge!(signal),` with an arm that validates, modelled on the `"material"` arm below it:

```rust
                "signal" => {
                    let part = SignalPart::decode_node(node, ctx)?;
                    if let Err(message) = part.validate() {
                        ctx.emit_error(DecodeError::unexpected(node, "signal", message));
                    }
                    config.borrow_mut().signal.merge_with(&part);
                }
```

Then update the two other `niri-config` tests that write `ring-drift-hz` in KDL: `focus_response_fields_parse_and_default` (line 924–958) uses `ring-sweep-ms 2000` / `ring-sweep-ms 0` instead and asserts `d.ring_sweep == Duration::from_millis(2000)`, `still.ring_sweep == Duration::ZERO`, `b.ring_sweep == Duration::from_millis(1500)`; `focus_response_rejects_out_of_range` (line 1004) swaps its `ring-drift-hz 31` body for `ring-sweep-ms 10001`. `src/` still compiles: it reads `ring_drift_hz` at its default.

- [ ] **Step 5: Run the suite**

Run: `just test 2>&1 | tail -30`
Expected: everything passes; the three new tests are listed as `ok`.

- [ ] **Step 6: Update `material-config.md`**

- Line 239 table row: replace `ring-drift-hz` with `` `ring-sweep-ms` | 0–10000 (integer) | 1500 ``.
- Line 261 example: `ring-sweep-ms 1500`.
- Lines 282–300: replace the drift prose with: "lights it on the focused window, in `ring-color`. On focus gain the light runs one lap of its travelling brightness over `ring-sweep-ms` milliseconds, easing out onto a fixed pattern, and then the ring costs no redraws until the next focus gain; `0` skips the lap. `signal { motion reduced }`, `motion off`, and `animations { off }` also skip it. A `ring-drift-hz` line is rejected with the replacement named."
- Lines 185–189 (aurora): replace "on the same rule as `ring-drift-hz`" with a self-contained sentence: "`drift-hz` is the field's clock: `0` pins the field, otherwise at least 1, and the error is `aurora drift-hz must be 0 or at least 1`."
- Add to the `signal` section (or create one near the `motion` policy description): "`idle-after-ms <int>` — sustained attention motion (`breathe`, `pulse`, `flash`) freezes on its current frame once no input has arrived for this long and resumes in step on the next input. Default 30000; `0` disables the gate; at most 3600000."

- [ ] **Step 7: Commit**

```bash
tasks check
git add niri-config/src/material/mod.rs niri-config/src/signal.rs niri-config/src/lib.rs docs/materials/material-config.md
git commit -m "feat(config): ring-sweep-ms and signal idle-after-ms; reject ring-drift-hz"
```

---

### Task 2: The sweep replaces the drift clock

The solver, the tile, and the uniform packing change together: `FrameInputs.drift_hz` is written by the tile and read by the solver, so this is one commit. Part A is the solver, Part B the tile.

**Files:**
- Modify: `src/render_helpers/signal.rs:12-20` (constants), `:94-130` (`SignalFrame`, `FrameInputs`), `:231-300` (drift clock), `:318-370` (`tick_deadline`, `solve`), `:373-425` (fingerprint), tests from `:610`
- Modify: `src/render_helpers/material/mod.rs:275, 349, 1606, 1663, 1691, 1811-1871`
- Modify: `niri-config/src/material/mod.rs` (remove `ResolvedResponse.ring_drift_hz` and its `Default`/`with_overrides` lines; keep the decode-only `Response.ring_drift_hz`)
- Modify: `src/layout/tile.rs:140` (field), `:363-373` (crossfade type; add `FocusSweep`), `:417-420` (init), `:425-470` (`update_config`), `:474-478` (`refresh_material`), `:487-498` (delete `focus_drift_hz`), `:500-575` (`signal_for_frame`), `:790-835` (`advance_animations`), `:840-860` (`are_transitions_ongoing`), `:870-905` (`update_render_elements`), `:2117-2140` (`tick_deadline`), tests `:2682-2790`
- Modify: `docs/materials/scripts/material-signals-smoke.sh`, `docs/materials/scripts/glass-optic-smoke-lib.sh`, `docs/materials/render-pipeline.md`
- Test: `src/render_helpers/signal.rs` `mod tests`, `src/layout/tile.rs` `mod tests`

**Interfaces:**
- Consumes: `response.ring_sweep` from Task 1.
- Produces:
  - `pub fn sweep_phase(started: Duration, now: Duration, duration: Duration) -> f32`
  - `pub fn tick_deadline(eff: &EffectiveSignal, in_view: bool, now: Duration) -> Option<Duration>`
  - `FrameInputs { level, accent, presence, focus, sweep: f32 }`, `SignalFrame { .., sweep: f32 }`
  - `pub fn drift_rate`, `phase_on`, `next_boundary_on` unchanged (aurora).
  - `Tile::update_render_elements(&mut self, is_active: bool, visible: bool, view_rect)` unchanged in this task (Task 3 adds `input_active`); private `focus_sweep: Option<FocusSweep>`; private `fn sweep_permitted(&self, response: &ResolvedResponse) -> bool`.

#### Part A: solver

- [ ] **Step A1: Write the failing solver tests**

Append to `mod tests` in `src/render_helpers/signal.rs`:

```rust
    #[test]
    fn sweep_phase_runs_one_eased_lap_then_rests() {
        let d = Duration::from_millis(1500);
        let s = ms(10_000);
        assert_eq!(sweep_phase(s, s, d), 0.);
        // Saturating: sampled before its own start (predicted presentation
        // time, then the clock re-fetched real time) is rest.
        assert_eq!(sweep_phase(s, ms(9_990), d), 0.);
        let mut last = 0.;
        for k in 1..1500 {
            let p = sweep_phase(s, s + Duration::from_millis(k), d);
            assert!(p > last, "monotonic at {k} ms: {p} <= {last}");
            last = p;
        }
        let before_end = sweep_phase(s, s + d - Duration::from_nanos(1), d);
        assert!((before_end - TAU).abs() < 1e-3, "{before_end}");
        assert_eq!(sweep_phase(s, s + d, d), 0.);
        assert_eq!(sweep_phase(s, s + d + Duration::from_secs(60), d), 0.);
        // Ease-out: the first half of the time covers more than half the lap.
        assert!(sweep_phase(s, s + d / 2, d) > TAU / 2.);
    }

    #[test]
    fn travel_is_periodic_in_the_sweep_phase() {
        for i in 0..16 {
            let a = i as f32 * TAU / 16.;
            for j in 0..8 {
                let phi = j as f32 * TAU / 8.;
                assert!((travel(a, phi) - travel(a, phi + TAU)).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn tick_deadline_has_no_focus_arm() {
        let quiet = EffectiveSignal {
            accent: None,
            level: SignalLevel::Quiet,
            motion: SignalMotion::Static,
            impulses: vec![],
        };
        assert_eq!(tick_deadline(&quiet, true, ms(123)), None);
        let breathing = EffectiveSignal { motion: SignalMotion::Breathe, ..quiet };
        assert!(tick_deadline(&breathing, true, ms(123)).is_some());
        assert_eq!(tick_deadline(&breathing, false, ms(123)), None);
    }

    #[test]
    fn settled_focus_fingerprints_to_a_constant() {
        let quiet = EffectiveSignal {
            accent: None,
            level: SignalLevel::Quiet,
            motion: SignalMotion::Static,
            impulses: vec![],
        };
        let inputs = FrameInputs { focus: 1., sweep: 0., ..FrameInputs::quiet() };
        let a = SignalFingerprint::quantize(&solve(&quiet, ms(1000), 0.3, inputs));
        let b = SignalFingerprint::quantize(&solve(&quiet, ms(2000), 0.3, inputs));
        assert_eq!(a, b);
        let mid = FrameInputs { sweep: 1.0, ..inputs };
        let c = SignalFingerprint::quantize(&solve(&quiet, ms(1000), 0.3, mid));
        assert_ne!(a, c, "a moving phase changes the fingerprint");
    }
```

`ms` and `TAU` already exist in that test module (`ms` is used at `:784`; add `use std::f32::consts::TAU;` if the module lacks it).

- [ ] **Step A2: Run to verify they fail**

Run: `just test 2>&1 | grep -E "^error|sweep_phase|no_focus_arm" | head`
Expected: compile errors: `sweep_phase` not found, `tick_deadline` takes 4 arguments, `FrameInputs` has no field `sweep`.

- [ ] **Step A3: Implement the phase and remove the drift clock**

In `src/render_helpers/signal.rs`:

1. Delete `DRIFT_PERIOD` (line 16–17) and the doc line above it.
2. Rename `SignalFrame.drift` → `sweep` with doc `/// Sweep phase in radians; 0 at rest.`; rename `FrameInputs.drift_hz: f64` → `sweep: f32` with doc `/// Sweep phase for this frame, from the tile's sweep state; 0 at rest.`; in `FrameInputs::quiet()` set `sweep: 0.`.
3. Delete `pub fn drift` and `pub fn drift_next_boundary` (lines 285–292) and their docs. Keep `drift_rate`, `buckets_on`, `phase_on`, `next_boundary_on` (aurora).
4. Add after `travel`:

```rust
/// Phase of the focus-gain sweep: one lap of `travel`'s phase from 0 to
/// `2π` over `duration`, easing out (`1 − (1 − x)³`), then exactly 0.
/// `travel` is 2π-periodic, so the lap ends on the rest pattern. The
/// subtraction saturates: `Niri::redraw` samples at the predicted
/// presentation time and the next iteration re-fetches real time, so a
/// sweep can be sampled before its own start; that sample is rest.
pub fn sweep_phase(started: Duration, now: Duration, duration: Duration) -> f32 {
    let t = now.saturating_sub(started);
    if duration.is_zero() || t >= duration {
        return 0.;
    }
    let x = t.as_secs_f32() / duration.as_secs_f32();
    let eased = 1. - (1. - x).powi(3);
    eased * TAU
}
```

5. `tick_deadline`: drop the `drift_hz` parameter and the drift arm:

```rust
/// Next redraw deadline for a tile: sustained signal motion, only while the
/// slab band is in view. The focus sweep runs on the animation loop and has
/// no arm here (design 2026-09-18 §2).
pub fn tick_deadline(eff: &EffectiveSignal, in_view: bool, now: Duration) -> Option<Duration> {
    if !in_view || !eff.is_sustained() {
        return None;
    }
    next_boundary(eff.motion, now)
}
```

6. `solve`: delete the `let drift = …` block; set `sweep: inputs.sweep` in the returned frame.
7. Fingerprint: rename `drift_q` → `sweep_q` (struct, `Default`, `quantize`: `sweep_q: (f.sweep * 1024.).round() as i32`).
8. Tests: delete `drift_is_pinned_at_rate_zero_and_bucketed_otherwise` and any test that calls `drift(`, `drift_next_boundary(`, or passes a fourth argument to `tick_deadline`; update `tick_deadline_requires_sustained_motion_in_view` (line 610) to the three-argument form; in the fingerprint test around line 751–768 rename `drift` → `sweep`, `drift_q` → `sweep_q`. Keep `drift_rate_follows_policy_and_animations`: aurora still uses it.

In `src/render_helpers/material/mod.rs`: line 275 doc → `/// Crossfaded focus and sweep phase.`; line 349 `focus: [frame.focus, frame.sweep],`; lines 1606, 1663, 1691 `sweep: 0.,`; test at 1811 rename to `signal_uniforms_carry_presence_focus_sweep_and_selectors` and `sweep: 1.5`, `sweep: 0.`.

In `niri-config/src/material/mod.rs`: delete `ResolvedResponse.ring_drift_hz`, its `Default` line, and its `with_overrides` line (the decode-only `Response.ring_drift_hz` and the rejection stay).

The tile does not compile yet; Part B fixes it. Do not run the suite between A3 and B3.

#### Part B: tile

- [ ] **Step B1: Write the failing tile tests**

Replace `only_a_focused_tile_drifts` and `a_focused_drifting_tile_keeps_its_signal_deadline` (lines 2682–2696, 2759–2775) with:

```rust
    fn sweep_of(tile: &Tile<TestWindow>) -> Option<(Duration, Duration)> {
        tile.focus_sweep.as_ref().map(|s| (s.started, s.duration))
    }

    #[test]
    fn focus_gain_from_rest_starts_one_lap_and_loss_does_not() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));

        tile.update_render_elements(false, true, view);
        assert_eq!(sweep_of(&tile), None);

        clock.set_unadjusted(Duration::from_millis(100));
        tile.update_render_elements(true, true, view);
        assert_eq!(
            sweep_of(&tile),
            Some((Duration::from_millis(100), Duration::from_millis(1500)))
        );
        assert!(tile.are_transitions_ongoing());

        // Loss mid-lap: the lap keeps its start; nothing new starts.
        clock.set_unadjusted(Duration::from_millis(600));
        tile.update_render_elements(false, true, view);
        assert_eq!(sweep_of(&tile).map(|s| s.0), Some(Duration::from_millis(100)));

        // Regain mid-lap: still the same lap.
        clock.set_unadjusted(Duration::from_millis(900));
        tile.update_render_elements(true, true, view);
        assert_eq!(sweep_of(&tile).map(|s| s.0), Some(Duration::from_millis(100)));

        // Finished laps are cleared, and the next gain starts a fresh one.
        clock.set_unadjusted(Duration::from_millis(2000));
        tile.update_render_elements(false, true, view);
        tile.advance_animations();
        assert_eq!(sweep_of(&tile), None);
        clock.set_unadjusted(Duration::from_millis(2100));
        tile.update_render_elements(true, true, view);
        assert_eq!(sweep_of(&tile).map(|s| s.0), Some(Duration::from_millis(2100)));
    }

    #[test]
    fn the_sweep_phase_reaches_the_frame_and_rests_at_zero() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, view);
        clock.set_unadjusted(Duration::from_millis(750));
        tile.update_render_elements(true, true, view);
        let (_, inputs) = tile.signal_frame_cache.borrow().clone().unwrap();
        assert!(inputs.sweep > 0., "{}", inputs.sweep);
        clock.set_unadjusted(Duration::from_millis(5000));
        tile.update_render_elements(true, true, view);
        tile.advance_animations();
        let (_, inputs) = tile.signal_frame_cache.borrow().clone().unwrap();
        assert_eq!(inputs.sweep, 0.);
    }

    #[test]
    fn settled_focus_reports_no_deadline_and_no_transition() {
        // Already focused, crossfade done, lap done: the three settled
        // gates of the design (§2).
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        tile.active = true;
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, view);
        assert!(tile.focus_crossfade.is_none());
        assert_eq!(sweep_of(&tile), None, "no change of focus, no lap");
        assert!(tile.signal_frame_cache.borrow().is_some());
        assert_eq!(tile.tick_deadline(Point::default(), view, Duration::ZERO), None);
        assert!(!tile.are_transitions_ongoing());
    }

    #[test]
    fn sweep_is_skipped_under_reduced_off_and_animations_off() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let cases: [(niri_config::SignalMotionPolicy, bool); 3] = [
            (niri_config::SignalMotionPolicy::Reduced, false),
            (niri_config::SignalMotionPolicy::Off, false),
            (niri_config::SignalMotionPolicy::Full, true),
        ];
        for (policy, animations_off) in cases {
            let clock = Clock::with_time(Duration::ZERO);
            let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
            let mut options = (*tile.options).clone();
            options.signal.motion = policy;
            options.animations.off = animations_off;
            tile.options = Rc::new(options);
            tile.update_render_elements(true, true, view);
            assert_eq!(sweep_of(&tile), None, "{policy:?} animations_off={animations_off}");
        }
    }

    #[test]
    fn sweep_duration_is_snapshotted_and_a_policy_change_cuts_the_lap() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, view);
        assert_eq!(sweep_of(&tile).map(|s| s.1), Some(Duration::from_millis(1500)));

        // A reload that only changes the duration leaves the running lap alone.
        let mut options = (*tile.options).clone();
        options.signal.motion = niri_config::SignalMotionPolicy::Full;
        tile.update_config(Size::from((1280., 720.)), 1., Rc::new(options));
        assert_eq!(sweep_of(&tile).map(|s| s.1), Some(Duration::from_millis(1500)));

        // A reload to reduced cuts it.
        let mut options = (*tile.options).clone();
        options.signal.motion = niri_config::SignalMotionPolicy::Reduced;
        tile.update_config(Size::from((1280., 720.)), 1., Rc::new(options));
        assert_eq!(sweep_of(&tile), None);
    }
```

Also fix the three existing tests that build `FrameInputs { drift_hz: .. }` (lines 2725, 2745): use `sweep: 0.`. `an_out_of_view_slab_reports_no_deadline` asserted a focused tile has a deadline; change its fixture to a breathing signal or delete it — the settled test above covers the in-view/out-of-view gate through `tick_deadline_has_no_focus_arm` in Task 2 and `an_unfocused_signal_free_aurora_tile_reports_its_next_bucket` here. Delete it.

- [ ] **Step B2: Run to verify they fail**

Run: `just test 2>&1 | grep -E "^error|focus_sweep|sweep_of" | head`
Expected: `focus_sweep` not found, `FrameInputs` has no field `drift_hz`, `focus_drift_hz` reads a missing field.

- [ ] **Step B3: Implement the sweep state**

In `src/layout/tile.rs`:

1. Next to `FocusCrossfade` (line 363):

```rust
/// One focus-gain lap of the ring's travelling light (design 2026-09-18 §1).
struct FocusSweep {
    /// Start instant on the unadjusted clock.
    started: Duration,
    /// Snapshotted at start so a reload mid-lap does not move the phase.
    duration: Duration,
}

impl FocusSweep {
    fn is_done(&self, now: Duration) -> bool {
        now >= self.started + self.duration
    }
}
```

2. Field after `focus_crossfade`: `focus_sweep: Option<FocusSweep>,`; initialize `focus_sweep: None,` in `Tile::new`.

3. Delete `focus_drift_hz` (lines 487–498). Add:

```rust
    /// Whether a focus-gain lap may run under the current response and policy.
    fn sweep_permitted(&self, response: &ResolvedResponse) -> bool {
        response.focus == niri_config::FocusResponse::RingLight
            && !response.ring_sweep.is_zero()
            && self.options.signal.motion == niri_config::SignalMotionPolicy::Full
            && !self.options.animations.off
    }

    /// Cut rule: a lap in progress ends at rest the moment it is no longer
    /// permitted (policy, animations, or the response changed).
    fn cut_sweep_if_forbidden(&mut self) {
        let permitted = self
            .material
            .as_ref()
            .is_some_and(|material| self.sweep_permitted(&material.material().response(None)));
        if !permitted {
            self.focus_sweep = None;
        }
    }
```

4. In `update_config` after `self.options = options;` call `self.cut_sweep_if_forbidden();`. In `refresh_material` after `apply_resolved(...)` call `self.cut_sweep_if_forbidden();`.

5. In `signal_for_frame`, replace `let drift_hz = self.focus_drift_hz(response);` with:

```rust
        let now = self.clock.now_unadjusted();
        let sweep = self
            .focus_sweep
            .as_ref()
            .map_or(0., |s| sweep_phase(s.started, now, s.duration));
```

and `drift_hz,` → `sweep,` in the `FrameInputs` literal. Import `sweep_phase` beside the other `signal::` imports in that function.

6. In `update_render_elements`, inside `if self.active != is_active {` after the crossfade is created, add the start rule:

```rust
            if is_active {
                let now = self.clock.now_unadjusted();
                let at_rest = self.focus_sweep.as_ref().is_none_or(|s| s.is_done(now));
                if at_rest && response.as_ref().is_some_and(|r| self.sweep_permitted(r)) {
                    let duration = response.as_ref().unwrap().ring_sweep;
                    self.focus_sweep = Some(FocusSweep { started: now, duration });
                }
            }
```

(`response` is the `Option<ResolvedResponse>` computed at the top of that function; borrow it before `self.active` is assigned if the borrow checker complains, e.g. `let sweep_allowed = response.as_ref().is_some_and(|r| self.sweep_permitted(r)); let sweep_duration = response.as_ref().map(|r| r.ring_sweep);` computed first.)

7. In `advance_animations`, after the `focus_crossfade` clearing block:

```rust
        if self
            .focus_sweep
            .as_ref()
            .is_some_and(|sweep| sweep.is_done(self.clock.now_unadjusted()))
        {
            self.focus_sweep = None;
        }
```

8. In `are_transitions_ongoing`, inside the `self.signal_render_visible && (...)` group add:

```rust
                    || self
                        .focus_sweep
                        .as_ref()
                        .is_some_and(|sweep| !sweep.is_done(self.clock.now_unadjusted()))
```

9. In `Tile::tick_deadline` (line 2135): `.and_then(|(eff, _)| tick_deadline(eff, true, now));` and update the doc comment: "the sustained-signal bucket boundary" (drop "or focus-drift").

- [ ] **Step B4: Run the whole suite**

Run: `just test 2>&1 | tail -30`
Expected: all pass, Part A's solver tests included. If `focus_none_never_crossfades_the_filament` fails, `sweep_permitted` must return false for `FocusResponse::None` — check the first condition.

- [ ] **Step B5: Update the smoke fixtures that name `ring-drift-hz`**

`docs/materials/scripts/material-signals-smoke.sh` lines 129, 155, 168, 170: replace `ring-drift-hz 0` with `ring-sweep-ms 0`. Lines 172–186: rename the focused fixtures to what they now test:

```bash
# Focused-window fixtures: on focus gain the filament runs one lap over
# ring-sweep-ms on the animation loop and then rests. The steady window
# starts long after the lap, so a focused window costs nothing there.
write_config "$WORK/sweep.kdl"          'material "tg2" { glass {}; response "default" { ring-sweep-ms 1500; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }'
write_config "$WORK/sweep-anim-off.kdl" 'material "tg2" { glass {}; response "default" { ring-sweep-ms 1500; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'animations { off; }'
write_config "$WORK/sweep-reduced.kdl"  'material "tg2" { glass {}; response "default" { ring-sweep-ms 1500; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'signal { motion "reduced"; }'
write_config "$WORK/focus-none.kdl"     'material "tg2" { glass {}; response "default" { focus "none"; ring-sweep-ms 1500; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'layout { focus-ring { off; }; }'
```

`docs/materials/scripts/glass-optic-smoke-lib.sh:182`: `ring-sweep-ms 0`. Grep `docs/materials/scripts` for any other `ring-drift-hz` and convert the same way. The `mode_cases` body is rewritten in Task 4; here only make the fixtures parse.

- [ ] **Step B6: Update `render-pipeline.md`**

Line 154 table row: replace `ring-drift-hz` with `ring-sweep-ms`. In the section that describes the focus ring's redraw contract (search "drift"), state: "The focus sweep is one lap on the animation loop, `ring-sweep-ms` long, reported through `are_transitions_ongoing` while the band is in view; it has no bucket clock and no deadline. Settled focus reports no deadline and a constant fingerprint."

- [ ] **Step B7: Commit**

```bash
tasks check
git add niri-config/src/material/mod.rs src/render_helpers/signal.rs src/render_helpers/material/mod.rs src/layout/tile.rs docs/materials/scripts docs/materials/render-pipeline.md
git commit -m "feat(material): run one eased lap on focus gain, then rest"
```

---

### Task 3: The input-activity gate

**Files:**
- Create: `src/activity.rs`
- Modify: `src/lib.rs:5` (add `pub mod activity;`)
- Modify: `src/niri.rs:362-370` (fields), `:2740-2765` (`Niri::new` init), `:1586-1590` and `:1680-1684` (reload), `:4256-4260` (`update_render_elements`), `:6614-6625` (`notify_activity`)
- Modify: `src/render_helpers/signal.rs:49-83` (`effective`)
- Modify: `src/layout/mod.rs:2805-2850, 4708`, `src/layout/monitor.rs:1092-1104`, `src/layout/workspace.rs:378-386, 1724`, `src/layout/scrolling.rs:399-408, 4129-4140`, `src/layout/floating.rs:268-281`, `src/layout/tile.rs` (`update_render_elements` signature and every test call)
- Test: `src/activity.rs` `mod tests`; `src/render_helpers/signal.rs` `mod tests`; `src/layout/tile.rs` `mod tests`

**Interfaces:**
- Produces:
  - `pub struct InputActivity` with `new(now, threshold)`, `observe(&mut self, now) -> bool` (true when idle was cleared), `poll(&mut self, now) -> Poll`, `set_threshold(&mut self, threshold, now) -> bool` (true when idle changed), `is_idle(&self) -> bool`, `next_check(&self) -> Option<Duration>`.
  - `pub enum Poll { Idle, Rearm(Duration) }` — `Rearm` carries the absolute instant to re-arm for.
  - `signal::effective(folded, policy, response, input_active: bool)`.
  - `Layout::set_input_active(&mut self, active: bool)`; `Tile::update_render_elements(is_active, input_active, visible, view_rect)`.

- [ ] **Step 1: Write the failing activity tests**

Create `src/activity.rs`:

```rust
//! Compositor-wide input activity for the attention gate
//! (docs/specs/2026-09-18-ring-focus-motion-design.md §3). Pure: the
//! event-loop timer in `Niri` asks this what to do and when.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Poll {
    /// The threshold has elapsed; the caller drops its timer.
    Idle,
    /// Input arrived during the wait; re-arm for this absolute instant.
    Rearm(Duration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputActivity {
    last: Duration,
    idle: bool,
    threshold: Duration,
}

impl InputActivity {
    pub fn new(now: Duration, threshold: Duration) -> Self {
        Self {
            last: now,
            idle: false,
            threshold,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.idle
    }

    /// Whether the gate is on at all.
    pub fn enabled(&self) -> bool {
        !self.threshold.is_zero()
    }

    /// The instant a timer should fire to check for idleness, or `None`
    /// when no timer is needed (gate off, or already idle).
    pub fn next_check(&self) -> Option<Duration> {
        (self.enabled() && !self.idle).then(|| self.last + self.threshold)
    }

    /// Input arrived. Returns true when this cleared idle.
    pub fn observe(&mut self, now: Duration) -> bool {
        self.last = now;
        std::mem::replace(&mut self.idle, false)
    }

    /// The timer fired.
    pub fn poll(&mut self, now: Duration) -> Poll {
        if !self.enabled() {
            return Poll::Rearm(Duration::MAX);
        }
        if now.saturating_sub(self.last) >= self.threshold {
            self.idle = true;
            Poll::Idle
        } else {
            Poll::Rearm(self.last + self.threshold)
        }
    }

    /// Config reload: recompute idle from the elapsed quiet time and the new
    /// threshold. Returns true when the idle state changed.
    pub fn set_threshold(&mut self, threshold: Duration, now: Duration) -> bool {
        self.threshold = threshold;
        let idle = self.enabled() && now.saturating_sub(self.last) >= threshold;
        std::mem::replace(&mut self.idle, idle) != idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    #[test]
    fn fresh_compositor_with_no_input_goes_idle_at_the_threshold() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.next_check(), Some(s(30)));
        assert_eq!(a.poll(s(30)), Poll::Idle);
        assert!(a.is_idle());
        assert_eq!(a.next_check(), None);
    }

    #[test]
    fn input_inside_the_threshold_rearms_without_flipping() {
        let mut a = InputActivity::new(s(0), s(30));
        assert!(!a.observe(s(10)));
        assert_eq!(a.poll(s(30)), Poll::Rearm(s(40)));
        assert!(!a.is_idle());
        assert_eq!(a.poll(s(40)), Poll::Idle);
    }

    #[test]
    fn input_clears_idle_once() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.poll(s(30)), Poll::Idle);
        assert!(a.observe(s(31)), "first input after idle reports the clear");
        assert!(!a.observe(s(32)), "further input is quiet");
        assert_eq!(a.next_check(), Some(s(62)));
    }

    #[test]
    fn zero_threshold_never_flips() {
        let mut a = InputActivity::new(s(0), s(0));
        assert_eq!(a.next_check(), None);
        assert_eq!(a.poll(s(1_000_000)), Poll::Rearm(Duration::MAX));
        assert!(!a.is_idle());
    }

    #[test]
    fn reload_recomputes_idle_in_both_directions() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.poll(s(45)), Poll::Idle);
        // Raise past the elapsed 45 s: active again, timer due at 60.
        assert!(a.set_threshold(s(60), s(45)));
        assert!(!a.is_idle());
        assert_eq!(a.next_check(), Some(s(60)));
        // Lower under the elapsed time: idle at once.
        assert!(a.set_threshold(s(10), s(45)));
        assert!(a.is_idle());
        // Same side: no change reported.
        assert!(!a.set_threshold(s(20), s(45)));
        // Zero clears and disables.
        assert!(a.set_threshold(s(0), s(45)));
        assert!(!a.is_idle());
        assert_eq!(a.next_check(), None);
    }
}
```

Add `pub mod activity;` to `src/lib.rs` (alphabetical, after `a11y`).

Add to `src/render_helpers/signal.rs` tests:

```rust
    #[test]
    fn idle_input_makes_sustained_motion_static_and_nothing_else() {
        let folded = Folded {
            level: SignalLevel::Demand,
            motion: SignalMotion::Pulse,
            accent: Some(Color::from_rgba8_unpremul(0xe5, 0xa3, 0x3c, 0xff)),
            tag: None,
            sources: vec![String::from("demo")],
            impulses: vec![],
        };
        let response = ResolvedResponse::default();
        let active = effective(&folded, SignalMotionPolicy::Full, &response, true);
        assert_eq!(active.motion, SignalMotion::Pulse);
        let idle = effective(&folded, SignalMotionPolicy::Full, &response, false);
        assert_eq!(idle.motion, SignalMotion::Static);
        assert_eq!(idle.level, active.level);
        assert_eq!(idle.accent, active.accent);
        assert_eq!(idle.impulses, active.impulses);
    }
```

(Check `Folded`'s exact fields at `src/window/signal.rs:126` and match them; if a builder or `Default` exists in that test module, use it.)

- [ ] **Step 2: Run to verify they fail**

Run: `just test 2>&1 | grep -E "^error|activity|idle_input" | head`
Expected: `effective` takes 3 arguments; activity tests compile and pass on their own once the module exists (they are new code) — the failing part is `effective`.

- [ ] **Step 3: Gate `effective` and thread `input_active`**

`src/render_helpers/signal.rs` `effective`: add `input_active: bool` as the last parameter; after the `attention_is_none` mapping:

```rust
    let motion = if input_active { motion } else { SignalMotion::Static };
```

Update its doc: "Stage 1: apply the global policy, the response block, and the input-activity gate (design 2026-09-18 §3): idle input makes sustained motion static and touches nothing else." Fix the existing `effective(` test calls in that file (pass `true`).

`src/layout/tile.rs`:
- `update_render_elements(&mut self, is_active: bool, input_active: bool, visible: bool, view_rect: ...)`; store `self.input_active = input_active;` in a new private field `input_active: bool` (init `true` in `Tile::new`); pass `self.input_active` into `effective(folded, self.options.signal.motion, response, self.input_active)` inside `signal_for_frame`. Update every call in tests to `(is_active, true, visible, view)`.
- Add a test:

```rust
    #[test]
    fn idle_input_freezes_attention_but_not_the_sweep() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, false, true, view);
        assert!(sweep_of(&tile).is_some(), "the lap is finite and not gated on idle");
        let (eff, _) = tile.signal_frame_cache.borrow().clone().unwrap();
        assert_eq!(eff.motion, niri_ipc::SignalMotion::Static);
    }
```

(The fixture window has no signal, so `eff.motion` is Static either way; to make the assertion meaningful, give the `TestWindow` a pulsing signal if `TestWindowParams` supports one — check `src/layout/tests.rs` / the `TestWindow` signal hook used by `signal_for_frame` (`self.window.signal(now)`); if no fixture can set it, keep only the sweep assertion and rely on the solver test for the motion rule, and say so in a comment.)

Thread the parameter, matching `is_active`:
- `src/layout/floating.rs:268`: `update_render_elements(&mut self, is_active: bool, input_active: bool, visible: bool, view_rect)` → `tile.update_render_elements(is_active, input_active, visible, tile_view_rect)`.
- `src/layout/scrolling.rs:4129` (Column): `update_render_elements(&mut self, is_active: bool, input_active: bool, view_rect)` → tile call gets `input_active`. `:399` (ScrollingSpace): add `input_active: bool`, pass to `col.update_render_elements(is_active, input_active, view_rect)`.
- `src/layout/workspace.rs:378`: add `input_active: bool`; pass to scrolling and floating. `:1724` (`tile.update_render_elements(false, false, view_rect)`) → `(false, true, false, view_rect)` — an unmapped/closing tile computes no attention, the value is inert; comment that.
- `src/layout/monitor.rs:1092`: add `input_active: bool`; `ws.update_render_elements(is_active, input_active)`.
- `src/layout/mod.rs`: add field `input_active: bool` to `Layout` (init `true` in every constructor; grep `Layout {` literals) and:

```rust
    /// Input-activity gate for sustained attention motion (design
    /// 2026-09-18 §3). Set by `Niri` when its idle state changes; read on
    /// the next `update_render_elements`.
    pub fn set_input_active(&mut self, active: bool) {
        self.input_active = active;
    }
```

  In `update_render_elements`: `move_.tile.update_render_elements(true, self.input_active, true, view_rect)` (line 2826), `mon.update_render_elements(is_active, self.input_active)` (2850); line 4708 `(false, true, false, view_rect)` with the same comment as workspace 1724.

Fix every other call site the compiler reports (tests in `src/layout/tests.rs` and the layout modules).

- [ ] **Step 4: Own the state in `Niri`**

`src/niri.rs`:
- Fields (near `pointer_inactivity_timer`): `pub input_activity: crate::activity::InputActivity,` and `pub input_idle_timer: Option<RegistrationToken>,`.
- `Niri::new`: `input_activity: crate::activity::InputActivity::new(get_monotonic_time(), config.signal.idle_after)` — `config` is the borrowed `Config` in that constructor; `input_idle_timer: None`. Immediately after the struct is built (where other timers are armed, or at the end of `new` before return), call `niri.arm_input_idle_timer();`.
- Methods, next to `reset_pointer_inactivity_timer`:

```rust
    /// (Re)arms the attention idle timer from `input_activity.next_check()`.
    pub fn arm_input_idle_timer(&mut self) {
        if let Some(token) = self.input_idle_timer.take() {
            self.event_loop.remove(token);
        }
        let Some(at) = self.input_activity.next_check() else {
            return;
        };
        let delay = at.saturating_sub(get_monotonic_time());
        let token = self
            .event_loop
            .insert_source(Timer::from_duration(delay), move |_, _, state| {
                state.niri.input_idle_timer = None;
                match state.niri.input_activity.poll(get_monotonic_time()) {
                    crate::activity::Poll::Idle => {
                        state.niri.layout.set_input_active(false);
                        state.niri.queue_redraw_all();
                        TimeoutAction::Drop
                    }
                    crate::activity::Poll::Rearm(at) => {
                        TimeoutAction::ToDuration(at.saturating_sub(get_monotonic_time()))
                    }
                }
            })
            .unwrap();
        self.input_idle_timer = Some(token);
    }

    /// Config reload changed `signal { idle-after-ms }`.
    pub fn set_input_idle_threshold(&mut self, threshold: Duration) {
        let changed = self
            .input_activity
            .set_threshold(threshold, get_monotonic_time());
        if changed {
            self.layout.set_input_active(!self.input_activity.is_idle());
            self.queue_redraw_all();
        }
        self.arm_input_idle_timer();
    }
```

  Note `TimeoutAction::ToDuration` keeps the same source registered; when it returns `Drop`, the token was already cleared on entry. If the calloop version in `Cargo.lock` lacks `ToDuration`, return `Drop` and call `state.niri.arm_input_idle_timer()` inside the `Rearm` arm instead.

- `notify_activity` (line 6614): after the early return, add:

```rust
        if self.input_activity.observe(get_monotonic_time()) {
            self.layout.set_input_active(true);
            self.queue_redraw_all();
            self.arm_input_idle_timer();
        }
```

  The timer is armed only on the idle→active edge; while active, an early fire re-arms itself (`Rearm`), so per-input work is one `observe`.

- Reload (`reload_config`, near line 1586): `if config.signal.idle_after != old_config.signal.idle_after { idle_threshold_changed = true; }` and near line 1680: `if idle_threshold_changed { self.niri.set_input_idle_threshold(config.signal.idle_after); }` — declare the flag with the other `*_changed` booleans at the top of that function.

- [ ] **Step 5: Run the whole suite**

Run: `just test 2>&1 | tail -30`
Expected: all pass, including the five `activity` tests and the tile/solver tests from Task 2 with the new signature.

- [ ] **Step 6: Document the gate in the signals design**

`docs/materials/2026-09-02-material-signals-design.md`: in §4 (Redraw and damage contract) add one paragraph: "Sustained motion is additionally gated on input activity: after `signal { idle-after-ms }` of no input the effective motion is `Static` and no bucket deadline is reported; the next input redraws every output and motion resumes from the absolute clock. Design: `docs/specs/2026-09-18-ring-focus-motion-design.md` §3." In §5 (Configuration contract) add the `idle-after-ms` line from `material-config.md`.

- [ ] **Step 7: Commit**

```bash
tasks check
git add src/activity.rs src/lib.rs src/niri.rs src/render_helpers/signal.rs src/layout docs/materials/2026-09-02-material-signals-design.md
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): freeze sustained attention motion while input is idle"
```

---

### Task 4: Evidence — redraw counts, clips, and closing docs

**Files:**
- Modify: `docs/materials/scripts/material-signals-smoke.sh:125-135` (base fixture gets `signal { idle-after-ms 0; }`), `:490-575` (cases)
- Modify: `docs/specs/2026-09-05-ring-light-focus-response-design.md:1-16` (status)
- Modify: `docs/specs/2026-09-18-ring-focus-motion-design.md:3` (status)
- Create: `docs/materials/2026-09-18-ring-focus-motion-evidence.md`

**Interfaces:**
- Consumes: everything above, built as a release binary.
- Produces: the evidence record and the user-reviewed clips under `$NIRI_MATERIAL_WORK_ROOT`.

**Host:** the headless verification host (a headless weston unit), never the desktop session. The smoke script refuses on host load; if it refuses, `tasks park <child> "<rerun command>" --reason quiet --waiting-on user --minutes 25` and stop.

- [ ] **Step 1: Keep the existing cases honest under the default gate**

The nested compositor receives no input, so with the default `idle-after-ms 30000` every fixture would freeze about when the capture starts. In `write_config` (line 125), add `signal { idle-after-ms 0; }` to the base lines so every existing case measures what it measured before. Add a second helper for the idle cases:

```bash
# Idle-gate fixtures: the nested compositor sees no input, so a short
# threshold makes it idle a few seconds after start. Freeze and resume are
# driven through config reloads (spec §3: a reload recomputes the state),
# since the headless host cannot inject input.
write_idle_config() {   # $1 = path, $2 = idle-after-ms, remaining = extra KDL lines
    local f=$1 ms=$2; shift 2
    write_config "$f" "$@"
    sed -i "s/signal { idle-after-ms 0; }/signal { idle-after-ms $ms; }/" "$f"
}
write_idle_config "$WORK/idle-5s.kdl" 5000
```

(`write_config` writes the `signal` line; make sure the replacement matches its exact spelling.)

- [ ] **Step 2: Rewrite the focus cases and add the idle and DPMS cases**

In `mode_cases`, replace the five `focused-*` / `other-focused` lines with:

```bash
    # The focus sweep: one lap on focus gain, finished long before the steady
    # window, so a focused window at rest costs nothing under every policy.
    steady_zero  focused-settled    "$WORK/sweep.kdl"          setup_focused_quiet
    steady_zero  focused-reduced    "$WORK/sweep-reduced.kdl"  setup_focused_quiet
    steady_zero  focused-anim-off   "$WORK/sweep-anim-off.kdl" setup_focused_quiet
    steady_zero  focus-none-sweep   "$WORK/focus-none.kdl"     setup_focused_quiet
    steady_zero  other-focused      "$WORK/sweep.kdl"          setup_other_focused
```

After `focus_toggle_case focus-none-toggle ...` add a sweep toggle case whose gate is the quiet tail, with the total recorded (each of the six gains runs a 1.5 s lap, so the total is not bounded by the control):

```bash
# Six focus changes with the sweep on: each gain runs one lap, so the total
# is recorded, not gated; the quiet tail after the toggles must be zero.
sweep_toggle_case() {   # $1 name, $2 cfg
    run_case "$1" "$2" setup_other_focused during_focus_toggles
    local total after quiet_tail
    total=$(count_steady "$1")
    quiet_tail=$(awk -v s="$TOGGLE_SPAN" 'BEGIN { printf "%.1f", 18 - s - 1.5 }')
    awk -v t="$quiet_tail" 'BEGIN { exit !(t >= 2) }' \
        || { echo "FAIL: $1: toggles took $TOGGLE_SPAN s, leaving only $quiet_tail s of quiet tail" >&2; exit 1; }
    after=$(count_window "$1" "$quiet_tail" 0)
    printf '%s: %d redraws for six focus changes, %d in the %s s after\n' "$1" "$total" "$after" "$quiet_tail" | tee -a "$WORK/rates.txt" >&2
    expect_zero "$1 after" "$after"
}
```

and call it: `sweep_toggle_case sweep-toggle "$WORK/sweep.kdl"`. The `- 1.5` leaves the last lap out of the tail.

Add the idle cases after `demand-flash`:

```bash
# Idle gate: 5 s threshold, no input, so the compositor is idle before the
# steady window and a pulsing window draws nothing. Resume is a reload to
# idle-after-ms 0, freeze again a reload back to 5000 (spec §3 reload rule).
during_idle_resume() {
    sed -i 's/idle-after-ms 5000/idle-after-ms 0/' "$IDLE_CFG"; msg action load-config-file
    sleep 6
    sed -i 's/idle-after-ms 0/idle-after-ms 5000/' "$IDLE_CFG"; msg action load-config-file
}
idle_case() {   # $1 name, $2 cfg, $3 setup, $4 pulse rate for a resumed window
    IDLE_CFG=$2
    run_case "$1" "$2" "$3" during_idle_resume
    local total resumed frozen
    total=$(count_steady "$1")
    # The resume window is [end-18 s, end-12 s): about 6 s of bucket-rate redraws.
    resumed=$(count_window "$1" 18 12)
    frozen=$(count_window "$1" 11 0)
    expect_about "$1 resumed" "$resumed" "$(( $4 * 6 / 20 ))" 0.25
    expect_zero "$1 frozen again" "$frozen"
}
    steady_zero  idle-pulse "$WORK/idle-5s.kdl" "setup_demand pulse"
    idle_case    idle-resume "$WORK/idle-5s.kdl" "setup_demand pulse" "$pulse_n"
    steady_zero  dpms-off-pulse "$WORK/base.kdl" setup_dpms_off_pulse
```

with `setup_dpms_off_pulse() { setup_demand pulse; msg action power-off-monitors; }`. Check `count_window`'s argument order (start offset from end, end offset from end) against its definition near line 291 before relying on `18 12`.

`inactive-workspace`, `hidden-tab`, and `offscreen-column` already cover the hidden cases; leave them.

- [ ] **Step 3: Run the cases on the headless host**

```bash
just build-release   # or the recipe the script's header names for NIRI
NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT docs/materials/scripts/material-signals-smoke.sh cases 2>&1 | tee "$NIRI_MATERIAL_WORK_ROOT/ring-motion-cases.log"
```

Expected: every `steady_zero` prints 0; `idle-resume resumed` is within 25 % of a quarter of the pulse rate; `sweep-toggle` prints a total and `0 in the … s after`. Copy the `rates.txt` lines into the evidence record.

- [ ] **Step 4: Capture the review clips**

Under the capture protocol (`tools/capture-meta`, strict cost timing; the GPU waiver covers pixels only), record four sequences from a nested instance on `sweep.kdl` with two kitty windows, as frame bursts at the fixture's refresh rate for the durations given, using the burst machinery of `docs/materials/scripts/focus-ring-light.sh` (its `burst` capture path) adapted to these drives:

1. `gain-from-rest`: focus the other window, wait 3 s, focus back; 2.5 s of frames from the focus command.
2. `alt-tab-three`: three windows; `focus-window` across all three at 0.4 s spacing; 4 s of frames.
3. `loss-mid-lap`: focus back, then away at 0.5 s; 2.5 s of frames.
4. `idle-freeze-resume`: `idle-5s.kdl`, a breathing window; frames from 4 s to 7 s (freeze), reload to `0`, 3 s more (resume).

Write each burst's directory and its `capture.json` path into the evidence record. These are for the owner's judgment of the 1500 ms starting value and the ease; the plan records them, it does not grade them.

- [ ] **Step 5: Write the evidence record and update the status headers**

Create `docs/materials/2026-09-18-ring-focus-motion-evidence.md` with: the commit under test, host, the `rates.txt` lines per case with the expectation each met, the four clip locations, and any case that was skipped with the reason. Model it on `docs/materials/2026-09-12-material-render-order-evidence.md`.

`docs/specs/2026-09-18-ring-focus-motion-design.md` line 3: `**Status:** implemented on materials-26.04 (<merge commit>); evidence in [2026-09-18-ring-focus-motion-evidence.md](../materials/2026-09-18-ring-focus-motion-evidence.md); the sweep duration awaits the owner's clip review (material-9306b5 tunes appearance).`

`docs/specs/2026-09-05-ring-light-focus-response-design.md` status: append "Its continuous drift (`ring-drift-hz`) is superseded by the finite sweep of [2026-09-18-ring-focus-motion-design.md](2026-09-18-ring-focus-motion-design.md)."

- [ ] **Step 6: Commit and close**

```bash
tasks check
git add docs/materials/scripts/material-signals-smoke.sh docs/materials/2026-09-18-ring-focus-motion-evidence.md docs/specs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
tasks done <this task's child id> "smoke cases pass on the headless host; clips recorded; status headers updated"
git add tasks
git commit -m "docs(material): record ring motion evidence and close the design"
```

Then, on `material-0e130e` itself once every child is done: `tasks done material-0e130e "one eased lap on focus gain, settled focus costs nothing, attention frozen while idle; evidence 2026-09-18-ring-focus-motion-evidence.md"`, in the same commit as the last child or a follow-up `chore(tasks)` commit.

---

## Cross-project follow-up (not a step of this plan)

Prism: one task under `material-743692`'s coordination, filed at plan approval, depending on Task 1's child: replace `glass.ring.driftHz` with `glass.ring.sweepMs` (`replaces:` on the definition), `prism migrate` over base and every context with a backup and report (`0 → 0`, positive → `1500`, an existing `sweepMs` kept), `doctor` hint, `render.js` emitting `ring-sweep-ms`, tests per spec §7, and the §5 rollout order when applied live.

## Self-review

- Spec §1 (phase, state, start, cut, redraw, removed): Task 2. §2 (three settled gates): Task 2 `tick_deadline_has_no_focus_arm`, `settled_focus_fingerprints_to_a_constant`, `settled_focus_reports_no_deadline_and_no_transition`. §3 (activity, init, reload, resume, matrix): Task 3; the smoke's hidden/DPMS/idle rows: Task 4. §4: Task 1. §5: cross-project follow-up (Prism repo). §6: file map. §7 headless and clips: Task 4. §8 docs: Task 1 (`material-config.md`), Task 2 (`render-pipeline.md`, smoke fixtures), Task 3 (signals design), Task 4 (status headers).
- Every task leaves the workspace compiling and `just test` green: Task 1 keeps `ResolvedResponse.ring_drift_hz` inert; Task 2 removes it together with its only readers.
- Deviation from §7 recorded: resume in the smoke is driven by a threshold reload, not synthetic input, because the headless host cannot inject input; the input path is covered by `InputActivity::observe` unit tests and `notify_activity` is a one-line call into it.
- Names used consistently: `sweep_phase`, `FocusSweep { started, duration }`, `FrameInputs.sweep`, `SignalFrame.sweep`, `sweep_q`, `InputActivity`, `Poll::{Idle, Rearm}`, `set_input_active`, `effective(.., input_active)`, `ring_sweep`, `idle_after`.
