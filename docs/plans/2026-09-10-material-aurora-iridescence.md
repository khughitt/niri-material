# Material Aurora and Iridescence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the `iridescence` and `aurora` optics to the glass pipeline through the optic recipe, ship the `rainbow` and `aurora` presets with their install plumbing, and record each optic's frame cost (and aurora's redraw rate against `drift-hz`) on the headless harness.

**Architecture:** Each optic follows `docs/materials/adding-an-optic.md`: one file in `niri-config` (node, resolved struct, `resolve`, `params`, and for aurora `validate`), one file in `niri` (marker type implementing `Optic`), one GLSL file, three one-line registrations, and a docs section. `iridescence` acts at the `specular` hook and is static. `aurora` acts at the `emissive` hook and is animated: `signal.rs` gains `phase_on` and `next_boundary_on`, the existing 10 s drift clock generalised over its period, and aurora runs them over a 600 s period so its `next_change` joins the tile deadline at `drift-hz` buckets per second. Presets are one-material KDL files under `resources/materials/` installed to `/usr/share/niri/materials/`.

**Tech Stack:** Rust (workspace crates `niri-config`, `niri`), knuffel (KDL), smithay GLES renderer (`Uniform`, `UniformType`), GLSL ES 1.00, bash smokes on a headless Weston host with Tracy 0.13.1 for redraw counts and GPU zone timing, ImageMagick 7 for decoded-pixel comparison, `just` front door.

**Spec:** `docs/specs/2026-09-10-material-optics-design.md` §7.2, §7.3, §8, §11. The optic recipe is `docs/materials/adding-an-optic.md`.

## Global Constraints

- Branch `material-f0fc7b`, worktree `.worktrees/material-f0fc7b`; base commit `7a2f78e9`. Run every command from the worktree. Paths shown to the user are prefixed `.worktrees/material-f0fc7b/`.
- Conventional commits; no AI-attribution or session trailers. `tasks done <id> "<what landed>"` goes in the same commit as the code for the task it closes; `tasks check` before every commit.
- Every commit passes the pre-commit hook (`just check`: ops-check, `cargo fmt --check`, `cargo clippy --all --all-targets`, tools unit tests, `tasks check`, `upstream-report --check`, `package-pin --check`).
- Every test run goes through a `just` recipe, as AGENTS.md requires. Focused runs are `python3 tools/tt test-fast -- cargo test -p <crate> [--lib] <one filter>` (exactly one positional filter per invocation); the full suite is `just test`.
- `tools/upstream-report --check` compares the committed report against the staged tree and every fork path counts, so a commit that adds a file must regenerate the report first: `python3 tools/upstream-report && git add docs/materials/upstream-divergence.md` after `git add` and before `git commit`. Every commit step below does this.
- Every amount is `0–1` and defaults to `0`; a material that does not name the optic renders exactly as today. Uniforms use the prefix `mat_<optic>_` (the amount itself is `mat_<optic>`). Hook functions are `<optic>_<hook>`; transforming hooks return their input at the neutral, `emissive` returns `vec3(0.0)`, and the neutral check is the first line.
- `ORDER` in `niri-config/src/material/optics/mod.rs` and `OPTICS` in `src/render_helpers/material/optics/mod.rs` must agree at every commit (a niri test pins them). Final render order: `iridescence`, `aurora`, `saturation`, `noise`. A config-side task therefore adds its module without registering it in `ORDER`; the renderer task registers both sides in one commit.
- The parameter table in `docs/materials/material-config.md` is generated: after any `params()` change run `MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs` and commit the regenerated block.
- The shader compiles at runtime. After any `.frag` change, validate the assembled source offline through a real file (`glslangValidator` cannot read a process substitution: "can't read input file"): write `#version 100` plus `prelude.frag`, each optic's `.frag` in `OPTICS` order, and `main.frag` to `target/material.frag` (the per-machine, Dropbox-ignored build dir) and run `glslangValidator -S frag target/material.frag`; expect exit 0. The exact command is in Tasks 2 and 5.
- If the hook fails with a "no field" compile error the source contradicts, run `cargo clean -p niri-config -p niri` and retry (shared target dir across worktrees).
- Live captures run on a headless Weston host started by the smoke itself, never on the desktop session. `NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material` holds the retained Tracy 0.13.1 tools under `material-roughness-b220152d/tools`.
- Spec deviation, decided here: §7.3's loop circle radius of 40 noise units moves the field 105 px/s at the 0.004 scale (251 noise units per 600 s lap), which is not a slow field. The radius is a GLSL constant `AURORA_LOOP_RADIUS = 2.0` (about 5 px/s). Task 8 corrects §7.3 to the landed value.

---

## File structure

| Path | Responsibility |
| --- | --- |
| `niri-config/src/material/optics/iridescence.rs` | `Iridescence` node type, `ResolvedIridescence`, `resolve`, `params` |
| `niri-config/src/material/optics/aurora.rs` | `Aurora` block node, `validate`, `ResolvedAurora` with its defaults, `resolve`, `params` |
| `niri-config/src/material/optics/mod.rs` | `ORDER`, aggregated `params()` |
| `niri-config/src/material/mod.rs` | `Glass` fields, `ResolvedGlass` fields and defaults, `resolve`, `validate` calling `Aurora::validate` |
| `niri-config/src/lib.rs` | re-exports; parse, default, and validation tests; preset file test |
| `src/render_helpers/signal.rs` | `phase_on`, `next_boundary_on`, `buckets_on`; `drift` and `drift_next_boundary` as calls to them |
| `src/render_helpers/material/optics/iridescence.rs` | `IridescenceOptic` |
| `src/render_helpers/material/optics/aurora.rs` | `AuroraOptic`, `AURORA_PERIOD` |
| `src/render_helpers/material/optics/mod.rs` | `OPTICS` registry |
| `src/render_helpers/shaders/material/iridescence.frag` | `mat_iridescence`, `iridescence_specular` |
| `src/render_helpers/shaders/material/aurora.frag` | `mat_aurora*`, `aurora_emissive` |
| `src/render_helpers/shaders/material/main.frag` | the two hook calls |
| `src/render_helpers/shaders/mod.rs` | assembly test markers |
| `src/layout/tile.rs` | `material_tile` fixture; the aurora deadline test |
| `resources/materials/rainbow.kdl`, `resources/materials/aurora.kdl` | presets |
| `packaging/arch/PKGBUILD`, `Cargo.toml` | install to `/usr/share/niri/materials/` |
| `docs/materials/material-config.md` | generated table; `### iridescence`, `### aurora`, `## Presets` |
| `docs/materials/render-pipeline.md` | stage 5 and 6 rows, §5 mapping rows |
| `docs/materials/scripts/glass-optic-smoke-lib.sh` | shared nested-host, capture, compare, and Tracy helpers |
| `docs/materials/scripts/glass-iridescence-smoke.sh`, `glass-aurora-smoke.sh` | the two smokes |
| `docs/materials/scripts/glass-parameter-sweep.sh` | allowlist entries for the two keys |
| `docs/materials/<run-date>-material-iridescence-evidence.md`, `<run-date>-material-aurora-evidence.md` | evidence |
| `docs/specs/2026-09-10-material-optics-design.md` | status header and §7.3 radius correction |

---

### Task 1: Iridescence optic in niri-config

**Files:**
- Create: `niri-config/src/material/optics/iridescence.rs`
- Modify: `niri-config/src/material/optics/mod.rs` (module export only)
- Modify: `niri-config/src/material/mod.rs` (`Glass`, `ResolvedGlass`, `Default`, `resolve`)
- Modify: `niri-config/src/lib.rs` (re-export, tests)

**Interfaces:**
- Produces: `pub type Iridescence = FloatOrInt<0, 1>`; `pub struct ResolvedIridescence { pub amount: f64 }` (`Default` amount 0); `pub fn resolve(node: Option<Iridescence>) -> ResolvedIridescence`; `pub fn params() -> Vec<ParamSpec>`; `ResolvedGlass.iridescence: ResolvedIridescence`.

- [ ] **Step 1: Write the failing tests**

Add to the material tests in `niri-config/src/lib.rs`, beside `noise_resolves_through_its_optic`:

```rust
    #[test]
    fn iridescence_resolves_through_its_optic() {
        let written = do_parse(r##"material "gem" { glass { iridescence 0.8; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.iridescence,
            ResolvedIridescence { amount: 0.8 }
        );
        let omitted = do_parse(r##"material "gem" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.iridescence,
            ResolvedIridescence::default()
        );
        assert_eq!(ResolvedIridescence::default().amount, 0.);
    }

    #[test]
    fn glass_iridescence_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"gem\" {{ glass {{ iridescence {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }
```

Add `ResolvedIridescence` to the test module's imports (the tests import from `crate::` / `super::*`; match how `ResolvedNoise` is imported there).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config iridescence`
Expected: compile error, `ResolvedIridescence` not found.

- [ ] **Step 3: Write the optic module**

`niri-config/src/material/optics/iridescence.rs`:

```rust
//! `iridescence <amount>`: stage 5, a thin-film hue on the Fresnel glint.
//! Neutral at amount 0.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Iridescence = FloatOrInt<0, 1>;

/// Final iridescence state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedIridescence {
    /// Mix of the hued glint over the plain one; 0 is the plain glint.
    pub amount: f64,
}

pub fn resolve(node: Option<Iridescence>) -> ResolvedIridescence {
    ResolvedIridescence {
        amount: node.map_or(ResolvedIridescence::default().amount, |x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "iridescence",
        kind: ParamKind::float::<Iridescence>(ResolvedIridescence::default().amount, "—"),
        write: |v| format!("iridescence {v}"),
        read: Some(|g| Some(g.iridescence.amount)),
    }]
}
```

- [ ] **Step 4: Register the module and the field (not yet in `ORDER`)**

In `niri-config/src/material/optics/mod.rs` add `pub mod iridescence;` before `pub mod noise;` (the module list is alphabetical). Leave `ORDER` and `params()` unchanged in this task (Task 2 registers both sides together).

In `niri-config/src/material/mod.rs`:

- `Glass`: after the `roughness` field add
  ```rust
      #[knuffel(child, unwrap(argument))]
      pub iridescence: Option<optics::iridescence::Iridescence>,
  ```
- `ResolvedGlass`: after `roughness: f64,` add `pub iridescence: optics::iridescence::ResolvedIridescence,`
- `impl Default for ResolvedGlass`: after `roughness: 0.,` add `iridescence: optics::iridescence::ResolvedIridescence::default(),`
- `Material::resolve`: after `roughness: g.roughness.map_or(d.roughness, |x| x.0),` add `iridescence: optics::iridescence::resolve(g.iridescence),`

In `niri-config/src/lib.rs` add the re-export line after the noise one:

```rust
pub use crate::material::optics::iridescence::{Iridescence, ResolvedIridescence};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config iridescence`
Expected: both tests PASS.

Run: `python3 tools/tt test-fast -- cargo test -p niri-config material_parameter`
Expected: PASS (the table is unchanged because `params()` is not aggregated yet). Run: `python3 tools/tt test-fast -- cargo test -p niri --lib optic_order`
Expected: PASS (`ORDER` unchanged).

- [ ] **Step 6: Commit**

```bash
tasks start material-b42087
git add niri-config/src/material/optics/iridescence.rs niri-config/src/material/optics/mod.rs niri-config/src/material/mod.rs niri-config/src/lib.rs
tasks done material-b42087 "Iridescence node, resolved struct, resolve, params, and parse tests in niri-config"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): iridescence optic in niri-config"
```

---

### Task 2: Iridescence optic in niri, its GLSL, and its docs

**Files:**
- Create: `src/render_helpers/material/optics/iridescence.rs`
- Create: `src/render_helpers/shaders/material/iridescence.frag`
- Modify: `src/render_helpers/material/optics/mod.rs` (`pub mod`, `OPTICS`)
- Modify: `src/render_helpers/shaders/material/main.frag` (specular hook call)
- Modify: `src/render_helpers/shaders/mod.rs` (assembly test)
- Modify: `niri-config/src/material/optics/mod.rs` (`ORDER`, `params()`)
- Modify: `docs/materials/material-config.md`, `docs/materials/render-pipeline.md`

**Interfaces:**
- Consumes: `ResolvedGlass.iridescence.amount` from Task 1.
- Produces: `IridescenceOptic` with uniform `mat_iridescence` (`_1f`); GLSL `vec3 iridescence_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)`; `ORDER == ["iridescence", "saturation", "noise"]` until Task 5 inserts `aurora`.

- [ ] **Step 1: Write the failing tests**

