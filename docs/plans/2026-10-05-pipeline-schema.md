# Pipeline Schema (renderer side) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Write the material pipeline's structure (sites, stages, interactions) as Rust tables, pin them to the parameter specs, the optics registry, the response block, and the shader hook calls, and generate `resources/materials/pipeline.json` from them.

**Architecture:** `niri-config/src/material/pipeline.rs` holds plain statics and their serde shapes; `response_fields()` sits beside `Response` in `material/mod.rs`. niri-config tests pin the tables to `all_params()`, `ORDER`, and the response fields and keep the generated JSON fresh under the existing `MATERIAL_DOCS_UPDATE=1` convention. niri tests pin optic stages to the hook calls in each shader program and each optic's GLSL reads to its stage's `reads`. No renderer behaviour changes.

**Tech Stack:** Rust 2021 in the `niri-config` and `niri` crates, `serde` and `serde_json` (workspace dependencies), nextest through `just test-one` and `just test-fast`.

**Spec:** prism `docs/specs/2026-10-04-pipeline-schema-design.md`, Sections 1 and 2 (the tables below copy its Section 1 verbatim). The prism side is prism's `docs/plans/2026-10-05-pipeline-schema.md`; its Task 1 vendors the file Task 3 here produces.

## Global Constraints

- `just test-one -p niri-config <filter>` and `just test-one -p niri <filter>` while editing; `just test-fast` before each commit; never call cargo test directly.
- Staging order for every commit: `git add <files>`, then `just upstream-report`, then `git add docs/materials/upstream-divergence.md`, then commit (the pre-commit hook compares the report against the staged tree).
- `cargo fmt --all` before each commit; `just check` runs clippy with `--all-targets`, so test code must be clippy-clean too.
- The schema describes the pipeline as built: no stage, placement, or interaction for code that does not exist. `orderable` is `false` on every site.
- `owns` partitions the 23 `all_params()` nodes plus the four global `blur` fields exactly once; `reads` is a superset of `owns` on every stage.
- Renderer output is byte-identical before and after this plan: no shader or uniform change.
- `tasks start <step id>` before each task, `tasks done <step id>` in the task's commit; `tasks check` before every commit.
- Conventional commits, no attribution trailers.

## Review Focus

1. A new `ParamSpec` added to `core_params()` without a stage must fail the ownership test with the node named, not pass because the test only iterates stages (Task 1 test `every_parameter_is_owned_by_exactly_one_stage` iterates `all_params()` and reports the unowned node).
2. A new field on `Response` that is not added to `response_fields()` must be caught at compile time: the test destructures `Response` exhaustively, so the pattern stops compiling until the field is listed, and the list must then equal `response_fields()` (Task 1 test `response_fields_match_the_struct`, proven in Step 4b).
3. A stage whose `optic` names a hook the program does not call (`noise_post` before the code exists, or a call left in a comment) must fail the per-program pin; the pin counts actual calls of `name_hook(` after the `// ---- main` marker with comments stripped and definitions excluded, and requires exactly one (Task 2).
4. `MATERIAL_DOCS_UPDATE=1` must write the JSON with a trailing newline and stable key order, or every regeneration churns the vendored copy (Task 3 asserts the rendered string ends with `\n` and that two renders are equal).
5. A stage listed before its site's turn (a `behind` stage between two `source` stages) must fail the site-order test with both stage ids named (Task 1 test `stages_are_grouped_by_site_in_site_order`).

---

### Task 1: The tables and their config-side pins

**Files:**
- Create: `niri-config/src/material/pipeline.rs`
- Modify: `niri-config/src/material/mod.rs` (add `pub mod pipeline;` after `pub mod params;` at line 16; add `response_fields()` after the `Response` struct, which ends at line 320)
- Test: inside `niri-config/src/material/pipeline.rs` (`mod tests`)

**Interfaces:**
- Produces: `niri_config::material::pipeline::{Carrier, Law, Coverage, Cost, Scope, Program, Kind, Site, OpticRef, Selector, Stage, Interaction, SITES, STAGES, INTERACTIONS, BLUR_FIELDS}` and `niri_config::material::response_fields() -> &'static [&'static str]`. Task 2 reads `STAGES` from the niri crate; Task 3 serializes the three statics.

- [ ] **Step 1: Write the failing tests**

Create `niri-config/src/material/pipeline.rs` with only the test module first (the items it names do not exist yet):

