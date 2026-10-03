# Accent Tint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the opt-in `accent-tint` response, which tints a material window's glass toward its signal accent's hue while keeping the face's density.

**Architecture:** A new response weight in `niri-config`. A pure CPU function computes the tinted attenuation coefficient in face-transmittance space. The tile's signal crossfade carries a tint chromaticity that interpolates between endpoints. `glass_signal_inputs` uploads the result through the existing `mat_attenuation_color` uniform. The shader, the uniform list and the pass order do not change.

**Tech Stack:** Rust (niri fork, smithay GLES renderer), knuffel config decoding, nextest via `just test-one` / `just test-fast`.

**Spec:** `docs/specs/2026-10-03-accent-tint-design.md` (accepted, spec review round 3). Read §5 and §6 before Tasks 2 and 3.

## Global Constraints

- Config: response field `accent-tint`, `FloatOrInt<0, 1>`, default `0` (off), inherited like every response field.
- `accent-tint` is independent of `accent`, `ring-accent` and `attention`; `accent "none"` still tints the body.
- No shader change: `main.frag`, the uniform list and the program stay untouched.
- Neutral path returns `attenuation_color.to_array_unpremul()` bit for bit, unclamped, when `w ≤ 0`, the frame has no accent, `p_f = 0`, or `Y(T_c)` underflows to 0.
- `w = accent-tint × presence`; `p_f = thickness / attenuation-distance`; `Y(x) = 0.2126 x.r + 0.7152 x.g + 0.0722 x.b`; the shader's coefficient floor is `0.001`, so `lo = 0.001 ^ p_f`.
- Tint arithmetic runs in `f64`; the accent arrives as straight linear RGB `[f32; 3]`.
- `accent_chroma(a)` is `a / Y(a)`, or `(1, 1, 1)` when `Y(a) < 1e-6`.
- During a crossfade the tint chromaticity interpolates between the endpoints' `accent_chroma`; it is never derived from the interpolated accent.
- `GlassSignalFingerprint` quantizes the uploaded attenuation color at 1/1024.
- No Prism change. Prism exposure is filed as an idea at closeout.
- Tests: `just test-one -p <crate> <filter>` while working, `just test-fast` before each commit; never call cargo or nextest directly.

## Review Focus

1. **Focus-split swap:** focused and unfocused materials with different `accent-tint`. On a focus change the tint steps to the other material's weight without a stale render (Task 5, `focus_split_materials_tint_per_material`).
2. **Reload that changes only `accent-tint`** while the accent is settled. The next render shows the new tint (Task 5, `reload_of_only_the_weight_rerenders`).
3. **`animations { off }`:** the crossfade is instant. The tint appears at once and nothing stays animating (Task 5, `animations_off_tints_at_once_and_settles`).
4. **`accent "none"` with a tint:** the band has no accent, but the body is still tinted (Task 5, `accent_none_still_tints_the_body`).
5. **Two windows, one material, different accents:** each tile tints by its own accent, and changing one leaves the other's pixels untouched (Task 5, `neighbor_accent_does_not_reach_a_window`).

## File Structure

| File | Change | Responsibility |
| --- | --- | --- |
| `niri-config/src/material/mod.rs` | modify | `Response::accent_tint`, `ResolvedResponse::accent_tint`, default and inheritance |
| `niri-config/src/lib.rs` | modify (tests) | parse, default, inheritance, range |
| `src/render_helpers/signal.rs` | modify | `luminance`, `accent_chroma`; `tint_chroma` on `FrameInputs` and `SignalFrame` |
| `src/render_helpers/material/tint.rs` | create | `accent_tint`: the §5 model, pure |
| `src/render_helpers/material/mod.rs` | modify | `pub mod tint`; `GlassSignalInputs::attenuation_color`; `glass_signal_inputs(.., response)`; upload; fingerprint |
| `src/layout/tile.rs` | modify | `CrossfadePoint`; tint chromaticity on `SignalCrossfade`; `crossfade_origin`; `signal_for_frame`; caller of `glass_signal_inputs` |
| `src/tests/accent_tint.rs` | create | headless render tests and the owner-review dump |
| `src/tests/mod.rs` | modify | register `accent_tint` |
| `src/tests/ring_pair.rs` | modify | make `window_rect` `pub(super)` |
| `docs/materials/material-config.md` | modify | response-table row and behavior paragraph |
| `docs/materials/render-pipeline.md` | modify | stage 4 parameters, §5 table, related designs |

---

### Task 1: The `accent-tint` response field

**Files:**
- Modify: `niri-config/src/material/mod.rs` (`Response` near line 289, `ResolvedResponse` near line 330, `Default` near line 358, `with_overrides` near line 381)
- Test: `niri-config/src/lib.rs` (tests module, next to `ring_beam_noise_defaults_off_bounds_and_inherits`)

**Interfaces:**
- Produces: `niri_config::ResolvedResponse::accent_tint: f64` (0–1, default 0).

- [ ] **Step 1: Write the failing test**

Add to the tests module in `niri-config/src/lib.rs`, after `ring_beam_noise_defaults_off_bounds_and_inherits`:

```rust
    #[test]
    fn accent_tint_defaults_off_bounds_and_inherits() {
        let parsed = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)]).unwrap();
        let d = parsed.materials[0].resolve().response(None);
        assert_eq!(d.accent_tint, 0., "off unless asked: every existing config is unchanged");

        let parsed = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { accent "none"; accent-tint 0.4; }; response "still" {}; response "bare" { accent-tint 0; }; }"#,
        )])
        .unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!(m.response(None).accent_tint, 0.4);
        assert_eq!(m.response(Some("still")).accent_tint, 0.4, "inherits from default");
        assert_eq!(m.response(Some("bare")).accent_tint, 0.);
        assert_eq!(
            m.response(None).accent,
            niri_config::AccentResponse::None,
            "independent of the accent selector"
        );

        for line in ["accent-tint 1.5;", "accent-tint -0.1;"] {
            let err = parse_files_err(&[(
                "config.kdl",
                &format!(r#"material "tg" {{ glass {{}}; response "default" {{ {line} }}; }}"#),
            )]);
            assert!(err.contains("value must be between 0 and 1"), "{line}: {err}");
        }
    }
```

If `niri_config::AccentResponse` does not resolve inside the crate's own tests, use `crate::AccentResponse` (match whatever path neighboring tests use).

- [ ] **Step 2: Run test to verify it fails**

Run: `just test-one -p niri-config accent_tint_defaults_off_bounds_and_inherits`
Expected: FAIL to compile: no field `accent_tint` on `ResolvedResponse`.

- [ ] **Step 3: Implement**

In `Response`, after the `ring_accent` field:

```rust
    #[knuffel(child, unwrap(argument))]
    pub accent_tint: Option<FloatOrInt<0, 1>>,
```

In `ResolvedResponse`, after `ring_accent`:

```rust
    /// Glass-body tint toward the signal accent's hue, 0–1; zero is off.
    /// Independent of `accent`, which governs the band (accent-tint design §3–§4).
    pub accent_tint: f64,
```

In `impl Default for ResolvedResponse`, after `ring_accent: 1.,`:

```rust
            accent_tint: 0.,
```

In `with_overrides`, after the `ring_accent` line:

```rust
            accent_tint: response.accent_tint.map_or(base.accent_tint, |x| x.0),
```

- [ ] **Step 4: Run test to verify it passes**

Run: `just test-one -p niri-config accent_tint_defaults_off_bounds_and_inherits`
Expected: PASS.

- [ ] **Step 5: Run the fast suite and commit**