In `src/render_helpers/material/optics/iridescence.rs` (the file does not exist yet; write it with the tests and a stub `pub struct IridescenceOptic;` so the test names exist):

```rust
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::{Blur, ResolvedGlass, ResolvedIridescence};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;

    fn frame(blur: &Blur) -> OpticFrame<'_> {
        OpticFrame {
            now: Duration::ZERO,
            motion: niri_config::signal::SignalMotionPolicy::Full,
            animations_off: false,
            backdrop_blur: false,
            blur,
            seed: 0.,
        }
    }

    #[test]
    fn the_amount_is_the_only_uniform_and_the_optic_is_static() {
        let glass = ResolvedGlass {
            iridescence: ResolvedIridescence { amount: 0.8 },
            ..Default::default()
        };
        let blur = Blur::default();
        let values = IridescenceOptic::values(&glass, &frame(&blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_iridescence");
        assert_eq!(values[0].value, UniformValue::_1f(0.8));
        assert_eq!(IridescenceOptic::next_change(&glass, &frame(&blur)), None);
    }
}
```

In `src/render_helpers/shaders/mod.rs`, extend `material_source_is_prelude_then_optics_in_order_then_main`: the marker list becomes `"// ---- optic: iridescence"`, `"// ---- optic: saturation"`, `"// ---- optic: noise"`, `"// ---- main"`, and add `assert!(source.contains("iridescence_specular(specular"));`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib iridescence`
Expected: compile error (`Optic` not implemented / `values` missing). Run: `python3 tools/tt test-fast -- cargo test -p niri --lib material_source_is_prelude`
Expected: FAIL, `// ---- optic: iridescence missing`.

- [ ] **Step 3: Write the optic, the GLSL, and the registrations**

`src/render_helpers/material/optics/iridescence.rs` (above the tests):

```rust
//! `iridescence`: stage 5, neutral at amount 0. Static: the amount is its
//! only uniform.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct IridescenceOptic;

impl Optic for IridescenceOptic {
    const NAME: &'static str = "iridescence";
    const GLSL: &'static str = include_str!("../../shaders/material/iridescence.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] =
        &[("mat_iridescence", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![Uniform::new(
            "mat_iridescence",
            glass.iridescence.amount as f32,
        )]
    }
}
```

`src/render_helpers/shaders/material/iridescence.frag`:

```glsl
// Optic: iridescence (render-pipeline.md stage 5). Neutral at amount 0.
uniform float mat_iridescence;

// A thin-film hue from the view angle: the glint runs through the cosine
// palette two and a half times from face-on to edge-on, so the chamfer
// carries a rainbow band and the flat face sits at the palette's start.
// Runs before the signal accent mix, so an accent still tints the result.
vec3 iridescence_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine) {
    if (mat_iridescence <= 0.0)
        return specular;
    const float TAU = 6.28318530718;
    float hue = fract(2.5 * (1.0 - surfaceCosine));
    vec3 palette = 0.5 + 0.5 * cos(TAU * (hue + vec3(0.0, 1.0 / 3.0, 2.0 / 3.0)));
    return mix(specular, specular * palette * 2.0, mat_iridescence);
}
```

`src/render_helpers/material/optics/mod.rs`: add `pub mod iridescence;` (alphabetical, before `noise`) and make the registry

```rust
pub static OPTICS: &[OpticEntry] = &[
    OpticEntry::of::<iridescence::IridescenceOptic>(),
    OpticEntry::of::<saturation::SaturationOptic>(),
    OpticEntry::of::<noise::NoiseOptic>(),
];
```

`niri-config/src/material/optics/mod.rs`:

```rust
pub const ORDER: &[&str] = &["iridescence", "saturation", "noise"];

pub fn params() -> Vec<ParamSpec> {
    let mut specs = iridescence::params();
    specs.extend(saturation::params());
    specs.extend(noise::params());
    specs
}
```

`src/render_helpers/shaders/material/main.frag`: directly after the line `vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));` insert

```glsl
        // Specular hooks, in OPTICS order (render-pipeline.md stage 5).
        specular = iridescence_specular(specular, surfaceNormal, surfaceCosine);
```

so the hook runs before the `if (mat_sig_accent.w > 0.0 && mat_sig_light.z > 0.0)` accent mix.

- [ ] **Step 4: Validate the shader offline**

Run:

```bash
{ printf '#version 100\n'; cat src/render_helpers/shaders/material/prelude.frag src/render_helpers/shaders/material/iridescence.frag src/render_helpers/shaders/material/saturation.frag src/render_helpers/shaders/material/noise.frag src/render_helpers/shaders/material/main.frag; } > target/material.frag
glslangValidator -S frag target/material.frag
```

Expected: exit 0, no errors.

- [ ] **Step 5: Regenerate the table and write the docs**

Run: `MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs`
Expected: PASS; `docs/materials/material-config.md` gains the row `| \`iridescence\` | float | 0 | 0–1 | — |` before the `saturation` row.

In `docs/materials/material-config.md`, under `## Optics`, insert before `### saturation`:

```markdown
### iridescence

Stage 5. `iridescence <amount>` gives the Fresnel glint a thin-film hue
from the view angle: `hue = fract(2.5 * (1 - cos))` through the cosine
palette `0.5 + 0.5 * cos(2π (hue + (0, ⅓, ⅔)))`, and the glint becomes
`mix(glint, glint * palette * 2, amount)`. It runs before the signal accent
mix, so an accent still tints the result. Its explicit neutral is 0, and
omission is 0; nothing inherits. The `rainbow` preset pairs it with
`chromatic-aberration`, which is the dispersion the refracted image carries;
iridescence colours the edge light.
```

In `docs/materials/render-pipeline.md`:

