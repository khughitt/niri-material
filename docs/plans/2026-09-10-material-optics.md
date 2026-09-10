# Material Optics API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the glass pipeline into a slab plus an ordered list of self-contained optics, migrate saturation and noise into the first two optic modules with byte-identical rendering, and tie the parameter docs to the parser.

**Architecture:** An optic owns one file in `niri-config` (node struct, resolved struct, `resolve`, `params`), one file in `niri` (uniform list, per-frame values, next-change), and one GLSL file (its uniforms and `<name>_<hook>` functions). `OPTICS` in `src/render_helpers/material/optics/mod.rs` is the one ordered registry; the shader is assembled as prelude, optics, main; optic values join the damage fingerprint; optic deadlines join the redraw deadline independently of the signal cache. Typed `ParamSpec`s render the docs table and drive a parser test.

**Tech Stack:** Rust (workspace crates `niri-config`, `niri`), knuffel (KDL), smithay GLES renderer (`Uniform`, `UniformName`, `UniformType`), GLSL ES 1.00, bash smoke scripts on a headless Weston host, `just` front door.

**Spec:** `docs/specs/2026-09-10-material-optics-design.md`

## Global Constraints

- Branch `feat/material-397fcb`, worktree `.worktrees/material-api`; base commit `22d10019`. Run every command from the worktree.
- Conventional commits; no AI-attribution or session trailers. `tasks done <id> "<what landed>"` goes in the same commit as the code for the task it closes.
- Every commit passes the pre-commit hook (`just check`: ops-check, `cargo fmt --check`, `cargo clippy --all --all-targets`, tools unit tests, `tasks check`, `upstream-report --check`, `package-pin --check`).
- Every test run goes through the timing wrapper, as AGENTS.md requires: `just test` for the suite, and for the inner loop `python3 tools/tt test-fast -- cargo test -p <crate> [--lib] <filter>` with exactly one filter per invocation (cargo rejects a second positional filter).
- `tools/upstream-report --check` compares the committed report against the staged tree and every fork path counts, so a commit that adds, moves, or removes a file must regenerate the report first: `python3 tools/upstream-report && git add docs/materials/upstream-divergence.md` after `git add` and before `git commit`. Every commit step below does this.
- The KDL surface is unchanged: `material "name" { glass { ... } }`, `noise <amount> type=<type>`, `saturation <amount>`, every existing parameter name and range.
- Rendering before and after the migration is byte-identical on the noise and saturation smoke fixtures (Task 9 proves it).
- Uniform names for optics use the prefix `mat_<optic>_`; the two migrated optics keep their existing names `mat_saturation`, `mat_noise`, `mat_noise_type`.
- GLSL hook functions are named `<optic>_<hook>`; hooks are `normal`, `specular`, `emissive`, `post`. Transforming hooks return their input at the optic's neutral; `emissive` returns `vec3(0.0)`.
- The shader compile path prepends `#version 100`; the prelude must not carry a version line.
- If the hook fails with a "no field" compile error the source contradicts, run `cargo clean -p niri-config -p niri` and retry (shared target dir across worktrees).
- Smithay is pinned at `ff5fa7df`; `Uniform` and `UniformValue` derive `PartialEq` there. `UniformName::new` is not `const`, so static uniform lists are `&[(&str, UniformType)]`.

---

## File structure

| Path | Responsibility |
| --- | --- |
| `niri-config/src/material/mod.rs` | moved from `material.rs`; `Material`, `Glass`, `ResolvedGlass`, responses, `MaterialRef`, `Milli`, `Positive`, `core_params()` |
| `niri-config/src/material/params.rs` | `Bounded`, `ParamKind`, `ParamSpec`, `render_param_table`, `all_params` |
| `niri-config/src/material/optics/mod.rs` | `ORDER`, `params()` |
| `niri-config/src/material/optics/saturation.rs` | `Saturation` node type, `ResolvedSaturation`, `resolve`, `params` |
| `niri-config/src/material/optics/noise.rs` | `Noise` node, `NoiseType`, `ResolvedNoise`, `resolve`, `params` |
| `src/render_helpers/material/mod.rs` | moved from `material.rs`; element, state, fingerprint |
| `src/render_helpers/material/optics/mod.rs` | `Optic`, `OpticFrame`, `OpticEntry`, `OPTICS`, `uniform_names`, `values`, `next_change` |
| `src/render_helpers/material/optics/saturation.rs` | `SaturationOptic` |
| `src/render_helpers/material/optics/noise.rs` | `NoiseOptic` |
| `src/render_helpers/shaders/material/prelude.frag` | core uniforms, globals, helper library, slab, taps |
| `src/render_helpers/shaders/material/main.frag` | `void main()` with the hook calls |
| `src/render_helpers/shaders/material/saturation.frag` | `mat_saturation`, `saturation_post` |
| `src/render_helpers/shaders/material/noise.frag` | `mat_noise`, `mat_noise_type`, `noise_post` |
| `src/render_helpers/shaders/mod.rs` | `material_source()`, `material_uniform_names()` |
| `src/layout/tile.rs` | `resolve_material`, `material_dynamics`, `tick_deadline`, render sites |
| `docs/materials/material-config.md` | generated table between markers; `## Optics` section |
| `docs/materials/adding-an-optic.md` | the contributor recipe |
| `docs/materials/render-pipeline.md`, `README.md`, `upstream-divergence.md` | path and stage updates |

---

### Task 1: Parameter metadata module

**Files:**
- Move: `niri-config/src/material.rs` → `niri-config/src/material/mod.rs`
- Create: `niri-config/src/material/params.rs`
- Test: unit tests inside `params.rs`

**Interfaces:**
- Produces: `pub trait Bounded { const MIN: f64; const MAX: f64; const MIN_EXCLUSIVE: bool; }` implemented for `FloatOrInt<MIN, MAX>`, `Milli<MIN_MILLI, MAX_MILLI>`, `Positive<MAX>`; `pub enum ParamKind`; `pub struct ParamSpec { node, kind, write: fn(&str) -> String, read: Option<fn(&ResolvedGlass) -> Option<f64>> }`; `ParamKind::float::<B>(default, unit)`, `ParamKind::inherit::<B>()`; `pub fn render_param_table(&[ParamSpec]) -> String`.

- [ ] **Step 1: Move the module into a directory**

```bash
cd /mnt/ssd/Dropbox/niri-material/.worktrees/material-api
mkdir -p niri-config/src/material
git mv niri-config/src/material.rs niri-config/src/material/mod.rs
cargo build -p niri-config
```

Expected: builds with no change (Rust resolves `material/mod.rs` the same as `material.rs`).

- [ ] **Step 2: Write the failing tests**

Create `niri-config/src/material/params.rs` with only the tests and a module doc so the file compiles against nothing yet:

```rust
//! Parameter metadata: the facts the docs table and the parser share.
//!
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §6.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{Milli, Positive};
    use crate::FloatOrInt;

    #[test]
    fn bounds_follow_the_type_parameters() {
        assert_eq!(<FloatOrInt<1, 3> as Bounded>::MIN, 1.);
        assert_eq!(<FloatOrInt<1, 3> as Bounded>::MAX, 3.);
        assert!(!<FloatOrInt<1, 3> as Bounded>::MIN_EXCLUSIVE);
        assert_eq!(<Milli<0, 20> as Bounded>::MAX, 0.02);
        assert_eq!(<Positive<65535> as Bounded>::MIN, 0.);
        assert!(<Positive<65535> as Bounded>::MIN_EXCLUSIVE);
    }

    #[test]
    fn the_table_renders_one_row_per_spec_in_order() {
        let specs = [
            ParamSpec {
                node: "ior",
                kind: ParamKind::float::<FloatOrInt<1, 3>>(1.5, "—"),
                write: |v| format!("ior {v}"),
                read: Some(|g| Some(g.ior)),
            },
            ParamSpec {
                node: "offset-x",
                kind: ParamKind::float::<FloatOrInt<-64, 64>>(6., "logical px"),
                write: |v| format!("offset-x {v}"),
                read: Some(|g| Some(g.offset_x)),
            },
            ParamSpec {
                node: "attenuation-distance",
                kind: ParamKind::float::<Positive<65535>>(60., "logical px"),
                write: |v| format!("attenuation-distance {v}"),
                read: Some(|g| Some(g.attenuation_distance)),
            },
            ParamSpec {
                node: "noise type=",
                kind: ParamKind::Enum {
                    default: "white",
                    variants: &["white", "fine", "lightness"],
                },
                write: |v| format!("noise 0.5 type=\"{v}\""),
                read: None,
            },
        ];
        let table = render_param_table(&specs);
        let expected = "\
| Parameter | Type | Default | Range | Unit |
| --- | --- | --- | --- | --- |
| `ior` | float | 1.5 | 1–3 | — |
| `offset-x` | float | 6 | −64–64 | logical px |
| `attenuation-distance` | float | 60 | > 0 through 65535 | logical px |
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |
";
        assert_eq!(table, expected);
    }
}
```

Add `pub mod params;` near the top of `niri-config/src/material/mod.rs`, after the `use` block.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config params::tests`
Expected: FAIL to compile with `cannot find trait Bounded` / `cannot find struct ParamSpec`.

- [ ] **Step 4: Implement the module**

Prepend to `niri-config/src/material/params.rs`, above the test module:

```rust
use crate::appearance::Color;
use crate::FloatOrInt;

use super::{Milli, Positive, ResolvedGlass};

/// The parse-time bounds of a scalar parameter type, as the type declares
/// them. The docs table and the parser test read these so a range can only
/// be written once, on the type.
pub trait Bounded {
    const MIN: f64;
    const MAX: f64;
    /// Whether `MIN` itself is rejected (`Positive`).
    const MIN_EXCLUSIVE: bool = false;
}

impl<const MIN: i32, const MAX: i32> Bounded for FloatOrInt<MIN, MAX> {
    const MIN: f64 = MIN as f64;
    const MAX: f64 = MAX as f64;
}

