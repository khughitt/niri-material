# Ring of Light Focus Response Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the invisible gradient focus ring on material windows with an embedded, refracted filament of light in the bevel that fades in with focus, drifts at a bucket-timer rate, breathes with the jelly residuals, and shares its band with the signal accent ring.

**Architecture:** The filament is a generalization of the existing signal accent ring. The tile records its active state and crossfades a `focus` value through the `material-signal` animation, alongside a new crossfaded accent `presence` that lets accent color be carried straight. A new drift oscillator in the pure solver quantizes its phase to `1 / ring-drift-hz` buckets and reports boundaries into the per-output bucket timer the signals design already runs. The shader replaces the accent band with a refracted band masked to the rendered bevel.

**Tech Stack:** Rust, Smithay, knuffel KDL config, GLES2 fragment shader, the `Animation` clock, the existing nested headless smoke harness.

**Spec:** `docs/specs/2026-09-05-ring-light-focus-response-design.md`. Task record: `material-26dd8a`, piece of `material-d1f471`.

## Global Constraints

- Lengths are logical pixels; colors reach the shader in linear RGB via `color_linear`; the fingerprint quantizes level, accent channels, presence, and focus to 1/256, breath to 1/32, drift phase to 1/1024 radian.
- Time: drift buckets and boundaries use the unadjusted monotonic clock (`Clock::now_unadjusted`); the focus crossfade uses `Clock::now` like the level crossfade.
- Drift: period 10 s, phase = clock in period plus the per-window jelly seed, boundaries aligned to the clock alone. Rate 0, `signal { motion "off" }`, and `animations { off }` pin the phase to 0 and report no boundary; `"reduced"` halves the rate.
- Filament constants stay constants: depth fraction 0.6 of thickness, aberration scale 0.1, halo 0.3 at `inset + 2` with width 9, attenuation exponent 0.2, jelly breath `1 + 2 * activity`.
- Defaults: `focus "ring-light"`, `ring-inset 5`, `ring-width 2.6`, `ring-color "#ccccff"`, `ring-drift-hz 15`, `light-ior 6`.
- Config errors are whole-config errors with the exact messages given in each task.
- Run affected tests with `just test-fast` from the worktree root; each task's final step runs `just test`. Commits pass the pre-commit hook (`just check`: rustfmt, clippy, tooling tests, `tasks check`). Conventional commits with scopes `feat(config)`, `feat(render)`, `feat(material)`, `docs(material)`, `test(material)`. No AI attribution trailer.
- Task lifecycle: every `### Task N` heading has a record linked with `plan:` and `step:`. Run `tasks start <id>` before a task; its final commit runs `tasks done <id> "<result>"` and stages `tasks/`. The parent `material-26dd8a` closes in Task 7's final step after Task 7's own record.

| Task | Record | | Task | Record |
|---|---|---|---|---|
| 1 | `material-ad58ba` | | 5 | `material-6172a1` |
| 2 | `material-181856` | | 6 | `material-be1e01` |
| 3 | `material-72d290` | | 7 | `material-23f92e` |
| 4 | `material-609cc4` | | | |

## File structure

| File | Responsibility |
| --- | --- |
| `niri-config/src/material.rs` | `FocusResponse`, new `Response` and `Glass` fields, resolved defaults, validation |
| `niri-config/src/lib.rs` | re-export, config tests |
| `src/render_helpers/signal.rs` | drift oscillator, `FrameInputs`, `SignalFrame` and `SignalFingerprint` fields, `tick_deadline` |
| `src/render_helpers/material.rs` | `SignalUniforms` fields and uniform upload |
| `src/render_helpers/shaders/mod.rs` | uniform registry |
| `src/render_helpers/shaders/material.frag` | refracted filament, bevel mask, specular tint by presence |
| `src/layout/tile.rs` | active flag, focus crossfade, presence crossfade, transitions term, drift deadline |
| `docs/materials/scripts/material-signals-smoke.sh` | focused-window wakeup cases |
| `docs/materials/material-config.md` | configuration reference |

---

### Task 1: Config: focus response vocabulary, filament defaults, positive width, light-ior

**Files:**
- Modify: `niri-config/src/material.rs` (enums at ~206-249, `Response` ~252-272, `ResolvedResponse` ~274-311, `Glass` ~380-411, `ResolvedGlass` ~433-475, `resolve` ~508-536, `validate` ~543-589)
- Modify: `niri-config/src/lib.rs` (re-export at 58-61, tests at 827-877 and 1093-1145)
- Modify: `src/layout/tile.rs:2199` (a `ResolvedGlass` literal in tests gains `light_ior: 6.`)

**Interfaces:**
- Produces: `niri_config::FocusResponse { None = 0, RingLight = 1 }`; `ResolvedResponse { focus: FocusResponse, ring_color: Color, ring_drift_hz: f64, .. }`; `ResolvedGlass { light_ior: f64, .. }`; validation errors `ring-width must be positive`.

- [ ] **Step 1: Write the failing config tests**

Append to the `mod tests` in `niri-config/src/lib.rs`, after `material_without_response_block_gets_builtin_default`:

```rust
    #[test]
    fn focus_response_fields_parse_and_default() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            material "tg" {
                glass { light-ior 3; }
                response "default" {
                    focus "none"
                    ring-color "#ff8800"
                    ring-drift-hz 20
                }
                response "still" { ring-drift-hz 0; }
            }
            "##,
        )])
        .unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!(m.glass.light_ior, 3.);
        let d = m.response(None);
        assert_eq!(d.focus, crate::FocusResponse::None);
        assert_eq!(d.ring_color, Color::from_rgba8_unpremul(0xff, 0x88, 0x00, 0xff));
        assert_eq!(d.ring_drift_hz, 20.);
        let still = m.response(Some("still"));
        assert_eq!(still.focus, crate::FocusResponse::None, "inherited from default");
        assert_eq!(still.ring_drift_hz, 0.);

        let builtin = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)])
            .unwrap()
            .materials[0]
            .resolve();
        let b = builtin.response(None);
        assert_eq!(b.focus, crate::FocusResponse::RingLight);
        assert_eq!(b.ring_color, Color::from_rgba8_unpremul(0xcc, 0xcc, 0xff, 0xff));
        assert_eq!(b.ring_drift_hz, 15.);
        assert_eq!((b.ring_inset, b.ring_width), (5., 2.6));
        assert_eq!(builtin.glass.light_ior, 6.);
    }

    #[test]
    fn ring_width_must_be_positive_but_may_be_fractional() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-width 0; }; }"#,
        )]);
        assert!(err.contains("ring-width must be positive"), "{err}");
        let ok = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-width 0.5; }; }"#,
        )])
        .unwrap();
        assert_eq!(ok.materials[0].resolve().response(None).ring_width, 0.5);
    }

    #[test]
    fn focus_response_rejects_out_of_range() {
        for body in [
            r#"material "tg" { glass {}; response "default" { ring-drift-hz 31; }; }"#,
            r#"material "tg" { glass { light-ior 13; }; }"#,
            r#"material "tg" { glass { light-ior 0.5; }; }"#,
            r#"material "tg" { glass {}; response "default" { focus "glow"; }; }"#,
        ] {
            parse_files_err(&[("config.kdl", body)]);
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-fast`
Expected: compile errors, `FocusResponse` and the new fields do not exist.