```rust
//! The pipeline schema: sites, stages, and interactions as data.
//!
//! Design: prism `docs/specs/2026-10-04-pipeline-schema-design.md`,
//! Sections 1 and 2. The tables below are the renderer's statement of what
//! runs where; tests pin them to `all_params()`, `ORDER`, the response
//! block, and (in niri) the shader hook calls, and `resources/materials/
//! pipeline.json` is generated from them.

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::material::optics::ORDER;
    use crate::material::params::{all_params, ParamKind};
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
            assert_eq!(who.len(), 1, "{node} must be owned by exactly one stage, owned by {who:?}");
        }
        for (node, who) in &owners {
            assert!(nodes.contains(node), "{who:?} own {node}, which is not a parameter");
        }
    }

    #[test]
    fn reads_cover_owns_and_name_only_parameters() {
        let nodes = all_nodes();
        for stage in STAGES {
            for node in stage.owns {
                assert!(stage.reads.contains(node), "{} must read what it owns ({node})", stage.id);
            }
            for node in stage.reads {
                assert!(nodes.contains(node), "{} reads {node}, which is not a parameter", stage.id);
            }
        }
    }

    #[test]
    fn responses_name_response_fields() {
        let fields: HashSet<&str> = response_fields().iter().copied().collect();
        for stage in STAGES {
            for field in stage.responses {
                assert!(fields.contains(field), "{} reads response field {field}, which Response lacks", stage.id);
            }
        }
    }

    #[test]
    fn response_fields_match_the_struct() {
        // Exhaustive destructuring, no `..`: a field added to `Response` fails
        // to compile here until it is added to this pattern, and the list
        // below must then grow with it or the equality fails.
        let parsed: Vec<crate::material::Response> = knuffel::parse("pipeline.kdl", "response \"t\" { }\n").unwrap();
        let crate::material::Response {
            name: _, accent: _, attention: _, ping: _, done: _, error: _,
            ring_inset: _, ring_gap: _, ring_glow: _, ring_rest: _, ring_accent: _, accent_tint: _,
            ring_beam_speed: _, ring_beam_noise: _, ring_beam_noise_hz: _, ring_beam_decay: _,
            ring_width: _, focus: _, ring_color: _, ring_drift_hz: _, ring_sweep_ms: _,
        } = parsed.into_iter().next().unwrap();
        // The live fields in struct order, one decodable KDL sample each. The
        // three retired fields (ring-inset, ring-drift-hz, ring-sweep-ms) are
        // destructured above and deliberately absent here.
        let live: &[(&str, &str)] = &[
            ("accent", "accent \"ring-tint\""),
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
        assert_eq!(response_fields(), listed.as_slice(), "response_fields() lists the live Response fields in struct order");
        for (field, line) in live {
            let kdl = format!("response \"t\" {{ {line}; }}\n");
            let parsed: Result<Vec<crate::material::Response>, _> = knuffel::parse("pipeline.kdl", &kdl);
            assert!(parsed.is_ok(), "{field}: sample `{line}` did not decode: {:?}", parsed.err());
        }
    }

    #[test]
    fn stages_are_grouped_by_site_in_site_order() {
        let site_index: HashMap<&str, usize> = SITES.iter().enumerate().map(|(i, s)| (s.id, i)).collect();
        let mut last = 0;
        let mut last_id = "";
        for stage in STAGES {
            let index = *site_index.get(stage.site).unwrap_or_else(|| panic!("{} names unknown site {}", stage.id, stage.site));
            assert!(index >= last, "stage {} (site {}) comes after stage {last_id}, whose site is later", stage.id, stage.site);
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
            assert!(STAGES.iter().any(|s| s.optic.map(|o| o.name) == Some(*name)), "optic {name} has no stage");
        }
        let mut pairs = HashSet::new();
        for stage in STAGES {
            if let Some(optic) = stage.optic {
                assert!(ORDER.contains(&optic.name), "{}: optic {} is not in ORDER", stage.id, optic.name);
                assert!(pairs.insert((optic.name, optic.hook)), "{}: ({}, {}) appears twice", stage.id, optic.name, optic.hook);
            }
        }
        // Within one (program, site), optic stages keep ORDER relative to one another.
        let mut groups: HashMap<(Program, &str), Vec<usize>> = HashMap::new();
        for stage in STAGES {
            if let Some(optic) = stage.optic {
                let rank = ORDER.iter().position(|n| *n == optic.name).unwrap();
                groups.entry((optic.program, stage.site)).or_default().push(rank);
            }
        }
        for ((program, site), ranks) in groups {
            let mut sorted = ranks.clone();
            sorted.sort_unstable();
            assert_eq!(ranks, sorted, "optic stages at ({program:?}, {site}) are out of ORDER");
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
            optic: Some(OpticRef { name: optic_name, hook: "behind", program: Program::Material }),
            selector: Some(Selector { param: "noise type=", variant }),
            animated: false,
        }
    }

    #[test]
    fn selector_rules_refuse_each_defect() {
        let specs = all_params();
        let complete = [selected("a", "white", "noise"), selected("b", "fine", "noise"), selected("c", "lightness", "noise")];
        check_selectors(&complete, &specs).unwrap();

        let missing = [selected("a", "white", "noise")];
        assert_eq!(check_selectors(&missing, &specs).unwrap_err(), "noise type=: variants fine, lightness select no stage");

        let twice = [selected("a", "white", "noise"), selected("b", "white", "noise"), selected("c", "fine", "noise"), selected("d", "lightness", "noise")];
        assert_eq!(check_selectors(&twice, &specs).unwrap_err(), "noise type=: variant white selects two stages (a, b)");

        let mixed = [selected("a", "white", "noise"), selected("b", "fine", "saturation"), selected("c", "lightness", "noise")];
        assert_eq!(check_selectors(&mixed, &specs).unwrap_err(), "noise type=: its stages belong to two optics (noise, saturation)");

        let mut unknown = selected("a", "coarse", "noise");
        unknown.id = "a";
        assert_eq!(check_selectors(&[unknown], &specs).unwrap_err(), "a: noise type= has no variant coarse");

        let mut not_enum = selected("a", "white", "noise");
        not_enum.selector = Some(Selector { param: "ior", variant: "white" });
        not_enum.reads = &["ior"];
        assert_eq!(check_selectors(&[not_enum], &specs).unwrap_err(), "a: selector ior is not an enum parameter");

        let mut unread = selected("a", "white", "noise");
        unread.reads = &[];
        assert_eq!(check_selectors(&[unread], &specs).unwrap_err(), "a: selector parameter noise type= must be in owns or reads");

        let mut plain = selected("a", "white", "noise");
        plain.optic = None;
        assert_eq!(check_selectors(&[plain], &specs).unwrap_err(), "a: a selected stage must be an optic stage");
    }

    #[test]
    fn interactions_name_two_distinct_stages() {
        let ids: HashSet<&str> = STAGES.iter().map(|s| s.id).collect();
        for edge in INTERACTIONS {
            assert!(ids.contains(edge.from), "interaction from unknown stage {}", edge.from);
            assert!(ids.contains(edge.on), "interaction on unknown stage {}", edge.on);
            assert_ne!(edge.from, edge.on, "an interaction needs two stages");
            assert!(edge.why.ends_with("[expose]") || edge.why.ends_with("[alternative]") || edge.why.ends_with("[drop]"),
                "{} -> {}: why must end with a [decision]", edge.from, edge.on);
        }
    }

    #[test]
    fn todays_shape() {
        assert_eq!(SITES.iter().map(|s| s.id).collect::<Vec<_>>(), [
            "source", "normal", "taps", "behind", "attenuation", "within", "specular", "emissive", "encode", "post", "background-effect",
        ]);
        assert!(SITES.iter().all(|s| !s.orderable), "no site is orderable yet");
        let behind: Vec<&str> = STAGES.iter().filter(|s| s.site == "behind").map(|s| s.id).collect();
        assert_eq!(behind, ["saturation", "noise"]);
        assert!(STAGES.iter().all(|s| s.selector.is_none()), "no selector yet");
        assert_eq!(INTERACTIONS.len(), 1);
        assert_eq!((INTERACTIONS[0].kind, INTERACTIONS[0].from, INTERACTIONS[0].on), (Kind::Attenuates, "refraction", "prefilter"));
        let refraction = STAGES.iter().find(|s| s.id == "refraction").unwrap();
        assert!(refraction.owns.contains(&"thickness"), "refraction owns thickness (its Depth control)");
        let slab = STAGES.iter().find(|s| s.id == "slab").unwrap();
        assert!(slab.reads.contains(&"thickness") && !slab.owns.contains(&"thickness"));
    }
}
```

Register the module: in `niri-config/src/material/mod.rs` after line 16 (`pub mod params;`) add `pub mod pipeline;`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri-config pipeline`
Expected: compile errors naming `SITES`, `STAGES`, `INTERACTIONS`, `BLUR_FIELDS`, `Program`, `Kind`, `response_fields`.