impl<const MIN_MILLI: i32, const MAX_MILLI: i32> Bounded for Milli<MIN_MILLI, MAX_MILLI> {
    const MIN: f64 = MIN_MILLI as f64 / 1000.;
    const MAX: f64 = MAX_MILLI as f64 / 1000.;
}

impl<const MAX: i32> Bounded for Positive<MAX> {
    const MIN: f64 = 0.;
    const MAX: f64 = MAX as f64;
    const MIN_EXCLUSIVE: bool = true;
}

/// What kind of value a parameter takes, with its default and bounds taken
/// from the resolved type and the bound type rather than typed by hand.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamKind {
    Float {
        default: f64,
        min: f64,
        max: f64,
        min_exclusive: bool,
        unit: &'static str,
    },
    /// A float whose omitted value inherits (noise and saturation).
    FloatOrInherit { min: f64, max: f64 },
    Color { default: Color },
    Bool { default: bool },
    Enum {
        default: &'static str,
        variants: &'static [&'static str],
    },
}

impl ParamKind {
    pub fn float<B: Bounded>(default: f64, unit: &'static str) -> Self {
        Self::Float {
            default,
            min: B::MIN,
            max: B::MAX,
            min_exclusive: B::MIN_EXCLUSIVE,
            unit,
        }
    }

    pub fn inherit<B: Bounded>() -> Self {
        Self::FloatOrInherit {
            min: B::MIN,
            max: B::MAX,
        }
    }
}

/// One parameter as the docs table and the parser test see it.
pub struct ParamSpec {
    /// The node as the table names it: `"noise"`, `"noise type="`,
    /// `"distortion scale="`. Words are rendered in separate backticks.
    pub node: &'static str,
    pub kind: ParamKind,
    /// The KDL line inside `glass { }` that sets this parameter to a value.
    pub write: fn(&str) -> String,
    /// The resolved value, for `Float` and `FloatOrInherit` kinds.
    pub read: Option<fn(&ResolvedGlass) -> Option<f64>>,
}

/// A number as the table prints it: integers without a decimal point, a
/// typographic minus.
fn num(v: f64) -> String {
    let s = if v.fract() == 0. {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    };
    s.replace('-', "\u{2212}")
}

fn range(min: f64, max: f64, min_exclusive: bool) -> String {
    if min_exclusive {
        format!("> {} through {}", num(min), num(max))
    } else {
        format!("{}–{}", num(min), num(max))
    }
}