- [ ] **Step 3: Add the enum, fields, defaults, and validation**

In `niri-config/src/material.rs`, after `ImpulseResponse`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FocusResponse {
    None = 0,
    #[default]
    RingLight = 1,
}
```

After the existing `response_from_str!` lines:

```rust
response_from_str!(FocusResponse, "none" => None, "ring-light" => RingLight);
```

In `Response`, after `ring_width`:

```rust
    #[knuffel(child, unwrap(argument, str))]
    pub focus: Option<FocusResponse>,
    #[knuffel(child)]
    pub ring_color: Option<Color>,
    #[knuffel(child, unwrap(argument))]
    pub ring_drift_hz: Option<FloatOrInt<0, 30>>,
```

In `ResolvedResponse`, after `ring_width: f64,`:

```rust
    pub focus: FocusResponse,
    /// Filament base color; alpha is ignored.
    pub ring_color: Color,
    /// Drift bucket rate in Hz; 0 pins the drift.
    pub ring_drift_hz: f64,
```

In `impl Default for ResolvedResponse`, change `ring_inset: 6.` to `ring_inset: 5.`, `ring_width: 2.` to `ring_width: 2.6`, and add:

```rust
            focus: FocusResponse::RingLight,
            ring_color: Color::from_rgba8_unpremul(0xcc, 0xcc, 0xff, 0xff),
            ring_drift_hz: 15.,
```

In `with_overrides`, after `ring_width`:

```rust
            focus: response.focus.unwrap_or(base.focus),
            ring_color: response.ring_color.unwrap_or(base.ring_color),
            ring_drift_hz: response.ring_drift_hz.map_or(base.ring_drift_hz, |x| x.0),
```

In `Glass`, after `offset_y`:

```rust
    /// Multiplier on the bend of the focus filament's light path. Prism's
    /// glass at ior 1.02 bends nothing visible; the filament travels the
    /// slab twice and is scattered, so its path is exaggerated.
    #[knuffel(child, unwrap(argument))]
    pub light_ior: Option<FloatOrInt<1, 12>>,
```

In `ResolvedGlass`, after `offset_y: f64,`: `pub light_ior: f64,`; in its `Default`: `light_ior: 6.,`; in `resolve`, after `offset_y`: `light_ior: g.light_ior.map_or(d.light_ior, |x| x.0),`.

In `validate`, after the `ring_fits` computation and before the `if !self.responses.is_empty() && !ring_fits` check:

```rust
        if resolved
            .responses
            .iter()
            .any(|(_, response)| response.ring_width <= 0.)
        {
            return Err(String::from("ring-width must be positive"));
        }
