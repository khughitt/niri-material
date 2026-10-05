# Noise Placement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the glass `noise` node a `site` property (`glass`, `backdrop`, `film`) so grain can land before the frosted blur, on the transmitted backdrop as today, or over the finished glass, with the renderer, its schema and its docs telling the truth about each site.

**Architecture:** `site` is parsed on the existing node and validated across the material table (backdrop grain is one texture per output, so every material placing noise there must agree). The effect buffer gains a cached grain pass between the sharp texture and every consumer (blur, both pyramids, the direct sample), publishing damage through its commit counter. The material program gains a site uniform, a gate on the behind hook and a new post hook. The pipeline schema gains three selector stages for the noise optic; the pins keep them honest. Evidence: an in-process headless pixel suite, a nested-Weston smoke against a baseline binary, and Tracy cost captures.

**Tech Stack:** Rust (niri fork, smithay GLES renderer), knuffel KDL config, GLSL ES 1.00, cargo-nextest through `just test-one` / `just test-fast`, ImageMagick 7, headless Weston, Tracy 0.13.1 (retained tools), `tasks` CLI.

**Spec:** `docs/specs/2026-10-05-noise-placement-design.md` (this worktree, `.worktrees/material-cf32e5`). Section numbers below (§3, §4, ...) refer to it. The prism-side contract is prism `docs/specs/2026-10-04-pipeline-schema-design.md` Section 5.