- [ ] **Step 3: Write the tables**

Above the test module in `pipeline.rs`:

```rust
/// The value that leaves a site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Carrier { Texture, Normal, Linear, Light, Encoded }

/// Whether the order of two stages at a site changes the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Law { Sequence, Sum, Product, Coupled }

/// Where a site's result lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coverage { Backdrop, Glass, Window }

/// Cached per backdrop damage, or evaluated per glass fragment per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cost { Cached, Fragment }

/// Where a stage's parameters can vary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope { Output, Material, Window }

/// The shader program a hook call lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Program { Material, Effect, Postprocess }

/// A structural interaction between two stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind { Requires, Attenuates, Shadows }

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
pub const BLUR_FIELDS: &[&str] = &["blur passes", "blur offset", "blur noise", "blur saturation"];

const fn site(id: &'static str, carrier: Carrier, law: Law, coverage: Coverage, cost: Cost) -> Site {
    Site { id, carrier, law, orderable: false, coverage, cost }
}

pub static SITES: &[Site] = &[
    site("source", Carrier::Texture, Law::Sequence, Coverage::Backdrop, Cost::Cached),
    site("normal", Carrier::Normal, Law::Sequence, Coverage::Glass, Cost::Fragment),
    site("taps", Carrier::Linear, Law::Coupled, Coverage::Glass, Cost::Fragment),
    site("behind", Carrier::Linear, Law::Sequence, Coverage::Glass, Cost::Fragment),
    site("attenuation", Carrier::Linear, Law::Product, Coverage::Glass, Cost::Fragment),
    site("within", Carrier::Light, Law::Sum, Coverage::Glass, Cost::Fragment),
    site("specular", Carrier::Light, Law::Sequence, Coverage::Glass, Cost::Fragment),
    site("emissive", Carrier::Light, Law::Sum, Coverage::Glass, Cost::Fragment),
    site("encode", Carrier::Encoded, Law::Coupled, Coverage::Glass, Cost::Fragment),
    site("post", Carrier::Encoded, Law::Sequence, Coverage::Glass, Cost::Fragment),
    site("background-effect", Carrier::Encoded, Law::Sequence, Coverage::Window, Cost::Fragment),
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
    Stage { id, site, scope, owns, reads, responses, optic, selector: None, animated }
}

const fn optic(name: &'static str, hook: &'static str) -> Option<OpticRef> {
    Some(OpticRef { name, hook, program: Program::Material })
}

const RING_RESPONSES: &[&str] = &[
    "ring-gap", "ring-width", "ring-color", "ring-glow", "ring-rest", "ring-accent",
    "ring-beam-speed", "ring-beam-noise", "ring-beam-noise-hz", "ring-beam-decay",
    "focus", "accent", "attention",
];

/// Section 1 of the design, in pipeline order.
pub static STAGES: &[Stage] = &[
    stage("blur", "source", Scope::Output, &["blur passes", "blur offset"], &["blur passes", "blur offset"], &[], None, false),
    stage("prefilter", "source", Scope::Material, &["backdrop-blur", "roughness"], &["backdrop-blur", "roughness", "ior"], &[], None, false),
    stage("slab", "normal", Scope::Material, &["bevel", "offset-x", "offset-y", "jelly-flex"], &["bevel", "offset-x", "offset-y", "jelly-flex", "thickness"], &[], None, true),
    stage("distortion", "normal", Scope::Material, &["distortion", "distortion scale="], &["distortion", "distortion scale="], &[], None, false),
    stage("ripple", "normal", Scope::Material, &["jelly-ripple"], &["jelly-ripple"], &[], None, true),
    stage("refraction", "taps", Scope::Material, &["ior", "thickness"], &["ior", "thickness", "backdrop-blur", "roughness"], &[], None, false),
    stage("fringing", "taps", Scope::Material, &["chromatic-aberration"], &["chromatic-aberration", "ior", "thickness", "anisotropic-blur"], &[], None, false),
    stage("directional-blur", "taps", Scope::Material, &["anisotropic-blur"], &["anisotropic-blur", "ior", "thickness", "chromatic-aberration"], &[], None, false),
    stage("saturation", "behind", Scope::Material, &["saturation"], &["saturation", "blur saturation", "backdrop-blur"], &[], optic("saturation", "behind"), false),
    stage("noise", "behind", Scope::Material, &["noise", "noise type="], &["noise", "noise type=", "blur noise", "backdrop-blur"], &[], optic("noise", "behind"), false),
    stage("tint", "attenuation", Scope::Material, &["attenuation-color", "attenuation-distance"], &["attenuation-color", "attenuation-distance", "thickness"], &["accent-tint", "accent"], None, false),
    stage("ring", "within", Scope::Material, &["light-ior"], &["light-ior", "ior", "thickness", "roughness", "chromatic-aberration"], RING_RESPONSES, None, true),
    stage("aurora", "within", Scope::Material, &["aurora", "aurora drift-hz", "aurora color"], &["aurora", "aurora drift-hz", "aurora color", "ior", "light-ior", "thickness"], &[], optic("aurora", "within"), true),
    stage("glint", "specular", Scope::Material, &[], &["ior"], &["attention", "accent"], None, true),
    stage("iridescence", "specular", Scope::Material, &["iridescence"], &["iridescence"], &[], optic("iridescence", "specular"), false),
    stage("sweeps", "emissive", Scope::Material, &[], &[], &["ping", "done", "error"], None, true),
    stage("encode", "encode", Scope::Material, &[], &[], &[], None, false),
    stage("effect-saturation", "background-effect", Scope::Window, &["blur saturation"], &["blur saturation"], &[], None, false),
    stage("effect-noise", "background-effect", Scope::Window, &["blur noise"], &["blur noise"], &[], None, false),
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
    use std::collections::BTreeMap;
    use super::params::ParamKind;

    let mut by_param: BTreeMap<&str, Vec<&Stage>> = BTreeMap::new();
    for stage in stages {
        let Some(selector) = stage.selector else { continue };
        let spec = specs
            .iter()
            .find(|spec| spec.node == selector.param)
            .ok_or_else(|| format!("{}: selector names unknown parameter {}", stage.id, selector.param))?;
        let ParamKind::Enum { variants, .. } = &spec.kind else {
            return Err(format!("{}: selector {} is not an enum parameter", stage.id, selector.param));
        };
        if !variants.contains(&selector.variant) {
            return Err(format!("{}: {} has no variant {}", stage.id, selector.param, selector.variant));
        }
        if !stage.owns.contains(&selector.param) && !stage.reads.contains(&selector.param) {
            return Err(format!("{}: selector parameter {} must be in owns or reads", stage.id, selector.param));
        }
        if stage.optic.is_none() {
            return Err(format!("{}: a selected stage must be an optic stage", stage.id));
        }
        by_param.entry(selector.param).or_default().push(stage);
    }
    for (param, selected) in by_param {
        let spec = specs.iter().find(|spec| spec.node == param).expect("checked above");
        let ParamKind::Enum { variants, .. } = &spec.kind else { unreachable!() };
        for variant in *variants {
            let ids: Vec<&str> = selected
                .iter()
                .filter(|stage| stage.selector.map(|s| s.variant) == Some(*variant))
                .map(|stage| stage.id)
                .collect();
            if ids.len() > 1 {
                return Err(format!("{param}: variant {variant} selects two stages ({})", ids.join(", ")));
            }
        }
        let missing: Vec<&str> = variants
            .iter()
            .copied()
            .filter(|variant| !selected.iter().any(|stage| stage.selector.map(|s| s.variant) == Some(*variant)))
            .collect();
        if !missing.is_empty() {
            return Err(format!("{param}: variants {} select no stage", missing.join(", ")));
        }
        let mut optics: Vec<&str> = selected.iter().filter_map(|stage| stage.optic.map(|o| o.name)).collect();
        optics.sort_unstable();
        optics.dedup();
        if optics.len() > 1 {
            return Err(format!("{param}: its stages belong to two optics ({})", optics.join(", ")));
        }
    }
    Ok(())
}
```