```

In `niri-config/src/lib.rs` line 58-61, add `FocusResponse` to the re-export list (alphabetical, after `AttentionResponse`).

- [ ] **Step 4: Fix the existing expectations for the new defaults and field**

In `niri-config/src/lib.rs`: `material_without_response_block_gets_builtin_default` asserts `(d.ring_inset, d.ring_width) == (6., 2.)`; change to `(5., 2.6)`. `material_parses_full_glass_block` builds a `ResolvedGlass` literal at ~1120; add `light_ior: 6.,` after `offset_y: 4.,`. `response_blocks_resolve_with_inheritance` sets `ring-inset 6` explicitly and asserts 6; it stays. In `src/layout/tile.rs:2199` the test `ResolvedGlass` literal gains `light_ior: 6.,` after `offset_y`.

- [ ] **Step 5: Run the tests**

Run: `just test-fast`
Expected: all pass, including the three new tests.

- [ ] **Step 6: Commit**

```bash
just test
tasks done material-ad58ba "focus, ring-color, ring-drift-hz, light-ior parse with defaults 5/2.6/#ccccff/15/6; ring-width 0 rejected"
git add niri-config/src/material.rs niri-config/src/lib.rs src/layout/tile.rs tasks/
git commit -m "feat(config): add the focus response vocabulary and light-ior"
```

---

### Task 2: Solver: drift oscillator, frame inputs, fingerprint

**Files:**
- Modify: `src/render_helpers/signal.rs` (constants at 12-19, `SignalFrame` ~91-97, `tick_deadline` ~213-221, `solve` ~225-247, `SignalFingerprint` ~252-296, tests)

**Interfaces:**
- Consumes: `ResolvedResponse.ring_drift_hz`, `SignalMotionPolicy`.
- Produces:
  - `pub const DRIFT_PERIOD: Duration` (10 s).
  - `pub fn drift_rate(hz: f64, policy: SignalMotionPolicy, animations_off: bool) -> f64`.
  - `pub fn drift(hz: f64, now: Duration, seed: f32) -> f32` (radians in `[0, TAU)`, 0 when `hz <= 0`).
  - `pub fn drift_next_boundary(hz: f64, now: Duration) -> Option<Duration>`.
  - `pub struct FrameInputs { pub level: f32, pub accent: Option<[f32; 3]>, pub presence: f32, pub focus: f32, pub drift_hz: f64 }` with `FrameInputs::quiet()`.
  - `SignalFrame { presence: f32, focus: f32, drift: f32, .. }`.
  - `pub fn solve(e: &EffectiveSignal, now: Duration, seed: f32, inputs: FrameInputs) -> SignalFrame`.
  - `pub fn tick_deadline(eff: &EffectiveSignal, in_view: bool, now: Duration, drift_hz: f64) -> Option<Duration>`.
  - `SignalFingerprint` gains `presence_q`, `focus_q`, `drift_q`.

- [ ] **Step 1: Write the failing solver tests**

Append inside `mod tests` in `src/render_helpers/signal.rs`:

```rust
    #[test]
    fn drift_rate_follows_policy_and_animations() {
        assert_eq!(drift_rate(15., P::Full, false), 15.);
        assert_eq!(drift_rate(15., P::Reduced, false), 7.5);
        assert_eq!(drift_rate(15., P::Off, false), 0.);
        assert_eq!(drift_rate(15., P::Full, true), 0.);
        assert_eq!(drift_rate(0., P::Full, false), 0.);
    }

    #[test]
    fn drift_is_pinned_at_rate_zero_and_bucketed_otherwise() {
        assert_eq!(drift(0., ms(1234), 0.3), 0.);
        assert_eq!(drift(0., ms(9999), 0.7), 0.);
        // 15 Hz buckets are 66.666 ms: two instants in one bucket agree, the next bucket differs.
        let a = drift(15., ms(100), 0.3);
        let b = drift(15., ms(130), 0.3);
        let c = drift(15., ms(140), 0.3);
        assert_eq!(a, b);
        assert_ne!(b, c);
        for t in (0..10_000).step_by(250) {
            let d = drift(15., ms(t), 0.3);
            assert!((0. ..TAU).contains(&d), "{t}: {d}");
        }
        // The seed offsets the phase, not the bucket.
        assert_ne!(drift(15., ms(100), 0.), drift(15., ms(100), 0.5));
    }

    #[test]
    fn drift_boundaries_are_clock_aligned_and_shared() {
        assert_eq!(drift_next_boundary(0., ms(100)), None);
        assert_eq!(drift_next_boundary(20., ms(0)), Some(ms(50)));
        assert_eq!(drift_next_boundary(20., ms(50)), Some(ms(100)));
        assert_eq!(drift_next_boundary(20., ms(51)), Some(ms(100)));
        // 15 Hz: 66.67 ms buckets; do not pin the last nanosecond of the float conversion.
        assert_eq!(drift_next_boundary(15., ms(0)).unwrap().as_millis(), 66);
        let days = Duration::from_secs(9 * 86_400);
        assert_eq!(
            drift_next_boundary(15., days + ms(10)).map(|n| n - days),
            drift_next_boundary(15., ms(10))
        );
    }

    #[test]
    fn solve_carries_focus_presence_and_drift() {
        let e = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let quiet = solve(&e, ms(5000), 0.1, FrameInputs::quiet());
        assert_eq!((quiet.presence, quiet.focus, quiet.drift), (0., 0., 0.));
        assert_eq!(SignalFingerprint::quantize(&quiet), SignalFingerprint::default());

        let focused = solve(
            &e,
            ms(5000),
            0.1,
            FrameInputs {
                focus: 1.,
                drift_hz: 15.,
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(focused.focus, 1.);
        assert_eq!(focused.drift, drift(15., ms(5000), 0.1));

        // Unfocused windows never carry drift, whatever the rate.
        let unfocused = solve(
            &e,
            ms(5000),
            0.1,
            FrameInputs {
                drift_hz: 15.,
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(unfocused.drift, 0.);

        let half = solve(
            &e,
            ms(5000),
            0.1,
            FrameInputs {
                accent: Some([1., 0.5, 0.]),
                presence: 0.5,
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(half.accent, Some([1., 0.5, 0.]), "accent is carried straight");
        assert_eq!(half.presence, 0.5);
    }

    #[test]
    fn fingerprint_changes_once_per_drift_bucket_and_never_when_pinned() {
        let e = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let at = |t: u64, hz: f64| {
            SignalFingerprint::quantize(&solve(
                &e,
                ms(t),
                0.1,
                FrameInputs {
                    focus: 1.,
                    drift_hz: hz,
                    ..FrameInputs::quiet()
                },
            ))
        };
        assert_eq!(at(100, 20.), at(149, 20.));
        assert_ne!(at(149, 20.), at(150, 20.));
        assert_eq!(at(100, 0.), at(7000, 0.));
        assert_ne!(at(100, 0.), SignalFingerprint::default(), "focus itself is fingerprinted");
    }

    #[test]
    fn tick_deadline_includes_drift_when_in_view() {
        let quiet = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        assert_eq!(tick_deadline(&quiet, true, ms(100), 0.), None);
        assert_eq!(tick_deadline(&quiet, true, ms(100), 20.), Some(ms(150)));
        assert_eq!(tick_deadline(&quiet, false, ms(100), 20.), None);
        let breathing = EffectiveSignal {
            motion: M::Breathe,
            ..quiet
        };
        // Breathe boundary at 125 ms beats the 20 Hz drift boundary at 150 ms.
        assert_eq!(tick_deadline(&breathing, true, ms(100), 20.), Some(ms(125)));
        assert_eq!(tick_deadline(&breathing, true, ms(130), 20.), Some(ms(150)));
    }
```

Also update the existing call sites in this test module: every `solve(&e, ms(..), 0.1, 0., None)` becomes `solve(&e, ms(..), 0.1, FrameInputs::quiet())`; `solve(&e, ms(80), 0., 0.75, Some([0.1, 0.2, 0.3]))` in `solve_limits_output_to_four_live_impulses` becomes

```rust
        let f = solve(
            &e,
            ms(80),
            0.,
            FrameInputs {
                level: 0.75,
                accent: Some([0.1, 0.2, 0.3]),
                presence: 1.,
                ..FrameInputs::quiet()
            },
        );
```

`tick_deadline(&sustained, true, ms(100))` calls gain a trailing `, 0.`. In `fingerprint_uses_the_documented_quantization_steps` the `SignalFrame` literal gains `presence: 1. / 512., focus: 1. / 256., drift: 1. / 2048.,` and the expected fingerprint gains `presence_q: 1, focus_q: 1, drift_q: 1,` (1/2048 radian rounds to 1 at 1/1024 quantization: `0.5` rounds away from zero to 1).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-fast`
Expected: compile errors for `drift_rate`, `drift`, `drift_next_boundary`, `FrameInputs`.

- [ ] **Step 3: Implement the oscillator, inputs, and fingerprint**

In `src/render_helpers/signal.rs`, after `FLASH_EDGE`:

```rust
/// Focus filament drift: one lap of the travelling brightness.
pub const DRIFT_PERIOD: Duration = Duration::from_millis(10_000);
```

After `SignalFrame`, replacing it with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SignalFrame {
    /// Straight (unpremultiplied) accent color; `presence` says how much of it shows.
    pub accent: Option<[f32; 3]>,
    pub level: f32,
    pub breath: f32,
    pub impulses: [ImpulseFrame; 4],
    /// Crossfaded accent presence, 0 to 1.
    pub presence: f32,
    /// Crossfaded focus, 0 to 1.
    pub focus: f32,
    /// Drift phase in radians, bucketed; 0 when unfocused or pinned.
    pub drift: f32,
}

/// The tile's crossfaded values and effective drift rate for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInputs {
    pub level: f32,
    pub accent: Option<[f32; 3]>,
    pub presence: f32,
    pub focus: f32,
    /// Effective drift bucket rate after policy; 0 pins.
    pub drift_hz: f64,
}

impl FrameInputs {
    pub fn quiet() -> Self {
        Self {
            level: 0.,
            accent: None,
            presence: 0.,
            focus: 0.,
            drift_hz: 0.,
        }
    }
}
```

After `next_boundary`:

```rust
/// Effective drift rate: the response's rate under the motion policy and
/// the global animation switch (design: Inputs and motion).
pub fn drift_rate(hz: f64, policy: SignalMotionPolicy, animations_off: bool) -> f64 {
    if animations_off {
        return 0.;
    }
    match policy {
        SignalMotionPolicy::Off => 0.,
        SignalMotionPolicy::Reduced => hz / 2.,
        SignalMotionPolicy::Full => hz,
    }
}

fn drift_bucket(hz: f64) -> Duration {
    Duration::from_secs_f64(1. / hz)
}

/// Drift phase in radians, constant within each `1 / hz` bucket aligned to
/// the absolute clock; the seed offsets the phase, not the boundary. Pinned
/// to 0 when the rate is 0 so a static filament fingerprints to a constant.
pub fn drift(hz: f64, now: Duration, seed: f32) -> f32 {
    if hz <= 0. {
        return 0.;
    }
    let bucket = drift_bucket(hz);
    let n = now.as_nanos() / bucket.as_nanos();
    let t = Duration::from_nanos((n * bucket.as_nanos()) as u64);
    let phase = in_period(t, DRIFT_PERIOD) / DRIFT_PERIOD.as_secs_f32() + seed;
    (phase.fract() * TAU).rem_euclid(TAU)
}

/// Next absolute-clock instant at which `drift` changes.
pub fn drift_next_boundary(hz: f64, now: Duration) -> Option<Duration> {
    if hz <= 0. {
        return None;
    }
    let bucket = drift_bucket(hz);
    let n = now.as_nanos() / bucket.as_nanos() + 1;
    Some(Duration::from_nanos((n * bucket.as_nanos()) as u64))
}
```

Replace `tick_deadline`:

```rust
/// Next redraw deadline for a tile: sustained signal motion and focus drift,
/// both only while the slab band is in view.
pub fn tick_deadline(
    eff: &EffectiveSignal,
    in_view: bool,
    now: Duration,
    drift_hz: f64,
) -> Option<Duration> {
    if !in_view {
        return None;
    }
    let signal = if eff.is_sustained() {
        next_boundary(eff.motion, now)
    } else {
        None
    };
    let drift = drift_next_boundary(drift_hz, now);
    match (signal, drift) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}
```

Replace `solve`'s signature and body:

```rust
/// Stage 2: pure solve (design §3). `inputs` are the crossfaded values the
/// tile already computed plus the effective drift rate.
pub fn solve(e: &EffectiveSignal, now: Duration, seed: f32, inputs: FrameInputs) -> SignalFrame {
    let mut impulses = [ImpulseFrame::default(); 4];
    let live = e.impulses.iter().filter(|i| now < i.at + IMPULSE_LIFETIME);
    for (slot, i) in impulses.iter_mut().zip(live) {
        let age = now.saturating_sub(i.at);
        *slot = ImpulseFrame {
            selector: i.selector,
            envelope: envelope(age),
            progress: progress(age),
            accent: i.accent.map(color_linear),
        };
    }
    let drift = if inputs.focus > 0. {
        drift(inputs.drift_hz, now, seed)
    } else {
        0.
    };
    SignalFrame {
        accent: inputs.accent,
        level: inputs.level,
        breath: breath(e.motion, now, seed),
        impulses,
        presence: inputs.presence,
        focus: inputs.focus,
        drift,
    }
}
```

`SignalFingerprint` gains three fields after `impulses_q`:

```rust
    presence_q: i32,
    focus_q: i32,
    drift_q: i32,
```

with `0` for each in `Default`, and in `quantize`:

```rust
            presence_q: q256(f.presence),
            focus_q: q256(f.focus),
            drift_q: (f.drift * 1024.).round() as i32,
```

- [ ] **Step 4: Fix the two non-test call sites so the crate compiles**

`src/layout/tile.rs` calls `solve(effective, now, material.jelly_seed()[0], *level, *accent)` in `material_dynamics` and `tick_deadline(eff, in_view, now)` in `signal_tick_deadline`. Task 4 rewires these properly; for now make them compile:

```rust
                let frame = solve(
                    effective,
                    now,
                    material.jelly_seed()[0],
                    FrameInputs {
                        level: *level,
                        accent: *accent,
                        presence: if accent.is_some() { 1. } else { 0. },
                        ..FrameInputs::quiet()
                    },
                );
```

and `tick_deadline(eff, in_view, now, 0.)`. Import `FrameInputs` from `crate::render_helpers::signal` where `solve` is imported. `src/render_helpers/material.rs` tests build `SignalFrame` literals (`rim_light_only_moves_under_rim_orbit`, `glass_signal_inputs_leaves_sweep_alone`, and any other `SignalFrame {` in that file): add `presence: 1.` where `accent` is `Some`, else `presence: 0.`, plus `focus: 0., drift: 0.` to each.

- [ ] **Step 5: Run the tests**

Run: `just test-fast`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
just test
tasks done material-181856 "drift oscillator with clock-aligned buckets, FrameInputs, presence/focus/drift in frame and fingerprint, drift in tick_deadline"
git add src/render_helpers/signal.rs src/layout/tile.rs src/render_helpers/material.rs tasks/
git commit -m "feat(render): add the focus drift oscillator to the signal solver"
```

---

### Task 3: Uniforms and shader registry

**Files:**
- Modify: `src/render_helpers/material.rs` (`SignalUniforms` ~256-335, uniform upload in `draw` ~835-860, tests)
- Modify: `src/render_helpers/shaders/mod.rs` (material uniform list ~153-200)

**Interfaces:**
- Consumes: `SignalFrame.presence/focus/drift`, `ResolvedResponse.focus/ring_color`, `ResolvedGlass.light_ior`.
- Produces: `SignalUniforms { focus: [f32; 2], ring_color: [f32; 3], response: [i32; 3], .. }` where `accent[3]` is presence; uniforms `mat_sig_focus` (`_2f`: focus, drift), `mat_sig_ring_color` (`_3f`, linear), `mat_sig_response` (`_3i`: accent, attention, focus), `mat_light_ior` (`_1f`).

- [ ] **Step 1: Write the failing uniform tests**

Append to `mod tests` in `src/render_helpers/material.rs`:

```rust
    #[test]
    fn signal_uniforms_carry_presence_focus_drift_and_selectors() {
        use crate::render_helpers::signal::{color_linear, SignalFrame};
        use niri_config::{FocusResponse, ResolvedResponse};

        let mut r = ResolvedResponse::default();
        let quiet = SignalUniforms::quiet(&r);
        assert_eq!(quiet.focus, [0., 0.]);
        assert_eq!(quiet.ring_color, color_linear(r.ring_color));
        assert_eq!(quiet.response, [1, 1, 1], "ring, rim-orbit, ring-light");
        assert_eq!(quiet.accent[3], 0.);

        let frame = SignalFrame {
            accent: Some([1., 0.5, 0.]),
            level: 0.,
            breath: 0.,
            impulses: Default::default(),
            presence: 0.25,
            focus: 0.75,
            drift: 1.5,
        };
        let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
        let u = SignalUniforms::from_frame(&frame, &g, &r);
        assert_eq!(u.accent, [1., 0.5, 0., 0.25], "straight rgb, presence in alpha");
        assert_eq!(u.focus, [0.75, 1.5]);

        r.focus = FocusResponse::None;
        assert_eq!(SignalUniforms::from_frame(&frame, &g, &r).response[2], 0);
    }

```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-fast`
Expected: compile errors, `focus` and `ring_color` fields do not exist.

- [ ] **Step 3: Extend the uniforms and the registry**

In `SignalUniforms`, change `pub response: [i32; 2]` to `pub response: [i32; 3]` and add after `ring`:

```rust
    /// Crossfaded focus and bucketed drift phase.
    pub focus: [f32; 2],
    /// Filament base color, linear RGB.
    pub ring_color: [f32; 3],
```

In `quiet`:

```rust
            response: [
                response.accent as i32,
                response.attention as i32,
                response.focus as i32,
            ],
            ring: [response.ring_inset as f32, response.ring_width as f32],
            focus: [0., 0.],
            ring_color: crate::render_helpers::signal::color_linear(response.ring_color),
```

In `from_frame`, replace the `accent:` line and `response:` and add the two fields:

```rust
            accent: {
                let rgb = frame.accent.unwrap_or([0.; 3]);
                [rgb[0], rgb[1], rgb[2], frame.presence]
            },
            ...
            response: [
                response.accent as i32,
                response.attention as i32,
                response.focus as i32,
            ],
            ring: [response.ring_inset as f32, response.ring_width as f32],
            focus: [frame.focus, frame.drift],
            ring_color: crate::render_helpers::signal::color_linear(response.ring_color),
```

In `draw`, replace the `mat_sig_response` uniform and add three more before the closing `]);`:

```rust
            Uniform::new(
                "mat_sig_response",
                UniformValue::_3i(
                    self.signal.response[0],
                    self.signal.response[1],
                    self.signal.response[2],
                ),
            ),
            Uniform::new("mat_sig_ring", self.signal.ring),
            Uniform::new("mat_sig_focus", self.signal.focus),
            Uniform::new("mat_sig_ring_color", self.signal.ring_color),
            Uniform::new("mat_light_ior", g.light_ior as f32),
```

In `src/render_helpers/shaders/mod.rs`, change `UniformName::new("mat_sig_response", UniformType::_2i)` to `UniformType::_3i` and add after `mat_sig_ring`:

```rust
                UniformName::new("mat_sig_focus", UniformType::_2f),
                UniformName::new("mat_sig_ring_color", UniformType::_3f),
                UniformName::new("mat_light_ior", UniformType::_1f),
```

The shader still declares `uniform ivec2 mat_sig_response;`; change that declaration to `uniform ivec3 mat_sig_response;` now and add `uniform vec2 mat_sig_focus; uniform vec3 mat_sig_ring_color; uniform float mat_light_ior;` after `mat_sig_ring`, so the program links with the new registry. Task 5 uses them.

- [ ] **Step 4: Run the tests and build**

Run: `just test-fast && cargo build --release`
Expected: tests pass; the release binary builds (the shader compiles at runtime, so also run `glslangValidator -S frag <(printf '#version 100\n'; cat src/render_helpers/shaders/material.frag)` and expect exit 0).

- [ ] **Step 5: Commit**

```bash
just test
tasks done material-72d290 "SignalUniforms carry focus, drift, ring color, focus selector, presence in accent alpha; registry gains mat_sig_focus, mat_sig_ring_color, mat_light_ior"
git add src/render_helpers/material.rs src/render_helpers/shaders/mod.rs src/render_helpers/shaders/material.frag tasks/
git commit -m "feat(material): upload the focus filament uniforms"
```

---

### Task 4: Tile: active flag, focus and presence crossfades, transitions, drift deadline

**Files:**
- Modify: `src/layout/tile.rs` (`SignalCrossfade` ~316-345, fields ~130-140, `new` ~386-392, animation cleanup ~735-741, `are_transitions_ongoing` ~747-770, `update_render_elements` ~775-800, `signal_for_frame` ~450-500, `material_dynamics` ~502-550, `signal_tick_deadline` ~1992-2006)

**Interfaces:**
- Consumes: `FrameInputs`, `solve`, `tick_deadline`, `drift_rate` from Task 2; `FocusResponse` from Task 1.
- Produces: `Tile::focus_value(&self) -> f32`; `signal_frame_cache: RefCell<Option<(EffectiveSignal, FrameInputs)>>`.

- [ ] **Step 1: Write the failing tile tests**

In the `#[cfg(test)] mod tests` of `src/layout/tile.rs` (near the `ResolvedGlass` literal at ~2199), add tests for the crossfade math, which is pure:

```rust
    #[test]
    fn signal_crossfade_carries_presence_and_straight_color() {
        use crate::animation::{Animation, Clock};

        // `Clock` is shared by clone (`src/animation/clock.rs`): advancing the
        // test's handle advances the animation's.
        let mut clock = Clock::with_time(Duration::ZERO);
        let config = niri_config::MaterialSignalAnim::default().0; // 400 ms ease-out-cubic
        let anim = Animation::new(clock.clone(), 0., 1., 0., config);
        clock.set_unadjusted(Duration::from_millis(200));
        let t = anim.clamped_value() as f32;
        assert!(t > 0. && t < 1., "mid-fade: {t}");

        // Arrival: the color is the arriving color throughout, presence rises with t.
        let arriving = SignalCrossfade {
            anim,
            level_from: 0.,
            level_to: 1.,
            accent_from: None,
            accent_to: Some([1., 0.5, 0.]),
            presence_from: 0.,
            presence_to: 1.,
        };
        let (_, accent, presence) = arriving.current();
        assert_eq!(accent, Some([1., 0.5, 0.]));
        assert!((presence - t).abs() < 1e-6);

        // Expiry: the color holds the last live color, presence falls.
        let expiring = SignalCrossfade {
            accent_from: Some([1., 0.5, 0.]),
            accent_to: None,
            presence_from: 1.,
            presence_to: 0.,
            ..arriving
        };
        let (_, accent, presence) = expiring.current();
        assert_eq!(accent, Some([1., 0.5, 0.]));
        assert!((presence - (1. - t)).abs() < 1e-6);

        // Live to live: straight interpolation by t, presence stays 1.
        let changing = SignalCrossfade {
            accent_from: Some([1., 0., 0.]),
            accent_to: Some([0., 0., 1.]),
            presence_from: 1.,
            presence_to: 1.,
            ..expiring
        };
        let (_, accent, presence) = changing.current();
        assert_eq!(accent, Some([1. - t, 0., t]));
        assert_eq!(presence, 1.);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-fast`
Expected: compile error, `presence_from` does not exist and `current` returns a pair.

- [ ] **Step 3: Extend the crossfades and the tile state**

Replace `SignalCrossfade` and add `FocusCrossfade`:

```rust
#[derive(Debug)]
struct SignalCrossfade {
    anim: Animation,
    level_from: f32,
    level_to: f32,
    /// Straight colors. `None` on one side means that side has no accent;
    /// the other side's color is held for the whole fade and `presence`
    /// carries the fade.
    accent_from: Option<[f32; 3]>,
    accent_to: Option<[f32; 3]>,
    presence_from: f32,
    presence_to: f32,
}

impl SignalCrossfade {
    /// (level, straight accent color, presence) at the current clock.
    fn current(&self) -> (f32, Option<[f32; 3]>, f32) {
        let t = self.anim.clamped_value() as f32;
        let level = self.level_from + (self.level_to - self.level_from) * t;
        let accent = match (self.accent_from, self.accent_to) {
            (Some(a), Some(b)) => Some([0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        };
        let presence = self.presence_from + (self.presence_to - self.presence_from) * t;
        (level, accent, presence)
    }
}

#[derive(Debug)]
struct FocusCrossfade {
    anim: Animation,
    from: f32,
    to: f32,
}

impl FocusCrossfade {
    fn current(&self) -> f32 {
        self.from + (self.to - self.from) * self.anim.clamped_value() as f32
    }
}
```

Change the frame cache type and add two fields to `Tile`:

```rust
    signal_frame_cache: RefCell<Option<(EffectiveSignal, FrameInputs)>>,
    signal_render_visible: bool,
    /// Whether this tile was active at the last `update_render_elements`.
    active: bool,
    focus_crossfade: Option<FocusCrossfade>,
```

Initialize `active: false, focus_crossfade: None,` in `Tile::new`. Where `signal_crossfade` is cleared when done (the block ending `self.signal_crossfade = None;` near line 735), add:

```rust
        if self
            .focus_crossfade
            .as_ref()
            .is_some_and(|crossfade| crossfade.anim.is_done())
        {
            self.focus_crossfade = None;
        }
```

Add the helpers next to `signal_for_frame`:

```rust
    /// Crossfaded focus, 0 to 1.
    fn focus_value(&self) -> f32 {
        self.focus_crossfade
            .as_ref()
            .map(FocusCrossfade::current)
            .unwrap_or(if self.active { 1. } else { 0. })
    }

    /// Effective drift rate for this frame: only a focused (or fading)
    /// tile with a ring-light response drifts.
    fn focus_drift_hz(&self, response: &ResolvedResponse) -> f64 {
        use crate::render_helpers::signal::drift_rate;
        if response.focus != niri_config::FocusResponse::RingLight || self.focus_value() <= 0. {
            return 0.;
        }
        drift_rate(
            response.ring_drift_hz,
            self.options.signal.motion,
            self.options.animations.off,
        )
    }
```

In `update_render_elements`, before `let response = ...`:

```rust
        if self.active != is_active {
            let from = self.focus_value();
            self.active = is_active;
            self.focus_crossfade = Some(FocusCrossfade {
                anim: Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.material_signal.0,
                ),
                from,
                to: if is_active { 1. } else { 0. },
            });
        }
```

Rewrite `signal_for_frame` to return `Option<(EffectiveSignal, FrameInputs)>`:

```rust
    fn signal_for_frame(&mut self, response: &ResolvedResponse) -> Option<(EffectiveSignal, FrameInputs)> {
        use crate::render_helpers::signal::{color_linear, effective, level_value};

        let folded = self.window.signal(self.clock.now_unadjusted());
        let eff = folded
            .as_ref()
            .map(|folded| effective(folded, self.options.signal.motion, response))
            .unwrap_or(EffectiveSignal {
                accent: None,
                level: niri_ipc::SignalLevel::Quiet,
                motion: niri_ipc::SignalMotion::Static,
                impulses: vec![],
            });
        let target = (level_value(eff.level), eff.accent.map(color_linear));

        if self.signal_target != Some(target) {
            let (from_level, from_accent, from_presence) = self
                .signal_crossfade
                .as_ref()
                .map(SignalCrossfade::current)
                .or(self
                    .signal_target
                    .map(|(l, a)| (l, a, if a.is_some() { 1. } else { 0. })))
                .unwrap_or((0., None, 0.));
            self.signal_crossfade = Some(SignalCrossfade {
                anim: Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.material_signal.0,
                ),
                level_from: from_level,
                level_to: target.0,
                accent_from: from_accent,
                accent_to: target.1,
                presence_from: from_presence,
                presence_to: if target.1.is_some() { 1. } else { 0. },
            });
            self.signal_target = Some(target);
        }

        let focus = if response.focus == niri_config::FocusResponse::RingLight {
            self.focus_value()
        } else {
            0.
        };
        let focus_fading = self
            .focus_crossfade
            .as_ref()
            .is_some_and(|crossfade| !crossfade.anim.is_done());
        if folded.is_none() && self.signal_crossfade.is_none() && focus <= 0. && !focus_fading {
            return None;
        }

        let (level, accent, presence) = self
            .signal_crossfade
            .as_ref()
            .map(SignalCrossfade::current)
            .unwrap_or((target.0, target.1, if target.1.is_some() { 1. } else { 0. }));
        let drift_hz = self.focus_drift_hz(response);
        Some((
            eff,
            FrameInputs {
                level,
                accent,
                presence,
                focus,
                drift_hz,
            },
        ))
    }
```

In `material_dynamics`, the `Some` arm becomes:

```rust
            Some((effective, inputs)) => {
                let frame = solve(effective, now, material.jelly_seed()[0], *inputs);
```

and the `None` arm is unchanged (quiet uniforms, bit-identical for an unfocused quiet window).

In `are_transitions_ongoing`, inside the `self.signal_render_visible && (...)` group add a third alternative:

```rust
                    || self
                        .focus_crossfade
                        .as_ref()
                        .is_some_and(|crossfade| !crossfade.anim.is_done())
```

In `signal_tick_deadline`:

```rust
        let cache = self.signal_frame_cache.borrow();
        let (eff, inputs) = cache.as_ref()?;
        let bevel = self.material.as_ref()?.material().glass.bevel;
        let in_view = slab_in_view(location, self.tile_size(), bevel, view);
        tick_deadline(eff, in_view, now, inputs.drift_hz)
```

Remove the temporary `FrameInputs { .. }` from Task 2 Step 4 since `*inputs` now flows through. Fix the one remaining pattern `is_some_and(|(eff, _, _)| ...)` in `are_transitions_ongoing` to `|(eff, _)|`.

- [ ] **Step 4: Run the tests**

Run: `just test-fast`
Expected: all pass, including the crossfade test.

- [ ] **Step 5: Commit**

```bash
just test
tasks done material-609cc4 "tile records active state, crossfades focus and accent presence with straight color, reports the drift boundary, and keeps quiet unfocused windows bit-identical"
git add src/layout/tile.rs tasks/
git commit -m "feat(material): crossfade focus and accent presence on the tile"
```

---

### Task 5: Shader: refracted filament with bevel mask

**Files:**
- Modify: `src/render_helpers/shaders/material.frag` (`slabSurface` ~261-316, specular tint ~424-426, accent band ~428-439)

**Interfaces:**
- Consumes: `mat_sig_focus`, `mat_sig_ring_color`, `mat_sig_response` (ivec3), `mat_light_ior`, `mat_sig_accent.w` as presence, from Task 3.

- [ ] **Step 1: Publish the slab geometry the filament needs**

Add globals after the uniform declarations:

```glsl
// Slab geometry published by slabSurface for the focus filament.
vec2 g_center;
vec2 g_half;
vec4 g_outer_r;
```

Change `slabSurface`'s signature to

```glsl
void slabSurface(vec2 p, out float coverage, out vec3 normal, out float outerDist,
                 out float innerDist, out float chamferOut) {
```

and after `vec4 outer_r = inner_r + vec4(chamfer);` add

```glsl
    g_center = center;
    g_half = half_ext;
    g_outer_r = outer_r;
    chamferOut = chamfer;
```

After `float di = sdRoundedBox(p - inner_center, inner_half, inner_r);` add `innerDist = di;`. In `main`, declare `float innerDist; float slabChamfer;` next to `slabDist` and pass them: `slabSurface(p, coverage, surfaceNormal, slabDist, innerDist, slabChamfer);`.

- [ ] **Step 2: Add the filament band function**

Before `void main()`:

```glsl
// Focus filament (design: Rendering). The ray through p refracts at the
// perturbed normal through the light-path index and lands `depth` px into
// the slab; the band is a Gaussian of that landing point's distance from the
// outer edge around `inset`, plus a soft halo bleeding into the glass.
float filamentBand(vec2 p, vec3 n, float ior, float depth, float inset, float width) {
    vec3 refr = refract(vec3(0.0, 0.0, -1.0), n, 1.0 / ior);
    vec2 q = p + refr.xy * depth;
    float d = -sdRoundedBox(q - g_center, g_half, g_outer_r);
    float core = (d - inset) / width;
    float halo = (d - inset - 2.0) / 9.0;
    return exp(-2.0 * core * core) + 0.3 * exp(-2.0 * halo * halo);
}
```

- [ ] **Step 3: Tint the specular by presence and replace the accent band**

Change the specular tint line to multiply by presence:

```glsl
        if (mat_sig_accent.w > 0.0 && mat_sig_light.z > 0.0)
            specular = mix(specular, specular * mat_sig_accent.rgb * 2.0,
                           mat_sig_light.z * mat_sig_accent.w);
```

Replace the whole `if (mat_sig_response.x == 1 && mat_sig_accent.w > 0.0) { ... }` block with:

```glsl
        bool showAccent = mat_sig_response.x == 1 && mat_sig_accent.w > 0.0;
        bool showFocus = mat_sig_response.z == 1 && mat_sig_focus.x > 0.0;
        if ((showAccent || showFocus) && slabChamfer > 0.0) {
            // Mask at the displayed fragment, never at the refracted point:
            // exactly 0 on the face and everywhere di <= 0, 1 one px into
            // the bevel, 0 everywhere when the rendered chamfer is 0.
            float mask = smoothstep(0.0, 1.0, innerDist);
            if (mask > 0.0) {
                float inset = mat_sig_ring.x;
                float width = mat_sig_ring.y;
                float depth = mat_thickness * 0.6;
                float ior = 1.0 + (mat_ior - 1.0) * mat_light_ior;
                float ca = mat_chromatic_aberration * 0.1;
                vec3 band = vec3(
                    filamentBand(p, n, ior, depth, inset, width),
                    filamentBand(p, n, ior * (1.0 + ca), depth, inset, width),
                    filamentBand(p, n, ior * (1.0 + 2.0 * ca), depth, inset, width));
                float presence = mat_sig_accent.w;
                float pulse = mat_sig_response.y == 2 ? mat_sig_breath : 0.0;
                float accentGlow = showAccent
                    ? (0.15 + 0.35 * mat_sig_level) * (1.0 + pulse * mat_sig_level)
                    : 0.0;
                float focusGlow = 0.0;
                if (showFocus) {
                    vec3 refr = refract(vec3(0.0, 0.0, -1.0), n, 1.0 / ior);
                    vec2 q = p + refr.xy * depth - g_center;
                    float ang = atan(q.y, q.x);
                    float drift = mat_sig_focus.y;
                    float travel = sin(ang * 2.0 + drift) * sin(ang * 3.0 - drift * 0.6);
                    focusGlow = mat_sig_focus.x * 0.7 * (0.55 + 0.45 * travel);
                }
                float glow = (accentGlow * presence + focusGlow) * (1.0 + 2.0 * mat_jelly_activity);
                vec3 color = showAccent
                    ? mix(mat_sig_ring_color, mat_sig_accent.rgb, presence)
                    : mat_sig_ring_color;
                emissive += color * glow * band * mask * pow(att, vec3(0.2));
            }
        }
```

`att` is the Beer-Lambert term computed earlier in the same scope; `n` is the perturbed normal.

- [ ] **Step 4: Validate and build**

Run:

```bash
glslangValidator -S frag <(printf '#version 100\n'; cat src/render_helpers/shaders/material.frag)
cargo build --release
```

Expected: validator exit 0, build succeeds.

- [ ] **Step 5: Capture the filament on the headless host**

The spike harness's `baseline` case runs with no probe and the gradient ring on; with this build the default response draws the filament. Run:

```bash
CASES=baseline docs/materials/scripts/focus-ring-light.sh
```

Open `$NIRI_MATERIAL_WORK_ROOT/focus-ring-light-<sha>/baseline-right-focused-corner.png` and compare with the spike's `ring-right-focused-corner.png` under `focus-ring-light-fcdc5ed4`: a lavender filament mid-bevel with a halo, brighter on some edges than others, none on the face, and the unfocused left window with no filament. Check the idle sheet shows the brightness moving between frames. If the filament is absent, confirm `mat_sig_response.z` is 1 by adding `focus "ring-light"` explicitly to the harness config material block and rerun.

- [ ] **Step 6: Commit**

```bash
just test
tasks done material-6172a1 "material.frag draws the refracted filament masked to the rendered bevel; accent tints it by presence; specular tint fades with presence"
git add src/render_helpers/shaders/material.frag tasks/
git commit -m "feat(material): render the focus filament in the bevel"
```

---

### Task 6: Smoke harness focused cases and captures

**Files:**
- Modify: `docs/materials/scripts/material-signals-smoke.sh` (`write_config` ~125-135, config list ~150-160, `mode_cases` ~469-496)
- Create: `docs/materials/2026-09-05-ring-light-focus-smoke.md`

**Interfaces:**
- Consumes: the built binary with defaults `focus "ring-light"`, `ring-drift-hz 15`.

- [ ] **Step 1: Pin the drift in the existing base configs**

Every existing zero-redraw case has a focused material window. With the new default drift they would redraw 15 times a second. In `write_config`, change the material line to

```bash
material "tg" { glass {}; response "default" { ring-drift-hz 0; }; }
```

and in `write_gpu_config` likewise. The `attention-none.kdl` and `impulse-none.kdl` extra materials `tg2` gain `ring-drift-hz 0;` inside their `response "default" { ... }` blocks.

- [ ] **Step 2: Add the focused-window cases**

After the `write_config "$WORK/impulse-none.kdl"` lines add:

```bash
write_config "$WORK/drift.kdl"          'material "tg2" { glass {}; response "default" { ring-drift-hz 15; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }'
write_config "$WORK/drift-anim-off.kdl" 'material "tg2" { glass {}; response "default" { ring-drift-hz 15; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'animations { off; }'
write_config "$WORK/drift-reduced.kdl"  'material "tg2" { glass {}; response "default" { ring-drift-hz 15; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'signal { motion "reduced"; }'
write_config "$WORK/focus-none.kdl"     'material "tg2" { glass {}; response "default" { focus "none"; ring-drift-hz 15; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }'
```

Add a setup that leaves the single kitty focused with no signal:

```bash
setup_focused_quiet() { assert_eq "$(win "$WID" .is_focused)" true "focused"; }
```

Add a second setup that spawns two windows, focuses the other one, and sets no signal, so exactly one window (the other) is focused and drifting:

```bash
setup_other_focused() { spawn_kitty_to 2; OTHER=$(other_kitty); msg action focus-window --id "$OTHER"; assert_eq "$(win "$WID" .is_focused)" false "WID unfocused"; }
```

In `mode_cases`, after `steady_zero  quiet-ring ...` add:

```bash
    steady_zero  focused-static    "$WORK/base.kdl"           setup_focused_quiet
    steady_about focused-drift     "$WORK/drift.kdl"          setup_focused_quiet 300; drift_n=$STEADY_N
    steady_about focused-reduced   "$WORK/drift-reduced.kdl"  setup_focused_quiet 150
    steady_zero  focused-anim-off  "$WORK/drift-anim-off.kdl" setup_focused_quiet
    steady_zero  focus-none-drift  "$WORK/focus-none.kdl"     setup_focused_quiet
    steady_about other-focused     "$WORK/drift.kdl"          setup_other_focused "$drift_n"
```

and add `drift_n` to the `local` declaration. `other-focused` shows that with two windows only the focused one drifts: the count matches one drifting window within the usual tolerance.

- [ ] **Step 3: Run the cases mode and record**

Run: `docs/materials/scripts/material-signals-smoke.sh cases`
Expected: `cases: OK`, with `focused-drift` about 300 in 20 s, `focused-reduced` about 150, the zero cases zero. Copy `rates.txt` lines for the new cases into the evidence doc.

- [ ] **Step 4: Run the DRM acceptance gate once**

Run the retained DRM gate (`niri-experiments` `fixtures/v1-drm-smoke.sh`, per `docs/materials/2026-08-27-v1-drm-acceptance-design.md`) against this build; the default response drifts on the focused window, so the gate exercises it without a config change. Record pass or fail with the numbers it prints.

- [ ] **Step 5: Write the evidence document**

Create `docs/materials/2026-09-05-ring-light-focus-smoke.md` with: the build SHA, the `rates.txt` lines for the six new cases and the unchanged `quiet-ring` line, the DRM gate result, and the path of the Task 5 capture directory. Add to `docs/materials/README.md` after the focus ring light spike line:

```markdown
- `2026-09-05-ring-light-focus-smoke.md`: ring of light focus response wakeup and DRM evidence.
```

- [ ] **Step 6: Commit**

```bash
just test
tasks done material-be1e01 "smoke: focused-drift ~15/s, reduced ~7.5/s, static/anim-off/focus-none zero; DRM gate result recorded"
git add docs/materials/scripts/material-signals-smoke.sh docs/materials/2026-09-05-ring-light-focus-smoke.md docs/materials/README.md tasks/
git commit -m "test(material): measure focus filament wakeups in the signals smoke"
```

---

### Task 7: Docs, Prism piece, closure

**Files:**
- Modify: `docs/materials/material-config.md` (response table ~86-100, example ~106-125, ring paragraph ~138-143, motion section ~144-166)
- Modify: `docs/specs/2026-09-05-ring-light-focus-response-design.md` (status line)

- [ ] **Step 1: Document the configuration**

In the response table add rows and change the defaults:

```markdown
| `focus` | `ring-light`, `none` | `ring-light` |
| `ring-inset` | 0–128 logical px | 5 logical px |
| `ring-width` | > 0, up to 128 logical px | 2.6 logical px |
| `ring-color` | `"#rrggbb"` | `#ccccff` |
| `ring-drift-hz` | 0–30 | 15 |
```

In the example `response "default"` block add `focus "ring-light"`, `ring-color "#ccccff"`, `ring-drift-hz 15` and change the inset and width to 5 and 2.6. Replace the ring paragraph with:

```markdown
The filament is one band shared by focus and signals. `focus "ring-light"`
lights it on the focused window, in `ring-color`, drifting at
`ring-drift-hz` buckets per second (0 pins it). `accent "ring"` lets a
window signal light and tint the same band on any window. Both together show
the drifting filament in the accent color. The band sits `ring-inset` px
inward from the slab's outer edge, is refracted through the glass, and is
masked to the bevel, so it never lights the window face. Every resolved
response must satisfy `ring-inset + ring-width <= bevel` and `ring-width > 0`.
Set `focus-ring { off }` (globally or in a window rule) for material
windows so the gradient ring does not draw a second ring; non-material
windows keep whatever ring the layout configures.
```

In the glass parameter table of the same document add:

```markdown
| `light-ior` | 1–12 | 6 | multiplier on the focus filament's refraction; the background taps are unaffected |
```

In the motion section, after the `material-signal` sentence add: "The focus filament fades in and out with `material-signal` too, and `reduced` halves `ring-drift-hz` while `off` and `animations { off }` pin the drift."

- [ ] **Step 2: File the Prism piece**

Prism's niri sink (`integrations/niri/render.js` in the `prism` project) emits the glass block; it needs a `light-ior` line and a definition so the palette can tune it. It cannot land until this build is installed (the generated fragment fails `niri validate` without the grammar). From this worktree, file it in the registered `prism` project:

```bash
tasks add "Emit light-ior in the niri glass block" --project prism -p 2 --size s --tag niri -b "niri-material material-26dd8a adds glass { light-ior } (1..12, default 6), a multiplier on the focus filament's refraction. Add a definition with default 6, emit 'light-ior <v>' in every glass block integrations/niri/render.js writes, and extend test/niri-render.test.js. Lands only after the native build is installed, since niri validate rejects the line until then."
tasks note material-26dd8a "Prism follow-up: <the prism id printed above>"
```

No dependency is recorded: the material piece is complete without Prism.

- [ ] **Step 3: Update the spec status and the exploration goal**

Change the spec's status line to `**Status:** implemented; evidence in [2026-09-05-ring-light-focus-smoke.md](../materials/2026-09-05-ring-light-focus-smoke.md).` Add a note to the exploration goal: `tasks note material-d1f471 "winner shipped as material-26dd8a"`.

- [ ] **Step 4: Commit and close**

```bash
just test
tasks done material-23f92e "material-config documents focus, ring-color, ring-drift-hz, light-ior; Prism piece filed"
tasks done material-26dd8a "ring of light focus response: config, solver drift, tile crossfades, refracted filament, smoke evidence"
tasks check
git add docs/materials/material-config.md docs/specs/2026-09-05-ring-light-focus-response-design.md tasks/
git commit -m "docs(material): document the ring of light focus response"
```

`material-d1f471` closes when its owner confirms the goal is met (`tasks done material-d1f471`), after the merge.

## Self-review notes

- Spec coverage: decisions (Task 4 fade, Task 5 shared band, Task 7 docs on `focus-ring { off }`); inputs and motion (Tasks 2 and 4); accent presence and straight color (Tasks 2, 3, 4); rendering, bevel confinement, gates (Task 5); configuration (Tasks 1 and 7); redraw contract (Task 6); verification (Tasks 1 to 6).
- Type consistency: `FrameInputs` fields `level, accent, presence, focus, drift_hz` are used identically in Tasks 2 and 4; `SignalUniforms.focus: [f32; 2]` and `response: [i32; 3]` in Tasks 3 and 5; `SignalCrossfade::current` returns a triple in Task 4 only.
- Known ordering: Task 2 Step 4 leaves a temporary `FrameInputs` construction in `tile.rs` that Task 4 Step 3 replaces.
