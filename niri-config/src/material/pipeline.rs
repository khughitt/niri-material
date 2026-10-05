//! The pipeline schema: sites, stages, and interactions as data.
//!
//! Design: prism `docs/specs/2026-10-04-pipeline-schema-design.md`,
//! Sections 1 and 2. The tables below are the renderer's statement of what
//! runs where; tests pin them to `all_params()`, `ORDER`, the response
//! block, and (in niri) the shader hook calls, and `resources/materials/
//! pipeline.json` is generated from them.

/// The value that leaves a site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Carrier {
    Texture,
    Normal,
    Linear,
    Light,
    Encoded,
}

/// Whether the order of two stages at a site changes the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Law {
    Sequence,
    Sum,
    Product,
    Coupled,
}

/// Where a site's result lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coverage {
    Backdrop,
    Glass,
    Window,
}

/// Cached per backdrop damage, or evaluated per glass fragment per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cost {
    Cached,
    Fragment,
}

/// Where a stage's parameters can vary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Output,
    Material,
    Window,
}

/// The shader program a hook call lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Program {
    Material,
    Effect,
    Postprocess,
}

/// A structural interaction between two stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Requires,
    Attenuates,
    Shadows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Site {
    pub id: &'static str,
    pub carrier: Carrier,
    pub law: Law,
    pub orderable: bool,
    pub coverage: Coverage,
    pub cost: Cost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpticRef {
    pub name: &'static str,
    pub hook: &'static str,
    pub program: Program,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selector {
    pub param: &'static str,
    pub variant: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage {
    pub id: &'static str,
    pub site: &'static str,
    pub scope: Scope,
    /// Parameters whose control belongs here; each parameter has one owner.
    pub owns: &'static [&'static str],
    /// Every parameter this stage's code consumes; a superset of `owns`.
    pub reads: &'static [&'static str],
    /// Response-block fields this stage reads, as the KDL spells them.
    pub responses: &'static [&'static str],
    pub optic: Option<OpticRef>,
    pub selector: Option<Selector>,
    pub animated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interaction {
    pub kind: Kind,
    pub from: &'static str,
    pub on: &'static str,
    /// The mechanism, ending with the rack's decision: `[expose]`,
    /// `[alternative]`, or `[drop]`.
    pub why: &'static str,
}

/// The four global `blur` block fields, owned by the source and
/// background-effect stages and inherited by the behind optics.
pub const BLUR_FIELDS: &[&str] = &[
    "blur passes",
    "blur offset",
    "blur noise",
    "blur saturation",
];

const fn site(
    id: &'static str,
    carrier: Carrier,
    law: Law,
    coverage: Coverage,
    cost: Cost,
) -> Site {
    Site {
        id,
        carrier,
        law,
        orderable: false,
        coverage,
        cost,
    }
}

pub static SITES: &[Site] = &[
    site(
        "source",
        Carrier::Texture,
        Law::Sequence,
        Coverage::Backdrop,
        Cost::Cached,
    ),
    site(
        "normal",
        Carrier::Normal,
        Law::Sequence,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "taps",
        Carrier::Linear,
        Law::Coupled,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "behind",
        Carrier::Linear,
        Law::Sequence,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "attenuation",
        Carrier::Linear,
        Law::Product,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "within",
        Carrier::Light,
        Law::Sum,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "specular",
        Carrier::Light,
        Law::Sequence,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "emissive",
        Carrier::Light,
        Law::Sum,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "encode",
        Carrier::Encoded,
        Law::Coupled,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "post",
        Carrier::Encoded,
        Law::Sequence,
        Coverage::Glass,
        Cost::Fragment,
    ),
    site(
        "background-effect",
        Carrier::Encoded,
        Law::Sequence,
        Coverage::Window,
        Cost::Fragment,
    ),
];

const fn stage(
    id: &'static str,
    site: &'static str,
    scope: Scope,
    owns: &'static [&'static str],
    reads: &'static [&'static str],
    responses: &'static [&'static str],
    optic: Option<OpticRef>,
    animated: bool,
) -> Stage {
    Stage {
        id,
        site,
        scope,
        owns,
        reads,
        responses,
        optic,
        selector: None,
        animated,
    }
}

