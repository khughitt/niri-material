# Noctalia glass cohesion and workspace project hue Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** niri recolors Noctalia's bar and panels to the terminal glass's face color and to the project hue held by each output's active workspace, with no Noctalia palette change.

**Architecture:** A layer rule names Noctalia's three surface-family colors and a glass material.

- **Config (`niri-config`).** Validates the rule at load, using an Oklab module and a glass face model.
- **Remap math.** A pure module turns the rule, the material and a hue state into shader uniforms.
- **Resolver.** A pure per-workspace resolver turns active windows and their folded accents into a held hue with a crossfade.
- **Rendering.** `MappedLayer` wraps its surface elements in a texture program that moves pixels near the family polyline. `Niri::update_render_elements` passes each output's blended hue to its layers.
- **Proof.** Nested-niri sheets prove the look to the owner.

**Tech Stack:** Rust (niri, `niri-config`, smithay GLES custom texture programs, knuffel), GLSL ES 1.00, nextest through `just test-one`, and bash with ImageMagick 7 for the nested sheet.

**Spec:** [docs/specs/2026-10-16-noctalia-glass-cohesion-design.md](../specs/2026-10-16-noctalia-glass-cohesion-design.md) (accepted for planning after spec review round 3, then revised after round 4). Round 4 added three things: unsupported face-lighting optics, the achromatic-accent policy, and the input-method popup exemption. Section 3's overlay claim was corrected while planning (see the spec's note there).

**Deviation from the spec, stated once:** the spec places the face model in `src/render_helpers/material/`. It lives in `niri-config/src/material/face.rs` instead, because the range check must run at config load (`niri validate` included), and only `niri-config` runs there. The niri crate imports it from there. The test that pins the model's constants against `main.frag` lives in the niri crate (Task 3), beside the shader.

## Global Constraints

- **Reference backdrop:** `B_REF = 0.2` (neutral, linear).
- **Specular floor:** `SPECULAR_FLOOR = 0.15`.
- **Coefficient floor:** `COEFF_FLOOR = 0.001`, as in `main.frag` and `tint.rs`.
- **Face:** `face = linearToSrgb(B_ref · T · (1 − f0) + 0.15 · f0)`, where `T = clamp(attenuation-color, 0.001, 1) ^ (thickness / attenuation-distance)` and `f0 = ((ior − 1) / (ior + 1))²`.
- **Supported range:** `max(T) · (1 − f0) ≤ 0.01`. The out-of-range message is exactly: `surface-remap: material "<name>" transmits <x> in its most open channel; the shell match supports dark glass up to 0.01`, with `<x>` printed `{:.3}`.
- **Supported effects:** a `surface-remap` material may not set aurora or iridescence. Both light the flat face. The message is exactly: `surface-remap: material "<name>" sets <aurora|iridescence>, which lights the flat face; the shell match supports glass without them`.
- **Guaranteed lightness bound:** within −0.047 and +0.04501 Oklab L, over neutral backdrops from 0.05 to 0.5 (linear), across IOR [1, 3].
- **Remap radii:** `r0 = 0.008`, `r1 = 0.02`.
- **Source surface:** Oklab L ≤ 0.4, otherwise a load error.
- **`hue-chroma`:** in [0, 0.1], default 0.
- **Muted accent:** an accent whose Oklab chroma is below 0.02 has no hue and resolves to neutral.
- **Hold dwell:** 1000 ms, owned by the binding. Focus timestamps play no part.
- **Crossfade:** duration and curve from `animations.material-signal`. The crossfade and switch blend interpolate target families (here the `HueBlend` coordinates, Task 3), never accents.
- **Popups:** the remap reaches xdg popups only. Input-method popups are exempt, decided in the same per-kind match that gives them the default popup rules.
- **Node replacement:** `surface-remap` nodes are complete and replaced whole by later matching rules. Fields never merge.
- **No fallback:** config errors fail config load, and no runtime failure path exists.
- **Surfaces in the sheet and the example rule:** `^noctalia-(bar-[^"]+|attached-panel|panel)$`.
- **Section 8 defaults (owner-confirmed):** hold the last hue, 1 s dwell, hue only, and the bar plus attached and floating panels.
- **Tests:** `just test-one -p <pkg> <filter>` while editing, `just test-fast` before each commit. Format with `rustup run nightly cargo fmt --all`. Never call cargo test or nextest directly.
- **Commits:** conventional commits, no attribution trailers. `tasks check` passes before every commit.

## Review Focus

These are the inputs the spec implies but no other test covers, each now pinned in its owning task:

1. **A workspace is destroyed** (an emptied dynamic workspace). The resolver forgets it, so a later workspace never inherits a stale record. Task 4: `removed_workspaces_are_forgotten`.
2. **A gray or white accent** (chroma below 0.02). It resolves to neutral without NaN reaching the uniforms. Task 3: `muted_accent_is_neutral`.
3. **A fade reversed mid-way** (A, then B, then back to A at 50%). The hue continues from the point it had reached, with no jump. Task 4: `interrupted_fade_continues_from_its_point`.
4. **The glass is edited by a config reload while the bar is mapped.** The face is recomputed on reload, without remapping the layer. Task 5: `config_reload_recomputes_the_face`.
5. **The recorded window moves to another workspace.** The origin goes neutral, and the destination does not inherit the hue until the window settles there. Task 4: `moved_window_clears_its_origin_record`.

---

### Task 1: Oklab helpers and the glass face model (`niri-config`)

**Files:**
- Create: `niri-config/src/oklab.rs`
- Create: `niri-config/src/material/face.rs`
- Modify: `niri-config/src/lib.rs` (add `pub mod oklab;` beside the other `pub mod` lines, around line 40)
- Modify: `niri-config/src/material/mod.rs` (add `pub mod face;` beside `pub mod optics;`, line 15)

**Interfaces:**
- Consumes: `niri_config::appearance::Color` (`to_array_unpremul() -> [f32; 4]`, sRGB-encoded) and `niri_config::material::ResolvedGlass` (`ior`, `thickness`, `attenuation_color`, `attenuation_distance`).
- Produces:
  - In `niri_config::oklab`:
    - `pub fn srgb_to_linear(c: f64) -> f64`
    - `pub fn linear_to_srgb(c: f64) -> f64`
    - `pub fn linear_to_oklab(c: [f64; 3]) -> [f64; 3]`
    - `pub fn oklab_to_linear(lab: [f64; 3]) -> [f64; 3]`
    - `pub fn color_to_oklab(c: Color) -> [f64; 3]`
    - `pub fn chroma(lab: [f64; 3]) -> f64`
  - In `niri_config::material::face`:
    - constants `B_REF`, `SPECULAR_FLOOR`, `COEFF_FLOOR`, `MAX_TRANSMISSION`
    - `pub fn fresnel_f0(ior: f64) -> f64`
    - `pub fn transmittance(glass: &ResolvedGlass) -> [f64; 3]`
    - `pub fn transmission(glass: &ResolvedGlass) -> f64`
    - `pub fn face_linear(glass: &ResolvedGlass, backdrop: [f64; 3]) -> [f64; 3]`
    - `pub fn face_srgb(glass: &ResolvedGlass) -> [f64; 3]`
    - `pub fn check_range(name: &str, glass: &ResolvedGlass) -> Result<(), String>`
    - `pub fn check_effects(name: &str, glass: &ResolvedGlass) -> Result<(), String>`

- [ ] **Step 1: Write the Oklab module with its failing tests**

Create `niri-config/src/oklab.rs`:

```rust
//! sRGB transfer functions and Oklab (Björn Ottosson, public domain) in f64.
//!
//! The same formulas as `src/render_helpers/shaders/material/common.frag`;
//! the shell remap (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md)
//! validates and prepares its uniforms with these.

use crate::appearance::Color;

pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1. / 2.4) - 0.055
    }
}

pub fn linear_to_oklab(c: [f64; 3]) -> [f64; 3] {
    unimplemented!()
}

pub fn oklab_to_linear(lab: [f64; 3]) -> [f64; 3] {
    unimplemented!()
}

/// Oklab of a config color (sRGB-encoded, straight alpha ignored).
pub fn color_to_oklab(c: Color) -> [f64; 3] {
    let [r, g, b, _] = c.to_array_unpremul();
    linear_to_oklab([r, g, b].map(|v| srgb_to_linear(f64::from(v))))
}

pub fn chroma(lab: [f64; 3]) -> f64 {
    lab[1].hypot(lab[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Color {
        s.parse().unwrap()
    }

    #[test]
    fn noctalia_surface_has_the_measured_lightness_and_chroma() {
        // Values from the planning computation on the live wallpaper's palette.
        let lab = color_to_oklab(hex("#151310"));
        assert!((lab[0] - 0.188).abs() < 0.001, "{lab:?}");
        assert!((chroma(lab) - 0.007).abs() < 0.001, "{lab:?}");
    }

    #[test]
    fn black_and_white_are_the_lightness_ends() {
        let black = color_to_oklab(hex("#000000"));
        let white = color_to_oklab(hex("#ffffff"));
        assert!(black.iter().all(|v| v.abs() < 1e-9), "{black:?}");
        assert!((white[0] - 1.).abs() < 1e-4 && chroma(white) < 1e-4, "{white:?}");
    }

    #[test]
    fn oklab_round_trips() {
        for rgb in [[0.2, 0.5, 0.9], [0.01, 0.003, 0.4], [0.9, 0.9, 0.1]] {
            let back = oklab_to_linear(linear_to_oklab(rgb));
            for i in 0..3 {
                assert!((back[i] - rgb[i]).abs() < 1e-9, "{rgb:?} -> {back:?}");
            }
        }
    }
}
```

Add `pub mod oklab;` to `niri-config/src/lib.rs` after `pub mod misc;`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri-config oklab`
Expected: FAIL with a panic `not implemented` in `linear_to_oklab`.

- [ ] **Step 3: Implement the conversions**

Replace the two `unimplemented!()` bodies:

```rust
pub fn linear_to_oklab(c: [f64; 3]) -> [f64; 3] {
    let l = 0.4122214708 * c[0] + 0.5363325363 * c[1] + 0.0514459929 * c[2];
    let m = 0.2119034982 * c[0] + 0.6806995451 * c[1] + 0.1073969566 * c[2];
    let s = 0.0883024619 * c[0] + 0.2817188376 * c[1] + 0.6299787005 * c[2];
    let [l, m, s] = [l, m, s].map(|v| v.max(0.).cbrt());
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

pub fn oklab_to_linear(lab: [f64; 3]) -> [f64; 3] {
    let l = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
    let m = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
    let s = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];
    let [l, m, s] = [l, m, s].map(|v| v * v * v);
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ]
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `just test-one -p niri-config oklab`
Expected: PASS (3 tests).

- [ ] **Step 5: Write the face model with its failing tests**

Create `niri-config/src/material/face.rs`:

