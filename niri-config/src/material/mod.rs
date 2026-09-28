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

pub mod optics;
pub mod params;

/// A window-rule reference to a material definition by name.
///
/// The referenced definition may appear later in the file or in an include,
/// so the reference cannot be resolved here. Each one records itself into the
/// shared parse context; `validate_material_refs` checks them all once every
/// include has merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRef {
    pub name: String,
    pub response: Option<String>,
}

/// Window-rule material references collected during a parse.
#[derive(Debug)]
pub struct MaterialRefs<S> {
    /// References from the root config file. Their spans are byte offsets
    /// into the source the root context reports against, so these can carry
    /// a proper caret.
    pub root: Vec<(
        knuffel::span::Spanned<knuffel::ast::Literal, S>,
        Option<String>,
    )>,
    /// References from included files, as (material name, response, file name).
    ///
    /// A `knuffel::Span` is a bare byte range with no file identity, and each
    /// include is parsed against its own `NamedSource`. Emitting an include's
    /// span into the root context would underline the wrong file, so these
    /// are reported without a snippet and name their file in the message.
    pub included: Vec<(String, Option<String>, String)>,
}

impl<S> Default for MaterialRefs<S> {
    fn default() -> Self {
        Self {
            root: Vec::new(),
            included: Vec::new(),
        }
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::Decode<S> for MaterialRef {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
        let Some(arg) = node.arguments.first() else {
            return Err(DecodeError::missing(node, "material name argument"));
        };
        let name: String = knuffel::traits::DecodeScalar::decode(arg, ctx)?;
        if let Some(extra) = node.arguments.get(1) {
            ctx.emit_error(DecodeError::unexpected(
                &extra.literal,
                "argument",
                "unexpected argument",
            ));
        }
        for child in node.children() {
            ctx.emit_error(DecodeError::unexpected(
                child,
                "node",
                format!("unexpected node `{}`", child.node_name.escape_default()),
            ));
        }
        let mut response = None;
        for (key, val) in &node.properties {
            match &***key {
                "response" => {
                    let key_span: miette::SourceSpan = key.span().clone().into();
                    let value_span: miette::SourceSpan = val
                        .type_name
                        .as_ref()
                        .map_or_else(|| val.literal.span().clone(), |ty| ty.span().clone())
                        .into();
                    // Knuffel's property map retains the first key and last value on duplicates.
                    let duplicate = value_span.offset() != key_span.offset() + key_span.len() + 1;
                    if response.is_some() || duplicate {
                        ctx.emit_error(DecodeError::unexpected(
                            key,
                            "property",
                            "unexpected duplicate property `response`",
                        ));
                    }
                    response = Some(knuffel::traits::DecodeScalar::decode(val, ctx)?);
                }
                other => {
                    return Err(DecodeError::unexpected(
                        key,
                        "property",
                        format!("unexpected property `{other}`"),
                    ));
                }
            }
        }

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
            refs.borrow_mut()
                .root
                .push((arg.literal.clone(), response.clone()));
        } else {
            refs.borrow_mut()
                .included
                .push((name.clone(), response.clone(), file));
        }

        Ok(Self { name, response })
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
    let find = |name: &str| materials.iter().find(|m| m.name == name);
    let response_known = |material: &Material, response: &str| {
        material.responses.is_empty() && response == "default"
            || material.responses.iter().any(|r| r.name == response)
    };

    for (val, response) in &refs.root {
        let knuffel::ast::Literal::String(ref s) = **val else {
            continue;
        };

        match find(s) {
            None => ctx.emit_error(DecodeError::unexpected(
                val,
                "material",
                format!("unknown material: {s}"),
            )),
            Some(material) => {
                if let Some(response) = response
                    .as_ref()
                    .filter(|response| !response_known(material, response))
                {
                    ctx.emit_error(DecodeError::unexpected(
                        val,
                        "material",
                        format!("material {s}: unknown response: {response}"),
                    ));
                }
            }
        }
    }

    for (name, response, file) in &refs.included {
        match find(name) {
            None => ctx.emit_error(DecodeError::Custom(
                format!("unknown material: {name} (referenced in {file})").into(),
            )),
            Some(material) => {
                if let Some(response) = response
                    .as_ref()
                    .filter(|response| !response_known(material, response))
                {
                    ctx.emit_error(DecodeError::Custom(
                        format!(
                            "material {name}: unknown response: {response} (referenced in {file})"
                        )
                        .into(),
                    ));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AccentResponse {
    None = 0,
    #[default]
    Ring = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AttentionResponse {
    None = 0,
    #[default]
    RimOrbit = 1,
    RingPulse = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ImpulseResponse {
    #[default]
    None = 0,
    Ripple = 1,
    Flash = 2,
    Sweep = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FocusResponse {
    None = 0,
    #[default]
    RingLight = 1,
}

macro_rules! response_from_str {
    ($ty:ident, $($s:literal => $v:ident),+ $(,)?) => {
        impl std::str::FromStr for $ty {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, String> {
                match s {
                    $($s => Ok(Self::$v),)+
                    _ => Err(format!(concat!("unknown ", stringify!($ty), " value: {}"), s)),
                }
            }
        }
    };
}

response_from_str!(AccentResponse, "none" => None, "ring" => Ring);
response_from_str!(AttentionResponse, "none" => None, "rim-orbit" => RimOrbit, "ring-pulse" => RingPulse);
response_from_str!(ImpulseResponse, "none" => None, "ripple" => Ripple, "flash" => Flash, "sweep" => Sweep);
response_from_str!(FocusResponse, "none" => None, "ring-light" => RingLight);

/// A `response "name" { ... }` block inside a material definition.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct Response {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument, str))]
    pub accent: Option<AccentResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub attention: Option<AttentionResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub ping: Option<ImpulseResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub done: Option<ImpulseResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub error: Option<ImpulseResponse>,
    /// Retired: decoded only so the error can name its replacement.
    #[knuffel(child, unwrap(argument))]
    pub ring_inset: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub ring_gap: Option<FloatOrInt<0, 128>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_glow: Option<FloatOrInt<0, 3>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_beam_speed: Option<FloatOrInt<0, 5000>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_beam_noise: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_beam_noise_hz: Option<FloatOrInt<0, 30>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_beam_decay: Option<FloatOrInt<0, 20000>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_width: Option<FloatOrInt<0, 128>>,
    #[knuffel(child, unwrap(argument, str))]
    pub focus: Option<FocusResponse>,
    #[knuffel(child)]
    pub ring_color: Option<Color>,
    /// Retired: decoded only so the error can name its replacement.
    #[knuffel(child, unwrap(argument))]
    pub ring_drift_hz: Option<f64>,
    /// Retired: decoded only so the error can name its replacement.
    #[knuffel(child, unwrap(argument))]
    pub ring_sweep_ms: Option<u32>,
}

/// A fully resolved response block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedResponse {
    pub accent: AccentResponse,
    pub attention: AttentionResponse,
    pub ping: ImpulseResponse,
    pub done: ImpulseResponse,
    pub error: ImpulseResponse,
    /// Gap from the face edge to the beam's centre, px.
    pub ring_gap: f64,
    pub ring_width: f64,
    /// Focus light scale; multiplies head, tail, rest and spill.
    pub ring_glow: f64,
    pub focus: FocusResponse,
    /// Filament base color; alpha is ignored.
    pub ring_color: Color,
    /// Beam pace along the perimeter, px/s; zero skips the beam.
    pub ring_beam_speed: f64,
    /// How far the beam head's brightness wanders either side of its
    /// envelope, 0–1; zero is a steady head.
    pub ring_beam_noise: f64,
    /// How fast it wanders, in wanders per second.
    pub ring_beam_noise_hz: f64,
    /// Distance along the beam line over which the comet darkens, px;
    /// zero is no decay.
    pub ring_beam_decay: f64,
}

impl Default for ResolvedResponse {
    fn default() -> Self {
        Self {
            accent: AccentResponse::Ring,
            attention: AttentionResponse::RimOrbit,
            ping: ImpulseResponse::Ripple,
            done: ImpulseResponse::Sweep,
            error: ImpulseResponse::Flash,
            ring_gap: 8.,
            ring_width: 2.6,
            ring_glow: 1.,
            focus: FocusResponse::RingLight,
            ring_color: Color::from_rgba8_unpremul(0xcc, 0xcc, 0xff, 0xff),
            ring_beam_speed: 300.,
            ring_beam_noise: 0.,
            ring_beam_noise_hz: 3.,
            ring_beam_decay: 0.,
        }
    }
}

impl ResolvedResponse {
    fn with_overrides(base: Self, response: &Response) -> Self {
        Self {
            accent: response.accent.unwrap_or(base.accent),
            attention: response.attention.unwrap_or(base.attention),
            ping: response.ping.unwrap_or(base.ping),
            done: response.done.unwrap_or(base.done),
            error: response.error.unwrap_or(base.error),
            ring_gap: response.ring_gap.map_or(base.ring_gap, |x| x.0),
            ring_width: response.ring_width.map_or(base.ring_width, |x| x.0),
            ring_glow: response.ring_glow.map_or(base.ring_glow, |x| x.0),
            focus: response.focus.unwrap_or(base.focus),
            ring_color: response.ring_color.unwrap_or(base.ring_color),
            ring_beam_speed: response
                .ring_beam_speed
                .map_or(base.ring_beam_speed, |x| x.0),
            ring_beam_noise: response
                .ring_beam_noise
                .map_or(base.ring_beam_noise, |x| x.0),
            ring_beam_noise_hz: response
                .ring_beam_noise_hz
                .map_or(base.ring_beam_noise_hz, |x| x.0),
            ring_beam_decay: response
                .ring_beam_decay
                .map_or(base.ring_beam_decay, |x| x.0),
        }
    }

    pub fn attention_is_none(&self) -> bool {
        self.attention == AttentionResponse::None
    }

    /// Resolves an impulse kind to an opaque glass selector under the policy.
    pub fn impulse_selector(
        &self,
        kind: niri_ipc::ImpulseKind,
        policy: crate::SignalMotionPolicy,
    ) -> Option<u8> {
        use crate::SignalMotionPolicy as P;

        if policy == P::Off {
            return None;
        }
        let response = match kind {
            niri_ipc::ImpulseKind::Ping => self.ping,
            niri_ipc::ImpulseKind::Done => self.done,
            niri_ipc::ImpulseKind::Error => self.error,
        };
        let response = match (policy, response) {
            (P::Reduced, ImpulseResponse::Flash) => ImpulseResponse::Sweep,
            (_, response) => response,
        };
        match response {
            ImpulseResponse::None => None,
            response => Some(response as u8),
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
    #[knuffel(children(name = "response"))]
    pub responses: Vec<Response>,
}

/// `distortion <amount> scale=<scale>`.
///
/// The scale does nothing while the amplitude is zero, so it rides the node
/// it depends on rather than standing alone. This matches niri's own idiom
/// for the shape (`spring damping-ratio=1.0 stiffness=800`). It does not make
/// inert state unrepresentable — `distortion 0 scale=1.5` is still valid and
/// still does nothing — but a scale can no longer be written without naming
/// the amplitude it belongs to.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Distortion {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property)]
    pub scale: Option<FloatOrInt<0, 2>>,
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
    #[knuffel(child)]
    pub distortion: Option<Distortion>,
    #[knuffel(child, unwrap(argument))]
    pub anisotropic_blur: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub roughness: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub iridescence: Option<optics::iridescence::Iridescence>,
    #[knuffel(child)]
    pub aurora: Option<optics::aurora::Aurora>,
    #[knuffel(child)]
    pub noise: Option<optics::noise::Noise>,
    #[knuffel(child, unwrap(argument))]
    pub saturation: Option<optics::saturation::Saturation>,
    #[knuffel(child, unwrap(argument))]
    pub backdrop_blur: Option<bool>,
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
    /// Multiplier on the bend of the focus filament's light path. Prism's
    /// glass at ior 1.02 bends nothing visible; the filament travels the
    /// slab twice and is scattered, so its path is exaggerated.
    #[knuffel(child, unwrap(argument))]
    pub light_ior: Option<FloatOrInt<1, 12>>,
}

/// A material definition with every parameter resolved to a final value.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMaterial {
    pub name: String,
    pub glass: ResolvedGlass,
    pub responses: Vec<(String, ResolvedResponse)>,
}

impl ResolvedMaterial {
    /// The named response, or `default`. Names are validated at parse time.
    pub fn response(&self, name: Option<&str>) -> ResolvedResponse {
        let name = name.unwrap_or("default");
        self.responses
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, response)| *response)
            .expect("response names are validated at parse time")
    }
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
    pub anisotropic_blur: f64,
    pub roughness: f64,
    pub iridescence: optics::iridescence::ResolvedIridescence,
    pub aurora: optics::aurora::ResolvedAurora,
    /// Whether the material samples the blurred backdrop. Strength comes from
    /// the global `blur` block; `blur { off }` overrides this.
    pub backdrop_blur: bool,
    pub jelly_flex: f64,
    pub jelly_ripple: f64,
    pub bevel: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub noise: optics::noise::ResolvedNoise,
    pub saturation: optics::saturation::ResolvedSaturation,
    pub light_ior: f64,
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
            anisotropic_blur: 0.,
            roughness: 0.,
            iridescence: optics::iridescence::ResolvedIridescence::default(),
            aurora: optics::aurora::ResolvedAurora::default(),
            backdrop_blur: false,
            jelly_flex: 0.004,
            jelly_ripple: 0.06,
            bevel: 12.,
            offset_x: 6.,
            offset_y: 6.,
            noise: optics::noise::ResolvedNoise::default(),
            saturation: optics::saturation::ResolvedSaturation::default(),
            light_ior: 6.,
        }
    }
}

/// Specs for the parameters that live on the core (everything that is not
/// an optic), in the order the docs table lists them.
pub fn core_params() -> Vec<params::ParamSpec> {
    use params::{ParamKind, ParamSpec};

    let d = ResolvedGlass::default();
    vec![
        ParamSpec {
            node: "ior",
            kind: ParamKind::float::<FloatOrInt<1, 3>>(d.ior, "—"),
            write: |v| format!("ior {v}"),
            read: Some(|g| Some(g.ior)),
        },
        ParamSpec {
            node: "thickness",
            kind: ParamKind::float::<FloatOrInt<0, 200>>(d.thickness, "logical px"),
            write: |v| format!("thickness {v}"),
            read: Some(|g| Some(g.thickness)),
        },
        ParamSpec {
            node: "attenuation-color",
            kind: ParamKind::Color {
                default: d.attenuation_color,
            },
            write: |v| format!("attenuation-color \"{v}\""),
            read: None,
        },
        ParamSpec {
            node: "attenuation-distance",
            kind: ParamKind::float::<Positive<65535>>(d.attenuation_distance, "logical px"),
            write: |v| format!("attenuation-distance {v}"),
            read: Some(|g| Some(g.attenuation_distance)),
        },
        ParamSpec {
            node: "chromatic-aberration",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.chromatic_aberration, "—"),
            write: |v| format!("chromatic-aberration {v}"),
            read: Some(|g| Some(g.chromatic_aberration)),
        },
        ParamSpec {
            node: "distortion",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.distortion, "—"),
            write: |v| format!("distortion {v}"),
            read: Some(|g| Some(g.distortion)),
        },
        ParamSpec {
            node: "distortion scale=",
            kind: ParamKind::float::<FloatOrInt<0, 2>>(d.distortion_scale, "—"),
            write: |v| format!("distortion 0 scale={v}"),
            read: Some(|g| Some(g.distortion_scale)),
        },
        ParamSpec {
            node: "anisotropic-blur",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.anisotropic_blur, "—"),
            write: |v| format!("anisotropic-blur {v}"),
            read: Some(|g| Some(g.anisotropic_blur)),
        },
        ParamSpec {
            node: "roughness",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.roughness, "—"),
            write: |v| format!("roughness {v}"),
            read: Some(|g| Some(g.roughness)),
        },
        ParamSpec {
            node: "backdrop-blur",
            kind: ParamKind::Bool {
                default: d.backdrop_blur,
            },
            write: |v| format!("backdrop-blur {v}"),
            read: None,
        },
        ParamSpec {
            node: "jelly-flex",
            kind: ParamKind::float::<Milli<0, 20>>(d.jelly_flex, "—"),
            write: |v| format!("jelly-flex {v}"),
            read: Some(|g| Some(g.jelly_flex)),
        },
        ParamSpec {
            node: "jelly-ripple",
            kind: ParamKind::float::<Milli<0, 500>>(d.jelly_ripple, "—"),
            write: |v| format!("jelly-ripple {v}"),
            read: Some(|g| Some(g.jelly_ripple)),
        },
        ParamSpec {
            node: "bevel",
            kind: ParamKind::float::<FloatOrInt<0, 128>>(d.bevel, "logical px"),
            write: |v| format!("bevel {v}"),
            read: Some(|g| Some(g.bevel)),
        },
        ParamSpec {
            node: "light-ior",
            kind: ParamKind::float::<FloatOrInt<1, 12>>(d.light_ior, "—"),
            write: |v| format!("light-ior {v}"),
            read: Some(|g| Some(g.light_ior)),
        },
        ParamSpec {
            node: "offset-x",
            kind: ParamKind::float::<FloatOrInt<-64, 64>>(d.offset_x, "logical px"),
            write: |v| format!("offset-x {v}"),
            read: Some(|g| Some(g.offset_x)),
        },
        ParamSpec {
            node: "offset-y",
            kind: ParamKind::float::<FloatOrInt<-64, 64>>(d.offset_y, "logical px"),
            write: |v| format!("offset-y {v}"),
            read: Some(|g| Some(g.offset_y)),
        },
    ]
}