const fn optic(name: &'static str, hook: &'static str) -> Option<OpticRef> {
    Some(OpticRef {
        name,
        hook,
        program: Program::Material,
    })
}

const RING_RESPONSES: &[&str] = &[
    "ring-gap",
    "ring-width",
    "ring-color",
    "ring-glow",
    "ring-rest",
    "ring-accent",
    "ring-beam-speed",
    "ring-beam-noise",
    "ring-beam-noise-hz",
    "ring-beam-decay",
    "focus",
    "accent",
    "attention",
];

/// Section 1 of the design, in pipeline order.
pub static STAGES: &[Stage] = &[
    stage(
        "blur",
        "source",
        Scope::Output,
        &["blur passes", "blur offset"],
        &["blur passes", "blur offset"],
        &[],
        None,
        false,
    ),
    stage(
        "prefilter",
        "source",
        Scope::Material,
        &["backdrop-blur", "roughness"],
        &["backdrop-blur", "roughness", "ior"],
        &[],
        None,
        false,
    ),
    stage(
        "slab",
        "normal",
        Scope::Material,
        &["bevel", "offset-x", "offset-y", "jelly-flex"],
        &["bevel", "offset-x", "offset-y", "jelly-flex", "thickness"],
        &[],
        None,
        true,
    ),
    stage(
        "distortion",
        "normal",
        Scope::Material,
        &["distortion", "distortion scale="],
        &["distortion", "distortion scale="],
        &[],
        None,
        false,
    ),
    stage(
        "ripple",
        "normal",
        Scope::Material,
        &["jelly-ripple"],
        &["jelly-ripple"],
        &[],
        None,
        true,
    ),
    stage(
        "refraction",
        "taps",
        Scope::Material,
        &["ior", "thickness"],
        &["ior", "thickness", "backdrop-blur", "roughness"],
        &[],
        None,
        false,
    ),
    stage(
        "fringing",
        "taps",
        Scope::Material,
        &["chromatic-aberration"],
        &[
            "chromatic-aberration",
            "ior",
            "thickness",
            "anisotropic-blur",
        ],
        &[],
        None,
        false,
    ),
    stage(
        "directional-blur",
        "taps",
        Scope::Material,
        &["anisotropic-blur"],
        &[
            "anisotropic-blur",
            "ior",
            "thickness",
            "chromatic-aberration",
        ],
        &[],
        None,
        false,
    ),
    stage(
        "saturation",
        "behind",
        Scope::Material,
        &["saturation"],
        &["saturation", "blur saturation", "backdrop-blur"],
        &[],
        optic("saturation", "behind"),
        false,
    ),
    stage(
        "noise",
        "behind",
        Scope::Material,
        &["noise", "noise type="],
        &["noise", "noise type=", "blur noise", "backdrop-blur"],
        &[],
        optic("noise", "behind"),
        false,
    ),
    stage(
        "tint",
        "attenuation",
        Scope::Material,
        &["attenuation-color", "attenuation-distance"],
        &["attenuation-color", "attenuation-distance", "thickness"],
        &["accent-tint", "accent"],
        None,
        false,
    ),
    stage(
        "ring",
        "within",
        Scope::Material,
        &["light-ior"],
        &[
            "light-ior",
            "ior",
            "thickness",
            "roughness",
            "chromatic-aberration",
        ],
        RING_RESPONSES,
        None,
        true,
    ),
    stage(
        "aurora",
        "within",
        Scope::Material,
        &["aurora", "aurora drift-hz", "aurora color"],
        &[
            "aurora",
            "aurora drift-hz",
            "aurora color",
            "ior",
            "light-ior",
            "thickness",
        ],
        &[],
        optic("aurora", "within"),
        true,
    ),
    stage(
        "glint",
        "specular",
        Scope::Material,
        &[],
        &["ior"],
        &["attention", "accent"],
        None,
        true,
    ),
    stage(
        "iridescence",
        "specular",
        Scope::Material,
        &["iridescence"],
        &["iridescence"],
        &[],
        optic("iridescence", "specular"),
        false,
    ),
    stage(
        "sweeps",
        "emissive",
        Scope::Material,
        &[],
        &[],
        &["ping", "done", "error"],
        None,
        true,
    ),
    stage(
        "encode",
        "encode",
        Scope::Material,
        &[],
        &[],
        &[],
        None,
        false,
    ),
    stage(
        "effect-saturation",
        "background-effect",
        Scope::Window,
        &["blur saturation"],
        &["blur saturation"],
        &[],
        None,
        false,
    ),
    stage(
        "effect-noise",
        "background-effect",
        Scope::Window,
        &["blur noise"],
        &["blur noise"],
        &[],
        None,
        false,
    ),
];