**Status:** revised after plan review round 2 (codex/gpt-6-astra, 2026-10-05: the cost cleanup carries the original exit status past its wallpaper stop; the trailing-newline schema test asserts the glass selector and checks omission on an unselected stage; the in-process fixture's output is 800 px wide so both windows fit, and regions are clipped to the output; the one-code tolerance is `-fuzz 0.392157%`; every capture cell is taken twice and compared). Round 1 (codex/gpt-6-astra, 2026-10-05): the precision line precedes the shared helpers; the cost script owns its wallpaper PIDs; Task 1 is green on its own (field migration, ownership on the existing stage); `film-grain` precedes `effect-saturation`; the coloured test buffer scales bytes to the single-pixel protocol; tests locate windows by material name; the smoke uses the lib's `spawn_probe`/`calibrate_probe_rect`/`probe_rect`/`assert_about` and a valid ring config; the grain program compiles lazily in the buffer and retries after invalidation; the cost section reports measurements, not verdicts; the capture matrix is the spec's; `just setup` precedes the baseline build.

## Global Constraints

- Branch `material-cf32e5` in `.worktrees/material-cf32e5`, on `materials-26.04`. Baseline commit `b261ad1a` (the task-start commit).
- Never call a test runner directly: `just test-one <nextest args>` for one test, `just test-fast` before every commit, `just check` before every commit, `just gate` before the merge.
- Commit order in this repository: `git add <files>` → `just upstream-report --stage` → `git add docs/materials/upstream-divergence.md` → `git commit`. Conventional commit subjects ending in `(material-cf32e5)`; no AI attribution of any kind.
- Each task is a child task of `material-cf32e5` (ids in each task's heading); `tasks start <id>` before its first step, `tasks done <id> "<one line>"` staged in its last commit.
- A config naming no `site` renders byte-identical to the baseline binary (§1, §8): no change to `noise_behind`'s arithmetic, no change to the material source's text other than the common-helper move and the post call (§5).
- Backdrop grain: one amount and type per output, agreed at config load (§3); applies whether or not `blur { off }` is set (§3); seed offset `(47, 113)` at every site (§4, §5).
- Every content read of the sharp texture goes through the grained source when grain is set: `prepare_blur`'s `prepare_textures`, both branches of `render`, and through `render` the pyramids (§4).
- A grain option change clears the grained texture, the blurred texture, both pyramids, and increments `commit_counter` whenever an offscreen exists (§4).
- A failed grain program warns once and renders ungrained until the next invalidation (§4).
- Schema version stays 1; the generated `resources/materials/pipeline.json` is regenerated, never hand-edited (§6).
- No live desktop: in-process tests are headless; the capture lane is nested Weston.

## Review Focus

Five inputs the spec implies but no test exercised when this plan was drafted; each now has a test in the task named.

1. `noise 0 site="film"` and `noise 0 site="backdrop"` must render as if no noise node existed: amount 0 is neutral at every site (§5 "amount-0 early return stays first"). Test `amount_zero_is_neutral_at_every_site`, Task 4.
2. Reloading from `site="backdrop"` back to `site="glass"` must remove the grain from the shared texture and re-render every window that sampled it: the `Some → None` case of §4. Test `backdrop_to_glass_reload_clears_the_grain`, Task 4, and the `GrainOptionsChanged` row of the invalidation table, Task 3.
3. Transparent regions of the effect buffer (no background layer there) must stay transparent and ungrained, since `noise_source` returns `a == 0` texels unchanged (§4). Test `backdrop_grain_leaves_transparent_texels_alone`, Task 4.
4. A window with no material but `background-effect { blur true }` must show the grained backdrop under it once any material places noise at the backdrop (§3 "the window-site element"). Test `window_site_element_sees_backdrop_grain`, Task 4.
5. The agreement rule must not depend on material order or on which material a window rule references: two agreeing materials pass whichever is first, and the disagreement error names both (§3). Tests `backdrop_grain_agreement_is_order_independent` and `backdrop_grain_disagreement_names_both_materials`, Task 1.

---

### Task 1: Config: the `site` property, the agreement rule, the parameter row

**Files:**
- Modify: `niri-config/src/material/optics/noise.rs`
- Modify: `niri-config/src/lib.rs` (re-exports near line 60, the `recursion == 0` block near line 533, `impl Config` near line 546, tests beside `glass_noise_type_parses_each_value` near line 1766, the `ResolvedNoise` literal near line 1619)
- Modify: `niri-config/src/material/pipeline.rs` (the existing `noise` row owns and reads `noise site=`)
- Modify: `src/render_helpers/material/optics/noise.rs:68` and `src/layout/tile.rs:2852` (exhaustive `ResolvedNoise` literals gain the field)
- Modify: `docs/materials/material-config.md` (generated table between `<!-- params:begin -->` and `<!-- params:end -->`)
- Modify: `resources/materials/pipeline.json` (regenerated: the `noise` stage's `owns` and `reads`)

This task is green on its own: the new field is written into every exhaustive literal, and the new parameter is owned by the existing `noise` stage, so the ownership pin passes without the new stages (those arrive with their hooks in Task 2).

**Interfaces:**
- Produces: `pub enum NoiseSite { Glass = 0, Backdrop = 1, Film = 2 }` with `NAMES`, `FromStr`, `Default = Glass`; `ResolvedNoise.site: NoiseSite`; `pub struct BackdropGrain { pub amount: f64, pub kind: NoiseType }`; `pub fn backdrop_grain(materials: &[Material]) -> Result<Option<BackdropGrain>, String>` in `niri_config::material::optics::noise`; `Config::backdrop_grain(&self) -> Option<BackdropGrain>`; `ParamSpec` node `"noise site="`, owned and read by the `noise` stage. All re-exported from `niri_config` (`NoiseSite`, `BackdropGrain`).
- Consumed by: Task 2 (`site` uniform), Task 3 (`GrainOptions::from(BackdropGrain)`), Task 4 (configs with `site=`).

- [x] **Step 1: Write the failing parse tests**

In `niri-config/src/lib.rs`, directly after `glass_noise_type_cannot_be_written_without_an_amount`, add:

```rust
    #[test]
    fn glass_noise_site_parses_each_value_and_defaults_to_glass() {
        for (written, expected) in [
            ("glass", NoiseSite::Glass),
            ("backdrop", NoiseSite::Backdrop),
            ("film", NoiseSite::Film),
        ] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.3 type=\"fine\" site=\"{written}\"; }}; }}\n"
            ));
            let noise = parsed.materials[0].resolve().glass.noise;
            assert_eq!(noise.site, expected, "{written}");
            assert_eq!(noise.amount, Some(0.3), "{written}");
            assert_eq!(noise.kind, NoiseType::Fine, "{written}");
        }
        let parsed = do_parse("material \"frost\" { glass { noise 0.3; }; }\n");
        assert_eq!(parsed.materials[0].resolve().glass.noise.site, NoiseSite::Glass);
        let parsed = do_parse("material \"frost\" { glass { }; }\n");
        assert_eq!(parsed.materials[0].resolve().glass.noise, ResolvedNoise::default());
        assert_eq!(ResolvedNoise::default().site, NoiseSite::Glass);
    }

    #[test]
    fn glass_noise_site_rejects_an_unknown_value() {
        let err = do_parse_err("material \"frost\" { glass { noise 0.3 site=\"roof\"; }; }\n");
        assert!(err.contains("unknown NoiseSite value: roof"), "{err}");
    }

    #[test]
    fn backdrop_grain_disagreement_names_both_materials() {
        let err = do_parse_err(
            "material \"a\" { glass { noise 0.3 type=\"fine\" site=\"backdrop\"; }; }\n\
             material \"b\" { glass { noise 0.1 type=\"fine\" site=\"backdrop\"; }; }\n",
        );
        assert!(
            err.contains(
                "materials \"a\" and \"b\" both place noise at the backdrop with different \
                 settings (0.3 fine, 0.1 fine); the backdrop is one texture per output"
            ),
            "{err}"
        );
        let err = do_parse_err(
            "material \"a\" { glass { noise 0.3 type=\"fine\" site=\"backdrop\"; }; }\n\
             material \"b\" { glass { noise 0.3 type=\"white\" site=\"backdrop\"; }; }\n",
        );
        assert!(err.contains("(0.3 fine, 0.3 white)"), "{err}");
    }

    #[test]
    fn backdrop_grain_agreement_is_order_independent() {
        for order in [["a", "b"], ["b", "a"]] {
            let parsed = do_parse(&format!(
                "material \"{}\" {{ glass {{ noise 0.3 type=\"fine\" site=\"backdrop\"; }}; }}\n\
                 material \"{}\" {{ glass {{ noise 0.3 type=\"fine\" site=\"backdrop\"; }}; }}\n\
                 material \"c\" {{ glass {{ noise 0.9 type=\"white\" site=\"glass\"; }}; }}\n\
                 material \"d\" {{ glass {{ noise 0.5 site=\"film\"; }}; }}\n",
                order[0], order[1]
            ));
            assert_eq!(
                parsed.backdrop_grain(),
                Some(BackdropGrain {
                    amount: 0.3,
                    kind: NoiseType::Fine
                }),
                "{order:?}"
            );
        }
        let parsed = do_parse("material \"c\" { glass { noise 0.9; }; }\n");
        assert_eq!(parsed.backdrop_grain(), None);
    }
```

Add `NoiseSite` and `BackdropGrain` to the test module's imports where `NoiseType` is imported (search `use crate::{` or the `NoiseType` import inside `mod tests`).

- [x] **Step 2: Run them to see them fail to compile**

Run: `just test-one -p niri-config glass_noise_site`
Expected: compile error, `NoiseSite` not found.

- [x] **Step 3: Add the enum, the property, the resolved field, the agreement function, the parameter row, and the field in every literal**

In `niri-config/src/material/optics/noise.rs`, after `impl FromStr for NoiseType`, add:

```rust
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
```

Extend the node and the resolved struct:

```rust
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
/// first two materials that disagree. The backdrop is one texture per
/// output, so there is no per-material value to fall back on.
pub fn backdrop_grain(
    materials: &[crate::material::Material],
) -> Result<Option<BackdropGrain>, String> {
    let mut agreed: Option<(&str, BackdropGrain)> = None;
    for material in materials {
        let noise = material.resolve().glass.noise;
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
                     different settings ({} {}, {} {}); the backdrop is one texture per output",
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
```

Append the parameter row to `params()`:

```rust
        ParamSpec {
            node: "noise site=",
            kind: ParamKind::Enum {
                default: "glass",
                variants: NoiseSite::NAMES,
            },
            write: |v| format!("noise 0.5 site=\"{v}\""),
            read: None,
        },
```

In `niri-config/src/lib.rs`:

Line 60 re-export becomes
`pub use crate::material::optics::noise::{BackdropGrain, Noise, NoiseSite, NoiseType, ResolvedNoise};`.

In the `recursion == 0` block, after the `validate_material_refs(...)` call and inside the same `if`:

```rust
            if let Err(message) = crate::material::optics::noise::backdrop_grain(&config.materials)
            {
                ctx.emit_error(DecodeError::Custom(message.into()));
            }
```

In `impl Config`, after `load_default`:

```rust
    /// The backdrop grain every material placing noise there agrees on
    /// (`None` when none does). Decoding refused a disagreeing table, so
    /// this never meets one.
    pub fn backdrop_grain(&self) -> Option<BackdropGrain> {
        material::optics::noise::backdrop_grain(&self.materials)
            .expect("the material table was validated when the config was decoded")
    }
```

Exhaustive `ResolvedNoise { amount, kind }` literals no longer compile; give each the field (the `..Default::default()` ones at `src/render_helpers/material/optics/mod.rs:207` and `src/layout/tile.rs:2808` need nothing):

- `niri-config/src/lib.rs:1619`: add `site: NoiseSite::Glass,` after `kind: NoiseType::Fine,`.
- `src/render_helpers/material/optics/noise.rs:68`: add `site: NoiseSite::Glass,` after `kind: NoiseType::Lightness,` and import `NoiseSite` beside `NoiseType` in that test module.
- `src/layout/tile.rs:2852`: add `site: niri_config::NoiseSite::Glass,` after `kind: noise_type,`.

In `niri-config/src/material/pipeline.rs`, the existing `noise` row owns and reads the new parameter (the new stages come in Task 2):

```rust
    stage(
        "noise",
        "behind",
        Scope::Material,
        &["noise", "noise type=", "noise site="],
        &["noise", "noise type=", "noise site=", "blur noise", "backdrop-blur"],
        &[],
        optic("noise", "behind"),
        false,
    ),
```

- [x] **Step 4: Run the new tests, the config crate, and the niri crate's pins**

Run: `just test-one -p niri-config glass_noise_site` then `just test-one -p niri-config backdrop_grain` then `just test-one -p niri-config material_` then `just test-one -p niri-config pipeline` then `just test-one -p niri pipeline_` then `just test-one -p niri noise`
Expected: the four new tests PASS; `material_parameter_specs_match_the_parser` PASS (it writes `noise 0.5 site="glass"` through `write`); `every_parameter_is_owned_by_exactly_one_stage` and `reads_cover_owns_and_name_only_parameters` PASS; the niri pins PASS (the GLSL-reads pin checks that what the GLSL reads is listed, not the converse, so the stage may list `noise site=` before the shader declares it); only `material_parameter_table_matches_the_docs` and `material_pipeline_schema_matches_the_file` FAIL, both as "stale".

- [x] **Step 5: Regenerate the parameter table and the schema file**

Run: `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_parameter_table_matches_the_docs` and `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file`
Then `git diff docs/materials/material-config.md`: exactly one new row, `| \`noise\` \`site=\` | \`glass\` / \`backdrop\` / \`film\` | \`glass\` | — | — |`, after the `noise type=` row. `git diff resources/materials/pipeline.json`: the `noise` stage's `owns` and `reads` each gain `"noise site="`; nothing else.

- [x] **Step 6: Gate and commit**

Run: `just test-fast` and `just check`. Expected: green.

```bash
git add niri-config/src/material/optics/noise.rs niri-config/src/lib.rs niri-config/src/material/pipeline.rs \
  src/render_helpers/material/optics/noise.rs src/layout/tile.rs docs/materials/material-config.md resources/materials/pipeline.json
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "feat(config): noise site= with the backdrop agreement rule (material-cf32e5)"
```

---

### Task 2: The three placements in the material program, the grain shader text, and the schema

**Files:**
- Create: `src/render_helpers/shaders/material/common.frag` (the six helpers moved out of the prelude)
- Create: `src/render_helpers/shaders/grain.frag` (the effect program's `main`)
- Modify: `src/render_helpers/shaders/material/prelude.frag` (remove the moved helpers and the `precision` line, which moves to the assembler)
- Modify: `src/render_helpers/shaders/material/noise.frag`
- Modify: `src/render_helpers/shaders/material/main.frag` (the post call)
- Modify: `src/render_helpers/shaders/mod.rs` (`material_source`, new `grain_source`, tests)
- Modify: `src/render_helpers/material/optics/noise.rs` (`mat_noise_site`)
- Modify: `niri-config/src/material/pipeline.rs` (three selector stages, `select`, `effect_optic`, tests)
- Modify: `resources/materials/pipeline.json` (regenerated)

**Interfaces:**
- Consumes: `NoiseSite` (Task 1).
- Produces: GLSL `vec3 noise_post(vec3 encoded, vec2 fragCoord)`, `vec4 noise_source(vec4 texel, vec2 fragCoord)`, uniform `mat_noise_site`; `pub(crate) fn grain_source() -> String` in `shaders/mod.rs` (a complete `#version 100` fragment program over `varying vec2 v_coords`, `uniform sampler2D tex`, `uniform float mat_noise`, `uniform float mat_noise_type`); schema stages `backdrop-grain` and `film-grain` with selectors, and the selector on `noise`.

- [x] **Step 1: Write the failing pin tests**

In `src/render_helpers/shaders/mod.rs` tests:

Replace the last line of `material_source_is_prelude_then_optics_in_order_then_main` (`assert!(!main.contains("noise_post("));`) with:

```rust
        let encode = main.find("vec3 glassColor = linearToSrgb(").unwrap();
        let post = main.find("glassColor = noise_post(glassColor, gl_FragCoord.xy);").unwrap();
        let coverage = main.find("glassed = vec4(glassColor, 1.0) * coverage;").unwrap();
        assert!(encode < post && post < coverage, "the post hook sits between encode and coverage");
        // GLSL ES needs the float precision declared before the first float
        // function; the shared helpers come right after it, before the prelude.
        assert!(source.starts_with("precision highp float;\n// ---- common\n"));
        assert_eq!(source.matches("precision highp float;").count(), 1);
        assert!(source.find("// ---- common").unwrap() < source.find("// ---- prelude").unwrap());
        assert!(source.contains("uniform float mat_noise_site;"));
```

Delete `pipeline_post_site_is_empty_until_a_stage_lands` entirely.

Replace `pipeline_other_programs_call_their_hooks_exactly_once` with the per-file form:

```rust
    /// The non-material programs, each as its files: an Effect hook must be
    /// called exactly once in exactly one effect file.
    fn program_files(program: Program) -> Vec<(&'static str, String)> {
        match program {
            Program::Material => Vec::new(),
            Program::Effect => vec![
                ("blur_down.frag", strip_comments(include_str!("blur_down.frag"))),
                ("blur_up.frag", strip_comments(include_str!("blur_up.frag"))),
                ("grain.frag", strip_comments(&grain_source())),
            ],
            Program::Postprocess => vec![(
                "postprocess.frag",
                strip_comments(include_str!("postprocess.frag")),
            )],
        }
    }

    #[test]
    fn pipeline_other_programs_call_their_hooks_exactly_once() {
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            if optic.program == Program::Material {
                continue;
            }
            let call = format!("{}_{}(", optic.name, optic.hook);
            let calling: Vec<&str> = program_files(optic.program)
                .iter()
                .filter_map(|(file, source)| match hook_calls(source, &call).len() {
                    0 => None,
                    1 => Some(*file),
                    n => panic!("{}: {file} calls {call} {n} times", stage.id),
                })
                .collect();
            assert_eq!(
                calling.len(),
                1,
                "{}: the {:?} program must call {call} in exactly one file, found {calling:?}",
                stage.id,
                optic.program
            );
        }
    }

    #[test]
    fn grain_source_is_a_complete_effect_program_over_the_noise_optic() {
        let source = grain_source();
        assert!(source.starts_with("#version 100\n"));
        assert_eq!(source.matches("void main()").count(), 1);
        assert!(source.contains("uniform sampler2D tex;"));
        assert!(source.contains("vec4 noise_source(vec4 texel, vec2 fragCoord)"));
        assert!(source.contains("float hash12(vec2 p)"));
        assert!(!source.contains("niri_v_coords"), "the grain program has no material prelude");
        let body = strip_comments(source.split_once("// ---- main").unwrap().1);
        assert_eq!(hook_calls(&body, "noise_source(").len(), 1);
    }
```

Add `"mat_noise_site" => vec!["noise site="],` to `uniform_nodes`.

In `niri-config/src/material/pipeline.rs` tests, in `todays_shape`, replace the `no selector yet` assertion with:

```rust
        let selected: Vec<(&str, &str, &str)> = STAGES
            .iter()
            .filter_map(|s| s.selector.map(|sel| (s.id, sel.param, sel.variant)))
            .collect();
        assert_eq!(
            selected,
            [
                ("backdrop-grain", "noise site=", "backdrop"),
                ("noise", "noise site=", "glass"),
                ("film-grain", "noise site=", "film"),
            ]
        );
        let source: Vec<&str> = STAGES.iter().filter(|s| s.site == "source").map(|s| s.id).collect();
        assert_eq!(source, ["backdrop-grain", "blur", "prefilter"]);
        let post: Vec<&str> = STAGES.iter().filter(|s| s.site == "post").map(|s| s.id).collect();
        assert_eq!(post, ["film-grain"]);
        let film = STAGES.iter().find(|s| s.id == "film-grain").unwrap();
        assert_eq!(film.optic.map(|o| o.program), Some(Program::Material));
        let grain = STAGES.iter().find(|s| s.id == "backdrop-grain").unwrap();
        assert_eq!(grain.optic.map(|o| (o.program, o.hook)), Some((Program::Effect, "source")));
        assert_eq!(grain.scope, Scope::Output);
        assert!(grain.owns.is_empty() && film.owns.is_empty());
```

In `schema_renders_stably_with_a_trailing_newline` (pipeline.rs tests, near line 1054), the `noise` stage now carries a selector, so replace `assert!(noise.get("selector").is_none(), ...)` with

```rust
        assert_eq!(
            noise["selector"],
            serde_json::json!({"param": "noise site=", "variant": "glass"})
        );
        let slab = value["stages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == "slab")
            .unwrap();
        assert!(
            slab.get("selector").is_none(),
            "an unselected stage omits the field rather than writing null"
        );
```

so the omitted-field check moves to a stage that stays unselected.

In `selector_rules_refuse_each_defect`, if no fixture yet covers two selector stages of different optics on one parameter, add one:

```rust
        let mixed = [
            selected("a", "white", "noise"),
            selected("b", "fine", "saturation"),
            selected("c", "lightness", "noise"),
        ];
        assert!(check_selectors(&mixed, &specs)
            .unwrap_err()
            .contains("optic"));
```

(Read the function first; the existing seven fixtures may already include it under another name. Keep exactly one such fixture.)

- [x] **Step 2: Run the pins to see them fail**

Run: `just test-one -p niri pipeline_` and `just test-one -p niri-config todays_shape` and `just test-one -p niri-config schema_renders_stably`
Expected: `grain_source` does not exist (compile error in the niri crate tests); `todays_shape` FAILS on the selector list; `schema_renders_stably_with_a_trailing_newline` FAILS on the noise selector.

- [x] **Step 3: Move the helpers into `common.frag`**

Create `src/render_helpers/shaders/material/common.frag` with, verbatim from `prelude.frag`, the functions `srgbToLinear`, `linearToSrgb` (with the comment block above them, lines 127–142), `hash12`, `fineGrain` (with its comment, lines 228–247) and `linearToOklab`, `oklabToLinear` (with the Oklab comment, lines 249–275). Head the file with:

```glsl
// Helpers shared by the material program and the effect-program grain pass
// (grain.frag): transfer functions, the hash grain, and Oklab. No uniforms;
// concatenated first in both programs.
```

Delete those lines from `prelude.frag`, and delete its `precision highp float;` line too: the assembler now emits it first, ahead of the helpers, because GLSL ES refuses a float function before the float precision is declared. Update the prelude's header comment ("part 1 of 3" becomes "the prelude: core uniforms, shared globals, the slab, and the refraction taps; common.frag precedes it and shaders/mod.rs emits the precision line"). Everything else in the prelude stays in place; `snoise` and its helpers stay there because only the material program uses them.

In `shaders/mod.rs`:

```rust
/// The material fragment shader: the precision line, the shared helpers,
/// the prelude, each optic's GLSL in `OPTICS` order, then main. A comment
/// marker per part keeps compile-error line numbers locatable by hand.
pub(crate) fn material_source() -> String {
    let mut source = String::from("precision highp float;\n// ---- common\n");
    source.push_str(include_str!("material/common.frag"));
    source.push_str("\n// ---- prelude\n");
    source.push_str(include_str!("material/prelude.frag"));
    for entry in OPTICS {
        source.push_str(&format!("\n// ---- optic: {}\n", entry.name));
        source.push_str(entry.glsl);
    }
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("material/main.frag"));
    source
}

/// The effect-program grain pass (`noise site=backdrop`): a complete
/// `#version 100` fragment program over `blur.vert`'s `v_coords`, built from
/// the shared helpers, the noise optic's GLSL and `grain.frag`'s main.
pub(crate) fn grain_source() -> String {
    let noise = OPTICS
        .iter()
        .find(|entry| entry.name == "noise")
        .expect("the noise optic is registered");
    let mut source = String::from(
        "#version 100\nprecision highp float;\nvarying vec2 v_coords;\nuniform sampler2D tex;\n",
    );
    source.push_str("\n// ---- common\n");
    source.push_str(include_str!("material/common.frag"));
    source.push_str("\n// ---- optic: noise\n");
    source.push_str(noise.glsl);
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("grain.frag"));
    source
}
```

Run: `just test-one -p niri material_source_is_prelude_then_optics_in_order_then_main`
Expected: FAIL only on the `noise_post` / `mat_noise_site` assertions. The move itself must compile on the GPU: run `just test-one -p niri ring_look` (an in-process render through the real material program) and expect PASS with no "error compiling" warning in its output. If `glslangValidator` is installed, also `cargo run`-free: dump the source from a scratch test or `println!` and run `glslangValidator -S frag <(printf '#version 100\n'; cat source.glsl)`; it must accept.

- [x] **Step 4: Extend `noise.frag` and call the post hook**

Replace `src/render_helpers/shaders/material/noise.frag` with:

```glsl
// Optic: noise. Three placements selected by mat_noise_site
// (0 glass, 1 backdrop, 2 film; design 2026-10-05-noise-placement-design.md):
// noise_behind at the glass site (render-pipeline.md stage 3b), noise_source
// in the effect-program grain pass at the backdrop site, noise_post at the
// film site (stage 9). Neutral at amount 0. Uses hash12, fineGrain,
// srgbToLinear, linearToSrgb, linearToOklab and oklabToLinear from
// common.frag.
uniform float mat_noise;
uniform float mat_noise_type;
uniform float mat_noise_site;

// Glass: the averaged linear backdrop, encoded for the grain, decoded on
// return. The body is the pre-site original, byte for byte, behind one gate.
vec3 noise_behind(vec3 color, vec2 fragCoord) {
    if (mat_noise <= 0.0 || mat_noise_site != 0.0)
        return color;
    vec3 encoded = linearToSrgb(color);
    vec2 noiseSeed = fragCoord + vec2(47.0, 113.0);
    if (mat_noise_type < 0.5)
        return srgbToLinear(encoded + (hash12(noiseSeed) - 0.5) * mat_noise);
    float grain = fineGrain(noiseSeed) * mat_noise;
    if (mat_noise_type < 1.5)
        return srgbToLinear(encoded + vec3(grain));
    // Retain lightness grain's gamut clamp, returning its linear result.
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return clamp(oklabToLinear(lab), 0.0, 1.0);
}

// The signed scalar grain for the white and fine kinds, scaled by amount;
// the lightness kind uses the fine value through noiseLightness.
float noiseGrain(vec2 seed) {
    return (mat_noise_type < 0.5 ? hash12(seed) - 0.5 : fineGrain(seed)) * mat_noise;
}

// Lightness grain on an encoded colour: Oklab L of the clamped colour moves
// by `grain`; the result is clamped and re-encoded.
vec3 noiseLightness(vec3 encoded, float grain) {
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}

// Grain on an encoded colour, every kind: white and fine add in encoding
// (signed, unclamped); lightness goes through noiseLightness.
vec3 noiseEncoded(vec3 encoded, vec2 seed) {
    float grain = noiseGrain(seed);
    if (mat_noise_type < 1.5)
        return encoded + vec3(grain);
    return noiseLightness(encoded, grain);
}

// Film: the encoded glass after ring, aurora, glint and sweeps, before the
// coverage multiply. Same seed offset as the glass site.
vec3 noise_post(vec3 encoded, vec2 fragCoord) {
    if (mat_noise <= 0.0 || mat_noise_site != 2.0)
        return encoded;
    return noiseEncoded(encoded, fragCoord + vec2(47.0, 113.0));
}

// Backdrop: one premultiplied texel of the effect buffer, in the effect
// program. The amount and type arrive from the agreed backdrop grain, not
// from a material, and mat_noise_site is not consulted. A transparent texel
// is left alone; the grain lands on the straight colour and is clamped,
// because the result is stored in an 8-bit premultiplied texture.
vec4 noise_source(vec4 texel, vec2 fragCoord) {
    if (mat_noise <= 0.0 || texel.a <= 0.0)
        return texel;
    vec3 straight = texel.rgb / texel.a;
    vec3 grained = noiseEncoded(straight, fragCoord + vec2(47.0, 113.0));
    return vec4(clamp(grained, 0.0, 1.0) * texel.a, texel.a);
}
```

In `main.frag`, replace

```glsl
        vec3 glassColor = linearToSrgb(transmitted + within + specular + emissive);
        // Post hooks are reserved here for future screen-space film effects.
        glassed = vec4(glassColor, 1.0) * coverage;
```

with

```glsl
        vec3 glassColor = linearToSrgb(transmitted + within + specular + emissive);
        // Post hooks, in OPTICS order (render-pipeline.md stage 9): film grain
        // over the finished, encoded glass.
        glassColor = noise_post(glassColor, gl_FragCoord.xy);
        glassed = vec4(glassColor, 1.0) * coverage;
```

Create `src/render_helpers/shaders/grain.frag`:

```glsl
// Effect program: the backdrop grain pass (noise site=backdrop). grain_source()
// in shaders/mod.rs assembles the effect header, common.frag and the noise
// optic's GLSL ahead of this main; blur.vert supplies v_coords.
void main() {
    gl_FragColor = noise_source(texture2D(tex, v_coords), gl_FragCoord.xy);
}
```

In `src/render_helpers/material/optics/noise.rs`:

```rust
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_1f),
        ("mat_noise_type", UniformType::_1f),
        ("mat_noise_site", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let amount = glass.noise.amount.unwrap_or(if ctx.backdrop_blur {
            ctx.blur.noise
        } else {
            0.
        });
        vec![
            Uniform::new("mat_noise", amount as f32),
            Uniform::new("mat_noise_type", glass.noise.kind as u8 as f32),
            Uniform::new("mat_noise_site", glass.noise.site as u8 as f32),
        ]
    }
```

Update the test helper `pair` in that file to a `triple` returning `(f32, f32, f32)` (assert `values.len() == 3` and `values[2].name == "mat_noise_site"`), update the two existing tests to the third component `0.`, and add:

```rust
    #[test]
    fn the_site_rides_the_third_uniform_and_leaves_amount_and_type_alone() {
        let blur = Blur::default();
        for (site, code) in [
            (NoiseSite::Glass, 0.),
            (NoiseSite::Backdrop, 1.),
            (NoiseSite::Film, 2.),
        ] {
            let glass = ResolvedGlass {
                noise: ResolvedNoise {
                    amount: Some(0.3),
                    kind: NoiseType::Fine,
                    site,
                },
                ..Default::default()
            };
            assert_eq!(triple(&glass, true, &blur), (0.3, 1., code), "{site:?}");
        }
    }
```

(import `NoiseSite` from `niri_config` in the test module).

- [x] **Step 5: Add the schema stages**

In `niri-config/src/material/pipeline.rs`, after `const fn optic`:

```rust
const fn effect_optic(name: &'static str, hook: &'static str) -> Option<OpticRef> {
    Some(OpticRef {
        name,
        hook,
        program: Program::Effect,
    })
}

/// A stage active only while an enum parameter picks it (design Section 2).
const fn select(stage: Stage, param: &'static str, variant: &'static str) -> Stage {
    Stage {
        selector: Some(Selector { param, variant }),
        ..stage
    }
}
```

In `STAGES`, insert as the first row (before `blur`):

```rust
    select(
        stage(
            "backdrop-grain",
            "source",
            Scope::Output,
            &[],
            &["noise", "noise type=", "noise site="],
            &[],
            effect_optic("noise", "source"),
            false,
        ),
        "noise site=",
        "backdrop",
    ),
```

Wrap the `noise` row (which Task 1 already gave `noise site=` in `owns` and `reads`) in `select(..., "noise site=", "glass")`:

```rust
    select(
        stage(
            "noise",
            "behind",
            Scope::Material,
            &["noise", "noise type=", "noise site="],
            &["noise", "noise type=", "noise site=", "blur noise", "backdrop-blur"],
            &[],
            optic("noise", "behind"),
            false,
        ),
        "noise site=",
        "glass",
    ),
```

Insert immediately before the `effect-saturation` row (line 489 today; it is the `background-effect` site's first row, ahead of `effect-noise`, and `stages_are_grouped_by_site_in_site_order` refuses any other position):

```rust
    select(
        stage(
            "film-grain",
            "post",
            Scope::Material,
            &[],
            &["noise", "noise type=", "noise site="],
            &[],
            optic("noise", "post"),
            false,
        ),
        "noise site=",
        "film",
    ),
```

- [x] **Step 6: Run every pin and regenerate the schema file**

Run: `just test-one -p niri-config pipeline` (the module's tests) and `just test-one -p niri pipeline_` and `just test-one -p niri grain_source` and `just test-one -p niri material_source` and `just test-one -p niri-config material_`
Expected: everything PASS except `material_pipeline_schema_matches_the_file` (stale).
Run: `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file`
Then `git diff resources/materials/pipeline.json`: two new stage objects (`backdrop-grain` first, `film-grain` before `effect-saturation`) and a `selector` on `noise`, each `"selector": {"param": "noise site=", "variant": ...}`; the `version` is unchanged.

- [x] **Step 7: Prove two drifts fail**

Comment out the `noise_post` call in `main.frag`; run `just test-one -p niri pipeline_material_hooks_are_called_exactly_once_in_stage_order`; expected FAIL naming `film-grain`. Restore the line. Delete `grain.frag`'s call (replace the body with `gl_FragColor = texture2D(tex, v_coords);`); run `just test-one -p niri pipeline_other_programs_call_their_hooks_exactly_once`; expected FAIL naming `backdrop-grain`. Restore it. Confirm `git status` shows no stray change.

- [x] **Step 8: Commit**

```bash
git add src/render_helpers/shaders/material/common.frag src/render_helpers/shaders/material/prelude.frag \
  src/render_helpers/shaders/material/noise.frag src/render_helpers/shaders/material/main.frag \
  src/render_helpers/shaders/grain.frag src/render_helpers/shaders/mod.rs \
  src/render_helpers/material/optics/noise.rs niri-config/src/material/pipeline.rs resources/materials/pipeline.json
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): noise sites in the shader and the schema; grain program source (material-cf32e5)"
```

---

### Task 3: The grain pass in the effect buffer

**Files:**
- Create: `src/render_helpers/grain.rs`
- Modify: `src/render_helpers/mod.rs` (declare `pub mod grain;` beside `pub mod blur;`)
- Modify: `src/render_helpers/effect_buffer.rs` (the buffer owns and lazily compiles its `GrainProgram`, so a failed compile retries after the next invalidation, as the spec's failure rule requires; `Shaders` is not touched)
- Modify: `src/niri.rs` (`update_xray_render_elements`, near line 4336)

**Interfaces:**
- Consumes: `grain_source()` (Task 2), `BackdropGrain`, `NoiseType`, `Config::backdrop_grain()` (Task 1).
- Produces: `pub struct GrainOptions { pub amount: f32, pub kind: NoiseType }` with `From<BackdropGrain>`; `pub struct GrainProgram` with `compile(&mut GlesRenderer) -> anyhow::Result<Self>`, `context_id(&self) -> ContextId<GlesTexture>` and `render(&self, &mut GlesRenderer, source: &GlesTexture, target: &GlesTexture, GrainOptions) -> anyhow::Result<()>`; `EffectBuffer::update_grain_options(&mut self, Option<GrainOptions>)`; Tracy spans `EffectBuffer::prepare_grain` (CPU) and `Grain::render` (GPU).

- [x] **Step 1: Write the failing invalidation table test**

In `effect_buffer.rs` tests:

```rust
    #[test]
    fn each_invalidation_clears_what_depends_on_it_and_publishes_damage() {
        use Invalidation::*;
        let all = Cleared {
            grain: true,
            blurred: true,
            sharp_pyramid: true,
            blurred_pyramid: true,
            publishes: true,
        };
        assert_eq!(cleared_by(SharpDamage, false), all);
        assert_eq!(cleared_by(SharpDamage, true), all);
        // Grain changes the source every consumer reads, blurred or not.
        assert_eq!(cleared_by(GrainOptionsChanged, false), all);
        assert_eq!(cleared_by(GrainOptionsChanged, true), all);
        // Blur options leave the sharp texture and its grain alone, and
        // publish only when a blurred texture existed to be stale.
        let blur_only = Cleared {
            grain: false,
            blurred: true,
            sharp_pyramid: false,
            blurred_pyramid: true,
            publishes: false,
        };
        assert_eq!(cleared_by(BlurOptionsChanged, false), blur_only);
        assert_eq!(
            cleared_by(BlurOptionsChanged, true),
            Cleared {
                publishes: true,
                ..blur_only
            }
        );
    }

    #[test]
    fn a_failed_grain_pass_retries_only_after_an_invalidation_that_clears_grain() {
        use Invalidation::*;
        assert_eq!(next_grain_status(GrainStatus::Failed, cleared_by(BlurOptionsChanged, true)), GrainStatus::Failed);
        assert_eq!(next_grain_status(GrainStatus::Failed, cleared_by(SharpDamage, true)), GrainStatus::Dirty);
        assert_eq!(next_grain_status(GrainStatus::Failed, cleared_by(GrainOptionsChanged, false)), GrainStatus::Dirty);
        assert_eq!(next_grain_status(GrainStatus::Clean, cleared_by(BlurOptionsChanged, true)), GrainStatus::Clean);
        assert_eq!(next_grain_status(GrainStatus::Clean, cleared_by(SharpDamage, false)), GrainStatus::Dirty);
    }

    #[test]
    fn grain_options_change_publishes_only_with_an_offscreen_and_never_for_equal_options() {
        let mut buffer = EffectBuffer::new();
        let before = buffer.commit();
        let grain = Some(GrainOptions {
            amount: 0.3,
            kind: NoiseType::Fine,
        });
        buffer.update_grain_options(grain);
        assert_eq!(buffer.commit(), before, "no offscreen yet, nothing to publish");
        buffer.update_grain_options(grain);
        assert_eq!(buffer.commit(), before, "equal options are a no-op");
        assert_eq!(buffer.grain, grain);
    }
```

Run: `just test-one -p niri each_invalidation` — expected: compile error (`Invalidation`, `Cleared`, `cleared_by`, `next_grain_status`, `GrainOptions` missing).

- [x] **Step 2: The grain program**

Create `src/render_helpers/grain.rs`:

```rust
//! The backdrop grain pass (`noise site=backdrop`, design
//! `2026-10-05-noise-placement-design.md` §4): one full-quad draw that grains
//! an output's sharp effect-buffer texture into a sibling texture, which the
//! blur, both prefilter pyramids and the direct sample then read.

use std::rc::Rc;

use anyhow::{ensure, Context as _};
use niri_config::{BackdropGrain, NoiseType};
use smithay::backend::renderer::gles::{ffi, link_program, GlesRenderer, GlesTexture};
use smithay::backend::renderer::{ContextId, Renderer as _, Texture as _};
use smithay::gpu_span_location;

use crate::render_helpers::shaders::grain_source;

/// The agreed backdrop grain, as the pass needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainOptions {
    pub amount: f32,
    pub kind: NoiseType,
}

