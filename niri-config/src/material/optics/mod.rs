//! Optics: the self-contained stages of the glass pipeline, config side.
//!
//! Each optic owns its node struct, its resolved struct and defaults, its
//! `resolve`, and its `params`. The renderer's `OPTICS` table must list the
//! optics in `ORDER`; a test in niri pins the two.
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §2.

use super::params::ParamSpec;

pub mod saturation;

/// Render order of the optics, which is also the order of their rows in
/// the parameter table.
pub const ORDER: &[&str] = &["saturation"];

/// Every optic's parameter specs, in `ORDER`.
pub fn params() -> Vec<ParamSpec> {
    saturation::params()
}
