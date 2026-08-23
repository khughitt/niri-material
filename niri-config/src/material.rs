//! Material definitions.
//!
//! This is the config surface for the native material system; the parameter
//! table, its defaults and its ranges are specified in
//! `docs/materials/2026-08-22-v1-design.md` §4.

use knuffel::errors::DecodeError;

use crate::appearance::Color;
use crate::FloatOrInt;

/// A `material "name" { ... }` definition block.
///
/// v1 has exactly one material type, so `glass` is a required single child
/// and knuffel enforces the design's "exactly one material type block per
/// definition" rule for us: a missing `glass` is a missing-node error and a
/// second one is `duplicate node 'glass', single node expected`. When a
/// second material type is added, this field becomes a hand-written
/// dispatcher and the rule needs re-implementing there.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct Material {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child)]
    pub glass: Glass,
}

/// Glass parameters as written in the config; every one is optional and an
/// omitted parameter takes its `ResolvedGlass::default()` value.
#[derive(knuffel::Decode, Debug, Clone, Default, PartialEq)]
pub struct Glass {
    #[knuffel(child, unwrap(argument))]
    pub ior: Option<FloatOrInt<1, 3>>,
    #[knuffel(child, unwrap(argument))]
    pub thickness: Option<FloatOrInt<0, 200>>,
    #[knuffel(child)]
    pub attenuation_color: Option<Color>,
    #[knuffel(child, unwrap(argument))]
    pub attenuation_distance: Option<Positive<65535>>,
    #[knuffel(child, unwrap(argument))]
    pub chromatic_aberration: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub distortion: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub distortion_scale: Option<FloatOrInt<0, 2>>,
    #[knuffel(child, unwrap(argument))]
    pub samples: Option<Samples>,
    #[knuffel(child, unwrap(argument))]
    pub anisotropic_blur: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub jelly_flex: Option<Milli<0, 20>>,
    #[knuffel(child, unwrap(argument))]
    pub jelly_ripple: Option<Milli<0, 500>>,
    #[knuffel(child, unwrap(argument))]
    pub lip: Option<FloatOrInt<0, 64>>,
    #[knuffel(child, unwrap(argument))]
    pub shift_x: Option<FloatOrInt<-64, 64>>,
    #[knuffel(child, unwrap(argument))]
    pub shift_y: Option<FloatOrInt<-64, 64>>,
}

/// A material definition with every parameter resolved to a final value.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMaterial {
    pub name: String,
    pub glass: ResolvedGlass,
}

/// Final glass parameter values. Lengths are logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedGlass {
    pub ior: f64,
    pub thickness: f64,
    pub attenuation_color: Color,
    pub attenuation_distance: f64,
    pub chromatic_aberration: f64,
    pub distortion: f64,
    pub distortion_scale: f64,
    pub samples: u8,
    pub anisotropic_blur: f64,
    pub jelly_flex: f64,
    pub jelly_ripple: f64,
    pub lip: f64,
    pub shift_x: f64,
    pub shift_y: f64,
}

impl Default for ResolvedGlass {
    // The v1 design §4 defaults, which are the proven niri-glass.json values.
    fn default() -> Self {
        Self {
            ior: 1.5,
            thickness: 20.,
            attenuation_color: Color::from_rgba8_unpremul(0xdf, 0xe8, 0xff, 0xff),
            attenuation_distance: 60.,
            chromatic_aberration: 0.,
            distortion: 0.,
            distortion_scale: 0.5,
            samples: 4,
            anisotropic_blur: 0.,
            jelly_flex: 0.004,
            jelly_ripple: 0.06,
            lip: 6.,
            shift_x: 6.,
            shift_y: 6.,
        }
    }
}

