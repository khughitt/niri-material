//! Optics: the self-contained stages of the glass pipeline, renderer side.
//!
//! `OPTICS` is the one ordered registry. The shader is assembled from each
//! entry's `glsl` in this order, the program's uniform list is extended with
//! each entry's `uniforms`, each frame's uniform values come from `values`,
//! and the redraw deadline takes the minimum of every `next_change`.
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §3.

use std::time::Duration;

use niri_config::signal::SignalMotionPolicy;
use niri_config::{Blur, ResolvedGlass};
use smithay::backend::renderer::gles::{Uniform, UniformName, UniformType};

use crate::activity::OpticTime;

pub mod aurora;
pub mod edge_highlight;
pub mod iridescence;
pub mod noise;
pub mod reflection;
pub mod saturation;

/// One pipeline stage. Implemented on a marker type per optic; the resolved
/// configuration arrives as `ResolvedGlass`, which owns every optic's state.
pub trait Optic {
    /// The registry name; matches `niri_config::material::optics::ORDER`. A
    /// hyphenated name keeps its hyphen here, in `ORDER` and in the KDL node, and
    /// becomes underscores in file, module, hook and uniform names.
    const NAME: &'static str;
    /// The optic's GLSL: its `uniform` declarations and `<NAME>_<hook>`
    /// functions.
    const GLSL: &'static str;
    /// The uniforms the GLSL declares, in declaration order.
    const UNIFORMS: &'static [(&'static str, UniformType)];
    /// This frame's uniforms, one per entry of `UNIFORMS`, in the same order.
    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>>;
    /// The next instant `values` changes with no config change, as a logical
    /// instant on the shared optic timeline (`ctx.logical_now`'s domain), not
    /// a real scheduler time: the module-level `next_change` maps it. `None` for a
    /// static optic.
    fn next_change(_glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Option<Duration> {
        None
    }
}

/// What an optic may depend on beyond its own configuration.
#[derive(Debug, Clone, Copy)]
pub struct OpticFrame<'a> {
    /// Shared optic timeline sample.
    pub logical_now: Duration,
    pub motion: SignalMotionPolicy,
    pub animations_off: bool,
    /// Effective: configured and the global `blur` block is not off.
    pub backdrop_blur: bool,
    /// The global block, for the inherit-or-neutral rule.
    pub blur: &'a Blur,
    /// The window's jelly seed, component 0: per-window variation.
    pub seed: f32,
}

pub struct OpticEntry {
    pub name: &'static str,
    pub glsl: &'static str,
    pub uniforms: &'static [(&'static str, UniformType)],
    pub values: fn(&ResolvedGlass, &OpticFrame<'_>) -> Vec<Uniform<'static>>,
    pub next_change: fn(&ResolvedGlass, &OpticFrame<'_>) -> Option<Duration>,
}

impl OpticEntry {
    pub const fn of<O: Optic>() -> Self {
        Self {
            name: O::NAME,
            glsl: O::GLSL,
            uniforms: O::UNIFORMS,
            values: O::values,
            next_change: O::next_change,
        }
    }
}

/// The optics in render order.
pub static OPTICS: &[OpticEntry] = &[
    OpticEntry::of::<saturation::SaturationOptic>(),
    OpticEntry::of::<noise::NoiseOptic>(),
    OpticEntry::of::<aurora::AuroraOptic>(),
    OpticEntry::of::<reflection::ReflectionOptic>(),
    OpticEntry::of::<edge_highlight::EdgeHighlightOptic>(),
    OpticEntry::of::<iridescence::IridescenceOptic>(),
];

/// Every optic's uniform names, in `OPTICS` order, for the program.
pub fn uniform_names() -> Vec<UniformName<'static>> {
    OPTICS
        .iter()
        .flat_map(|entry| {
            entry
                .uniforms
                .iter()
                .map(|(name, ty)| UniformName::new(*name, *ty))
        })
        .collect()
}

