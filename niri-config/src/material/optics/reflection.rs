//! `reflection <amount>`: stage 6, the scene just beyond the silhouette,
//! reflected by the bevel and untinted. Neutral at amount 0.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Reflection = FloatOrInt<0, 1>;

/// Final reflection state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedReflection {
    /// Gain on the Fresnel-weighted reflection; 0 is none.
    pub amount: f64,
}

pub fn resolve(node: Option<Reflection>) -> ResolvedReflection {
    ResolvedReflection {
        amount: node.map_or(ResolvedReflection::default().amount, |x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "reflection",
        kind: ParamKind::float::<Reflection>(ResolvedReflection::default().amount, "—"),
        write: |v| format!("reflection {v}"),
        read: Some(|g| Some(g.reflection.amount)),
    }]
}