In `niri-config/src/material/mod.rs`, after the `Response` struct (its closing brace is at line 320), add:

```rust
/// The KDL names of every live `Response` field, in struct order. Retired
/// fields (`ring-inset`, `ring-drift-hz`, `ring-sweep-ms`) are decoded only
/// to name their replacement and are not listed. `pipeline::STAGES` names
/// these in `responses`; a test there decodes a sample per entry and pins the
/// count, so adding a field here without adding it there fails.
pub fn response_fields() -> &'static [&'static str] {
    &[
        "accent", "attention", "ping", "done", "error",
        "ring-gap", "ring-glow", "ring-rest", "ring-accent", "accent-tint",
        "ring-beam-speed", "ring-beam-noise", "ring-beam-noise-hz", "ring-beam-decay",
        "ring-width", "focus", "ring-color",
    ]
}
```

If `all_params()` turns out to carry a node this plan's `owns` lists do not (the spec counted 23 nodes: `backdrop-blur`, `roughness`, `bevel`, `offset-x`, `offset-y`, `jelly-flex`, `distortion`, `distortion scale=`, `jelly-ripple`, `ior`, `thickness`, `chromatic-aberration`, `anisotropic-blur`, `saturation`, `noise`, `noise type=`, `attenuation-color`, `attenuation-distance`, `light-ior`, `aurora`, `aurora drift-hz`, `aurora color`, `iridescence`), the ownership test names it; add it to the stage whose code reads it as its control, following the rule that ownership follows the control, and record the addition in the task's notes. Do not add an exemption.