Run: `just test-fast`
Expected: PASS (the `ResolvedResponse { .., ..Default::default() }` literals in `src/layout/tile.rs` tests compile unchanged).

```bash
git add niri-config/src/material/mod.rs niri-config/src/lib.rs
git commit -m "feat(material): accent-tint response weight (config only)"
```

---

### Task 2: Tint chromaticity and the face-density tint

**Files:**
- Modify: `src/render_helpers/signal.rs` (after `color_linear`, near line 157; tests module)
- Create: `src/render_helpers/material/tint.rs`
- Modify: `src/render_helpers/material/mod.rs` (module list near line 26)

**Interfaces:**
- Produces:
  - `crate::render_helpers::signal::luminance(c: [f32; 3]) -> f32`
  - `crate::render_helpers::signal::accent_chroma(accent: [f32; 3]) -> [f32; 3]`
  - `crate::render_helpers::material::tint::accent_tint(base: niri_config::Color, thickness: f64, attenuation_distance: f64, chroma: Option<[f32; 3]>, weight: f64) -> [f32; 4]`

- [ ] **Step 1: Write the failing tests for `accent_chroma`**

Add to the tests module of `src/render_helpers/signal.rs`:

```rust
    fn hex_linear(r: u8, g: u8, b: u8) -> [f32; 3] {
        color_linear(niri_config::Color::from_rgba8_unpremul(r, g, b, 0xff))
    }

    #[test]
    fn accent_chroma_has_unit_luminance_and_ignores_brightness() {
        let orange = accent_chroma(hex_linear(0xff, 0x66, 0x00));
        assert!((luminance(orange) - 1.).abs() < 1e-6, "{orange:?}");

        // A near-black colored accent tints as strongly as a bright one.
        let red = accent_chroma(hex_linear(0xff, 0x00, 0x00));
        let dark_red = accent_chroma(hex_linear(0x03, 0x00, 0x00));
        for i in 0..3 {
            assert!((red[i] - dark_red[i]).abs() < 1e-4, "{red:?} vs {dark_red:?}");
        }

        // Black and any neutral accent give the neutral chroma.
        assert_eq!(accent_chroma([0.; 3]), [1.; 3]);
        let gray = accent_chroma(hex_linear(0x80, 0x80, 0x80));
        for v in gray {
            assert!((v - 1.).abs() < 1e-5, "{gray:?}");
        }
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `just test-one -p niri accent_chroma_has_unit_luminance`
Expected: FAIL to compile: `accent_chroma` / `luminance` not found.

- [ ] **Step 3: Implement `luminance` and `accent_chroma`**

In `src/render_helpers/signal.rs`, after `color_linear`:

```rust
/// Linear-light luminance, Rec. 709 weights.
pub fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// The tint chromaticity of a settled accent (accent-tint design §5): the
/// straight linear accent scaled to unit luminance, or neutral for black.
/// It carries hue and saturation only, never brightness.
pub fn accent_chroma(accent: [f32; 3]) -> [f32; 3] {
    let y = luminance(accent);
    if y < 1e-6 {
        [1.; 3]
    } else {
        accent.map(|v| v / y)
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `just test-one -p niri accent_chroma_has_unit_luminance`
Expected: PASS.

- [ ] **Step 5: Write the failing tests for `accent_tint`**

Create `src/render_helpers/material/tint.rs` with only the module doc and the tests (the function comes in Step 7):

```rust
//! Accent tint of the attenuation color (docs/specs/2026-10-03-accent-tint-design.md §5).

#[cfg(test)]
mod tests {
    use niri_config::Color;

    use super::*;
    use crate::render_helpers::signal::{accent_chroma, color_linear};

    fn hex(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8_unpremul(r, g, b, 0xff)
    }

    /// (name, attenuation-color, thickness, attenuation-distance): the
    /// default glass, the owner's accepted look, and a mid-density glass
    /// whose face exponent is below 1.
    fn glasses() -> [(&'static str, Color, f64, f64); 3] {
        [
            ("default", hex(0xdf, 0xe8, 0xff), 20., 60.),
            ("accepted", hex(0x0d, 0x1d, 0x1e), 31.2, 11.),
            ("mid", hex(0x7a, 0x88, 0x99), 20., 40.),
        ]
    }

    fn accents() -> [(&'static str, [f32; 3]); 7] {
        [
            ("orange", color_linear(hex(0xff, 0x66, 0x00))),
            ("blue", color_linear(hex(0x00, 0x66, 0xff))),
            ("magenta", color_linear(hex(0xff, 0x00, 0xff))),
            ("near-black red", color_linear(hex(0x03, 0x00, 0x00))),
            ("gray", color_linear(hex(0x80, 0x80, 0x80))),
            ("black", [0.; 3]),
            ("white", [1.; 3]),
        ]
    }

    fn y(c: [f64; 3]) -> f64 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    /// Face transmittance of an uploaded coefficient, clamped as the shader clamps.
    fn face(x: [f32; 4], p_f: f64) -> [f64; 3] {
        [0, 1, 2].map(|i| f64::from(x[i]).clamp(0.001, 1.).powf(p_f))
    }

    #[test]
    fn neutral_path_returns_the_configured_color_bitwise() {
        let orange = Some(accent_chroma(color_linear(hex(0xff, 0x66, 0x00))));
        for (name, base, thickness, distance) in glasses() {
            let configured = base.to_array_unpremul();
            assert_eq!(accent_tint(base, thickness, distance, orange, 0.), configured, "{name} w=0");
            assert_eq!(accent_tint(base, thickness, distance, None, 1.), configured, "{name} no accent");
            assert_eq!(accent_tint(base, 0., distance, orange, 1.), configured, "{name} zero thickness");
        }
        // A face that transmits nothing: T_c underflows to 0.
        let dark = hex(0x0d, 0x1d, 0x1e);
        assert_eq!(
            accent_tint(dark, 200., 1e-3, orange, 1.),
            dark.to_array_unpremul(),
            "underflowing face"
        );
    }

    #[test]
    fn face_luminance_is_kept_at_every_weight() {
        for (gname, base, thickness, distance) in glasses() {
            let p_f = thickness / distance;
            let untinted = y(face(base.to_array_unpremul(), p_f));
            for (aname, accent) in accents() {
                for w in [0.25, 0.5, 1.] {
                    let x = accent_tint(base, thickness, distance, Some(accent_chroma(accent)), w);
                    let rel = (y(face(x, p_f)) / untinted - 1.).abs();
                    assert!(rel < 1e-5, "{gname} {aname} w={w}: {rel}");
                    assert!(
                        x[..3].iter().all(|&v| (0.001..=1.).contains(&v)),
                        "{gname} {aname} w={w}: {x:?} outside the shader clamp"
                    );
                    assert_eq!(x[3], base.to_array_unpremul()[3], "alpha unchanged");
                }
            }
        }
    }

    #[test]
    fn full_weight_shows_the_accents_linear_hue_on_the_face() {
        let (_, base, thickness, distance) = glasses()[1];
        let p_f = thickness / distance;
        for (name, accent) in [accents()[1], accents()[2]] {
            let x = accent_tint(base, thickness, distance, Some(accent_chroma(accent)), 1.);
            let t = face(x, p_f);
            let ts: f64 = t.iter().sum();
            let a: [f64; 3] = accent.map(f64::from);
            let as_: f64 = a.iter().sum();
            for i in 0..3 {
                assert!((t[i] / ts - a[i] / as_).abs() < 1e-5, "{name}: {t:?} vs {a:?}");
            }
        }
    }

    #[test]
    fn light_glass_gives_up_saturation_not_luminance() {
        let (_, base, thickness, distance) = glasses()[0];
        let p_f = thickness / distance;
        let magenta = accent_chroma(color_linear(hex(0xff, 0x00, 0xff)));
        let t = face(accent_tint(base, thickness, distance, Some(magenta), 1.), p_f);
        let g = y(t);
        let spread = (t[0].max(t[1]).max(t[2]) - t[0].min(t[1]).min(t[2])) / g;
        assert!(spread > 0., "some hue remains: {t:?}");
        assert!(spread < 0.1, "near-white glass has little room for hue: {t:?}");
    }
}
```

In `src/render_helpers/material/mod.rs`, next to `pub mod ring;`:

```rust
pub mod tint;
```

- [ ] **Step 6: Run to verify they fail**

Run: `just test-one -p niri -E 'test(/material::tint::/)'`
Expected: FAIL to compile: `accent_tint` not found.

- [ ] **Step 7: Implement `accent_tint`**

In `src/render_helpers/material/tint.rs`, between the module doc and the tests module:

```rust
use niri_config::Color;

/// The shader's floor on the attenuation coefficient (`main.frag`, stage 4).
const COEFF_FLOOR: f64 = 0.001;

fn luminance(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// The attenuation color to upload: `base` tinted toward `chroma` by
/// `weight`, matched on the flat face's transmittance so the face passes the
/// untinted luminance of a neutral backdrop at every weight (design §5).
///
/// `chroma` is `accent_chroma` of the accent (or its crossfaded value);
/// `weight` is `accent-tint × presence`. The neutral path returns the
/// configured color bit for bit.
pub fn accent_tint(
    base: Color,
    thickness: f64,
    attenuation_distance: f64,
    chroma: Option<[f32; 3]>,
    weight: f64,
) -> [f32; 4] {
    let configured = base.to_array_unpremul();
    let Some(chroma) = chroma else {
        return configured;
    };
    let p_f = thickness / attenuation_distance;
    if weight <= 0. || p_f <= 0. {
        return configured;
    }

    let c = [0, 1, 2].map(|i| f64::from(configured[i]).clamp(COEFF_FLOOR, 1.));
    let t_c = c.map(|v| v.powf(p_f));
    let gray = luminance(t_c);
    if gray <= 0. {
        // A face that transmits nothing has no hue to show.
        return configured;
    }
    let lo = COEFF_FLOOR.powf(p_f);

    // The accent's hue at the face's density, pulled toward the gray until
    // every channel is inside [lo, 1]. The gray itself is inside, so a pull
    // always exists; it keeps luminance and gives up saturation.
    let target = chroma.map(|k| f64::from(k) * gray);
    let mut s: f64 = 1.;
    for v in target {
        if v > 1. {
            s = s.min((1. - gray) / (v - gray));
        }
        if v < lo {
            s = s.min((gray - lo) / (gray - v));
        }
    }
    let target = target.map(|v| gray + s * (v - gray));

    let w = weight.min(1.);
    let t = [0, 1, 2].map(|i| t_c[i] + w * (target[i] - t_c[i]));
    let x = t.map(|v| (v.powf(1. / p_f).clamp(COEFF_FLOOR, 1.)) as f32);
    [x[0], x[1], x[2], configured[3]]
}
```

- [ ] **Step 8: Run to verify they pass**

Run: `just test-one -p niri -E 'test(/material::tint::/) | test(accent_chroma)'`
Expected: PASS (5 tests).

- [ ] **Step 9: Fast suite and commit**

Run: `just test-fast`
Expected: PASS. Rust may warn that `accent_tint` is unused outside tests until Task 4. If `just check`'s clippy denies `dead_code`, add `#[allow(dead_code)] // wired in by glass_signal_inputs (plan Task 4)` above `accent_tint`, and remove it in Task 4.

```bash
git add src/render_helpers/signal.rs src/render_helpers/material/tint.rs src/render_helpers/material/mod.rs
git commit -m "feat(material): face-density accent tint model"
```

---

### Task 3: Crossfade the tint chromaticity between endpoints

**Files:**
- Modify: `src/render_helpers/signal.rs` (`SignalFrame` near line 101, `FrameInputs` near line 117, `FrameInputs::quiet`, `solve` near line 333; test literals at lines ~701, ~716, ~821, ~846, ~885, ~893)
- Modify: `src/render_helpers/material/mod.rs` (test `SignalFrame` literals at lines ~1679, ~1736, ~1764, ~1904, ~1955)
- Modify: `src/layout/tile.rs` (`SignalCrossfade` near line 320, `impl SignalCrossfade` near line 343, `crossfade_origin` near line 362, `signal_for_frame` near line 588; test `signal_crossfade_carries_presence_and_straight_color` near line 2841)

**Interfaces:**
- Consumes: `accent_chroma` (Task 2); `accent_tint` (Task 2, in the continuity test only).
- Produces:
  - `SignalFrame::tint_chroma: Option<[f32; 3]>` and `FrameInputs::tint_chroma: Option<[f32; 3]>`, `None` exactly when `accent` is `None`.
  - Private to `tile.rs`: `struct CrossfadePoint { level: f32, accent: Option<[f32; 3]>, presence: f32, tint_chroma: Option<[f32; 3]> }`, `CrossfadePoint::settled(level, accent)`, `SignalCrossfade::at(&self, t: f32) -> CrossfadePoint`, `SignalCrossfade::current(&self) -> CrossfadePoint`, `crossfade_origin(..) -> CrossfadePoint`.

- [ ] **Step 1: Write the failing continuity test**

Add to the tests module of `src/layout/tile.rs`, after `signal_crossfade_carries_presence_and_straight_color`:

```rust
    #[test]
    fn tint_chroma_crossfades_between_endpoints_through_black() {
        use crate::animation::{Animation, Clock};
        use crate::render_helpers::material::tint::accent_tint;
        use crate::render_helpers::signal::{accent_chroma, color_linear};

        let mut clock = Clock::with_time(Duration::ZERO);
        let config = niri_config::animations::MaterialSignalAnim::default().0;
        let black = [0.; 3];
        let orange = color_linear(niri_config::Color::from_rgba8_unpremul(0xff, 0x66, 0x00, 0xff));
        let k_orange = accent_chroma(orange);
        let fade = |from: [f32; 3], to: [f32; 3]| SignalCrossfade {
            anim: Animation::new(clock.clone(), 0., 1., 0., config),
            level_from: 0.,
            level_to: 0.,
            accent_from: Some(from),
            accent_to: Some(to),
            presence_from: 1.,
            presence_to: 1.,
            tint_from: Some(accent_chroma(from)),
            tint_to: Some(accent_chroma(to)),
        };
        let lerp = |a: [f32; 3], b: [f32; 3], t: f32| [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t);
        let up = fade(black, orange);
        let down = fade(orange, black);

        for t in [0., 1e-5, 0.25, 0.5, 0.75, 1.] {
            assert_eq!(up.at(t).tint_chroma, Some(lerp([1.; 3], k_orange, t)), "up {t}");
            assert_eq!(down.at(t).tint_chroma, Some(lerp(k_orange, [1.; 3], t)), "down {t}");
        }

        // Why the chroma is interpolated, not derived: the interpolated
        // accent from black is `t × orange`, whose chroma is already orange's.
        let derived = accent_chroma(up.at(1e-5).accent.unwrap());
        assert!((0..3).all(|i| (derived[i] - k_orange[i]).abs() < 1e-3));

        // The uploaded coefficient has no step at either end (spec §8).
        let base = niri_config::Color::from_rgba8_unpremul(0x0d, 0x1d, 0x1e, 0xff);
        let upload = |k: Option<[f32; 3]>| accent_tint(base, 31.2, 11., k, 1.);
        for fade in [&up, &down] {
            for (a, b) in [(0., 1e-5), (1. - 1e-5, 1.)] {
                let (x, y) = (upload(fade.at(a).tint_chroma), upload(fade.at(b).tint_chroma));
                let step = (0..3).map(|i| (x[i] - y[i]).abs()).fold(0., f32::max);
                assert!(step < 1e-2, "step {step} between {a} and {b}");
            }
        }

        // Interrupted mid-fade: the next fade starts from the running chroma.
        clock.set_unadjusted(Duration::from_millis(200));
        let origin = crossfade_origin(Some(&up), Some((0., Some(orange))));
        assert_eq!(origin.tint_chroma, up.current().tint_chroma);
        assert_ne!(origin.tint_chroma, Some(k_orange), "not the settled target");
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `just test-one -p niri tint_chroma_crossfades_between_endpoints_through_black`
Expected: FAIL to compile: no fields `tint_from`/`tint_to`, no method `at`.

- [ ] **Step 3: Add `tint_chroma` to the frame types**

In `src/render_helpers/signal.rs`, `SignalFrame`, after `presence`:

```rust
    /// Tint chromaticity for `accent-tint` (design §5–§6); interpolated
    /// between the crossfade's endpoints, never derived from `accent`.
    /// `None` exactly when `accent` is.
    pub tint_chroma: Option<[f32; 3]>,
```

`FrameInputs`, after `presence`:

```rust
    pub tint_chroma: Option<[f32; 3]>,
```

`FrameInputs::quiet`: add `tint_chroma: None,`. `solve`'s `SignalFrame { .. }`: add `tint_chroma: inputs.tint_chroma,`.

Update every test literal of `SignalFrame { .. }` and `FrameInputs { .. }` in `src/render_helpers/signal.rs` and `src/render_helpers/material/mod.rs`, keeping the invariant: `tint_chroma: None` where `accent: None`, and `tint_chroma: Some(crate::render_helpers::signal::accent_chroma(<same array>))` where `accent: Some(<array>)`. Find them with:

```bash
grep -n 'SignalFrame {\|FrameInputs {' src/render_helpers/signal.rs src/render_helpers/material/mod.rs
```

- [ ] **Step 4: Rework the crossfade in `tile.rs`**

Replace `struct SignalCrossfade`, `impl SignalCrossfade` and `crossfade_origin` (lines ~320–370) with:

```rust
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
    /// Tint chromaticity at each end (`accent_chroma`), held like the
    /// accent when one side has none (accent-tint design §6).
    tint_from: Option<[f32; 3]>,
    tint_to: Option<[f32; 3]>,
}

/// One point of the signal crossfade.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CrossfadePoint {
    level: f32,
    /// Straight linear accent; `presence` says how much of it shows.
    accent: Option<[f32; 3]>,
    presence: f32,
    /// Interpolated between the endpoints, never derived from `accent`;
    /// `None` exactly when `accent` is.
    tint_chroma: Option<[f32; 3]>,
}