```rust
//! The glass's flat-face color over a reference backdrop, and the range in
//! which the opaque shell can match it
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §2).
//!
//! The terms mirror `main.frag`'s flat face: reflection and the facing term
//! vanish there, so the face is the transmitted backdrop plus the specular
//! floor. A test in `src/render_helpers/surface_remap.rs` checks the shader
//! still carries these constants.

use super::ResolvedGlass;
use crate::oklab::linear_to_srgb;

/// Neutral reference backdrop, linear.
pub const B_REF: f64 = 0.2;
/// `main.frag`: `vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));`.
pub const SPECULAR_FLOOR: f64 = 0.15;
/// The shader's floor on the attenuation coefficient.
pub const COEFF_FLOOR: f64 = 0.001;
/// Largest share of any backdrop channel the face may pass.
pub const MAX_TRANSMISSION: f64 = 0.01;

pub fn fresnel_f0(ior: f64) -> f64 {
    let r = (ior - 1.) / (ior + 1.);
    r * r
}

/// Per-channel transmittance `T` along the face path.
pub fn transmittance(glass: &ResolvedGlass) -> [f64; 3] {
    let [r, g, b, _] = glass.attenuation_color.to_array_unpremul();
    let p = glass.thickness / glass.attenuation_distance;
    [r, g, b].map(|c| f64::from(c).clamp(COEFF_FLOOR, 1.).powf(p))
}

/// `max(T) · (1 − f0)`: the share of the most open channel that reaches the eye.
pub fn transmission(glass: &ResolvedGlass) -> f64 {
    unimplemented!()
}

/// The face over `backdrop` (linear), linear.
pub fn face_linear(glass: &ResolvedGlass, backdrop: [f64; 3]) -> [f64; 3] {
    unimplemented!()
}

/// The reference swatch: the face over `B_REF`, sRGB-encoded in 0..1.
pub fn face_srgb(glass: &ResolvedGlass) -> [f64; 3] {
    face_linear(glass, [B_REF; 3]).map(linear_to_srgb)
}

pub fn check_range(name: &str, glass: &ResolvedGlass) -> Result<(), String> {
    unimplemented!()
}

/// The model covers transmitted light and the specular floor only. Aurora
/// lights the whole face and iridescence recolors the floor, so a material
/// with either cannot be matched (spec §2, supported effects).
pub fn check_effects(name: &str, glass: &ResolvedGlass) -> Result<(), String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appearance::Color;
    use crate::material::optics::aurora::ResolvedAurora;
    use crate::material::optics::iridescence::ResolvedIridescence;
    use crate::oklab::linear_to_oklab;

    fn glass(color: &str, thickness: f64, distance: f64, ior: f64) -> ResolvedGlass {
        ResolvedGlass {
            attenuation_color: color.parse().unwrap(),
            thickness,
            attenuation_distance: distance,
            ior,
            ..ResolvedGlass::default()
        }
    }

    fn live() -> ResolvedGlass {
        glass("#0D1D1E", 31.2, 12., 1.28)
    }

    fn levels(linear: [f64; 3]) -> [i32; 3] {
        linear.map(|v| (linear_to_srgb(v) * 255.).round() as i32)
    }

    #[track_caller]
    fn assert_levels(got: [i32; 3], want: [i32; 3], tol: i32) {
        for i in 0..3 {
            assert!((got[i] - want[i]).abs() <= tol, "{got:?} vs {want:?}");
        }
    }

    fn lightness(glass: &ResolvedGlass, backdrop: f64) -> f64 {
        linear_to_oklab(face_linear(glass, [backdrop; 3]))[0]
    }

    #[test]
    fn live_glass_matches_the_b156e7_measurement() {
        let face = face_srgb(&live()).map(|v| (v * 255.).round() as i32);
        assert_levels(face, [8, 11, 10], 2);
    }

    #[test]
    fn section_2_table() {
        let live = live();
        assert_levels(levels(face_linear(&live, [0.; 3])), [7, 7, 7], 1);
        assert_levels(levels(face_linear(&live, [B_REF; 3])), [8, 10, 10], 1);
        assert_levels(levels(face_linear(&live, [1.; 3])), [9, 17, 18], 1);

        let d20 = glass("#0D1D1E", 31.2, 20., 1.28);
        assert_levels(levels(face_linear(&d20, [0.; 3])), [7, 7, 7], 1);
        assert_levels(levels(face_linear(&d20, [B_REF; 3])), [13, 24, 24], 1);
        assert_levels(levels(face_linear(&d20, [1.; 3])), [28, 53, 54], 1);
    }

    #[test]
    fn range_accepts_the_live_glass() {
        assert!(check_range("terminal-glass", &live()).is_ok());
        assert!((transmission(&live()) - 0.004).abs() < 0.0005);
    }

    #[test]
    fn range_rejects_lighter_glass_with_the_section_2_message() {
        for (glass, name) in [
            (glass("#0D1D1E", 31.2, 20., 1.28), "d20"),
            (glass("#dfe8ff", 31.2, 30., 1.28), "light"),
        ] {
            let err = check_range(name, &glass).unwrap_err();
            assert!(
                err.starts_with(&format!("surface-remap: material \"{name}\" transmits ")),
                "{err}"
            );
            assert!(
                err.ends_with(
                    "in its most open channel; the shell match supports dark glass up to 0.01"
                ),
                "{err}"
            );
        }
    }

    #[test]
    fn range_bounds_every_channel_not_neutral_luminance() {
        // Review round 2: Y(T) passes this glass, a blue backdrop gets 12.5%.
        let blue = glass("#000080", 30., 10., 1.28);
        assert!(check_range("blue", &blue).is_err());
        let floor = SPECULAR_FLOOR * fresnel_f0(1.28);
        let through = face_linear(&blue, [0., 0., 1.])[2] - floor;
        assert!((through - 0.1246).abs() < 0.001, "{through}");
    }

    #[test]
    fn worst_case_counterexample_is_accepted_and_inside_the_bound() {
        // Review round 2: neutral T at about 1%, IOR 1.
        let g = glass("#191919", 20., 10., 1.0);
        assert!(check_range("gray", &g).is_ok());
        let reference = lightness(&g, B_REF);
        let lo = lightness(&g, 0.05) - reference;
        let hi = lightness(&g, 0.5) - reference;
        assert!((lo + 0.046).abs() < 0.001 && (hi - 0.044).abs() < 0.001, "{lo} {hi}");
        assert!(lo >= -0.047 && hi <= 0.04501);
    }

    #[test]
    fn ior_table() {
        // Neutral T exactly at the bound: color^1 · (1 − f0) = 0.01.
        for (ior, want_lo, want_hi) in [
            (1.0, -0.0466, 0.04500549),
            (1.1, -0.0384, 0.0420),
            (1.28, -0.0218, 0.0315),
            (1.5, -0.0134, 0.0224),
            (3.0, -0.0044, 0.0084),
        ] {
            let v = (MAX_TRANSMISSION / (1. - fresnel_f0(ior))) as f32;
            let g = ResolvedGlass {
                attenuation_color: Color::new_unpremul(v, v, v, 1.),
                thickness: 10.,
                attenuation_distance: 10.,
                ior,
                ..ResolvedGlass::default()
            };
            let reference = lightness(&g, B_REF);
            let lo = lightness(&g, 0.05) - reference;
            let hi = lightness(&g, 0.5) - reference;
            assert!((lo - want_lo).abs() < 5e-4, "ior {ior}: {lo}");
            assert!((hi - want_hi).abs() < 5e-4, "ior {ior}: {hi}");
        }
    }

    #[test]
    fn effects_accept_the_live_glass() {
        assert!(check_effects("terminal-glass", &live()).is_ok());
    }

    #[test]
    fn aurora_passes_the_range_but_is_rejected() {
        // Round-4 review: white aurora at strength 1 on the live glass.
        let white = Color::new_unpremul(1., 1., 1., 1.);
        let glass = ResolvedGlass {
            aurora: ResolvedAurora {
                amount: 1.,
                color_a: white,
                color_b: white,
                ..ResolvedAurora::default()
            },
            ..live()
        };
        assert!(check_range("aurora", &glass).is_ok());
        let err = check_effects("aurora", &glass).unwrap_err();
        assert_eq!(
            err,
            "surface-remap: material \"aurora\" sets aurora, which lights the flat face; \
             the shell match supports glass without them"
        );
    }

    #[test]
    fn iridescence_is_rejected() {
        let glass = ResolvedGlass {
            iridescence: ResolvedIridescence { amount: 1. },
            ..live()
        };
        let err = check_effects("iri", &glass).unwrap_err();
        assert!(err.contains("sets iridescence, which lights the flat face"), "{err}");
    }

    #[test]
    fn sweep_of_accepted_configurations_stays_within_the_guarantee() {
        let steps = [0.0, 0.02, 0.05, 0.1, 0.2, 0.4, 0.8];
        let mut accepted = 0;
        for r in steps {
            for g in steps {
                for b in steps {
                    for thickness in [5., 20., 31.2, 100.] {
                        for distance in [1., 5., 12., 30., 100.] {
                            for ior in [1.0, 1.1, 1.28, 1.5, 2., 3.] {
                                let glass = ResolvedGlass {
                                    attenuation_color: Color::new_unpremul(r, g, b, 1.),
                                    thickness,
                                    attenuation_distance: distance,
                                    ior,
                                    ..ResolvedGlass::default()
                                };
                                if check_range("s", &glass).is_err() {
                                    continue;
                                }
                                accepted += 1;
                                let reference = lightness(&glass, B_REF);
                                let lo = lightness(&glass, 0.05) - reference;
                                let hi = lightness(&glass, 0.5) - reference;
                                assert!(lo >= -0.047 && hi <= 0.04501, "{glass:?}: {lo} {hi}");
                                let floor = SPECULAR_FLOOR * fresnel_f0(ior);
                                for channel in 0..3 {
                                    let mut backdrop = [0.; 3];
                                    backdrop[channel] = 1.;
                                    let through = face_linear(&glass, backdrop)[channel] - floor;
                                    assert!(through <= MAX_TRANSMISSION + 1e-12, "{glass:?}");
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(accepted > 1000, "the sweep must reach many accepted configs: {accepted}");
    }
}
```

Add `pub mod face;` to `niri-config/src/material/mod.rs` after `pub mod optics;`.

- [ ] **Step 6: Run the tests to see them fail**

Run: `just test-one -p niri-config material::face`
Expected: FAIL with `not implemented` panics. If `ResolvedAurora` has no `Default` with those field names, read `niri-config/src/material/optics/aurora.rs` (`amount`, `drift_hz`, `color_a`, `color_b` at planning time) and build it field by field.

- [ ] **Step 7: Implement the face model**

Replace the three `unimplemented!()` bodies:

```rust
pub fn transmission(glass: &ResolvedGlass) -> f64 {
    let t = transmittance(glass);
    t[0].max(t[1]).max(t[2]) * (1. - fresnel_f0(glass.ior))
}

pub fn face_linear(glass: &ResolvedGlass, backdrop: [f64; 3]) -> [f64; 3] {
    let t = transmittance(glass);
    let f0 = fresnel_f0(glass.ior);
    [0, 1, 2].map(|i| backdrop[i] * t[i] * (1. - f0) + SPECULAR_FLOOR * f0)
}

pub fn check_range(name: &str, glass: &ResolvedGlass) -> Result<(), String> {
    let x = transmission(glass);
    if x <= MAX_TRANSMISSION {
        Ok(())
    } else {
        Err(format!(
            "surface-remap: material \"{name}\" transmits {x:.3} in its most open channel; \
             the shell match supports dark glass up to 0.01"
        ))
    }
}
```

```rust
pub fn check_effects(name: &str, glass: &ResolvedGlass) -> Result<(), String> {
    let optic = if glass.aurora.amount > 0. {
        "aurora"
    } else if glass.iridescence.amount > 0. {
        "iridescence"
    } else {
        return Ok(());
    };
    Err(format!(
        "surface-remap: material \"{name}\" sets {optic}, which lights the flat face; \
         the shell match supports glass without them"
    ))
}
```

- [ ] **Step 8: Run the tests to see them pass**

Run: `just test-one -p niri-config material::face`
Expected: PASS (11 tests). If `section_2_table` misses by more than one level, recompute the table entry with the same formula and record the difference in the task note before changing either the test or the spec.

- [ ] **Step 9: Format, run the fast suite, commit**

```bash
rustup run nightly cargo fmt --all
just test-fast
tasks check
git add niri-config/src/oklab.rs niri-config/src/material/face.rs niri-config/src/lib.rs niri-config/src/material/mod.rs
git commit -m "feat(config): add Oklab helpers and the glass face model"
```

---

### Task 2: The `surface-remap` layer-rule node and its load-time validation

**Files:**
- Modify: `niri-config/src/layer_rule.rs` (the whole file: new struct, field, validator)
- Modify: `niri-config/src/lib.rs` (post-merge validation call at about line 540; tests; the config debug snapshot at about line 3723)
- Modify: `src/layer/mod.rs` (`ResolvedLayerRules`: field and whole-node replacement)

**Interfaces:**
- Consumes:
  - `niri_config::material::{Material, MaterialRef, Milli}`;
  - `Material::validate(&self) -> Result<(), String>` (`pub(crate)`);
  - `Material::resolve()`;
  - `niri_config::material::face::{check_range, check_effects}`;
  - `niri_config::oklab::color_to_oklab`.
- Produces:
  - `niri_config::layer_rule::SurfaceRemap { surface: Color, container: Color, outline: Color, material: MaterialRef, hue_chroma: Option<Milli<0, 100>> }`;
  - `LayerRule::surface_remap: Option<SurfaceRemap>`;
  - `pub const MAX_SOURCE_SURFACE_L: f64 = 0.4`;
  - `pub fn validate_surface_remaps(rules: &[LayerRule], materials: &[Material]) -> Vec<String>`;
  - `ResolvedLayerRules::surface_remap: Option<SurfaceRemap>`.

- [ ] **Step 1: Write the failing config tests**

Append to the `tests` module in `niri-config/src/lib.rs` (beside `window_rule_rejects_unknown_material`):

```rust
    const REMAP_GLASS: &str = r##"
        material "tg" {
            glass {
                ior 1.28
                thickness 31.2
                attenuation-color "#0D1D1E"
                attenuation-distance 12
            }
        }
    "##;

    fn remap_config(body: &str) -> String {
        format!(
            "{REMAP_GLASS}\nlayer-rule {{\n    match namespace=\"^bar$\"\n    surface-remap {{\n{body}\n    }}\n}}\n"
        )
    }

    const REMAP_BODY: &str = r##"
        surface "#151310"
        container "#221f1c"
        outline "#4d453c"
        material "tg"
        hue-chroma 0.03
    "##;

    #[test]
    fn parses_surface_remap() {
        let parsed = do_parse(&remap_config(REMAP_BODY));
        let remap = parsed.layer_rules[0].surface_remap.as_ref().unwrap();
        assert_eq!(remap.surface, Color::from_rgba8_unpremul(0x15, 0x13, 0x10, 0xff));
        assert_eq!(remap.container, Color::from_rgba8_unpremul(0x22, 0x1f, 0x1c, 0xff));
        assert_eq!(remap.outline, Color::from_rgba8_unpremul(0x4d, 0x45, 0x3c, 0xff));
        assert_eq!(remap.material.name, "tg");
        assert_eq!(remap.hue_chroma.map(|x| x.0), Some(0.03));
    }

    #[test]
    fn surface_remap_hue_chroma_defaults_to_none() {
        let body = REMAP_BODY.replace("hue-chroma 0.03", "");
        let parsed = do_parse(&remap_config(&body));
        assert_eq!(parsed.layer_rules[0].surface_remap.as_ref().unwrap().hue_chroma, None);
    }

    #[test]
    fn surface_remap_requires_every_color_and_the_material() {
        for (line, field) in [
            ("surface \"#151310\"", "surface"),
            ("container \"#221f1c\"", "container"),
            ("outline \"#4d453c\"", "outline"),
            ("material \"tg\"", "material"),
        ] {
            let err = do_parse_err(&remap_config(&REMAP_BODY.replace(line, "")));
            assert!(err.contains(field), "{field}: {err}");
        }
    }

    #[test]
    fn surface_remap_rejects_unknown_material() {
        let err = do_parse_err(&remap_config(&REMAP_BODY.replace("\"tg\"", "\"nope\"")));
        assert!(err.contains("unknown material: nope"), "{err}");
    }

    #[test]
    fn surface_remap_rejects_hue_chroma_out_of_range() {
        let err = do_parse_err(&remap_config(&REMAP_BODY.replace("0.03", "0.2")));
        assert!(err.contains("value must be between 0 and 0.1"), "{err}");
    }

    #[test]
    fn surface_remap_rejects_glass_outside_the_range() {
        let config = remap_config(REMAP_BODY).replace("attenuation-distance 12", "attenuation-distance 20");
        let err = do_parse_err(&config);
        assert!(err.contains("surface-remap: material \"tg\" transmits"), "{err}");
        assert!(err.contains("the shell match supports dark glass up to 0.01"), "{err}");
    }

    #[test]
    fn surface_remap_rejects_a_light_source_surface() {
        let err = do_parse_err(&remap_config(&REMAP_BODY.replace("#151310", "#e8e1dc")));
        assert!(err.contains("the remap supports a dark Noctalia scheme"), "{err}");
    }

    #[test]
    fn surface_remap_rejects_glass_that_lights_its_face() {
        let config = remap_config(REMAP_BODY).replace(
            "attenuation-distance 12",
            "attenuation-distance 12\n                iridescence 1",
        );
        let err = do_parse_err(&config);
        assert!(err.contains("sets iridescence, which lights the flat face"), "{err}");
    }

    #[test]
    fn surface_remap_rejects_a_response() {
        let err = do_parse_err(&remap_config(
            &REMAP_BODY.replace("material \"tg\"", "material \"tg\" response=\"default\""),
        ));
        assert!(err.contains("surface-remap: material \"tg\" takes no response"), "{err}");
    }

    #[test]
    fn two_rules_keep_their_own_complete_nodes() {
        let second = REMAP_BODY.replace("#151310", "#111318").replace("hue-chroma 0.03", "");
        let text = format!(
            "{}\nlayer-rule {{\n    match namespace=\"^bar$\"\n    surface-remap {{\n{second}\n    }}\n}}\n",
            remap_config(REMAP_BODY)
        );
        let parsed = do_parse(&text);
        let first = parsed.layer_rules[0].surface_remap.as_ref().unwrap();
        let later = parsed.layer_rules[1].surface_remap.as_ref().unwrap();
        assert_eq!(first.hue_chroma.map(|x| x.0), Some(0.03));
        assert_eq!(later.hue_chroma, None);
        assert_eq!(later.surface, Color::from_rgba8_unpremul(0x11, 0x13, 0x18, 0xff));
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri-config surface_remap`
Expected: FAIL to compile with `no field surface_remap on type LayerRule`.

- [ ] **Step 3: Add the node, the field and the validator**

In `niri-config/src/layer_rule.rs`, replace the imports and add the struct, field and validator:

