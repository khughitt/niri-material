//! `iridescence <amount>`: stage 5, a thin-film hue on the Fresnel glint.
//! Neutral at amount 0.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Iridescence = FloatOrInt<0, 1>;

/// Final iridescence state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedIridescence {
    /// Mix of the hued glint over the plain one; 0 is the plain glint.
    pub amount: f64,
}

pub fn resolve(node: Option<Iridescence>) -> ResolvedIridescence {
    ResolvedIridescence {
        amount: node.map_or(ResolvedIridescence::default().amount, |x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "iridescence",
        kind: ParamKind::float::<Iridescence>(ResolvedIridescence::default().amount, "—"),
        write: |v| format!("iridescence {v}"),
        read: Some(|g| Some(g.iridescence.amount)),
    }]
}
