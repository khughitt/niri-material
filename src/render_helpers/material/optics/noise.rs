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

    /// Correlation of two fine-lattice values `d` apart, from the high-pass
    /// definition `h(p) - mean(h at p's eight neighbours)`, by enumerating
    /// each value's hash coefficients (design §4).
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
    fn the_fine_norm_coefficients_are_the_correlations_times_their_offset_counts() {
        let expected = [
            ((1, 0), -1. / 6.),
            ((1, 1), -7. / 36.),
            ((2, 0), 1. / 24.),
            ((2, 1), 1. / 36.),
            ((2, 2), 1. / 72.),
        ];
        for (d, want) in expected {
            let got = fine_corner_correlation(d);
            assert!((got - want).abs() < 1e-12, "{d:?}: {got}");
        }
        for d in [(3, 0), (3, 1), (3, 2), (3, 3)] {
            assert_eq!(fine_corner_correlation(d), 0., "{d:?}");
        }
        // Offsets (1, 0) and (2, 0) occur along each axis in both directions
        // (two each); (1, 1), (2, 1) and (2, 2) in all four sign combinations.
        let c = |d| fine_corner_correlation(d);
        let (adjacent, diagonal) = (-2. * c((1, 0)), -4. * c((1, 1)));
        let (two, knight, far) = (2. * c((2, 0)), 4. * c((2, 1)), 4. * c((2, 2)));
        // The comments name the coefficients too, so only code counts, and
        // the whole norm must be there, not the bare literals.
        let code = NoiseOptic::GLSL
            .lines()
            .map(|line| line.split("//").next().unwrap())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let expected = format!(
            "norm = x0 * y0 - {adjacent:.8} * (x1 * y0 + x0 * y1) - {diagonal:.8} * x1 * y1 \
             + {two:.8} * (x2 * y0 + x0 * y2) + {knight:.8} * (x2 * y1 + x1 * y2) \
             + {far:.8} * x2 * y2;"
        );
        assert!(
            code.contains(&expected),
            "noise.frag's fine branch must carry the value covariance: `{expected}`"
        );
        for sums in [
            "float x1 = bx.x * bx.y + bx.y * bx.z + bx.z * bx.w;",
            "float x2 = bx.x * bx.z + bx.y * bx.w;",
        ] {
            assert!(code.contains(sums), "noise.frag must compute `{sums}`");
        }

        // Those sums with those coefficients are w^T C w over the sixteen
        // values, w the B-spline weights' outer product.
        let bspline = |t: f64| {
            let s = 1. - t;
            [
                s * s * s / 6.,
                (3. * t * t * t - 6. * t * t + 4.) / 6.,
                (-3. * t * t * t + 3. * t * t + 3. * t + 1.) / 6.,
                t * t * t / 6.,
            ]
        };
        let lag = |b: [f64; 4], k: usize| (0..4 - k).map(|j| b[j] * b[j + k]).sum::<f64>();
        for (tx, ty) in [(0., 0.), (0.5, 0.5), (0.2, 0.7), (0.9, 0.35)] {
            let (bx, by) = (bspline(tx), bspline(ty));
            let (x0, x1, x2) = (lag(bx, 0), lag(bx, 1), lag(bx, 2));
            let (y0, y1, y2) = (lag(by, 0), lag(by, 1), lag(by, 2));
            let grouped = x0 * y0 - adjacent * (x1 * y0 + x0 * y1) - diagonal * x1 * y1
                + two * (x2 * y0 + x0 * y2)
                + knight * (x2 * y1 + x1 * y2)
                + far * x2 * y2;
            let mut brute = 0.;
            for (a, b) in (0..16).flat_map(|a| (0..16).map(move |b| (a, b))) {
                let d = (
                    (b % 4) as i32 - (a % 4) as i32,
                    (b / 4) as i32 - (a / 4) as i32,
                );
                brute += bx[a % 4] * by[a / 4] * bx[b % 4] * by[b / 4] * fine_corner_correlation(d);
            }
            assert!(
                (grouped - brute).abs() < 1e-12,
                "t ({tx}, {ty}): {grouped} vs {brute}"
            );
        }
    }
}