```rust
use crate::appearance::{BackgroundEffectRule, BlockOutFrom, Color, CornerRadius, ShadowRule};
use crate::material::{Material, MaterialRef, Milli};
use crate::oklab::color_to_oklab;
use crate::utils::RegexEq;
use crate::window_rule::PopupsRule;
```

Append to `LayerRule`, after `popups`:

```rust
    #[knuffel(child)]
    pub surface_remap: Option<SurfaceRemap>,
```

Append below `Match`:

```rust
/// `surface-remap { … }`: recolor a layer surface's surface family to a
/// glass face and the output's held hue
/// (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §3, §5).
///
/// Every field but `hue-chroma` is required, and a later matching rule
/// replaces the whole node, so the node a surface gets is always one that was
/// validated at load.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct SurfaceRemap {
    #[knuffel(child)]
    pub surface: Color,
    #[knuffel(child)]
    pub container: Color,
    #[knuffel(child)]
    pub outline: Color,
    #[knuffel(child)]
    pub material: MaterialRef,
    #[knuffel(child, unwrap(argument))]
    pub hue_chroma: Option<Milli<0, 100>>,
}

/// The lightest source surface the remap accepts: a dark Noctalia scheme.
pub const MAX_SOURCE_SURFACE_L: f64 = 0.4;

/// Checks every `surface-remap` against the merged material table.
///
/// Unknown material names are reported by `validate_material_refs`, which
/// sees the same reference; a material whose own definition failed
/// validation is skipped here, since its error is already emitted.
pub fn validate_surface_remaps(rules: &[LayerRule], materials: &[Material]) -> Vec<String> {
    let mut errors = Vec::new();
    for remap in rules.iter().filter_map(|rule| rule.surface_remap.as_ref()) {
        let name = &remap.material.name;
        if remap.material.response.is_some() {
            errors.push(format!("surface-remap: material \"{name}\" takes no response"));
        }
        let l = color_to_oklab(remap.surface)[0];
        if l > MAX_SOURCE_SURFACE_L {
            errors.push(format!(
                "surface-remap: the source surface has Oklab L {l:.2}; \
                 the remap supports a dark Noctalia scheme (L up to 0.4)"
            ));
        }
        let Some(material) = materials.iter().find(|m| &m.name == name) else {
            continue;
        };
        if material.validate().is_err() {
            continue;
        }
        let glass = material.resolve().glass;
        for check in [crate::material::face::check_range, crate::material::face::check_effects] {
            if let Err(message) = check(name, &glass) {
                errors.push(message);
            }
        }
    }
    errors
}
```

In `niri-config/src/lib.rs`, inside the `if recursion == 0 { … }` block after the `backdrop_grain` check (about line 548):

```rust
            for message in
                crate::layer_rule::validate_surface_remaps(&config.layer_rules, &config.materials)
            {
                ctx.emit_error(DecodeError::Custom(message.into()));
            }
```

`MaterialRef`'s decoder records the reference into the parse context, so `validate_material_refs` reports an unknown name with the existing message. No second check is needed.

- [ ] **Step 4: Run the config tests**

Run: `just test-one -p niri-config surface_remap`
Expected: PASS (10 tests).

If `surface_remap_requires_every_color_and_the_material` fails because knuffel's missing-child message lacks the field name, assert on the message knuffel prints, which names the node (`child node \`surface\` is required` in knuffel 3). Do not add a custom check.

- [ ] **Step 5: Update the config debug snapshot**

Run: `just test-one -p niri-config parse`
Expected: FAIL in the full-config debug snapshot: the `LayerRule` now prints `surface_remap: None,` after its `popups: PopupsRule { … },` block (about line 3730). Insert that line, at the same indentation as `popups:`, in the inline snapshot. Rerun: PASS.

- [ ] **Step 6: Resolve the node whole in `ResolvedLayerRules`**

In `src/layer/mod.rs`, add to the struct after `popups`:

```rust
    /// Surface-family remap, replaced whole by later rules.
    pub surface_remap: Option<niri_config::layer_rule::SurfaceRemap>,
```

In `compute`, after `resolved.popups.merge_with(&rule.popups);`:

```rust
            if let Some(x) = &rule.surface_remap {
                resolved.surface_remap = Some(x.clone());
            }
```

The integration test `two_rules_replace_the_node_whole` in Task 5 checks this replacement on a mapped layer.

- [ ] **Step 7: Format, run the fast suite, commit**

```bash
rustup run nightly cargo fmt --all
just test-fast
tasks check
git add niri-config/src/layer_rule.rs niri-config/src/lib.rs src/layer/mod.rs
git commit -m "feat(config): add the surface-remap layer rule with load-time validation"
```

---

### Task 3: Remap math: `HueBlend`, `RemapSetup`, `RemapUniforms`

**Files:**
- Create: `src/render_helpers/surface_remap.rs`
- Modify: `src/render_helpers/mod.rs` (add `pub mod surface_remap;` in the module list, alphabetically after `pub mod solid_color;`)

**Interfaces:**
- Consumes:
  - `niri_config::layer_rule::SurfaceRemap` (Task 2);
  - from `niri_config::material::face` (Task 1): `face_srgb`, `SPECULAR_FLOOR`;
  - from `niri_config::oklab` (Task 1): `color_to_oklab`, `linear_to_oklab`, `oklab_to_linear`, `srgb_to_linear`, `linear_to_srgb`, `chroma`;
  - `niri_config::material::ResolvedGlass`.