impl From<BackdropGrain> for GrainOptions {
    fn from(grain: BackdropGrain) -> Self {
        Self {
            amount: grain.amount as f32,
            kind: grain.kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrainProgram(Rc<GrainProgramInner>);

#[derive(Debug)]
struct GrainProgramInner {
    program: ffi::types::GLuint,
    uniform_tex: ffi::types::GLint,
    uniform_amount: ffi::types::GLint,
    uniform_kind: ffi::types::GLint,
    attrib_vert: ffi::types::GLint,
    context_id: ContextId<GlesTexture>,
}

impl GrainProgram {
    pub fn compile(renderer: &mut GlesRenderer) -> anyhow::Result<Self> {
        let source = grain_source();
        let context_id = renderer.context_id();
        renderer
            .with_context(move |gl| unsafe {
                let program = link_program(gl, include_str!("shaders/blur.vert"), &source)
                    .context("error compiling grain shader")?;
                Ok(Self(Rc::new(GrainProgramInner {
                    program,
                    uniform_tex: gl.GetUniformLocation(program, c"tex".as_ptr()),
                    uniform_amount: gl.GetUniformLocation(program, c"mat_noise".as_ptr()),
                    uniform_kind: gl.GetUniformLocation(program, c"mat_noise_type".as_ptr()),
                    attrib_vert: gl.GetAttribLocation(program, c"vert".as_ptr()),
                    context_id,
                }))
            })
            .context("error making GL context current")?
    }

    /// The renderer context the program was linked in; a buffer drops the
    /// program when its renderer changes, as `Blur` does.
    pub fn context_id(&self) -> ContextId<GlesTexture> {
        self.0.context_id.clone()
    }

    /// Grains `source` into `target`, same size, one texel to one texel.
    pub fn render(
        &self,
        renderer: &mut GlesRenderer,
        source: &GlesTexture,
        target: &GlesTexture,
        options: GrainOptions,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("Grain::render");
        let size = target.size();
        ensure!(
            source.size() == size,
            "grain target size {size:?} differs from the source {:?}",
            source.size()
        );
        let p = &self.0;
        renderer.with_profiled_context(gpu_span_location!("Grain::render"), |gl| unsafe {
            while gl.GetError() != ffi::NO_ERROR {}
            gl.Disable(ffi::BLEND);
            gl.Disable(ffi::SCISSOR_TEST);
            gl.ActiveTexture(ffi::TEXTURE0);

            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);
            gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, fbo);
            gl.FramebufferTexture2D(
                ffi::DRAW_FRAMEBUFFER,
                ffi::COLOR_ATTACHMENT0,
                ffi::TEXTURE_2D,
                target.tex_id(),
                0,
            );
            gl.Viewport(0, 0, size.w, size.h);

            gl.UseProgram(p.program);
            gl.Uniform1i(p.uniform_tex, 0);
            gl.Uniform1f(p.uniform_amount, options.amount);
            gl.Uniform1f(p.uniform_kind, options.kind as u8 as f32);

            let vertices: [f32; 12] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
            gl.EnableVertexAttribArray(p.attrib_vert as u32);
            gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
            gl.VertexAttribPointer(
                p.attrib_vert as u32,
                2,
                ffi::FLOAT,
                ffi::FALSE,
                0,
                vertices.as_ptr().cast(),
            );

            gl.BindTexture(ffi::TEXTURE_2D, source.tex_id());
            // One texel in, one texel out: no filtering.
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MIN_FILTER, ffi::NEAREST as i32);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_MAG_FILTER, ffi::NEAREST as i32);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_S, ffi::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_T, ffi::CLAMP_TO_EDGE as i32);

            gl.DrawArrays(ffi::TRIANGLES, 0, 6);

            gl.DisableVertexAttribArray(p.attrib_vert as u32);
            gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, 0);
            gl.DeleteFramebuffers(1, &fbo);
        })?;
        Ok(())
    }
}
```

Declare `pub mod grain;` in `src/render_helpers/mod.rs`. The program is not registered in `Shaders`: it is compiled lazily by the buffer that needs it (Step 3), so a compile failure is retried after the next invalidation instead of being frozen at renderer start.

- [x] **Step 3: The effect buffer**

In `effect_buffer.rs`:

Add imports: `use crate::render_helpers::grain::{GrainOptions, GrainProgram};` (drop `GrainProgram` if unused after the step).

Add the pure cascade:

```rust
/// Why a cache is being thrown away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Invalidation {
    /// The sharp texture was redrawn.
    SharpDamage,
    /// The agreed backdrop grain changed, including to or from none.
    GrainOptionsChanged,
    /// The global blur passes or offset changed.
    BlurOptionsChanged,
}