The `accent` response sample in `response_fields_match_the_struct` uses `"ring-tint"`; if `AccentResponse` names its variants differently, read `response_from_str!(AccentResponse, …)` in `material/mod.rs` and use a listed name. The same holds for `attention`, the impulse responses, and `focus`.

- [ ] **Step 4: Run the tests to see them pass**

Run: `just test-one -p niri-config pipeline`
Expected: PASS, 10 tests.

- [ ] **Step 4b: Prove the response pin bites**

Temporarily add `#[knuffel(child, unwrap(argument))] pub ring_probe: Option<f64>,` to `Response` in `material/mod.rs` and run `just test-one -p niri-config response_fields_match`. Expected: a compile error, `pattern does not mention field ring_probe`, at the destructuring in `pipeline.rs`. Remove the field (`git checkout -- niri-config/src/material/mod.rs` restores it only if nothing else in that file is staged; otherwise delete the line by hand) and rerun: PASS.

- [ ] **Step 5: Format, fast suite, commit**

```bash
cargo fmt --all
just test-fast
tasks done <step id> "pipeline.rs tables (11 sites, 19 stages, 1 interaction), response_fields() tied to the struct, check_selectors() with negative fixtures, ten config-side pins"
git add niri-config/src/material/pipeline.rs niri-config/src/material/mod.rs tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(config): pipeline schema tables with ownership, order, and response pins (material-a00785)"
```

---

### Task 2: Shader pins in the niri crate

**Files:**
- Modify: `src/render_helpers/shaders/mod.rs` (the `tests` module starting at line 459)
- Test: the same module

