//! `noise <amount> type=<type> site=<site>`: grain on the shared backdrop,
//! transmitted backdrop, or encoded glass. Neutral at amount 0.

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

/// Where `noise` acts (design `2026-10-05-noise-placement-design.md` §3).
/// `Glass` is the behind hook on the material's averaged taps; `Backdrop`
/// grains the output's shared effect-buffer texture before the blur and the
/// prefilter pyramids; `Film` grains the encoded glass after the light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NoiseSite {
    #[default]
    Glass = 0,
    Backdrop = 1,
    Film = 2,
}

impl NoiseSite {
    pub const NAMES: &'static [&'static str] = &["glass", "backdrop", "film"];
}

impl FromStr for NoiseSite {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "glass" => Ok(Self::Glass),
            "backdrop" => Ok(Self::Backdrop),
            "film" => Ok(Self::Film),
            _ => Err(format!("unknown NoiseSite value: {s}")),
        }
    }
}

/// `noise <amount> type=<type> site=<site>`.
///
/// The type and the site do nothing while the amount is zero, so they ride
/// the node they depend on, as `distortion` carries `scale=`. An omitted
/// type is `white` and an omitted site is `glass`, which render exactly as
/// the node did before the properties existed.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
    #[knuffel(property(name = "site"), str)]
    pub site: Option<NoiseSite>,
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
    /// No inheritance either; a site is a property of a written node, so a
    /// site other than `Glass` always comes with `Some` amount.
    pub site: NoiseSite,
}

pub fn resolve(node: Option<Noise>) -> ResolvedNoise {
    ResolvedNoise {
        amount: node.map(|n| n.amount.0),
        kind: node.and_then(|n| n.kind).unwrap_or_default(),
        site: node.and_then(|n| n.site).unwrap_or_default(),
    }
}

/// The one grain an output's effect buffer applies for every material that
/// places noise at the backdrop (design §3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropGrain {
    pub amount: f64,
    pub kind: NoiseType,
}

/// The backdrop grain the material table agrees on: `None` when no material
/// places noise there, the agreed pair otherwise, and an error naming the
/// first two materials that disagree. Grain is applied to the output's shared
/// backdrop buffers, so there is one setting per output and no per-material
/// value to fall back on.
pub fn backdrop_grain(
    materials: &[crate::material::Material],
) -> Result<Option<BackdropGrain>, String> {
    let mut agreed: Option<(&str, BackdropGrain)> = None;
    for material in materials {
        let noise = resolve(material.glass.noise);
        if noise.site != NoiseSite::Backdrop {
            continue;
        }
        let amount = noise
            .amount
            .expect("a site is a property of a written noise node, which has an amount");
        let grain = BackdropGrain {
            amount,
            kind: noise.kind,
        };
        match agreed {
            None => agreed = Some((&material.name, grain)),
            Some((first, g)) if g != grain => {
                return Err(format!(
                    "materials \"{first}\" and \"{}\" both place noise at the backdrop with \
                     different settings ({} {}, {} {}); backdrop grain is one setting per output",
                    material.name,
                    g.amount,
                    NoiseType::NAMES[g.kind as usize],
                    grain.amount,
                    NoiseType::NAMES[grain.kind as usize],
                ));
            }
            Some(_) => {}
        }
    }
    Ok(agreed.map(|(_, grain)| grain))
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
        ParamSpec {
            node: "noise site=",
            kind: ParamKind::Enum {
                default: "glass",
                variants: NoiseSite::NAMES,
            },
            write: |v| format!("noise 0.5 site=\"{v}\""),
            read: None,
        },
    ]
}