- Produces (in `crate::render_helpers::surface_remap`):
  - Constants `R0`, `R1`, `MIN_ACCENT_CHROMA`.
  - `HueBlend { pub neutral: f32, pub hue: [f32; 2] }`, with `HueBlend::NEUTRAL`, `HueBlend::from_accent(Color) -> HueBlend` and `HueBlend::lerp(a, b, t: f32) -> HueBlend`. It implements `Default`, `Copy` and `PartialEq`.
  - `OutputHue { pub blend: HueBlend, pub animating: bool }`, implementing `Default` (neutral, not animating).
  - `RemapSetup`, with:
    - `RemapSetup::new(&SurfaceRemap, &ResolvedGlass) -> RemapSetup`
    - `RemapSetup::with_face(&SurfaceRemap, face_srgb: [f64; 3]) -> RemapSetup`
    - `RemapSetup::family(&self, HueBlend) -> [[f64; 3]; 3]`
    - `RemapSetup::uniforms(&self, HueBlend) -> RemapUniforms`
  - `RemapUniforms { pub anchors: [[f32; 3]; 3], pub offsets: [[f32; 3]; 3], pub radii: [f32; 2] }`, with:
    - `RemapUniforms::weight(&self, lab: [f32; 3]) -> f32`
    - `RemapUniforms::apply(&self, rgba: [f32; 4]) -> [f32; 4]` (premultiplied sRGB, the shader's CPU mirror)

**Why `HueBlend`:** the target family is affine in two quantities:
- the surface target's ab, `neutral · face_ab + hue_chroma · hue`;
- the hue direction, `neutral · face_dir + hue`.

The container and outline add `ΔC_role · dir` to the surface ab, and every role's L is `face_L + ΔL_role`. So lerping `HueBlend` componentwise lerps the target families exactly, which is what the spec requires (§4, "Targets come first"). An accent maps to `neutral = 0` and `hue` = its unit hue vector, and neutral maps to `neutral = 1, hue = 0`. Red to cyan therefore passes `hue ≈ 0`, a near-neutral family, never an undefined hue.

- [ ] **Step 1: Write the module skeleton and its failing tests**

Create `src/render_helpers/surface_remap.rs`:

```rust
//! Color math for the shell's surface remap
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §3, §4).
//!
//! `RemapSetup` turns a validated `surface-remap` node and its glass into
//! source anchors and target families; `RemapUniforms` is what
//! `surface_remap.frag` receives, and `apply` is that shader's CPU mirror.

use niri_config::layer_rule::SurfaceRemap;
use niri_config::material::face::face_srgb;
use niri_config::material::ResolvedGlass;
use niri_config::oklab::{
    chroma, color_to_oklab, linear_to_oklab, linear_to_srgb, oklab_to_linear, srgb_to_linear,
};
use niri_config::Color;

/// Distance to the family polyline (Oklab) where the remap starts to fade.
pub const R0: f64 = 0.008;
/// Distance at which the remap is gone.
pub const R1: f64 = 0.02;
/// Below this Oklab chroma an accent has no hue to show.
pub const MIN_ACCENT_CHROMA: f64 = 0.02;

/// A hue state in the coordinates where the target family is affine:
/// `neutral` is the weight of the unhued face, `hue` the weighted sum of
/// accent unit hue vectors (Oklab a, b). Lerping these lerps families.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HueBlend {
    pub neutral: f32,
    pub hue: [f32; 2],
}

impl HueBlend {
    pub const NEUTRAL: Self = Self {
        neutral: 1.,
        hue: [0., 0.],
    };

    pub fn from_accent(accent: Color) -> Self {
        unimplemented!()
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        unimplemented!()
    }
}

impl Default for HueBlend {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

/// What every remapped layer on an output shows this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OutputHue {
    pub blend: HueBlend,
    /// A fade or switch blend is running, so frames must keep coming.
    pub animating: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemapSetup {
    /// Oklab of surface, container, outline as Noctalia draws them.
    sources: [[f64; 3]; 3],
    /// Oklab of the glass face over the reference backdrop.
    face: [f64; 3],
    /// Unit hue direction of the face, zero when it has no chroma.
    face_dir: [f64; 2],
    /// Per role: (ΔL, ΔC ≥ 0) from the source surface.
    role_offsets: [(f64, f64); 3],
    hue_chroma: f64,
}

impl RemapSetup {
    pub fn new(remap: &SurfaceRemap, glass: &ResolvedGlass) -> Self {
        Self::with_face(remap, face_srgb(glass))
    }

    /// Builds the setup against an explicit face swatch (sRGB in 0..1).
    pub fn with_face(remap: &SurfaceRemap, face_srgb: [f64; 3]) -> Self {
        unimplemented!()
    }

    /// Target Oklab for surface, container and outline at `blend`.
    pub fn family(&self, blend: HueBlend) -> [[f64; 3]; 3] {
        unimplemented!()
    }

    pub fn uniforms(&self, blend: HueBlend) -> RemapUniforms {
        let family = self.family(blend);
        RemapUniforms {
            anchors: self.sources.map(|s| s.map(|v| v as f32)),
            offsets: [0, 1, 2].map(|i| [0, 1, 2].map(|k| (family[i][k] - self.sources[i][k]) as f32)),
            radii: [R0 as f32, R1 as f32],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemapUniforms {
    pub anchors: [[f32; 3]; 3],
    pub offsets: [[f32; 3]; 3],
    pub radii: [f32; 2],
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// Distance from `p` to the segment `a → b`, and the fraction along it.
fn segment(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> (f32, f32) {
    let ab = [0, 1, 2].map(|i| b[i] - a[i]);
    let len2: f32 = ab.iter().map(|v| v * v).sum();
    let t = if len2 > 0. {
        ([0, 1, 2].map(|i| (p[i] - a[i]) * ab[i]).iter().sum::<f32>() / len2).clamp(0., 1.)
    } else {
        0.
    };
    let d: f32 = [0, 1, 2]
        .map(|i| p[i] - (a[i] + t * ab[i]))
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt();
    (d, t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

impl RemapUniforms {
    /// Distance to the family polyline and the interpolated offset there.
    /// Mirrors `surface_remap.frag`'s `nearest`.
    fn nearest(&self, lab: [f32; 3]) -> (f32, [f32; 3]) {
        let [a0, a1, a2] = self.anchors;
        let [o0, o1, o2] = self.offsets;
        let (d0, t0) = segment(lab, a0, a1);
        let (d1, t1) = segment(lab, a1, a2);
        if d0 <= d1 {
            (d0, mix3(o0, o1, t0))
        } else {
            (d1, mix3(o1, o2, t1))
        }
    }

    pub fn weight(&self, lab: [f32; 3]) -> f32 {
        1. - smoothstep(self.radii[0], self.radii[1], self.nearest(lab).0)
    }

    /// The shader on one premultiplied sRGB pixel.
    pub fn apply(&self, rgba: [f32; 4]) -> [f32; 4] {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use niri_config::material::face::SPECULAR_FLOOR;

    use super::*;

    fn hex(s: &str) -> Color {
        s.parse().unwrap()
    }

    /// Builds the node directly: the default `glass {}` is light glass and
    /// would fail the range check in a full config parse.
    fn remap(surface: &str, container: &str, outline: &str, hue_chroma: f64) -> SurfaceRemap {
        SurfaceRemap {
            surface: hex(surface),
            container: hex(container),
            outline: hex(outline),
            material: niri_config::material::MaterialRef {
                name: String::from("tg"),
                response: None,
            },
            hue_chroma: (hue_chroma > 0.).then_some(niri_config::material::Milli(hue_chroma)),
        }
    }

    fn live() -> SurfaceRemap {
        remap("#151310", "#221f1c", "#4d453c", 0.03)
    }

    fn srgb_levels(lab: [f64; 3]) -> [i32; 3] {
        oklab_to_linear(lab).map(|v| (linear_to_srgb(v.clamp(0., 1.)) * 255.).round() as i32)
    }

    fn lab32(s: &str) -> [f32; 3] {
        color_to_oklab(hex(s)).map(|v| v as f32)
    }

    fn lab32_mix(base: &str, over: &str, alpha: f64) -> [f32; 3] {
        let [br, bg, bb, _] = hex(base).to_array_unpremul();
        let [or, og, ob, _] = hex(over).to_array_unpremul();
        let mixed = [(br, or), (bg, og), (bb, ob)]
            .map(|(b, o)| srgb_to_linear(f64::from(b) * (1. - alpha) + f64::from(o) * alpha));
        linear_to_oklab(mixed).map(|v| v as f32)
    }

    #[test]
    fn neutral_family_reproduces_the_palette_helper() {
        // noctalia-glass-palette.py with target #090a0a on the live palette.
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let family = setup.family(HueBlend::NEUTRAL);
        for (got, want) in family.iter().zip([[9, 10, 10], [20, 22, 22], [51, 61, 61]]) {
            let got = srgb_levels(*got);
            for i in 0..3 {
                assert!((got[i] - want[i]).abs() <= 1, "{got:?} vs {want:?}");
            }
        }
    }

    #[test]
    fn hued_surface_keeps_the_face_lightness_and_takes_the_accent_hue() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let accent = hex("#e5533c");
        let family = setup.family(HueBlend::from_accent(accent));
        let face = linear_to_oklab([9., 10., 10.].map(|v: f64| srgb_to_linear(v / 255.)));
        assert!((family[0][0] - face[0]).abs() < 1e-9);
        assert!((chroma(family[0]) - 0.03).abs() < 1e-6);
        let want = color_to_oklab(accent);
        let hue = |lab: [f64; 3]| lab[2].atan2(lab[1]);
        assert!((hue(family[0]) - hue(want)).abs() < 1e-6);
        // Container and outline share the hue at their chroma steps.
        assert!((hue(family[2]) - hue(want)).abs() < 1e-6);
        assert!(chroma(family[2]) > chroma(family[0]));
    }

    #[test]
    fn muted_accent_is_neutral() {
        for gray in ["#ffffff", "#808080", "#000000", "#7f8081"] {
            let blend = HueBlend::from_accent(hex(gray));
            assert_eq!(blend, HueBlend::NEUTRAL, "{gray}");
            let u = RemapSetup::with_face(&live(), [0.03; 3]).uniforms(blend);
            assert!(u.offsets.iter().flatten().all(|v| v.is_finite()), "{gray}: {u:?}");
        }
    }

    #[test]
    fn red_to_cyan_never_snaps() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let red = HueBlend::from_accent(hex("#ff0000"));
        let cyan = HueBlend::from_accent(hex("#00ffff"));
        let mut previous = setup.family(red);
        for step in 1..=2000 {
            let t = step as f32 / 2000.;
            let family = setup.family(HueBlend::lerp(red, cyan, t));
            for role in 0..3 {
                for k in 0..3 {
                    let jump = (family[role][k] - previous[role][k]).abs();
                    assert!(jump < 0.01, "t {t}: role {role} component {k} jumps {jump}");
                }
            }
            previous = family;
        }
        // The midpoint passes near neutral: hue directions cancel.
        let mid = HueBlend::lerp(red, cyan, 0.5);
        assert!(mid.hue[0].hypot(mid.hue[1]) < 0.2, "{mid:?}");
    }

    #[test]
    fn lerping_blends_lerps_families() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let a = HueBlend::NEUTRAL;
        let b = HueBlend::from_accent(hex("#3cc6e5"));
        for t in [0.1_f32, 0.37, 0.5, 0.9] {
            let direct = setup.family(HueBlend::lerp(a, b, t));
            let (fa, fb) = (setup.family(a), setup.family(b));
            for role in 0..3 {
                for k in 0..3 {
                    let lerped = fa[role][k] + (fb[role][k] - fa[role][k]) * f64::from(t);
                    assert!((direct[role][k] - lerped).abs() < 1e-6);
                }
            }
        }
    }

    // Role colors of three m3-content palettes (planning, 2026-10-17): the live
    // wallpaper, a blue one and a vivid green one.
    const PALETTES: [[&str; 14]; 3] = [
        [
            "#151310", "#221f1c", "#4d453c", "#ffe7cb", "#402d10", "#d5c4b0", "#392f21",
            "#e9eec4", "#2e3316", "#ffb4ab", "#690005", "#e8e1dc", "#d1c5b8", "#e9eec4",
        ],
        [
            "#111318", "#1d2025", "#414751", "#a4c9ff", "#00315d", "#b1c8ec", "#1a314e",
            "#f4aeff", "#510f62", "#ffb4ab", "#690005", "#e1e2e9", "#c1c7d3", "#f4aeff",
        ],
        [
            "#12150a", "#1e2115", "#444935", "#b5e435", "#273500", "#b8d07b", "#273500",
            "#5cef7e", "#003913", "#ffb4ab", "#690005", "#e2e4d2", "#c4c9af", "#5cef7e",
        ],
    ];
    // Indices into a PALETTES row.
    const SURFACE: usize = 0;
    const CONTAINER: usize = 1;
    const OUTLINE: usize = 2;
    const PRIMARY: usize = 3;
    const SECONDARY: usize = 5;
    const TERTIARY: usize = 7;
    const ERROR: usize = 9;
    const ON_ERROR: usize = 10;
    const ON_SURFACE: usize = 11;
    const ON_SURFACE_VARIANT: usize = 12;

    fn uniforms_for(p: &[&str; 14]) -> RemapUniforms {
        RemapSetup::with_face(&remap(p[SURFACE], p[CONTAINER], p[OUTLINE], 0.03), [0.035; 3])
            .uniforms(HueBlend::NEUTRAL)
    }

    #[test]
    fn family_anchors_take_full_weight() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for role in [SURFACE, CONTAINER, OUTLINE] {
                assert!((u.weight(lab32(p[role])) - 1.).abs() < 1e-6, "{}", p[role]);
            }
        }
    }

    #[test]
    fn text_accents_errors_and_black_are_untouched() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for role in [ON_SURFACE, ON_SURFACE_VARIANT, PRIMARY, SECONDARY, TERTIARY, ERROR, ON_ERROR] {
                assert_eq!(u.weight(lab32(p[role])), 0., "{}", p[role]);
            }
            assert_eq!(u.weight(lab32("#000000")), 0.);
        }
    }

    #[test]
    fn neutral_overlays_move_with_the_family() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for alpha in [0.04, 0.08, 0.12, 0.16] {
                let w = u.weight(lab32_mix(p[SURFACE], p[ON_SURFACE], alpha));
                assert!(w >= 0.5, "{} at {alpha}: {w}", p[SURFACE]);
            }
        }
    }

    #[test]
    fn muted_primary_overlays_move_with_the_family() {
        // Spec §3: true for the live and blue palettes, not the vivid one.
        for p in &PALETTES[..2] {
            let u = uniforms_for(p);
            for alpha in [0.04, 0.08, 0.12, 0.16] {
                let w = u.weight(lab32_mix(p[SURFACE], p[PRIMARY], alpha));
                assert!(w >= 0.5, "{} at {alpha}: {w}", p[SURFACE]);
            }
        }
    }

    #[test]
    fn black_white_and_transparent_pixels_pass_through_unchanged() {
        let u = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.])
            .uniforms(HueBlend::from_accent(hex("#e5533c")));
        for px in [[0., 0., 0., 1.], [1., 1., 1., 1.], [0., 0., 0., 0.], [0., 0., 0., 0.5]] {
            assert_eq!(u.apply(px), px);
        }
    }

    #[test]
    fn apply_moves_a_surface_pixel_to_the_target() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let u = setup.uniforms(HueBlend::NEUTRAL);
        let [r, g, b, _] = hex("#151310").to_array_unpremul();
        // A 0.95-alpha surface pixel, premultiplied.
        let out = u.apply([r * 0.95, g * 0.95, b * 0.95, 0.95]);
        let straight = [out[0] / 0.95, out[1] / 0.95, out[2] / 0.95].map(|v| (v * 255.).round() as i32);
        for (got, want) in straight.iter().zip([9, 10, 10]) {
            assert!((got - want).abs() <= 1, "{straight:?}");
        }
        assert_eq!(out[3], 0.95);
    }

    #[test]
    fn the_shader_names_the_face_constants() {
        let frag = include_str!("shaders/material/main.frag");
        assert_eq!(SPECULAR_FLOOR, 0.15);
        for line in [
            "vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));",
            "vec3 transmitted = sampled * att * (1.0 - fresnel);",
            "float f0 = (mat_ior - 1.0) / (mat_ior + 1.0);",
            "float fresnel = f0 + (1.0 - f0) * pow(1.0 - surfaceCosine, 5.0);",
        ] {
            assert!(frag.contains(line), "main.frag lacks `{line}`");
        }
    }
}
```

Add `pub mod surface_remap;` to `src/render_helpers/mod.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri surface_remap`
Expected: FAIL with `not implemented` panics.

- [ ] **Step 3: Implement `HueBlend`, the setup, the family and `apply`**

```rust
    pub fn from_accent(accent: Color) -> Self {
        let lab = color_to_oklab(accent);
        let c = chroma(lab);
        if c < MIN_ACCENT_CHROMA {
            return Self::NEUTRAL;
        }
        Self {
            neutral: 0.,
            hue: [(lab[1] / c) as f32, (lab[2] / c) as f32],
        }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            neutral: a.neutral + (b.neutral - a.neutral) * t,
            hue: [0, 1].map(|i| a.hue[i] + (b.hue[i] - a.hue[i]) * t),
        }
    }
```

```rust
    pub fn with_face(remap: &SurfaceRemap, face_srgb: [f64; 3]) -> Self {
        let sources = [remap.surface, remap.container, remap.outline].map(color_to_oklab);
        let face = linear_to_oklab(face_srgb.map(srgb_to_linear));
        let face_c = chroma(face);
        let face_dir = if face_c > 1e-9 {
            [face[1] / face_c, face[2] / face_c]
        } else {
            [0., 0.]
        };
        let c0 = chroma(sources[0]);
        let role_offsets = sources.map(|s| (s[0] - sources[0][0], (chroma(s) - c0).max(0.)));
        Self {
            sources,
            face,
            face_dir,
            role_offsets,
            hue_chroma: remap.hue_chroma.map_or(0., |x| x.0),
        }
    }

    pub fn family(&self, blend: HueBlend) -> [[f64; 3]; 3] {
        let n = f64::from(blend.neutral);
        let h = blend.hue.map(f64::from);
        let ab = [0, 1].map(|i| n * self.face[i + 1] + self.hue_chroma * h[i]);
        let dir = [0, 1].map(|i| n * self.face_dir[i] + h[i]);
        self.role_offsets
            .map(|(dl, dc)| [self.face[0] + dl, ab[0] + dc * dir[0], ab[1] + dc * dir[1]])
    }
```

```rust
    pub fn apply(&self, rgba: [f32; 4]) -> [f32; 4] {
        let a = rgba[3];
        if a <= 0. {
            return rgba;
        }
        let straight = [0, 1, 2].map(|i| f64::from((rgba[i] / a).clamp(0., 1.)));
        let lab = linear_to_oklab(straight.map(srgb_to_linear)).map(|v| v as f32);
        let (d, offset) = self.nearest(lab);
        let w = 1. - smoothstep(self.radii[0], self.radii[1], d);
        if w <= 0. {
            return rgba;
        }
        let moved = [0, 1, 2].map(|i| f64::from(lab[i] + w * offset[i]));
        let srgb = oklab_to_linear(moved).map(|v| linear_to_srgb(v.clamp(0., 1.)) as f32);
        [srgb[0] * a, srgb[1] * a, srgb[2] * a, a]
    }
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `just test-one -p niri surface_remap`
Expected: PASS (12 tests). If a radius test fails on a palette, do not retune the radii silently. Record the failing palette, role and weight in the task note, and stop for the controller: the spec pins the starting values.

- [ ] **Step 5: Format, run the fast suite, commit**

```bash
rustup run nightly cargo fmt --all
just test-fast
tasks check
git add src/render_helpers/surface_remap.rs src/render_helpers/mod.rs
git commit -m "feat(render): add the surface remap color math"
```

---

### Task 4: The per-workspace hue resolver

**Files:**
- Create: `src/layout/shell_hue.rs`
- Modify: `src/layout/mod.rs` (add `pub mod shell_hue;` beside the other `pub mod` lines near the top)

**Interfaces:**
- Consumes:
  - `crate::render_helpers::surface_remap::{HueBlend, OutputHue}` (Task 3);
  - `crate::animation::{Animation, Clock}`, where `Animation::new(clock, from, to, initial_velocity, config: niri_config::Animation)`, plus `clamped_value()` and `is_done()`;
  - `niri_config::Color`.
- Produces (in `crate::layout::shell_hue`):
  - `pub const HOLD_DWELL: Duration`;
  - `WorkspaceObs<Ws, K> { pub id: Ws, pub active: Option<K>, pub windows: Vec<(K, Option<Color>)> }`;
  - `ShellHue<Ws, K>`, where `Ws: Copy + Eq + Hash` and `K: Copy + Eq`, with:
    - `new(clock: Clock) -> Self`
    - `update(&mut self, now: Duration, anim: niri_config::Animation, obs: &[WorkspaceObs<Ws, K>])`
    - `current(&self, ws: Ws) -> HueBlend`
    - `is_fading(&self, ws: Ws) -> bool`
    - `output_hue(&self, blend: (Ws, Ws, f64)) -> OutputHue`
    - `next_deadline(&self) -> Option<Duration>`

- [ ] **Step 1: Write the resolver skeleton and its failing tests**

Create `src/layout/shell_hue.rs`:

```rust
//! Which project hue each workspace holds for the shell
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §4).
//!
//! Pure: the caller passes what each workspace shows at `now`. A window
//! counts once it has stayed its workspace's active window for
//! `HOLD_DWELL`; the workspace then holds that window's accent while
//! windows without one are active, and goes neutral when the held window
//! closes, leaves, or loses its accent. Focus timestamps play no part.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use niri_config::Color;

use crate::animation::{Animation, Clock};
use crate::render_helpers::surface_remap::{HueBlend, OutputHue};

/// How long a window must stay active before it counts (spec §8).
pub const HOLD_DWELL: Duration = Duration::from_millis(1000);

/// One workspace as the resolver sees it this frame.
#[derive(Debug, Clone)]
pub struct WorkspaceObs<Ws, K> {
    pub id: Ws,
    pub active: Option<K>,
    /// Every window on the workspace with its folded accent.
    pub windows: Vec<(K, Option<Color>)>,
}

#[derive(Debug)]
struct Fade {
    from: HueBlend,
    to: HueBlend,
    anim: Animation,
}

#[derive(Debug)]
struct WorkspaceHue<K> {
    /// The active window and when it became active.
    candidate: Option<(K, Duration)>,
    /// The settled window whose accent the workspace holds.
    record: Option<K>,
    target: HueBlend,
    fade: Option<Fade>,
}

impl<K> WorkspaceHue<K> {
    fn current(&self) -> HueBlend {
        match &self.fade {
            Some(fade) => HueBlend::lerp(fade.from, fade.to, fade.anim.clamped_value() as f32),
            None => self.target,
        }
    }
}

#[derive(Debug)]
pub struct ShellHue<Ws, K> {
    clock: Clock,
    workspaces: HashMap<Ws, WorkspaceHue<K>>,
    now: Duration,
}

impl<Ws: Copy + Eq + Hash, K: Copy + Eq> ShellHue<Ws, K> {
    pub fn new(clock: Clock) -> Self {
        Self {
            clock,
            workspaces: HashMap::new(),
            now: Duration::ZERO,
        }
    }

    pub fn update(&mut self, now: Duration, anim: niri_config::Animation, obs: &[WorkspaceObs<Ws, K>]) {
        unimplemented!()
    }

    pub fn current(&self, ws: Ws) -> HueBlend {
        self.workspaces
            .get(&ws)
            .map_or(HueBlend::NEUTRAL, WorkspaceHue::current)
    }

    pub fn is_fading(&self, ws: Ws) -> bool {
        self.workspaces.get(&ws).is_some_and(|w| w.fade.is_some())
    }

    /// The output's point: the two workspaces it shows, blended by the
    /// fractional render index (`Monitor::workspace_blend`).
    pub fn output_hue(&self, (a, b, frac): (Ws, Ws, f64)) -> OutputHue {
        OutputHue {
            blend: HueBlend::lerp(self.current(a), self.current(b), frac as f32),
            animating: self.is_fading(a) || self.is_fading(b),
        }
    }

    /// The earliest moment a pending window settles, so a redraw can be
    /// scheduled for it.
    pub fn next_deadline(&self) -> Option<Duration> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use niri_config::animations::MaterialSignalAnim;

