//! `aurora <amount> { drift-hz; color; color }`: stage 6, a slow colour
//! field inside the glass. Neutral at amount 0.

use crate::appearance::Color;
use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// `aurora <amount> { drift-hz <hz>; color <a>; color <b>; }`.
///
/// A block node because it carries two colours. `color` appears zero times
/// (both defaults) or exactly twice; `validate` refuses any other count.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct Aurora {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(child, unwrap(argument))]
    pub drift_hz: Option<FloatOrInt<0, 30>>,
    #[knuffel(children(name = "color"))]
    pub colors: Vec<Color>,
}

impl Aurora {
    /// The two rules the scalar bounds cannot express: the colour count,
    /// and the `ring-drift-hz` rule for the field's clock.
    pub fn validate(&self) -> Result<(), String> {
        if !(self.colors.is_empty() || self.colors.len() == 2) {
            return Err(String::from("aurora: expected two color nodes"));
        }
        // The clock divides a 600 s period into `hz * 600` buckets; a rate
        // below 1 Hz (before the reduced-motion halving) is refused rather
        // than clamped, as `ring-drift-hz` is.
        if self.drift_hz.is_some_and(|hz| hz.0 > 0. && hz.0 < 1.) {
            return Err(String::from("aurora drift-hz must be 0 or at least 1"));
        }
        Ok(())
    }
}

/// Final aurora state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedAurora {
    /// Strength of the field's light; 0 adds nothing.
    pub amount: f64,
    /// Bucket rate of the field's clock in Hz; 0 pins the field.
    pub drift_hz: f64,
    /// The field mixes from `color_a` (noise 0) to `color_b` (noise 1).
    pub color_a: Color,
    pub color_b: Color,
}

impl Default for ResolvedAurora {
    fn default() -> Self {
        Self {
            amount: 0.,
            drift_hz: 4.,
            color_a: Color::from_rgba8_unpremul(0x3d, 0xff, 0xb0, 0xff),
            color_b: Color::from_rgba8_unpremul(0x7a, 0x5c, 0xff, 0xff),
        }
    }
}

pub fn resolve(node: Option<&Aurora>) -> ResolvedAurora {
    let d = ResolvedAurora::default();
    let Some(node) = node else {
        return d;
    };
    let (color_a, color_b) = match node.colors.as_slice() {
        [] => (d.color_a, d.color_b),
        [a, b] => (*a, *b),
        _ => unreachable!("Aurora::validate runs before resolution"),
    };
    ResolvedAurora {
        amount: node.amount.0,
        drift_hz: node.drift_hz.map_or(d.drift_hz, |x| x.0),
        color_a,
        color_b,
    }
}

pub fn params() -> Vec<ParamSpec> {
    let d = ResolvedAurora::default();
    vec![
        ParamSpec {
            node: "aurora",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.amount, "—"),
            write: |v| format!("aurora {v}"),
            read: Some(|g| Some(g.aurora.amount)),
        },
        ParamSpec {
            node: "aurora drift-hz",
            kind: ParamKind::float::<FloatOrInt<0, 30>>(d.drift_hz, "Hz"),
            write: |v| format!("aurora 0.5 {{ drift-hz {v}; }}"),
            read: Some(|g| Some(g.aurora.drift_hz)),
        },
        // Two rows with the same node: the first `color` is the field's
        // start and the second its end. The prose under the table says so.
        ParamSpec {
            node: "aurora color",
            kind: ParamKind::Color { default: d.color_a },
            write: |v| format!("aurora 0.5 {{ color \"{v}\"; color \"#000000\"; }}"),
            read: None,
        },
        ParamSpec {
            node: "aurora color",
            kind: ParamKind::Color { default: d.color_b },
            write: |v| format!("aurora 0.5 {{ color \"#000000\"; color \"{v}\"; }}"),
            read: None,
        },
    ]
}
