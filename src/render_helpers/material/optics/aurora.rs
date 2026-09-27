//! `aurora`: within stage, neutral at amount 0. Animated: the field's phase
//! steps at `drift-hz` buckets per second on the generalised drift clock
//! over a 600 s period, so its `next_change` is the next bucket boundary.

use std::time::Duration;

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};
use crate::render_helpers::signal::{color_linear, drift_rate, next_boundary_on, phase_on};

/// One lap of the field's loop. Long enough that the repeat is never
/// noticed; `drift-hz` steps per second give `hz × 600` buckets.
pub const AURORA_PERIOD: Duration = Duration::from_secs(600);

pub struct AuroraOptic;

impl AuroraOptic {
    /// Effective bucket rate: 0 while the field is unlit (the neutral must
    /// fingerprint to a constant and schedule nothing), else the configured
    /// rate under the motion policy and the animation switch.
    fn rate(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> f64 {
        if glass.aurora.amount <= 0. {
            return 0.;
        }
        drift_rate(glass.aurora.drift_hz, ctx.motion, ctx.animations_off)
    }
}

impl Optic for AuroraOptic {
    const NAME: &'static str = "aurora";
    const GLSL: &'static str = include_str!("../../shaders/material/aurora.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_aurora", UniformType::_1f),
        ("mat_aurora_phase", UniformType::_1f),
        ("mat_aurora_color_a", UniformType::_3f),
        ("mat_aurora_color_b", UniformType::_3f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let aurora = &glass.aurora;
        vec![
            Uniform::new("mat_aurora", aurora.amount as f32),
            Uniform::new(
                "mat_aurora_phase",
                phase_on(AURORA_PERIOD, Self::rate(glass, ctx), ctx.now, ctx.seed),
            ),
            Uniform::new("mat_aurora_color_a", color_linear(aurora.color_a)),
            Uniform::new("mat_aurora_color_b", color_linear(aurora.color_b)),
        ]
    }

    fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration> {
        next_boundary_on(AURORA_PERIOD, Self::rate(glass, ctx), ctx.now)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::signal::SignalMotionPolicy;
    use niri_config::{Blur, Color, ResolvedAurora, ResolvedGlass};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::signal::phase_on;

    fn frame(
        blur: &Blur,
        now: Duration,
        motion: SignalMotionPolicy,
        animations_off: bool,
    ) -> OpticFrame<'_> {
        OpticFrame {
            now,
            motion,
            animations_off,
            backdrop_blur: false,
            blur,
            seed: 0.,
        }
    }

    fn sky(amount: f64, drift_hz: f64) -> ResolvedGlass {
        ResolvedGlass {
            aurora: ResolvedAurora {
                amount,
                drift_hz,
                color_a: Color::from_rgba8_unpremul(0xff, 0, 0, 0xff),
                color_b: Color::from_rgba8_unpremul(0, 0, 0xff, 0xff),
            },
            ..Default::default()
        }
    }

    fn f1(v: &UniformValue) -> f32 {
        match v {
            UniformValue::_1f(v) => *v,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn values_carry_amount_phase_and_linear_colours() {
        let blur = Blur::default();
        let ctx = frame(
            &blur,
            Duration::from_millis(250),
            SignalMotionPolicy::Full,
            false,
        );
        let values = AuroraOptic::values(&sky(0.5, 4.), &ctx);
        let names: Vec<_> = values.iter().map(|u| u.name.as_ref()).collect();
        assert_eq!(
            names,
            [
                "mat_aurora",
                "mat_aurora_phase",
                "mat_aurora_color_a",
                "mat_aurora_color_b"
            ]
        );
        assert_eq!(f1(&values[0].value), 0.5);
        assert_eq!(
            f1(&values[1].value),
            phase_on(AURORA_PERIOD, 4., Duration::from_millis(250), 0.)
        );
        assert_eq!(values[2].value, UniformValue::_3f(1., 0., 0.));
        assert_eq!(values[3].value, UniformValue::_3f(0., 0., 1.));
    }

    #[test]
    fn the_phase_follows_the_motion_policy_and_the_animation_switch() {
        let blur = Blur::default();
        let now = Duration::from_millis(250);
        let full = AuroraOptic::values(
            &sky(0.5, 4.),
            &frame(&blur, now, SignalMotionPolicy::Full, false),
        );
        let reduced = AuroraOptic::values(
            &sky(0.5, 4.),
            &frame(&blur, now, SignalMotionPolicy::Reduced, false),
        );
        let off = AuroraOptic::values(
            &sky(0.5, 4.),
            &frame(&blur, now, SignalMotionPolicy::Off, false),
        );
        let anim_off = AuroraOptic::values(
            &sky(0.5, 4.),
            &frame(&blur, now, SignalMotionPolicy::Full, true),
        );
        assert_eq!(f1(&full[1].value), phase_on(AURORA_PERIOD, 4., now, 0.));
        assert_eq!(f1(&reduced[1].value), phase_on(AURORA_PERIOD, 2., now, 0.));
        assert_eq!(f1(&off[1].value), 0.);
        assert_eq!(f1(&anim_off[1].value), 0.);
    }

    #[test]
    fn next_change_is_the_next_bucket_boundary_only_while_the_field_is_lit_and_moving() {
        let blur = Blur::default();
        let now = Duration::ZERO;
        let full = frame(&blur, now, SignalMotionPolicy::Full, false);
        assert_eq!(
            AuroraOptic::next_change(&sky(0.5, 4.), &full),
            Some(Duration::from_millis(250))
        );
        assert_eq!(
            AuroraOptic::next_change(
                &sky(0.5, 4.),
                &frame(&blur, now, SignalMotionPolicy::Reduced, false)
            ),
            Some(Duration::from_millis(500))
        );
        assert_eq!(
            AuroraOptic::next_change(
                &sky(0.5, 4.),
                &frame(&blur, now, SignalMotionPolicy::Off, false)
            ),
            None
        );
        assert_eq!(
            AuroraOptic::next_change(
                &sky(0.5, 4.),
                &frame(&blur, now, SignalMotionPolicy::Full, true)
            ),
            None
        );
        assert_eq!(AuroraOptic::next_change(&sky(0.5, 0.), &full), None);
        // Amount 0 is the neutral: no redraw, and a pinned phase so the
        // fingerprint stays constant.
        assert_eq!(AuroraOptic::next_change(&sky(0., 4.), &full), None);
        let neutral = AuroraOptic::values(
            &sky(0., 4.),
            &frame(
                &blur,
                Duration::from_millis(250),
                SignalMotionPolicy::Full,
                false,
            ),
        );
        assert_eq!(f1(&neutral[1].value), 0.);
    }
}