    use super::*;

    const RED: &str = "#e5533c";
    const CYAN: &str = "#3cc6e5";

    fn color(s: &str) -> Color {
        s.parse().unwrap()
    }

    fn hued(s: &str) -> HueBlend {
        HueBlend::from_accent(color(s))
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn off() -> niri_config::Animation {
        niri_config::Animation {
            off: true,
            ..MaterialSignalAnim::default().0
        }
    }

    fn on() -> niri_config::Animation {
        MaterialSignalAnim::default().0
    }

    fn ws(id: u64, active: Option<u64>, windows: &[(u64, Option<&str>)]) -> WorkspaceObs<u64, u64> {
        WorkspaceObs {
            id,
            active,
            windows: windows.iter().map(|(k, a)| (*k, a.map(color))).collect(),
        }
    }

    fn resolver() -> ShellHue<u64, u64> {
        ShellHue::new(Clock::with_time(Duration::ZERO))
    }

    /// Agent 1 (red) settled on workspace 1 at 1000 ms.
    fn settled_on_red() -> ShellHue<u64, u64> {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED)), (2, None)])];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.current(1), hued(RED));
        r
    }

    #[test]
    fn settles_after_the_dwell() {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(0), off(), &w);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        r.update(ms(999), off(), &w);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn a_brief_visit_does_not_count() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (2, None), (3, Some(CYAN))];
        r.update(ms(2000), off(), &[ws(1, Some(3), &windows)]);
        r.update(ms(2300), off(), &[ws(1, Some(2), &windows)]);
        r.update(ms(8000), off(), &[ws(1, Some(2), &windows)]);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn hold_across_a_window_without_accent() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (2, None)];
        r.update(ms(2000), off(), &[ws(1, Some(2), &windows)]);
        r.update(ms(60_000), off(), &[ws(1, Some(2), &windows)]);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn a_settled_second_agent_takes_over() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (3, Some(CYAN))];
        r.update(ms(2000), off(), &[ws(1, Some(3), &windows)]);
        r.update(ms(3000), off(), &[ws(1, Some(3), &windows)]);
        assert_eq!(r.current(1), hued(CYAN));
    }

    #[test]
    fn closing_the_recorded_window_goes_neutral() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(1, Some(2), &[(2, None)])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn lost_accent_goes_neutral() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(1, Some(1), &[(1, None)])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn recorded_accent_change_is_followed() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(1, Some(2), &[(1, Some(CYAN)), (2, None)])]);
        assert_eq!(r.current(1), hued(CYAN));
    }

    #[test]
    fn moved_window_clears_its_origin_record() {
        let mut r = settled_on_red();
        let both = |t: u64, r: &mut ShellHue<u64, u64>| {
            r.update(
                ms(t),
                off(),
                &[ws(1, Some(2), &[(2, None)]), ws(2, Some(1), &[(1, Some(RED))])],
            );
        };
        both(2000, &mut r);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        assert_eq!(r.current(2), HueBlend::NEUTRAL);
        both(2999, &mut r);
        assert_eq!(r.current(2), HueBlend::NEUTRAL);
        both(3000, &mut r);
        assert_eq!(r.current(2), hued(RED));
    }

    #[test]
    fn switch_blend_mixes_two_workspaces() {
        let mut r = resolver();
        let w = [
            ws(1, Some(1), &[(1, Some(RED))]),
            ws(2, Some(2), &[(2, Some(CYAN))]),
        ];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        let point = r.output_hue((1, 2, 0.25));
        assert_eq!(point.blend, HueBlend::lerp(hued(RED), hued(CYAN), 0.25));
        assert!(!point.animating);
    }

    #[test]
    fn two_outputs_hold_their_own_hues() {
        let mut r = resolver();
        let w = [
            ws(1, Some(1), &[(1, Some(RED))]),
            ws(2, Some(2), &[(2, Some(CYAN))]),
        ];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.output_hue((1, 1, 0.)).blend, hued(RED));
        assert_eq!(r.output_hue((2, 2, 0.)).blend, hued(CYAN));
    }

    #[test]
    fn fade_reports_animating_until_it_ends() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut r: ShellHue<u64, u64> = ShellHue::new(clock.clone());
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(0), on(), &w);
        clock.set_unadjusted(ms(1000));
        r.update(ms(1000), on(), &w);
        assert!(r.is_fading(1));
        assert!(r.output_hue((1, 1, 0.)).animating);
        clock.set_unadjusted(ms(1500));
        r.update(ms(1500), on(), &w);
        assert!(!r.is_fading(1));
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn interrupted_fade_continues_from_its_point() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut r: ShellHue<u64, u64> = ShellHue::new(clock.clone());
        let red = [ws(1, Some(1), &[(1, Some(RED)), (2, Some(CYAN))])];
        let cyan = [ws(1, Some(2), &[(1, Some(RED)), (2, Some(CYAN))])];
        r.update(ms(0), on(), &red);
        clock.set_unadjusted(ms(1000));
        r.update(ms(1000), on(), &red); // fade neutral -> red starts
        clock.set_unadjusted(ms(1500));
        r.update(ms(1500), on(), &red); // red settled
        r.update(ms(1500), on(), &cyan);
        clock.set_unadjusted(ms(2500));
        r.update(ms(2500), on(), &cyan); // fade red -> cyan starts
        clock.set_unadjusted(ms(2600));
        r.update(ms(2600), on(), &cyan);
        let before = r.current(1);
        // Agent 1's accent turns cyan too, so the record (agent 2) stays but
        // agent 1 being re-recorded is impossible within this frame; instead
        // drop agent 2's accent: the target flips to neutral mid-fade.
        let lost = [ws(1, Some(2), &[(1, Some(RED)), (2, None)])];
        r.update(ms(2600), on(), &lost);
        let after = r.current(1);
        assert_eq!(before, after, "a new fade starts where the old one was");
        assert!(r.is_fading(1));
        clock.set_unadjusted(ms(3200));
        r.update(ms(3200), on(), &lost);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn removed_workspaces_are_forgotten() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(2, None, &[])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        assert!(!r.workspaces.contains_key(&1));
    }

    #[test]
    fn next_deadline_is_the_pending_settle() {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(400), off(), &w);
        assert_eq!(r.next_deadline(), Some(ms(1400)));
        r.update(ms(1400), off(), &w);
        assert_eq!(r.next_deadline(), None);
    }
}
```

Add `pub mod shell_hue;` to `src/layout/mod.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri shell_hue`
Expected: FAIL with `not implemented` panics.

- [ ] **Step 3: Implement `update` and `next_deadline`**

```rust
    pub fn update(&mut self, now: Duration, anim: niri_config::Animation, obs: &[WorkspaceObs<Ws, K>]) {
        self.now = now;
        self.workspaces.retain(|id, _| obs.iter().any(|o| o.id == *id));

        for o in obs {
            let ws = self.workspaces.entry(o.id).or_insert_with(|| WorkspaceHue {
                candidate: None,
                record: None,
                target: HueBlend::NEUTRAL,
                fade: None,
            });
            // `None`: not on this workspace; `Some(None)`: here, no accent.
            let accent_of = |k: K| o.windows.iter().find(|(w, _)| *w == k).map(|(_, a)| *a);

            if ws.candidate.map(|(k, _)| k) != o.active {
                ws.candidate = o.active.map(|k| (k, now));
            }
            if let Some((k, since)) = ws.candidate {
                if now.saturating_sub(since) >= HOLD_DWELL && matches!(accent_of(k), Some(Some(_))) {
                    ws.record = Some(k);
                }
            }

            let accent = ws.record.and_then(|k| accent_of(k).flatten());
            if accent.is_none() {
                ws.record = None;
            }
            let target = accent.map_or(HueBlend::NEUTRAL, HueBlend::from_accent);

            if target != ws.target {
                let from = ws.current();
                ws.fade = Some(Fade {
                    from,
                    to: target,
                    anim: Animation::new(self.clock.clone(), 0., 1., 0., anim),
                });
                ws.target = target;
            }
            if ws.fade.as_ref().is_some_and(|fade| fade.anim.is_done()) {
                ws.fade = None;
            }
        }
    }
```

```rust
    pub fn next_deadline(&self) -> Option<Duration> {
        self.workspaces
            .values()
            .filter_map(|w| {
                let (k, since) = w.candidate?;
                (w.record != Some(k)).then_some(since + HOLD_DWELL)
            })
            .filter(|deadline| *deadline > self.now)
            .min()
    }
```

If the borrow checker rejects `self.clock.clone()` while `ws` borrows `self.workspaces`, clone the clock into a local before the loop (`let clock = self.clock.clone();`) and use `clock.clone()` inside.

- [ ] **Step 4: Run the tests to see them pass**

Run: `just test-one -p niri shell_hue`
Expected: PASS (14 tests). In `interrupted_fade_continues_from_its_point`, `before == after` holds because the new fade's `from` is `current()` at the same clock time. If `Animation::new` returns a nonzero value at t = 0, the test catches it.

- [ ] **Step 5: Format, run the fast suite, commit**

```bash
rustup run nightly cargo fmt --all
just test-fast
tasks check
git add src/layout/shell_hue.rs src/layout/mod.rs
git commit -m "feat(layout): add the per-workspace shell hue resolver"
```

---

### Task 5: The remap shader, its render element, and `MappedLayer`

**Files:**
- Create: `src/render_helpers/shaders/surface_remap.frag`
- Create: `src/render_helpers/surface_remap_element.rs`
- Modify: `src/render_helpers/mod.rs` (add `pub mod surface_remap_element;`)
- Modify: `src/render_helpers/shaders/mod.rs` (the `Shaders` field and its compile, beside `clipped_surface`, lines 106–200 and 268)
- Modify: `src/layer/mapped.rs` (fields, `new`, `update_config`, `render_normal`, `render_popups`, `are_animations_ongoing`, new methods, and the render element enum)
- Create: `src/tests/shell_hue.rs`
- Modify: `src/tests/mod.rs` (add `mod shell_hue;` in alphabetical order)

**Interfaces:**
- Consumes:
  - `RemapSetup`, `RemapUniforms` and `OutputHue` (Task 3);
  - `ResolvedLayerRules::surface_remap` (Task 2);
  - `crate::render_helpers::damage::ExtraDamage`;
  - `Shaders::get(renderer)`;
  - smithay's `GlesTexProgram`, `Uniform`, `UniformName` and `UniformType`.
- Produces:
  - `SurfaceRemapRenderElement<R>`, with:
    - `new(inner: WaylandSurfaceRenderElement<R>, program: GlesTexProgram, uniforms: RemapUniforms)`
    - `shader(renderer: &mut R) -> Option<&GlesTexProgram>`
  - `Shaders::surface_remap: Option<GlesTexProgram>`.
  - New `MappedLayer` methods:
    - `set_shell_hue(&mut self, hue: OutputHue)`
    - `shell_hue(&self) -> OutputHue`
    - `remap_uniforms(&self) -> Option<RemapUniforms>`
    - `remap_damage_commit(&self) -> CommitCounter` (test inspection)

- [ ] **Step 1: Write the failing layer integration tests**

Create `src/tests/shell_hue.rs`:

```rust
use niri_config::Config;
use smithay::backend::renderer::element::Element as _;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity,
};

use super::client::{ClientId, LayerConfigureProps};
use super::*;
use crate::render_helpers::surface_remap::{HueBlend, OutputHue};

const GLASS: &str = r##"
material "tg" {
    glass {
        ior 1.28
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 12
    }
}
"##;

const RULE: &str = r##"
layer-rule {
    match namespace="^bar$"
    surface-remap {
        surface "#151310"
        container "#221f1c"
        outline "#4d453c"
        material "tg"
        hue-chroma 0.03
    }
}
"##;

fn config(text: &str) -> Config {
    Config::parse_mem(text).unwrap()
}

pub(super) fn look(extra: &str) -> Config {
    config(&format!("animations {{ off; }}\n{GLASS}\n{RULE}\n{extra}"))
}

pub(super) fn open_layer(
    f: &mut Fixture,
    id: ClientId,
    namespace: &str,
    kb: Option<KeyboardInteractivity>,
) {
    let layer = f.client(id).create_layer(None, Layer::Top, namespace);
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Left | Anchor::Right | Anchor::Top),
        size: Some((0, 50)),
        kb_interactivity: kb,
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(id);
    let layer = f.client(id).layer(&surface);
    layer.attach_new_buffer();
    layer.set_size(100, 50);
    layer.ack_last_and_commit();
    f.double_roundtrip(id);
}

pub(super) fn layer<'a>(f: &'a mut Fixture, namespace: &str) -> &'a mut crate::layer::MappedLayer {
    f.niri()
        .mapped_layer_surfaces
        .values_mut()
        .find(|m| m.surface().namespace() == namespace)
        .unwrap()
}

fn red() -> HueBlend {
    HueBlend::from_accent("#e5533c".parse().unwrap())
}

#[test]
fn layer_with_the_rule_gets_a_remap() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_layer(&mut f, id, "other", None);
    assert!(layer(&mut f, "bar").remap_uniforms().is_some());
    assert!(layer(&mut f, "other").remap_uniforms().is_none());
}

#[test]
fn two_rules_replace_the_node_whole() {
    let second = r##"
layer-rule {
    match namespace="^bar$"
    surface-remap {
        surface "#111318"
        container "#1d2025"
        outline "#414751"
        material "tg"
    }
}
"##;
    let mut f = Fixture::with_config(look(second));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let remap = layer(&mut f, "bar").rules().surface_remap.clone().unwrap();
    assert_eq!(remap.surface, "#111318".parse().unwrap());
    assert_eq!(remap.hue_chroma, None, "the later node's absent field is not merged");
}

#[test]
fn config_reload_recomputes_the_face() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let before = layer(&mut f, "bar").remap_uniforms().unwrap();
    let text = format!("animations {{ off; }}\n{}\n{RULE}", GLASS.replace("distance 12", "distance 11"));
    f.niri_state().reload_config(Ok(config(&text)));
    let after = layer(&mut f, "bar").remap_uniforms().unwrap();
    assert_eq!(before.anchors, after.anchors, "Noctalia's colors did not change");
    assert_ne!(before.offsets, after.offsets, "the face moved with the glass");
}

#[test]
fn hue_change_damages_the_layer_once() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let mapped = layer(&mut f, "bar");
    mapped.set_shell_hue(OutputHue::default());
    let start = mapped.remap_damage_commit();
    mapped.set_shell_hue(OutputHue {
        blend: red(),
        animating: false,
    });
    let changed = mapped.remap_damage_commit();
    assert_ne!(start, changed);
    mapped.set_shell_hue(OutputHue {
        blend: red(),
        animating: false,
    });
    assert_eq!(changed, mapped.remap_damage_commit(), "an unchanged hue adds no damage");
}