impl CrossfadePoint {
    /// A settled target: presence 1 with an accent, else 0.
    fn settled(level: f32, accent: Option<[f32; 3]>) -> Self {
        Self {
            level,
            accent,
            presence: if accent.is_some() { 1. } else { 0. },
            tint_chroma: accent.map(crate::render_helpers::signal::accent_chroma),
        }
    }
}

/// Linear interpolation that holds the present side when the other is absent.
fn lerp_held(from: Option<[f32; 3]>, to: Option<[f32; 3]>, t: f32) -> Option<[f32; 3]> {
    match (from, to) {
        (Some(a), Some(b)) => Some([0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)),
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (None, None) => None,
    }
}

impl SignalCrossfade {
    /// The crossfade at the current clock.
    fn current(&self) -> CrossfadePoint {
        self.at(self.anim.clamped_value() as f32)
    }

    /// The crossfade at fraction `t`.
    fn at(&self, t: f32) -> CrossfadePoint {
        CrossfadePoint {
            level: self.level_from + (self.level_to - self.level_from) * t,
            accent: lerp_held(self.accent_from, self.accent_to, t),
            presence: self.presence_from + (self.presence_to - self.presence_from) * t,
            tint_chroma: lerp_held(self.tint_from, self.tint_to, t),
        }
    }
}

