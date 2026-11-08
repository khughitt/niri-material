use crate::appearance::{BackgroundEffectRule, BlockOutFrom, Color, CornerRadius, ShadowRule};
use crate::material::{Material, MaterialRef, Milli};
use crate::oklab::color_to_oklab;
use crate::utils::RegexEq;
use crate::window_rule::PopupsRule;

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct LayerRule {
    #[knuffel(children(name = "match"))]
    pub matches: Vec<Match>,
    #[knuffel(children(name = "exclude"))]
    pub excludes: Vec<Match>,

    #[knuffel(child, unwrap(argument))]
    pub opacity: Option<f32>,
    #[knuffel(child, unwrap(argument))]
    pub block_out_from: Option<BlockOutFrom>,
    #[knuffel(child, default)]
    pub shadow: ShadowRule,
    #[knuffel(child)]
    pub geometry_corner_radius: Option<CornerRadius>,
    #[knuffel(child, unwrap(argument))]
    pub place_within_backdrop: Option<bool>,
    #[knuffel(child, unwrap(argument))]
    pub baba_is_float: Option<bool>,
    #[knuffel(child, default)]
    pub background_effect: BackgroundEffectRule,
    #[knuffel(child, default)]
    pub popups: PopupsRule,
    #[knuffel(child)]
    pub surface_remap: Option<SurfaceRemap>,
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct Match {
    #[knuffel(property, str)]
    pub namespace: Option<RegexEq>,
    #[knuffel(property)]
    pub at_startup: Option<bool>,
    #[knuffel(property, str)]
    pub layer: Option<niri_ipc::Layer>,
}

/// `surface-remap { … }`: recolor a layer surface's surface family to a
/// glass face and the output's held hue
/// (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §3, §5).
///
/// Every field but `hue-chroma` is required, and a later matching rule
/// replaces the whole node, so the node a surface gets is always one that was
/// validated at load.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct SurfaceRemap {
    #[knuffel(child)]
    pub surface: Color,
    #[knuffel(child)]
    pub container: Color,
    #[knuffel(child)]
    pub outline: Color,
    #[knuffel(child)]
    pub material: MaterialRef,
    #[knuffel(child, unwrap(argument))]
    pub hue_chroma: Option<Milli<0, 100>>,
}

/// The lightest source surface the remap accepts: a dark Noctalia scheme.
pub const MAX_SOURCE_SURFACE_L: f64 = 0.4;

/// Checks every `surface-remap` against the merged material table.
///
/// Unknown material names are reported by `validate_material_refs`, which
/// sees the same reference; a material whose own definition failed
/// validation is skipped here, since its error is already emitted.
pub fn validate_surface_remaps(rules: &[LayerRule], materials: &[Material]) -> Vec<String> {
    let mut errors = Vec::new();
    for remap in rules.iter().filter_map(|rule| rule.surface_remap.as_ref()) {
        let name = &remap.material.name;
        if remap.material.response.is_some() {
            errors.push(format!(
                "surface-remap: material \"{name}\" takes no response"
            ));
        }
        let l = color_to_oklab(remap.surface)[0];
        if l > MAX_SOURCE_SURFACE_L {
            errors.push(format!(
                "surface-remap: the source surface has Oklab L {l:.2}; \
                 the remap supports a dark Noctalia scheme (L up to 0.4)"
            ));
        }
        let Some(material) = materials.iter().find(|m| &m.name == name) else {
            continue;
        };
        if material.validate().is_err() {
            continue;
        }
        let glass = material.resolve().glass;
        for check in [
            crate::material::face::check_range,
            crate::material::face::check_effects,
        ] {
            if let Err(message) = check(name, &glass) {
                errors.push(message);
            }
        }
    }
    errors
}