- stage 5 row: change the parameters cell to `` `ior`, `iridescence` (optic `iridescence`; neutral 0), response `attention`, signal accent `` and append to the description: `Then the \`iridescence\` optic hues the glint from the view angle, before the accent mix.`
- §5 table: add the row `` | `iridescence` | (pending, prism-763054) | 5 | `` after the `chromatic-aberration` row.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib iridescence`
Expected: PASS. Run: `python3 tools/tt test-fast -- cargo test -p niri --lib shaders::tests`
Expected: PASS (order pin, uniform declarations, assembly). Run: `python3 tools/tt test-fast -- cargo test -p niri-config material_parameter`
Expected: PASS (table and parser tests).

- [ ] **Step 7: Commit**

```bash
tasks start material-fad5de
git add src/render_helpers/material/optics/iridescence.rs src/render_helpers/shaders/material/iridescence.frag src/render_helpers/material/optics/mod.rs src/render_helpers/shaders/material/main.frag src/render_helpers/shaders/mod.rs niri-config/src/material/optics/mod.rs docs/materials/material-config.md docs/materials/render-pipeline.md
tasks done material-fad5de "IridescenceOptic, iridescence.frag specular hook, registry and docs"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): iridescence optic on the Fresnel glint"
```

---

### Task 3: Generalise the drift clock over its period

**Files:**
- Modify: `src/render_helpers/signal.rs` (`drift_buckets` → `buckets_on`, new `phase_on` and `next_boundary_on`, `drift` and `drift_next_boundary` delegate; tests)

**Interfaces:**
- Produces: `pub fn phase_on(period: Duration, hz: f64, now: Duration, seed: f32) -> f32` (radians, constant within a bucket, 0 when `hz <= 0`); `pub fn next_boundary_on(period: Duration, hz: f64, now: Duration) -> Option<Duration>` (first nanosecond of the next bucket; `None` when `hz <= 0`). `drift(hz, now, seed) == phase_on(DRIFT_PERIOD, hz, now, seed)` and `drift_next_boundary(hz, now) == next_boundary_on(DRIFT_PERIOD, hz, now)`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module of `src/render_helpers/signal.rs`, after `drift_boundary_is_strictly_future_and_advances_the_bucket`:

```rust
    #[test]
    fn drift_is_the_ten_second_case_of_the_general_clock() {
        for t in [0, 33, 66, 100, 9_999, 10_000, 12_345] {
            assert_eq!(drift(15., ms(t), 0.3), phase_on(DRIFT_PERIOD, 15., ms(t), 0.3));
            assert_eq!(
                drift_next_boundary(15., ms(t)),
                next_boundary_on(DRIFT_PERIOD, 15., ms(t))
            );
        }
    }

    #[test]
    fn a_ten_minute_period_buckets_at_the_rate_per_second() {
        let period = Duration::from_secs(600);
        // 4 Hz over 600 s is 2400 buckets of 250 ms, anchored to the clock.
        assert_eq!(next_boundary_on(period, 4., ms(0)), Some(ms(250)));
        assert_eq!(next_boundary_on(period, 4., ms(250)), Some(ms(500)));
        assert_eq!(next_boundary_on(period, 4., ms(251)), Some(ms(500)));
        assert_eq!(next_boundary_on(period, 4., ms(599_990)), Some(ms(600_000)));
        assert_eq!(next_boundary_on(period, 0., ms(100)), None);
        // The phase advances one 2400th of a turn per bucket and is
        // constant inside a bucket.
        assert_eq!(phase_on(period, 4., ms(0), 0.), 0.);
        assert_eq!(phase_on(period, 4., ms(249), 0.), 0.);
        let one_bucket = TAU / 2400.;
        assert!((phase_on(period, 4., ms(250), 0.) - one_bucket).abs() < 1e-6);
        assert_eq!(phase_on(period, 0., ms(250), 0.3), 0.);
        // Boundaries are strictly future and each one changes the phase.
        for hz in [1., 4., 30.] {
            let mut now = Duration::ZERO;
            for _ in 0..100 {
                let b = next_boundary_on(period, hz, now).unwrap();
                assert!(b > now, "{hz} Hz at {now:?}");
                assert_ne!(
                    phase_on(period, hz, b, 0.1),
                    phase_on(period, hz, now, 0.1),
                    "{hz} Hz at {now:?}"
                );
                now = b;
            }
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib signal::tests`
Expected: compile error, `phase_on` / `next_boundary_on` not found.

- [ ] **Step 3: Generalise the clock**

Replace `drift_buckets`, `drift`, and `drift_next_boundary` in `src/render_helpers/signal.rs` with:

```rust
/// Buckets per period for a rate: `hz` steps per second over the period,
/// rounded, at least one bucket. Config guarantees `hz == 0 || hz >= 1`
/// before the reduced-motion halving, so the 10 s drift has at least 5.
fn buckets_on(period: Duration, hz: f64) -> Option<u128> {
    if hz <= 0. {
        return None;
    }
    Some(((hz * period.as_secs_f64()).round() as u128).max(1))
}

/// Phase in radians of a bucketed clock, constant within each bucket.
/// Buckets divide `period` evenly and are anchored to period starts on the
/// absolute clock, in integer nanoseconds, so no rounding accumulates over
/// uptime and every window on an output shares the same boundaries. The
/// seed offsets the phase, not the boundary. Pinned to 0 when the rate is
/// 0 so a static value fingerprints to a constant.
pub fn phase_on(period: Duration, hz: f64, now: Duration, seed: f32) -> f32 {
    let Some(n) = buckets_on(period, hz) else {
        return 0.;
    };
    let period = period.as_nanos();
    let k = (now.as_nanos() % period) * n / period;
    let phase = (k as f32 / n as f32 + seed).fract();
    (phase * TAU).rem_euclid(TAU)
}

/// Next absolute-clock instant at which `phase_on` changes: the first
/// nanosecond of the next bucket, so it is strictly in the future and
/// `phase_on` evaluated there is already the next bucket's value (ceiling
/// division; a floor would land one nanosecond early and re-arm the same
/// deadline). The `as u64` cast is exact below 584 years of uptime.
pub fn next_boundary_on(period: Duration, hz: f64, now: Duration) -> Option<Duration> {
    let n = buckets_on(period, hz)?;
    let period = period.as_nanos();
    let start = now.as_nanos() - now.as_nanos() % period;
    let k = (now.as_nanos() - start) * n / period + 1;
    Some(Duration::from_nanos(
        (start + (period * k).div_ceil(n)) as u64,
    ))
}

/// Focus filament drift phase: `phase_on` over the 10 s `DRIFT_PERIOD`.
pub fn drift(hz: f64, now: Duration, seed: f32) -> f32 {
    phase_on(DRIFT_PERIOD, hz, now, seed)
}

/// Next instant `drift` changes: `next_boundary_on` over `DRIFT_PERIOD`.
pub fn drift_next_boundary(hz: f64, now: Duration) -> Option<Duration> {
    next_boundary_on(DRIFT_PERIOD, hz, now)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib signal::tests`
Expected: every test in the module PASSES, including the pre-existing drift tests unchanged.

- [ ] **Step 5: Commit**

```bash
tasks start material-af2778
git add src/render_helpers/signal.rs
tasks done material-af2778 "phase_on and next_boundary_on generalise the drift clock over its period; drift delegates"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "refactor(signal): generalise the drift clock over its period"
```

---

### Task 4: Aurora optic in niri-config

**Files:**
- Create: `niri-config/src/material/optics/aurora.rs`
- Modify: `niri-config/src/material/optics/mod.rs` (module export only)
- Modify: `niri-config/src/material/mod.rs` (`Glass`, `ResolvedGlass`, `Default`, `resolve`, `validate`)
- Modify: `niri-config/src/lib.rs` (re-export, tests)

**Interfaces:**
- Produces: `pub struct Aurora { pub amount: FloatOrInt<0, 1>, pub drift_hz: Option<FloatOrInt<0, 30>>, pub colors: Vec<Color> }` with `pub fn validate(&self) -> Result<(), String>`; `pub struct ResolvedAurora { pub amount: f64, pub drift_hz: f64, pub color_a: Color, pub color_b: Color }` (`Default`: 0, 4, `#3dffb0`, `#7a5cff`); `pub fn resolve(node: Option<&Aurora>) -> ResolvedAurora`; `pub fn params() -> Vec<ParamSpec>`; `ResolvedGlass.aurora: ResolvedAurora`.

- [ ] **Step 1: Write the failing tests**

Add to the material tests in `niri-config/src/lib.rs`:

```rust
    #[test]
    fn aurora_resolves_through_its_optic() {
        let written = do_parse(
            r##"material "sky" { glass { aurora 0.5 { drift-hz 2; color "#ff0000"; color "#0000ff"; }; }; }"##,
        );
        let aurora = written.materials[0].resolve().glass.aurora;
        assert_eq!(aurora.amount, 0.5);
        assert_eq!(aurora.drift_hz, 2.);
        assert_eq!(aurora.color_a, Color::from_rgba8_unpremul(0xff, 0, 0, 0xff));
        assert_eq!(aurora.color_b, Color::from_rgba8_unpremul(0, 0, 0xff, 0xff));

        let bare = do_parse(r##"material "sky" { glass { aurora 0.5; }; }"##);
        assert_eq!(
            bare.materials[0].resolve().glass.aurora,
            ResolvedAurora {
                amount: 0.5,
                ..Default::default()
            }
        );

        let omitted = do_parse(r##"material "sky" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.aurora,
            ResolvedAurora::default()
        );
        let d = ResolvedAurora::default();
        assert_eq!(d.amount, 0.);
        assert_eq!(d.drift_hz, 4.);
        assert_eq!(d.color_a, Color::from_rgba8_unpremul(0x3d, 0xff, 0xb0, 0xff));
        assert_eq!(d.color_b, Color::from_rgba8_unpremul(0x7a, 0x5c, 0xff, 0xff));
    }

    #[test]
    fn aurora_takes_zero_or_two_colors() {
        for colors in [
            r##"color "#ff0000";"##,
            r##"color "#ff0000"; color "#00ff00"; color "#0000ff";"##,
        ] {
            let err = do_parse_err(&format!(
                r##"material "sky" {{ glass {{ aurora 0.5 {{ {colors} }}; }}; }}"##
            ));
            assert!(err.contains("aurora: expected two color nodes"), "{err}");
        }
    }

    #[test]
    fn aurora_drift_hz_is_zero_or_at_least_one() {
        let err = do_parse_err(r##"material "sky" { glass { aurora 0.5 { drift-hz 0.5; }; }; }"##);
        assert!(err.contains("aurora drift-hz must be 0 or at least 1"), "{err}");
        for ok in ["0", "1", "7.5", "30"] {
            let parsed = do_parse(&format!(
                r##"material "sky" {{ glass {{ aurora 0.5 {{ drift-hz {ok}; }}; }}; }}"##
            ));
            assert_eq!(
                parsed.materials[0].resolve().glass.aurora.drift_hz,
                ok.parse::<f64>().unwrap()
            );
        }
        let err = do_parse_err(r##"material "sky" { glass { aurora 0.5 { drift-hz 31; }; }; }"##);
        assert!(err.contains("must be"), "{err}");
    }

    #[test]
    fn glass_aurora_rejects_amounts_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"sky\" {{ glass {{ aurora {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }
```

Import `ResolvedAurora` in the test module the way `ResolvedNoise` is imported; `Color` is already in scope through `crate::appearance::*` (check the module's imports and add `use crate::appearance::Color;` if not).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config aurora`
Expected: compile error, `ResolvedAurora` not found.

- [ ] **Step 3: Write the optic module**

`niri-config/src/material/optics/aurora.rs`:

```rust
//! `aurora <amount> { drift-hz; color; color }`: stage 6, a slow colour
//! field inside the glass. Neutral at amount 0.

use crate::appearance::Color;
use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// `aurora <amount> { drift-hz <hz>; color <a>; color <b>; }`.
///
/// A block node because it carries two colours. `color` appears zero times
/// (both defaults) or exactly twice; `validate` refuses any other count.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct Aurora {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(child, unwrap(argument))]
    pub drift_hz: Option<FloatOrInt<0, 30>>,
    #[knuffel(children(name = "color"))]
    pub colors: Vec<Color>,
}

impl Aurora {
    /// The two rules the scalar bounds cannot express: the colour count,
    /// and the `ring-drift-hz` rule for the field's clock.
    pub fn validate(&self) -> Result<(), String> {
        if !(self.colors.is_empty() || self.colors.len() == 2) {
            return Err(String::from("aurora: expected two color nodes"));
        }
        // The clock divides a 600 s period into `hz * 600` buckets; a rate
        // below 1 Hz (before the reduced-motion halving) is refused rather
        // than clamped, as `ring-drift-hz` is.
        if self.drift_hz.is_some_and(|hz| hz.0 > 0. && hz.0 < 1.) {
            return Err(String::from("aurora drift-hz must be 0 or at least 1"));
        }
        Ok(())
    }
}

/// Final aurora state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedAurora {
    /// Strength of the field's light; 0 adds nothing.
    pub amount: f64,
    /// Bucket rate of the field's clock in Hz; 0 pins the field.
    pub drift_hz: f64,
    /// The field mixes from `color_a` (noise 0) to `color_b` (noise 1).
    pub color_a: Color,
    pub color_b: Color,
}

impl Default for ResolvedAurora {
    fn default() -> Self {
        Self {
            amount: 0.,
            drift_hz: 4.,
            color_a: Color::from_rgba8_unpremul(0x3d, 0xff, 0xb0, 0xff),
            color_b: Color::from_rgba8_unpremul(0x7a, 0x5c, 0xff, 0xff),
        }
    }
}

pub fn resolve(node: Option<&Aurora>) -> ResolvedAurora {
    let d = ResolvedAurora::default();
    let Some(node) = node else {
        return d;
    };
    let (color_a, color_b) = match node.colors.as_slice() {
        [] => (d.color_a, d.color_b),
        [a, b] => (*a, *b),
        _ => unreachable!("Aurora::validate runs before resolution"),
    };
    ResolvedAurora {
        amount: node.amount.0,
        drift_hz: node.drift_hz.map_or(d.drift_hz, |x| x.0),
        color_a,
        color_b,
    }
}

pub fn params() -> Vec<ParamSpec> {
    let d = ResolvedAurora::default();
    vec![
        ParamSpec {
            node: "aurora",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.amount, "—"),
            write: |v| format!("aurora {v}"),
            read: Some(|g| Some(g.aurora.amount)),
        },
        ParamSpec {
            node: "aurora drift-hz",
            kind: ParamKind::float::<FloatOrInt<0, 30>>(d.drift_hz, "Hz"),
            write: |v| format!("aurora 0.5 {{ drift-hz {v}; }}"),
            read: Some(|g| Some(g.aurora.drift_hz)),
        },
        // Two rows with the same node: the first `color` is the field's
        // start and the second its end. The prose under the table says so.
        ParamSpec {
            node: "aurora color",
            kind: ParamKind::Color { default: d.color_a },
            write: |v| format!("aurora 0.5 {{ color \"{v}\"; color \"#000000\"; }}"),
            read: None,
        },
        ParamSpec {
            node: "aurora color",
            kind: ParamKind::Color { default: d.color_b },
            write: |v| format!("aurora 0.5 {{ color \"#000000\"; color \"{v}\"; }}"),
            read: None,
        },
    ]
}
```

- [ ] **Step 4: Register the module, the field, and the validation (not yet in `ORDER`)**

In `niri-config/src/material/optics/mod.rs` add `pub mod aurora;` first (alphabetical). Leave `ORDER` and `params()` unchanged (Task 5).

In `niri-config/src/material/mod.rs`:

- `Glass`: after the `iridescence` field add
  ```rust
      #[knuffel(child)]
      pub aurora: Option<optics::aurora::Aurora>,
  ```
- `ResolvedGlass`: after `iridescence` add `pub aurora: optics::aurora::ResolvedAurora,`
- `impl Default for ResolvedGlass`: after `iridescence` add `aurora: optics::aurora::ResolvedAurora::default(),`
- `Material::resolve`: after `iridescence` add `aurora: optics::aurora::resolve(g.aurora.as_ref()),`
- `Material::validate`: before `let resolved = self.resolve();` add
  ```rust
          // Optic rules run before resolution: `resolve` relies on them.
          if let Some(aurora) = &self.glass.aurora {
              aurora.validate()?;
          }
  ```
  and update the method's doc comment to say it also runs each optic's `validate`.

`Glass` derives `Default` and `PartialEq` only; `Aurora` holds a `Vec`, so `Glass` stays non-`Copy` as it is today.

In `niri-config/src/lib.rs` add the re-export before the iridescence one:

```rust
pub use crate::material::optics::aurora::{Aurora, ResolvedAurora};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config aurora`
Expected: four tests PASS. Run: `python3 tools/tt test-fast -- cargo test -p niri-config material`
Expected: PASS (existing material tests, table unchanged, parser test unchanged).

- [ ] **Step 6: Commit**

```bash
tasks start material-99eda2
git add niri-config/src/material/optics/aurora.rs niri-config/src/material/optics/mod.rs niri-config/src/material/mod.rs niri-config/src/lib.rs
tasks done material-99eda2 "Aurora block node with drift-hz and two colours, validation, resolved defaults, params, and tests"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): aurora optic in niri-config"
```

---

### Task 5: Aurora optic in niri, its GLSL, the tile deadline test, and its docs

**Files:**
- Create: `src/render_helpers/material/optics/aurora.rs`
- Create: `src/render_helpers/shaders/material/aurora.frag`
- Modify: `src/render_helpers/material/optics/mod.rs` (`pub mod`, `OPTICS`)
- Modify: `src/render_helpers/shaders/material/main.frag` (emissive hook call)
- Modify: `src/render_helpers/shaders/mod.rs` (assembly test)
- Modify: `niri-config/src/material/optics/mod.rs` (`ORDER`, `params()`)
- Modify: `src/layout/tile.rs` (test fixture and the deadline test)
- Modify: `docs/materials/material-config.md`, `docs/materials/render-pipeline.md`

**Interfaces:**
- Consumes: `ResolvedGlass.aurora` (Task 4); `phase_on`, `next_boundary_on` (Task 3); `drift_rate`, `color_linear` from `src/render_helpers/signal.rs`.
- Produces: `AuroraOptic` with uniforms `mat_aurora` (`_1f`), `mat_aurora_phase` (`_1f`, radians), `mat_aurora_color_a` (`_3f`, linear), `mat_aurora_color_b` (`_3f`, linear); `pub const AURORA_PERIOD: Duration = 600 s`; GLSL `vec3 aurora_emissive(vec2 p, vec3 n, vec3 att, float innerDist)`; final `ORDER == ["iridescence", "aurora", "saturation", "noise"]`.

- [ ] **Step 1: Write the failing tests**

`src/render_helpers/material/optics/aurora.rs` tests (write the file with a stub `pub struct AuroraOptic;` and these tests):

```rust
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use niri_config::signal::SignalMotionPolicy;
    use niri_config::{Blur, Color, ResolvedAurora, ResolvedGlass};
    use smithay::backend::renderer::gles::UniformValue;

    use super::*;
    use crate::render_helpers::signal::phase_on;

    fn frame(blur: &Blur, now: Duration, motion: SignalMotionPolicy, animations_off: bool) -> OpticFrame<'_> {
        OpticFrame {
            now,
            motion,
            animations_off,
            backdrop_blur: false,
            blur,
            seed: 0.,
        }
    }

    fn sky(amount: f64, drift_hz: f64) -> ResolvedGlass {
        ResolvedGlass {
            aurora: ResolvedAurora {
                amount,
                drift_hz,
                color_a: Color::from_rgba8_unpremul(0xff, 0, 0, 0xff),
                color_b: Color::from_rgba8_unpremul(0, 0, 0xff, 0xff),
            },
            ..Default::default()
        }
    }

    fn f1(v: &UniformValue) -> f32 {
        match v {
            UniformValue::_1f(v) => *v,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn values_carry_amount_phase_and_linear_colours() {
        let blur = Blur::default();
        let ctx = frame(&blur, Duration::from_millis(250), SignalMotionPolicy::Full, false);
        let values = AuroraOptic::values(&sky(0.5, 4.), &ctx);
        let names: Vec<_> = values.iter().map(|u| u.name.as_ref()).collect();
        assert_eq!(
            names,
            ["mat_aurora", "mat_aurora_phase", "mat_aurora_color_a", "mat_aurora_color_b"]
        );
        assert_eq!(f1(&values[0].value), 0.5);
        assert_eq!(
            f1(&values[1].value),
            phase_on(AURORA_PERIOD, 4., Duration::from_millis(250), 0.)
        );
        assert_eq!(values[2].value, UniformValue::_3f(1., 0., 0.));
        assert_eq!(values[3].value, UniformValue::_3f(0., 0., 1.));
    }

    #[test]
    fn the_phase_follows_the_motion_policy_and_the_animation_switch() {
        let blur = Blur::default();
        let now = Duration::from_millis(250);
        let full = AuroraOptic::values(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Full, false));
        let reduced = AuroraOptic::values(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Reduced, false));
        let off = AuroraOptic::values(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Off, false));
        let anim_off = AuroraOptic::values(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Full, true));
        assert_eq!(f1(&full[1].value), phase_on(AURORA_PERIOD, 4., now, 0.));
        assert_eq!(f1(&reduced[1].value), phase_on(AURORA_PERIOD, 2., now, 0.));
        assert_eq!(f1(&off[1].value), 0.);
        assert_eq!(f1(&anim_off[1].value), 0.);
    }

    #[test]
    fn next_change_is_the_next_bucket_boundary_only_while_the_field_is_lit_and_moving() {
        let blur = Blur::default();
        let now = Duration::ZERO;
        let full = frame(&blur, now, SignalMotionPolicy::Full, false);
        assert_eq!(AuroraOptic::next_change(&sky(0.5, 4.), &full), Some(Duration::from_millis(250)));
        assert_eq!(
            AuroraOptic::next_change(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Reduced, false)),
            Some(Duration::from_millis(500))
        );
        assert_eq!(AuroraOptic::next_change(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Off, false)), None);
        assert_eq!(AuroraOptic::next_change(&sky(0.5, 4.), &frame(&blur, now, SignalMotionPolicy::Full, true)), None);
        assert_eq!(AuroraOptic::next_change(&sky(0.5, 0.), &full), None);
        // Amount 0 is the neutral: no redraw, and a pinned phase so the
        // fingerprint stays constant.
        assert_eq!(AuroraOptic::next_change(&sky(0., 4.), &full), None);
        let neutral = AuroraOptic::values(&sky(0., 4.), &frame(&blur, Duration::from_millis(250), SignalMotionPolicy::Full, false));
        assert_eq!(f1(&neutral[1].value), 0.);
    }
}
```

In `src/render_helpers/shaders/mod.rs` extend the assembly test: the marker list becomes `iridescence`, `aurora`, `saturation`, `noise`, `main`, and add `assert!(source.contains("aurora_emissive(p, n, att, innerDist)"));`.

In `src/layout/tile.rs` tests, generalise the fixture and add the deadline test:

```rust
    /// A tile carrying the material "frost" with the given glass, whose only
    /// response selects `focus`. Everything else is stock: `motion "full"`,
    /// animations on.
    fn material_tile(
        glass: niri_config::ResolvedGlass,
        focus: niri_config::FocusResponse,
        clock: Clock,
    ) -> Tile<TestWindow> {
        let material = niri_config::ResolvedMaterial {
            name: String::from("frost"),
            glass,
            responses: vec![(
                String::from("default"),
                niri_config::ResolvedResponse {
                    focus,
                    ..Default::default()
                },
            )],
        };
        let options = Options {
            materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
            ..Default::default()
        };
        let mut params = TestWindowParams::new(1);
        params.rules = Some(ResolvedWindowRules {
            material: Some(MaterialRef {
                name: String::from("frost"),
                response: None,
            }),
            ..Default::default()
        });
        Tile::new(
            TestWindow::new(params),
            Size::from((1280., 720.)),
            1.,
            clock,
            Rc::new(options),
        )
    }

    fn focus_tile(focus: niri_config::FocusResponse, clock: Clock) -> Tile<TestWindow> {
        material_tile(niri_config::ResolvedGlass::default(), focus, clock)
    }

    #[test]
    fn an_unfocused_signal_free_aurora_tile_reports_its_next_bucket() {
        // The scheduling gate of the optics design §3: no focus filament, no
        // signal, so no signal frame cache, and still a deadline from the
        // optic while the slab band is in view.
        let clock = Clock::with_time(Duration::ZERO);
        let lit = niri_config::ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 4.,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut tile = material_tile(lit, niri_config::FocusResponse::None, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(false, true, view);
        assert!(!tile.active);
        assert!(tile.signal_frame_cache.borrow().is_none());
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            Some(Duration::from_millis(250))
        );
        let far = Point::from((10_000., 10_000.));
        assert_eq!(tile.tick_deadline(far, view, Duration::ZERO), None);

        let pinned = niri_config::ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 0.,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut tile = material_tile(pinned, niri_config::FocusResponse::None, clock);
        tile.update_render_elements(false, true, view);
        assert_eq!(tile.tick_deadline(Point::default(), view, Duration::ZERO), None);
    }
```

`Clock` derives `Clone`, so the same clock serves both tiles.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib optics::aurora`
Expected: compile error. Run: `python3 tools/tt test-fast -- cargo test -p niri --lib aurora_tile`
Expected: compile error (`material_tile` undefined) or FAIL with `None` for the deadline.

- [ ] **Step 3: Write the optic, the GLSL, and the registrations**

`src/render_helpers/material/optics/aurora.rs` (above the tests):

```rust
//! `aurora`: stage 6, neutral at amount 0. Animated: the field's phase
//! steps at `drift-hz` buckets per second on the generalised drift clock
//! over a 600 s period, so its `next_change` is the next bucket boundary.

use std::time::Duration;

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};
use crate::render_helpers::signal::{color_linear, drift_rate, next_boundary_on, phase_on};

/// One lap of the field's loop. Long enough that the repeat is never
/// noticed; `drift-hz` steps per second give `hz × 600` buckets.
pub const AURORA_PERIOD: Duration = Duration::from_secs(600);

pub struct AuroraOptic;

impl AuroraOptic {
    /// Effective bucket rate: 0 while the field is unlit (the neutral must
    /// fingerprint to a constant and schedule nothing), else the configured
    /// rate under the motion policy and the animation switch.
    fn rate(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> f64 {
        if glass.aurora.amount <= 0. {
            return 0.;
        }
        drift_rate(glass.aurora.drift_hz, ctx.motion, ctx.animations_off)
    }
}

impl Optic for AuroraOptic {
    const NAME: &'static str = "aurora";
    const GLSL: &'static str = include_str!("../../shaders/material/aurora.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_aurora", UniformType::_1f),
        ("mat_aurora_phase", UniformType::_1f),
        ("mat_aurora_color_a", UniformType::_3f),
        ("mat_aurora_color_b", UniformType::_3f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        let aurora = &glass.aurora;
        vec![
            Uniform::new("mat_aurora", aurora.amount as f32),
            Uniform::new(
                "mat_aurora_phase",
                phase_on(AURORA_PERIOD, Self::rate(glass, ctx), ctx.now, ctx.seed),
            ),
            Uniform::new("mat_aurora_color_a", color_linear(aurora.color_a)),
            Uniform::new("mat_aurora_color_b", color_linear(aurora.color_b)),
        ]
    }

    fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration> {
        next_boundary_on(AURORA_PERIOD, Self::rate(glass, ctx), ctx.now)
    }
}
```

`src/render_helpers/shaders/material/aurora.frag`:

```glsl
// Optic: aurora (render-pipeline.md stage 6). Neutral at amount 0.
// Uses snoise, snoiseJelly, and mat_jelly_seed from the prelude.
uniform float mat_aurora;
uniform float mat_aurora_phase;
uniform vec3 mat_aurora_color_a;
uniform vec3 mat_aurora_color_b;

// One noise unit is 250 px at this scale.
const float AURORA_SCALE = 0.004;
// The lookup point traces a circle in noise space over one lap of the
// phase, so the loop closes without a seam. The radius sets how far the
// field travels per lap: 2.0 is about 12.6 noise units per 600 s lap,
// about 5 px per second.
const float AURORA_LOOP_RADIUS = 2.0;

vec3 aurora_emissive(vec2 p, vec3 n, vec3 att, float innerDist) {
    if (mat_aurora <= 0.0)
        return vec3(0.0);
    vec3 q = vec3(p * AURORA_SCALE, 0.0) + mat_jelly_seed
           + vec3(cos(mat_aurora_phase), 0.0, sin(mat_aurora_phase)) * AURORA_LOOP_RADIUS;
    // Two octaves for the field, one coarser octave for its brightness.
    float field = 0.5 + 0.5 * snoiseJelly(q);
    float coarse = 0.5 + 0.5 * snoise(q * 0.5);
    vec3 color = mix(mat_aurora_color_a, mat_aurora_color_b, field);
    // att ^ 0.2 is the ring's factor: the field sits inside the glass
    // rather than on it.
    return mat_aurora * 0.35 * color * (0.5 + 0.5 * coarse) * pow(att, vec3(0.2));
}
```

`src/render_helpers/material/optics/mod.rs`: add `pub mod aurora;` first and make the registry

```rust
pub static OPTICS: &[OpticEntry] = &[
    OpticEntry::of::<iridescence::IridescenceOptic>(),
    OpticEntry::of::<aurora::AuroraOptic>(),
    OpticEntry::of::<saturation::SaturationOptic>(),
    OpticEntry::of::<noise::NoiseOptic>(),
];
```

`niri-config/src/material/optics/mod.rs`:

```rust
pub const ORDER: &[&str] = &["iridescence", "aurora", "saturation", "noise"];

pub fn params() -> Vec<ParamSpec> {
    let mut specs = iridescence::params();
    specs.extend(aurora::params());
    specs.extend(saturation::params());
    specs.extend(noise::params());
    specs
}
```

`src/render_helpers/shaders/material/main.frag`: after the ring-of-light block (the closing brace of `if ((showAccent || showFocus) && slabChamfer > 0.0) { ... }`) and before `float diag = (p.x + p.y) / ...` insert

```glsl
        // Emissive hooks, in OPTICS order (render-pipeline.md stage 6).
        emissive += aurora_emissive(p, n, att, innerDist);
```

- [ ] **Step 4: Validate the shader offline**

Run:

```bash
{ printf '#version 100\n'; cat src/render_helpers/shaders/material/prelude.frag src/render_helpers/shaders/material/iridescence.frag src/render_helpers/shaders/material/aurora.frag src/render_helpers/shaders/material/saturation.frag src/render_helpers/shaders/material/noise.frag src/render_helpers/shaders/material/main.frag; } > target/material.frag
glslangValidator -S frag target/material.frag
```

Expected: exit 0.

- [ ] **Step 5: Regenerate the table and write the docs**

Run: `MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs`
Expected: PASS; the table gains four rows after `iridescence`: `` `aurora` `` float 0 0–1 —; `` `aurora` `drift-hz` `` float 4 0–30 Hz; two `` `aurora` `color` `` rows with defaults `` `#3dffb0` `` and `` `#7a5cff` ``.

In `docs/materials/material-config.md`, under `## Optics`, insert after `### iridescence`:

```markdown
### aurora

Stage 6. `aurora <amount> { drift-hz <hz>; color <a>; color <b>; }` adds a
slow colour field inside the glass: two octaves of simplex noise on the
element position (one noise unit is 250 px), offset by the window seed,
mix the two colours, and a coarser octave sets the brightness; the light
is weighted by `att ^ 0.2` like the ring, so it sits inside the slab. The
first `color` node is the field's start (noise 0), the second its end
(noise 1); both may be omitted, and any other count is the error
`aurora: expected two color nodes`. Its explicit neutral is amount 0;
nothing inherits.

`drift-hz` is the field's clock, on the same rule as `ring-drift-hz`: `0`
pins the field, otherwise at least 1, and the error is `aurora drift-hz
must be 0 or at least 1`. The field's lookup point traces a small circle
in noise space once per 600 s, so the loop closes seamlessly; the clock
steps the phase `drift-hz` times per second in buckets anchored to the
absolute clock, and a lit, visible aurora window redraws at that rate
whether or not it is focused or carries a signal. `signal { motion
"reduced" }` halves the rate; `motion "off"` and `animations { off }` pin
the field at phase 0.
```

Also update the `## Optics` intro line "Contributors: see `adding-an-optic.md`." to precede a sentence: "Optics are listed in render order: `iridescence`, `aurora`, `saturation`, `noise`."

In `docs/materials/render-pipeline.md`:

- stage 6 row: append to the description `Then the \`aurora\` optic adds its colour field, weighted by the same \`att ^ 0.2\`.` and to the parameters cell `` , `aurora`, `aurora drift-hz`, `aurora color` (optic `aurora`; neutral 0) ``.
- §5 table: add the row `` | `aurora`, `drift-hz`, `color` × 2 | (pending, prism-763054) | 6 | `` after the `iridescence` row.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri --lib optics::aurora`
Expected: three tests PASS. Run: `python3 tools/tt test-fast -- cargo test -p niri --lib aurora_tile`
Expected: PASS. Run: `python3 tools/tt test-fast -- cargo test -p niri --lib shaders::tests`
Expected: PASS. Run: `python3 tools/tt test-fast -- cargo test -p niri-config material_parameter`
Expected: PASS. Then `just test` for the whole suite.
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
tasks start material-589b0d
git add src/render_helpers/material/optics/aurora.rs src/render_helpers/shaders/material/aurora.frag src/render_helpers/material/optics/mod.rs src/render_helpers/shaders/material/main.frag src/render_helpers/shaders/mod.rs niri-config/src/material/optics/mod.rs src/layout/tile.rs docs/materials/material-config.md docs/materials/render-pipeline.md
tasks done material-589b0d "AuroraOptic on the 600 s clock, aurora.frag emissive hook, registry, tile deadline test, docs"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): aurora optic, a slow colour field inside the glass"
```

---

### Task 6: Presets and their install plumbing

**Files:**
- Create: `resources/materials/rainbow.kdl`, `resources/materials/aurora.kdl`
- Modify: `packaging/arch/PKGBUILD` (`package()`), `Cargo.toml` (rpm and deb asset lists)
- Modify: `niri-config/src/lib.rs` (preset test), `docs/materials/material-config.md` (`## Presets`)

**Interfaces:**
- Consumes: the `iridescence` and `aurora` nodes (Tasks 1–5).
- Produces: `/usr/share/niri/materials/<name>.kdl`, each defining one material named after its file. The ice task (`material-bb3fe5`) adds `ice.kdl` to the same lists.

- [ ] **Step 1: Write the failing test**

Add to the material tests in `niri-config/src/lib.rs`:

```rust
    #[test]
    fn every_preset_parses_alone_and_through_an_absolute_include() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../resources/materials")
            .canonicalize()
            .unwrap();
        let mut seen = Vec::new();
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "kdl") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
            let alone = do_parse(&fs::read_to_string(&path).unwrap());
            assert_eq!(alone.materials.len(), 1, "{stem}");
            assert_eq!(alone.materials[0].name, stem);
            // The documented use: an absolute `include` of the installed file.
            let included = parse_files(&[(
                "config.kdl",
                &format!(
                    "include \"{}\"\nwindow-rule {{ match app-id=\"^x$\"; material \"{stem}\"; }}\n",
                    path.display()
                ),
            )])
            .unwrap();
            assert_eq!(included.materials[0].name, stem);
            seen.push(stem);
        }
        seen.sort();
        assert!(seen.contains(&String::from("aurora")), "{seen:?}");
        assert!(seen.contains(&String::from("rainbow")), "{seen:?}");
    }
```

`Path` and `fs` are used elsewhere in the test module; add `use std::path::Path;` if the module lacks it.

- [ ] **Step 2: Run the test to verify it fails**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config every_preset`
Expected: FAIL, `read_dir` on a missing directory.

- [ ] **Step 3: Write the presets**

`resources/materials/rainbow.kdl`:

```kdl
// Rainbow glass: dispersion in the refracted image (chromatic-aberration)
// and a thin-film hue on the edge light (iridescence).
//
// include "/usr/share/niri/materials/rainbow.kdl"
// window-rule { match app-id="^kitty$"; material "rainbow"; }
material "rainbow" {
    glass {
        ior 1.7
        chromatic-aberration 0.5
        iridescence 0.8
    }
}
```

`resources/materials/aurora.kdl`:

```kdl
// Aurora glass: a slow green-to-violet colour field inside a cool slab,
// stepping four times a second on a ten-minute loop.
//
// include "/usr/share/niri/materials/aurora.kdl"
// window-rule { match app-id="^kitty$"; material "aurora"; }
material "aurora" {
    glass {
        attenuation-color "#cfe0ff"
        aurora 0.5 {
            drift-hz 4
            color "#3dffb0"
            color "#7a5cff"
        }
    }
}
```

Check that KDL `//` line comments parse (the noise smoke configs use none; the default config `resources/default-config.kdl` uses `//` comments throughout, so they do).

- [ ] **Step 4: Install plumbing**

`packaging/arch/PKGBUILD`, in `package()` after the `default-config.kdl` install line:

```bash
  install -vDm 644 resources/materials/*.kdl \
    -t "$pkgdir/usr/share/niri/materials/"
```

`Cargo.toml`, `[package.metadata.generate-rpm]` assets, after the `niri-shutdown.target` entry:

```toml
    { source = "resources/materials/aurora.kdl", dest = "/usr/share/niri/materials/", mode = "644" },
    { source = "resources/materials/rainbow.kdl", dest = "/usr/share/niri/materials/", mode = "644" },
```

`[package.metadata.deb]` assets, after the `niri-shutdown.target` entry:

```toml
    ["resources/materials/aurora.kdl", "/usr/share/niri/materials/", "644"],
    ["resources/materials/rainbow.kdl", "/usr/share/niri/materials/", "644"],
```

- [ ] **Step 5: Document the presets**

In `docs/materials/material-config.md`, after the `## Optics` section (before `## Signal responses`), add:

```markdown
## Presets

`resources/materials/` holds one file per preset material, named after the
material it defines, installed to `/usr/share/niri/materials/`. `include`
accepts an absolute path, so a preset is used with

```kdl
include "/usr/share/niri/materials/aurora.kdl"

window-rule {
    match app-id="^kitty$"
    material "aurora"
}
```

A preset is one material; the focus split is Prism's, as today. `rainbow`
pairs `chromatic-aberration` with `iridescence`; `aurora` lights a cool slab
with the `aurora` optic at `drift-hz 4`. The evidence docs named in
`../specs/2026-09-10-material-optics-design.md` record how each preset's
values were tuned.
```

(Nest the inner fence with four backticks or indent it, whichever the file already does for KDL inside Markdown; the file uses plain triple fences at top level, so write the inner block with four-backtick outer fences.)

- [ ] **Step 6: Run the tests to verify they pass**

Run: `python3 tools/tt test-fast -- cargo test -p niri-config every_preset`
Expected: PASS. Run: `just check`
Expected: PASS. `package-pin --check` only verifies that `pkgver`, the `#commit=` source pin, and `NIRI_BUILD_COMMIT` agree with each other; the pin still names `31f7fbe2`, which has no `resources/materials/`, so `makepkg` would fail on the new install line until Task 9 repins on the merged, pushed head.

- [ ] **Step 7: Commit**

```bash
tasks start material-b6a1be
git add resources/materials/rainbow.kdl resources/materials/aurora.kdl packaging/arch/PKGBUILD Cargo.toml niri-config/src/lib.rs docs/materials/material-config.md
tasks done material-b6a1be "rainbow.kdl and aurora.kdl presets, installed to /usr/share/niri/materials/ by PKGBUILD and the rpm/deb asset lists; preset parse test"
tasks note material-bb3fe5 "resources/materials/ and its install plumbing landed with material-f0fc7b; ice adds ice.kdl to PKGBUILD package() and both Cargo.toml asset lists"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): rainbow and aurora presets under resources/materials"
```

---

### Task 7: The optic smoke library, the iridescence smoke, and its evidence

**Files:**
- Create: `docs/materials/scripts/glass-optic-smoke-lib.sh`
- Create: `docs/materials/scripts/glass-iridescence-smoke.sh`
- Modify: `docs/materials/scripts/glass-parameter-sweep.sh` (allowlist)
- Create: `docs/materials/<run-date>-material-iridescence-evidence.md`
- Modify: `resources/materials/rainbow.kdl` if tuning moves a value

**Interfaces:**
- Produces (lib, sourced): `build_binaries` (sets `NIRI`, `NIRI_TRACY`), `write_config <path>` (reads `GLASS_EXTRA`, `RESPONSE_EXTRA`, `TOP_EXTRA`), `start_nested <bin> <kdl>`, `stop_nested`, `spawn_probe <bin> <sh -c body>`, `steal_focus <bin>`, `probe_rect` (sets `PX PY PW PH`), `shot <bin> <name>`, `roi <name> <geometry> <suffix>`, `face_roi`, `chamfer_roi`, `ae`/`rmse`/`sd`/`oklab_ab` (result in `METRIC`), `assert_zero/positive/greater/less/about`, `tools_ready`, `reserve_tracy_port`, `trace_run <name> <sh -c body> <steal 0|1>`, `count_last20 <name>`, `gpu_median_ns <trace>`, `median3 <file>`, `finish` (hashes, log scan, PASS line). `IDLE` and `TICK` are the two probe bodies.

- [ ] **Step 1: Write the library**

`docs/materials/scripts/glass-optic-smoke-lib.sh`:

```bash
#!/usr/bin/env bash
# glass-optic-smoke-lib.sh: the helpers the per-optic smokes share
# (glass-iridescence-smoke.sh, glass-aurora-smoke.sh). Sourced, never run.
#
# A smoke nests niri under a headless Weston host, opens one transparent
# kitty (app-id gos-probe) over a pinned glass material on a flat warm
# backdrop, and compares decoded pixels of screen captures. Redraw counts
# and GPU cost come from Tracy: the `profile-with-tracy` build is captured
# for 30 s, `Niri::redraw` zones are counted in the final 20 s, and the
# `MaterialRenderElement::draw` GPU zone's median between 20 s and 28 s is
# the frame cost, following material-signals-smoke.sh.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT (holds the retained Tracy
# 0.13.1 tools under material-roughness-b220152d/tools).
# Requires: weston, kitty, swaybg, jq, rg, flock, ss, ImageMagick 7 with
# Oklab, cargo.
set -eu
OUT=${OUT:?artifact directory}
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?evidence root with the retained Tracy tools}
# A run owns its directory: traces are exported unconditionally and the
# GPU medians files accumulate one line per round, so a rerun into a used
# directory would mix measurements. Refuse anything but a fresh one.
if [ -e "$OUT" ] && [ -n "$(ls -A "$OUT")" ]; then
    echo "FAIL: OUT must be a fresh directory, $OUT is not empty" >&2; exit 1
fi
mkdir -p "$OUT"
ROOT=$(git rev-parse --show-toplevel)
# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/gos.XXXXXX")
RUN=$(basename "$RT")
HOST=$RUN-host; UNIT=$RUN-weston; NIRI_PID=; CAP_PID=
fail() { echo "FAIL: $*" >&2; exit 1; }
cleanup() {
    local rc=$?
    if [ -n "$CAP_PID" ]; then kill "$CAP_PID" 2>/dev/null || true; fi
    if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; fi
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -rf "$RT"
    if [ -S "$XDG_RUNTIME_DIR/$HOST" ]; then echo "WARN: weston socket $HOST still present" >&2; fi
    exit "$rc"
}
trap cleanup EXIT

# --- binaries ---------------------------------------------------------------
# Two builds: the release build for captures and the Tracy build for counts
# and cost, snapshotted under $OUT so a concurrent build cannot replace the
# executable mid-run. The target dir is shared across worktrees; toggling the
# feature rebuilds the niri crate.
build_binaries() {
    local target
    target=$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
    (cd "$ROOT" && cargo build --release)
    cp "$target/release/niri" "$OUT/niri"
    (cd "$ROOT" && cargo build --release --features profile-with-tracy)
    cp "$target/release/niri" "$OUT/niri-tracy"
    NIRI=$OUT/niri; NIRI_TRACY=$OUT/niri-tracy
    sha256sum "$NIRI" "$NIRI_TRACY" > "$OUT/binaries.sha256"
    "$NIRI" --version > "$OUT/impl.version"
    git -C "$ROOT" rev-parse HEAD > "$OUT/source.commit"
}

# --- configs ----------------------------------------------------------------
WALL=$OUT/warm-mid.png
magick -size 1280x720 xc:'rgb(140,115,90)' "$WALL"
GLASS_EXTRA=; RESPONSE_EXTRA=; TOP_EXTRA=
IDLE='printf "\033[?25l"; exec sleep 600'
TICK='printf "\033[?25l"; while :; do date +%s%N; sleep 0.1; done'
# The material is pinned rather than taken from a generated prism.kdl, which
# drifts. `animations { off }` is deliberately absent: it pins every optic
# clock through OpticFrame, which the aurora smoke measures. The response
# lights no filament and no accent so the probe is signal-free; two windows
# at proportion 0.4 both fit in view, so a second window can take focus
# without scrolling the probe.
#
# GLASS_EXTRA is newline-separated glass lines. A line whose node name
# matches a baseline line replaces it, as the sweep's emit_block does: KDL
# rejects a duplicate single node (`ior 1.5` then `ior 1.7` fails
# validation), so a preset body cannot simply be appended.
GLASS_BASELINE=(
    "ior 1.5"
    "thickness 20"
    "attenuation-color \"#dfe8ff\""
    "attenuation-distance 60"
    "chromatic-aberration 0"
    "distortion 0 scale=0.5"
    "anisotropic-blur 0"
    "roughness 0"
    "backdrop-blur false"
    "jelly-flex 0"
    "jelly-ripple 0"
    "bevel 12"
    "offset-x 6"
    "offset-y 6"
)
emit_glass() {   # the baseline with GLASS_EXTRA's nodes replaced, then GLASS_EXTRA
    local line key extra_keys=" "
    while IFS= read -r line; do
        line=${line#"${line%%[! ]*}"}
        [ -n "$line" ] && extra_keys+="${line%% *} "
    done <<< "$GLASS_EXTRA"
    for line in "${GLASS_BASELINE[@]}"; do
        key=${line%% *}
        case $extra_keys in *" $key "*) continue ;; esac
        printf '        %s\n' "$line"
    done
    while IFS= read -r line; do
        line=${line#"${line%%[! ]*}"}
        [ -n "$line" ] && printf '        %s\n' "$line"
    done <<< "$GLASS_EXTRA"
    return 0
}
write_config() {   # $1 path; reads GLASS_EXTRA, RESPONSE_EXTRA, TOP_EXTRA
    local glass; glass=$(emit_glass)
    cat > "$1" <<KDL
prefer-no-csd
layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.4; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
$TOP_EXTRA
material "gos-probe" {
    glass {
$glass
    }
    response "default" {
        focus "none"
        accent "none"
        ring-drift-hz 0
        $RESPONSE_EXTRA
    }
}
window-rule {
    match app-id="^gos-probe$"
    material "gos-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
KDL
}

# --- nested host ------------------------------------------------------------
start_nested() {   # $1 niri binary, $2 config
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$HOST" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || fail "no Weston socket"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    "$1" validate -c "$2" || fail "config $2 does not validate with $1"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$1" -c "$2" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1) || fail "no niri socket"
    export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
msg() { "$1" msg "${@:2}"; }
windows_with() { msg "$1" -j windows | jq -r --arg id "$2" '[.[] | select(.app_id==$id)] | length'; }
spawn_probe() {   # $1 binary, $2 body for sh -c
    msg "$1" action spawn -- kitty --config NONE --class gos-probe -o background_opacity=0 \
        -o cursor_blink_interval=0 sh -c "$2"
    for _ in $(seq 100); do [ "$(windows_with "$1" gos-probe)" -ge 1 ] && break; sleep 0.1; done
    [ "$(windows_with "$1" gos-probe)" -eq 1 ] || fail "expected exactly one probe window"
}
# A second, plain kitty takes focus so the probe is an unfocused window.
steal_focus() {   # $1 binary
    msg "$1" action spawn -- kitty --config NONE --class gos-other -o cursor_blink_interval=0 \
        sh -c "$IDLE"
    for _ in $(seq 100); do [ "$(windows_with "$1" gos-other)" -ge 1 ] && break; sleep 0.1; done
    sleep 0.5
    [ "$(msg "$1" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .is_focused')" = false ] \
        || fail "the probe is still focused"
}
# The probe's window rect in output pixels, from the IPC layout: the ROIs
# are derived from it rather than assumed.
probe_rect() {   # $1 binary; sets PX PY PW PH
    local j
    j=$(msg "$1" -j windows | jq -c '.[] | select(.app_id=="gos-probe") | .layout')
    PX=$(jq -r '.tile_pos_in_workspace_view[0] | floor' <<< "$j")
    PY=$(jq -r '.tile_pos_in_workspace_view[1] | floor' <<< "$j")
    PW=$(jq -r '.window_size[0]' <<< "$j")
    PH=$(jq -r '.window_size[1]' <<< "$j")
    [ "$PW" -gt 200 ] && [ "$PH" -gt 400 ] || fail "probe rect too small: ${PW}x${PH}+$PX+$PY"
}
shot() {   # $1 binary, $2 name -> $OUT/$2.png
    local png=$OUT/$2.png
    msg "$1" action screenshot-screen --write-to-disk true --show-pointer false --path "$png"
    for _ in $(seq 50); do [ -s "$png" ] && break; sleep 0.1; done
    [ -s "$png" ] || fail "capture $2 not written"
}
roi() {   # $1 name, $2 geometry, $3 suffix -> $OUT/$1-$3.png
    magick "$OUT/$1.png" -crop "$2" +repage -define png:color-type=2 "$OUT/$1-$3.png" \
        || fail "crop $1 $3 failed"
}
# Face: a 200x400 block inside the window, 60 px in from its left edge.
# Chamfer: the right chamfer, 12 px wide outside the window's right edge
# (bevel 12, offset-x 6: the slab's right outer edge is the window's right
# edge plus 12), with 4 px of margin either side.
face_roi() { echo "200x400+$((PX + 60))+$((PY + 120))"; }
chamfer_roi() { echo "20x400+$((PX + PW - 4))+$((PY + 120))"; }

# --- comparison -------------------------------------------------------------
# magick compare exits 0 for an identical pair, 1 for a differing pair, and
# 2 on an execution error; only 0 and 1 are comparisons. The metric is the
# first field on stderr. Helpers leave their number in METRIC so `fail` runs
# in this shell and exits the script rather than a $(...) subshell.
is_number() { [[ $1 =~ ^-?[0-9]+([.][0-9]+)?([eE][-+]?[0-9]+)?$ ]]; }
compare_metric() {   # $1 metric, $2 image, $3 image; result in METRIC
    local out status
    set +e
    out=$(magick compare -metric "$1" "$2" "$3" null: 2>&1 >/dev/null)
    status=$?
    set -e
    case $status in
        0|1) ;;
        *) fail "magick compare $1 $2 $3 exited $status: $out" ;;
    esac
    out=${out%% *}
    is_number "$out" || fail "magick compare $1 $2 $3 returned a non-numeric metric: $out"
    METRIC=$out
}
ae() { compare_metric AE "$1" "$2"; }
rmse() { compare_metric RMSE "$1" "$2"; }
sd() {   # $1 image; result in METRIC
    local out
    out=$(magick "$1" -colorspace Gray -format '%[fx:standard_deviation]' info:) || fail "magick info on $1 failed"
    is_number "$out" || fail "standard deviation of $1 is not numeric: $out"
    METRIC=$out
}
mean() {   # $1 image; result in METRIC
    local out
    out=$(magick "$1" -colorspace Gray -format '%[fx:mean]' info:) || fail "magick mean on $1 failed"
    is_number "$out" || fail "mean of $1 is not numeric: $out"
    METRIC=$out
}
# Oklab a and b only, L pinned, frozen as raw channels in a 16-bit image.
oklab_ab() {   # $1 in, $2 out
    magick "$1" -colorspace Oklab -channel R -evaluate set 50% +channel -set colorspace sRGB -depth 16 "$2" \
        || fail "oklab ab $2 failed"
}
one_code() { magick xc: -format '%[fx:quantumrange/255]' info: || fail "magick quantum range failed"; }
assert_zero() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v == 0) }' || fail "$1 expected 0, got $2"
}
assert_positive() {
    is_number "$2" || fail "$1 is not numeric: $2"
    awk -v v="$2" 'BEGIN { exit !(v > 0) }' || fail "$1 expected > 0, got $2"
}
assert_greater() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a > b) }' || fail "$1 expected $2 > $3"
}
assert_less() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a < b) }' || fail "$1 expected $2 < $3"
}
assert_about() {   # $1 name, $2 got, $3 want, $4 tolerance fraction
    is_number "$2" && is_number "$3" && is_number "$4" || fail "$1 is not numeric: $2 vs $3 (tol $4)"
    awk -v g="$2" -v w="$3" -v t="$4" 'BEGIN { exit !(g >= w*(1-t) && g <= w*(1+t)) }' \
        || fail "$1 expected $2 within $(awk -v t="$4" 'BEGIN { printf "%d%%", t*100 }') of $3"
}

# --- Tracy ------------------------------------------------------------------
reserve_tracy_port() {
    local port listeners
    for port in $(seq 20000 39999); do
        exec {TRACY_LOCK_FD}>"$XDG_RUNTIME_DIR/niri-material-tracy-$port.lock"
        if flock -n "$TRACY_LOCK_FD" \
            && listeners=$(ss -H -ltn "sport = :$port") && [ -z "$listeners" ]; then
            TRACY_PORT=$port; export TRACY_PORT
            return
        fi
        exec {TRACY_LOCK_FD}>&-
    done
    fail "no free Tracy port in 20000-39999"
}
tools_ready() {
    local retained=$EVIDENCE/material-roughness-b220152d/tools
    TOOLS=$retained
    sha256sum -c --quiet - <<EOF 2>/dev/null || fail "retained Tracy 0.13.1 tools missing or altered under $retained; rebuild them as material-signals-smoke.sh tools_ready does"
957db02d917aaf217021ff04c59fbe196415ddb5e8b9615d9e0f91ff25d75ca6  $retained/tracy-capture
588642902b24282d831cf4f4088ba7c3f28e59f0c506da3fa8f02288ec3efc30  $retained/tracy-csvexport
EOF
}
# tracy-capture's -s timer starts once a client connects and its connect loop
# is unbounded, so the capture is wrapped in a timeout.
capture_bg() {   # $1 name
    : > "$OUT/$1.capture.log"
    timeout 90 "$TOOLS/tracy-capture" -o "$OUT/$1.tracy" -a 127.0.0.1 -p "$TRACY_PORT" -s 30 \
        > "$OUT/$1.capture.log" 2>&1 & CAP_PID=$!
}
capture_ready() {   # $1 name
    local _; for _ in $(seq 300); do
        [ -n "$(ss -H -tn state established "( sport = :$TRACY_PORT or dport = :$TRACY_PORT )")" ] && return
        kill -0 "$CAP_PID" 2>/dev/null || break
        sleep 0.1
    done
    fail "tracy-capture never reported a connection (see $OUT/$1.capture.log)"
}
capture_wait() {
    local rc=0; wait "$CAP_PID" || rc=$?; CAP_PID=
    [ "$rc" -eq 0 ] || fail "tracy-capture exited $rc (no client connected, or timed out)"
}
# Column lookup by header name, never by position: the CPU (--unwrap) and
# GPU (--gpu) exports lay their columns out differently.
col() {   # $1 csv, $2 header; prints the 1-based index
    local idx; idx=$(head -1 "$1" | tr ',' '\n' | grep -nx "$2" | cut -d: -f1)
    [ -n "$idx" ] || fail "column '$2' not in $(head -1 "$1")"
    echo "$idx"
}
export_cpu() { "$TOOLS/tracy-csvexport" --unwrap "$OUT/$1.tracy" > "$OUT/$1.csv"; }
trace_end() {   # $1 name; latest timestamp of any zone, ns
    local c; c=$(col "$OUT/$1.csv" ns_since_start)
    awk -F, -v c="$c" 'NR>1 { t=$c+0; if (t>end) end=t } END { printf "%d", end }' "$OUT/$1.csv"
}
# Zones named exactly `Niri::redraw` in the final 20 s of the trace.
count_last20() {   # $1 name; prints the count
    export_cpu "$1"
    local end c; end=$(trace_end "$1"); c=$(col "$OUT/$1.csv" ns_since_start)
    awk -F, -v c="$c" -v end="$end" \
        'NR>1 && $1=="Niri::redraw" { t=$c+0; if (t>=end-20e9) n++ } END { printf "%d", n+0 }' "$OUT/$1.csv"
}
# Median exec time of the first 14 `MaterialRenderElement::draw` GPU zones
# between 20 s and 28 s of a trace, in ns; fewer than 14 samples fails.
gpu_median_ns() {   # $1 trace path
    local csv=${1%.tracy}.gpu.csv
    "$TOOLS/tracy-csvexport" --gpu "$1" > "$csv"
    local ct ce; ct=$(col "$csv" "Time from start of program"); ce=$(col "$csv" "GPU execution time")
    awk -F, -v ct="$ct" -v ce="$ce" 'NR>1 && $1=="MaterialRenderElement::draw" && $ct+0>=20e9 && $ct+0<28e9 { print $ce+0 }' "$csv" \
        | head -14 | sort -n | awk '{ a[NR]=$1 } END { if (NR!=14) { print "FAIL: " NR " MaterialRenderElement::draw samples in 20-28 s, need 14" > "/dev/stderr"; exit 1 }
              printf "%d", (a[7]+a[8])/2 }'
}
median3() { sort -n "$1" | sed -n 2p; }
# One traced session: the Tracy build, the probe, an optional focus thief,
# a 30 s capture, written to $OUT/$1.tracy.
trace_run() {   # $1 name, $2 probe body, $3 steal focus 0|1
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI_TRACY" "$OUT/$1.kdl"
    spawn_probe "$NIRI_TRACY" "$2"
    [ "$3" = 1 ] && steal_focus "$NIRI_TRACY"
    sleep 2
    capture_bg "$1"; capture_ready "$1"; capture_wait
    stop_nested
}
ns_to_ms() { awk -v n="$1" 'BEGIN { printf "%.3f", n/1e6 }'; }
pct_delta() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%+.1f", (b-a)/a*100 }'; }

finish() {
    sha256sum "$OUT"/*.png "$OUT"/*.kdl >> "$OUT/SHA256SUMS"
    if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
        fail "material error, fallback or panic in niri.log"
    fi
    echo "PASS: artifacts in $OUT"
}
```

- [ ] **Step 2: Write the iridescence smoke**

`docs/materials/scripts/glass-iridescence-smoke.sh`:

```bash
#!/usr/bin/env bash
# Glass iridescence smoke (material-6102b2): the optic's neutral renders
# identically to the unconfigured material, its hue lands on the chamfer
# rather than the face, and its frame cost is recorded.
#
# Captures: plain (no node), zero (`iridescence 0`), on (`iridescence 0.8`),
# and rainbow (the preset body) for the evidence doc. The chamfer ROI is the
# right chamfer; the face ROI is inside the window. Oklab a/b RMSE against
# the zero capture measures added chroma. Cost is the GPU median of the
# material draw for plain, zero, and on, three rounds in rotated order.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT.
set -eu
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
build_binaries

capture() {   # $1 name, $2 glass extra
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    sleep 2
    probe_rect "$NIRI"
    shot "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    roi "$1" "$(chamfer_roi)" chamfer
    stop_nested
}
capture plain ""
capture zero "iridescence 0"
capture on "iridescence 0.8"
capture rainbow $'ior 1.7\nchromatic-aberration 0.5\niridescence 0.8'   # the preset body; replaces the baseline ior and aberration

ae "$OUT/plain.png" "$OUT/zero.png"; zero_vs_plain_ae=$METRIC
for n in zero on; do
    oklab_ab "$OUT/$n-chamfer.png" "$OUT/$n-chamfer-ab.png"
    oklab_ab "$OUT/$n-face.png" "$OUT/$n-face-ab.png"
done
rmse "$OUT/on-chamfer-ab.png" "$OUT/zero-chamfer-ab.png"; chamfer_ab_rmse=$METRIC
rmse "$OUT/on-face-ab.png" "$OUT/zero-face-ab.png";       face_ab_rmse=$METRIC
rmse "$OUT/on-chamfer.png" "$OUT/zero-chamfer.png";       chamfer_rmse=$METRIC
rmse "$OUT/on-face.png" "$OUT/zero-face.png";             face_rmse=$METRIC
one_code=$(one_code)

# Cost: three rounds, order rotated to balance host drift.
tools_ready; reserve_tracy_port
gpu_case() {   # $1 case, $2 round
    case $1 in plain) GLASS_EXTRA= ;; zero) GLASS_EXTRA="iridescence 0" ;; on) GLASS_EXTRA="iridescence 0.8" ;; esac
    trace_run "gpu-$1-$2" "$TICK" 0
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}
for c in plain zero on; do gpu_case "$c" 1; done
for c in zero on plain; do gpu_case "$c" 2; done
for c in on plain zero; do gpu_case "$c" 3; done
plain_ns=$(median3 "$OUT/gpu-plain.medians")
zero_ns=$(median3 "$OUT/gpu-zero.medians")
on_ns=$(median3 "$OUT/gpu-on.medians")

{
    printf '%s=%s\n' zero_vs_plain_ae "$zero_vs_plain_ae" chamfer_ab_rmse "$chamfer_ab_rmse" \
        face_ab_rmse "$face_ab_rmse" chamfer_rmse "$chamfer_rmse" face_rmse "$face_rmse" one_code "$one_code" \
        gpu_plain_ms "$(ns_to_ms "$plain_ns")" gpu_zero_ms "$(ns_to_ms "$zero_ns")" gpu_on_ms "$(ns_to_ms "$on_ns")" \
        gpu_zero_vs_plain_pct "$(pct_delta "$plain_ns" "$zero_ns")" gpu_on_vs_plain_pct "$(pct_delta "$plain_ns" "$on_ns")"
} | tee "$OUT/metrics.txt"

assert_zero zero_vs_plain_ae "$zero_vs_plain_ae"
assert_greater chamfer_ab_rmse_over_one_code "$chamfer_ab_rmse" "$one_code"
assert_greater chamfer_ab_rmse_over_face "$chamfer_ab_rmse" "$face_ab_rmse"
finish
```

- [ ] **Step 3: Allow the key in the sweep**

In `docs/materials/scripts/glass-parameter-sweep.sh`, in the `case $BLOCK:$KEY` allowlist, add the line

```bash
    glass:iridescence|glass:aurora) ;;
```

before the `blur:noise|...` line. (`aurora <v>` with no block takes the default `drift-hz 4`, and the sweep's `animations { off; }` pins its phase through `OpticFrame`, so the sweep stays deterministic.)

- [ ] **Step 4: Run the smoke and the sweep**

```bash
chmod +x docs/materials/scripts/glass-iridescence-smoke.sh
RUN=/mnt/ssd3/niri-material/glass-iridescence-$(git rev-parse --short HEAD)-$(date +%Y%m%dT%H%M%S)
OUT=$RUN docs/materials/scripts/glass-iridescence-smoke.sh
```

Expected: `PASS: artifacts in ...`, `metrics.txt` with all eleven values. Every run gets its own directory (the lib refuses a non-empty one); a retry is a new `RUN`. If a ROI assertion fails, open `on.png` and `on-chamfer.png` and check the crop sits on the right chamfer; adjust `chamfer_roi` in the lib (the spec-derived geometry is bevel 12 outside the window's right edge with offset-x 6) and rerun. If `gpu_median_ns` reports fewer than 14 samples, the ticking probe is not damaging at 10 Hz; check `niri.log` and the kitty command.

Then the tuning sweep for the preset value:

```bash
NIRI=$RUN/niri OUT=$RUN/sweep \
BLOCK=glass KEY=iridescence VALUES="0 0.2 0.4 0.6 0.8 1" \
  docs/materials/scripts/glass-parameter-sweep.sh
```

Expected: a table of neighbour deltas per ROI. Keep `0.8` in `rainbow.kdl` unless the bevel delta has flattened by `0.6` (then use the last value before the flat region) and record the table and the decision in the evidence doc.

- [ ] **Step 5: Write the evidence doc**

`docs/materials/<YYYY-MM-DD>-material-iridescence-evidence.md`, dated the day of the run, following `2026-09-06-material-glass-noise-type-evidence.md`:

```markdown
# Glass iridescence: verification evidence

**Result:** PASS, <date>, headless Weston <version>. All captures completed
without renderer errors, fallbacks, or panics in `niri.log`.

## Pinned revisions

- Source commit `<full sha>`, release binary SHA-256 `<sha>`, Tracy build
  SHA-256 `<sha>` (from `binaries.sha256`).

## Metrics

| Metric | Value |
| --- | ---: |
| `zero_vs_plain_ae` | <v> |
| `chamfer_ab_rmse` | <v> |
| `face_ab_rmse` | <v> |
| `chamfer_rmse` | <v> |
| `face_rmse` | <v> |
| `one_code` | <v> |
| `gpu_plain_ms` | <v> |
| `gpu_zero_ms` | <v> |
| `gpu_on_ms` | <v> |
| `gpu_zero_vs_plain_pct` | <v> |
| `gpu_on_vs_plain_pct` | <v> |

<Two or three sentences: the zero capture matched the unconfigured one
decoded pixel for pixel (AE 0); the chamfer gained <v> Q16 of Oklab a/b
RMSE against <v> on the face; the material draw cost <v> ms plain,
<v> ms at amount 0 (the uniform branch), <v> ms at 0.8.>

## Preset tuning

<The sweep table for `iridescence` at 0, 0.2, 0.4, 0.6, 0.8, 1 and the
value kept in `rainbow.kdl`, with the reason.>

Captures and traces: `<OUT path>`; `SHA256SUMS` there lists every capture.
```

- [ ] **Step 6: Commit**

```bash
chmod +x docs/materials/scripts/glass-optic-smoke-lib.sh
tasks start material-c07577
git add docs/materials/scripts/glass-optic-smoke-lib.sh docs/materials/scripts/glass-iridescence-smoke.sh docs/materials/scripts/glass-parameter-sweep.sh docs/materials/<date>-material-iridescence-evidence.md resources/materials/rainbow.kdl
tasks done material-c07577 "Iridescence smoke: neutral AE 0, chamfer chroma <v> vs face <v>, draw cost <plain> / <zero> / <on> ms; rainbow tuned at <v>"
tasks done material-6102b2 "Iridescence optic landed: specular hook, rainbow preset, cost <on> ms vs <plain> ms plain (<pct>)"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "test(material): iridescence smoke and evidence"
```

---

### Task 8: The aurora smoke, its evidence, and the closing docs

**Files:**
- Create: `docs/materials/scripts/glass-aurora-smoke.sh`
- Create: `docs/materials/<run-date>-material-aurora-evidence.md`
- Modify: `resources/materials/aurora.kdl` if tuning moves a value
- Modify: `docs/specs/2026-09-10-material-optics-design.md` (status header, §7.3 radius, §8 tuned values)
- Modify: `docs/materials/material-config.md` (`### aurora` cost/redraw sentence if the numbers warrant a caveat)

**Interfaces:**
- Consumes: the lib from Task 7; `aurora` from Tasks 4–5; the preset from Task 6.

- [ ] **Step 1: Write the aurora smoke**

`docs/materials/scripts/glass-aurora-smoke.sh`:

```bash
#!/usr/bin/env bash
# Glass aurora smoke (material-8db3b0): the optic's neutral renders
# identically to the unconfigured material; a pinned field (`drift-hz 0`)
# is deterministic; a moving field on an unfocused, signal-free window
# changes across a bucket boundary and not within one; the field adds
# light on the face; redraws per second track drift-hz under each motion
# policy; and frame cost is recorded.
#
# Bucket check: at `drift-hz 1` a bucket is 1 s. Four captures taken back
# to back span well under a second, so at least one consecutive pair lies
# in one bucket and must be identical; a capture 2.5 s after the first lies
# in a later bucket and must differ. Redraw rates come from Tracy counts of
# `Niri::redraw` in the final 20 s of a 30 s capture with both windows idle.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT.
set -eu
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
build_binaries

session() {   # $1 name, $2 glass extra; leaves the session running, probe unfocused
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    steal_focus "$NIRI"
    sleep 2
    probe_rect "$NIRI"
}
capture() {   # $1 name, $2 glass extra: one shot with ROIs
    session "$1" "$2"
    shot "$NIRI" "$1"
    roi "$1" "$(face_roi)" face
    stop_nested
}

capture plain ""
capture zero "aurora 0 { drift-hz 4; }"
capture preset $'attenuation-color "#cfe0ff"\naurora 0.5 { drift-hz 0; color "#3dffb0"; color "#7a5cff"; }'   # the preset body pinned; replaces the baseline attenuation-color

# Pinned: two shots 2 s apart are identical, and the field lights the face.
session pinned "aurora 0.5 { drift-hz 0; }"
shot "$NIRI" pinned-a; roi pinned-a "$(face_roi)" face
sleep 2
shot "$NIRI" pinned-b
stop_nested

# Moving at 1 Hz, unfocused, signal-free.
session moving "aurora 0.5 { drift-hz 1; }"
for i in 1 2 3 4; do shot "$NIRI" "moving-$i"; done
sleep 2.5
shot "$NIRI" moving-late
stop_nested

ae "$OUT/plain.png" "$OUT/zero.png";           zero_vs_plain_ae=$METRIC
ae "$OUT/pinned-a.png" "$OUT/pinned-b.png";    pinned_ae=$METRIC
rmse "$OUT/pinned-a-face.png" "$OUT/zero-face.png"; face_rmse=$METRIC
mean "$OUT/pinned-a-face.png"; lit_face_mean=$METRIC
mean "$OUT/zero-face.png";     zero_face_mean=$METRIC
within_min=
for i in 1 2 3; do
    ae "$OUT/moving-$i.png" "$OUT/moving-$((i + 1)).png"
    printf -v "moving_ae_$i" '%s' "$METRIC"
    if [ -z "$within_min" ] || awk -v a="$METRIC" -v b="$within_min" 'BEGIN { exit !(a < b) }'; then within_min=$METRIC; fi
done
ae "$OUT/moving-1.png" "$OUT/moving-late.png"; across_ae=$METRIC
one_code=$(one_code)

# Redraw rate against drift-hz: idle windows, the probe unfocused.
tools_ready; reserve_tracy_port
rate_case() {   # $1 name, $2 glass extra, $3 top extra
    GLASS_EXTRA=$2; TOP_EXTRA=$3
    trace_run "rate-$1" "$IDLE" 1
    count_last20 "rate-$1"
}
rate_plain=$(rate_case plain "" "")
rate_pinned=$(rate_case pinned "aurora 0.5 { drift-hz 0; }" "")
rate_4hz=$(rate_case 4hz "aurora 0.5 { drift-hz 4; }" "")
rate_reduced=$(rate_case reduced "aurora 0.5 { drift-hz 4; }" 'signal { motion "reduced"; }')
rate_off=$(rate_case off "aurora 0.5 { drift-hz 4; }" 'signal { motion "off"; }')
TOP_EXTRA=

# Cost: three rounds, order rotated.
gpu_case() {   # $1 case, $2 round
    case $1 in plain) GLASS_EXTRA= ;; zero) GLASS_EXTRA="aurora 0 { drift-hz 4; }" ;; on) GLASS_EXTRA="aurora 0.5 { drift-hz 4; }" ;; esac
    trace_run "gpu-$1-$2" "$TICK" 0
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}
for c in plain zero on; do gpu_case "$c" 1; done
for c in zero on plain; do gpu_case "$c" 2; done
for c in on plain zero; do gpu_case "$c" 3; done
plain_ns=$(median3 "$OUT/gpu-plain.medians")
zero_ns=$(median3 "$OUT/gpu-zero.medians")
on_ns=$(median3 "$OUT/gpu-on.medians")

{
    printf '%s=%s\n' zero_vs_plain_ae "$zero_vs_plain_ae" pinned_ae "$pinned_ae" face_rmse "$face_rmse" \
        lit_face_mean "$lit_face_mean" zero_face_mean "$zero_face_mean" one_code "$one_code" \
        moving_ae_1 "$moving_ae_1" moving_ae_2 "$moving_ae_2" moving_ae_3 "$moving_ae_3" \
        within_bucket_min_ae "$within_min" across_bucket_ae "$across_ae" \
        redraws_20s_plain "$rate_plain" redraws_20s_pinned "$rate_pinned" redraws_20s_4hz "$rate_4hz" \
        redraws_20s_reduced "$rate_reduced" redraws_20s_off "$rate_off" \
        gpu_plain_ms "$(ns_to_ms "$plain_ns")" gpu_zero_ms "$(ns_to_ms "$zero_ns")" gpu_on_ms "$(ns_to_ms "$on_ns")" \
        gpu_zero_vs_plain_pct "$(pct_delta "$plain_ns" "$zero_ns")" gpu_on_vs_plain_pct "$(pct_delta "$plain_ns" "$on_ns")"
} | tee "$OUT/metrics.txt"

assert_zero zero_vs_plain_ae "$zero_vs_plain_ae"
assert_zero pinned_ae "$pinned_ae"
assert_greater face_rmse_over_one_code "$face_rmse" "$one_code"
assert_greater lit_face_brighter "$lit_face_mean" "$zero_face_mean"
assert_zero within_bucket_min_ae "$within_min"
assert_positive across_bucket_ae "$across_ae"
assert_zero redraws_20s_plain "$rate_plain"
assert_zero redraws_20s_pinned "$rate_pinned"
assert_about redraws_20s_4hz "$rate_4hz" 80 0.25
assert_about redraws_20s_reduced "$rate_reduced" 40 0.25
assert_zero redraws_20s_off "$rate_off"
finish
```

- [ ] **Step 2: Run the smoke and the sweep**

```bash
chmod +x docs/materials/scripts/glass-aurora-smoke.sh
RUN=/mnt/ssd3/niri-material/glass-aurora-$(git rev-parse --short HEAD)-$(date +%Y%m%dT%H%M%S)
OUT=$RUN docs/materials/scripts/glass-aurora-smoke.sh
```

Expected: `PASS`, `metrics.txt` with all twenty-two values. Every run gets its own directory (the lib refuses a non-empty one); a retry is a new `RUN`. Failure notes: `within_bucket_min_ae` non-zero means every consecutive shot pair straddled a boundary, which at 1 Hz means a shot took over a second; check `niri.log` timing and rerun. `redraws_20s_4hz` far above 80 means something else redraws (the focus thief's cursor, an animation); far below means the deadline is not firing for the unfocused tile, which contradicts the Task 5 tile test and is a bug in `tick_deadline`'s call site, not in the smoke.

Then the tuning sweep:

```bash
NIRI=$RUN/niri OUT=$RUN/sweep \
BLOCK=glass KEY=aurora VALUES="0 0.2 0.35 0.5 0.7 1" \
  docs/materials/scripts/glass-parameter-sweep.sh
```

Keep `0.5` in `aurora.kdl` unless the face delta has flattened before it; record the table and the decision in the evidence doc. Look at `preset.png` once: the field should read as a soft green-to-violet wash inside the slab, not a flat tint; if the coarse-octave brightness makes it patchy, that is a finding for the evidence doc, not a value to change here.

- [ ] **Step 3: Write the evidence doc**

`docs/materials/<YYYY-MM-DD>-material-aurora-evidence.md`, same shape as the iridescence one, with the twenty-two metrics in the table and these paragraphs:

- Neutral and determinism: `zero_vs_plain_ae` 0 and `pinned_ae` 0.
- The scheduling check of the optics design §11: the unfocused, signal-free window changed across a bucket (`across_bucket_ae`) and not within one (`within_bucket_min_ae` 0, per-pair values listed).
- Redraw rate against `drift-hz`: the five counts, and the derived per-second rates (`count / 20`) against 4, 2, 0.
- Cost: plain, amount 0, and lit at 4 Hz, with the percentage deltas.
- Preset tuning: the sweep table and the value kept.
- The loop radius: 2.0 noise units, about 5 px/s, and why the spec's 40 was not used (105 px/s).

- [ ] **Step 4: Close the docs**

`docs/specs/2026-09-10-material-optics-design.md`:

- Status header: replace "Sections 7–9 (the three optics, presets, and Prism mappings) remain separate plans." with "§7.2, §7.3, and §8's `rainbow` and `aurora` presets landed on `materials-26.04` (plan: `../plans/2026-09-10-material-aurora-iridescence.md`; evidence: `../materials/<date>-material-iridescence-evidence.md`, `../materials/<date>-material-aurora-evidence.md`). §7.1, `ice`, and §9 remain open (`material-bb3fe5`, `prism-763054`, `prism-08c1de`)."
- §7.3: replace "the lookup point also traces a circle of radius 40 noise units over the period, so the loop is seamless" with "the lookup point also traces a circle of radius 2 noise units (about 5 px/s at the 0.004 scale) over the period, so the loop is seamless; the landed constant is `AURORA_LOOP_RADIUS` in `aurora.frag`".
- §8: if tuning changed a preset value, update the KDL block to the tuned value.

Grep the user-facing docs for the same claims: `rg -n 'radius 40|remain separate plans' docs/` must return nothing after the edits.

- [ ] **Step 5: Run the full gate**

Run: `just gate`
Expected: PASS.

- [ ] **Step 6: Commit and close the goal**

```bash
tasks start material-1bcd1d
git add docs/materials/scripts/glass-aurora-smoke.sh docs/materials/<date>-material-aurora-evidence.md resources/materials/aurora.kdl docs/specs/2026-09-10-material-optics-design.md docs/materials/material-config.md
tasks done material-1bcd1d "Aurora smoke: neutral AE 0, bucket check live, <n>/20 s redraws at 4 Hz (<n> reduced, 0 pinned/off), draw cost <plain> / <zero> / <on> ms; aurora tuned at <v>"
tasks done material-8db3b0 "Aurora optic landed: emissive hook on the 600 s clock, redraws track drift-hz (<rate>/s at 4 Hz), cost <on> ms vs <plain> ms plain (<pct>)"
git add tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "test(material): aurora smoke and evidence"
```

Then hand off to `superpowers:finishing-a-development-branch` for the merge into `materials-26.04`, and finish with Task 9 there.

---

### Task 9: Repin the Arch package on the merged head

**Files:**
- Modify: `packaging/arch/PKGBUILD` (through `tools/package-pin`)

**Interfaces:**
- Consumes: the merged `materials-26.04` head carrying `resources/materials/` and the PKGBUILD install line from Task 6.

The pin names the commit `makepkg` fetches from GitHub. `package-pin --check` only verifies the three derived values agree with each other, not that the pinned tree can build the `package()` function around it, so the pin must move to a commit that has `resources/materials/`. The commit must be on `origin/materials-26.04` before `makepkg` runs, and the pin points at the head before the pin commit itself.

- [ ] **Step 1: Verify the merged head is pushed and carries the presets**

Run, on `materials-26.04` in the main checkout after the merge:

```bash
git fetch origin
git status -sb | head -1                       # expect: no ahead/behind against origin/materials-26.04
git ls-tree --name-only HEAD resources/materials/   # expect: aurora.kdl rainbow.kdl
rg -n 'resources/materials' packaging/arch/PKGBUILD  # expect: the install line from Task 6
```

If the branch is ahead, `git push` first. If `origin/materials-26.04` was rewritten since (see the pin rollout note: `filter-branch` has killed a pin before), re-fetch and confirm the intended commit is the one on origin.

- [ ] **Step 2: Repin and verify**

```bash
just package-pin
just check
rg -n 'pkgver=|#commit=|NIRI_BUILD_COMMIT' packaging/arch/PKGBUILD
```

Expected: `package-pin` rewrites the three values to the current HEAD (a commit that includes Task 6); `just check` passes; the three lines name that commit's full and short hashes with an `r<N>` greater than 353.

- [ ] **Step 3: Commit, close the goal, and push**

```bash
tasks start material-3be13f
git add packaging/arch/PKGBUILD
tasks done material-3be13f "PKGBUILD repinned on the merged head carrying resources/materials"
tasks done material-f0fc7b "Iridescence and aurora optics with the rainbow and aurora presets, packaged; particles deferred to material-1c5a30 and material-54bcac"
git add tasks/
git commit -m "chore(packaging): pin the aurora and rainbow presets build"
git push
```

Then the operator runs `makepkg -si` (the install needs sudo) and confirms with `niri validate -c <config using include "/usr/share/niri/materials/aurora.kdl">` that the installed build parses the presets, per the deployment rule.
