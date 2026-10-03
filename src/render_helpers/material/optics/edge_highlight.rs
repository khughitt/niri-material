//! `edge-highlight`: stage 6, neutral at amount 0. Static: its gain and the
//! roughness-derived lobe width are its uniforms.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};
use crate::render_helpers::material::bevel::highlight_alpha;

pub struct EdgeHighlightOptic;

impl Optic for EdgeHighlightOptic {
    const NAME: &'static str = "edge-highlight";
    const GLSL: &'static str = include_str!("../../shaders/material/edge_highlight.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_edge_highlight", UniformType::_1f),
        ("mat_edge_highlight_alpha", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![
            Uniform::new("mat_edge_highlight", glass.edge_highlight.amount as f32),
            Uniform::new(
                "mat_edge_highlight_alpha",
                highlight_alpha(glass.roughness) as f32,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedEdgeHighlight, ResolvedGlass};
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
    fn uniforms_are_the_gain_and_the_roughness_alpha() {
        let glass = ResolvedGlass {
            edge_highlight: ResolvedEdgeHighlight { amount: 0.5 },
            roughness: 1.,
            ..Default::default()
        };
        let blur = Blur::default();
        let values = EdgeHighlightOptic::values(&glass, &frame(&blur));
        assert_eq!(values[0].name, "mat_edge_highlight");
        assert_eq!(values[0].value, UniformValue::_1f(0.5));
        assert_eq!(values[1].name, "mat_edge_highlight_alpha");
        assert_eq!(values[1].value, UniformValue::_1f(0.5));
        assert_eq!(EdgeHighlightOptic::next_change(&glass, &frame(&blur)), None);
    }
}
