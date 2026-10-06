//! `noise`: up to four layers, each at its selected behind/source/post
//! placement, neutral at amount 0. Only slot 0's amount inherits, and only
//! when no node is written; type, site and scale never do.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_4f),
        ("mat_noise_type", UniformType::_4f),
        ("mat_noise_site", UniformType::_4f),
        ("mat_noise_scale", UniformType::_4f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let mut amount = [0f32; 4];
        let mut kind = [0f32; 4];
        let mut site = [0f32; 4];
        let mut scale = [1f32; 4];
        if glass.noise.is_omitted() {
            amount[0] = if ctx.backdrop_blur {
                ctx.blur.noise
            } else {
                0.
            } as f32;
        }
        for (k, layer) in glass.noise.layers.iter().enumerate() {
            let Some(layer) = layer else { continue };
            amount[k] = layer.amount as f32;
            kind[k] = layer.kind as u8 as f32;
            site[k] = layer.site as u8 as f32;
            scale[k] = layer.scale as f32;
        }
        vec![
            Uniform::new("mat_noise", amount),
            Uniform::new("mat_noise_type", kind),
            Uniform::new("mat_noise_site", site),
            Uniform::new("mat_noise_scale", scale),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use niri_config::{
        Blur, NoiseSite, NoiseType, ResolvedGlass, ResolvedNoise, ResolvedNoiseLayer,
    };
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::material::optics::OpticFrame;

    fn frame(backdrop_blur: bool, blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            logical_now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur,
            blur,
            seed: 0.,
        }
    }

    /// The four uniform vectors: amount, type, site, scale.
    fn vectors(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> [[f32; 4]; 4] {
        let values = NoiseOptic::values(glass, &frame(backdrop_blur, blur));
        let names: Vec<_> = values.iter().map(|u| u.name.clone()).collect();
        assert_eq!(
            names,
            [
                "mat_noise",
                "mat_noise_type",
                "mat_noise_site",
                "mat_noise_scale"
            ]
        );
        let v = |u: &UniformValue| match *u {
            UniformValue::_4f(a, b, c, d) => [a, b, c, d],
            ref other => panic!("{other:?}"),
        };
        [
            v(&values[0].value),
            v(&values[1].value),
            v(&values[2].value),
            v(&values[3].value),
        ]
    }

    fn layer(
        amount: f64,
        kind: NoiseType,
        site: NoiseSite,
        scale: f64,
    ) -> Option<ResolvedNoiseLayer> {
        Some(ResolvedNoiseLayer {
            amount,
            kind,
            site,
            scale,
        })
    }

    #[test]
    fn written_layers_pack_in_config_order() {
        let glass = ResolvedGlass {
            noise: ResolvedNoise {
                layers: [
                    layer(0.04, NoiseType::Lightness, NoiseSite::Glass, 1.),
                    layer(0.03, NoiseType::White, NoiseSite::Glass, 4.),
                    layer(0.02, NoiseType::Fine, NoiseSite::Film, 2.5),
                    None,
                ],
            },
            ..Default::default()
        };
        assert_eq!(
            vectors(&glass, true, &Blur::default()),
            [
                [0.04, 0.03, 0.02, 0.],
                [2., 0., 1., 0.],
                [0., 0., 2., 0.],
                [1., 4., 2.5, 1.],
            ]
        );
    }

    #[test]
    fn omitted_noise_inherits_into_slot_zero_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            noise: 0.02,
            ..Default::default()
        };
        let on = vectors(&glass, true, &blur);
        assert_eq!(on[0], [0.02, 0., 0., 0.]);
        assert_eq!(on[1], [0.; 4]);
        assert_eq!(on[2], [0.; 4]);
        assert_eq!(on[3], [1.; 4]);
        assert_eq!(vectors(&glass, false, &blur)[0], [0.; 4]);
    }

    #[test]
    fn a_written_layer_stops_inheritance_at_every_site() {
        let blur = Blur {
            noise: 0.05,
            ..Default::default()
        };
        for site in [NoiseSite::Glass, NoiseSite::Backdrop, NoiseSite::Film] {
            let glass = ResolvedGlass {
                noise: ResolvedNoise::single(0.2, NoiseType::White, site),
                ..Default::default()
            };
            let v = vectors(&glass, true, &blur);
            assert_eq!(v[0], [0.2, 0., 0., 0.], "{site:?}");
            assert_eq!(v[2], [site as u8 as f32, 0., 0., 0.], "{site:?}");
        }
    }

    /// Correlation of two fine-lattice corners `d` apart, from the high-pass
    /// definition `h(p) - mean(h at p's eight neighbours)`, by enumerating
    /// each corner's hash coefficients (design §4).
    fn fine_corner_correlation(d: (i32, i32)) -> f64 {
        let coefficients = |c: (i32, i32)| {
            let mut m = HashMap::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let w = if (dx, dy) == (0, 0) { 1. } else { -1. / 8. };
                    *m.entry((c.0 + dx, c.1 + dy)).or_insert(0.) += w;
                }
            }
            m
        };
        let cov = |a: (i32, i32), b: (i32, i32)| {
            let (a, b) = (coefficients(a), coefficients(b));
            a.iter()
                .map(|(k, v)| v * b.get(k).copied().unwrap_or(0.))
                .sum::<f64>()
        };
        cov((0, 0), d) / cov((0, 0), (0, 0))
    }

    #[test]
    fn the_fine_norm_coefficients_are_twice_the_corner_correlations() {
        let adjacent = fine_corner_correlation((1, 0));
        let diagonal = fine_corner_correlation((1, 1));
        assert!((adjacent + 1. / 6.).abs() < 1e-12, "{adjacent}");
        assert!((diagonal + 7. / 36.).abs() < 1e-12, "{diagonal}");
        // The comments name the coefficients too, so only code counts, and
        // the whole subtraction must be there, not the bare literals.
        let code = NoiseOptic::GLSL
            .lines()
            .map(|line| line.split("//").next().unwrap())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let expected = format!(
            "norm -= {:.8} * (w00 * w10 + w00 * w01 + w10 * w11 + w01 * w11) \
             + {:.8} * (w00 * w11 + w10 * w01);",
            -2. * adjacent,
            -2. * diagonal
        );
        assert!(
            code.contains(&expected),
            "noise.frag's fine branch must subtract the corner covariance: `{expected}`"
        );
    }
}