/// Where a new signal crossfade starts: the current point of a running
/// one (so an interrupted fade continues from where it is), else the last
/// settled target, else quiet.
fn crossfade_origin(
    running: Option<&SignalCrossfade>,
    settled: Option<(f32, Option<[f32; 3]>)>,
) -> CrossfadePoint {
    running
        .map(SignalCrossfade::current)
        .or(settled.map(|(level, accent)| CrossfadePoint::settled(level, accent)))
        .unwrap_or(CrossfadePoint::settled(0., None))
}
```

In `signal_for_frame`, replace the block from `if self.signal_target != Some(target) {` to the closing `}` of that `if` with:

```rust
        if self.signal_target != Some(target) {
            let from = crossfade_origin(self.signal_crossfade.as_ref(), self.signal_target);
            let to = CrossfadePoint::settled(target.0, target.1);
            self.signal_crossfade = Some(SignalCrossfade {
                anim: Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.material_signal.0,
                ),
                level_from: from.level,
                level_to: to.level,
                accent_from: from.accent,
                accent_to: to.accent,
                presence_from: from.presence,
                presence_to: to.presence,
                tint_from: from.tint_chroma,
                tint_to: to.tint_chroma,
            });
            self.signal_target = Some(target);
        }
```

Replace the `let (level, accent, presence) = self ... .unwrap_or(...)` statement and the returned `FrameInputs` with:

```rust
        let point = self
            .signal_crossfade
            .as_ref()
            .map(SignalCrossfade::current)
            .unwrap_or(CrossfadePoint::settled(target.0, target.1));
        // The beam's frame values need the geometry; `material_dynamics`
        // fills them on its local copy of these inputs.
        Some((
            eff,
            FrameInputs {
                level: point.level,
                accent: point.accent,
                presence: point.presence,
                tint_chroma: point.tint_chroma,
                focus,
                beam: BeamFrame::REST,
            },
        ))
```

- [ ] **Step 5: Update the existing crossfade test**

In `signal_crossfade_carries_presence_and_straight_color`:
- add `tint_from: None, tint_to: Some(crate::render_helpers::signal::accent_chroma([1., 0.5, 0.])),` to `arriving`;
- add `tint_from: arriving.tint_to, tint_to: None,` to `expiring`;
- add `tint_from: Some(crate::render_helpers::signal::accent_chroma([1., 0., 0.])), tint_to: Some(crate::render_helpers::signal::accent_chroma([0., 0., 1.])),` to `changing`;
- replace each `let (_, accent, presence) = X.current();` with `let CrossfadePoint { accent, presence, .. } = X.current();`;
- replace the two settled-origin assertions with:

```rust
        assert_eq!(
            crossfade_origin(None, Some((0.5, Some([1., 0., 0.])))),
            CrossfadePoint::settled(0.5, Some([1., 0., 0.])),
            "settled accent has presence 1"
        );
        assert_eq!(crossfade_origin(None, Some((0.5, Some([1., 0., 0.])))).presence, 1.);
        assert_eq!(crossfade_origin(None, None), CrossfadePoint::settled(0., None));