/// Every optic's uniform values for this frame, in `OPTICS` order.
pub fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
    OPTICS
        .iter()
        .flat_map(|entry| (entry.values)(glass, ctx))
        .collect()
}

/// The earliest logical instant any optic changes on its own.
pub(crate) fn next_logical_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration> {
    OPTICS
        .iter()
        .filter_map(|entry| (entry.next_change)(glass, ctx))
        .min()
}

/// Map the earliest logical change to the real scheduler clock.
pub(crate) fn next_change(
    glass: &ResolvedGlass,
    frame: &OpticFrame<'_>,
    time: &OpticTime,
) -> Option<Duration> {
    debug_assert_eq!(
        frame.logical_now, time.logical_now,
        "optic frame and timeline samples must match",
    );
    if !time.running {
        return None;
    }
    next_logical_change(glass, frame)
        .map(|boundary| time.real_anchor + (boundary - time.logical_anchor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Clock;
    use niri_config::{
        NoiseSite, NoiseType, ResolvedIridescence, ResolvedNoise, ResolvedSaturation,
    };

    #[test]
    fn optic_settling_translates_deadline_and_suppresses_it_while_paused() {
        let clock = Clock::with_time(Duration::ZERO);
        let blur = Blur::default();
        let glass = ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 4.,
                ..Default::default()
            },
            ..Default::default()
        };
        clock.set_optic_active(false, Duration::from_millis(100));
        clock.set_optic_active(true, Duration::from_secs(10));
        let time = clock.optic_time(Duration::from_secs(10));
        let frame = OpticFrame {
            logical_now: time.logical_now,
            motion: SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur: false,
            blur: &blur,
            seed: 0.37,
        };
        assert_eq!(
            next_logical_change(&glass, &frame),
            Some(Duration::from_millis(250))
        );
        assert_eq!(
            next_change(&glass, &frame, &time),
            Some(Duration::from_millis(10_150))
        );
        clock.set_optic_active(false, Duration::from_secs(10));
        let paused = clock.optic_time(Duration::from_secs(20));
        assert_eq!(
            next_logical_change(&glass, &frame),
            Some(Duration::from_millis(250))
        );
        assert_eq!(next_change(&glass, &frame, &paused), None);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "optic frame and timeline samples must match")]
    fn optic_settling_rejects_mismatched_samples() {
        let clock = Clock::with_time(Duration::ZERO);
        let time = clock.optic_time(Duration::from_secs(1));
        let blur = Blur::default();
        let frame = OpticFrame {
            logical_now: Duration::ZERO,
            motion: SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur: false,
            blur: &blur,
            seed: 0.,
        };
        next_change(&ResolvedGlass::default(), &frame, &time);
    }

    #[test]
    fn optic_settling_holds_every_uniform_and_uses_future_boundaries() {
        let clock = Clock::with_time(Duration::ZERO);
        let blur = Blur::default();
        let glass = ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 4.,
                ..Default::default()
            },
            saturation: ResolvedSaturation { amount: Some(0.7) },
            noise: ResolvedNoise::single(0.2, NoiseType::White, NoiseSite::Glass),
            iridescence: ResolvedIridescence { amount: 0.8 },
            ..Default::default()
        };
        let frame = |time: &OpticTime| OpticFrame {
            logical_now: time.logical_now,
            motion: SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur: true,
            blur: &blur,
            seed: 0.37,
        };
        let before = clock.optic_time(Duration::from_nanos(249_999_999));
        assert_eq!(
            next_change(&glass, &frame(&before), &before),
            Some(Duration::from_millis(250))
        );
        let at = clock.optic_time(Duration::from_millis(250));
        assert_eq!(
            next_change(&glass, &frame(&at), &at),
            Some(Duration::from_millis(500))
        );
        assert_ne!(values(&glass, &frame(&before)), values(&glass, &frame(&at)));

        clock.set_optic_active(false, Duration::from_millis(250));
        let held = clock.optic_time(Duration::from_secs(10_000));
        let held_frame = frame(&held);
        for real in [Duration::from_secs(10_000), Duration::from_secs(20_000)] {
            let sample = clock.optic_time(real);
            let sampled_frame = frame(&sample);
            assert_eq!(values(&glass, &sampled_frame), values(&glass, &held_frame));
            for entry in OPTICS {
                assert_eq!(
                    (entry.values)(&glass, &sampled_frame),
                    (entry.values)(&glass, &held_frame)
                );
            }
            assert_eq!(next_change(&glass, &sampled_frame, &sample), None);
            assert_eq!(
                next_logical_change(&glass, &sampled_frame),
                Some(Duration::from_millis(500))
            );
        }
        clock.set_optic_active(true, Duration::from_secs(20_000));
        let resumed = clock.optic_time(Duration::from_secs(20_000));
        assert_eq!(
            next_change(&glass, &frame(&resumed), &resumed),
            Some(Duration::from_millis(20_000_250))
        );
        let individual = OPTICS
            .iter()
            .filter_map(|entry| (entry.next_change)(&glass, &frame(&resumed)))
            .min()
            .map(|boundary| resumed.real_anchor + (boundary - resumed.logical_anchor));
        assert_eq!(next_change(&glass, &frame(&resumed), &resumed), individual);
        clock.set_optic_active(false, Duration::from_millis(20_000_050));
        let second_hold = clock.optic_time(Duration::from_secs(21_000));
        assert_eq!(second_hold.logical_now, Duration::from_millis(300));
        clock.set_optic_active(true, Duration::from_secs(21_000));
        let second_resume = clock.optic_time(Duration::from_secs(21_000));
        assert_eq!(
            next_change(&glass, &frame(&second_resume), &second_resume),
            Some(Duration::from_millis(21_000_200))
        );

        for (rate, motion, expected_ns) in [
            (2.3, SignalMotionPolicy::Full, 434_782_609_u64),
            (2.3, SignalMotionPolicy::Reduced, 869_565_218_u64),
        ] {
            let mut changed = glass.clone();
            changed.aurora.drift_hz = rate;
            let initial = clock.optic_time(Duration::from_secs(21_000));
            let sample = OpticFrame {
                motion,
                ..frame(&initial)
            };
            assert_eq!(
                next_logical_change(&changed, &sample),
                Some(Duration::from_nanos(expected_ns))
            );
            let before_boundary = OpticFrame {
                logical_now: Duration::from_nanos(expected_ns - 1),
                ..sample
            };
            let at_boundary = OpticFrame {
                logical_now: Duration::from_nanos(expected_ns),
                ..sample
            };
            assert_ne!(
                values(&changed, &before_boundary),
                values(&changed, &at_boundary)
            );
        }
        let mut neutral = glass.clone();
        neutral.aurora.amount = 0.;
        assert_eq!(next_logical_change(&neutral, &frame(&resumed)), None);
        neutral.aurora.amount = 0.5;
        neutral.aurora.drift_hz = 0.;
        assert_eq!(next_logical_change(&neutral, &frame(&resumed)), None);
        assert_eq!(
            next_logical_change(
                &glass,
                &OpticFrame {
                    motion: SignalMotionPolicy::Off,
                    ..frame(&resumed)
                }
            ),
            None
        );
        assert_eq!(
            next_logical_change(
                &glass,
                &OpticFrame {
                    animations_off: true,
                    ..frame(&resumed)
                }
            ),
            None
        );

        let period_clock = Clock::with_time(Duration::ZERO);
        let before_wrap = period_clock.optic_time(Duration::from_nanos(599_999_999_999));
        let at_wrap = period_clock.optic_time(Duration::from_secs(600));
        assert_eq!(
            next_change(&glass, &frame(&before_wrap), &before_wrap),
            Some(Duration::from_secs(600))
        );
        assert_eq!(
            next_change(&glass, &frame(&at_wrap), &at_wrap),
            Some(Duration::from_millis(600_250))
        );
        assert_ne!(
            values(&glass, &frame(&before_wrap)),
            values(&glass, &frame(&at_wrap))
        );
    }
}