#[test]
fn a_running_fade_keeps_the_layer_animating() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_layer(&mut f, id, "other", None);
    let fading = OutputHue {
        blend: red(),
        animating: true,
    };
    layer(&mut f, "bar").set_shell_hue(fading);
    layer(&mut f, "other").set_shell_hue(fading);
    assert!(layer(&mut f, "bar").are_animations_ongoing());
    assert!(!layer(&mut f, "other").are_animations_ongoing(), "no rule, nothing to animate");
}
```

Add `mod shell_hue;` to `src/tests/mod.rs` after `mod signal;`.

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri tests::shell_hue`
Expected: FAIL to compile with `no method named remap_uniforms`.

- [ ] **Step 3: Write the shader**

Create `src/render_helpers/shaders/surface_remap.frag`:

```glsl
#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision highp float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

// The shell remap (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md
// §3). Mirrors RemapUniforms::apply in surface_remap.rs.
uniform vec3 remap_anchor0;
uniform vec3 remap_anchor1;
uniform vec3 remap_anchor2;
uniform vec3 remap_offset0;
uniform vec3 remap_offset1;
uniform vec3 remap_offset2;
uniform vec2 remap_radii;

// From material/common.frag, concatenated after this file.
vec3 srgbToLinear(vec3 c);
vec3 linearToSrgb(vec3 c);
vec3 linearToOklab(vec3 c);
vec3 oklabToLinear(vec3 lab);

// Distance from p to the segment a -> b, and the fraction along it.
vec2 segment(vec3 p, vec3 a, vec3 b) {
    vec3 ab = b - a;
    float len2 = dot(ab, ab);
    float t = len2 > 0.0 ? clamp(dot(p - a, ab) / len2, 0.0, 1.0) : 0.0;
    return vec2(length(p - (a + t * ab)), t);
}

vec4 remap(vec4 color) {
    if (color.a <= 0.0)
        return color;
    vec3 straight = clamp(color.rgb / color.a, 0.0, 1.0);
    vec3 lab = linearToOklab(srgbToLinear(straight));
    vec2 s0 = segment(lab, remap_anchor0, remap_anchor1);
    vec2 s1 = segment(lab, remap_anchor1, remap_anchor2);
    float d;
    vec3 offset;
    if (s0.x <= s1.x) {
        d = s0.x;
        offset = mix(remap_offset0, remap_offset1, s0.y);
    } else {
        d = s1.x;
        offset = mix(remap_offset1, remap_offset2, s1.y);
    }
    float w = 1.0 - smoothstep(remap_radii.x, remap_radii.y, d);
    if (w <= 0.0)
        return color;
    vec3 moved = linearToSrgb(clamp(oklabToLinear(lab + w * offset), 0.0, 1.0));
    return vec4(moved * color.a, color.a);
}

void main() {
    vec4 color = texture2D(tex, v_coords);
#if defined(NO_ALPHA)
    color = vec4(color.rgb, 1.0);
#endif

    color = remap(color);

    // Apply final alpha and tint.
    color = color * alpha;

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = vec4(0.0, 0.2, 0.0, 0.2) + color * 0.8;
#endif

    gl_FragColor = color;
}
```

- [ ] **Step 4: Compile the program in `Shaders`**

In `src/render_helpers/shaders/mod.rs`, add the field after `clipped_surface`:

```rust
    pub surface_remap: Option<GlesTexProgram>,
```

After the `clipped_surface` compile block:

```rust
        let surface_remap = renderer
            .compile_custom_texture_shader(
                concat!(
                    include_str!("surface_remap.frag"),
                    "\n// ---- common\n",
                    include_str!("material/common.frag"),
                ),
                &[
                    UniformName::new("remap_anchor0", UniformType::_3f),
                    UniformName::new("remap_anchor1", UniformType::_3f),
                    UniformName::new("remap_anchor2", UniformType::_3f),
                    UniformName::new("remap_offset0", UniformType::_3f),
                    UniformName::new("remap_offset1", UniformType::_3f),
                    UniformName::new("remap_offset2", UniformType::_3f),
                    UniformName::new("remap_radii", UniformType::_2f),
                ],
            )
            .map_err(|err| {
                warn!("error compiling surface remap shader: {err:?}");
            })
            .ok();
```

Add `surface_remap,` to the `Self { … }` initializer after `clipped_surface,`.

- [ ] **Step 5: Write the render element**

Create `src/render_helpers/surface_remap_element.rs`:

```rust
//! A layer surface drawn through the surface remap program
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §7). Geometry,
//! damage and opaque regions are the client's: the remap moves colors only.

use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement, UnderlyingStorage};
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesRenderer, GlesTexProgram, Uniform};
use smithay::backend::renderer::utils::{CommitCounter, DamageSet, OpaqueRegions};
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Physical, Rectangle, Scale, Transform};

use super::renderer::{AsGlesFrame as _, NiriRenderer};
use super::shaders::Shaders;
use super::surface_remap::RemapUniforms;
use crate::backend::tty::{TtyFrame, TtyRenderer, TtyRendererError};

#[derive(Debug)]
pub struct SurfaceRemapRenderElement<R: NiriRenderer> {
    inner: WaylandSurfaceRenderElement<R>,
    program: GlesTexProgram,
    uniforms: RemapUniforms,
}

impl<R: NiriRenderer> SurfaceRemapRenderElement<R> {
    pub fn new(
        inner: WaylandSurfaceRenderElement<R>,
        program: GlesTexProgram,
        uniforms: RemapUniforms,
    ) -> Self {
        Self {
            inner,
            program,
            uniforms,
        }
    }

    pub fn shader(renderer: &mut R) -> Option<&GlesTexProgram> {
        Shaders::get(renderer).surface_remap.as_ref()
    }

    fn compute_uniforms(&self) -> Vec<Uniform<'static>> {
        let u = &self.uniforms;
        vec![
            Uniform::new("remap_anchor0", u.anchors[0]),
            Uniform::new("remap_anchor1", u.anchors[1]),
            Uniform::new("remap_anchor2", u.anchors[2]),
            Uniform::new("remap_offset0", u.offsets[0]),
            Uniform::new("remap_offset1", u.offsets[1]),
            Uniform::new("remap_offset2", u.offsets[2]),
            Uniform::new("remap_radii", u.radii),
        ]
    }
}

impl<R: NiriRenderer> Element for SurfaceRemapRenderElement<R> {
    fn id(&self) -> &Id {
        self.inner.id()
    }

    fn current_commit(&self) -> CommitCounter {
        self.inner.current_commit()
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.inner.geometry(scale)
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        self.inner.src()
    }

    fn transform(&self) -> Transform {
        self.inner.transform()
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        // Hue and target changes are damaged by MappedLayer's ExtraDamage.
        self.inner.damage_since(scale, commit)
    }

    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.inner.opaque_regions(scale)
    }

    fn alpha(&self) -> f32 {
        self.inner.alpha()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }
}

impl RenderElement<GlesRenderer> for SurfaceRemapRenderElement<GlesRenderer> {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        frame.override_default_tex_program(self.program.clone(), self.compute_uniforms());
        RenderElement::<GlesRenderer>::draw(
            &self.inner,
            frame,
            src,
            dst,
            damage,
            opaque_regions,
            cache,
        )?;
        frame.clear_tex_program_override();
        Ok(())
    }

    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        // The remap must run, so the surface is never scanned out directly.
        None
    }
}

impl<'render> RenderElement<TtyRenderer<'render>>
    for SurfaceRemapRenderElement<TtyRenderer<'render>>
{
    fn draw(
        &self,
        frame: &mut TtyFrame<'render, '_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), TtyRendererError<'render>> {
        frame
            .as_gles_frame()
            .override_default_tex_program(self.program.clone(), self.compute_uniforms());
        RenderElement::draw(&self.inner, frame, src, dst, damage, opaque_regions, cache)?;
        frame.as_gles_frame().clear_tex_program_override();
        Ok(())
    }

    fn underlying_storage(
        &self,
        _renderer: &mut TtyRenderer<'render>,
    ) -> Option<UnderlyingStorage<'_>> {
        None
    }
}
```

Add `pub mod surface_remap_element;` to `src/render_helpers/mod.rs`.

- [ ] **Step 6: Give `MappedLayer` its remap state and render through it**

In `src/layer/mapped.rs`:

Imports, added to the existing ones:

```rust
use smithay::backend::renderer::utils::CommitCounter;

use crate::render_helpers::damage::ExtraDamage;
use crate::render_helpers::surface_remap::{OutputHue, RemapSetup, RemapUniforms};
use crate::render_helpers::surface_remap_element::SurfaceRemapRenderElement;
```

Fields, appended to `MappedLayer`:

```rust
    /// The surface remap for this layer's rule, rebuilt on config and rule changes.
    remap: Option<RemapSetup>,

    /// What the layer's output shows this frame.
    shell_hue: OutputHue,

    /// Uniforms last drawn, so a change damages the surface once.
    remap_drawn: Option<RemapUniforms>,

    /// Damage for the surface and for its pop-ups when the remap changes.
    remap_damage: ExtraDamage,
    popup_remap_damage: ExtraDamage,
```

A helper above `impl MappedLayer`:

```rust
fn remap_setup(rules: &ResolvedLayerRules, config: &Config) -> Option<RemapSetup> {
    let remap = rules.surface_remap.as_ref()?;
    let material = config
        .materials
        .iter()
        .find(|m| m.name == remap.material.name)
        .expect("surface-remap materials are validated at config load");
    Some(RemapSetup::new(remap, &material.resolve().glass))
}
```

In `new`, before `Self { … }`, add `let remap = remap_setup(&rules, config);`. Then add these fields to the initializer:

```rust
            remap,
            shell_hue: OutputHue::default(),
            remap_drawn: None,
            remap_damage: ExtraDamage::new(),
            popup_remap_damage: ExtraDamage::new(),
```

At the end of `update_config`:

```rust
        self.remap = remap_setup(&self.rules, config);
```

The render element enum:

```rust
niri_render_elements! {
    LayerSurfaceRenderElement<R> => {
        Wayland = WaylandSurfaceRenderElement<R>,
        SurfaceRemap = SurfaceRemapRenderElement<R>,
        SolidColor = SolidColorRenderElement,
        Shadow = ShadowRenderElement,
        BackgroundEffect = BackgroundEffectElement,
        Damage = ExtraDamage,
    }
}
```

New methods in `impl MappedLayer`:

```rust
    pub fn remap_uniforms(&self) -> Option<RemapUniforms> {
        self.remap
            .as_ref()
            .map(|setup| setup.uniforms(self.shell_hue.blend))
    }

    pub fn shell_hue(&self) -> OutputHue {
        self.shell_hue
    }

    /// Takes this frame's hue for the layer's output; a change in what the
    /// remap draws damages the surface and its pop-ups once.
    pub fn set_shell_hue(&mut self, hue: OutputHue) {
        self.shell_hue = hue;
        let uniforms = self.remap_uniforms();
        if uniforms != self.remap_drawn {
            self.remap_drawn = uniforms;
            self.remap_damage.damage_all();
            self.popup_remap_damage.damage_all();
        }
    }

    pub fn remap_damage_commit(&self) -> CommitCounter {
        self.remap_damage.current_commit()
    }
```

`current_commit` comes from smithay's `Element` trait: add `use smithay::backend::renderer::element::Element as _;` to the imports.

`are_animations_ongoing`:

```rust
    pub fn are_animations_ongoing(&self) -> bool {
        self.rules.baba_is_float || (self.remap.is_some() && self.shell_hue.animating)
    }
```

In `render_normal`, replace the non-block-out branch's `push_elements_from_surface_tree(…)` call:

```rust
            // Layer surfaces don't have extra geometry like windows.
            let buf_pos = location;

            let remap = self.remap_uniforms().and_then(|uniforms| {
                SurfaceRemapRenderElement::shader(ctx.renderer)
                    .cloned()
                    .map(|program| (uniforms, program))
            });
            if let Some((uniforms, program)) = remap {
                let geometry = Rectangle::new(location, self.block_out_buffer.size());
                push(self.remap_damage.render(geometry).into());
                push_elements_from_surface_tree(
                    ctx.renderer,
                    surface,
                    buf_pos.to_physical_precise_round(scale),
                    scale,
                    alpha,
                    Kind::ScanoutCandidate,
                    &mut |elem| {
                        push(SurfaceRemapRenderElement::new(elem, program.clone(), uniforms).into())
                    },
                );
            } else {
                push_elements_from_surface_tree(
                    ctx.renderer,
                    surface,
                    buf_pos.to_physical_precise_round(scale),
                    scale,
                    alpha,
                    Kind::ScanoutCandidate,
                    &mut |elem| push(elem.into()),
                );
            }
```

A program that failed to compile has already logged `error compiling surface remap shader`. The layer then draws unremapped, the same policy niri applies to every custom texture program (`clipped_surface`). The Task 8 pilot greps the nested log for that line.

In `render_popups`, compute the program once before the loop:

```rust
        let remap = self.remap_uniforms().and_then(|uniforms| {
            SurfaceRemapRenderElement::shader(ctx.renderer)
                .cloned()
                .map(|program| (uniforms, program))
        });
        let mut popup_bounds: Option<Rectangle<f64, Logical>> = None;
```

Replace the loop's existing `popup_rules` match with one decision for both the popup rules and the remap. Input-method popups belong to another client, and get neither the rules nor the remap (spec §5, §7):

```rust
            let (popup_rules, popup_remap) =
                popup_treatment(PopupClass::of(&popup), self.rules.popups, remap.clone());
```

Add the decision as free items at the bottom of `mapped.rs`, above `impl Drop`:

```rust
/// Which kind of popup a layer surface's popup is, for rule decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupClass {
    Xdg,
    InputMethod,
}

impl PopupClass {
    fn of(popup: &PopupKind) -> Self {
        match popup {
            PopupKind::Xdg(_) => Self::Xdg,
            PopupKind::InputMethod(_) => Self::InputMethod,
        }
    }
}

/// The popup rules and remap a popup gets. Input-method popups (candidate
/// windows) belong to the input method's client: they take neither the
/// layer's popup rules nor its remap.
fn popup_treatment<T>(
    class: PopupClass,
    popups: niri_config::ResolvedPopupsRules,
    remap: Option<T>,
) -> (niri_config::ResolvedPopupsRules, Option<T>) {
    match class {
        PopupClass::Xdg => (popups, remap),
        PopupClass::InputMethod => (niri_config::ResolvedPopupsRules::default(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_method_popups_take_neither_rules_nor_remap() {
        let popups = niri_config::ResolvedPopupsRules {
            opacity: Some(0.5),
            ..Default::default()
        };
        assert_eq!(
            popup_treatment(PopupClass::InputMethod, popups, Some(7u8)),
            (niri_config::ResolvedPopupsRules::default(), None)
        );
        assert_eq!(popup_treatment(PopupClass::Xdg, popups, Some(7u8)), (popups, Some(7u8)));
    }
}
```

