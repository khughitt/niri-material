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
    let t = transmittance(glass);
    t[0].max(t[1]).max(t[2]) * (1. - fresnel_f0(glass.ior))
}

/// The face over `backdrop` (linear), linear.
pub fn face_linear(glass: &ResolvedGlass, backdrop: [f64; 3]) -> [f64; 3] {
    let t = transmittance(glass);
    let f0 = fresnel_f0(glass.ior);
    [0, 1, 2].map(|i| backdrop[i] * t[i] * (1. - f0) + SPECULAR_FLOOR * f0)
}

/// The reference swatch: the face over `B_REF`, sRGB-encoded in 0..1.
pub fn face_srgb(glass: &ResolvedGlass) -> [f64; 3] {
    face_linear(glass, [B_REF; 3]).map(linear_to_srgb)
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

/// The model covers transmitted light and the specular floor only. Aurora
/// lights the whole face and iridescence recolors the floor, so a material
/// with either cannot be matched (spec §2, supported effects).
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
        let err = check_range("blue", &blue).unwrap_err();
        assert!(
            err.starts_with("surface-remap: material \"blue\" transmits "),
            "{err}"
        );
        assert!(
            err.ends_with(
                "in its most open channel; the shell match supports dark glass up to 0.01"
            ),
            "{err}"
        );
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
        assert!(
            (lo + 0.046).abs() < 0.001 && (hi - 0.044).abs() < 0.001,
            "{lo} {hi}"
        );
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
        assert!(
            err.contains("sets iridescence, which lights the flat face"),
            "{err}"
        );
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
        assert!(
            accepted > 1000,
            "the sweep must reach many accepted configs: {accepted}"
        );
    }
}
