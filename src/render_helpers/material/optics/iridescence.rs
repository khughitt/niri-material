//! `iridescence`: stage 5, neutral at amount 0. Static: the amount is its
//! only uniform.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct IridescenceOptic;

impl Optic for IridescenceOptic {
    const NAME: &'static str = "iridescence";
    const GLSL: &'static str = include_str!("../../shaders/material/iridescence.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] =
        &[("mat_iridescence", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![Uniform::new(
            "mat_iridescence",
            glass.iridescence.amount as f32,
        )]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedGlass, ResolvedIridescence};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    fn frame(blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur: false,
            blur,
            seed: 0.,
        }
    }

    #[test]
    fn the_amount_is_the_only_uniform_and_the_optic_is_static() {
        let glass = ResolvedGlass {
            iridescence: ResolvedIridescence { amount: 0.8 },
            ..Default::default()
        };
        let blur = Blur::default();
        let values = IridescenceOptic::values(&glass, &frame(&blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_iridescence");
        assert_eq!(values[0].value, UniformValue::_1f(0.8));
        assert_eq!(IridescenceOptic::next_change(&glass, &frame(&blur)), None);
    }
}