impl Material {
    /// Resolves the definition, applying the design §4 default for every
    /// omitted parameter.
    pub fn resolve(&self) -> ResolvedMaterial {
        let g = &self.glass;
        let d = ResolvedGlass::default();

        ResolvedMaterial {
            name: self.name.clone(),
            glass: ResolvedGlass {
                ior: g.ior.map_or(d.ior, |x| x.0),
                thickness: g.thickness.map_or(d.thickness, |x| x.0),
                attenuation_color: g.attenuation_color.unwrap_or(d.attenuation_color),
                attenuation_distance: g
                    .attenuation_distance
                    .map_or(d.attenuation_distance, |x| x.0),
                chromatic_aberration: g
                    .chromatic_aberration
                    .map_or(d.chromatic_aberration, |x| x.0),
                distortion: g.distortion.map_or(d.distortion, |x| x.0),
                distortion_scale: g.distortion_scale.map_or(d.distortion_scale, |x| x.0),
                samples: g.samples.map_or(d.samples, |x| x.0),
                anisotropic_blur: g.anisotropic_blur.map_or(d.anisotropic_blur, |x| x.0),
                jelly_flex: g.jelly_flex.map_or(d.jelly_flex, |x| x.0),
                jelly_ripple: g.jelly_ripple.map_or(d.jelly_ripple, |x| x.0),
                lip: g.lip.map_or(d.lip, |x| x.0),
                shift_x: g.shift_x.map_or(d.shift_x, |x| x.0),
                shift_y: g.shift_y.map_or(d.shift_y, |x| x.0),
            },
        }
    }
}

/// A float parameter whose inclusive bounds are given in thousandths.
///
/// `FloatOrInt` takes its bounds as whole numbers, which cannot express
/// ranges like `jelly-flex`'s 0–0.02. `Milli<0, 20>` is that range.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Milli<const MIN_MILLI: i32, const MAX_MILLI: i32>(pub f64);

impl<S: knuffel::traits::ErrorSpan, const MIN_MILLI: i32, const MAX_MILLI: i32>
    knuffel::DecodeScalar<S> for Milli<MIN_MILLI, MAX_MILLI>
{
    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(type_name) = &type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
    }

    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let min = f64::from(MIN_MILLI) / 1000.;
        let max = f64::from(MAX_MILLI) / 1000.;

        let value: Option<f64> = match &**val {
            knuffel::ast::Literal::Int(ref value) => i32::try_from(value).ok().map(f64::from),
            knuffel::ast::Literal::Decimal(ref value) => f64::try_from(value).ok(),
            _ => {
                ctx.emit_error(DecodeError::unsupported(val, "expected a number"));
                return Ok(Self::default());
            }
        };

        match value {
            Some(v) if (min..=max).contains(&v) => Ok(Self(v)),
            _ => {
                ctx.emit_error(DecodeError::conversion(
                    val,
                    format!("value must be between {min} and {max}"),
                ));
                Ok(Self::default())
            }
        }
    }
}

/// A strictly positive float parameter with an inclusive upper bound.
///
/// `FloatOrInt<0, MAX>` would admit 0, which `attenuation-distance` must
/// reject: it is a divisor in the Beer-Lambert term.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Positive<const MAX: i32>(pub f64);

impl<S: knuffel::traits::ErrorSpan, const MAX: i32> knuffel::DecodeScalar<S> for Positive<MAX> {
    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(type_name) = &type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
    }

    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let value: Option<f64> = match &**val {
            knuffel::ast::Literal::Int(ref value) => i32::try_from(value).ok().map(f64::from),
            knuffel::ast::Literal::Decimal(ref value) => f64::try_from(value).ok(),
            _ => {
                ctx.emit_error(DecodeError::unsupported(val, "expected a number"));
                return Ok(Self::default());
            }
        };

        match value {
            Some(v) if v > 0. && v <= f64::from(MAX) => Ok(Self(v)),
            _ => {
                ctx.emit_error(DecodeError::conversion(
                    val,
                    format!("value must be greater than 0 and at most {MAX}"),
                ));
                Ok(Self::default())
            }
        }
    }
}

/// The `samples` parameter: an integer in 1–8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Samples(pub u8);

impl Default for Samples {
    fn default() -> Self {
        Self(ResolvedGlass::default().samples)
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::DecodeScalar<S> for Samples {
    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(type_name) = &type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
    }

    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let knuffel::ast::Literal::Int(ref value) = **val else {
            ctx.emit_error(DecodeError::unsupported(val, "samples must be an integer"));
            return Ok(Self::default());
        };

        match i32::try_from(value) {
            Ok(v) if (1..=8).contains(&v) => Ok(Self(v as u8)),
            _ => {
                ctx.emit_error(DecodeError::conversion(
                    val,
                    "value must be between 1 and 8",
                ));
                Ok(Self::default())
            }
        }
    }
}