fn color_hex(color: &Color) -> String {
    let [r, g, b, _] = color.to_array_unpremul();
    let channel = |c: f32| (c * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

/// Renders the Markdown parameter table `material-config.md` carries.
pub fn render_param_table(specs: &[ParamSpec]) -> String {
    let mut out =
        String::from("| Parameter | Type | Default | Range | Unit |\n| --- | --- | --- | --- | --- |\n");
    for spec in specs {
        let node = spec
            .node
            .split(' ')
            .map(|word| format!("`{word}`"))
            .collect::<Vec<_>>()
            .join(" ");
        let (ty, default, range, unit) = match &spec.kind {
            ParamKind::Float {
                default,
                min,
                max,
                min_exclusive,
                unit,
            } => (
                String::from("float"),
                num(*default),
                self::range(*min, *max, *min_exclusive),
                String::from(*unit),
            ),
            ParamKind::FloatOrInherit { min, max } => (
                String::from("float"),
                String::from("inherit"),
                self::range(*min, *max, false),
                String::from("—"),
            ),
            ParamKind::Color { default } => (
                String::from("color"),
                format!("`{}`", color_hex(default)),
                String::from("any color"),
                String::from("—"),
            ),
            ParamKind::Bool { default } => (
                String::from("bool"),
                default.to_string(),
                String::from("true / false"),
                String::from("—"),
            ),
            ParamKind::Enum { default, variants } => (
                variants
                    .iter()
                    .map(|v| format!("`{v}`"))
                    .collect::<Vec<_>>()
                    .join(" / "),
                format!("`{default}`"),
                String::from("—"),
                String::from("—"),
            ),
        };
        out.push_str(&format!("| {node} | {ty} | {default} | {range} | {unit} |\n"));
    }
    out
}
```

`Color::to_array_unpremul` exists (`src/render_helpers/material.rs` uses it for `mat_attenuation_color`). If `Milli` and `Positive` are not `pub` at the crate root, `use super::{Milli, Positive}` inside the module still resolves: both are `pub struct`s in `material/mod.rs`.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config params::tests`
Expected: 2 passed.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add niri-config/src/material/mod.rs niri-config/src/material/params.rs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "refactor(config): add material parameter metadata with bounds from the types"
```

---

### Task 2: Saturation optic in niri-config

**Files:**
- Create: `niri-config/src/material/optics/mod.rs`, `niri-config/src/material/optics/saturation.rs`
- Modify: `niri-config/src/material/mod.rs` (`Glass.saturation` type, `ResolvedGlass.saturation`, `Default`, `resolve()`), `niri-config/src/lib.rs` (re-exports, tests), `src/layout/tile.rs:205-212` and tests, `src/render_helpers/material.rs` tests
- Test: `niri-config/src/lib.rs` existing saturation tests

**Interfaces:**
- Produces: `niri_config::material::optics::saturation::{Saturation, ResolvedSaturation, resolve, params}`; `ResolvedGlass.saturation: ResolvedSaturation { amount: Option<f64> }`; re-export `niri_config::ResolvedSaturation`.

- [x] **Step 1: Write the failing test**

Add to the `tests` module of `niri-config/src/lib.rs`, next to `glass_noise_and_saturation_resolve_independently`:

```rust
    #[test]
    fn saturation_resolves_through_its_optic() {
        let written = do_parse(r##"material "frost" { glass { saturation 0.85; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.saturation,
            ResolvedSaturation { amount: Some(0.85) }
        );
        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.saturation,
            ResolvedSaturation::default()
        );
        assert_eq!(ResolvedSaturation::default().amount, None);
    }
```

- [x] **Step 2: Run the test to verify it fails**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config saturation_resolves_through_its_optic`
Expected: FAIL to compile, `cannot find type ResolvedSaturation`.

- [x] **Step 3: Create the optic module**

`niri-config/src/material/optics/mod.rs`:

```rust
//! Optics: the self-contained stages of the glass pipeline, config side.
//!
//! Each optic owns its node struct, its resolved struct and defaults, its
//! `resolve`, and its `params`. The renderer's `OPTICS` table must list the
//! optics in `ORDER`; a test in niri pins the two.
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §2.

use super::params::ParamSpec;

pub mod saturation;

/// Render order of the optics, which is also the order of their rows in
/// the parameter table.
pub const ORDER: &[&str] = &["saturation"];

/// Every optic's parameter specs, in `ORDER`.
pub fn params() -> Vec<ParamSpec> {
    saturation::params()
}
```

`niri-config/src/material/optics/saturation.rs`:

```rust
//! `saturation <amount>`: stage 9, `mix(luma, color, amount)` on the
//! encoded glass colour. Neutral at 1.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Saturation = FloatOrInt<0, 3>;

/// Final saturation state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedSaturation {
    /// `None` inherits: the global `blur` block's value while backdrop blur
    /// is effective, the neutral 1 otherwise. A written value applies
    /// regardless of either switch. The renderer's optic applies the rule.
    pub amount: Option<f64>,
}

pub fn resolve(node: Option<Saturation>) -> ResolvedSaturation {
    ResolvedSaturation {
        amount: node.map(|x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "saturation",
        kind: ParamKind::inherit::<Saturation>(),
        write: |v| format!("saturation {v}"),
        read: Some(|g| g.saturation.amount),
    }]
}
```

- [x] **Step 4: Route `Glass` and `ResolvedGlass` through it**

In `niri-config/src/material/mod.rs`:

1. After `pub mod params;` add `pub mod optics;`.
2. In `Glass`, change `pub saturation: Option<FloatOrInt<0, 3>>,` to `pub saturation: Option<optics::saturation::Saturation>,`.
3. In `ResolvedGlass`, replace the `saturation` field and its doc comment with:

```rust
    pub saturation: optics::saturation::ResolvedSaturation,
```

4. In `impl Default for ResolvedGlass`, change `saturation: None,` to `saturation: optics::saturation::ResolvedSaturation::default(),`.
5. In `Material::resolve`, change `saturation: g.saturation.map(|x| x.0),` to `saturation: optics::saturation::resolve(g.saturation),`.

In `niri-config/src/lib.rs`, extend the `pub use crate::material::{...}` list with `ResolvedSaturation` by adding a second line after it:

```rust
pub use crate::material::optics::saturation::ResolvedSaturation;
```

- [x] **Step 5: Update the existing config tests**

In `niri-config/src/lib.rs` tests: every `saturation: None,` inside a `ResolvedGlass { ... }` literal (around line 1237) becomes `saturation: ResolvedSaturation::default(),`; every `glass.saturation` comparison (lines near 1309, 1317, 1322, 1329) becomes `glass.saturation.amount`. `ResolvedGlass` literals that name `noise` and `noise_type` are untouched in this task.

- [x] **Step 6: Keep the niri crate compiling**

`src/layout/tile.rs`, in `resolve_material`, change

```rust
    let saturation = material
        .glass
        .saturation
        .unwrap_or_else(|| inherited(options.blur.saturation, 1.)) as f32;
```

to

```rust
    let saturation = material
        .glass
        .saturation
        .amount
        .unwrap_or_else(|| inherited(options.blur.saturation, 1.)) as f32;
```

In the `tile.rs` tests, the test `written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur` builds `ResolvedGlass { noise, saturation, backdrop_blur, ..Default::default() }`; change `saturation,` to `saturation: niri_config::ResolvedSaturation { amount: saturation },`.

`src/render_helpers/material.rs` does not read `glass.saturation` (it receives the tile's `f32`), so it needs no change here.

- [x] **Step 7: Run the tests**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config` then `python3 tools/tt test-fast -- cargo test -p niri --lib layout::tile`
Expected: all pass, including `saturation_resolves_through_its_optic`.

- [x] **Step 8: Commit**

```bash
cargo fmt --all
git add niri-config/src src/layout/tile.rs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "refactor(config): move saturation into the first optic module"
```

---

### Task 3: Noise optic in niri-config

**Files:**
- Create: `niri-config/src/material/optics/noise.rs`
- Modify: `niri-config/src/material/mod.rs` (remove `Noise`, `NoiseType`, its `response_from_str!` line; `Glass.noise` type; `ResolvedGlass.noise`, drop `noise_type`; `Default`; `resolve()`), `niri-config/src/material/optics/mod.rs`, `niri-config/src/lib.rs` (re-exports, tests), `src/layout/tile.rs:205-208` and tests, `src/render_helpers/material.rs:841` and test `:1281`

**Interfaces:**
- Produces: `niri_config::material::optics::noise::{Noise, NoiseType, ResolvedNoise, resolve, params}`; `ResolvedGlass.noise: ResolvedNoise { amount: Option<f64>, kind: NoiseType }`; re-exports `niri_config::{Noise, NoiseType, ResolvedNoise}` (the first two keep their current paths).

- [x] **Step 1: Write the failing test**

Add to the `tests` module of `niri-config/src/lib.rs`:

```rust
    #[test]
    fn noise_resolves_through_its_optic() {
        let written =
            do_parse(r##"material "frost" { glass { noise 0.5 type="fine"; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.noise,
            ResolvedNoise {
                amount: Some(0.5),
                kind: NoiseType::Fine,
            }
        );
        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.noise,
            ResolvedNoise::default()
        );
        assert_eq!(ResolvedNoise::default().kind, NoiseType::White);
        assert_eq!(ResolvedNoise::default().amount, None);
    }
```

- [x] **Step 2: Run the test to verify it fails**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config noise_resolves_through_its_optic`
Expected: FAIL to compile, `cannot find type ResolvedNoise`.

- [x] **Step 3: Create the optic module**

`niri-config/src/material/optics/noise.rs`. Move the `Noise` struct, the `NoiseType` enum, and their doc comments out of `material/mod.rs` verbatim, and replace the `response_from_str!(NoiseType, ...)` line there with a hand-written `FromStr` here so the module is self-contained:

```rust
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
```

The error string `unknown NoiseType value: {s}` is what the existing test `glass_noise_type_rejects_an_unknown_value` asserts.

- [x] **Step 4: Route `Glass` and `ResolvedGlass` through it**

In `niri-config/src/material/mod.rs`:

1. Delete the moved `Noise` struct, `NoiseType` enum, and the `response_from_str!(NoiseType, ...)` line.
2. In `Glass`, `pub noise: Option<Noise>,` becomes `pub noise: Option<optics::noise::Noise>,`.
3. In `ResolvedGlass`, delete the `noise` and `noise_type` fields and their doc comments; add `pub noise: optics::noise::ResolvedNoise,` where `noise` was.
4. In `Default`, delete `noise: None,` and `noise_type: NoiseType::White,`; add `noise: optics::noise::ResolvedNoise::default(),`.
5. In `resolve()`, delete the `noise:` and `noise_type:` lines; add `noise: optics::noise::resolve(g.noise),`.

In `niri-config/src/material/optics/mod.rs`: add `pub mod noise;`, set `pub const ORDER: &[&str] = &["saturation", "noise"];`, and make `params()`:

```rust
pub fn params() -> Vec<ParamSpec> {
    let mut specs = saturation::params();
    specs.extend(noise::params());
    specs
}
```

In `niri-config/src/lib.rs`: remove `Noise, NoiseType,` from the `pub use crate::material::{...}` list and add:

```rust
pub use crate::material::optics::noise::{Noise, NoiseType, ResolvedNoise};
```

- [x] **Step 5: Update the existing config tests**

In `niri-config/src/lib.rs` tests: `noise: None,` and `noise_type: NoiseType::White,` in `ResolvedGlass` literals (around line 1236) become one line `noise: ResolvedNoise::default(),`; `glass.noise` comparisons become `glass.noise.amount`; `glass.noise_type` becomes `glass.noise.kind`; `ResolvedGlass::default().noise_type` becomes `ResolvedGlass::default().noise.kind`.

- [x] **Step 6: Keep the niri crate compiling**

`src/layout/tile.rs` `resolve_material`: `.noise` → `.noise.amount` in the `let noise = material.glass.noise...` chain. In its tests: `noise,` in the `ResolvedGlass` literal of `written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur` becomes `noise: niri_config::ResolvedNoise { amount: noise, ..Default::default() },`; in `noise_type_reaches_the_render_config_regardless_of_blur`, `noise: Some(0.3), noise_type,` becomes `noise: niri_config::ResolvedNoise { amount: Some(0.3), kind: noise_type },` and the assertion `resolved.material.glass.noise_type` becomes `resolved.material.glass.noise.kind`.

`src/render_helpers/material.rs:841`: `g.noise_type as u8 as f32` → `g.noise.kind as u8 as f32`. Test `noise_type_change_advances_the_commit_in_place`: `changed.material.glass.noise_type = niri_config::NoiseType::Fine;` → `changed.material.glass.noise.kind = niri_config::NoiseType::Fine;`.

- [x] **Step 7: Run the tests**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config` then `python3 tools/tt test-fast -- cargo test -p niri --lib`
Expected: all pass.

- [x] **Step 8: Commit**

```bash
cargo fmt --all
git add niri-config/src src/layout/tile.rs src/render_helpers/material.rs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "refactor(config): move noise into its optic module"
```

---

### Task 4: Core parameter specs, the generated table, and the parser test

**Files:**
- Modify: `niri-config/src/material/mod.rs` (`core_params()`), `niri-config/src/material/params.rs` (`all_params()`, two tests), `docs/materials/material-config.md` (markers, generated table)

**Interfaces:**
- Produces: `niri_config::material::core_params() -> Vec<ParamSpec>`, `niri_config::material::params::all_params() -> Vec<ParamSpec>`; markers `<!-- params:begin -->` / `<!-- params:end -->` in `material-config.md`; env var `MATERIAL_DOCS_UPDATE=1` rewrites the table.

- [ ] **Step 1: Write the failing tests**

The parser test decodes a bare `glass { }` node and resolves it through a
`Material` it builds itself, so only the scalar decode runs. It must not go
through `Config::parse_mem`: that path also runs `Material::validate`, whose
cross-parameter rules reject valid scalar bounds on their own (`bevel 0`
fails `ring-inset + ring-width <= bevel` for every response, and
`offset-x 64` fails `offset must not exceed bevel` at the default bevel).
The documented scalar ranges stay as they are; the cross-parameter rules
keep their own tests.

Add to the `tests` module of `niri-config/src/material/params.rs` (it gets
`use std::path::Path;` and `use crate::material::{Glass, Material};`):

```rust
    #[test]
    fn material_parameter_table_matches_the_docs() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../docs/materials/material-config.md");
        let doc = std::fs::read_to_string(&path).unwrap();
        let begin = "<!-- params:begin -->";
        let end = "<!-- params:end -->";
        let a = doc.find(begin).expect("begin marker") + begin.len();
        let b = doc.find(end).expect("end marker");
        let expected = format!("\n{}\n", render_param_table(&all_params()));
        if std::env::var_os("MATERIAL_DOCS_UPDATE").is_some() {
            std::fs::write(&path, format!("{}{}{}", &doc[..a], expected, &doc[b..])).unwrap();
            return;
        }
        assert_eq!(
            &doc[a..b],
            expected,
            "material-config.md parameter table is stale; rerun with MATERIAL_DOCS_UPDATE=1"
        );
    }

    #[test]
    fn material_parameter_specs_match_the_parser() {
        /// Decodes one `glass { <line> }` node and resolves it. Scalar
        /// bounds only: `Material::validate` is not run.
        fn glass_with(line: &str) -> Result<ResolvedGlass, String> {
            let body = if line.is_empty() {
                String::new()
            } else {
                format!("{line};")
            };
            let mut nodes: Vec<Glass> =
                knuffel::parse("spec.kdl", &format!("glass {{ {body} }}\n"))
                    .map_err(|err| format!("{:?}", miette::Report::new(err)))?;
            let material = Material {
                name: String::from("t"),
                glass: nodes.remove(0),
                responses: Vec::new(),
            };
            Ok(material.resolve().glass)
        }

        let neutral = glass_with("").unwrap();
        for spec in all_params() {
            let node = spec.node;
            let write = spec.write;
            let (bounds, default) = match spec.kind {
                ParamKind::Float {
                    default,
                    min,
                    max,
                    min_exclusive,
                    ..
                } => (Some((min, max, min_exclusive)), Some(default)),
                ParamKind::FloatOrInherit { min, max } => (Some((min, max, false)), None),
                ParamKind::Enum { variants, .. } => {
                    for variant in variants {
                        glass_with(&write(variant))
                            .unwrap_or_else(|e| panic!("{node} {variant}: {e}"));
                    }
                    assert!(glass_with(&write("nope")).is_err(), "{node} accepted nope");
                    (None, None)
                }
                ParamKind::Bool { .. } => {
                    for v in ["true", "false"] {
                        glass_with(&write(v)).unwrap_or_else(|e| panic!("{node} {v}: {e}"));
                    }
                    (None, None)
                }
                ParamKind::Color { .. } => {
                    glass_with(&write("#123456")).unwrap_or_else(|e| panic!("{node}: {e}"));
                    (None, None)
                }
            };
            let Some((min, max, min_exclusive)) = bounds else {
                continue;
            };
            let read = spec.read.unwrap_or_else(|| panic!("{node}: float spec without read"));
            for v in [min, max] {
                if v == min && min_exclusive {
                    continue;
                }
                let glass = glass_with(&write(&v.to_string()))
                    .unwrap_or_else(|e| panic!("{node} {v}: {e}"));
                assert_eq!(read(&glass), Some(v), "{node} {v} did not read back");
            }
            let below = if min_exclusive { min } else { min - 0.001 };
            for v in [max + 0.001, below] {
                let err = glass_with(&write(&v.to_string()))
                    .err()
                    .unwrap_or_else(|| panic!("{node} accepted {v}"));
                assert!(err.contains("must be"), "{node} {v}: {err}");
            }
            assert_eq!(read(&neutral), default, "{node} omitted default");
        }
    }
