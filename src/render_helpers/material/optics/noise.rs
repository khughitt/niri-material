//! `noise`: selected behind/source/post placement, neutral at amount 0.
//! Only the amount inherits; the grain type and site never do.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_1f),
        ("mat_noise_type", UniformType::_1f),
        ("mat_noise_site", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        // Interim (plan Task 1): slot 0 only; Task 2 uploads every slot.
        let first = glass.noise.layers[0];
        let inherited = if ctx.backdrop_blur {
            ctx.blur.noise
        } else {
            0.
        };
        vec![
            Uniform::new("mat_noise", first.map_or(inherited, |l| l.amount) as f32),
            Uniform::new("mat_noise_type", first.map_or(0., |l| l.kind as u8 as f32)),
            Uniform::new("mat_noise_site", first.map_or(0., |l| l.site as u8 as f32)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, NoiseSite, NoiseType, ResolvedGlass, ResolvedNoise};
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

    fn triple(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> (f32, f32, f32) {
        let values = NoiseOptic::values(glass, &frame(backdrop_blur, blur));
        assert_eq!(values.len(), 3);
        assert_eq!(values[2].name, "mat_noise_site");
        assert_eq!(values[0].name, "mat_noise");
        assert_eq!(values[1].name, "mat_noise_type");
        let f = |v: &UniformValue| match v {
            UniformValue::_1f(v) => *v,
            other => panic!("{other:?}"),
        };
        (
            f(&values[0].value),
            f(&values[1].value),
            f(&values[2].value),
        )
    }

    #[test]
    fn written_noise_applies_regardless_of_blur_and_carries_its_type() {
        let glass = ResolvedGlass {
            noise: ResolvedNoise::single(0.3, NoiseType::Lightness, NoiseSite::Glass),
            ..Default::default()
        };
        let blur = Blur {
            noise: 0.02,
            off: true,
            ..Default::default()
        };
        assert_eq!(triple(&glass, true, &blur), (0.3, 2., 0.));
        assert_eq!(triple(&glass, false, &blur), (0.3, 2., 0.));
    }

    #[test]
    fn omitted_noise_inherits_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            noise: 0.02,
            ..Default::default()
        };
        assert_eq!(triple(&glass, true, &blur), (0.02, 0., 0.));
        assert_eq!(triple(&glass, false, &blur), (0., 0., 0.));
    }
    #[test]
    fn the_site_rides_the_third_uniform_and_leaves_amount_and_type_alone() {
        let blur = Blur::default();
        for (site, code) in [
            (NoiseSite::Glass, 0.),
            (NoiseSite::Backdrop, 1.),
            (NoiseSite::Film, 2.),
        ] {
            let glass = ResolvedGlass {
                noise: ResolvedNoise::single(0.3, NoiseType::Fine, site),
                ..Default::default()
            };
            assert_eq!(triple(&glass, true, &blur), (0.3, 1., code), "{site:?}");
        }
    }
}
