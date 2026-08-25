//! Material definitions.
//!
//! This is the config surface for the native material system; the parameter
//! table, its defaults and its ranges are specified in
//! `docs/materials/2026-08-22-v1-design.md` §4.

use std::cell::RefCell;
use std::rc::Rc;

use knuffel::errors::DecodeError;

use crate::appearance::Color;
use crate::FloatOrInt;

/// A window-rule reference to a material definition by name.
///
/// The referenced definition may appear later in the file or in an include,
/// so the reference cannot be resolved here. Each one records itself into the
/// shared parse context; `validate_material_refs` checks them all once every
/// include has merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRef(pub String);

/// Window-rule material references collected during a parse.
#[derive(Debug)]
pub struct MaterialRefs<S> {
    /// References from the root config file. Their spans are byte offsets
    /// into the source the root context reports against, so these can carry
    /// a proper caret.
    pub root: Vec<knuffel::span::Spanned<knuffel::ast::Literal, S>>,
    /// References from included files, as (material name, file name).
    ///
    /// A `knuffel::Span` is a bare byte range with no file identity, and each
    /// include is parsed against its own `NamedSource`. Emitting an include's
    /// span into the root context would underline the wrong file, so these
    /// are reported without a snippet and name their file in the message.
    pub included: Vec<(String, String)>,
}

impl<S> Default for MaterialRefs<S> {
    fn default() -> Self {
        Self {
            root: Vec::new(),
            included: Vec::new(),
        }
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::DecodeScalar<S> for MaterialRef {
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
        let knuffel::ast::Literal::String(ref s) = **val else {
            ctx.emit_error(DecodeError::unsupported(
                val,
                "material references must be strings",
            ));
            return Ok(Self(String::new()));
        };

        let refs = ctx
            .get::<Rc<RefCell<MaterialRefs<S>>>>()
            .expect("material refs must be set in the parse context")
            .clone();
        let recursion = ctx
            .get::<crate::Recursion>()
            .expect("recursion must be set in the parse context")
            .0;
        let file = ctx
            .get::<crate::FileName>()
            .expect("file name must be set in the parse context")
            .0
            .clone();

        if recursion == 0 {
            refs.borrow_mut().root.push(val.clone());
        } else {
            refs.borrow_mut().included.push((s.to_string(), file));
        }

        Ok(Self(s.clone().into()))
    }
}

/// Reports every window-rule material reference that names no definition.
///
/// Runs once, after all includes have merged into `materials`.
pub fn validate_material_refs<S: knuffel::traits::ErrorSpan>(
    refs: &MaterialRefs<S>,
    materials: &[Material],
    ctx: &mut knuffel::decode::Context<S>,
) {
    let known = |name: &str| materials.iter().any(|m| m.name == name);

    for val in &refs.root {
        let knuffel::ast::Literal::String(ref s) = **val else {
            continue;
        };

        if !known(s) {
            ctx.emit_error(DecodeError::unexpected(
                val,
                "material",
                format!("unknown material: {s}"),
            ));
        }
    }

    for (name, file) in &refs.included {
        if !known(name) {
            ctx.emit_error(DecodeError::Custom(
                format!("unknown material: {name} (referenced in {file})").into(),
            ));
        }
    }
}

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
    pub bevel: Option<FloatOrInt<0, 128>>,
    #[knuffel(child, unwrap(argument))]
    pub offset_x: Option<FloatOrInt<-64, 64>>,
    #[knuffel(child, unwrap(argument))]
    pub offset_y: Option<FloatOrInt<-64, 64>>,
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
    pub bevel: f64,
    pub offset_x: f64,
    pub offset_y: f64,
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
            bevel: 12.,
            offset_x: 6.,
            offset_y: 6.,
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
                bevel: g.bevel.map_or(d.bevel, |x| x.0),
                offset_x: g.offset_x.map_or(d.offset_x, |x| x.0),
                offset_y: g.offset_y.map_or(d.offset_y, |x| x.0),
            },
        }
    }

    /// Checks the one rule that spans two parameters.
    ///
    /// `material_frame` derives the uniform inflation as
    /// `bevel - max(|offset-x|, |offset-y|)`. A negative inflation would put
    /// the slab inside the window on one side, which has no meaning, so the
    /// offsets are bounded by the bevel rather than clamped silently.
    pub(crate) fn validate(&self) -> Result<(), String> {
        let d = ResolvedGlass::default();
        let bevel = self.glass.bevel.map_or(d.bevel, |x| x.0);
        let offset_x = self.glass.offset_x.map_or(d.offset_x, |x| x.0);
        let offset_y = self.glass.offset_y.map_or(d.offset_y, |x| x.0);

        if offset_x.abs().max(offset_y.abs()) > bevel {
            return Err(String::from("offset must not exceed bevel"));
        }

        Ok(())
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
