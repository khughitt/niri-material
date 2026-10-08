//! `noise <amount> type=<type> site=<site> scale=<px>`, up to four layers: grain on the shared backdrop,
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

/// How many `noise` nodes one material may write (design
/// `2026-10-06-noise-layers-design.md` §3).
pub const NOISE_LAYERS: usize = 4;

/// `noise <amount> type=<type> site=<site> scale=<px>`, one layer.
///
/// The type, site and scale do nothing while the amount is zero, so they
/// ride the node they depend on, as `distortion` carries `scale=`. An omitted
/// type is `white`, an omitted site `glass` and an omitted scale 1, which
/// render exactly as the node did before the properties existed.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
    #[knuffel(property(name = "site"), str)]
    pub site: Option<NoiseSite>,
    /// Grain cell size in physical output pixels; 1 is per-pixel grain.
    #[knuffel(property)]
    pub scale: Option<FloatOrInt<1, 16>>,
}

/// One written layer, every property resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedNoiseLayer {
    pub amount: f64,
    pub kind: NoiseType,
    pub site: NoiseSite,
    pub scale: f64,
}

/// Final noise state: the written layers in config order, then `None`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedNoise {
    /// All `None` means no node was written. Only then does the inherit rule
    /// apply, in the renderer's optic: slot 0 becomes white grain at the
    /// glass site, scale 1, with the global `blur` block's amount while
    /// backdrop blur is effective and 0 otherwise. A written layer never
    /// inherits.
    pub layers: [Option<ResolvedNoiseLayer>; NOISE_LAYERS],
}

impl ResolvedNoise {
    /// One written layer at scale 1.
    pub fn single(amount: f64, kind: NoiseType, site: NoiseSite) -> Self {
        let mut layers = [None; NOISE_LAYERS];
        layers[0] = Some(ResolvedNoiseLayer {
            amount,
            kind,
            site,
            scale: 1.,
        });
        Self { layers }
    }

    /// Whether no node was written, so slot 0 inherits.
    pub fn is_omitted(&self) -> bool {
        self.layers[0].is_none()
    }
}

/// The written nodes, in order. A rejected material still reaches the
/// post-include backdrop check, so this takes at most `NOISE_LAYERS` nodes
/// rather than assuming `validate` passed; an accepted config never has more.
pub fn resolve(nodes: &[Noise]) -> ResolvedNoise {
    let mut layers = [None; NOISE_LAYERS];
    for (slot, node) in layers.iter_mut().zip(nodes) {
        *slot = Some(ResolvedNoiseLayer {
            amount: node.amount.0,
            kind: node.kind.unwrap_or_default(),
            site: node.site.unwrap_or_default(),
            scale: node.scale.map_or(1., |scale| scale.0),
        });
    }
    ResolvedNoise { layers }
}

pub fn validate(nodes: &[Noise]) -> Result<(), String> {
    if nodes.len() > NOISE_LAYERS {
        return Err(format!(
            "at most {NOISE_LAYERS} noise layers, found {}",
            nodes.len()
        ));
    }
    Ok(())
}

/// One backdrop layer as the output's grain pass applies it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropLayer {
    pub amount: f64,
    pub kind: NoiseType,
    pub scale: f64,
}

/// The grain an output's effect buffer applies: the backdrop-site layers,
/// in order, that every material placing noise there agrees on (design §3).
/// Slot `k` here is the `k`-th backdrop layer, which also picks its seed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropGrain {
    pub layers: [Option<BackdropLayer>; NOISE_LAYERS],
}

impl BackdropGrain {
    fn of(noise: &ResolvedNoise) -> Option<Self> {
        let mut layers = [None; NOISE_LAYERS];
        let placed = noise
            .layers
            .iter()
            .flatten()
            .filter(|layer| layer.site == NoiseSite::Backdrop);
        for (slot, layer) in layers.iter_mut().zip(placed) {
            *slot = Some(BackdropLayer {
                amount: layer.amount,
                kind: layer.kind,
                scale: layer.scale,
            });
        }
        layers[0].is_some().then_some(Self { layers })
    }
}

impl std::fmt::Display for BackdropGrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, layer) in self.layers.iter().flatten().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(
                f,
                "{} {}",
                layer.amount,
                NoiseType::NAMES[layer.kind as usize]
            )?;
            if layer.scale != 1. {
                write!(f, " scale {}", layer.scale)?;
            }
        }
        f.write_str("]")
    }
}

/// The backdrop grain the material table agrees on: `None` when no material
/// places noise there, the agreed list otherwise, and an error naming the
/// first two materials that disagree. Grain is applied to the output's shared
/// backdrop buffers, so there is one setting per output and no per-material
/// value to fall back on.
pub fn backdrop_grain(
    materials: &[crate::material::Material],
) -> Result<Option<BackdropGrain>, String> {
    let mut agreed: Option<(&str, BackdropGrain)> = None;
    for material in materials {
        let Some(grain) = BackdropGrain::of(&resolve(&material.glass.noise)) else {
            continue;
        };
        match agreed {
            None => agreed = Some((&material.name, grain)),
            Some((first, g)) if g != grain => {
                return Err(format!(
                    "materials \"{first}\" and \"{}\" place different noise at the backdrop \
                     ({g}, {grain}); backdrop grain is one setting per output",
                    material.name,
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
            read: Some(|g| g.noise.layers[0].map(|layer| layer.amount)),
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
        ParamSpec {
            node: "noise scale=",
            kind: ParamKind::float::<FloatOrInt<1, 16>>(1., "px"),
            write: |v| format!("noise 0.5 scale={v}"),
            read: Some(|g| Some(g.noise.layers[0].map_or(1., |layer| layer.scale))),
        },
    ]
}
