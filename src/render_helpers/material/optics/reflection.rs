//! `reflection`: stage 6, neutral at amount 0. Static: the amount is its only
//! uniform.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct ReflectionOptic;

impl Optic for ReflectionOptic {
    const NAME: &'static str = "reflection";
    const GLSL: &'static str = include_str!("../../shaders/material/reflection.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] =
        &[("mat_reflection", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![Uniform::new(
            "mat_reflection",
            glass.reflection.amount as f32,
        )]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedGlass, ResolvedReflection};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    fn frame(blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            logical_now: Duration::ZERO,
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
            reflection: ResolvedReflection { amount: 0.6 },
            ..Default::default()
        };
        let blur = Blur::default();
        let values = ReflectionOptic::values(&glass, &frame(&blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_reflection");
        assert_eq!(values[0].value, UniformValue::_1f(0.6));
        assert_eq!(ReflectionOptic::next_change(&glass, &frame(&blur)), None);
    }

    #[test]
    fn reflection_guards_a_cancelled_direction() {
        // Review Focus 3: a perturbation can cancel the across-bevel direction.
        let glsl = ReflectionOptic::GLSL;
        assert!(glsl.contains("vec2 dir = lb > 0.0001 ? bent / lb : s.acrossDir;"));
        assert!(glsl.contains("if (mat_reflection <= 0.0)\n        return specular;"));
    }
}