**Interfaces:**
- Consumes: `niri_config::material::pipeline::{STAGES, Program}` (Task 1), `material_source()` (line 17), `OPTICS`.
- Produces: nothing new; tests only.

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` of `src/render_helpers/shaders/mod.rs`:

```rust
    use niri_config::material::pipeline::{Program, STAGES};

    /// GLSL without its comments, so a commented-out call does not count.
    fn strip_comments(source: &str) -> String {
        let mut out = String::with_capacity(source.len());
        let mut rest = source;
        while !rest.is_empty() {
            if let Some(stripped) = rest.strip_prefix("/*") {
                let end = stripped.find("*/").map(|i| i + 2).unwrap_or(stripped.len());
                rest = &stripped[end..];
            } else if let Some(stripped) = rest.strip_prefix("//") {
                let end = stripped.find('\n').unwrap_or(stripped.len());
                rest = &stripped[end..];
            } else {
                let mut chars = rest.chars();
                out.push(chars.next().unwrap());
                rest = chars.as_str();
            }
        }
        out
    }

    /// Actual calls of `call` (`noise_behind(`): occurrences that are not a
    /// definition (preceded by a GLSL type) and not the tail of a longer
    /// identifier. Comments must already be stripped.
    fn hook_calls(source: &str, call: &str) -> Vec<usize> {
        const TYPES: &[&str] = &["void", "float", "vec2", "vec3", "vec4"];
        let bytes = source.as_bytes();
        let mut found = Vec::new();
        let mut from = 0;
        while let Some(at) = source[from..].find(call) {
            let at = from + at;
            from = at + 1;
            if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
                continue;
            }
            let before = source[..at].trim_end();
            let token_start = before.rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).map(|i| i + 1).unwrap_or(0);
            if TYPES.contains(&&before[token_start..]) {
                continue;
            }
            found.push(at);
        }
        found
    }

    #[test]
    fn hook_call_counter_ignores_comments_and_definitions() {
        let glsl = strip_comments(
            "vec3 noise_behind(vec3 c, vec2 p) { return c; }\n// sampled = noise_behind(sampled, p);\n/* noise_behind( */\nsampled = noise_behind(sampled, p);\nx = xnoise_behind(1);\n",
        );
        assert_eq!(hook_calls(&glsl, "noise_behind(").len(), 1);
        let twice = strip_comments("a = noise_behind(a, p);\nb = noise_behind(b, p);\n");
        assert_eq!(hook_calls(&twice, "noise_behind(").len(), 2);
    }

    /// The material program after the `// ---- main` marker, comments
    /// stripped: hook calls are counted there, not in an optic's own GLSL
    /// where the function is defined.
    fn main_body() -> String {
        let source = material_source();
        let marker = "\n// ---- main\n";
        let at = source.find(marker).expect("material_source carries the main marker");
        strip_comments(&source[at + marker.len()..])
    }

    #[test]
    fn pipeline_material_hooks_are_called_exactly_once_in_stage_order() {
        let body = main_body();
        let mut last = 0;
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            if optic.program != Program::Material {
                continue;
            }
            let call = format!("{}_{}(", optic.name, optic.hook);
            let calls = hook_calls(&body, &call);
            assert_eq!(calls.len(), 1, "{}: main.frag calls {call} {} times, expected exactly one", stage.id, calls.len());
            assert!(calls[0] >= last, "{}: {call} is called before an earlier stage's hook", stage.id);
            last = calls[0];
        }
    }

    #[test]
    fn pipeline_other_programs_call_their_hooks_exactly_once() {
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            let source = match optic.program {
                Program::Material => continue,
                Program::Effect => strip_comments(concat!(include_str!("blur_down.frag"), include_str!("blur_up.frag"))),
                Program::Postprocess => strip_comments(include_str!("postprocess.frag")),
            };
            let call = format!("{}_{}(", optic.name, optic.hook);
            let calls = hook_calls(&source, &call);
            assert_eq!(calls.len(), 1, "{}: the {:?} program calls {call} {} times, expected exactly one", stage.id, optic.program, calls.len());
        }
    }

    /// `_post(` is a suffix, not a full name, so this test scans for the
    /// suffix directly rather than through `hook_calls`, whose identifier
    /// guard would reject every match.
    #[test]
    fn pipeline_post_site_is_empty_until_a_stage_lands() {
        let has_post_stage = STAGES.iter().any(|s| s.site == "post");
        let has_post_call = main_body().contains("_post(");
        assert_eq!(has_post_stage, has_post_call, "the post site and the _post( call must arrive together");
    }

    /// Uniform names that are not parameters: slab geometry, jelly and signal
    /// state, textures, and the window's seed.
    const NON_PARAMETER_UNIFORMS: &[&str] = &[
        "mat_area_size", "mat_geo_rect", "mat_win_rect", "mat_slab_rect", "mat_bg_rect", "mat_backdrop_rect",
        "mat_ws_rect", "mat_ws_color", "mat_backdrop_color", "mat_corner_radius", "mat_chamfer", "mat_samples",
        "mat_bg_prefilter_mix", "mat_backdrop_prefilter_mix", "mat_jelly_seed", "mat_jelly_time", "mat_jelly_activity",
        "mat_jelly_move", "mat_jelly_resize", "mat_aurora_phase",
    ];

    /// Uniforms whose name is not the parameter's node: each maps to the
    /// nodes it carries.
    fn uniform_nodes(uniform: &str) -> Vec<&'static str> {
        match uniform {
            "mat_scatter" => vec!["roughness", "ior"],
            "mat_noise_type" => vec!["noise type="],
            "mat_aurora_color_a" | "mat_aurora_color_b" => vec!["aurora color"],
            "mat_distortion_scale" => vec!["distortion scale="],
            _ => Vec::new(),
        }
    }

    #[test]
    fn pipeline_optic_glsl_reads_are_declared() {
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            let entry = OPTICS.iter().find(|e| e.name == optic.name).unwrap();
            let mut seen = HashSet::new();
            for token in entry.glsl.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if !token.starts_with("mat_") || !seen.insert(token) {
                    continue;
                }
                if token.starts_with("mat_sig_") || NON_PARAMETER_UNIFORMS.contains(&token) {
                    continue;
                }
                let nodes = match uniform_nodes(token) {
                    v if v.is_empty() => vec![token.trim_start_matches("mat_").replace('_', "-").leak() as &str],
                    v => v,
                };
                for node in nodes {
                    assert!(stage.reads.contains(&node), "{}: its GLSL reads {token} ({node}), which `reads` omits", stage.id);
                }
            }
        }
    }