```

`knuffel::parse::<Vec<Glass>>` decodes a document of top-level `glass` nodes; `Glass` derives `knuffel::Decode`, and its range errors (`value must be between ...`, `value must be greater than 0 and at most ...`) surface through the `miette::Report` text the test checks for `must be`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_`
Expected: FAIL to compile, `cannot find function all_params`.

- [ ] **Step 3: Write `core_params` and `all_params`**

In `niri-config/src/material/mod.rs`, after `impl Default for ResolvedGlass`:

```rust
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
```

Each `float::<T>` names the same type as the matching `Glass` field (`FloatOrInt<1, 3>` for `ior`, and so on); the parser test is what catches a mismatch.

In `niri-config/src/material/params.rs`, above the tests:

```rust
/// The core specs followed by every optic's, in render order: the whole
/// docs table.
pub fn all_params() -> Vec<ParamSpec> {
    let mut specs = super::core_params();
    specs.extend(super::optics::params());
    specs
}
```

- [ ] **Step 4: Put the markers into the docs and generate the table**

In `docs/materials/material-config.md`, replace the hand-written table (the lines from `| Parameter | Type | Default | Range | Unit |` through `| \`offset-x\` / \`offset-y\` | float | 6 | −64–64 | logical px |`) with:

```markdown
<!-- params:begin -->
<!-- params:end -->
```

Then generate:

```bash
MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs
git diff docs/materials/material-config.md
```

Expected: the diff shows the table regenerated between the markers, with `saturation`, `noise`, and `noise type=` now as the last three rows and `offset-x` and `offset-y` on separate rows.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_`
Expected: 2 passed. If `material_parameter_specs_match_the_parser` fails on one node, the spec's type and the `Glass` field's type disagree; fix the spec. A failure mentioning `offset must not exceed bevel` or `ring-inset` means the test went through `Config::parse_mem`; it must decode `Glass` directly.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add niri-config/src docs/materials/material-config.md
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(config): generate the material parameter table and test it against the parser"
```

---

### Task 5: Optic registry in niri with the two optic modules and their GLSL

**Files:**
- Move: `src/render_helpers/material.rs` → `src/render_helpers/material/mod.rs`
- Create: `src/render_helpers/material/optics/mod.rs`, `src/render_helpers/material/optics/saturation.rs`, `src/render_helpers/material/optics/noise.rs`, `src/render_helpers/shaders/material/saturation.frag`, `src/render_helpers/shaders/material/noise.frag`
- Test: unit tests in `optics/saturation.rs` and `optics/noise.rs`

**Interfaces:**
- Produces: `crate::render_helpers::material::optics::{Optic, OpticFrame, OpticEntry, OPTICS, uniform_names, values, next_change}`.
- `pub struct OpticFrame<'a> { pub now: Duration, pub motion: SignalMotionPolicy, pub animations_off: bool, pub backdrop_blur: bool, pub blur: &'a Blur, pub seed: f32 }`
- `pub fn values(&ResolvedGlass, &OpticFrame<'_>) -> Vec<Uniform<'static>>`, `pub fn next_change(&ResolvedGlass, &OpticFrame<'_>) -> Option<Duration>`, `pub fn uniform_names() -> Vec<UniformName<'static>>`.

- [ ] **Step 1: Move the module into a directory**

```bash
mkdir -p src/render_helpers/material
git mv src/render_helpers/material.rs src/render_helpers/material/mod.rs
cargo build -p niri
```

Expected: builds unchanged.

- [ ] **Step 2: Write the failing tests**

Create `src/render_helpers/material/optics/saturation.rs` with tests only for now:

```rust
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedGlass, ResolvedSaturation};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::material::optics::OpticFrame;

    fn frame(backdrop_blur: bool, blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur,
            blur,
            seed: 0.,
        }
    }

    fn amount(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> f32 {
        let values = SaturationOptic::values(glass, &frame(backdrop_blur, blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_saturation");
        match values[0].value {
            UniformValue::_1f(v) => v,
            ref other => panic!("{other:?}"),
        }
    }

    #[test]
    fn written_saturation_applies_regardless_of_blur() {
        let glass = ResolvedGlass {
            saturation: ResolvedSaturation { amount: Some(0.5) },
            ..Default::default()
        };
        let blur = Blur {
            saturation: 1.5,
            ..Default::default()
        };
        assert_eq!(amount(&glass, true, &blur), 0.5);
        assert_eq!(amount(&glass, false, &blur), 0.5);
    }

    #[test]
    fn omitted_saturation_inherits_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            saturation: 1.5,
            ..Default::default()
        };
        assert_eq!(amount(&glass, true, &blur), 1.5);
        assert_eq!(amount(&glass, false, &blur), 1.);
    }

    #[test]
    fn saturation_is_static() {
        let blur = Blur::default();
        assert_eq!(
            SaturationOptic::next_change(&ResolvedGlass::default(), &frame(true, &blur)),
            None
        );
    }
}
```

Create `src/render_helpers/material/optics/noise.rs` with tests only:

```rust
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, NoiseType, ResolvedGlass, ResolvedNoise};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::material::optics::OpticFrame;

    fn frame(backdrop_blur: bool, blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur,
            blur,
            seed: 0.,
        }
    }

    fn pair(glass: &ResolvedGlass, backdrop_blur: bool, blur: &Blur) -> (f32, f32) {
        let values = NoiseOptic::values(glass, &frame(backdrop_blur, blur));
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].name, "mat_noise");
        assert_eq!(values[1].name, "mat_noise_type");
        let f = |v: &UniformValue| match v {
            UniformValue::_1f(v) => *v,
            other => panic!("{other:?}"),
        };
        (f(&values[0].value), f(&values[1].value))
    }

    #[test]
    fn written_noise_applies_regardless_of_blur_and_carries_its_type() {
        let glass = ResolvedGlass {
            noise: ResolvedNoise {
                amount: Some(0.3),
                kind: NoiseType::Lightness,
            },
            ..Default::default()
        };
        let blur = Blur {
            noise: 0.02,
            off: true,
            ..Default::default()
        };
        assert_eq!(pair(&glass, true, &blur), (0.3, 2.));
        assert_eq!(pair(&glass, false, &blur), (0.3, 2.));
    }

    #[test]
    fn omitted_noise_inherits_only_while_backdrop_blur_is_effective() {
        let glass = ResolvedGlass::default();
        let blur = Blur {
            noise: 0.02,
            ..Default::default()
        };
        assert_eq!(pair(&glass, true, &blur), (0.02, 0.));
        assert_eq!(pair(&glass, false, &blur), (0., 0.));
    }
}
```

Add `pub mod optics;` to `src/render_helpers/material/mod.rs` after its `use` block, and create `src/render_helpers/material/optics/mod.rs` declaring `pub mod noise; pub mod saturation;` for now.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib render_helpers::material::optics`
Expected: FAIL to compile, `cannot find type OpticFrame` / `SaturationOptic`.

- [ ] **Step 4: Write the GLSL files**

`src/render_helpers/shaders/material/saturation.frag`:

```glsl
// Optic: saturation (render-pipeline.md stage 9). Neutral at 1.
uniform float mat_saturation;

vec3 saturation_post(vec3 color, vec2 fragCoord) {
    if (mat_saturation == 1.0)
        return color;
    const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
    return mix(vec3(dot(color, luma)), color, mat_saturation);
}
```

`src/render_helpers/shaders/material/noise.frag`:

```glsl
// Optic: noise (render-pipeline.md stage 10). Neutral at amount 0.
// Uses hash12, fineGrain, srgbToLinear, linearToSrgb, linearToOklab and
// oklabToLinear from the prelude.
uniform float mat_noise;
uniform float mat_noise_type;

vec3 noise_post(vec3 color, vec2 fragCoord) {
    if (mat_noise <= 0.0)
        return color;
    vec2 noiseSeed = fragCoord + vec2(47.0, 113.0);
    if (mat_noise_type < 0.5)
        return color + (hash12(noiseSeed) - 0.5) * mat_noise;
    float grain = fineGrain(noiseSeed) * mat_noise;
    if (mat_noise_type < 1.5)
        return color + vec3(grain);
    // Clamp before re-encoding: a lightness pushed past the gamut yields
    // negative linear values, and pow() of a negative is undefined.
    vec3 lab = linearToOklab(srgbToLinear(clamp(color, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}
```

These are the stage 9 and 10 bodies of `material.frag` (its lines 573–592) rewritten as early-return functions; the arithmetic is identical. `material.frag` itself is untouched until Task 6.

- [ ] **Step 5: Write the registry**

`src/render_helpers/material/optics/mod.rs`:

```rust
//! Optics: the self-contained stages of the glass pipeline, renderer side.
//!
//! `OPTICS` is the one ordered registry. The shader is assembled from each
//! entry's `glsl` in this order, the program's uniform list is extended with
//! each entry's `uniforms`, each frame's uniform values come from `values`,
//! and the redraw deadline takes the minimum of every `next_change`.
//! Design: `docs/specs/2026-09-10-material-optics-design.md` §3.

use std::time::Duration;

use niri_config::signal::SignalMotionPolicy;
use niri_config::{Blur, ResolvedGlass};
use smithay::backend::renderer::gles::{Uniform, UniformName, UniformType};

pub mod noise;
pub mod saturation;

/// One pipeline stage. Implemented on a marker type per optic; the resolved
/// configuration arrives as `ResolvedGlass`, which owns every optic's state.
pub trait Optic {
    /// The registry name; matches `niri_config::material::optics::ORDER`.
    const NAME: &'static str;
    /// The optic's GLSL: its `uniform` declarations and `<NAME>_<hook>`
    /// functions.
    const GLSL: &'static str;
    /// The uniforms the GLSL declares, in declaration order.
    const UNIFORMS: &'static [(&'static str, UniformType)];
    /// This frame's uniforms, one per entry of `UNIFORMS`, in the same order.
    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>>;
    /// The next instant `values` changes with no config change; `None` for
    /// a static optic.
    fn next_change(_glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Option<Duration> {
        None
    }
}

/// What an optic may depend on beyond its own configuration.
#[derive(Debug, Clone, Copy)]
pub struct OpticFrame<'a> {
    /// The tile clock, unadjusted.
    pub now: Duration,
    pub motion: SignalMotionPolicy,
    pub animations_off: bool,
    /// Effective: configured and the global `blur` block is not off.
    pub backdrop_blur: bool,
    /// The global block, for the inherit-or-neutral rule.
    pub blur: &'a Blur,
    /// The window's jelly seed, component 0: per-window variation.
    pub seed: f32,
}

pub struct OpticEntry {
    pub name: &'static str,
    pub glsl: &'static str,
    pub uniforms: &'static [(&'static str, UniformType)],
    pub values: fn(&ResolvedGlass, &OpticFrame<'_>) -> Vec<Uniform<'static>>,
    pub next_change: fn(&ResolvedGlass, &OpticFrame<'_>) -> Option<Duration>,
}

impl OpticEntry {
    pub const fn of<O: Optic>() -> Self {
        Self {
            name: O::NAME,
            glsl: O::GLSL,
            uniforms: O::UNIFORMS,
            values: O::values,
            next_change: O::next_change,
        }
    }
}

/// The optics in render order.
pub static OPTICS: &[OpticEntry] = &[
    OpticEntry::of::<saturation::SaturationOptic>(),
    OpticEntry::of::<noise::NoiseOptic>(),
];

/// Every optic's uniform names, in `OPTICS` order, for the program.
pub fn uniform_names() -> Vec<UniformName<'static>> {
    OPTICS
        .iter()
        .flat_map(|entry| {
            entry
                .uniforms
                .iter()
                .map(|(name, ty)| UniformName::new(*name, *ty))
        })
        .collect()
}