/// What an invalidation clears, and whether consumers must be told through
/// the commit counter (design §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cleared {
    grain: bool,
    blurred: bool,
    sharp_pyramid: bool,
    blurred_pyramid: bool,
    publishes: bool,
}

const fn cleared_by(cause: Invalidation, had_blurred: bool) -> Cleared {
    match cause {
        // Everything hangs off the sharp texture; grain changes the source
        // every consumer reads, blurred or not.
        Invalidation::SharpDamage | Invalidation::GrainOptionsChanged => Cleared {
            grain: true,
            blurred: true,
            sharp_pyramid: true,
            blurred_pyramid: true,
            publishes: true,
        },
        // Blur options leave the sharp texture and its grain alone; there is
        // nothing to publish unless a blurred texture existed to go stale.
        Invalidation::BlurOptionsChanged => Cleared {
            grain: false,
            blurred: true,
            sharp_pyramid: false,
            blurred_pyramid: true,
            publishes: had_blurred,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrainStatus {
    /// Needs the pass (no grained texture, or a stale one); also where a
    /// failed compile or draw is retried.
    Dirty,
    /// `grain_texture` holds the sharp texture grained with the current options.
    Clean,
    /// The compile or the pass failed since the last invalidation that
    /// cleared grain; the sharp texture stands in until then.
    Failed,
}

/// The grain status after an invalidation: anything that clears grain
/// returns it to `Dirty`, which is also how a failure gets its retry.
const fn next_grain_status(status: GrainStatus, cleared: Cleared) -> GrainStatus {
    if cleared.grain {
        GrainStatus::Dirty
    } else {
        status
    }
}
```

`EffectBuffer` gains `grain: Option<GrainOptions>` and `grain_program: Option<GrainProgram>` (both `None` in `new`). `Offscreen` gains `grain_texture: Option<GlesTexture>` and `grain: GrainStatus` (initialised `None` / `GrainStatus::Dirty` where the offscreen is built, near line 378).

Add to `Offscreen`:

```rust
impl Offscreen {
    /// What every consumer reads: the grained texture while grain is set and
    /// prepared, the sharp texture otherwise.
    fn source(&self, grain: Option<GrainOptions>) -> &GlesTexture {
        match (grain, self.grain, &self.grain_texture) {
            (Some(_), GrainStatus::Clean, Some(texture)) => texture,
            _ => &self.texture,
        }
    }

    fn clear(&mut self, cleared: Cleared) {
        self.grain = next_grain_status(self.grain, cleared);
        if cleared.blurred {
            self.blurred = None;
        }
        if cleared.sharp_pyramid {
            self.sharp_prefilter.invalidate();
        }
        if cleared.blurred_pyramid {
            self.blurred_prefilter.invalidate();
        }
    }
}
```

Add to `EffectBuffer`:

```rust
    fn invalidate(&mut self, cause: Invalidation) {
        let Some(offscreen) = &mut self.offscreen else {
            return;
        };
        let cleared = cleared_by(cause, offscreen.blurred.is_some());
        offscreen.clear(cleared);
        if cleared.publishes {
            self.commit_counter.increment();
        }
    }

    pub fn update_grain_options(&mut self, options: Option<GrainOptions>) {
        if self.grain == options {
            return;
        }
        self.grain = options;
        self.invalidate(Invalidation::GrainOptionsChanged);
    }
```

Rewrite `update_blur_options` to the same shape:

```rust
    pub fn update_blur_options(&mut self, options: BlurOptions) {
        if self.blur_options == options {
            return;
        }
        self.blur_options = options;
        self.invalidate(Invalidation::BlurOptionsChanged);
    }
```

In `prepare_offscreen`, the `if res.damage.is_some()` branch becomes:

```rust
        if res.damage.is_some() {
            // Original texture changed; everything derived from it is stale.
            let cleared = cleared_by(Invalidation::SharpDamage, true);
            offscreen.clear(cleared);
            self.commit_counter.increment();
        }
```

(the branch already incremented unconditionally; `cleared.publishes` is `true` for this cause, so the increment stays unconditional.) The renderer-context and scale branches that call `sharp_prefilter.invalidate(); blurred_prefilter.invalidate();` (near lines 399 and 430) also set `offscreen.grain = GrainStatus::Dirty` and `offscreen.blurred = None` where they do not already; the recreated-offscreen path builds a fresh `Offscreen` and needs nothing.

In `prepare`, after `prepare_offscreen` succeeds and before the blur:

```rust
        self.prepare_grain(renderer);
```

with

```rust
    /// Runs the grain pass when grain is set and the grained texture is
    /// stale. A failure is logged once per invalidation and the sharp
    /// texture stands in until the next one (the prefilter's pattern).
    fn prepare_grain(&mut self, renderer: &mut GlesRenderer) {
        let Some(options) = self.grain else {
            return;
        };
        let Some(offscreen) = self.offscreen.as_mut() else {
            return;
        };
        if offscreen.grain != GrainStatus::Dirty {
            return;
        }
        let _span = tracy_client::span!("EffectBuffer::prepare_grain");
        // The program is the buffer's, compiled on first need and after a
        // renderer change; a failed compile is retried the next time grain
        // is Dirty, which the invalidation table decides.
        if self
            .grain_program
            .as_ref()
            .is_some_and(|p| p.context_id() != renderer.context_id())
        {
            self.grain_program = None;
        }
        let program = match &self.grain_program {
            Some(program) => program.clone(),
            None => match GrainProgram::compile(renderer) {
                Ok(program) => self.grain_program.insert(program).clone(),
                Err(err) => {
                    offscreen.grain = GrainStatus::Failed;
                    warn!("backdrop grain shader failed to compile; the sharp texture stands in until the next damage: {err:?}");
                    return;
                }
            },
        };
        let result = (|| -> anyhow::Result<()> {
            let size = offscreen.texture.size();
            let reusable = offscreen
                .grain_texture
                .as_mut()
                .is_some_and(|t| t.size() == size && t.is_unique_reference());
            if !reusable {
                offscreen.grain_texture = Some(
                    renderer
                        .create_buffer(Fourcc::Abgr8888, size)
                        .context("error creating grain texture")?,
                );
            }
            let target = offscreen.grain_texture.as_ref().expect("just ensured");
            program.render(renderer, &offscreen.texture, target, options)
        })();
        match result {
            Ok(()) => offscreen.grain = GrainStatus::Clean,
            Err(err) => {
                offscreen.grain = GrainStatus::Failed;
                warn!("backdrop grain pass failed; the sharp texture stands in until the next damage: {err:?}");
            }
        }
    }
```

In `prepare_blur`, pass `offscreen.source(self.grain)` to `prepare_textures` instead of `&offscreen.texture` (bind `let grain = self.grain;` before the mutable borrows).

In `render`:

```rust
        if !blur {
            return Ok(offscreen.source(self.grain).clone());
        }
        ...
            let blurred = blur
                .render(renderer, offscreen.source(self.grain), self.blur_options)
                .context("error rendering blur")?;
```

`render_prefiltered` obtains its source through `self.render` and needs no change; check with `grep -n 'offscreen.texture' src/render_helpers/effect_buffer.rs` that the remaining uses are the creation, the damage render target, `prepare_grain`'s input and the size reads.

In `src/niri.rs`, in `update_xray_render_elements`, after `let blur_options = ...`:

```rust
                let grain_options = self
                    .config
                    .borrow()
                    .backdrop_grain()
                    .map(GrainOptions::from);
```

and in both `for buf in ...` loops, after `update_blur_options`: `buffer.update_grain_options(grain_options);` (import `crate::render_helpers::grain::GrainOptions`).

(`offscreen` and `self.grain_program` are disjoint borrows of `self`; take `let grain = self.grain;` and the offscreen as `self.offscreen.as_mut()` before touching `self.grain_program`, or split the compile into a small `fn grain_program(&mut self, renderer) -> Option<GrainProgram>` called first.)

- [x] **Step 4: Run the unit tests and the material tests**

Run: `just test-one -p niri each_invalidation` and `just test-one -p niri a_failed_grain_pass` and `just test-one -p niri grain_options_change` and `just test-one -p niri prefilter_` and `just test-one -p niri material`
Expected: all PASS. The in-process renders of Task 4 are the first to compile the grain program on a GPU; a compile failure there appears as the "failed to compile" warning and a `Failed` status, never a panic. If the warning appears, fix the GLSL before moving on.

- [x] **Step 5: Commit**

```bash
git add src/render_helpers/grain.rs src/render_helpers/mod.rs src/render_helpers/effect_buffer.rs src/niri.rs
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "feat(render): backdrop grain pass in the effect buffer, published through the commit counter (material-cf32e5)"
```

---

### Task 4: In-process pixel and damage tests

**Files:**
- Create: `src/tests/noise_site.rs`
- Modify: `src/tests/mod.rs` (`mod noise_site;`)
- Modify: `src/tests/client.rs` (a coloured layer buffer helper beside `attach_new_buffer` on the layer surface, near line 488; channels scaled from bytes to the single-pixel protocol's `u32` range)
- Modify: `src/render_helpers/material/mod.rs` (`MaterialState::commit()` accessor)

**Interfaces:**
- Consumes: `render_at`, `set_time`, `diff` from `ring_pair.rs` (`pub(super)`); `Fixture`; `tile.material()` and `MaterialState::material().name`; `Niri::output_state[...].xray.background`.
- Produces: nothing other tasks consume; Task 5's smoke mirrors the identities.

- [x] **Step 1: Helpers**

In `src/tests/client.rs`, on the layer surface impl beside `attach_new_buffer`:

```rust
    /// A single-pixel buffer of this 8-bit colour; `set_size` stretches it
    /// over the surface through the viewport, which is how a test paints a
    /// background layer the effect buffer can sample. The single-pixel
    /// protocol takes each channel as `0..=u32::MAX`, so a byte is scaled.
    pub fn attach_new_colored_buffer(&self, r: u8, g: u8, b: u8, a: u8) {
        let scale = |v: u8| u32::from(v) * (u32::MAX / 255);
        let buffer = self
            .spbm
            .create_u32_rgba_buffer(scale(r), scale(g), scale(b), scale(a), &self.qh, ());
        self.surface.attach(Some(&buffer), 0, 0);
    }
```

In `src/render_helpers/material/mod.rs`, on `MaterialState`:

```rust
    /// The element commit `advance_commit` last returned; tests read it to
    /// see that a changed input was noticed.
    pub fn commit(&self) -> CommitCounter {
        self.commit.get()
    }
```

- [x] **Step 2: The fixture and the first failing test**

Create `src/tests/noise_site.rs`:

```rust
//! Noise placement (design 2026-10-05-noise-placement-design.md §5, §8): the
//! three sites' identities, the backdrop grain's softening under blur, the
//! coverage facts, and the damage contract, rendered in process through the
//! headless GLES renderer under a frozen clock.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Rectangle};

use super::client::LayerConfigureProps;
use super::fixture::Fixture;
use super::ring_pair::{diff, render_at, set_time};
use crate::render_helpers::RenderTarget;

// Two 320 px windows, three 16 px gaps: 688 px, inside the 800 px output,
// so neither window is pushed offscreen when the second takes focus.
const OUT_W: u16 = 800;
const OUT_H: u16 = 480;
const W: u16 = 320;
const H: u16 = 240;
const BEVEL: u16 = 12;
const OFFSET: u16 = 6;

/// `noise` lines for the focused material (`a`) and the second one (`b`).
struct Look<'a> {
    noise_a: &'a str,
    noise_b: &'a str,
    ior: &'a str,
    backdrop_blur: bool,
    blur_passes: u8,
    /// The second window's rule: a material, or a background-effect blur with no material.
    second_rule: &'a str,
}

impl Default for Look<'static> {
    fn default() -> Self {
        Self {
            noise_a: "",
            noise_b: "",
            ior: "1",
            backdrop_blur: false,
            blur_passes: 3,
            second_rule: "material \"b\"",
        }
    }
}

fn glass(noise: &str, ior: &str, backdrop_blur: bool) -> String {
    format!(
        r##"
            glass {{
                ior {ior}
                thickness 20
                bevel {BEVEL}
                offset-x {OFFSET}
                offset-y {OFFSET}
                attenuation-color "#ffffff"
                attenuation-distance 60
                chromatic-aberration 0
                anisotropic-blur 0
                distortion 0 scale=0.5
                roughness 0
                backdrop-blur {backdrop_blur}
                jelly-flex 0
                jelly-ripple 0
                saturation 1
                aurora 0 {{ drift-hz 0; }}
                iridescence 0
                {noise}
            }}
            response "default" {{ focus "none"; accent "none"; ring-beam-speed 0; }}
        "##
    )
}

fn config(look: &Look) -> Config {
    let a = glass(look.noise_a, look.ior, look.backdrop_blur);
    let b = glass(look.noise_b, look.ior, look.backdrop_blur);
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 16
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
            background-color "transparent"
        }}
        animations {{ off; }}
        blur {{ passes {passes}; offset 3; noise 0; saturation 1; }}
        material "a" {{ {a} }}
        material "b" {{ {b} }}
        window-rule {{
            match is-active=true
            material "a"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        window-rule {{
            match is-active=false
            {second}
        }}
        "##,
        passes = look.blur_passes,
        second = look.second_rule,
    ))
    .unwrap()
}