```

`String::leak` is stable since Rust 1.72; if the toolchain pin refuses it, collect the derived names into a `Vec<String>` first and compare with `stage.reads.iter().any(|r| *r == node)`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri pipeline_` and `just test-one -p niri hook_call_counter`
Expected: the counter test passes on its fixtures; the four pipeline tests compile and run; `pipeline_optic_glsl_reads_are_declared` must pass against Task 1's `reads` (aurora's `mat_ior`, `mat_light_ior`, `mat_thickness` are listed). If one fails naming a uniform, the failure is the finding: add the node to that stage's `reads` in Task 1's table, since the GLSL is the truth. The other three pass as written. A fully green first run is acceptable for this task: these are pins on existing code, and the drift case is proven in Step 3.

- [ ] **Step 3: Prove the pins bite**

Three edits to `src/render_helpers/shaders/material/main.frag` lines 80 and 81, each followed by `just test-one -p niri pipeline_material_hooks` and then `git checkout -- src/render_helpers/shaders/material/main.frag`:

1. Swap the two `behind` calls. Expected: FAIL, `noise: noise_behind( is called before an earlier stage's hook`.
2. Prefix line 81 with `//`. Expected: FAIL, `noise: main.frag calls noise_behind( 0 times, expected exactly one`.
3. Duplicate line 81. Expected: FAIL, `noise: main.frag calls noise_behind( 2 times, expected exactly one`.

After the last restore, rerun: PASS.

- [ ] **Step 4: Format, fast suite, commit**

```bash
cargo fmt --all
just test-fast
tasks done <step id> "shader pins: exactly one real hook call per program in stage order (comments and definitions excluded), post site emptiness, optic GLSL reads against the tables"
git add src/render_helpers/shaders/mod.rs tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "test(render): pin the pipeline schema to the shader hook calls and optic reads (material-a00785)"
```

---

### Task 3: The generated file and the docs

**Files:**
- Modify: `niri-config/Cargo.toml` (add `serde.workspace = true` and `serde_json.workspace = true` under `[dependencies]`)
- Modify: `niri-config/src/material/pipeline.rs` (serde derives, `render_schema()`, the freshness test)
- Create: `resources/materials/pipeline.json` (generated)
- Modify: `docs/materials/render-pipeline.md` (one paragraph before `## Related designs`, line 180)
- Modify: `docs/materials/adding-an-optic.md` (one step in section 3, after the `OPTICS` registration paragraph)

**Interfaces:**
- Produces: `niri_config::material::pipeline::render_schema() -> String` (the JSON text with a trailing newline) and the file prism vendors.

- [ ] **Step 1: Write the failing tests**

Append to `mod tests` in `pipeline.rs`:

```rust
    #[test]
    fn schema_renders_stably_with_a_trailing_newline() {
        let once = render_schema();
        assert_eq!(once, render_schema(), "two renders are identical");
        assert!(once.ends_with('\n'));
        let value: serde_json::Value = serde_json::from_str(&once).unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(value["sites"][0]["id"], "source");
        assert_eq!(value["sites"][0]["orderable"], false);
        let noise = value["stages"].as_array().unwrap().iter().find(|s| s["id"] == "noise").unwrap();
        assert_eq!(noise["optic"], serde_json::json!({"name": "noise", "hook": "behind", "program": "material"}));
        assert!(noise.get("selector").is_none(), "absent optional fields are omitted, not null");
        let encode = value["stages"].as_array().unwrap().iter().find(|s| s["id"] == "encode").unwrap();
        assert!(encode.get("optic").is_none());
        assert_eq!(value["interactions"][0]["kind"], "attenuates");
    }

    #[test]
    fn material_pipeline_schema_matches_the_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../resources/materials/pipeline.json");
        let expected = render_schema();
        if std::env::var_os("MATERIAL_DOCS_UPDATE").is_some() {
            std::fs::write(&path, &expected).unwrap();
            return;
        }
        let actual = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(actual, expected, "resources/materials/pipeline.json is stale; rerun with MATERIAL_DOCS_UPDATE=1");
    }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri-config pipeline`