```

The assertion `assert_eq!(origin, changing.current());` compiles unchanged.

- [ ] **Step 6: Run to verify both pass**

Run: `just test-one -p niri -E 'test(tint_chroma_crossfades) | test(signal_crossfade_carries)'`
Expected: PASS.

- [ ] **Step 7: Fast suite and commit**

Run: `just test-fast`
Expected: PASS.

```bash
git add src/render_helpers/signal.rs src/render_helpers/material/mod.rs src/layout/tile.rs
git commit -m "feat(material): crossfade the accent tint chromaticity between endpoints"
```

---

### Task 4: Upload the tinted attenuation color, and document it

**Files:**
- Modify: `src/render_helpers/material/mod.rs` (`GlassSignalInputs` ~128, `quiet` ~137, `glass_signal_inputs` ~148, element uniforms ~873, `GlassSignalFingerprint` ~407; test callers of `glass_signal_inputs` ~1713, ~1745, ~1779, ~1917, ~1976, ~1991)
- Modify: `src/layout/tile.rs:771` (caller)
- Modify: `docs/materials/material-config.md`, `docs/materials/render-pipeline.md`

**Interfaces:**
- Consumes: `accent_tint` (Task 2), `SignalFrame::tint_chroma` (Task 3), `ResolvedResponse::accent_tint` (Task 1).
- Produces: `GlassSignalInputs::attenuation_color: [f32; 4]`; `glass_signal_inputs(frame: &SignalFrame, glass: &ResolvedGlass, response: &ResolvedResponse) -> GlassSignalInputs`.

- [ ] **Step 1: Write the failing tests**

Add to the tests module of `src/render_helpers/material/mod.rs`:

```rust
    fn accent_frame(accent: Option<[f32; 3]>, presence: f32) -> crate::render_helpers::signal::SignalFrame {
        crate::render_helpers::signal::SignalFrame {
            accent,
            level: 1. / 3.,
            breath: 0.,
            impulses: Default::default(),
            presence,
            tint_chroma: accent.map(crate::render_helpers::signal::accent_chroma),
            focus: 0.,
            beam: BeamFrame::REST,
        }
    }

    fn accepted_glass() -> ResolvedGlass {
        ResolvedGlass {
            thickness: 31.2,
            attenuation_color: niri_config::Color::from_rgba8_unpremul(0x0d, 0x1d, 0x1e, 0xff),
            attenuation_distance: 11.,
            ..ResolvedGlass::default()
        }
    }

    #[test]
    fn untinted_attenuation_is_the_configured_color_bitwise() {
        use niri_config::ResolvedResponse;

        let orange = Some([1., 0.133, 0.]);
        let tinted = ResolvedResponse { accent_tint: 1., ..Default::default() };
        for glass in [ResolvedGlass::default(), accepted_glass()] {
            let configured = glass.attenuation_color.to_array_unpremul();
            let cases = [
                ("weight 0", glass_signal_inputs(&accent_frame(orange, 1.), &glass, &ResolvedResponse::default())),
                ("presence 0", glass_signal_inputs(&accent_frame(orange, 0.), &glass, &tinted)),
                ("no accent", glass_signal_inputs(&accent_frame(None, 0.), &glass, &tinted)),
                ("quiet", GlassSignalInputs::quiet(&glass)),
            ];
            for (name, g) in cases {
                assert_eq!(g.attenuation_color, configured, "{name}");
            }
        }
    }

    #[test]
    fn attenuation_tint_weight_is_accent_tint_times_presence() {
        use niri_config::ResolvedResponse;

        use crate::render_helpers::material::tint::accent_tint;
        use crate::render_helpers::signal::accent_chroma;

        let glass = accepted_glass();
        let half = ResolvedResponse { accent_tint: 0.5, ..Default::default() };
        let orange = [1., 0.133, 0.];
        let blue = [0., 0.133, 1.];
        // A frame mid-way through orange → blue, and black → orange.
        for (from, to) in [(orange, blue), ([0.; 3], orange)] {
            for t in [0., 0.5, 1.] {
                let k = [0, 1, 2].map(|i| {
                    accent_chroma(from)[i] + (accent_chroma(to)[i] - accent_chroma(from)[i]) * t
                });
                let mut frame = accent_frame(Some(to), 0.5);
                frame.tint_chroma = Some(k);
                let g = glass_signal_inputs(&frame, &glass, &half);
                let expected = accent_tint(
                    glass.attenuation_color,
                    glass.thickness,
                    glass.attenuation_distance,
                    Some(k),
                    0.25,
                );
                assert_eq!(g.attenuation_color, expected, "t={t}");
            }
        }
    }

    #[test]
    fn a_weight_only_change_changes_the_glass_fingerprint() {
        use niri_config::ResolvedResponse;

        let glass = accepted_glass();
        let frame = accent_frame(Some([1., 0.133, 0.]), 1.);
        let at = |w| {
            let r = ResolvedResponse { accent_tint: w, ..Default::default() };
            GlassSignalFingerprint::quantize(&glass_signal_inputs(&frame, &glass, &r))
        };
        assert_ne!(at(0.4), at(0.6), "a reload that changes only accent-tint commits damage");
        assert_eq!(at(0.4), at(0.4));
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `just test-one -p niri -E 'test(untinted_attenuation) | test(attenuation_tint_weight) | test(weight_only_change)'`
Expected: FAIL to compile: no field `attenuation_color`; `glass_signal_inputs` takes 2 arguments.

- [ ] **Step 3: Implement**

`GlassSignalInputs`: add the field after `distortion`:

```rust
    /// `attenuation-color` after the accent tint (accent-tint design §5);
    /// the configured color, bit for bit, when untinted.
    pub attenuation_color: [f32; 4],
```

`GlassSignalInputs::quiet`: add `attenuation_color: glass.attenuation_color.to_array_unpremul(),`.

`glass_signal_inputs`: change the signature and the doc comment's last sentence, and add the field:

```rust
/// Glass-specific interpretation of a `SignalFrame` (design §6). The one
/// place glass selectors and the accent tint are read.
pub fn glass_signal_inputs(
    frame: &SignalFrame,
    glass: &ResolvedGlass,
    response: &niri_config::ResolvedResponse,
) -> GlassSignalInputs {
```

and in its returned struct:

```rust
        attenuation_color: tint::accent_tint(
            glass.attenuation_color,
            glass.thickness,
            glass.attenuation_distance,
            frame.tint_chroma,
            response.accent_tint * f64::from(frame.presence),
        ),
```

Element uniforms: replace

```rust
            Uniform::new(
                "mat_attenuation_color",
                g.attenuation_color.to_array_unpremul(),
            ),
```

with

```rust
            Uniform::new("mat_attenuation_color", self.glass_signal.attenuation_color),
```

`GlassSignalFingerprint`: add a field `attenuation_q: [i32; 3],` and in `quantize`:

```rust
            attenuation_q: [0, 1, 2]
                .map(|i| (glass_signal.attenuation_color[i] * 1024.).round() as i32),
```

Callers: `src/layout/tile.rs:771` becomes `glass_signal_inputs(&frame, glass, &response)`. The test callers in `material/mod.rs` add `&niri_config::ResolvedResponse::default()` (at line ~1745 a `ResolvedResponse` `r` is built after the call; pass `&ResolvedResponse::default()` there as well). If Task 2 added `#[allow(dead_code)]` on `accent_tint`, remove it now.

- [ ] **Step 4: Run to verify they pass**

Run: `just test-one -p niri -E 'test(untinted_attenuation) | test(attenuation_tint_weight) | test(weight_only_change) | test(glass_signal)'`
Expected: PASS, including the existing `default_glass_signal_fingerprint_matches_quiet_default_glass`.

- [ ] **Step 5: Document**

In `docs/materials/material-config.md`, signal-response table, add a row directly after the `accent` row:

```markdown
| `accent-tint` | 0–1 | 0 (off) |
```

After the paragraph that begins ``accent "ring"` lets a window signal light``, add:

```markdown
`accent-tint` tints the glass body toward the signal accent's hue,
independently of `accent`: `accent "none"` takes the accent off the band and
leaves the tint. The weight moves hue and saturation only. The flat face
transmits the same luminance as the untinted glass over a neutral backdrop,
so dark glass stays dark; a saturated backdrop shifts, brighter in the
channels the tint opens and darker in the others. The chamfer and the ring's
light pass the same tinted glass: under `accent "ring"` the band and the
focus light brighten with the tint (magenta at weight 1 by about 1.7× on
dark glass), and under `accent "none"` a near-white `ring-color` band dims.
Near-white glass has little room for hue at its own luminance, so the tint
barely shows on the default glass. Without an accent the glass is unchanged,
and the tint follows the accent's crossfade in and out. Design:
[accent tint](../specs/2026-10-03-accent-tint-design.md).
```

In `docs/materials/render-pipeline.md`:
- stage 4's Parameters cell becomes `` `attenuation-color`, `attenuation-distance`, `thickness`, response `accent-tint` (tints `attenuation-color` on the CPU before upload) ``;
- in §5's table, after the `attenuation-distance` row, add `` | response `accent-tint` | (none) | 4 | ``;
- under "Related designs", add `` - `../specs/2026-10-03-accent-tint-design.md`: the accent tint of step 4. ``

- [ ] **Step 6: Fast suite, check, commit**

Run: `just test-fast` then `just check`
Expected: both PASS.

```bash
git add src/render_helpers/material/mod.rs src/layout/tile.rs docs/materials/material-config.md docs/materials/render-pipeline.md
git commit -m "feat(material): upload the accent-tinted attenuation color"
```

---

### Task 5: Headless render tests

**Files:**
- Create: `src/tests/accent_tint.rs`
- Modify: `src/tests/mod.rs` (add `mod accent_tint;` before `mod animations;`)
- Modify: `src/tests/ring_pair.rs` (`fn window_rect` → `pub(super) fn window_rect`)

**Interfaces:**
- Consumes: `super::ring_pair::{diff, render_at, set_time, window_rect}`; `Niri::{set_window_signal, clear_window_signal, arm_signal_timer}`; `State::reload_config`.
- Produces: `src/tests/accent_tint.rs` helpers used by Task 6: `look(glass, response, background, extra) -> Config`, `ACCEPTED_GLASS`, `DEFAULT_GLASS`, `open(f, id, (w, h), argb) -> WlSurface`, `signal(f, window_index, Some(hex))`, `OUT_W`, `OUT_H`.

- [ ] **Step 1: Write the test file**

Create `src/tests/accent_tint.rs`:

```rust
//! The accent tint (docs/specs/2026-10-03-accent-tint-design.md §8): what
//! must hold for any weight. The look is judged by eye on the dumps
//! (`accent_tint_dumps`, plan Task 6).

use std::time::Duration;

use niri_config::Config;
use smithay::backend::renderer::element::Element as _;
use smithay::utils::{Logical, Rectangle};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::ring_pair::{diff, render_at, set_time, window_rect};
use super::*;
use crate::niri::SetWindowSignalArgs;
use crate::render_helpers::{RenderCtx, RenderTarget};

pub(super) const OUT_W: u16 = 1280;
pub(super) const OUT_H: u16 = 800;

/// A dark fill at 0.6 opacity (premultiplied), kitty-like: the glass shows through.
pub(super) const TRANSLUCENT: u32 = 0x990a_0c0e;
/// The same fill, opaque: every pixel bypasses the glass.
const OPAQUE: u32 = 0xff0a_0c0e;

/// The terminal-glass block of the owner's accepted look (`ring_look.rs`, `ACCEPTED`).
pub(super) const ACCEPTED_GLASS: &str = r##"
        ior 1.28
        light-ior 4.5
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 11
        chromatic-aberration 0.36
        distortion 0 scale=0.09
        roughness 0.24
        noise 0.03 type="white"
        saturation 0.95
        backdrop-blur true
        bevel 10
        offset-x -6
        offset-y -5
"##;
/// Every glass parameter at its default.
pub(super) const DEFAULT_GLASS: &str = "";

/// The crossfade, made linear so a time fraction is a crossfade fraction.
const FADE_MS: u64 = 400;
/// Well after any fade.
const REST: Duration = Duration::from_secs(4);
const REST_LATER: Duration = Duration::from_secs(5);

pub(super) fn look(glass: &str, response: &str, background: &str, extra: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        animations {{ material-signal {{ duration-ms {FADE_MS}; curve "linear"; }}; }}
        layout {{
            gaps 64
            background-color "{background}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "tg" {{
            glass {{ {glass} }}
            response "default" {{ {response} }}
        }}
        window-rule {{ material "tg"; }}
        {extra}
        "##
    ))
    .unwrap()
}