/// Two transparent windows over a warm mid-tone background layer that fills
/// the output. Window 1 (mapped first, then window 2 takes focus) is the
/// second rule's; window 2 is active and gets material `a`.
fn fixture(look: &Look) -> Fixture {
    let mut f = Fixture::with_config(config(look));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));

    let bg = f.add_client();
    let layer = f.client(bg).create_layer(None, Layer::Background, "");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right),
        size: Some((0, 0)),
        exclusive_zone: Some(-1),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(bg);
    let layer = f.client(bg).layer(&surface);
    layer.attach_new_colored_buffer(140, 115, 90, 255);
    layer.set_size(OUT_W, OUT_H);
    layer.ack_last_and_commit();
    f.double_roundtrip(bg);

    for _ in 0..2 {
        let id = f.add_client();
        let window = f.client(id).create_window();
        let surface = window.surface.clone();
        window.commit();
        f.roundtrip(id);
        let window = f.client(id).window(&surface);
        window.attach_new_shm_buffer(0);
        window.set_size(W, H);
        window.ack_last_and_commit();
        f.double_roundtrip(id);
        f.niri_state().update_keyboard_focus();
        f.double_roundtrip(id);
    }
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

fn reload(f: &mut Fixture, look: &Look) {
    f.niri_state().reload_config(Ok(config(look)));
    f.niri_state().refresh_and_flush_clients();
}

/// The rectangle of the tile whose material has this name, or of the tile
/// with no material when `name` is `None`. `window_rect` in ring_pair.rs
/// takes the first tile, which here is the second rule's window, so every
/// test names the window it means.
fn tile_rect(f: &mut Fixture, name: Option<&str>) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.material().map(|m| m.material().name.as_str()) == name)
        .unwrap_or_else(|| panic!("no tile with material {name:?}"));
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

/// RGBA pixels of the output, and material `a`'s flat face in output px.
fn render(f: &mut Fixture) -> (Vec<u8>, Rectangle<i32, Logical>) {
    let pixels = render_at(f, Duration::ZERO);
    let rect = tile_rect(f, Some("a"));
    let inset = f64::from(BEVEL + OFFSET) + 2.;
    let face = Rectangle::new(
        (rect.loc + smithay::utils::Point::from((inset, inset))).to_i32_round(),
        (rect.size - smithay::utils::Size::from((2. * inset, 2. * inset))).to_i32_round(),
    );
    (pixels, face)
}

fn region_pixels<'a>(pixels: &'a [u8], rect: Rectangle<i32, Logical>) -> impl Iterator<Item = &'a [u8]> {
    // Clip to the output: a region that reaches past it would index out of
    // range (or wrap, without overflow checks), and an empty one would make
    // every mean a division by zero.
    let output = Rectangle::from_size(smithay::utils::Size::from((i32::from(OUT_W), i32::from(OUT_H))));
    let rect = rect
        .intersection(output)
        .unwrap_or_else(|| panic!("region {rect:?} lies outside the {OUT_W}x{OUT_H} output"));
    let w = usize::from(OUT_W);
    (rect.loc.y..rect.loc.y + rect.size.h).flat_map(move |y| {
        (rect.loc.x..rect.loc.x + rect.size.w).map(move |x| {
            let i = (y as usize * w + x as usize) * 4;
            &pixels[i..i + 4]
        })
    })
}

/// Max channel delta and mean absolute delta over the RGB of a region.
fn region_diff(a: &[u8], b: &[u8], rect: Rectangle<i32, Logical>) -> (u8, f64) {
    let mut max = 0u8;
    let mut sum = 0u64;
    let mut n = 0u64;
    for (pa, pb) in region_pixels(a, rect).zip(region_pixels(b, rect)) {
        for c in 0..3 {
            let d = pa[c].abs_diff(pb[c]);
            max = max.max(d);
            sum += u64::from(d);
            n += 1;
        }
    }
    (max, sum as f64 / n as f64)
}