Expected: compile error, `render_schema` not found (and `serde_json` unresolved until the Cargo change).

- [ ] **Step 3: Add the dependencies, the derives, and the renderer**

In `niri-config/Cargo.toml` under `[dependencies]` add:

```toml
serde.workspace = true
serde_json.workspace = true
```

In `pipeline.rs`, add `use serde::Serialize;` at the top and `#[derive(Serialize)]` with `#[serde(rename_all = "lowercase")]` on each of the seven enums, and `#[derive(Serialize)]` on `Site`, `OpticRef`, `Selector`, `Stage`, and `Interaction`. On `Stage`, annotate the two optionals:

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optic: Option<OpticRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<Selector>,
```

`Program::Postprocess` renders as `postprocess` under `rename_all = "lowercase"`, matching the spec's `material | effect | postprocess`; `Kind::Attenuates` as `attenuates`.

Add the renderer:

```rust
#[derive(Serialize)]
struct Schema {
    version: u32,
    sites: &'static [Site],
    stages: &'static [Stage],
    interactions: &'static [Interaction],
}

/// The schema as `resources/materials/pipeline.json` carries it: version 1,
/// the three tables verbatim, pretty-printed with a trailing newline so the
/// file is stable across regenerations.
pub fn render_schema() -> String {
    let schema = Schema { version: 1, sites: SITES, stages: STAGES, interactions: INTERACTIONS };
    let mut out = serde_json::to_string_pretty(&schema).expect("the schema serializes");
    out.push('\n');
    out
}
```

- [ ] **Step 4: Generate the file, then run the tests**

Run: `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file`, then `just test-one -p niri-config pipeline`.
Expected: the first run writes `resources/materials/pipeline.json`; the second passes, 11 tests.

Open the file and confirm `"stages"` lists `saturation` and `noise` with `"site": "behind"` before `tint`, and that no `null` appears anywhere (`grep -c null resources/materials/pipeline.json` prints 0).

- [ ] **Step 5: Document**

In `docs/materials/render-pipeline.md`, before `## Related designs` (line 180), add:

```markdown
## 6. The schema file

The structure above is also data: `niri-config/src/material/pipeline.rs`
holds the sites (carrier, composition law, orderable, coverage, cost), the
stages (site, scope, the parameters each owns and reads, the response fields
it reads, its optic hook and program), and the structural interactions, and
tests pin them to `all_params()`, `ORDER`, the `Response` block, and the hook
calls in each shader program. `resources/materials/pipeline.json` is
generated from them (`MATERIAL_DOCS_UPDATE=1 just test` regenerates it) and
is what prism vendors to validate its device rack. Design: prism
`docs/specs/2026-10-04-pipeline-schema-design.md`.
```

In `docs/materials/adding-an-optic.md`, in section 3 after the paragraph that ends `Add one call per used hook to `shaders/material/main.frag`, in `OPTICS` order.`, add:

```markdown
Then add the optic's stage to `niri-config/src/material/pipeline.rs`: its
site, scope, the parameters it owns and every parameter its GLSL reads, its
response fields, and `optic(name, hook)`; add an `Interaction` if it makes
another stage inert or scales it unconditionally. The config tests refuse an
unowned parameter and a GLSL read the stage does not list; the niri tests
refuse a hook the program does not call. Regenerate the schema file with the
parameter table (section 4).
```

In section 4 of the same file, the regenerate command already covers both tables since `just test` runs every niri-config test; no change there.

- [ ] **Step 6: Format, fast suite, check, commit**

```bash
cargo fmt --all
just test-fast
just check
tasks done <step id> "render_schema(), resources/materials/pipeline.json generated under MATERIAL_DOCS_UPDATE, render-pipeline.md §6 and the adding-an-optic step"
git add niri-config/Cargo.toml niri-config/src/material/pipeline.rs resources/materials/pipeline.json docs/materials/render-pipeline.md docs/materials/adding-an-optic.md tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(config): generate resources/materials/pipeline.json from the pipeline tables (material-a00785)"
```

`just check` runs the package-pin and docs checks the material docs need; it is the pre-commit gate and must pass before the commit is attempted.

---

## Closing the task

After Task 3: `just gate` in the task worktree, then `tasks done material-a00785 "<what landed>"` in a final commit. Tell prism: its plan's Task 1 copies `resources/materials/pipeline.json` from this tree's merge commit. Integration follows the finishing-a-development-branch skill; this is a personal-profile checkout, so a local merge is the owner's default. Run `tt-report` before removing the worktree.