pub(super) fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16), argb: u32) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(argb);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

/// Sets (`Some`) or clears (`None`) the accent of the `index`th window.
pub(super) fn signal(f: &mut Fixture, index: usize, accent: Option<&str>) {
    use niri_ipc::{SignalLevel, SignalMotion};

    let id = f.niri().layout.windows().nth(index).unwrap().1.id().get();
    match accent {
        Some(accent) => f
            .niri()
            .set_window_signal(SetWindowSignalArgs {
                id,
                source: String::from("t"),
                accent: Some(String::from(accent)),
                level: SignalLevel::Active,
                motion: SignalMotion::Static,
                tag: None,
                ttl_ms: None,
                after_level: None,
                after_motion: None,
                until_focus: false,
            })
            .unwrap(),
        None => f.niri().clear_window_signal(id, "t").unwrap(),
    }
    f.niri_state().refresh_and_flush_clients();
}

/// One 800×500 window filled with `argb` under `config`, settled,
/// optionally with an accent; returns the fixture and its render at rest.
fn one_window(config: Config, argb: u32, accent: Option<&str>) -> (Fixture, Vec<u8>) {
    let mut f = Fixture::with_config(config);
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    open(&mut f, id, (800, 500), argb);
    set_time(&mut f, Duration::ZERO);
    if let Some(accent) = accent {
        signal(&mut f, 0, Some(accent));
    }
    f.niri_complete_animations();
    // The tile starts the signal's crossfade on the first frame that sees it.
    let _ = render_at(&mut f, REST);
    let pixels = render_at(&mut f, REST_LATER);
    (f, pixels)
}

fn rest(glass: &str, response: &str, argb: u32, accent: Option<&str>) -> Vec<u8> {
    one_window(look(glass, response, "#202020", ""), argb, accent).1
}

/// The pixels of `rect` inset by `inset`, row by row.
fn region(pixels: &[u8], rect: Rectangle<f64, Logical>, inset: f64) -> Vec<u8> {
    let x0 = (rect.loc.x + inset).ceil() as usize;
    let y0 = (rect.loc.y + inset).ceil() as usize;
    let x1 = (rect.loc.x + rect.size.w - inset).floor() as usize;
    let y1 = (rect.loc.y + rect.size.h - inset).floor() as usize;
    let stride = usize::from(OUT_W) * 4;
    (y0..y1)
        .flat_map(|y| pixels[y * stride + x0 * 4..y * stride + x1 * 4].iter().copied())
        .collect()
}

