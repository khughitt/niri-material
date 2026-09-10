//! `noise <amount> type=<type>`: stage 10, screen-space grain on the
//! encoded glass colour. Neutral at amount 0.

use std::str::FromStr;

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The grain pattern `noise` renders with. `White` is the original per-pixel
/// uniform hash; `Fine` is its high-pass, bell-shaped form; `Lightness`
/// applies `Fine` to Oklab lightness so chroma and hue hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NoiseType {
    #[default]
    White = 0,
    Fine = 1,
    Lightness = 2,
}

impl NoiseType {
    pub const NAMES: &'static [&'static str] = &["white", "fine", "lightness"];
}

impl FromStr for NoiseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "white" => Ok(Self::White),
            "fine" => Ok(Self::Fine),
            "lightness" => Ok(Self::Lightness),
            _ => Err(format!("unknown NoiseType value: {s}")),
        }
    }
}

/// `noise <amount> type=<type>`.
///
/// The type does nothing while the amount is zero, so it rides the node it
/// depends on, as `distortion` carries `scale=`. An omitted type is `white`,
/// which renders exactly as the node did before the property existed.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
}

/// Final noise state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedNoise {
    /// `None` inherits: the global `blur` block's value while backdrop blur
    /// is effective, the neutral 0 otherwise. A written value applies
    /// regardless of either switch. The renderer's optic applies the rule.
    pub amount: Option<f64>,
    /// No inheritance: the global `blur` block has no notion of grain type,
    /// so omission is `White` regardless of backdrop blur.
    pub kind: NoiseType,
}

pub fn resolve(node: Option<Noise>) -> ResolvedNoise {
    ResolvedNoise {
        amount: node.map(|n| n.amount.0),
        kind: node.and_then(|n| n.kind).unwrap_or_default(),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![
        ParamSpec {
            node: "noise",
            kind: ParamKind::inherit::<FloatOrInt<0, 1>>(),
            write: |v| format!("noise {v}"),
            read: Some(|g| g.noise.amount),
        },
        ParamSpec {
            node: "noise type=",
            kind: ParamKind::Enum {
                default: "white",
                variants: NoiseType::NAMES,
            },
            write: |v| format!("noise 0.5 type=\"{v}\""),
            read: None,
        },
    ]
}