pub static INTERACTIONS: &[Interaction] = &[
    Interaction {
        kind: Kind::Attenuates,
        from: "refraction",
        on: "prefilter",
        why: "the prefilter level is roughness * clamp(ior * 2 - 2, 0, 1), so ior 1 flattens Blur while frosted backdrop still selects the blurred source [expose]",
    },
];

/// The selector rules (design Section 2): a selector names an `Enum`
/// parameter the stage owns or reads and one of its variants; across the
/// stages that select on one parameter, every variant selects exactly one
/// stage and every stage is the same optic. Checked on `STAGES` by a test,
/// and on fixtures so the negatives are proven while `STAGES` has none.
pub fn check_selectors(stages: &[Stage], specs: &[super::params::ParamSpec]) -> Result<(), String> {
    use super::params::ParamKind;
    use std::collections::BTreeMap;

    let mut by_param: BTreeMap<&str, Vec<&Stage>> = BTreeMap::new();
    for stage in stages {
        let Some(selector) = stage.selector else {
            continue;
        };
        let spec = specs
            .iter()
            .find(|spec| spec.node == selector.param)
            .ok_or_else(|| {
                format!(
                    "{}: selector names unknown parameter {}",
                    stage.id, selector.param
                )
            })?;
        let ParamKind::Enum { variants, .. } = &spec.kind else {
            return Err(format!(
                "{}: selector {} is not an enum parameter",
                stage.id, selector.param
            ));
        };
        if !variants.contains(&selector.variant) {
            return Err(format!(
                "{}: {} has no variant {}",
                stage.id, selector.param, selector.variant
            ));
        }
        if !stage.owns.contains(&selector.param) && !stage.reads.contains(&selector.param) {
            return Err(format!(
                "{}: selector parameter {} must be in owns or reads",
                stage.id, selector.param
            ));
        }
        if stage.optic.is_none() {
            return Err(format!(
                "{}: a selected stage must be an optic stage",
                stage.id
            ));
        }
        by_param.entry(selector.param).or_default().push(stage);
    }
    for (param, selected) in by_param {
        let spec = specs
            .iter()
            .find(|spec| spec.node == param)
            .expect("checked above");
        let ParamKind::Enum { variants, .. } = &spec.kind else {
            unreachable!()
        };
        for variant in *variants {
            let ids: Vec<&str> = selected
                .iter()
                .filter(|stage| stage.selector.map(|s| s.variant) == Some(*variant))
                .map(|stage| stage.id)
                .collect();
            if ids.len() > 1 {
                return Err(format!(
                    "{param}: variant {variant} selects two stages ({})",
                    ids.join(", ")
                ));
            }
        }
        let missing: Vec<&str> = variants
            .iter()
            .copied()
            .filter(|variant| {
                !selected
                    .iter()
                    .any(|stage| stage.selector.map(|s| s.variant) == Some(*variant))
            })
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{param}: variants {} select no stage",
                missing.join(", ")
            ));
        }
        let mut optics: Vec<&str> = selected
            .iter()
            .filter_map(|stage| stage.optic.map(|o| o.name))
            .collect();
        optics.sort_unstable();
        optics.dedup();
        if optics.len() > 1 {
            return Err(format!(
                "{param}: its stages belong to two optics ({})",
                optics.join(", ")
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::material::optics::ORDER;
    use crate::material::params::all_params;
    use crate::material::response_fields;

    fn all_nodes() -> HashSet<&'static str> {
        let mut nodes: HashSet<&'static str> = all_params().iter().map(|spec| spec.node).collect();
        nodes.extend(BLUR_FIELDS);
        nodes
    }

    #[test]
    fn every_parameter_is_owned_by_exactly_one_stage() {
        let mut owners: HashMap<&str, Vec<&str>> = HashMap::new();
        for stage in STAGES {
            for node in stage.owns {
                owners.entry(node).or_default().push(stage.id);
            }
        }
        let nodes = all_nodes();
        for node in &nodes {
            let who = owners.get(node).cloned().unwrap_or_default();
            assert_eq!(
                who.len(),
                1,
                "{node} must be owned by exactly one stage, owned by {who:?}"
            );
        }
        for (node, who) in &owners {
            assert!(
                nodes.contains(node),
                "{who:?} own {node}, which is not a parameter"
            );
        }
    }

    #[test]
    fn reads_cover_owns_and_name_only_parameters() {
        let nodes = all_nodes();
        for stage in STAGES {
            for node in stage.owns {
                assert!(
                    stage.reads.contains(node),
                    "{} must read what it owns ({node})",
                    stage.id
                );
            }
            for node in stage.reads {
                assert!(
                    nodes.contains(node),
                    "{} reads {node}, which is not a parameter",
                    stage.id
                );
            }
        }
    }

    #[test]
    fn responses_name_response_fields() {
        let fields: HashSet<&str> = response_fields().iter().copied().collect();
        for stage in STAGES {
            for field in stage.responses {
                assert!(
                    fields.contains(field),
                    "{} reads response field {field}, which Response lacks",
                    stage.id
                );
            }
        }
    }

    #[test]
    fn response_fields_match_the_struct() {
        // Exhaustive destructuring, no `..`: a field added to `Response` fails
        // to compile here until it is added to this pattern. That is a
        // compile-time review guard: whoever extends the pattern is looking at
        // the list below and at response_fields(), and decides whether the
        // field is live. Nothing forces the lists to grow on their own.
        let parsed: Vec<crate::material::Response> =
            knuffel::parse("pipeline.kdl", "response \"t\" { }\n").unwrap();
        let crate::material::Response {
            name: _,
            accent: _,
            attention: _,
            ping: _,
            done: _,
            error: _,
            ring_inset: _,
            ring_gap: _,
            ring_glow: _,
            ring_rest: _,
            ring_accent: _,
            accent_tint: _,
            ring_beam_speed: _,
            ring_beam_noise: _,
            ring_beam_noise_hz: _,
            ring_beam_decay: _,
            ring_width: _,
            focus: _,
            ring_color: _,
            ring_drift_hz: _,
            ring_sweep_ms: _,
        } = parsed.into_iter().next().unwrap();
        // The live fields in struct order, one decodable KDL sample each. The
        // three retired fields (ring-inset, ring-drift-hz, ring-sweep-ms) are
        // destructured above and deliberately absent here.
        let live: &[(&str, &str)] = &[
            ("accent", "accent \"ring\""),
            ("attention", "attention \"none\""),
            ("ping", "ping \"none\""),
            ("done", "done \"none\""),
            ("error", "error \"none\""),
            ("ring-gap", "ring-gap 4"),
            ("ring-glow", "ring-glow 1"),
            ("ring-rest", "ring-rest 0.5"),
            ("ring-accent", "ring-accent 1"),
            ("accent-tint", "accent-tint 0.5"),
            ("ring-beam-speed", "ring-beam-speed 600"),
            ("ring-beam-noise", "ring-beam-noise 0.2"),
            ("ring-beam-noise-hz", "ring-beam-noise-hz 2"),
            ("ring-beam-decay", "ring-beam-decay 2000"),
            ("ring-width", "ring-width 2"),
            ("focus", "focus \"ring-light\""),
            ("ring-color", "ring-color \"#ffffff\""),
        ];
        let listed: Vec<&str> = live.iter().map(|(field, _)| *field).collect();
        assert_eq!(
            response_fields(),
            listed.as_slice(),
            "response_fields() lists the live Response fields in struct order"
        );
        for (field, line) in live {
            let kdl = format!("response \"t\" {{ {line}; }}\n");
            let parsed: Result<Vec<crate::material::Response>, _> =
                knuffel::parse("pipeline.kdl", &kdl);
            assert!(
                parsed.is_ok(),
                "{field}: sample `{line}` did not decode: {:?}",
                parsed.err()
            );
        }
    }

    #[test]
    fn stages_are_grouped_by_site_in_site_order() {
        let site_index: HashMap<&str, usize> =
            SITES.iter().enumerate().map(|(i, s)| (s.id, i)).collect();
        let mut last = 0;
        let mut last_id = "";
        for stage in STAGES {
            let index = *site_index
                .get(stage.site)
                .unwrap_or_else(|| panic!("{} names unknown site {}", stage.id, stage.site));
            assert!(
                index >= last,
                "stage {} (site {}) comes after stage {last_id}, whose site is later",
                stage.id,
                stage.site
            );
            last = index;
            last_id = stage.id;
        }
        let ids: HashSet<&str> = STAGES.iter().map(|s| s.id).collect();
        assert_eq!(ids.len(), STAGES.len(), "stage ids are unique");
        let sites: HashSet<&str> = SITES.iter().map(|s| s.id).collect();
        assert_eq!(sites.len(), SITES.len(), "site ids are unique");
    }

    #[test]
    fn optic_stages_follow_the_registry_within_each_program_and_site() {
        for name in ORDER {
            assert!(
                STAGES
                    .iter()
                    .any(|s| s.optic.map(|o| o.name) == Some(*name)),
                "optic {name} has no stage"
            );
        }
        let mut pairs = HashSet::new();
        for stage in STAGES {
            if let Some(optic) = stage.optic {
                assert!(
                    ORDER.contains(&optic.name),
                    "{}: optic {} is not in ORDER",
                    stage.id,
                    optic.name
                );
                assert!(
                    pairs.insert((optic.name, optic.hook)),
                    "{}: ({}, {}) appears twice",
                    stage.id,
                    optic.name,
                    optic.hook
                );
            }
        }
        // Within one (program, site), optic stages keep ORDER relative to one another.
        let mut groups: HashMap<(Program, &str), Vec<usize>> = HashMap::new();
        for stage in STAGES {
            if let Some(optic) = stage.optic {
                let rank = ORDER.iter().position(|n| *n == optic.name).unwrap();
                groups
                    .entry((optic.program, stage.site))
                    .or_default()
                    .push(rank);
            }
        }
        for ((program, site), ranks) in groups {
            let mut sorted = ranks.clone();
            sorted.sort_unstable();
            assert_eq!(
                ranks, sorted,
                "optic stages at ({program:?}, {site}) are out of ORDER"
            );
        }
    }

    #[test]
    fn todays_stages_pass_the_selector_rules() {
        check_selectors(STAGES, &all_params()).unwrap();
    }

    /// A stage selected by `noise type=`; the fixtures below break one rule each.
    const fn selected(id: &'static str, variant: &'static str, optic_name: &'static str) -> Stage {
        Stage {
            id,
            site: "behind",
            scope: Scope::Material,
            owns: &[],
            reads: &["noise type="],
            responses: &[],
            optic: Some(OpticRef {
                name: optic_name,
                hook: "behind",
                program: Program::Material,
            }),
            selector: Some(Selector {
                param: "noise type=",
                variant,
            }),
            animated: false,
        }
    }

    #[test]
    fn selector_rules_refuse_each_defect() {
        let specs = all_params();
        let complete = [
            selected("a", "white", "noise"),
            selected("b", "fine", "noise"),
            selected("c", "lightness", "noise"),
        ];
        check_selectors(&complete, &specs).unwrap();

        let missing = [selected("a", "white", "noise")];
        assert_eq!(
            check_selectors(&missing, &specs).unwrap_err(),
            "noise type=: variants fine, lightness select no stage"
        );

        let twice = [
            selected("a", "white", "noise"),
            selected("b", "white", "noise"),
            selected("c", "fine", "noise"),
            selected("d", "lightness", "noise"),
        ];
        assert_eq!(
            check_selectors(&twice, &specs).unwrap_err(),
            "noise type=: variant white selects two stages (a, b)"
        );

        let mixed = [
            selected("a", "white", "noise"),
            selected("b", "fine", "saturation"),
            selected("c", "lightness", "noise"),
        ];
        assert_eq!(
            check_selectors(&mixed, &specs).unwrap_err(),
            "noise type=: its stages belong to two optics (noise, saturation)"
        );

        let mut unknown = selected("a", "coarse", "noise");
        unknown.id = "a";
        assert_eq!(
            check_selectors(&[unknown], &specs).unwrap_err(),
            "a: noise type= has no variant coarse"
        );

        let mut not_enum = selected("a", "white", "noise");
        not_enum.selector = Some(Selector {
            param: "ior",
            variant: "white",
        });
        not_enum.reads = &["ior"];
        assert_eq!(
            check_selectors(&[not_enum], &specs).unwrap_err(),
            "a: selector ior is not an enum parameter"
        );

        let mut unread = selected("a", "white", "noise");
        unread.reads = &[];
        assert_eq!(
            check_selectors(&[unread], &specs).unwrap_err(),
            "a: selector parameter noise type= must be in owns or reads"
        );

        let mut plain = selected("a", "white", "noise");
        plain.optic = None;
        assert_eq!(
            check_selectors(&[plain], &specs).unwrap_err(),
            "a: a selected stage must be an optic stage"
        );
    }

    #[test]
    fn interactions_name_two_distinct_stages() {
        let ids: HashSet<&str> = STAGES.iter().map(|s| s.id).collect();
        for edge in INTERACTIONS {
            assert!(
                ids.contains(edge.from),
                "interaction from unknown stage {}",
                edge.from
            );
            assert!(
                ids.contains(edge.on),
                "interaction on unknown stage {}",
                edge.on
            );
            assert_ne!(edge.from, edge.on, "an interaction needs two stages");
            assert!(
                edge.why.ends_with("[expose]")
                    || edge.why.ends_with("[alternative]")
                    || edge.why.ends_with("[drop]"),
                "{} -> {}: why must end with a [decision]",
                edge.from,
                edge.on
            );
        }
    }

    #[test]
    fn todays_shape() {
        assert_eq!(
            SITES.iter().map(|s| s.id).collect::<Vec<_>>(),
            [
                "source",
                "normal",
                "taps",
                "behind",
                "attenuation",
                "within",
                "specular",
                "emissive",
                "encode",
                "post",
                "background-effect",
            ]
        );
        assert!(
            SITES.iter().all(|s| !s.orderable),
            "no site is orderable yet"
        );
        let behind: Vec<&str> = STAGES
            .iter()
            .filter(|s| s.site == "behind")
            .map(|s| s.id)
            .collect();
        assert_eq!(behind, ["saturation", "noise"]);
        assert!(
            STAGES.iter().all(|s| s.selector.is_none()),
            "no selector yet"
        );
        assert_eq!(INTERACTIONS.len(), 1);
        assert_eq!(
            (
                INTERACTIONS[0].kind,
                INTERACTIONS[0].from,
                INTERACTIONS[0].on
            ),
            (Kind::Attenuates, "refraction", "prefilter")
        );
        let refraction = STAGES.iter().find(|s| s.id == "refraction").unwrap();
        assert!(
            refraction.owns.contains(&"thickness"),
            "refraction owns thickness (its Depth control)"
        );
        let slab = STAGES.iter().find(|s| s.id == "slab").unwrap();
        assert!(slab.reads.contains(&"thickness") && !slab.owns.contains(&"thickness"));
    }
}