/// Standard deviation of the signed green-channel difference over a region.
fn region_sd(a: &[u8], b: &[u8], rect: Rectangle<i32, Logical>) -> f64 {
    let diffs: Vec<f64> = region_pixels(a, rect)
        .zip(region_pixels(b, rect))
        .map(|(pa, pb)| f64::from(pa[1]) - f64::from(pb[1]))
        .collect();
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    (diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / diffs.len() as f64).sqrt()
}

#[test]
fn omitted_site_equals_glass_pixel_for_pixel() {
    for kind in ["white", "fine", "lightness"] {
        let omitted = Look {
            noise_a: &format!("noise 0.3 type=\"{kind}\""),
            ..Default::default()
        };
        let mut f = fixture(&omitted);
        let (a, _) = render(&mut f);
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"glass\""),
                ..Default::default()
            },
        );
        let (b, _) = render(&mut f);
        assert_eq!(diff(&a, &b), (0, 0), "{kind}");
    }
}
```

Run: `just test-one -p niri omitted_site_equals_glass`
Expected: before Tasks 1–3 this would not compile; with them in place it PASSES. Confirm it also discriminates: temporarily change the second look's amount to `0.31`, run, expect FAIL, restore.

- [x] **Step 3: The remaining tests**

Append to `noise_site.rs`:

```rust
#[test]
fn amount_zero_is_neutral_at_every_site() {
    let mut f = fixture(&Look::default());
    let (none, _) = render(&mut f);
    for site in ["glass", "backdrop", "film"] {
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0 type=\"fine\" site=\"{site}\""),
                ..Default::default()
            },
        );
        let (zero, _) = render(&mut f);
        assert_eq!(diff(&none, &zero), (0, 0), "{site}");
    }
}

#[test]
fn film_equals_glass_on_the_face_within_one_code() {
    for kind in ["white", "fine", "lightness"] {
        let mut f = fixture(&Look {
            noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"glass\""),
            ..Default::default()
        });
        let (glass, face) = render(&mut f);
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"film\""),
                ..Default::default()
            },
        );
        let (film, _) = render(&mut f);
        let (max, _) = region_diff(&glass, &film, face);
        assert!(max <= 1, "{kind}: film differs from glass by {max} codes on the face");
        let (none, _) = {
            reload(&mut f, &Look::default());
            render(&mut f)
        };
        assert!(region_sd(&film, &none, face) > 5., "{kind}: film grain is present");
    }
}

#[test]
fn backdrop_equals_glass_on_the_face_at_blur_off_within_two_codes() {
    let mut f = fixture(&Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"glass\"",
        ..Default::default()
    });
    let (glass, face) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let (backdrop, _) = render(&mut f);
    let (max, mean) = region_diff(&glass, &backdrop, face);
    assert!(
        mean <= 2.,
        "backdrop grain differs from glass grain by {mean:.2} codes on average (max {max}); \
         a mean near the grain's own spread means the seeds do not land on the same pixels"
    );
}

#[test]
fn backdrop_grain_softens_under_three_blur_passes() {
    let sharp = Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
        ..Default::default()
    };
    let mut f = fixture(&sharp);
    let (grained, face) = render(&mut f);
    reload(&mut f, &Look::default());
    let (plain, _) = render(&mut f);
    let sd_sharp = region_sd(&grained, &plain, face);

    let frosted = Look {
        backdrop_blur: true,
        ..sharp
    };
    reload(&mut f, &frosted);
    let (grained, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            backdrop_blur: true,
            ..Default::default()
        },
    );
    let (plain, _) = render(&mut f);
    let sd_frosted = region_sd(&grained, &plain, face);
    assert!(sd_sharp > 5., "sharp backdrop grain is present: sd {sd_sharp:.2}");
    assert!(
        sd_frosted < 0.5 * sd_sharp,
        "three passes should remove at least half the grain: {sd_frosted:.2} vs {sd_sharp:.2}"
    );
}

#[test]
fn backdrop_to_glass_reload_clears_the_grain() {
    let glass_look = Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&glass_look);
    let (glass, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let _ = render(&mut f);
    reload(&mut f, &glass_look);
    let (again, _) = render(&mut f);
    assert_eq!(diff(&glass, &again), (0, 0), "glass after backdrop equals glass before it");
}

#[test]
fn backdrop_grain_leaves_transparent_texels_alone() {
    // The background layer is anchored to the top half only, so the effect
    // buffer is transparent under the lower window.
    let mut f = fixture(&Look::default());
    // Re-anchor: map a second, half-height layer and unmap the first by
    // reconfiguring it to size 0 is more than this test needs; instead
    // compare the band below the layer directly: region below OUT_H/2 of
    // a window whose rect crosses it. Simplest honest form: a fixture whose
    // layer covers only the top half.
    let _ = &mut f;
}
```

Replace the last test's placeholder body with a real half-height fixture: factor `fixture` into `fixture_with_layer(look, layer_h: u16)` where the layer's `set_size(OUT_W, layer_h)` and `anchor: Top | Left | Right`, `size: Some((0, layer_h as u32))`; `fixture(look)` calls it with `OUT_H`. Then:

```rust
#[test]
fn backdrop_grain_leaves_transparent_texels_alone() {
    let half = OUT_H / 2;
    let mut f = fixture_with_layer(&Look::default(), half);
    let (plain, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let (grained, _) = render(&mut f);
    let below = Rectangle::new(
        smithay::utils::Point::from((0, i32::from(half) + 2)),
        smithay::utils::Size::from((i32::from(OUT_W), i32::from(OUT_H - half) - 2)),
    );
    assert_eq!(region_diff(&plain, &grained, below), (0, 0.), "no layer, no grain");
    let above = Rectangle::new(
        smithay::utils::Point::from((0, 0)),
        smithay::utils::Size::from((i32::from(OUT_W), i32::from(half) - 2)),
    );
    assert!(region_sd(&plain, &grained, above) > 0., "the layer's half is grained where glass covers it");
}

#[test]
fn window_site_element_sees_backdrop_grain() {
    let plain = Look {
        second_rule: "background-effect { blur true; noise 0; saturation 1; }",
        ..Default::default()
    };
    let mut f = fixture(&plain);
    let (before, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..plain
        },
    );
    let (after, _) = render(&mut f);
    // The window with no material has the blurred background effect; its
    // area must change with the grain.
    let rect = tile_rect(&mut f, None).to_i32_round();
    assert!(
        region_sd(&before, &after, rect) > 0.5,
        "the background-effect element under a window with no material samples the grained backdrop"
    );
}

#[test]
fn a_backdrop_only_reload_rerenders_the_unchanged_glass_window() {
    let look = Look {
        noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"",
        noise_b: "noise 0.1 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&look);
    let _ = render(&mut f);
    let read = |f: &mut Fixture| {
        let output = f.niri_output(1);
        let niri = f.niri();
        let buffer = niri.output_state[&output].xray.background[RenderTarget::ScreenCapture as usize]
            .borrow()
            .commit();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let glass_window = workspace
            .tiles_with_render_positions()
            .find_map(|(tile, _, _)| tile.material().filter(|m| m.material().name == "b"))
            .expect("material b")
            .commit();
        (buffer, glass_window)
    };
    let (buffer_0, window_0) = read(&mut f);

    // Same config again: nothing moves.
    reload(&mut f, &look);
    let _ = render(&mut f);
    let (buffer_1, window_1) = read(&mut f);
    assert_eq!(buffer_1, buffer_0, "an unchanged backdrop grain publishes nothing");
    assert_eq!(window_1, window_0, "an unchanged glass window keeps its commit");

    // Only the backdrop amount changes; material `b` is byte-identical.
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.4 type=\"fine\" site=\"backdrop\"",
            ..look
        },
    );
    let _ = render(&mut f);
    let (buffer_2, window_2) = read(&mut f);
    assert_ne!(buffer_2, buffer_1, "the effect buffer publishes the grain change");
    assert_ne!(window_2, window_1, "the glass window's fingerprint carries the buffer's commit");
}
```

Note `niri.output_state` and `xray` are `pub` (`src/niri.rs:261`, `:491`); `RenderTarget` is in `crate::render_helpers`.

Register the module in `src/tests/mod.rs` (`mod noise_site;`, alphabetical).

- [x] **Step 4: Run, then prove the damage test needs the increment**

Run: `just test-one -p niri noise_site`
Expected: all PASS. If `backdrop_equals_glass_on_the_face_at_blur_off_within_two_codes` fails with a mean near the grain's own spread (around 8 to 12 codes at amount 0.3), the backdrop seed is vertically flipped relative to the screen; the fix is in `grain.frag` and `grain.rs`: add `uniform float grain_height;` to the effect header in `grain_source()`, set it from `size.h as f32` in `GrainProgram::render`, and seed with `vec2(gl_FragCoord.x, grain_height - gl_FragCoord.y)`. Rerun; record which orientation held in the commit message. If the mean stays above 2 with the seeds aligned, the identity claim of §5 (c) does not hold as written: stop, record the measured mean in a task note, and raise it at the whole-branch review rather than loosening the bound.

Then, in `effect_buffer.rs`, temporarily change `cleared_by`'s `GrainOptionsChanged` arm to `publishes: false` (move it to its own arm). Run `just test-one -p niri a_backdrop_only_reload_rerenders_the_unchanged_glass_window` and `just test-one -p niri each_invalidation`: both must FAIL. Restore the arm, rerun, PASS.

- [x] **Step 5: Commit**

```bash
git add src/tests/noise_site.rs src/tests/mod.rs src/tests/client.rs src/render_helpers/material/mod.rs
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "test(material): noise site identities, softening, coverage and damage, in process (material-cf32e5)"
```

---

### Task 5: Docs, the nested-Weston smoke against a baseline binary, and the evidence document

**Files:**
- Create: `docs/materials/scripts/glass-noise-site-smoke.sh`
- Create: `docs/materials/2026-10-<dd>-noise-placement-evidence.md` (dated the day it runs)
- Modify: `docs/materials/render-pipeline.md` (§1 step 2, §3 rows 3b and 9, §4 bullets, §5 table)
- Modify: `docs/materials/material-config.md` (the noise paragraphs and `### noise`)
- Modify: `docs/materials/adding-an-optic.md` (one paragraph on several placements)
- Modify: `docs/materials/README.md` (index entries for the spec and the evidence)

**Interfaces:**
- Consumes: the implementation of Tasks 1–4; `docs/materials/scripts/glass-optic-smoke-lib.sh` (`capture_preflight`, `build_binaries`, `start_nested`, `shot`, `roi`, `sd`, `ae`, `assert_*`, `finish`); a baseline binary built from `b261ad1a`.
- Produces: the evidence document §7.2 (seven assertions) and the §7.1 record.

- [ ] **Step 1: Documentation**

`render-pipeline.md`:
- §1 step 2: after "its elements into a full-output offscreen (the *sharp* texture);" insert "when any material places `noise` at the `backdrop` site, a grain pass over that texture (`grain.rs`, `grain.frag`), cached with it and shared by every consumer below;".
- §3 row 3b: "**Behind: noise (glass site).** Grain the averaged backdrop once ... Inert while `noise site=` is `backdrop` or `film`." and add `noise site=` to its parameters.
- §3 row 9: `| 9 | **Post: film grain.** \`noise_post\` grains the encoded glass, over transmitted light, ring, aurora, glint and sweeps, before coverage; active only while \`noise site=\` is \`film\`. The same formulas and seed as 3b, in encoding. | — | \`noise\`, \`noise type=\`, \`noise site=\`; neutral 0 |`.
- §4: replace the "Two noise sites exist" bullet with: "**Noise has three placements and a fourth site that is not the material's.** `noise site=` puts the material's grain before the blur (`backdrop`, one texture per output, every glass window and the background-effect element see it, blur and roughness soften it), on the transmitted backdrop (`glass`, today's), or over the finished glass (`film`, grains the light too, glass coverage only). `postprocess.frag`'s `blur { noise }` is the window-site element's own grain and is unchanged. A design that changes 'the noise' must say which."
- §5 table: add `| \`noise\` \`site=\` | \`glass.noiseSite\` (pending, prism-be5abe) | source grain pass / 3b / 9 |`.

`material-config.md`, in the paragraph beginning "`noise` takes an optional `type=`", append a paragraph:

