//! Optics: the self-contained stages of the glass pipeline, config side.
//!
//! Each optic owns its node struct, its resolved struct and defaults, its
//! `resolve`, and its `params`. The renderer's `OPTICS` table must list the
//! optics in `ORDER`; a test in niri pins the two.
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §2.

use super::params::ParamSpec;

pub mod aurora;
pub mod iridescence;
pub mod noise;
pub mod saturation;

/// Render order of the optics, which is also the order of their rows in
/// the parameter table.
pub const ORDER: &[&str] = &["saturation", "noise", "iridescence", "aurora"];

/// Every optic's parameter specs, in `ORDER`.
pub fn params() -> Vec<ParamSpec> {
    let mut specs = saturation::params();
    specs.extend(noise::params());
    specs.extend(iridescence::params());
    specs.extend(aurora::params());
    specs
}