Inside the loop, replace the popup's `push_elements_from_surface_tree(…)` with the same two-armed form as `render_normal`. Use `surface_loc` as the position and `popup_remap` as the arm's condition (`if let Some((uniforms, program)) = popup_remap.clone()`). In the remap arm, grow the bounds after `let geometry = …`:

```rust
            if popup_remap.is_some() {
                popup_bounds = Some(popup_bounds.map_or(geometry, |b| b.merge(geometry)));
            }
```

After the loop:

```rust
        if let Some(bounds) = popup_bounds {
            push(self.popup_remap_damage.render(bounds).into());
        }
```

`render_popups` computes `geometry` after the surface push. Move its `let geometry = Rectangle::new(location + offset.to_f64(), popup_geo.size.to_f64());` above the push so both arms can use it.

- [ ] **Step 7: Run the layer tests**

Run: `just test-one -p niri tests::shell_hue` and `just test-one -p niri input_method_popups`
Expected: PASS (5 tests and 1 test).

- [ ] **Step 8: Run the whole niri suite affected by the enum change**

Run: `just test-fast`
Expected: PASS. The new enum variants break nothing that matches exhaustively on `LayerSurfaceRenderElement`. If a match elsewhere is exhaustive, add the two arms there, following the existing `Wayland` arm.

- [ ] **Step 9: Format, commit**

```bash
rustup run nightly cargo fmt --all
tasks check
git add src/render_helpers/shaders/surface_remap.frag src/render_helpers/surface_remap_element.rs src/render_helpers/mod.rs src/render_helpers/shaders/mod.rs src/layer/mapped.rs src/tests/shell_hue.rs src/tests/mod.rs
git commit -m "feat(layer): draw remapped layer surfaces through the surface remap program"
```

---

### Task 6: Drive the resolver from niri and hand each output's hue to its layers

**Files:**
- Modify: `src/layout/monitor.rs` (add `workspace_blend` after `workspace_render_idx`, about line 1440)
- Modify: `src/niri.rs`:
  - fields `shell_hue` and `shell_hue_timer`, beside `signal_deadlines`, initialized in `Niri::new` beside `mapped_layer_surfaces: HashMap::new()` (about line 2707);
  - `update_render_elements` (line 4293);
  - `update_shell_hue`, a new method;
  - `arm_shell_hue_timer`, a new method beside `arm_signal_timer` (line 2402);
  - the redraw path that calls `arm_signal_timer` (about line 4864).
- Modify: `src/tests/shell_hue.rs` (append the resolver-driven tests)

**Interfaces:**
- Consumes:
  - `ShellHue`, `WorkspaceObs` and `HOLD_DWELL` (Task 4);
  - `MappedLayer::set_shell_hue` (Task 5);
  - `Workspace::id()`, `Workspace::active_window()` and `Workspace::windows()`;
  - `Mapped::id() -> MappedId`, and `LayoutElement::signal(now) -> Option<Folded>` with `Folded::accent: Option<Color>`;
  - `Layout::workspaces()` and `Layout::monitor_for_output()`.
- Produces:
  - `Monitor::workspace_blend(&self) -> (WorkspaceId, WorkspaceId, f64)`;
  - `Niri::shell_hue: ShellHue<WorkspaceId, MappedId>`;
  - `Niri::update_shell_hue(&mut self)`;
  - `Niri::arm_shell_hue_timer(&mut self)`.

- [ ] **Step 1: Write the failing resolver-driven tests**

Append to `src/tests/shell_hue.rs`:

```rust
use std::time::Duration;

use niri_ipc::{SignalLevel, SignalMotion};
use wayland_client::protocol::wl_surface::WlSurface;

use crate::layout::LayoutElement as _;
use crate::niri::SetWindowSignalArgs;
use crate::utils::with_toplevel_role;

fn open_window(f: &mut Fixture, id: ClientId, title: &str) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.set_title(title);
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

fn window_id(f: &mut Fixture, title: &str) -> u64 {
    f.niri()
        .layout
        .windows()
        .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title)))
        .map(|(_, m)| m.id().get())
        .unwrap()
}

fn set_accent(f: &mut Fixture, id: u64, accent: &str) {
    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id,
            source: String::from("familiar"),
            accent: Some(String::from(accent)),
            level: SignalLevel::Quiet,
            motion: SignalMotion::Static,
            tag: None,
            ttl_ms: None,
            after_level: None,
            after_motion: None,
            until_focus: false,
        })
        .unwrap();
}

fn frame_at(f: &mut Fixture, ms: u64) {
    let niri = f.niri();
    niri.clock.set_unadjusted(Duration::from_millis(ms));
    niri.update_render_elements(None);
}

#[test]
fn hue_follows_the_active_agent_after_the_dwell() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_window(&mut f, id, "agent");
    let wid = window_id(&mut f, "agent");
    set_accent(&mut f, wid, "#e5533c");

    frame_at(&mut f, 10_000);
    assert_eq!(layer(&mut f, "bar").shell_hue().blend, HueBlend::NEUTRAL);
    frame_at(&mut f, 10_500);
    assert_eq!(layer(&mut f, "bar").shell_hue().blend, HueBlend::NEUTRAL);
    frame_at(&mut f, 11_100);
    assert_eq!(layer(&mut f, "bar").shell_hue().blend, red());
}

#[test]
fn an_exclusive_panel_keeps_the_hue() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_window(&mut f, id, "agent");
    let wid = window_id(&mut f, "agent");
    set_accent(&mut f, wid, "#e5533c");
    frame_at(&mut f, 10_000);
    frame_at(&mut f, 11_100);

    open_layer(&mut f, id, "panel", Some(KeyboardInteractivity::Exclusive));
    frame_at(&mut f, 12_000);
    frame_at(&mut f, 14_000);
    assert_eq!(layer(&mut f, "bar").shell_hue().blend, red());
}

#[test]
fn a_pending_settle_arms_a_redraw_deadline() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_window(&mut f, id, "agent");
    let wid = window_id(&mut f, "agent");
    set_accent(&mut f, wid, "#e5533c");
    frame_at(&mut f, 10_000);
    assert_eq!(
        f.niri().shell_hue.next_deadline(),
        Some(Duration::from_millis(11_000))
    );
}

#[test]
fn workspace_blend_names_the_active_workspace_at_rest() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "agent");
    let niri = f.niri();
    let mon = niri.layout.monitors().next().unwrap();
    let (a, _b, frac) = mon.workspace_blend();
    assert_eq!(a, mon.active_workspace_ref().id());
    assert_eq!(frac, 0.);
}
```

Merge the new `use` lines into the file's existing import block (rustfmt groups them). Remove any import duplicated from Task 5.

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri tests::shell_hue`
Expected: FAIL to compile with `no field shell_hue on type Niri` and `no method workspace_blend`.

- [ ] **Step 3: Add `Monitor::workspace_blend`**

In `src/layout/monitor.rs`, after `workspace_render_idx`:

```rust
    /// The two workspaces the output shows and how far it is between them,
    /// for the shell hue's switch blend (spec §4).
    pub fn workspace_blend(&self) -> (WorkspaceId, WorkspaceId, f64) {
        let last = self.workspaces.len() - 1;
        let idx = self.workspace_render_idx().clamp(0., last as f64);
        let lo = idx.floor() as usize;
        let hi = (lo + 1).min(last);
        (self.workspaces[lo].id(), self.workspaces[hi].id(), idx - lo as f64)
    }
```

Import `WorkspaceId` from `super::workspace` if `monitor.rs` does not already.

- [ ] **Step 4: Add the resolver to `Niri` and drive it every frame**

In `src/niri.rs`, add the fields beside `signal_deadlines`:

```rust
    /// Which project hue each workspace holds for remapped layers (spec §4).
    pub shell_hue: crate::layout::shell_hue::ShellHue<WorkspaceId, MappedId>,
    /// Redraw armed for the next pending hue settle.
    pub shell_hue_timer: Option<RegistrationToken>,
```

Initialize them in `Niri::new` beside `mapped_layer_surfaces`. The clock there is the `clock` local `Niri::new` builds the layout with:

```rust
            shell_hue: crate::layout::shell_hue::ShellHue::new(clock.clone()),
            shell_hue_timer: None,
```

The new method:

```rust
    /// Feeds the shell hue resolver what every workspace shows now.
    fn update_shell_hue(&mut self) {
        use crate::layout::shell_hue::WorkspaceObs;
        use crate::layout::LayoutElement as _;

        let now = self.clock.now_unadjusted();
        let anim = self.config.borrow().animations.material_signal.0;
        let obs: Vec<_> = self
            .layout
            .workspaces()
            .map(|(_, _, ws)| WorkspaceObs {
                id: ws.id(),
                active: ws.active_window().map(|w| w.id()),
                windows: ws
                    .windows()
                    .map(|w| (w.id(), w.signal(now).and_then(|s| s.accent)))
                    .collect(),
            })
            .collect();
        self.shell_hue.update(now, anim, &obs);
    }
```

In `update_render_elements`, call `self.update_shell_hue();` right after `self.layout.update_render_elements(output);`. In the per-output loop, before the layer loop:

```rust
                let hue = self
                    .layout
                    .monitor_for_output(out)
                    .map(|mon| self.shell_hue.output_hue(mon.workspace_blend()))
                    .unwrap_or_default();
```

and after `mapped.update_render_elements(geo.size.to_f64());`:

```rust
                    mapped.set_shell_hue(hue);
```

If the borrow checker rejects reading `self.layout` and `self.shell_hue` while `self.output_state` is borrowed mutably, compute a `HashMap<Output, OutputHue>` before the loop, keyed by `out.clone()` from `self.layout.monitors()` (`mon.output()`), and look each output up inside the loop.

- [ ] **Step 5: Arm a redraw for the next settle**

Beside `arm_signal_timer`:

```rust
    /// A window settles after the dwell with nothing else redrawing; wake
    /// the outputs then so the hue arrives on time.
    pub fn arm_shell_hue_timer(&mut self) {
        if let Some(token) = self.shell_hue_timer.take() {
            self.event_loop.remove(token);
        }
        let Some(deadline) = self.shell_hue.next_deadline() else {
            return;
        };
        let delay = deadline.saturating_sub(get_monotonic_time());
        let token = self
            .event_loop
            .insert_source(Timer::from_duration(delay), |_, _, state| {
                state.niri.shell_hue_timer = None;
                state.niri.queue_redraw_all();
                TimeoutAction::Drop
            })
            .unwrap();
        self.shell_hue_timer = Some(token);
    }
```

In the redraw path, after `self.arm_signal_timer(output);` (inside `if res != RenderResult::Skipped`), add:

```rust
                self.arm_shell_hue_timer();
