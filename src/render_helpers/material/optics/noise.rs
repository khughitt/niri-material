//! `noise`: behind, stage 3b, neutral at amount 0. Applies the inherit-or-neutral
//! rule to the amount; the grain type never inherits.

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
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let amount = glass.noise.amount.unwrap_or(if ctx.backdrop_blur {
            ctx.blur.noise
        } else {
            0.
        });
        vec![
            Uniform::new("mat_noise", amount as f32),
            Uniform::new("mat_noise_type", glass.noise.kind as u8 as f32),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, NoiseType, ResolvedGlass, ResolvedNoise};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::material::optics::OpticFrame;

    fn frame(backdrop_blur: bool, blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur,
            blur,
            seed: 0.,
        }
    }

    fn pair(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> (f32, f32) {
        let values = NoiseOptic::values(glass, &frame(backdrop_blur, blur));
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].name, "mat_noise");
        assert_eq!(values[1].name, "mat_noise_type");
        let f = |v: &UniformValue| match v {
            UniformValue::_1f(v) => *v,
            other => panic!("{other:?}"),
        };
        (f(&values[0].value), f(&values[1].value))
    }

    #[test]
    fn written_noise_applies_regardless_of_blur_and_carries_its_type() {
        let glass = ResolvedGlass {
            noise: ResolvedNoise {
                amount: Some(0.3),
                kind: NoiseType::Lightness,
            },
            ..Default::default()
        };
        let blur = Blur {
            noise: 0.02,
            off: true,
            ..Default::default()
        };
        assert_eq!(pair(&glass, true, &blur), (0.3, 2.));
        assert_eq!(pair(&glass, false, &blur), (0.3, 2.));
    }

    #[test]
    fn omitted_noise_inherits_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            noise: 0.02,
            ..Default::default()
        };
        assert_eq!(pair(&glass, true, &blur), (0.02, 0.));
        assert_eq!(pair(&glass, false, &blur), (0., 0.));
    }
}