/// An output pass as the real loop renders it: element commits, and
/// whether a signal timer was armed.
fn output_pass(f: &mut Fixture, time: Duration) -> (Vec<String>, bool) {
    set_time(f, time);
    f.niri().advance_animations();
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let commits = backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target: RenderTarget::Output,
                xray: None,
                signal_ticks: None,
            };
            niri.render_to_vec(ctx, &output, false)
                .iter()
                .map(|e| format!("{:?}@{:?}", e.id(), e.current_commit()))
                .collect()
        })
        .unwrap();
    niri.arm_signal_timer(&output);
    (commits, niri.output_state[&output].signal_timer.is_some())
}

#[test]
fn zero_weight_and_default_render_identically() {
    for glass in [ACCEPTED_GLASS, DEFAULT_GLASS] {
        let absent = rest(glass, r#"accent "ring""#, TRANSLUCENT, Some("#ff6600"));
        let zero = rest(glass, "accent \"ring\"\n accent-tint 0", TRANSLUCENT, Some("#ff6600"));
        assert_eq!(diff(&absent, &zero), (0, 0), "default vs explicit 0");

        let full_quiet = rest(glass, "accent-tint 1", TRANSLUCENT, None);
        let zero_quiet = rest(glass, "accent-tint 0", TRANSLUCENT, None);
        assert_eq!(diff(&full_quiet, &zero_quiet), (0, 0), "no accent, no tint");
    }
}

#[test]
fn opaque_pixels_are_untouched() {
    let (mut f, tinted) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 1", "#202020", ""),
        OPAQUE,
        Some("#ff00ff"),
    );
    let rect = window_rect(&mut f);
    let plain = rest(ACCEPTED_GLASS, "accent-tint 0", OPAQUE, Some("#ff00ff"));
    assert_eq!(region(&tinted, rect, 16.), region(&plain, rect, 16.));
}

#[test]
fn full_weight_tints_the_slab_and_translucent_content() {
    let tinted = rest(ACCEPTED_GLASS, "accent-tint 1", TRANSLUCENT, Some("#ff6600"));
    let plain = rest(ACCEPTED_GLASS, "accent-tint 0", TRANSLUCENT, Some("#ff6600"));
    assert!(diff(&tinted, &plain).0 > 0, "no visible tint");
}

#[test]
fn accent_none_still_tints_the_body() {
    let tinted = rest(ACCEPTED_GLASS, "accent \"none\"\n accent-tint 1", TRANSLUCENT, Some("#ff6600"));
    let plain = rest(ACCEPTED_GLASS, "accent \"none\"\n accent-tint 0", TRANSLUCENT, Some("#ff6600"));
    assert!(diff(&tinted, &plain).0 > 0, "accent \"none\" removed the body tint");
}

#[test]
fn removal_returns_to_the_never_tinted_render() {
    let config = || look(ACCEPTED_GLASS, "accent-tint 1", "#202020", "");
    let (mut f, _) = one_window(config(), TRANSLUCENT, Some("#ff6600"));
    signal(&mut f, 0, None);
    let start = REST_LATER + Duration::from_secs(1);
    let _ = render_at(&mut f, start); // starts the fade out
    let _ = render_at(&mut f, start + Duration::from_millis(FADE_MS / 2));
    let _ = render_at(&mut f, start + Duration::from_millis(FADE_MS + 100));
    let after = render_at(&mut f, start + Duration::from_secs(2));

    let (_, never) = one_window(config(), TRANSLUCENT, None);
    assert_eq!(diff(&after, &never), (0, 0));
}

#[test]
fn settled_tint_neither_commits_nor_animates_nor_arms_a_timer() {
    let (mut f, _) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 1", "#202020", ""),
        TRANSLUCENT,
        Some("#ff00ff"),
    );
    let (first, _) = output_pass(&mut f, REST_LATER + Duration::from_secs(1));
    let (second, armed) = output_pass(&mut f, REST_LATER + Duration::from_secs(2));
    assert_eq!(first, second, "a settled tint commits damage");
    assert!(!armed, "a settled tint arms a signal timer");
    assert!(!f.niri().layout.are_animations_ongoing(None));
}

#[test]
fn animations_off_tints_at_once_and_settles() {
    let config = |w: &str| {
        look(ACCEPTED_GLASS, &format!("accent-tint {w}"), "#202020", "animations { off; }")
    };
    let (mut f, at_once) = one_window(config("1"), TRANSLUCENT, Some("#ff6600"));
    assert!(!f.niri().layout.are_animations_ongoing(None));
    let (_, plain) = one_window(config("0"), TRANSLUCENT, Some("#ff6600"));
    assert!(diff(&at_once, &plain).0 > 0);
}

#[test]
fn reload_of_only_the_weight_rerenders() {
    let (mut f, before) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 0.4", "#202020", ""),
        TRANSLUCENT,
        Some("#ff6600"),
    );
    f.niri_state()
        .reload_config(Ok(look(ACCEPTED_GLASS, "accent-tint 0.6", "#202020", "")));
    f.niri_state().refresh_and_flush_clients();
    let after = render_at(&mut f, REST_LATER + Duration::from_secs(1));
    assert!(diff(&before, &after).0 > 0, "the new weight never rendered");
}

#[test]
fn neighbor_accent_does_not_reach_a_window() {
    let scene = |neighbor: &str| {
        let mut f = Fixture::with_config(look(ACCEPTED_GLASS, "accent-tint 1", "#202020", ""));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (500, 400), TRANSLUCENT);
        open(&mut f, id, (500, 400), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some("#ff6600"));
        signal(&mut f, 1, Some(neighbor));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        let pixels = render_at(&mut f, REST_LATER);
        let niri = f.niri();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
        let rect = Rectangle::new(pos + tile.window_loc(), tile.animated_window_size());
        region(&pixels, rect, 24.)
    };
    assert_eq!(scene("#0066ff"), scene("#ff00ff"));
}

