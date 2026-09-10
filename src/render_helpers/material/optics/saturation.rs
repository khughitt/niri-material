//! `saturation`: stage 9, neutral at 1. Applies the inherit-or-neutral rule.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct SaturationOptic;

impl Optic for SaturationOptic {
    const NAME: &'static str = "saturation";
    const GLSL: &'static str = include_str!("../../shaders/material/saturation.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] =
        &[("mat_saturation", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let amount = glass.saturation.amount.unwrap_or(if ctx.backdrop_blur {
            ctx.blur.saturation
        } else {
            1.
        });
        vec![Uniform::new("mat_saturation", amount as f32)]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedGlass, ResolvedSaturation};
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

    fn amount(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> f32 {
        let values = SaturationOptic::values(glass, &frame(backdrop_blur, blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_saturation");
        match values[0].value {
            UniformValue::_1f(v) => v,
            ref other => panic!("{other:?}"),
        }
    }

    #[test]
    fn written_saturation_applies_regardless_of_blur() {
        let glass = ResolvedGlass {
            saturation: ResolvedSaturation { amount: Some(0.5) },
            ..Default::default()
        };
        let blur = Blur {
            saturation: 1.5,
            ..Default::default()
        };
        assert_eq!(amount(&glass, true, &blur), 0.5);
        assert_eq!(amount(&glass, false, &blur), 0.5);
    }

    #[test]
    fn omitted_saturation_inherits_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            saturation: 1.5,
            ..Default::default()
        };
        assert_eq!(amount(&glass, true, &blur), 1.5);
        assert_eq!(amount(&glass, false, &blur), 1.);
    }

    #[test]
    fn saturation_is_static() {
        let blur = Blur::default();
        assert_eq!(
            SaturationOptic::next_change(&ResolvedGlass::default(), &frame(true, &blur)),
            None
        );
    }
}
