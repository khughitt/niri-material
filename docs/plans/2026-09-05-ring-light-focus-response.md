# Ring of Light Focus Response Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the invisible gradient focus ring on material windows with an embedded, refracted filament of light in the bevel that fades in with focus, drifts at a bucket-timer rate, breathes with the jelly residuals, and shares its band with the signal accent ring.

**Architecture:** The filament is a generalization of the existing signal accent ring. The tile records its active state and crossfades a `focus` value through the `material-signal` animation, alongside a new crossfaded accent `presence` that lets accent color be carried straight. A new drift oscillator in the pure solver quantizes its phase to `1 / ring-drift-hz` buckets and reports boundaries into the per-output bucket timer the signals design already runs. The shader replaces the accent band with a refracted band masked to the rendered bevel.

**Tech Stack:** Rust, Smithay, knuffel KDL config, GLES2 fragment shader, the `Animation` clock, the existing nested headless smoke harness.

**Spec:** `docs/specs/2026-09-05-ring-light-focus-response-design.md`. Task record: `material-26dd8a`, piece of `material-d1f471`.

## Global Constraints

- Lengths are logical pixels; colors reach the shader in linear RGB via `color_linear`; the fingerprint quantizes level, accent channels, presence, and focus to 1/256, breath to 1/32, drift phase to 1/1024 radian.
- Time: drift buckets and boundaries use the unadjusted monotonic clock (`Clock::now_unadjusted`); the focus crossfade uses `Clock::now` like the level crossfade.
- Drift: period 10 s divided into `round(hz * 10)` buckets anchored to period starts on the unadjusted clock, so boundaries repeat exactly across any multiple of 10 s of uptime; phase = bucket fraction plus the per-window jelly seed, wrapped to `[0, 2π)`. Rate 0, `signal { motion "off" }`, and `animations { off }` pin the phase to 0 and report no boundary; `"reduced"` halves the rate. A configured rate is 0 or at least 1 Hz.
- Travel waveform, identical in the solver mirror and the shader: `sin(2a + d) * sin(3a - 2d)` with `a` the perimeter angle and `d` the drift phase; every term is 2π-periodic in `d`, so the wrap is continuous.
- Filament constants stay constants: depth fraction 0.6 of thickness, aberration scale 0.1, halo 0.3 at `inset + 2` with width 9, attenuation exponent 0.2, jelly breath `1 + 2 * activity`, shift cap 0.5 × inset on the shared light path, aberration offsets on top.
- Defaults: `focus "ring-light"`, `ring-inset 5`, `ring-width 2.6`, `ring-color "#ccccff"`, `ring-drift-hz 15`, `light-ior 6`.
- Config errors are whole-config errors with the exact messages given in each task.
- Run affected tests with `just test-fast` from the worktree root; each task's final step runs `just test`. Commits pass the pre-commit hook (`just check`: rustfmt, clippy, tooling tests, `tasks check`). Conventional commits with scopes `feat(config)`, `feat(render)`, `feat(material)`, `docs(material)`, `test(material)`. No AI attribution trailer.
- Task lifecycle: every `### Task N` heading has a record linked with `plan:` and `step:`. Run `tasks start <id>` before a task; its final commit runs `tasks done <id> "<result>"` and stages `tasks/`. The parent `material-26dd8a` stays open until the operator's DRM acceptance runs land; Task 7 closes its own record and notes the parent.

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
    fn ring_drift_hz_is_zero_or_at_least_one() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-drift-hz 0.5; }; }"#,
        )]);
        assert!(err.contains("ring-drift-hz must be 0 or at least 1"), "{err}");
        for ok in ["0", "1", "7.5", "30"] {
            let parsed = parse_files(&[(
                "config.kdl",
                &format!(r#"material "tg" {{ glass {{}}; response "default" {{ ring-drift-hz {ok}; }}; }}"#),
            )])
            .unwrap();
            assert_eq!(parsed.materials[0].resolve().response(None).ring_drift_hz, ok.parse::<f64>().unwrap());
        }
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
    /// Drift bucket rate in Hz; 0 pins the drift, otherwise at least 1.
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
        // The solver divides the 10 s period into `hz * 10` buckets; a rate
        // below 1 Hz (before the reduced-motion halving) has no sensible
        // bucket and is refused rather than clamped.
        if resolved
            .responses
            .iter()
            .any(|(_, response)| response.ring_drift_hz > 0. && response.ring_drift_hz < 1.)
        {
            return Err(String::from("ring-drift-hz must be 0 or at least 1"));
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
tasks done material-ad58ba "focus, ring-color, ring-drift-hz, light-ior parse with defaults 5/2.6/#ccccff/15/6; ring-width 0 and drift rates between 0 and 1 rejected"
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
  - `pub fn travel(angle: f32, drift: f32) -> f32`: the shader's brightness waveform, mirrored for tests.
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
        // 15 Hz: 150 buckets per 10 s period, 66.67 ms each, anchored to the period start.
        assert_eq!(drift_next_boundary(15., ms(0)).unwrap().as_millis(), 66);
        assert_eq!(drift_next_boundary(15., ms(9_990)), Some(ms(10_000)));
        // Nine days is a whole number of periods, so buckets and boundaries repeat exactly.
        let days = Duration::from_secs(9 * 86_400);
        for t in [ms(10), ms(66), ms(67), ms(4_321), ms(9_999)] {
            assert_eq!(
                drift_next_boundary(15., days + t).map(|n| n - days),
                drift_next_boundary(15., t),
                "{t:?}"
            );
            assert_eq!(drift(15., days + t, 0.3), drift(15., t, 0.3), "{t:?}");
        }
        // 7.5 Hz (reduced from 15) is 75 buckets; still exact.
        assert_eq!(drift_next_boundary(7.5, ms(0)).unwrap().as_millis(), 133);
    }

    #[test]
    fn drift_boundary_is_strictly_future_and_advances_the_bucket() {
        for hz in [15., 7.5, 20., 1., 30.] {
            let mut now = Duration::ZERO;
            for _ in 0..400 {
                let b = drift_next_boundary(hz, now).unwrap();
                assert!(b > now, "{hz} Hz at {now:?}: {b:?}");
                assert_ne!(drift(hz, b, 0.3), drift(hz, now, 0.3), "{hz} Hz at {now:?}");
                // One nanosecond earlier is still the old bucket.
                assert_eq!(
                    drift(hz, b - Duration::from_nanos(1), 0.3),
                    drift(hz, now, 0.3),
                    "{hz} Hz at {now:?}"
                );
                now = b;
            }
        }
        // The exact 15 Hz value: first nanosecond of bucket 1.
        assert_eq!(
            drift_next_boundary(15., ms(0)),
            Some(Duration::from_nanos(66_666_667))
        );
    }

    #[test]
    fn travel_is_continuous_across_the_phase_wrap() {
        for k in 0..16 {
            let a = k as f32 / 16. * TAU;
            let before = travel(a, TAU - 1e-4);
            let after = travel(a, 0.);
            assert!((before - after).abs() < 2e-3, "angle {a}: {before} vs {after}");
            assert!((travel(a, 1.) - travel(a, 1. + TAU)).abs() < 1e-3);
        }
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

/// Buckets per drift period for a rate: the rate quantized to tenths of a
/// hertz, at least one bucket. Config guarantees `hz == 0 || hz >= 1`
/// before the reduced-motion halving, so this is at least 5.
fn drift_buckets(hz: f64) -> Option<u128> {
    if hz <= 0. {
        return None;
    }
    Some(((hz * DRIFT_PERIOD.as_secs_f64()).round() as u128).max(1))
}

/// Drift phase in radians, constant within each bucket. Buckets divide the
/// 10 s period evenly and are anchored to period starts on the absolute
/// clock, in integer nanoseconds, so no rounding accumulates over uptime
/// and every drifting window on an output shares the same boundaries. The
/// seed offsets the phase, not the boundary. Pinned to 0 when the rate is
/// 0 so a static filament fingerprints to a constant.
pub fn drift(hz: f64, now: Duration, seed: f32) -> f32 {
    let Some(n) = drift_buckets(hz) else {
        return 0.;
    };
    let period = DRIFT_PERIOD.as_nanos();
    let k = (now.as_nanos() % period) * n / period;
    let phase = (k as f32 / n as f32 + seed).fract();
    (phase * TAU).rem_euclid(TAU)
}

/// Next absolute-clock instant at which `drift` changes: the first
/// nanosecond of the next bucket, so it is strictly in the future and
/// `drift` evaluated there is already the next bucket's value (ceiling
/// division; a floor would land one nanosecond early and re-arm the same
/// deadline). The `as u64` cast is exact below 584 years of uptime.
pub fn drift_next_boundary(hz: f64, now: Duration) -> Option<Duration> {
    let n = drift_buckets(hz)?;
    let period = DRIFT_PERIOD.as_nanos();
    let start = now.as_nanos() - now.as_nanos() % period;
    let k = (now.as_nanos() - start) * n / period + 1;
    Some(Duration::from_nanos((start + (period * k + n - 1) / n) as u64))
}

/// The filament's travelling brightness in `[-1, 1]`, mirrored exactly by
/// the shader: two sines of the perimeter angle whose phases are integer
/// multiples of the drift, so the `2π` wrap is continuous.
pub fn travel(angle: f32, drift: f32) -> f32 {
    (2. * angle + drift).sin() * (3. * angle - 2. * drift).sin()
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
tasks done material-181856 "drift oscillator with period-anchored buckets exact over uptime, travel mirror continuous at the wrap, FrameInputs, presence/focus/drift in frame and fingerprint, drift in tick_deadline"
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

        // All four accent/focus selector combinations reach the shader independently.
        use niri_config::AccentResponse;
        for (accent, focus, want) in [
            (AccentResponse::Ring, FocusResponse::RingLight, [1, 1]),
            (AccentResponse::Ring, FocusResponse::None, [1, 0]),
            (AccentResponse::None, FocusResponse::RingLight, [0, 1]),
            (AccentResponse::None, FocusResponse::None, [0, 0]),
        ] {
            r.accent = accent;
            r.focus = focus;
            let u = SignalUniforms::from_frame(&frame, &g, &r);
            assert_eq!([u.response[0], u.response[2]], want, "{accent:?} {focus:?}");
        }
    }

    #[test]
    fn inherited_impulse_color_fades_with_presence() {
        use crate::render_helpers::signal::{ImpulseFrame, SignalFrame};
        use niri_config::{ImpulseResponse as R, ResolvedResponse};

        let r = ResolvedResponse::default();
        let mut frame = SignalFrame {
            accent: Some([1., 0.5, 0.]),
            level: 0.,
            breath: 0.,
            impulses: Default::default(),
            presence: 0.5,
            focus: 0.,
            drift: 0.,
        };
        frame.impulses[0] = ImpulseFrame {
            selector: R::Sweep as u8,
            envelope: 1.,
            progress: 0.2,
            accent: None,
        };
        frame.impulses[1] = ImpulseFrame {
            selector: R::Sweep as u8,
            envelope: 1.,
            progress: 0.2,
            accent: Some([0., 0., 1.]),
        };
        let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
        let u = SignalUniforms::from_frame(&frame, &g, &r);
        assert_eq!(u.impulse_rgb[0], [0.5, 0.25, 0.], "inherited window accent scaled by presence");
        assert_eq!(u.impulse_rgb[1], [0., 0., 1.], "explicit impulse color untouched");

        frame.accent = None;
        frame.presence = 0.;
        let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
        let u = SignalUniforms::from_frame(&frame, &g, &r);
        assert_eq!(u.impulse_rgb[0], [1., 1., 1.], "no accent falls back to white");
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

In `from_frame`, the impulse loop line `impulse_rgb[slot] = impulse.accent.or(frame.accent).unwrap_or([1.; 3]);` becomes

```rust
            impulse_rgb[slot] = impulse
                .accent
                .or_else(|| frame.accent.map(|c| c.map(|v| v * frame.presence)))
                .unwrap_or([1.; 3]);
```

so an impulse inheriting the window accent fades with it while an explicit impulse color and the no-accent white fallback are unchanged. Then replace the `accent:` line and `response:` and add the two fields:

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
tasks done material-72d290 "SignalUniforms carry focus, drift, ring color, focus selector, presence in accent alpha, presence-scaled inherited impulse color; registry gains mat_sig_focus, mat_sig_ring_color, mat_light_ior"
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

        // Interrupted mid-fade: the next crossfade starts from the current point,
        // not from the settled target.
        let origin = crossfade_origin(Some(&changing), Some((0., None)));
        assert_eq!(origin, changing.current());
        assert_eq!(
            crossfade_origin(None, Some((0.5, Some([1., 0., 0.])))),
            (0.5, Some([1., 0., 0.]), 1.),
            "settled accent has presence 1"
        );
        assert_eq!(crossfade_origin(None, None), (0., None, 0.));
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

/// Where a new signal crossfade starts: the current point of a running
/// one (so an interrupted fade continues from where it is), else the last
/// settled target with presence 1 when it had an accent, else quiet.
fn crossfade_origin(
    running: Option<&SignalCrossfade>,
    settled: Option<(f32, Option<[f32; 3]>)>,
) -> (f32, Option<[f32; 3]>, f32) {
    running
        .map(SignalCrossfade::current)
        .or(settled.map(|(level, accent)| (level, accent, if accent.is_some() { 1. } else { 0. })))
        .unwrap_or((0., None, 0.))
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

In `update_render_elements`, directly after the existing `let response = self.material.as_ref().map(|material| material.material().response(None));`, so that a response with `focus "none"` never starts a crossfade and never sets the transitions term:

```rust
        let lights_focus = response
            .as_ref()
            .is_some_and(|response| response.focus == niri_config::FocusResponse::RingLight);
        if self.active != is_active {
            let from = self.focus_value();
            self.active = is_active;
            self.focus_crossfade = lights_focus.then(|| FocusCrossfade {
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
            let (from_level, from_accent, from_presence) =
                crossfade_origin(self.signal_crossfade.as_ref(), self.signal_target);
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
tasks done material-609cc4 "tile records active state, crossfades focus (only under focus ring-light) and accent presence with straight color from the interrupted point, reports the drift boundary, and keeps quiet unfocused windows bit-identical"
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
                    // Mirrors signal.rs `travel`: integer multiples of the
                    // drift keep the 2π wrap continuous.
                    float travel = sin(ang * 2.0 + drift) * sin(ang * 3.0 - 2.0 * drift);
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

- [ ] **Step 5: Look at the filament on the headless host**

The spike harness's `baseline` case runs with no probe and the gradient ring on; with this build the default response draws the filament. Run:

```bash
CASES=baseline docs/materials/scripts/focus-ring-light.sh
```

Open `$NIRI_MATERIAL_WORK_ROOT/focus-ring-light-<sha>/baseline-right-focused-corner.png` and compare with the spike's `ring-right-focused-corner.png` under `focus-ring-light-fcdc5ed4`: a lavender filament mid-bevel with a halo, brighter on some edges than others, none on the face, and the unfocused left window with no filament. Check the idle sheet shows the brightness moving between frames. If the filament is absent, confirm `mat_sig_response.z` is 1 by adding `focus "ring-light"` explicitly to the harness config material block and rerun. The measured checks (face confinement, mid-fade color, selectors, tiny window) are Task 6.

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
- Modify: `docs/materials/scripts/material-signals-smoke.sh` (`write_config` ~125-135, config list ~150-160, setups ~394-405, `mode_cases` ~469-496)
- Modify: `docs/materials/scripts/focus-ring-light.sh` (case list and `run_case`; the probe cases are retired now that the probe patch is not in the tree)
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
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }' \
                                         'layout { focus-ring { off; }; }'
```

`focus-none.kdl` turns the gradient ring off so nothing but the material and the client can redraw on a focus change, which is what lets the toggle case below measure the material against a control.

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

Add a toggle case proving `focus "none"` starts no crossfade: two windows, focus moved back and forth three times inside the window, costing no more than the fixture itself costs and nothing after.

A focus change is client damage before it is anything else: kitty repaints its cursor as it gains and loses activation, and the compositor must serve that whatever the material does. So the acceptance is measured, not assumed. A `toggle-control` case runs the same six toggles on the same fixture with **no material on the windows at all** (`write_no_material_config`), and `focus-none-toggle` is gated at the control's total plus 6 — the one coalescible redraw each change can request of its own focus bookkeeping.

The "nothing after" window is derived, not fixed: each `niri msg action focus-window` spawns the niri binary, so six changes take about 6 s rather than the 3 s of their sleeps, and a fixed window would straddle the toggles themselves. `during_focus_toggles` times its own sequence, waits out the client's repaint burst, and reports the span so the after-window starts where the toggles actually ended.

```bash
TOGGLE_QUIET=1.0
TOGGLE_SPAN=
during_focus_toggles() {
    local k t0; t0=$(date +%s.%N)
    for k in 1 2 3; do msg action focus-window --id "$OTHER"; sleep 0.5; msg action focus-window --id "$WID"; sleep 0.5; done
    sleep "$TOGGLE_QUIET"
    TOGGLE_SPAN=$(awk -v a="$(date +%s.%N)" -v b="$t0" 'BEGIN { printf "%.1f", a - b }')
}
toggle_control_case() {   # $1 name, $2 cfg: recorded, never gated
    run_case "$1" "$2" setup_other_focused during_focus_toggles
    STEADY_N=$(count_steady "$1")
}
focus_toggle_case() {   # $1 name, $2 cfg, $3 control total
    run_case "$1" "$2" setup_other_focused during_focus_toggles
    local total after tail bound
    total=$(count_steady "$1")
    tail=$(awk -v s="$TOGGLE_SPAN" 'BEGIN { printf "%.1f", 18 - s }')   # `during` starts 12 s into a 30 s capture
    after=$(count_window "$1" "$tail" 0)
    bound=$(( $3 + 6 ))
    [ "$total" -le "$bound" ] || { echo "FAIL: $1: $total redraws for six focus changes, control $3 plus 6 = $bound" >&2; exit 1; }
    expect_zero "$1 after" "$after"
}
```

and in `mode_cases` after `other-focused`:

```bash
toggle_control_case toggle-control "$WORK/no-material.kdl"; toggle_n=$STEADY_N
focus_toggle_case   focus-none-toggle "$WORK/focus-none.kdl" "$toggle_n"
```

- [ ] **Step 3: Run the cases mode and record**

Run: `docs/materials/scripts/material-signals-smoke.sh cases`
Expected: `cases: OK`, with `focused-drift` about 300 in 20 s, `focused-reduced` about 150, `focus-none-toggle` within 6 of `toggle-control` then zero, the zero cases zero. Copy `rates.txt` lines for the new cases into the evidence doc. Any failure reopens Task 4 or 5; it is not recorded as a result and moved past.

- [ ] **Step 4: Measured captures on the spike harness**

`docs/materials/scripts/focus-ring-light.sh` keeps its nested-host plumbing (`start_nested`, `spawn_kitty`, `shot`, the dark split glass) and its `baseline` case. Retire the probe cases: the default `CASES` becomes `baseline rest-confinement accent-midfade resize-flex selectors tiny` (`rest-confinement` writes the shared `off` frame the other cases compare against, so it has to run first), `start_nested` drops the `NIRI_FOCUS_PROBE*` variables, and `run_case` dispatches by name. Add a linear-light pixel sampler and the cases. Coordinates below are for the 1280 x 720 host with the harness layout (right window at about x 656 to 1228, y 48 to 672; slab edge 8 px outside the window on the right window's top edge, so the bevel spans about y 40 to 51 and the face starts at about y 52); confirm them once against `baseline-right-focused-corner.png` and adjust the constants at the top of the script if the geometry moved.

```bash
FIL_X=940; FIL_Y=45                       # top edge of the right window: filament row
FACE_CROP=520x560+700+90                  # face region: the right window inset by at least 40 px on every side
plin() {   # $1 image, $2 x, $3 y: linear-light r g b in 0..1 (PNG is sRGB; -colorspace RGB decodes it)
    magick "$1" -colorspace RGB -format '%[fx:p{'"$2"','"$3"'}.r] %[fx:p{'"$2"','"$3"'}.g] %[fx:p{'"$2"','"$3"'}.b]' info:
}
# Emissive contribution at a pixel: linear(with filament) - linear(same scene, filament disabled).
emissive() { paste -d' ' <(plin "$1" "$3" "$4" | tr ' ' '\n') <(plin "$2" "$3" "$4" | tr ' ' '\n') | awk '{ printf "%f ", $1 - $2 }'; }
red_fraction() { awk -v e="$1" 'BEGIN { split(e, c, " "); s = c[1] + c[2] + c[3]; if (s <= 0) { print 0; exit }; printf "%.3f", c[1] / s }'; }
face_ae() {   # $1 a, $2 b: count of differing pixels over the face region
    magick compare -metric AE \( "$1" -crop "$FACE_CROP" +repage \) \( "$2" -crop "$FACE_CROP" +repage \) null: 2>&1
}
face_max() {   # $1 a, $2 b: max per-channel difference over the face region, 0..255
    magick \( "$1" -crop "$FACE_CROP" +repage \) \( "$2" -crop "$FACE_CROP" +repage \) -compose difference -composite -format '%[fx:int(255*maxima)]' info:
}
```

Every case renders the same scene twice, once with the filament and once with it disabled (`response "default" { focus "none"; accent "none"; }`), so refraction, attenuation, Fresnel, and jelly are identical in both and only the filament differs. Rest scenes are deterministic, so the disabled render is taken at rest in its own nested instance. Every case pins the drift (`ring-drift-hz 0`) so the clock does not enter. The capture material is the dark split glass with two changes for measurability: `attenuation-color "#888888"` (neutral, so the attenuation exponent scales all channels equally and hue survives) and `chromatic-aberration 0` (so the per-channel band is identical). The material base color stays `#ccccff` (linear 0.604, 0.604, 1.0) and the accent is `#ff0000` (linear 1, 0, 0).

- `rest-confinement`: focus the right window, `shot` `on`; in the disabled instance `shot` `off`. Require `face_ae on off` to be 0: at rest, not one face pixel changes when the filament is enabled. Also require `red_fraction "$(emissive on off FIL_X FIL_Y)"` between 0.24 and 0.31 (the base color's red share is 0.273) and the emissive luminance above 0.01: the filament is present and untinted.
- `accent-midfade`: config adds `animations { slowdown 20; material-signal { duration-ms 400; curve "linear"; }; }`, so the presence fade is linear over 8 s. Focus the right window; `msg set-window-signal --id "$RIGHT" --source demo --accent '#ff0000'`; sleep 4.0; `shot` `mid` (presence between 0.46 and 0.54 given screenshot latency); sleep 6; `shot` `settled`. With `off` from `rest-confinement`, compute `f=$(red_fraction "$(emissive mid off FIL_X FIL_Y)")`. Straight color at presence 0.46 to 0.54 gives a red share between 0.476 and 0.526; the double-faded regression (RGB already scaled by presence, then mixed by presence) gives at most 0.435. Require `0.46 <= f <= 0.54`. Require the settled frame's red share at least 0.90 (color is the accent at presence 1).
- `resize-flex`: two nested hosts started back to back (a second `HOST`/`UNIT` pair), identical configs except the disabled response, `animations { slowdown 50; }`. Focus the right window in both; issue `msg action set-column-width +200` to both within the same second; sleep 3 (jelly at high flex, the 20 s resize far from settled); `shot` `flex-on` and `flex-off` back to back. **Only the rest comparison is gated**: `face_ae` of the two *rest* frames from before the resize must be 0. The two instances are not on one clock — one `niri msg` process spawn separates the two resizes and another the two screenshots — so mid-resize they sit at different points in the animation and their frames differ for reasons the filament has nothing to do with. Face confinement under flex and the jelly-breath ratio are therefore **informational; not verified by measurement**: record, with an `info` marker and never a gate, the hosts' layout skew (the horizontal shift that best aligns a strip spanning the column gap, at rest and in flex), the face `AE` and `face_max` between the two flex frames, and the filament row's emissive luminance in `flex-on` against `flex-off` as a ratio of the rest value from `rest-confinement`. A deterministic mid-flex probe is filed as a follow-up idea.
- `selectors`: four filament-enabled instances, one per response `{ accent "ring"; focus "ring-light"; }`, `{ accent "ring"; focus "none"; }`, `{ accent "none"; focus "ring-light"; }`, `{ accent "none"; focus "none"; }`, each with the red accent set on the focused right window and left to settle 2 s. Against `off`: face `AE` 0 for all four; filament emissive red share at least 0.90 for the first two (tinted by the settled accent), between 0.24 and 0.31 for the third (never tinted), and emissive luminance at most 0.005 for the fourth (nothing drawn).
- `tiny` (zero chamfer): the shader's `slabChamfer > 0.0` gate is reached only when the slab is at most 2 px on one axis, because a chamfer of 0 is otherwise unconfigurable with a filament (`ring-inset + ring-width <= bevel` with `ring-width > 0`). No client on the host produces such a window (kitty and foot have a one-cell minimum, `weston-simple-egl` is fixed at 250 px), and `niri-visual-tests` renders with `xray: None`, so materials do not draw there. Record this in the evidence document as **not verified by render**, with the gate quoted, and list it in the closure note. It does not block Task 6, and the owner decides at closure whether a tiny-client fixture is worth a follow-up task.

Run `docs/materials/scripts/focus-ring-light.sh` and require every check to pass. Keep the run directory as evidence.

- [ ] **Step 5: DRM acceptance: pinned run is the gate, drifting run is bounded**

The DRM acceptance runs are performed by the operator on VT2 (done 2026-09-05; results in the evidence document); Task 6 prepares the pinned config and documents both runs. `--run` needs an active VT2 session on seat0, which no agent session can provide, and `--prepare` refuses until two reviewable edits land in `niri-experiments` (the candidate source commit added to the `material_source_commit` allowlist, and the pinned config made the tracked `fixtures/v1-drm-smoke.kdl`).

The retained DRM gate (`niri-experiments` `fixtures/v1-drm-smoke.sh` with `v1-drm-smoke.kdl`, procedure in `docs/materials/plans/2026-08-27-v1-drm-acceptance.md`) requires paired settled frames to be byte-identical, which a drifting filament cannot satisfy, so the operator's procedure is two runs:

1. **Pinned (the acceptance gate).** Copy `v1-drm-smoke.kdl` to `v1-drm-smoke-pinned.kdl` and add `response "default" { ring-drift-hz 0; }` inside `material "frost"`; run `--prepare` against this build with that config, then `--run` and `--analyze`. Every gate, including `static-repeat`, `move-repeat`, `resize-return`, `remap-return`, and `final-return`, must pass. A failure blocks closure.
2. **Drifting (bounded to the bevel band).** Run the unmodified `v1-drm-smoke.kdl` the same way. Expected: the run completes with no compositor error in its log; every non-identity gate passes; and for each identity gate that fails, the differing pixels lie entirely in the probe's bevel band. Take the probe window rectangle `x,y,w,h` from the gate's artifact JSON (the settled probe geometry it already records) and, for each failing pair `a.png b.png`:

```bash
magick a.png b.png -compose difference -composite -threshold 0 diff.png
# Outside the band (everything but the window inflated by the 12 px bevel): must be empty.
magick diff.png -fill black -draw "rectangle $((x-12)),$((y-12)) $((x+w+12)),$((y+h+12))" -format '%[fx:int(mean*w*h)]' info:
# Inside the window body (opaque probe pixels pass through the shader untouched): must be empty.
magick diff.png -crop "${w}x${h}+${x}+${y}" +repage -format '%[fx:int(mean*w*h)]' info:
# The band itself: at most its area.
magick diff.png -format '%[fx:int(mean*w*h)]' info:
```

The first two counts must be 0 and the third at most `2 * (w + h + 24) * 12`. Any other outcome, or any non-identity failure, blocks closure.

The operator records both results with their numbers in the evidence document, and `material-be1e01` and `material-26dd8a` close after they pass.

- [ ] **Step 6: Write the evidence document**

Create `docs/materials/2026-09-05-ring-light-focus-smoke.md` with: the build SHA, the `rates.txt` lines for the seven new cases and the unchanged `quiet-ring` line, the four measured capture checks with the sampled values, both DRM runs with their numbers, and the capture directory paths. Add to `docs/materials/README.md` after the focus ring light spike line:

```markdown
- `2026-09-05-ring-light-focus-smoke.md`: ring of light focus response wakeup and DRM evidence.
```

- [ ] **Step 7: Commit**

Only when every smoke case and every measured capture check passes. The two DRM runs are the operator's; the evidence document records them as awaiting, and `material-be1e01` stays open until they pass:

```bash
just test
tasks done material-be1e01 "smoke: focused-drift 15.0/s, reduced 7.5/s, static/anim-off/focus-none/other-focused zero, focus-none-toggle 21 against a 22 control then zero; captures: zero face pixels changed at rest, mid-fade red share 0.498, all four selectors pass; resize-flex mid-resize informational, tiny not verified by render; DRM acceptance awaiting the operator's VT2 runs"
git add docs/materials/scripts/material-signals-smoke.sh docs/materials/scripts/focus-ring-light.sh docs/materials/2026-09-05-ring-light-focus-smoke.md docs/materials/README.md tasks/
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
| `ring-drift-hz` | 0, or 1–30 (quantized to tenths of a hertz) | 15 |
```

In the example `response "default"` block add `focus "ring-light"`, `ring-color "#ccccff"`, `ring-drift-hz 15` and change the inset and width to 5 and 2.6. Replace the ring paragraph with:

```markdown
The filament is one band shared by focus and signals. `focus "ring-light"`
lights it on the focused window, in `ring-color`, drifting at
`ring-drift-hz` steps per second (0 pins it; otherwise at least 1, and the
rate is quantized to tenths of a hertz so the steps divide the 10 s drift
period evenly). `accent "ring"` lets a
window signal light and tint the same band on any window. Both together show
the drifting filament in the accent color. The band sits `ring-inset` px
inward from the slab's outer edge, is refracted through the glass, and is
masked to the bevel, so it never lights the window face. Every resolved
response must satisfy `ring-inset + ring-width <= bevel` and `ring-width > 0`.
Set `focus-ring { off }` (globally or in a window rule) for material
windows so the gradient ring does not draw a second ring; non-material
windows keep whatever ring the layout configures.
```

In the glass parameter table of the same document add a `light-ior` row in the table's own five-column shape (`| \`light-ior\` | float | 6 | 1–12 | — |`), and a short paragraph under it: the parameter multiplies the bend on the focus filament's light path only, never the background taps; the light-path index is `1 + (ior - 1) * light-ior` and the calibrated look has that product near 0.12, which the default 6 gives at Prism's `ior 1.02`; denser glass saturates the shared shift, capped at half `ring-inset`, after which `light-ior` only widens the chromatic split.

Add the two validation errors Task 1 introduced to the validation list: `ring-width must be positive` and `ring-drift-hz must be 0 or at least 1`.

In the motion section, after the `material-signal` sentence add: "The focus filament fades in and out with `material-signal` too, and `reduced` halves `ring-drift-hz` while `off` and `animations { off }` pin the drift."

- [ ] **Step 2: File the Prism piece**

Prism's niri sink (`integrations/niri/render.js` in the `prism` project) emits the glass block; it needs a `light-ior` line and a definition so the palette can tune it. It cannot land until this build is installed (the generated fragment fails `niri validate` without the grammar). From this worktree, file it in the registered `prism` project:

```bash
tasks add "Emit light-ior in the niri glass block" --project prism -p 2 --size s --tag niri -b "niri-material material-26dd8a adds glass { light-ior } (1..12, default 6), a multiplier on the focus filament's refraction. Add a definition with default 6, emit 'light-ior <v>' in every glass block integrations/niri/render.js writes, and extend test/niri-render.test.js. Lands only after the native build is installed, since niri validate rejects the line until then."
tasks note material-26dd8a "Prism follow-up: <the prism id printed above>"
```

No dependency is recorded: the material piece is complete without Prism.

- [ ] **Step 3: Update the spec status and the exploration goal**

Change the spec's status line to `**Status:** implemented on design/ring-light; smoke evidence in [2026-09-05-ring-light-focus-smoke.md](../materials/2026-09-05-ring-light-focus-smoke.md); DRM acceptance awaiting the operator run described there.` Add a note to the exploration goal: `tasks note material-d1f471 "winner shipped as material-26dd8a"`.

- [ ] **Step 4: Commit and close**

```bash
just test
tasks done material-23f92e "material-config documents focus, ring-color, ring-drift-hz, light-ior; Prism piece filed"
tasks note material-26dd8a "closure pending the operator DRM run recorded in material-be1e01; everything else landed"
tasks check
git add docs/materials/material-config.md docs/specs/2026-09-05-ring-light-focus-response-design.md docs/plans/2026-09-05-ring-light-focus-response.md tasks/
git commit -m "docs(material): document the ring of light focus response"
```

Closure of `material-26dd8a` requires Task 6's DRM acceptance runs, which are the operator's on VT2 and have not happened; `material-be1e01` and the parent both stay open until they pass, and the parent is closed then rather than here. `material-d1f471` closes when its owner confirms the goal is met (`tasks done material-d1f471`), after the merge.

## Self-review notes

- Spec coverage: decisions (Task 4 fade, Task 5 shared band, Task 7 docs on `focus-ring { off }`); inputs and motion (Tasks 2 and 4); accent presence and straight color (Tasks 2, 3, 4); rendering, bevel confinement, gates (Task 5); configuration (Tasks 1 and 7); redraw contract (Task 6); verification (Tasks 1 to 6).
- Type consistency: `FrameInputs` fields `level, accent, presence, focus, drift_hz` are used identically in Tasks 2 and 4; `SignalUniforms.focus: [f32; 2]` and `response: [i32; 3]` in Tasks 3 and 5; `SignalCrossfade::current` returns a triple in Task 4 only.
- Known ordering: Task 2 Step 4 leaves a temporary `FrameInputs` construction in `tile.rs` that Task 4 Step 3 replaces.
- Review round 2: boundary uses ceiling division with a strictly-future test over 400 consecutive deadlines; captures compare filament-enabled against filament-disabled renders of the same scene in linear light with drift pinned and a neutral, aberration-free capture glass, and the mid-fade check uses a linear curve and a red-share window that rejects the double-faded regression; face confinement is measured as zero differing face pixels at rest and a two-level bound under lockstep flex; the zero-chamfer case is recorded as not renderable on this host rather than claimed; the drifting DRM run requires zero differences outside the bevel band and inside the opaque body.
- Review round 1: travel waveform made 2π-periodic with a solver mirror and wrap test; inherited impulse color scaled by presence; drift rates in (0, 1) refused; bucket math anchored to period starts so uptime repeats exactly; focus crossfade gated by the response with a toggle smoke case; measured captures for mid-fade color, face confinement under flex, the tiny-window clamp, and the selector matrix; interrupted-fade origin extracted and tested; DRM gate run pinned as the acceptance and drifting as a bounded check, and closure conditioned on passing.