```

- [ ] **Step 6: Run the tests**

Run: `just test-one -p niri tests::shell_hue`
Expected: PASS (9 tests).

If `hue_follows_the_active_agent_after_the_dwell` stays neutral, check whether something between frames calls `Clock::clear` and reverts the clock to real time. If so, set the clock in `frame_at` after any dispatch, as `src/tests/animations.rs` does.

- [ ] **Step 7: Format, run the fast suite, commit**

```bash
rustup run nightly cargo fmt --all
just test-fast
tasks check
git add src/layout/monitor.rs src/niri.rs src/tests/shell_hue.rs
git commit -m "feat(niri): resolve each output's shell hue and hand it to remapped layers"
```

---

### Task 7: Documentation

**Files:**
- Modify: `docs/wiki/Configuration:-Layer-Rules.md` (overview comment lines and a new `surface-remap` section before `popups`)
- Modify: `docs/materials/render-pipeline.md` (section 1: one row and one sentence)
- Modify: `docs/notes/2026-10-14-noctalia-material-brief.md` (the design pointer: implemented, with this plan)

**Interfaces:**
- Consumes: the KDL surface from Task 2 and the rendering placement from Task 5.
- Produces: docs that `niri-config/tests/wiki-parses.rs` parses.

- [ ] **Step 1: Add the overview lines**

In the overview snippet, after the `background-effect { … }` block and before `popups {`, add commented lines. Comments keep the snippet standalone, since `surface-remap` needs a material:

```kdl
    // surface-remap {
    //     surface "#151310"
    //     container "#221f1c"
    //     outline "#4d453c"
    //     material "terminal-glass"
    //     hue-chroma 0.03
    // }
```

- [ ] **Step 2: Add the section**

Before `#### \`popups\``:

````markdown
#### `surface-remap`

<sup>Since: next release</sup>

Recolor a shell's surfaces to a glass material's face, and to the project hue of the output's active workspace.
niri moves every pixel near the surface family (`surface`, `container`, `outline`, the colors the shell draws its backgrounds, cards and outlines with) to the face the named material shows over a neutral backdrop.
Text, icons and accents keep their colors.

- `surface`, `container`, `outline`: the shell's surface family, required. Prism writes these from Noctalia's palette.
- `material`: a material whose glass is dark enough for an opaque surface to match it, required. Its most open channel may pass at most 1% of the backdrop; lighter glass is a config error.
- `hue-chroma`: how strongly the surfaces take the held hue, from `0` (no hue, the default) to `0.1`.

A workspace holds the hue of the last window that stayed its active window for a second while carrying a signal accent; windows without an accent keep that hue, and it clears when the window closes, leaves the workspace, or loses its accent.
Each output shows its active workspace's hue and crossfades over `animations { material-signal }`.

Every `surface-remap` must be complete; a later matching rule replaces the whole node.

```kdl
material "terminal-glass" {
    glass {
        ior 1.28
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 12
    }
}

layer-rule {
    match namespace=r#"^noctalia-(bar-[^"]+|attached-panel|panel)$"#

    surface-remap {
        surface "#151310"
        container "#221f1c"
        outline "#4d453c"
        material "terminal-glass"
        hue-chroma 0.03
    }
}
```
````

- [ ] **Step 3: Run the wiki parse test**

Run: `just test-one -p niri-config wiki_docs_parses`
Expected: PASS.

- [ ] **Step 4: Place the remap in the pipeline doc**

In `docs/materials/render-pipeline.md` section 1, append to item 4 ("Below the tiles"): a new paragraph after the item list:

```markdown
Layer surfaces with a `surface-remap` rule draw through `surface_remap.frag`
instead of the default texture program: their own pixels, before
compositing. It is not a material stage and changes no pass order above;
background effects behind the layer are unchanged.
```

Add one row to the table that follows:

```markdown
| Surface remap element | `surface_remap.frag` (+ `common.frag`) | A layer surface's pixels with its surface family moved to the glass face and the held hue (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md). |
```

- [ ] **Step 5: Update the brief's pointer**

In `docs/notes/2026-10-14-noctalia-material-brief.md`, replace `Design (2026-10-16, draft):` with `Design (2026-10-16, accepted 2026-10-17; plan docs/plans/2026-10-17-noctalia-glass-cohesion.md):`.

- [ ] **Step 6: Commit**

```bash
tasks check
git add docs/wiki/Configuration:-Layer-Rules.md docs/materials/render-pipeline.md docs/notes/2026-10-14-noctalia-material-brief.md
git commit -m "docs: document surface-remap and place it in the render pipeline"
```

---

### Task 8: The nested sheet and transition strip (owner gate)

**Files:**
- Modify: `docs/materials/scripts/noctalia-surfaces.sh`

**Interfaces:**
- Consumes: the worktree's niri build, with `surface-remap` from Tasks 2–6.
- Produces: `noctalia-cohesion-sheet.png` (columns A–D) and `noctalia-hue-transition.png`, attached to `material-62347d`.

This task is a gate: it ends with the owner's verdict, and the controller does not mark it complete without one.

- [ ] **Step 1: Extend the header and add the cases**

In the header comment's `Cases:` list, after `matched`:

```bash
#   remap      the matched look from niri instead of a palette: Noctalia on its
#              wallpaper palette, bar and attached panels 0.95, floating panels
#              solid, and a surface-remap rule moving Noctalia's surface family
#              to the pinned glass's face (no hue).
#   hue-quiet  remap, with the probe terminal carrying the accent $ACCENT and
#              hue-chroma 0.015.
#   hue-strong remap with hue-chroma 0.03.
# STRIP=1 adds a transition strip: two accented probes on two workspaces and a
# slowed workspace switch, sampled across the crossfade.
```

Change the usage line to `# Usage: OUT=<dir> WALL=<image> [NIRI=niri] [CASES="matched remap hue-quiet hue-strong"] [ACCENT=#e5533c] [ACCENT2=#3cc6e5] [STRIP=1] noctalia-surfaces.sh`.

Below `CASES=…`, add:

```bash
ACCENT=${ACCENT:-#e5533c}
ACCENT2=${ACCENT2:-#3cc6e5}
STRIP=${STRIP:-0}
```

and set the default to `CASES=${CASES:-matched remap hue-quiet hue-strong}`.

- [ ] **Step 2: Generate the remap rule from the wallpaper's palette**

Below `GLASS_SURFACE=…`:

```bash
# Noctalia's surface family for WALL, the colors Prism will write live.
family_colors() {
    local tokens="$OUT/family.tokens.json"
    if [ ! -s "$tokens" ]; then
        noctalia theme "$WALL" --scheme m3-content --dark -o "$tokens" \
            > "$OUT/family.theme.log" 2>&1 || fail "noctalia theme failed (see $OUT/family.theme.log)"
    fi
    FAM_SURFACE=$(jq -r '(.dark // .).surface' "$tokens")
    FAM_CONTAINER=$(jq -r '(.dark // .).surface_container' "$tokens")
    FAM_OUTLINE=$(jq -r '(.dark // .).outline_variant' "$tokens")
}
remap_rule() {   # $1 hue-chroma
    family_colors
    cat <<KDL
layer-rule {
    match namespace=r#"^noctalia-(bar-[^"]+|attached-panel|panel)$"#
    surface-remap {
        surface "$FAM_SURFACE"
        container "$FAM_CONTAINER"
        outline "$FAM_OUTLINE"
        material "nsf-glass"
        hue-chroma $1
    }
}
KDL
}
```

`family_colors` must run before `write_niri_config` uses it inside a command substitution. Call it once at the top of `run_case` and `run_strip` (Step 5), so its file exists before any subshell needs it.

- [ ] **Step 3: Teach `write_niri_config` and `write_noctalia_config` the new cases**

In `write_niri_config`, replace the `case` and the `animations` line:

```bash
    local effect remap='' anims='animations { off; }'
    case "$1" in
        baseline) effect='xray false;' ;;
        treated|matched) effect='xray false; noise 0.03; saturation 0.95;' ;;
        remap) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0) ;;
        hue-quiet) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.015) ;;
        hue-strong) effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.03) ;;
        strip)
            effect='xray false; noise 0.03; saturation 0.95;'; remap=$(remap_rule 0.03)
            anims='animations { slowdown 8; }'
            ;;
        *) fail "unknown case $1" ;;
    esac
```

In the heredoc, replace `animations { off; }` with `$anims`, and add `$remap` after the second `layer-rule` block (before `$GLASS`).

In `write_noctalia_config`'s `case`, before `*)`:

```bash
        remap|hue-quiet|hue-strong|strip) bar_opacity=0.95; corner_scale=1.6; panel_mode=solid ;;
```

- [ ] **Step 4: Give hued cases their accent**

Below `has_probe()`:

```bash
probe_ids() { nmsg -j windows | jq -r '[.[] | select(.app_id == "nsf-probe") | .id] | sort | .[]'; }
accent_window() {   # $1 window id, $2 accent
    nmsg set-window-signal --id "$1" --source familiar --accent "$2"
}
```

In `run_case`, add `family_colors` as its first line. After `wait_for 10 has_probe || fail "no probe terminal"`:

```bash
    case $c in
        hue-quiet|hue-strong)
            accent_window "$(probe_ids | head -1)" "$ACCENT"
            ;;
    esac
```

The existing `sleep 3` already covers the 1 s dwell. The case runs with `animations { off; }`, so the hue lands at once.

- [ ] **Step 5: Add the transition strip**

After `run_case`:

```bash
run_strip() {
    local c=strip i
    family_colors
    N=$(mktemp -d "$OUT/$c.noctalia.XXXX")
    write_niri_config "$c" "$OUT/$c.niri.kdl"
    write_noctalia_config "$c" "$N/config"
    start_stack "$c"

    spawn_probe
    wait_for 10 has_probe || fail "no first probe"
    accent_window "$(probe_ids | head -1)" "$ACCENT"
    sleep 6   # dwell, then the slowed fade
    nmsg action focus-workspace-down
    spawn_probe
    wait_for 10 sh -c "[ \$(NIRI_SOCKET='$NIRI_SOCKET' '$NIRI' msg -j windows | jq '[.[] | select(.app_id == \"nsf-probe\")] | length') -ge 2 ]" \
        || fail "no second probe"
    accent_window "$(probe_ids | tail -1)" "$ACCENT2"
    sleep 6
    nmsg action focus-workspace-up
    for i in $(seq 0 11); do shot "$OUT/strip.$i.png"; sleep 0.2; done

    local frames=()
    for i in $(seq 0 11); do
        magick "$OUT/strip.$i.png" -crop 1280x110+0+0 +repage -gravity northwest \
            -fill '#00000099' -draw 'rectangle 0,0 60,34' -fill '#ffffff' \
            -font DejaVu-Sans-Bold -pointsize 22 -annotate +8+5 "F$i" "$OUT/strip.crop.$i.png"
        frames+=("$OUT/strip.crop.$i.png")
    done
    label "Does the bar crossfade from cyan to red without a snap? Frames F0-F11 in order, about 0.3 s apart, of a workspace switch slowed 8x: workspace 2's agent ($ACCENT2) to workspace 1's agent ($ACCENT), hue-chroma 0.03. Top 110 px of the nested output." \
        1280 20 DejaVu-Sans "$OUT/strip.title.png"
    magick "$OUT/strip.title.png" "${frames[@]}" -background '#1b1b1b' -append +repage -depth 8 \
        -define png:compression-level=9 "$OUT/noctalia-hue-transition.png"
    stop_case
    echo "strip: $OUT/noctalia-hue-transition.png"
}
```

`run_strip` uses two helpers. Factor them out of `run_case` instead of duplicating its code: move `run_case`'s lines from Weston's start through `wait_for 30 has_layer noctalia-bar-default …` into `start_stack() { local c=$1; …; }`, and the `nmsg action spawn -- kitty …` line into `spawn_probe()`. Then call both from `run_case`. `run_case` keeps its config writing, validation and recording.

At the bottom, replace the sheet condition and add the strip:

```bash
read -r -a CASE_LIST <<< "$CASES"
for c in "${CASE_LIST[@]}"; do run_case "$c"; done
[ "${#CASE_LIST[@]}" -lt 2 ] || make_sheet
[ "$STRIP" = 0 ] || run_strip
echo "OUT=$OUT"
```

- [ ] **Step 6: Label the remap sheet**

In `make_sheet`, pick the title, the look line and the cell scale by sheet:

```bash
    local title look scale=60
    if [[ " ${CASE_LIST[*]} " == *" remap "* ]]; then
        title="Does the niri remap read as the approved look, and which hue strength reads as the project?"
        look="Column A is the approved look (control): Noctalia's custom glass palette. B is niri's surface remap over Noctalia's own wallpaper palette, with no hue. C and D give the probe terminal the accent $ACCENT and remap the bar and panels toward its hue at two strengths. Nested niri, 1280x720, live wallpaper, cells at 50%."
        scale=50
    else
        title="Do the bar and panels read as the same dark glass as the terminal?"
        look="No column puts a niri material on the bar or panels: niri has none for layer surfaces yet. Every difference is Noctalia styling or niri's blur. Compare each column's bar and panels with the glass terminal on the left. Nested niri, 1280x720, live wallpaper, cells at 60%."
    fi
```

Use `"$title"` and `"$look"` in the two `label` calls, `-resize ${scale}%` in the cell command, and `cw=$((1280 * scale / 100))`. Extend `ids=(A B C D)`. Add notes to `case_note`:

```bash
        remap) echo "remap. No palette change: Noctalia's wallpaper palette, niri's surface-remap to the glass face, hue-chroma 0; bar and attached panels 0.95, floating panels solid. Should read as A." ;;
        hue-quiet) echo "hue, quiet. B, plus the probe's accent $ACCENT held for the workspace: hue-chroma 0.015. Is the hue present but quiet?" ;;
        hue-strong) echo "hue, strong. As C at hue-chroma 0.03. Which of C and D reads as the project, legibly?" ;;
```

- [ ] **Step 7: Build niri from the worktree**

Run, in `.worktrees/material-62347d`:

```bash
cargo build --release -p niri
```

Expected: the build succeeds, and the binary is at `.worktrees/material-62347d/target/release/niri`.

- [ ] **Step 8: Pilot two cases**

```bash
OUT=/mnt/ssd3/niri-material/material-62347d/pilot-$(date +%s) \
WALL=$(noctalia msg wallpaper-get) \
NIRI=$PWD/target/release/niri CASES="matched remap" \
bash docs/materials/scripts/noctalia-surfaces.sh
grep -l "error compiling surface remap shader" "$OUT"/*.niri.log && echo SHADER-FAILED
```

Expected:
- both cases finish and a two-column sheet is written;
- no `SHADER-FAILED`;
- in the sheet, column B's control center and launcher read as near-black glass like A, not as the warm baseline surface.

Sample one pixel from each to confirm. Both should be within a few levels of 10,10,10:

```bash
magick "$OUT/remap.control-center.png" -format '%[pixel:p{640,360}]\n' info:
magick "$OUT/matched.control-center.png" -format '%[pixel:p{640,360}]\n' info:
```

If B still shows the warm baseline, the remap did not apply. Check `niri msg layers` in `$OUT/remap.control-center.layers.json` for the namespace, and the nested niri log for config errors, before going further.

Record the pilot as a task note: `tasks note material-62347d "run: pilot matched+remap — <result>"`.

- [ ] **Step 9: Run the full sheet and the strip**

```bash
OUT=/mnt/ssd3/niri-material/material-62347d/sheet-$(date +%s) \
WALL=$(noctalia msg wallpaper-get) \
NIRI=$PWD/target/release/niri STRIP=1 \
bash docs/materials/scripts/noctalia-surfaces.sh
```

Expected: `noctalia-cohesion-sheet.png` with four columns and three rows, plus `noctalia-hue-transition.png`.

- [ ] **Step 10: Check the sheets at the owner's scale, then attach them**

View both images with the Read tool at full size. Check each against the global sheet rule:
- the title states the question;
- each column is labelled in plain language;
- A is marked as the control;
- cell ids are legible.

Then check sizes against the 2 MiB attachment cap:

```bash
stat -c %s "$OUT/noctalia-cohesion-sheet.png" "$OUT/noctalia-hue-transition.png"
```

If a file is over 2,097,152 bytes, rerun `make_sheet` with a smaller cell scale (45) rather than lowering the PNG depth. Attach both:

```bash
tasks attach material-62347d "$OUT/noctalia-cohesion-sheet.png"
tasks attach material-62347d "$OUT/noctalia-hue-transition.png"
```

- [ ] **Step 11: Commit the script and the attachments**

```bash
tasks check
git add docs/materials/scripts/noctalia-surfaces.sh tasks/
git commit -m "docs(materials): add remap and hue cases to the Noctalia sheet"
```

- [ ] **Step 12: Owner gate**

Ask the owner one question with the two attachment paths. The question names:
- the cells to compare: A1–A3 against B1–B3, the remap against the approved look;
- the hue strengths: C against D;
- the strip.

Record the verdict as a task note (`owner verdict <date>: …`), including the chosen `hue-chroma`. If B does not read as A, stop: the verdict sends the work back to the spec, not to retuning constants in place.

---

### Task 9: Hand-off: Prism task, rollout task, close-out

**Files:**
- Modify: `tasks/` records only (this repository and Prism's)

**Interfaces:**
- Consumes: the owner's verdict and the chosen `hue-chroma` (Task 8).
- Produces: a Prism task for spec §6, a material rollout task, and `material-62347d` closed.

- [ ] **Step 1: File the Prism task in Prism's checkout**

Find Prism's path with `tasks projects`, then run `tasks add` from that checkout:

```bash
cd "<prism checkout from tasks projects>"
tasks add "Emit niri's surface-remap rule for the Noctalia bar and panels" \
  --body "Spec: niri-material docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §6. Template: add surface_container and outline_variant to integrations/niri/noctalia-palette.template. Sink: emit one complete surface-remap node while glass.enabled and a new shell.remap (default on) are both on, whatever the tint source: the three family colors, the active glass material's name, hue-chroma from a new shell.hueChroma (default <owner's pick from material-62347d>). Omit it with 'shell remap off: Noctalia is in light mode' when the resolved surface is lighter than Oklab L 0.4. A missing palette fails the apply only when the rule is to be emitted. Tests: spec Verification, Prism tests (glass on, glass off with no palette requirement, remap off, light mode, niri validate on each fragment)."
```

Record the returned id.

- [ ] **Step 2: File the rollout task here**

In this repository:

```bash
tasks add "Roll out Noctalia glass cohesion on the live desktop" --parent material-55380c \
  --body "Waits on <prism id>. Host-use, authorized at the time: install niri with surface-remap, apply Prism, then the owner sets Noctalia's settings layer: [bar.default] background_opacity 0.95, [shell.panel] transparency_mode solid, [shell] corner_radius_scale 1.6. 30-second check: switch between two agent workspaces with the control center open. Spec: docs/specs/2026-10-16-noctalia-glass-cohesion-design.md, Verification, Live rollout."
tasks note <new id> "concerns: material-62347d extension — live rollout of the reviewed niri side"
```

- [ ] **Step 3: Close `material-62347d` in the final commit**

```bash
tasks note material-62347d "done: niri side implemented and judged (sheet verdict above); Prism side <prism id>; rollout <new id>"
tasks done material-62347d
tasks check
git add tasks/
git commit -m "chore(tasks): close material-62347d and file the Prism and rollout follow-ups"
```