"`noise` also takes `site="glass"` (default), `"backdrop"` or `"film"`. `backdrop` grains the output's shared effect-buffer texture before the Kawase blur and the roughness pyramids: one amount and one type per output, so every material that places noise there must agree, and a config with two that differ fails to load naming both (`materials "a" and "b" both place noise at the backdrop with different settings ...`). It applies whether or not `blur { off }` is set, and a window whose rule asks for a blurred `background-effect` shows the grained backdrop too. `film` grains the finished, encoded glass after ring, aurora, glint and sweeps, on glass coverage only. Grain softens under blur at the backdrop site: see `2026-10-<dd>-noise-placement-evidence.md`."

`### noise` subsection: replace the first sentence with "Stage 3b (`behind`) at `site="glass"`; the source-site grain pass at `site="backdrop"`; stage 9 (`post`) at `site="film"`. `noise <amount> type=<type> site=<site>` ..." and add "The site never inherits."

`adding-an-optic.md`, at the end of section 3 before section 4: "An optic with several placements has one stage per placement in `pipeline.rs`, each with a `selector` on the enum parameter that picks it (`select(stage(..), "noise site=", "film")`), one hook function per placement, and, for a placement in another program, that program's hook called in that program's file (`noise_source` in `grain.frag`). `check_selectors` requires every variant to select exactly one stage and all of them to be one optic."

`README.md`: add index lines for the design spec and the evidence document in the materials list, in the existing one-line style.

- [ ] **Step 2: The smoke**

Create `docs/materials/scripts/glass-noise-site-smoke.sh` on `glass-optic-smoke-lib.sh`, the way `glass-render-order-smoke.sh` is built: `source "$(dirname "$0")/glass-optic-smoke-lib.sh"`, then `capture_preflight headless`, `build_binaries`, `capture_identity`. Required env: `OUT`, `BASE_NIRI` (the baseline release binary, Step 3), `CAPTURE_TASK=material-cf32e5`, and the lib's `NIRI_MATERIAL_WORK_ROOT`. The lib's API, used as it is defined: `write_config <path>` reads `GLASS_EXTRA` (lines appended inside `glass {}`; a key already in `GLASS_BASELINE` is replaced), `TOP_EXTRA` (top-level nodes), `FOCUS_RESPONSE`, `RESPONSE_EXTRA`; `start_nested <niri> <kdl>`; `spawn_probe <niri> "$IDLE"` (the second argument is the shell command the probe runs); `calibrate_probe_rect <niri> 0` once before any material capture (it measures the one-window layout and writes `probe-rect-0.txt`); `probe_rect <niri>` after each spawn, which sets `PX PY PW PH` for `face_roi`; `shot <niri> <name>`; `roi <name> <geometry> <suffix>`; `sd`, `mean`, `ae`, `rmse` (result in `METRIC`); `assert_zero`, `assert_less`, `assert_greater`, `assert_about <name> <got> <want> <relative tolerance>`; `finish`. `signed_diff` is not in the lib: define it locally as `glass-noise-type-smoke.sh` does.

Wallpaper: `WALL` is set before `write_config` is first called; the identity and roughness fixtures use `magick -size 1280x720 xc:'rgb(140,115,90)'` with one textured quadrant composited (`-seed 11 plasma:fractal` at 400x300 in the lower right), the ring fixture a flat `xc:'rgb(128,128,128)'` (encoded 0.5).

```bash
IDENTITY=$'ior 1\nattenuation-color "#ffffff"\nsaturation 1'
blur_top() { printf 'blur { passes %s; offset 3; noise 0; saturation 1; }' "$1"; }
# Every cell is captured twice in its session and the two must be identical
# (spec §7.2 assertion 1); the second capture is kept as <name>-again.png.
shot_twice() {   # $1 binary, $2 name
    shot "$1" "$2"
    sleep 1
    shot "$1" "$2-again"
    ae "$OUT/$2.png" "$OUT/$2-again.png"
    assert_zero "$2 determinism" "$METRIC"
    printf '%s_determinism_ae=%s\n' "$2" "$METRIC" >> "$OUT/metrics.txt"
}
cell() {   # $1 name, $2 binary, $3 glass extra, $4 blur passes
    GLASS_EXTRA="$IDENTITY"$'\n'"$3" TOP_EXTRA=$(blur_top "$4") write_config "$OUT/$1.kdl"
    start_nested "$2" "$OUT/$1.kdl"
    spawn_probe "$2" "$IDLE"
    probe_rect "$2"
    sleep 2
    shot_twice "$2" "$1"
    roi "$1" "$(face_roi)" face
    stop_nested
}
calibrate_probe_rect "$NIRI" 0

# The spec's matrix (§7.2): site × blur × kind on the identity fixture.
for kind in white fine lightness; do
    cell "omitted-$kind"       "$NIRI"      "noise 0.3 type=\"$kind\""                      3
    cell "baseline-$kind"      "$BASE_NIRI" "noise 0.3 type=\"$kind\""                      3
    for site in glass backdrop film; do
        cell "$site-$kind-off" "$NIRI" "noise 0.3 type=\"$kind\" site=\"$site\""             3
    done
done
cell "zero-off" "$NIRI" "noise 0" 3
for p in 1 3; do
    cell "zero-p$p" "$NIRI" $'backdrop-blur true\nnoise 0' "$p"
    for site in glass backdrop film; do
        cell "$site-fine-p$p" "$NIRI" $'backdrop-blur true\nnoise 0.3 type="fine" site="'"$site"'"' "$p"
    done
done
# Roughness fixture: ior 1.5, so the normalized level is roughness.
ROUGH=$'ior 1.5\nattenuation-color "#ffffff"\nsaturation 1'
rough_cell() {   # $1 name, $2 glass extra, $3 passes
    GLASS_EXTRA="$ROUGH"$'\n'"$2" TOP_EXTRA=$(blur_top "$3") write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"; spawn_probe "$NIRI" "$IDLE"; probe_rect "$NIRI"; sleep 2
    shot_twice "$NIRI" "$1"; roi "$1" "$(face_roi)" face; stop_nested
}
for blur in false true; do for r in 0 0.5 1; do
    tag="b$blur-r$r"
    rough_cell "rough-zero-$tag" $'backdrop-blur '"$blur"$'\nroughness '"$r"$'\nnoise 0' 3
    for site in glass backdrop; do
        rough_cell "rough-$site-$tag" $'backdrop-blur '"$blur"$'\nroughness '"$r"$'\nnoise 0.3 type="fine" site="'"$site"'"' 3
    done
done; done
```

Metrics (each appended to `$OUT/metrics.txt` as `name=value`) and the seven assertions:

1. Determinism is asserted inside `shot_twice` for every cell of every fixture (identity, blur, roughness, ring), so no cell is captured only once.
2. `ae "$OUT/omitted-$kind.png" "$OUT/glass-$kind-off.png"` → `assert_zero` per kind.
3. `ae "$OUT/omitted-$kind.png" "$OUT/baseline-$kind.png"` → `assert_zero` per kind: byte identity against the baseline binary.
4. `magick compare -metric MAE "$OUT/glass-fine-off-face.png" "$OUT/backdrop-fine-off-face.png" null:` (through a local `mae()` built like `compare_metric`) → `assert_less "backdrop vs glass face MAE" "$METRIC" "$(magick xc: -format '%[fx:2*quantumrange/255]' info:)"`.
5. Grain `sd`: `signed_diff "$OUT/$site-fine-$b-face.png" "$OUT/zero-$b-face.png" "$OUT/grain-$site-$b.png"; sd "$OUT/grain-$site-$b.png"` for `b` in `off p1 p3` and `site` in `glass backdrop film`. `assert_greater` backdrop off > p1 > p3; `assert_about "glass grain under blur" <glass p1> <glass off> 0.05` and the same for p3 and for film. Roughness: the same over `rough-$site-btrue-r{0,0.5,1}` against `rough-zero-btrue-r*`: backdrop strictly decreasing, glass within 5 %; report the `bfalse` row (where roughness still selects pyramid levels of the sharp texture) the same way.
6. `magick compare -metric AE -fuzz 0.392157% "$OUT/film-$kind-off-face.png" "$OUT/glass-$kind-off-face.png" null:` → `assert_zero` per kind: every face pixel within one code (`-fuzz` takes a percentage of the quantum range; `1/255` is not a tolerance ImageMagick understands, and `0.392157%` accepts a one-code difference and rejects two).
7. The ring fixture, in its own function with its own config writer (the lib's `write_config` hard-codes `ring-beam-speed 0`, and a second singleton node does not override the first):

```bash
write_ring_config() {   # $1 path, $2 site, $3 amount
    cat > "$1" <<KDL
prefer-no-csd
layout { gaps 40; background-color "transparent"; default-column-width { proportion 0.4; }; focus-ring { off; }; border { off; }; shadow { off; } }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$RING_WALL"
blur { passes 3; offset 3; noise 0; saturation 1; }
material "gos-probe" {
    glass {
        ior 1
        thickness 20
        attenuation-color "#ffffff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        roughness 0
        backdrop-blur false
        jelly-flex 0
        jelly-ripple 0
        bevel 12
        offset-x 6
        offset-y 6
        saturation 1
        noise $3 type="white" site="$2"
    }
    response "default" {
        focus "ring-light"
        accent "none"
        ring-gap 6
        ring-width 3
        ring-color "#ffffff"
        ring-glow 2
        ring-rest 2
        ring-beam-speed 400
        ring-beam-decay 0
    }
}
window-rule {
    match app-id="^gos-probe$"
    material "gos-probe"
    geometry-corner-radius 0
    background-effect { blur false; noise 0; saturation 1; }
}
KDL
}
ring_cell() {   # $1 name, $2 site, $3 amount
    write_ring_config "$OUT/$1.kdl" "$2" "$3"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    probe_rect "$NIRI"
    # The comet's whole run: a lap (perimeter) plus its tail (25 % of it),
    # at 400 px/s, plus a margin; only the rest glow remains afterwards.
    sleep "$(awk -v w="$PW" -v h="$PH" 'BEGIN { printf "%d", 2*(w+h)*1.25/400 + 3 }')"
    shot_twice "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    # The band: the ring sits ring-gap inside where the face begins, which is
    # bevel - offset inside the window edge; a 3 px strip there, 300 px tall.
    roi "$1" "3x300+$((PX + PW - 6 - 6 - 2))+$((PY + 200))" band
    stop_nested
}
for site in glass film; do
    ring_cell "ring-$site-on"  "$site" 0.1
    ring_cell "ring-$site-off" "$site" 0
done
mean "$OUT/ring-glass-off-band.png"; e_b=$METRIC
awk -v e="$e_b" 'BEGIN { exit !(e >= 0.85 && e <= 0.95) }' \
    || fail "ring band level $e_b outside 0.85..0.95: the band crop or the glow/rest values need adjusting (a fixture fix, not a renderer finding)"
expected_glass=$(awk -v e="$e_b" 'function dec(x) { return x <= 0.04045 ? x/12.92 : ((x+0.055)/1.055)^2.4 }
    BEGIN { d1 = 2.4/1.055 * ((0.5+0.055)/1.055)^1.4; y = dec(e); d2 = 1.055/2.4 * y^(1/2.4 - 1); printf "%.4f", d1*d2 }')
for site in glass film; do
    signed_diff "$OUT/ring-$site-on-face.png" "$OUT/ring-$site-off-face.png" "$OUT/ring-$site-face-grain.png"; sd "$OUT/ring-$site-face-grain.png"; face_sd=$METRIC
    signed_diff "$OUT/ring-$site-on-band.png" "$OUT/ring-$site-off-band.png" "$OUT/ring-$site-band-grain.png"; sd "$OUT/ring-$site-band-grain.png"; band_sd=$METRIC
    ratio=$(awk -v a="$band_sd" -v b="$face_sd" 'BEGIN { printf "%.4f", a/b }')
    printf 'ring_%s_band_over_face=%s\n' "$site" "$ratio" >> "$OUT/metrics.txt"
    case $site in
        film)  assert_about "film grain in the ring band" "$ratio" 1.0 0.1 ;;
        glass) assert_about "glass grain compressed under the ring" "$ratio" "$expected_glass" 0.1 ;;
    esac
done
printf 'ring_band_level=%s\nring_expected_glass_ratio=%s\n' "$e_b" "$expected_glass" >> "$OUT/metrics.txt"
```

(`assert_about`'s tolerance is relative: 0.1 means ±10 % of the wanted value, which is ±0.1 at 1.0 and ±0.047 at 0.47; the spec's ±0.1 absolute is the looser bound for glass, so pass `0.2` there if the first run lands between.) The ring and rest values 2 and 2 put the rest glow near `0.7 * 2 * 0.2 * 2 = 0.56` linear over the 0.214 backdrop, about 0.89 encoded; both are within their `0..3` bounds. `"$NIRI" validate -c` runs inside `start_nested` and refuses an invalid config.

End with `finish` and a PASS line. Keep `set -eu`, the lib's `trap cleanup EXIT`, and the one-run-dir-per-run pattern.

- [ ] **Step 3: Build the baseline binary and run the smoke**

```bash
# from the main checkout
work-link --ensure .worktrees
git worktree add .worktrees/material-cf32e5-baseline b261ad1a
git worktree lock --reason "on WORK_ROOT storage (host: $(uname -n))" .worktrees/material-cf32e5-baseline
(cd .worktrees/material-cf32e5-baseline && just setup && cargo build --release)
BASE=$(cd .worktrees/material-cf32e5-baseline && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/niri
```

Then, from `.worktrees/material-cf32e5`:

```bash
OUT=/mnt/ssd3/tmp/material-cf32e5/noise-site-$(date +%Y%m%d-%H%M) BASE_NIRI=$BASE CAPTURE_TASK=material-cf32e5 \
  bash docs/materials/scripts/glass-noise-site-smoke.sh
```

Expected: `PASS` and a populated `metrics.txt`. A FAIL names the assertion; a failed assertion 4 or 6 is handled as Task 4 Step 4 says (seed orientation, or a recorded finding), never by widening the bound. Remove the baseline worktree afterwards (`git worktree unlock` then `git worktree remove`), keeping the binary's SHA-256 in the evidence document.

- [ ] **Step 4: The evidence document**

Write `docs/materials/2026-10-<dd>-noise-placement-evidence.md` in the style of `2026-09-06-material-glass-noise-type-evidence.md`: Result line, pinned revisions (implementation commit and binary SHA-256, baseline commit and SHA-256), the `tools/capture-meta show <run-dir>` block, a metrics table from `metrics.txt`, the seven assertions with their values, the §7.1 table copied from the spec with the sheet's path under `/mnt/ssd3/tmp/material-cf32e5/` (regenerate the sheets there with `OUT=... bash docs/materials/scripts/noise-placement-sim.sh` so the evidence points at a run dir, not the agent scratchpad), the owner's verdict line left as "owner's look: pending" until they record it, and a "Limitations" section (the model's p/L equivalence; the ring fixture's measured `e_b`). Task 6 appends the cost section.

- [ ] **Step 5: Commit**

```bash
git add docs/materials/scripts/glass-noise-site-smoke.sh docs/materials/2026-10-*-noise-placement-evidence.md \
  docs/materials/render-pipeline.md docs/materials/material-config.md docs/materials/adding-an-optic.md docs/materials/README.md
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "docs(materials): noise placement smoke against the baseline binary, evidence, and the three sites in the references (material-cf32e5)"
```

---

### Task 6: Cost captures with Tracy

**Files:**
- Create: `docs/materials/scripts/noise-placement-cost.sh`
- Modify: `docs/materials/2026-10-<dd>-noise-placement-evidence.md` (the cost section)

**Interfaces:**
- Consumes: the lib's Tracy helpers (`reserve_tracy_port`, `tools_ready`, `capture_bg`, `capture_ready`, `capture_wait`, `export_cpu`, `csvexport --gpu`, `col`), `NIRI_TRACY` from `build_binaries`; the spans `EffectBuffer::prepare_grain`, `Grain::render`, `Blur::render`, `Prefilter::downsample` (Task 3 and existing).
- Produces: the §7.3 numbers.

- [ ] **Step 1: The script**

Create `docs/materials/scripts/noise-placement-cost.sh` on the lib (`capture_preflight headless`, `build_binaries`, `capture_identity`, `reserve_tracy_port`, `tools_ready`). Common glass: the identity fixture of Task 5 with `backdrop-blur true`, `roughness 0.5`, `ior 1.5`, three blur passes, and `noise 0.3 type="fine" site=<backdrop|glass>`. The script writes its own config (a copy of the lib's `write_config` body without the `spawn-at-startup "swaybg"` line) and launches the wallpaper itself, so it owns every process it starts:

```bash
WALL_PIDS=()
start_wall() {   # $1 png; the nested niri's display is the newest wayland-* socket in $RT
    local display; display=$(cd "$RT" && ls -t wayland-* 2>/dev/null | grep -v '\.lock$' | head -1)
    [ -n "$display" ] || fail "no nested wayland display in $RT"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$display swaybg -m fill -i "$1" >> "$OUT/swaybg.log" 2>&1 &
    WALL_PIDS+=($!)
}
stop_walls() {   # only the PIDs this run started; never pkill by name, the desktop has its own swaybg
    local pid
    for pid in "${WALL_PIDS[@]}"; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done
    WALL_PIDS=()
}
cleanup_cost() {   # the lib's cleanup reads $? on entry: carry the original status past stop_walls
    local rc=$?
    stop_walls
    (exit "$rc")
    cleanup
}
trap cleanup_cost EXIT
```

Three runs under `$NIRI_TRACY` per site, each a 30 s Tracy capture (`capture_bg`, `capture_ready`, `capture_wait`), for `site` in `backdrop` and `glass`:

- `static-$site`: one wallpaper, the probe window, no stimulus. From the CPU csv (`export_cpu`), count rows named `EffectBuffer::prepare_grain`, `Blur::render` and `EffectBuffer::prepare_prefilter` in the last 20 s (`trace_end`, `ns_since_start`): `assert_zero` each for both sites. Report the first-frame `Grain::render` GPU time for `backdrop` from the GPU csv (`csvexport --gpu`; columns `Time from start of program`, `GPU execution time`).
- `damage-$site`: every second for 20 s, `stop_walls` then `start_wall` with a new solid colour (`magick -size 1280x720 xc:"rgb($((i*10)),100,120)" "$OUT/wall-$i.png"`). From the GPU csv take the median `GPU execution time` of `Grain::render`, `Blur::render`, `Prefilter::downsample` and `MaterialRenderElement::draw` rows during the stimulus window; report each in ms (`ns_to_ms`) per site, and the sum per damage per site.
- `drag-$site`: every 100 ms for 5 s, rewrite the config with `noise <0.10..0.59>` at the same site and reload it (`msg "$NIRI_TRACY" action load-config-file` if that action exists in `"$NIRI_TRACY" msg action --help`; otherwise write the file in place and let the watcher reload, then sleep the remaining 100 ms). Count `EffectBuffer::prepare_grain`, `Blur::render`, `EffectBuffer::prepare_prefilter` rows in the drag window per site (`assert_greater 40` for backdrop, `assert_zero` for glass), and take the median `MaterialRenderElement::draw` GPU time per site over the same window. Report the per-change cascade in ms for backdrop and the per-redraw material draw for both.

Write everything to `$OUT/metrics.txt`, then `finish`.

- [ ] **Step 2: Run it**

```bash
OUT=/mnt/ssd3/tmp/material-cf32e5/noise-cost-$(date +%Y%m%d-%H%M) CAPTURE_TASK=material-cf32e5 \
  bash docs/materials/scripts/noise-placement-cost.sh
```

Expected: PASS; six case blocks in `metrics.txt`. The headless GPU's numbers are the headless GPU's: the document says so and names the renderer string `capture-meta` recorded. Before and after the run, `pgrep -a swaybg` shows the same desktop wallpaper process as before; the run started and stopped only its own.

- [ ] **Step 3: Append the cost section to the evidence document and commit**

A table per case with both sites side by side: counts of cached passes (static), per-damage span medians and their sum (damage), cascade counts and per-change time plus the per-redraw material draw (drag). The conclusions are limited to what was measured: cache behaviour (what runs when nothing changes), the added pass per damage and what it costs next to the blur and the pyramid it precedes, the cascade a backdrop change triggers against the glass site's none, and the material draw at equal redraw workload (the drag runs redraw at the same rate at both sites; the glass site grains per fragment there and the backdrop site does not). The animated-backdrop figure is the per-damage sum times 60 per second, labelled derived. No verdict word ("cheaper") appears unless a measured pair supports it, and then with the pair beside it. The paragraph is written so prism's interaction document can quote it under the cost class.

```bash
git add docs/materials/scripts/noise-placement-cost.sh docs/materials/2026-10-*-noise-placement-evidence.md
just upstream-report --stage && git add docs/materials/upstream-divergence.md
git commit -m "docs(materials): noise placement cost captures: static, per damage, dragging (material-cf32e5)"
```

---

### Task 7: Gate, whole-branch review, merge, prism hand-off, close

**Files:**
- Modify: task records only in this repository; one commit in prism (`defs/rack/pipeline.json`).

- [ ] **Step 1: Gate**

Run: `just gate` in `.worktrees/material-cf32e5`. Expected: green. `tasks check`: zero errors.

- [ ] **Step 2: Whole-branch review**

Dispatch one fresh reviewer on the most capable model over `git log materials-26.04..material-cf32e5` with the spec and this plan; record `review: impl round <n> — verdict ...` on `material-cf32e5`; run corrective rounds while Critical or Important findings reproduce (at most five).

- [ ] **Step 3: Merge and hand off**

```bash
git merge --no-ff -m "merge: noise placement: backdrop, glass and film sites (material-cf32e5)" material-cf32e5
```

Then in prism, in a worktree per the prism rules, copy `resources/materials/pipeline.json` to `defs/rack/pipeline.json`, run `just test-fast` and `just check`, commit `chore(rack): vendor the pipeline schema with the noise site stages (prism-be5abe)` (start `prism-be5abe` for it, or file an xs task if `prism-be5abe` should not start yet), and merge.

- [ ] **Step 4: Follow-ups and close**

```bash
tasks add "Film-site saturation: the first shadows edge" --parent material-3aa1f2 --size m --complexity mid -b "Spec 2026-10-05-noise-placement-design.md §9: saturation at the post site greys tint and ring light alike; the schema's first shadows edge."
tasks add "Noise layers select a site each" --parent material-3aa1f2 --depends material-3fcba2 --size s -b "Spec §9: one site per layer; the backdrop agreement rule applies per output across layers."
tasks done material-cf32e5 "noise site= glass|backdrop|film: grain pass in the effect buffer published through the commit counter, film hook at post, three selector stages in the schema; evidence <doc>: baseline byte identity, softening 32/15/8 % predicted and measured <values>, cost <summary>"
```

Also note on `prism-be5abe`: the spec's Section 5 `shadows` sentence is wrong (plan §6 of the renderer spec) and the interaction document needs the softening and cost prose.

---

## Self-review

**Spec coverage.** §3 config → Task 1 (enum, property, agreement, accessor, table row, window-site consequence documented in Task 5). §4 grain pass → Task 3 (options, texture, pass, invalidation table, failure, spans, `source()` at every read including the blurred draw). §5 shader → Task 2 (gate, `noise_post`, `noise_source`, helpers, identities tested in Task 4). §6 schema → Task 2 (three stages, `select`, pins reshaped, JSON regenerated); prism re-vendoring → Task 7. §7.1 → recorded in the spec, re-pointed from the evidence doc in Task 5. §7.2 seven assertions → Task 5 Step 2. §7.3 three cases → Task 6. §8 tests → Tasks 1–4 and 5; the fifth in-process test with its demonstration → Task 4 Step 4. §9 docs → Task 5 Step 1; follow-ups → Task 7.

**Placeholders.** The child tasks (`tasks add --parent material-cf32e5`, each depending on the one before) link to these headings through their `step` field: material-dcc79f, 6f299e, eab3ae, 0bed95, 2a1689, 40b563, e50969 for Tasks 1 to 7. `2026-10-<dd>` is the capture date, fixed when Task 5 runs. The one deliberately empty test body in Task 4 Step 3 is replaced in the same step by the half-height fixture form that follows it; execute the second form.

**Type consistency.** `GrainOptions { amount: f32, kind: NoiseType }` in Tasks 3 and 4; `BackdropGrain { amount: f64, kind: NoiseType }` in Tasks 1 and 3; `update_grain_options(Option<GrainOptions>)` in Tasks 3 and 4; `grain_source()` in Tasks 2 and 3; `GrainProgram::context_id()` in Task 3; `next_grain_status(GrainStatus, Cleared)` in Task 3's code and tests; `MaterialState::commit()` and `tile_rect(f, Option<&str>)` in Task 4 only; `attach_new_colored_buffer(u8, u8, u8, u8)` in Task 4; `noise_source(vec4, vec2)`, `noise_post(vec3, vec2)` in Tasks 2 and 5.

**Review Focus.** Each of the five lines names its test and task; all five tests are written in the task bodies above.