/// Every optic's uniform values for this frame, in `OPTICS` order.
pub fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
    OPTICS
        .iter()
        .flat_map(|entry| (entry.values)(glass, ctx))
        .collect()
}

/// The earliest instant any optic changes on its own.
pub fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration> {
    OPTICS
        .iter()
        .filter_map(|entry| (entry.next_change)(glass, ctx))
        .min()
}
```

Prepend to `optics/saturation.rs`, above its tests:

```rust
//! `saturation`: stage 9, neutral at 1. Applies the inherit-or-neutral rule.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct SaturationOptic;

impl Optic for SaturationOptic {
    const NAME: &'static str = "saturation";
    const GLSL: &'static str = include_str!("../../shaders/material/saturation.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] =
        &[("mat_saturation", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        // A written value is a material optic and renders as written. An
        // omitted one inherits the global value only while backdrop blur is
        // effective, and is the neutral 1 otherwise.
        let amount = glass.saturation.amount.unwrap_or(if ctx.backdrop_blur {
            ctx.blur.saturation
        } else {
            1.
        });
        vec![Uniform::new("mat_saturation", amount as f32)]
    }
}
```

Prepend to `optics/noise.rs`, above its tests:

```rust
//! `noise`: stage 10, neutral at amount 0. Applies the inherit-or-neutral
//! rule to the amount; the grain type never inherits.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_1f),
        ("mat_noise_type", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let amount = glass.noise.amount.unwrap_or(if ctx.backdrop_blur {
            ctx.blur.noise
        } else {
            0.
        });
        vec![
            Uniform::new("mat_noise", amount as f32),
            // The shader reads the discriminant as float thresholds.
            Uniform::new("mat_noise_type", glass.noise.kind as u8 as f32),
        ]
    }
}
```

If `niri_config::signal::SignalMotionPolicy` is private at that path, check `niri-config/src/lib.rs` for the re-export of `signal` items and use that path; the type is declared in `niri-config/src/signal.rs` and `src/layout/tile.rs` reads it as `self.options.signal.motion`.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib render_helpers::material::optics`
Expected: 5 passed.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add src/render_helpers/material src/render_helpers/shaders/material
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): add the optic registry with saturation and noise optics"
```

---

### Task 6: Split the material shader and assemble it from the registry

**Files:**
- Create: `src/render_helpers/shaders/material/prelude.frag`, `src/render_helpers/shaders/material/main.frag`
- Delete: `src/render_helpers/shaders/material.frag`
- Modify: `src/render_helpers/shaders/mod.rs:153-216` (compile site), add `material_source()`, `material_uniform_names()`, and tests

**Interfaces:**
- Consumes: `crate::render_helpers::material::optics::{OPTICS, uniform_names}`.
- Produces: `pub(crate) fn material_source() -> String`, `pub(crate) fn material_uniform_names() -> Vec<UniformName<'static>>` in `shaders/mod.rs`.

- [ ] **Step 1: Write the failing tests**

Append to `src/render_helpers/shaders/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::render_helpers::material::optics::OPTICS;

    #[test]
    fn optic_order_matches_the_config_crate() {
        let renderer: Vec<&str> = OPTICS.iter().map(|entry| entry.name).collect();
        assert_eq!(renderer, niri_config::material::optics::ORDER);
    }

    #[test]
    fn every_optic_declares_its_uniforms_in_its_glsl() {
        for entry in OPTICS {
            for (name, _) in entry.uniforms {
                assert!(
                    entry.glsl.contains(&format!(" {name};")),
                    "{}: {name} is not declared in its GLSL",
                    entry.name
                );
            }
            assert!(
                entry.glsl.contains(&format!("{}_", entry.name)),
                "{}: no hook function",
                entry.name
            );
        }
    }

    #[test]
    fn material_uniform_names_are_unique() {
        let names = material_uniform_names();
        let unique: HashSet<_> = names.iter().map(|n| n.name.clone()).collect();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn material_source_is_prelude_then_optics_in_order_then_main() {
        let source = material_source();
        let mut last = 0;
        for marker in ["// ---- optic: saturation", "// ---- optic: noise", "// ---- main"] {
            let at = source.find(marker).unwrap_or_else(|| panic!("{marker} missing"));
            assert!(at > last, "{marker} out of order");
            last = at;
        }
        assert_eq!(source.matches("void main()").count(), 1);
        assert!(!source.starts_with("#version"));
        assert!(source.contains("saturation_post(glassColor"));
        assert!(source.contains("noise_post(glassColor"));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib render_helpers::shaders::tests`
Expected: FAIL to compile, `cannot find function material_source`.

- [ ] **Step 3: Split the shader file**