#[test]
fn focus_split_materials_tint_per_material() {
    // Focused windows take "on" (tinted), unfocused "off"; versus both off.
    let split = |on_weight: &str| {
        Config::parse_mem(&format!(
            r##"
            hotkey-overlay {{ skip-at-startup; }}
            layout {{ gaps 64; background-color "#202020"; focus-ring {{ off; }}; border {{ off; }}; shadow {{ off; }}; }}
            material "on" {{ glass {{ {ACCEPTED_GLASS} }}; response "default" {{ accent-tint {on_weight}; }}; }}
            material "off" {{ glass {{ {ACCEPTED_GLASS} }}; response "default" {{ accent-tint 0; }}; }}
            window-rule {{ match is-active=true; material "on"; }}
            window-rule {{ match is-active=false; material "off"; }}
            "##
        ))
        .unwrap()
    };
    let first_window = |config: Config, focus_first: bool| {
        let mut f = Fixture::with_config(config);
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (500, 400), TRANSLUCENT);
        open(&mut f, id, (500, 400), TRANSLUCENT);
        if focus_first {
            f.niri().layout.focus_left();
        }
        f.niri_state().update_keyboard_focus();
        f.niri().refresh_window_rules();
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some("#ff6600"));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        let pixels = render_at(&mut f, REST_LATER);
        let niri = f.niri();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
        let rect = Rectangle::new(pos + tile.window_loc(), tile.animated_window_size());
        region(&pixels, rect, 24.)
    };
    assert_eq!(
        first_window(split("1"), false),
        first_window(split("0"), false),
        "the unfocused material's weight applies"
    );
    assert_ne!(
        first_window(split("1"), true),
        first_window(split("0"), true),
        "the focused material's weight applies"
    );
}
```

`Layout::focus_left` is the call `src/tests/focus_swap.rs` uses; `windows().nth(0)` is the first-opened window. In `src/tests/mod.rs` add `mod accent_tint;` before `mod animations;`. In `src/tests/ring_pair.rs` make `window_rect` `pub(super)`.

- [ ] **Step 2: Run the new tests**

Run: `just test-one -p niri -E 'test(/tests::accent_tint::/)'`
Expected: PASS (10 tests). These tests check behavior that Tasks 1–4 already implemented, so a failure here is a real defect. Debug it with superpowers:systematic-debugging before changing any assertion.

- [ ] **Step 3: Prove the tests can fail**

Temporarily change `accent_tint`'s first line to `return base.to_array_unpremul();` and run the same command. Expected: `full_weight_tints_the_slab_and_translucent_content`, `accent_none_still_tints_the_body`, `animations_off_tints_at_once_and_settles`, `reload_of_only_the_weight_rerenders` and `focus_split_materials_tint_per_material` fail. Revert the change and re-run: PASS.

- [ ] **Step 4: Fast suite and commit**

Run: `just test-fast`
Expected: PASS.

```bash
git add src/tests/accent_tint.rs src/tests/mod.rs src/tests/ring_pair.rs
git commit -m "test(material): headless accent tint neutrality, opacity, settling and swaps"
```

---

### Task 6: Owner-review dumps, acceptance, and integration

**Files:**
- Modify: `src/tests/accent_tint.rs` (the ignored dump test)
- Modify: `docs/materials/material-config.md` (the recommended weight, after the owner picks it)

**Interfaces:**
- Consumes: Task 5's `look`, `open`, `signal`, `ACCEPTED_GLASS`, `DEFAULT_GLASS`, `TRANSLUCENT`, `OUT_W`, `OUT_H`, `render_at`, `set_time`.

- [ ] **Step 1: Write the dump test**

Append to `src/tests/accent_tint.rs`:

```rust
fn dump(dir: &std::path::Path, name: &str, pixels: &[u8]) {
    let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
}

/// The owner-review series (spec §8). Ignored by default; run with
/// `ACCENT_TINT_DUMP=<dir> just test-one -p niri accent_tint_dumps --run-ignored only`.
#[test]
#[ignore = "writes owner-review PNGs; set ACCENT_TINT_DUMP"]
fn accent_tint_dumps() {
    let dir = std::path::PathBuf::from(
        std::env::var_os("ACCENT_TINT_DUMP").expect("set ACCENT_TINT_DUMP=<dir>"),
    );
    std::fs::create_dir_all(&dir).unwrap();

    let glasses = [("accepted", ACCEPTED_GLASS), ("default", DEFAULT_GLASS)];
    let accents = [("orange", "#ff6600"), ("blue", "#0066ff"), ("magenta", "#ff00ff")];
    let weights = ["0", "0.25", "0.5", "1"];
    // Focused, so the ring band and its focus light are in view.
    let still = |glass: &str, response: &str, background: &str, accent: &str| {
        let mut f = Fixture::with_config(look(glass, response, background, ""));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (800, 500), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some(accent));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        render_at(&mut f, REST_LATER)
    };

    for (gname, glass) in glasses {
        for (aname, accent) in accents {
            for w in weights {
                for (mode, selector) in [("ring", "ring"), ("none", "none")] {
                    let response = format!("accent \"{selector}\"\n accent-tint {w}");
                    let name = format!("still-{gname}-{aname}-w{w}-accent-{mode}");
                    dump(&dir, &name, &still(glass, &response, "#808080", accent));
                }
                for (bname, background) in [("red", "#ff0000"), ("blue", "#0000ff")] {
                    let response = format!("accent-tint {w}");
                    let name = format!("backdrop-{gname}-{aname}-w{w}-{bname}");
                    dump(&dir, &name, &still(glass, &response, background, accent));
                }
            }
        }
    }

    // Fades at time fractions 0, ¼, ½, ¾, 1 of the linear crossfade.
    for (fname, from, to) in [
        ("orange-to-blue", Some("#ff6600"), Some("#0066ff")),
        ("black-to-orange", Some("#000000"), Some("#ff6600")),
        ("orange-to-black", Some("#ff6600"), Some("#000000")),
        ("orange-to-none", Some("#ff6600"), None),
    ] {
        let mut f = Fixture::with_config(look(ACCEPTED_GLASS, "accent-tint 1", "#808080", ""));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (800, 500), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, from);
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        signal(&mut f, 0, to);
        let start = REST_LATER;
        for quarter in 0..=4u64 {
            let at = start + Duration::from_millis(FADE_MS * quarter / 4);
            dump(&dir, &format!("fade-{fname}-q{quarter}"), &render_at(&mut f, at));
        }
    }
}
```

Run: `just test-one -p niri accent_tint_dumps --run-ignored only` without the variable.
Expected: FAIL with "set ACCENT_TINT_DUMP=<dir>". This proves the test runs only on request.

- [ ] **Step 2: Generate the dumps**

```bash
mkdir -p target/accent-tint-dumps
ACCENT_TINT_DUMP="$PWD/target/accent-tint-dumps" just test-one -p niri accent_tint_dumps --run-ignored only
ls target/accent-tint-dumps | wc -l
```

Expected: PASS, and 116 files (48 stills + 48 backdrops + 20 fades). The run is headless and takes no desktop. If ImageMagick's `montage` is installed (`command -v montage`), assemble one sheet per group for the owner:

```bash
cd target/accent-tint-dumps
for g in still-accepted still-default backdrop-accepted backdrop-default fade; do
  montage "$g"-*.png -tile 4x -geometry 640x400+4+4 -label '%t' "../accent-tint-$g.png"
done
```

- [ ] **Step 3: Commit the dump test**

Run: `just test-fast`
Expected: PASS.

```bash
git add src/tests/accent_tint.rs
git commit -m "test(material): accent tint owner-review dumps"
```

- [ ] **Step 4: Owner visual acceptance (gate)**

Park the execution task for the owner: `tasks park <task> "Owner judges the accent-tint dumps in .worktrees/<name>/target/accent-tint-dumps (sheets: target/accent-tint-*.png): hue as identity at the glass's darkness over a neutral backdrop; the brighter accent band and dimmer ring-color band; saturated-backdrop shift; the near-invisible tint on default glass; the four fades. Owner picks the weight to recommend." --waiting-on user --reason review`. On rejection, record the finding and return to the spec: a model change is a spec revision, not a plan tweak.

- [ ] **Step 5: Record the recommended weight**

After the owner accepts, append to the `accent-tint` paragraph in `docs/materials/material-config.md`: ``A weight around `<owner's pick>` reads as identity on dark glass; the default stays 0.``

Run: `just check`
Expected: PASS.

```bash
git add docs/materials/material-config.md
git commit -m "docs(material): recommended accent-tint weight"
```

- [ ] **Step 6: Close out**

- File the Prism follow-up as an idea in the Prism project: `tasks add "Expose the niri accent-tint response weight" --project prism --status idea -b "niri gained response accent-tint (0–1, default 0); see niri-material docs/specs/2026-10-03-accent-tint-design.md. Prism has no key for it yet."`
- Run `tt-report`. Then integrate the branch into `materials-26.04` by local merge (personal profile), after `just gate` passes on the merged tree.
- `tasks done` each step child, then `material-6f45a0` with the outcome, in the commit that lands them.
