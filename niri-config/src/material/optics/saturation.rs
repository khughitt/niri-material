//! `saturation <amount>`: stage 9, `mix(luma, color, amount)` on the
//! encoded glass colour. Neutral at 1.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Saturation = FloatOrInt<0, 3>;

/// Final saturation state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedSaturation {
    /// `None` inherits: the global `blur` block's value while backdrop blur
    /// is effective, the neutral 1 otherwise. A written value applies
    /// regardless of either switch. The renderer's optic applies the rule.
    pub amount: Option<f64>,
}

pub fn resolve(node: Option<Saturation>) -> ResolvedSaturation {
    ResolvedSaturation {
        amount: node.map(|x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "saturation",
        kind: ParamKind::inherit::<Saturation>(),
        write: |v| format!("saturation {v}"),
        read: Some(|g| g.saturation.amount),
    }]
}