```bash
cd src/render_helpers/shaders
# prelude: everything before main(), minus the three post-stage uniforms (lines 31-33)
sed '31,33d' material.frag | sed -n '1,396p' > material/prelude.frag
# main: from `void main() {` to the end
sed -n '401,603p' material.frag > material/main.frag
git rm -q material.frag
cd -
```

Verify the boundaries: `head -1 src/render_helpers/shaders/material/main.frag` prints `void main() {`, and `tail -3 src/render_helpers/shaders/material/prelude.frag` ends with the closing brace of `filamentBand` followed by a blank line. `grep -n "mat_noise\|mat_saturation" src/render_helpers/shaders/material/prelude.frag` prints nothing.

In `material/main.frag`, replace the stage 9 and 10 block, which reads

```glsl
        vec3 glassColor = linearToSrgb(transmitted + specular + emissive);
        if (mat_saturation != 1.0) {
            const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
            glassColor = mix(vec3(dot(glassColor, luma)), glassColor, mat_saturation);
        }
        if (mat_noise > 0.0) {
```

through the closing brace of that `if (mat_noise > 0.0)` block (the line before `glassed = vec4(glassColor, 1.0) * coverage;`), with

```glsl
        vec3 glassColor = linearToSrgb(transmitted + specular + emissive);
        // Post hooks, in OPTICS order (render-pipeline.md stages 9 and 10).
        glassColor = saturation_post(glassColor, gl_FragCoord.xy);
        glassColor = noise_post(glassColor, gl_FragCoord.xy);
```

Add a header comment at the top of `prelude.frag`, before `precision highp float;`:

```glsl
// Material shader, part 1 of 3: core uniforms, shared globals, the helper
// library, the slab, and the refraction taps. shaders/mod.rs assembles this,
// then each optic's file in OPTICS order, then main.frag. No #version here:
// the compile path prepends it.
```

- [ ] **Step 4: Assemble from the registry**

In `src/render_helpers/shaders/mod.rs`, add after the `use` block:

```rust
use crate::render_helpers::material::optics::{self, OPTICS};

/// The material fragment shader: prelude, each optic's GLSL in `OPTICS`
/// order, then main. A comment marker per part keeps compile-error line
/// numbers locatable by hand.
pub(crate) fn material_source() -> String {
    let mut source = String::from(include_str!("material/prelude.frag"));
    for entry in OPTICS {
        source.push_str(&format!("\n// ---- optic: {}\n", entry.name));
        source.push_str(entry.glsl);
    }
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("material/main.frag"));
    source
}

/// The material program's uniforms: the core list, then every optic's.
pub(crate) fn material_uniform_names() -> Vec<UniformName<'static>> {
    let mut names = vec![
        UniformName::new("mat_win_rect", UniformType::_4f),
        UniformName::new("mat_geo_rect", UniformType::_4f),
        UniformName::new("mat_slab_rect", UniformType::_4f),
        UniformName::new("mat_area_size", UniformType::_2f),
        UniformName::new("mat_chamfer", UniformType::_1f),
        UniformName::new("mat_corner_radius", UniformType::_4f),
        UniformName::new("mat_jelly_move", UniformType::_2f),
        UniformName::new("mat_jelly_resize", UniformType::_2f),
        UniformName::new("mat_jelly_activity", UniformType::_1f),
        UniformName::new("mat_jelly_time", UniformType::_1f),
        UniformName::new("mat_jelly_seed", UniformType::_3f),
        UniformName::new("mat_bg_rect", UniformType::_4f),
        UniformName::new("mat_backdrop_rect", UniformType::_4f),
        UniformName::new("mat_ws_rect", UniformType::_4f),
        UniformName::new("mat_ws_color", UniformType::_4f),
        UniformName::new("mat_backdrop_color", UniformType::_4f),
        UniformName::new("mat_bg_prefilter_mix", UniformType::_1f),
        UniformName::new("mat_backdrop_prefilter_mix", UniformType::_1f),
        UniformName::new("mat_ior", UniformType::_1f),
        UniformName::new("mat_thickness", UniformType::_1f),
        UniformName::new("mat_attenuation_color", UniformType::_4f),
        UniformName::new("mat_attenuation_distance", UniformType::_1f),
        UniformName::new("mat_chromatic_aberration", UniformType::_1f),
        UniformName::new("mat_distortion", UniformType::_1f),
        UniformName::new("mat_distortion_scale", UniformType::_1f),
        UniformName::new("mat_samples", UniformType::_1f),
        UniformName::new("mat_anisotropic_blur", UniformType::_1f),
        UniformName::new("mat_jelly_ripple", UniformType::_1f),
        UniformName::new("mat_sig_accent", UniformType::_4f),
        UniformName::new("mat_sig_level", UniformType::_1f),
        UniformName::new("mat_sig_breath", UniformType::_1f),
        UniformName::new("mat_sig_light", UniformType::_3f),
        UniformName::new("mat_sig_impulse_env", UniformType::_4f),
        UniformName::new("mat_sig_impulse_prog", UniformType::_4f),
        UniformName::new("mat_sig_impulse_rgb0", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb1", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb2", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb3", UniformType::_3f),
        UniformName::new("mat_sig_impulse_resp", UniformType::_4i),
        UniformName::new("mat_sig_response", UniformType::_3i),
        UniformName::new("mat_sig_ring", UniformType::_2f),
        UniformName::new("mat_sig_focus", UniformType::_2f),
        UniformName::new("mat_sig_ring_color", UniformType::_3f),
        UniformName::new("mat_light_ior", UniformType::_1f),
    ];
    names.extend(optics::uniform_names());
    names
}
```

This is the existing inline list with `mat_noise`, `mat_noise_type`, and `mat_saturation` removed. Then replace the compile site:

```rust
        let material_source = material_source();
        let material_uniforms = material_uniform_names();
        let material = ShaderProgram::compile(
            renderer,
            &material_source,
            &material_uniforms,
            &[
                "niri_tex_win",
                "niri_tex_bg",
                "niri_tex_bg_high",
                "niri_tex_backdrop",
                "niri_tex_backdrop_high",
            ],
        )
        .map_err(|err| {
            warn!("error compiling material shader: {err:?}");
        })
        .ok();
```

The upload in `material/mod.rs` `draw()` still sends `mat_noise`, `mat_noise_type`, and `mat_saturation` by their unchanged names, so rendering is unchanged after this task; Task 7 moves those three uploads into the optic values.

- [ ] **Step 5: Run the tests, then the whole suite**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib render_helpers::shaders::tests` then `just test`
Expected: 4 passed; full suite green.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add src/render_helpers/shaders
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "refactor(material): assemble the glass shader from a prelude, the optics, and main"
```

---

### Task 7: Wire optic values into the element, the fingerprint, and the tile

**Files:**
- Modify: `src/render_helpers/material/mod.rs` (`MaterialRenderConfig`, `InputFingerprint`, `MaterialRenderElement`, `MaterialState::element`, `draw`, tests), `src/layout/tile.rs` (`resolve_material`, `MaterialDynamics`, `material_dynamics`, both render sites, tests)

**Interfaces:**
- Consumes: `optics::{OpticFrame, values}` from Task 5.
- Produces: `MaterialRenderConfig { material: ResolvedMaterial }` (no `noise`/`saturation`); `InputFingerprint.optics: Vec<Uniform<'static>>`; `MaterialState::element(..., glass_signal, optics: Vec<Uniform<'static>>, scale, ...)`; `MaterialDynamics.optics`; `Tile::optic_frame(&self, material: &MaterialState, now: Duration) -> OpticFrame<'_>`.

- [ ] **Step 1: Write the failing tests**

In `src/render_helpers/material/mod.rs` tests, replace `postprocess_change_advances_the_commit_in_place` with:

```rust
    #[test]
    fn optic_config_change_advances_the_commit_in_place() {
        let mut slot = Some(MaterialState::new(render_config("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();
        let initial_commit = slot.as_ref().unwrap().commit.get();

        let mut changed = render_config("frost");
        changed.material.glass.noise.amount = Some(0.04);
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        let noise_commit = slot.as_ref().unwrap().commit.get();
        assert_ne!(noise_commit, initial_commit);

        changed.material.glass.saturation.amount = Some(0.8);
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        assert_ne!(slot.as_ref().unwrap().commit.get(), noise_commit);
    }

    #[test]
    fn optic_values_are_damage() {
        let state = MaterialState::new(render_config("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let quiet = fingerprint(1, 1, &background_id, &backdrop_id);
        let mut grained = quiet.clone();
        grained.optics = vec![Uniform::new("mat_noise", 0.1f32)];

        let first = state.advance_commit(RenderTarget::Output, quiet.clone());
        let second = state.advance_commit(RenderTarget::Output, grained);
        let third = state.advance_commit(RenderTarget::Output, quiet);
        assert_ne!(first, second);
        assert_ne!(second, third);
    }
```

In `src/layout/tile.rs` tests, add a helper after `options_with` and rewrite the three post-processing tests to use it:

```rust
    /// The post-stage uniforms a tile would upload for the material "frost"
    /// under `options`: what the renderer sees, after the inherit rule.
    fn post_uniforms(options: &Options) -> Vec<smithay::backend::renderer::gles::Uniform<'static>> {
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let resolved = resolve_material(Some(&reference), options).unwrap();
        let frame = OpticFrame {
            now: Duration::ZERO,
            motion: options.signal.motion,
            animations_off: options.animations.off,
            backdrop_blur: resolved.material.glass.backdrop_blur,
            blur: &options.blur,
            seed: 0.,
        };
        optics::values(&resolved.material.glass, &frame)
    }

    fn uniform_f32(
        uniforms: &[smithay::backend::renderer::gles::Uniform<'static>],
        name: &str,
    ) -> f32 {
        match uniforms.iter().find(|u| u.name == name).unwrap().value {
            smithay::backend::renderer::gles::UniformValue::_1f(v) => v,
            ref other => panic!("{name}: {other:?}"),
        }
    }
```

Then in `material_postprocess_follows_effective_backdrop_blur`, replace

```rust
            let resolved = resolve_material(Some(&reference), &options).unwrap();
            assert_eq!((resolved.noise, resolved.saturation), expected);
```

with

```rust
            let uniforms = post_uniforms(&options);
            assert_eq!(
                (
                    uniform_f32(&uniforms, "mat_noise"),
                    uniform_f32(&uniforms, "mat_saturation")
                ),
                expected
            );
```

and delete its now-unused `reference`. Make the same replacement in `written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur`. In `noise_type_reaches_the_render_config_regardless_of_blur`, replace the two assertions with:

```rust
            let uniforms = post_uniforms(&options);
            assert_eq!(uniform_f32(&uniforms, "mat_noise_type"), noise_type as u8 as f32);
            assert_eq!(uniform_f32(&uniforms, "mat_noise"), 0.3);
```

Add `use crate::render_helpers::material::optics::{self, OpticFrame};` to the tile test module's imports (and to `tile.rs` itself in Step 3).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib render_helpers::material::tests::optic_` then `python3 tools/tt test-fast -- cargo test -p niri --lib layout::tile::tests`
Expected: FAIL to compile, `no field optics on InputFingerprint` and `cannot find optics`.

- [ ] **Step 3: Change the render element**

In `src/render_helpers/material/mod.rs`:

1. `InputFingerprint`: add after `glass_signal`:

```rust
    /// Every optic's uniform values this frame, in `OPTICS` order. Static
    /// optics repeat their values; an animated one changes them, which is
    /// damage.
    pub optics: Vec<Uniform<'static>>,
```

2. `MaterialRenderConfig`: delete the `noise: f32` and `saturation: f32` fields, leaving `pub material: ResolvedMaterial`.
3. `MaterialRenderElement`: delete `noise: f32,` and `saturation: f32,`; add `/// Optic uniforms for this frame, appended after the core uniforms.` `optics: Vec<Uniform<'static>>,`.
4. `MaterialState::element`: add a parameter `optics: Vec<Uniform<'static>>,` immediately after `glass_signal: GlassSignalInputs,`; in the struct literal delete `noise: self.config.noise,` and `saturation: self.config.saturation,` and add `optics,`.
5. `draw()`: the `let uniforms: Rc<[Uniform<'static>]> = Rc::new([ ... ]);` literal becomes a `Vec` with the three post entries removed and the optic values appended:

```rust
        let mut uniforms: Vec<Uniform<'static>> = vec![
            Uniform::new("mat_win_rect", self.win_rect),
            // ... every existing entry except mat_noise, mat_noise_type,
            // and mat_saturation, unchanged ...
            Uniform::new("mat_light_ior", g.light_ior as f32),
        ];
        uniforms.extend(self.optics.iter().cloned());
        let uniforms: Rc<[Uniform<'static>]> = uniforms.into();
```

6. Tests: `render_config` drops `noise: 0.,` and `saturation: 1.,`; `fingerprint2` adds `optics: Vec::new(),`.

- [ ] **Step 4: Change the tile**

In `src/layout/tile.rs`:

1. Import: `use crate::render_helpers::material::optics::{self, OpticFrame};`.
2. `resolve_material`: delete the `inherited` closure and the `noise` and `saturation` bindings; return `Some(MaterialRenderConfig { material })`. Trim its doc comment's last paragraph to: "The global `blur { off }` switch is applied here, so every consumer downstream reads one already-gated value rather than re-deriving it. The noise and saturation inherit rule lives in their optics, which read the gated value through `OpticFrame`."
3. `MaterialDynamics`: add `optics: Vec<smithay::backend::renderer::gles::Uniform<'static>>,`.
4. Add a method next to `material_dynamics`:

```rust
    /// What the optics may depend on this frame beyond their configuration.
    fn optic_frame<'a>(&'a self, material: &MaterialState, now: Duration) -> OpticFrame<'a> {
        OpticFrame {
            now,
            motion: self.options.signal.motion,
            animations_off: self.options.animations.off,
            backdrop_blur: material.material().glass.backdrop_blur,
            blur: &self.options.blur,
            seed: material.jelly_seed()[0],
        }
    }
```

5. In `material_dynamics`, after `let now = self.clock.now_unadjusted();` add `let optics = optics::values(glass, &self.optic_frame(material, now));` and add `optics,` to the returned `MaterialDynamics`.
6. At both render sites (the resize path near line 1690 and the normal path near line 1860): in the `InputFingerprint { ... }` literal add `optics: dynamics.optics.clone(),` after `glass_signal: dynamics.glass_signal_fingerprint,`; in the `material.element(...)` call add `dynamics.optics,` after `dynamics.glass_signal,`.

- [ ] **Step 5: Run the tests, then the whole suite**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib` then `just test`
Expected: green. Clippy may flag `too_many_arguments` on `element`; the `#[allow]` is already present.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add src/render_helpers/material src/layout/tile.rs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): upload optic values from the registry and fingerprint them"
```

---

### Task 8: Optic deadlines independent of the signal cache

**Files:**
- Modify: `src/layout/tile.rs` (`signal_tick_deadline` → `tick_deadline`, its call site near line 2025, tests)

**Interfaces:**
- Consumes: `optics::next_change`, `Tile::optic_frame` (Task 7).
- Produces: `pub fn tick_deadline(&self, location: Point<f64, Logical>, view: Rectangle<f64, Logical>, now: Duration) -> Option<Duration>`.

- [ ] **Step 1: Write the failing tests**

Add to the `tile.rs` tests module, after `only_a_focused_tile_drifts`:

```rust
    #[test]
    fn static_optics_report_no_deadline_without_a_signal_cache() {
        // An unfocused tile whose response lights nothing and that carries no
        // signal has no signal frame cache. Optic deadlines are still
        // evaluated; with only static optics registered they are None, and
        // the call must not short-circuit on the missing cache.
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::None, clock);
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(false, true, view);
        assert!(tile.signal_frame_cache.borrow().is_none());
        assert_eq!(tile.tick_deadline(Point::default(), view, Duration::ZERO), None);
    }

    #[test]
    fn a_focused_drifting_tile_keeps_its_signal_deadline() {
        // Start the fixture already focused: a focus *change* would begin a
        // crossfade from 0, and `focus_drift_hz` is 0 until that crossfade
        // has advanced, so a tile focused at the same instant reports no
        // drift yet. An already-focused tile has focus value 1.
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
        tile.active = true;
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, view);
        assert!(tile.focus_crossfade.is_none());
        assert!(tile.signal_frame_cache.borrow().is_some());
        let deadline = tile.tick_deadline(Point::default(), view, Duration::ZERO);
        assert!(deadline.is_some_and(|d| d > Duration::ZERO), "{deadline:?}");
    }

    #[test]
    fn an_out_of_view_slab_reports_no_deadline() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
        tile.active = true;
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, view);
        assert!(tile.tick_deadline(Point::default(), view, Duration::ZERO).is_some());
        let far = Point::from((10_000., 10_000.));
        assert_eq!(tile.tick_deadline(far, view, Duration::ZERO), None);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib layout::tile::tests`
Expected: FAIL to compile, `no method named tick_deadline`.

- [ ] **Step 3: Replace the method**

Replace `signal_tick_deadline` in `src/layout/tile.rs` with:

```rust
    /// Next instant this tile needs a redraw for its material: the earliest
    /// of any optic's own change and the sustained-signal or focus-drift
    /// bucket boundary, while the slab band is in view.
    ///
    /// The optic deadline does not sit behind the signal frame cache: an
    /// unfocused, signal-free window has no cache, and an animated optic
    /// must keep it redrawing all the same (design §3).
    pub fn tick_deadline(
        &self,
        location: Point<f64, Logical>,
        view: Rectangle<f64, Logical>,
        now: Duration,
    ) -> Option<Duration> {
        use crate::render_helpers::signal::{slab_in_view, tick_deadline};

        let material = self.material.as_ref()?;
        let glass = &material.material().glass;
        if !slab_in_view(location, self.tile_size(), glass.bevel, view) {
            return None;
        }
        let optic = optics::next_change(glass, &self.optic_frame(material, now));
        let signal = self
            .signal_frame_cache
            .borrow()
            .as_ref()
            .and_then(|(eff, inputs)| tick_deadline(eff, true, now, inputs.drift_hz));
        match (optic, signal) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
```

At the call site in `Tile::render` (near line 2025) rename `self.signal_tick_deadline(` to `self.tick_deadline(`.

- [ ] **Step 4: Run the tests, then the whole suite**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib layout::tile` then `just test`
Expected: green.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add src/layout/tile.rs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): evaluate optic redraw deadlines independently of the signal cache"
```

---

### Task 9: Byte-identical evidence for the migration

**Files:**
- Create: `docs/materials/2026-09-<dd>-material-optics-evidence.md` (dated the day of the run)
- Modify: `docs/materials/README.md` (index entry)

**Interfaces:**
- Consumes: `docs/materials/scripts/glass-noise-saturation-smoke.sh` and `glass-noise-type-smoke.sh`, both taking `IMPL` (niri binary) and `OUT` (artifact dir) and starting their own headless Weston unit. Requires `weston`, `kitty`, `swaybg`, `jq`, `rg`, ImageMagick 7.

- [ ] **Step 1: Build the before binary from the base commit**

```bash
git worktree add /mnt/ssd3/niri-material/before-optics 22d10019
mkdir -p /mnt/ssd3/niri-material/before-optics/.cargo
printf '[build]\ntarget-dir = "/mnt/ssd3/niri-material/target-before"\n' \
  > /mnt/ssd3/niri-material/before-optics/.cargo/config.toml
( cd /mnt/ssd3/niri-material/before-optics && cargo build --release -p niri )
ls -la /mnt/ssd3/niri-material/target-before/release/niri
```

A separate target dir avoids the stale-artifact sharing between worktrees.

- [ ] **Step 2: Build the after binary**

```bash
cargo build --release -p niri
ls -la /mnt/ssd3/niri-material/target/release/niri
```

- [ ] **Step 3: Run both smokes against both binaries**

```bash
E=/mnt/ssd3/niri-material/evidence-optics; mkdir -p $E
B=/mnt/ssd3/niri-material/target-before/release/niri
A=/mnt/ssd3/niri-material/target/release/niri
for s in glass-noise-saturation-smoke glass-noise-type-smoke; do
  IMPL=$B OUT=$E/before-$s bash docs/materials/scripts/$s.sh
  IMPL=$A OUT=$E/after-$s  bash docs/materials/scripts/$s.sh
done
```

Expected: each run exits 0 (every in-script assertion held, including each script's own determinism check).

- [ ] **Step 4: Compare every capture pixel for pixel**

The scripts' PNGs are not byte-stable across runs: ImageMagick writes a
creation timestamp into the crops and analysis images it produces, so two
files with identical pixels differ in bytes. Compare decoded pixels with the
absolute-error metric, which counts differing pixels and exits non-zero when
any differ:

```bash
cd /mnt/ssd3/niri-material/evidence-optics
status=0
for s in glass-noise-saturation-smoke glass-noise-type-smoke; do
  for f in before-$s/*.png; do
    g=after-$s/$(basename "$f")
    if [ ! -f "$g" ]; then echo "MISSING $g"; status=1; continue; fi
    if magick compare -metric AE "$f" "$g" null: >/dev/null 2>&1; then
      echo "SAME $f"
    else
      echo "DIFF or ERROR $f"
      status=1
    fi
  done
done
echo "status=$status"
cd -
exit $status
```

Run it as a script file (`bash compare.sh`) so the final `exit` is the
verdict. Expected: every comparison prints `SAME ...`, `status=0`, exit 0.
Any `DIFF or ERROR` or `MISSING` fails the migration: check for image-read
errors, then diff the two shader sources (`prelude` +
optics + `main` against the base commit's `material.frag`) for an
arithmetic change before anything else.

- [ ] **Step 5: Write the evidence doc**

`docs/materials/2026-09-<dd>-material-optics-evidence.md`, with the date of the run in the file name and these sections filled from the run, no value left symbolic:

```markdown
# Material optics: byte-identical migration evidence

**Design:** `../specs/2026-09-10-material-optics-design.md` §5.
**Run:** <date>, headless Weston (`weston --backend=headless --renderer=gl`), nested niri.

## Binaries

| | Commit | sha256 |
| --- | --- | --- |
| before | `22d10019` | <sha256sum of the before binary> |
| after | `<HEAD sha>` | <sha256sum of the after binary> |

## Captures

Every PNG the two smokes produced, before against after, decoded and compared pixel for pixel (`magick compare -metric AE`; bytes differ by ImageMagick timestamps alone):

| Smoke | Captures | Zero differing pixels |
| --- | --- | --- |
| glass-noise-saturation-smoke | <count> | <count> |
| glass-noise-type-smoke | <count> | <count> |

Both scripts' own assertions held on both binaries (exit 0 each run).

## Hashes

<the four `SHA256SUMS` files' contents, or the path they were archived to>
```

Add an index line to `docs/materials/README.md` under the material documentation list, after the `2026-09-10-material-optics-design.md` entry:

```markdown
- `2026-09-<dd>-material-optics-evidence.md`: byte-identical captures before and after the saturation and noise optic migration.
```

- [ ] **Step 6: Remove the before worktree**

```bash
git worktree remove --force /mnt/ssd3/niri-material/before-optics
rm -rf /mnt/ssd3/niri-material/target-before
```

- [ ] **Step 7: Commit**

```bash
git add docs/materials
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "docs(materials): record the byte-identical optics migration evidence"
```

---

### Task 10: The contributor guide and the remaining docs

**Files:**
- Create: `docs/materials/adding-an-optic.md`
- Modify: `docs/materials/material-config.md` (`## Optics` section), `docs/materials/render-pipeline.md` (sources, §2, §3 rows 9 and 10), `docs/materials/README.md`, `docs/materials/upstream-divergence.md` (feature table paths), `docs/specs/2026-09-10-material-optics-design.md` (status)

- [ ] **Step 1: Write the guide**

`docs/materials/adding-an-optic.md`:

````markdown
# Adding an optic

An optic is one stage of the glass pipeline: its KDL node, its resolved
values, its uniforms, one GLSL file, and one docs section. This is the whole
recipe, using `noise` as the worked example. Design:
`../specs/2026-09-10-material-optics-design.md`.

## 1. The config side: `niri-config/src/material/optics/<name>.rs`

The node struct (knuffel), the resolved struct with `Default`, `resolve`,
and `params`. `noise` carries an amount and a `type=` property:

```rust
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedNoise {
    pub amount: Option<f64>,
    pub kind: NoiseType,
}

pub fn resolve(node: Option<Noise>) -> ResolvedNoise { /* defaults for omissions */ }

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "noise",
        kind: ParamKind::inherit::<FloatOrInt<0, 1>>(),
        write: |v| format!("noise {v}"),
        read: Some(|g| g.noise.amount),
    }, /* one spec per value the node carries */]
}
```

`ParamKind::float::<T>(default, unit)` takes its range from `T`, which must
be the type of the node field; `read` returns the resolved value. A test
(`material_parameter_specs_match_the_parser`) writes the min, the max, and a
value just outside each through `write` and checks `read`, so a spec that
disagrees with the field fails. The `node` string is the one fact nothing
checks: spell it as the table should show it, words separated by spaces
(`"noise type="`).

Add a cross-parameter `pub fn validate(&self) -> Result<(), String>` on the
resolved struct only when the optic has such a rule, and call it from
`Material::validate`.

Then three one-line registrations in `niri-config/src/material/`:

- `optics/mod.rs`: `pub mod <name>;`, the name appended to `ORDER`, and
  `specs.extend(<name>::params());` in `params()`.
- `mod.rs`: `#[knuffel(child)] pub <name>: Option<optics::<name>::<Node>>`
  on `Glass`, `pub <name>: optics::<name>::Resolved<Name>` on
  `ResolvedGlass` and its `Default`, and
  `<name>: optics::<name>::resolve(g.<name>),` in `Material::resolve`.
