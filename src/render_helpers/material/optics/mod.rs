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

pub mod aurora;
pub mod iridescence;
pub mod noise;
pub mod saturation;

/// One pipeline stage. Implemented on a marker type per optic; the resolved
/// configuration arrives as `ResolvedGlass`, which owns every optic's state.
pub trait Optic {
    /// The registry name; matches `niri_config::material::optics::ORDER`.
    const NAME: &'static str;
    /// The optic's GLSL: its `uniform` declarations and `<NAME>_<hook>`
    /// functions.
    const GLSL: &'static str;
    /// The uniforms the GLSL declares, in declaration order.
    const UNIFORMS: &'static [(&'static str, UniformType)];
    /// This frame's uniforms, one per entry of `UNIFORMS`, in the same order.
    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>>;
    /// The next instant `values` changes with no config change; `None` for
    /// a static optic.
    fn next_change(_glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Option<Duration> {
        None
    }
}

/// What an optic may depend on beyond its own configuration.
#[derive(Debug, Clone, Copy)]
pub struct OpticFrame<'a> {
    /// The tile clock, unadjusted.
    pub now: Duration,
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
    OpticEntry::of::<iridescence::IridescenceOptic>(),
    OpticEntry::of::<aurora::AuroraOptic>(),
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

/// The earliest instant any optic changes on its own.
pub fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration> {
    OPTICS
        .iter()
        .filter_map(|entry| (entry.next_change)(glass, ctx))
        .min()
}
