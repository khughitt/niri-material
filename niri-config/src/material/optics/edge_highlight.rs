//! `edge-highlight <amount>`: stage 6, a key-light lobe on the bevel. Neutral at amount 0.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type EdgeHighlight = FloatOrInt<0, 1>;

/// Final edge-highlight state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedEdgeHighlight {
    /// Peak added linear brightness of the lobe; 0 is none.
    pub amount: f64,
}

pub fn resolve(node: Option<EdgeHighlight>) -> ResolvedEdgeHighlight {
    ResolvedEdgeHighlight {
        amount: node.map_or(ResolvedEdgeHighlight::default().amount, |x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "edge-highlight",
        kind: ParamKind::float::<EdgeHighlight>(ResolvedEdgeHighlight::default().amount, "—"),
        write: |v| format!("edge-highlight {v}"),
        read: Some(|g| Some(g.edge_highlight.amount)),
    }]
}