- `lib.rs`: re-export the public types.

Tests for parse, defaults, and every validation error string go in
`niri-config/src/lib.rs` beside the other material tests.

## 2. The GLSL: `src/render_helpers/shaders/material/<name>.frag`

Declare your uniforms with the prefix `mat_<name>_` (the two migrated optics
keep their older names) and define one function per hook you act at, named
`<name>_<hook>`:

| Hook | Signature | Return at neutral |
| --- | --- | --- |
| `normal` | `vec3 <name>_normal(vec3 n, vec2 p)` | `n` |
| `specular` | `vec3 <name>_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)` | `specular` |
| `emissive` | `vec3 <name>_emissive(vec2 p, vec3 n, vec3 att, float innerDist)` | `vec3(0.0)` |
| `post` | `vec3 <name>_post(vec3 color, vec2 fragCoord)` | `color` |

Your file is concatenated after `prelude.frag`, so the helper library is
available: `snoise`, `snoiseFractal`, `hash12`, `fineGrain`, `srgbToLinear`,
`linearToSrgb`, `linearToOklab`, `oklabToLinear`, `sdRoundedBox`, and the
slab globals `g_center`, `g_half`, `g_outer_r`. `mat_jelly_seed` is the
per-window seed; `mat_thickness` and `mat_ior` are the slab's. Put the
neutral check first so an unconfigured optic costs one branch.