impl Material {
    /// Resolves the definition, applying the design §4 default for every
    /// omitted parameter.
    pub fn resolve(&self) -> ResolvedMaterial {
        let g = &self.glass;
        let d = ResolvedGlass::default();
        let responses = if self.responses.is_empty() {
            vec![(String::from("default"), ResolvedResponse::default())]
        } else {
            let default_block = self
                .responses
                .iter()
                .find(|response| response.name == "default")
                .expect("response blocks are validated before resolution");
            let default =
                ResolvedResponse::with_overrides(ResolvedResponse::default(), default_block);
            let mut responses = vec![(String::from("default"), default)];
            responses.extend(
                self.responses
                    .iter()
                    .filter(|response| response.name != "default")
                    .map(|response| {
                        (
                            response.name.clone(),
                            ResolvedResponse::with_overrides(default, response),
                        )
                    }),
            );
            responses
        };

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
                distortion: g.distortion.map_or(d.distortion, |x| x.amount.0),
                distortion_scale: g
                    .distortion
                    .and_then(|x| x.scale)
                    .map_or(d.distortion_scale, |x| x.0),
                anisotropic_blur: g.anisotropic_blur.map_or(d.anisotropic_blur, |x| x.0),
                roughness: g.roughness.map_or(d.roughness, |x| x.0),
                iridescence: optics::iridescence::resolve(g.iridescence),
                aurora: optics::aurora::resolve(g.aurora.as_ref()),
                backdrop_blur: g.backdrop_blur.unwrap_or(d.backdrop_blur),
                jelly_flex: g.jelly_flex.map_or(d.jelly_flex, |x| x.0),
                jelly_ripple: g.jelly_ripple.map_or(d.jelly_ripple, |x| x.0),
                bevel: g.bevel.map_or(d.bevel, |x| x.0),
                offset_x: g.offset_x.map_or(d.offset_x, |x| x.0),
                offset_y: g.offset_y.map_or(d.offset_y, |x| x.0),
                noise: optics::noise::resolve(g.noise),
                saturation: optics::saturation::resolve(g.saturation),
                light_ior: g.light_ior.map_or(d.light_ior, |x| x.0),
            },
            responses,
        }
    }

    /// Checks the cross-parameter rule and each optic's validation rules.
    ///
    /// `material_frame` derives the uniform inflation as
    /// `bevel - max(|offset-x|, |offset-y|)`. A negative inflation would put
    /// the slab inside the window on one side, which has no meaning, so the
    /// offsets are bounded by the bevel rather than clamped silently.
    pub(crate) fn validate(&self) -> Result<(), String> {
        // Optic rules run before resolution: `resolve` relies on them.
        if let Some(aurora) = &self.glass.aurora {
            aurora.validate()?;
        }
        let d = ResolvedGlass::default();
        let bevel = self.glass.bevel.map_or(d.bevel, |x| x.0);
        let offset_x = self.glass.offset_x.map_or(d.offset_x, |x| x.0);
        let offset_y = self.glass.offset_y.map_or(d.offset_y, |x| x.0);

        if !self.responses.is_empty()
            && !self
                .responses
                .iter()
                .any(|response| response.name == "default")
        {
            return Err(format!(
                "material {}: missing response \"default\"",
                self.name
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for response in &self.responses {
            if !seen.insert(response.name.as_str()) {
                return Err(format!("duplicate response: {}", response.name));
            }
        }
        let resolved = self.resolve();
        if resolved
            .responses
            .iter()
            .any(|(_, response)| response.ring_width <= 0.)
        {
            return Err(String::from("ring-width must be positive"));
        }
        if self
            .responses
            .iter()
            .any(|response| response.ring_drift_hz.is_some())
        {
            return Err(String::from(
                "ring-drift-hz was replaced by ring-beam-speed; see material-config.md",
            ));
        }
        if self
            .responses
            .iter()
            .any(|response| response.ring_sweep_ms.is_some())
        {
            return Err(String::from(
                "ring-sweep-ms was replaced by ring-beam-speed; see material-config.md",
            ));
        }
        if self
            .responses
            .iter()
            .any(|response| response.ring_inset.is_some())
        {
            return Err(String::from(
                "ring-inset was replaced by ring-gap; see material-config.md",
            ));
        }

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