## 3. The renderer side: `src/render_helpers/material/optics/<name>.rs`

A marker type implementing `Optic`:

```rust
pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_1f),
        ("mat_noise_type", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        // one Uniform per UNIFORMS entry, same order
    }

    // Only for an animated optic: when `values` next changes on its own.
    // fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration>
}
```

`OpticFrame` carries the clock, the motion policy, the animations switch,
whether backdrop blur is effective, the global `blur` block, and the window
seed. Damage and redraw scheduling are automatic: the values join the frame
fingerprint, and `next_change` joins the tile's tick deadline. Unit tests
for `values` under each input that matters go in the same file.

Then two registrations:

- `optics/mod.rs`: `pub mod <name>;` and `OpticEntry::of::<<name>::<Name>Optic>()`
  appended to `OPTICS` at the stage position (render order; it must match
  `ORDER` in niri-config, a test pins the two).
- `shaders/material/main.frag`: one call per hook, at the hook's position,
  in `OPTICS` order.

## 4. The docs

- Run `MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs`
  to regenerate the parameter table in `material-config.md`.
- Add a `### <name>` section under `## Optics` in `material-config.md`: what
  it does, which stage it acts at, and its neutral.
- Add the stage's row detail to `render-pipeline.md` §3 and, if Prism maps
  it, the Prism key to §5.

## 5. Proving it

- `just test` for the unit tests and the two table tests.
- A smoke under `docs/materials/scripts/` on the headless Weston host: the
  neutral configuration renders identically to a material that never names
  the optic, the claimed effect is measurable, and the frame cost is
  recorded in an evidence doc.
````

- [ ] **Step 2: Add the Optics section to the config reference**

In `docs/materials/material-config.md`, before `## Signal responses`, insert:

```markdown
## Optics

The glass pipeline is a slab plus an ordered list of optics, each a stage
with its own node, uniforms, and GLSL. Every optic has a neutral
configuration, the one a material that never names it resolves to; at its
neutral the optic changes nothing. Contributors: see `adding-an-optic.md`.

### saturation

Stage 9. `saturation <amount>` mixes the encoded glass colour toward its
luma. Neutral at 1; 0 is grayscale. An omitted amount inherits the global
`blur` block's `saturation` while backdrop blur is effective and is 1
otherwise.

### noise

Stage 10. `noise <amount> type=<type>` grains the encoded glass colour per
screen pixel; `white`, `fine`, and `lightness` are described above. Neutral
at amount 0. An omitted amount inherits the global `blur` block's `noise`
while backdrop blur is effective and is 0 otherwise; the type never
inherits.
```

- [ ] **Step 3: Update the pipeline doc**

In `docs/materials/render-pipeline.md`:

- Sources paragraph: `src/render_helpers/material.rs` → `src/render_helpers/material/mod.rs`; `src/render_helpers/shaders/material.frag` → `src/render_helpers/shaders/material/prelude.frag`, `main.frag`, and one file per optic; add `src/render_helpers/material/optics/`.
- §2, after "Uniforms carry the slab frame ...", add: "The program is assembled at compile time from `prelude.frag`, each optic's GLSL in `OPTICS` order (`src/render_helpers/material/optics/mod.rs`), and `main.frag`; each optic appends its uniforms and uploads their values through `Optic::values`, and an animated optic's `next_change` joins the tile's redraw deadline."
- §3 rows 9 and 10: prefix the stage names with the optic: `**Saturation** (optic \`saturation\`)` and `**Noise** (optic \`noise\`)`, and append to each row's Parameters cell "; neutral 1" and "; neutral 0".
- §3 intro: after the table, add: "Stages 2, 5, 6, and 9–10 are also the four optic hooks `normal`, `specular`, `emissive`, and `post` (`adding-an-optic.md`)."

- [ ] **Step 4: Update the index, the divergence feature table, and the spec status**

`docs/materials/README.md`: add `- \`adding-an-optic.md\`: the recipe for a new pipeline stage.` under the material documentation list, and change the spec's index line from "not implemented" to "implemented".

`docs/materials/upstream-divergence.md` feature table (outside the generated markers): change `\`src/render_helpers/material.rs\`, \`shaders/material.frag\`` to `\`src/render_helpers/material/\`, \`shaders/material/\`` and `\`niri-config/src/material.rs\`` to `\`niri-config/src/material/\``.

`docs/specs/2026-09-10-material-optics-design.md`: change the status line to `**Status:** implemented on \`feat/material-397fcb\` (this plan: \`docs/plans/2026-09-10-material-optics.md\`); sections 1–6 landed, byte-identical evidence in \`docs/materials/<evidence file>\`. Sections 7–9 (the three optics, presets, Prism) are separate plans.`

- [ ] **Step 5: Check and commit**

```bash
git add docs
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
just check
git commit -m "docs(materials): add the adding-an-optic guide and record the optics stages"
```

---

## Self-review

**Spec coverage.** §1 hooks and neutral: Tasks 5, 6, 10 (the `normal`, `specular`, `emissive` hooks have no optic yet and are documented in the guide; their first call sites arrive with cracks, iridescence, and aurora). §2 file layout and registrations: Tasks 1–6. §3 trait, frame, registry, damage, scheduling: Tasks 5, 7, 8. §4 assembly: Task 6. §5 migration and byte-identical check: Tasks 2, 3, 7, 9. §6 typed specs, two tests, narrowed claim: Tasks 1, 4, 10. §10 task shape: the goal carries this plan's steps. §11 verification: unit tests in Tasks 2–8, layout tests in Task 8, evidence in Task 9; the aurora integration check belongs to the aurora plan.

**Review fixes (2026-09-10).** Scalar bounds are tested without cross-parameter validation (Task 4); captures are compared as decoded pixels with a non-zero exit on any difference (Task 9); the focused-deadline fixture starts focused so no crossfade gates the drift (Task 8); every commit regenerates the divergence report first; every test run goes through `tools/tt` with one filter.

**Placeholders.** The only angle-bracket values are the evidence doc's date and measured results in Task 9, which come from the run.

**Type consistency.** `OpticFrame<'a>` fields (`now`, `motion`, `animations_off`, `backdrop_blur`, `blur`, `seed`) match between Task 5's definition and Tasks 7 and 8's construction. `Optic::values(glass, ctx)` is a static function on a marker type in Task 5 and the guide; `OpticEntry::of::<T>()` reads `T::values` as a fn pointer. `ResolvedGlass.saturation.amount` and `.noise.amount` / `.noise.kind` are used identically in Tasks 2, 3, 5, 7. `InputFingerprint.optics` and `MaterialDynamics.optics` are both `Vec<Uniform<'static>>`.
