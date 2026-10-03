# Glass Edge Optics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** draft for plan review, 2026-10-02. Task `material-be611b`.

**Goal:** Give the glass a height-field bevel with a profile control, energy-conserving Fresnel, an untinted scene reflection and a light-facing edge highlight, so a glass edge reads as a clear, lit edge of this glass at any tint and thickness.

**Architecture:** The bevel's geometry and light path move into one place per side: `src/render_helpers/material/bevel.rs` holds the two-boundary coordinate, the profile, the softened outer gradient, the normal and the ray model in f64, and `prelude.frag` mirrors it line for line (a test pins the shared constants, as `ring.rs` does for the beam). `slabSurface` gains the local height `h`, the across-bevel coordinate `u` and its direction; the taps and Beer-Lambert follow one refracted ray to the backdrop plane; transmitted light is scaled by `1 - F`. The specular hook takes a `Surface` struct, so two new static optics, `reflection` and `edge-highlight`, sit in the specular stage ahead of `iridescence`. Evidence comes from frozen-clock renders through the real shader in the headless test backend (deterministic, comparable across commits), plus one contact sheet on the headless Weston host for the owner's judgement.

**Tech Stack:** Rust (workspace crates `niri-config`, `niri`), knuffel (KDL), glam (`DVec2`, `DVec3`), smithay GLES renderer, GLSL ES 1.00, the headless test backend (`src/tests/`, surfaceless EGL), Python 3 stdlib for the comparison script, bash plus ImageMagick 7 for the contact sheet, `just` front door.

**Spec:** `docs/specs/2026-09-30-glass-edge-optics-design.md` (accepted in spec review round 4). Read it alongside this plan; section numbers below refer to it. Read `docs/materials/render-pipeline.md` before touching the shaders (AGENTS.md).

## Global Constraints

- Branch `glass-edges`, worktree `.worktrees/glass-edges`. Run every command from the worktree; paths shown to the user are prefixed `.worktrees/glass-edges/`. The worktree builds into its own `target/` (its `.cargo/config.toml` is comment-only, material-77be96): the first build is a full one.
- Conventional commits; no AI-attribution or session trailers. Each plan task is a child of `material-be611b`, in order: Task 1 `material-10e12e`, 2 `material-ce3229`, 3 `material-f1b307`, 4 `material-d37c1a`, 5 `material-c210a0`, 6 `material-d63184`, 7 `material-02b42a`, 8 `material-5e64ef`, 9 `material-124f1f`. `tasks start <id>` before a task, `tasks done <id> "<what landed>"` in the same commit as its code, `tasks check` before every commit.
- Run `cargo fmt --all` before every `git add` of Rust: the plan's code blocks are not rustfmt-formatted, and the hook runs `cargo fmt --all -- --check`.
- Every commit passes the pre-commit hook (`just check`). Never bypass it. A file added anywhere counts as a fork path: after `git add`, run `python3 tools/upstream-report && git add docs/materials/upstream-divergence.md` before `git commit` (the hook also stages it, but a failed hook leaves the index stale).
- Every test run goes through `just`: `just test-one -p <crate> <filter>` while working (nextest arguments), `just test-fast` before each commit. Never call cargo test or nextest directly.
- `bevel-profile` is `FloatOrInt<1, 8>`, default 1. `reflection` and `edge-highlight` are `FloatOrInt<0, 1>`, default 0, neutral at 0 (the optic returns its input first thing). Uniform prefixes: `mat_bevel_profile` (core), `mat_reflection`, `mat_edge_highlight`, `mat_edge_highlight_alpha`.
- `ORDER` in `niri-config/src/material/optics/mod.rs` and `OPTICS` in `src/render_helpers/material/optics/mod.rs` agree at every commit. Final order: `saturation`, `noise`, `aurora`, `reflection`, `edge-highlight`, `iridescence`. A config-side change and its renderer registration land in one commit.
- The parameter table in `docs/materials/material-config.md` is generated: after any `params()` change run `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_parameter_table_matches_the_docs` and commit the regenerated block.
- After any `.frag` change, validate the assembled shader offline with the command in Task 4 Step 6 (it writes `target/material.frag` and runs `glslangValidator -S frag`; expect exit 0), listing the optics in `OPTICS` order.
- Constants shared by `bevel.rs` and `prelude.frag` carry the same names; `bevel::tests::the_shader_names_the_same_constants_and_formulas` pins them (and the highlight's lines, `the_highlight_shader_names_the_same_formulas`). Change both together.
- Evidence artifacts (dumps, sheets) go under `EV=/mnt/ssd3/niri-material/material-be611b/<run-stamp>`, never into the repository. Evidence documents that cite them are committed under `docs/materials/`.
- No compatibility switch restores the old bevel (§4). Pixels outside the slab and opaque window pixels never change.
- The contact sheet (Task 9) needs an idle host for the headless Weston lane. If `capture_preflight headless` refuses, record a `run:` note for the attempt and park: `tasks park material-124f1f "Rerun docs/materials/scripts/glass-edge-sheet.sh with SHEET_PILOT=1, then the full sheet (Task 9 Steps 2-3): build about 10 min, cells about 10 min; last attempt refused by the headless preflight" --reason quiet --waiting-on user --minutes 25 --needs headless`.
- The test renderer may be llvmpipe (surfaceless EGL picks what the host offers). Task 5 records the renderer string; if it is not the NVIDIA driver, Task 9's sheet is the shader's first compile on the real driver, and a compile failure there stops the task.

## Deviations from the spec, decided here

- **Intended-change, neutrality and motion evidence** (§6) come from frozen-clock renders in the headless test backend (`src/tests/glass_edge.rs`, modelled on `ring_pair.rs`), not from headless Weston captures or the niri-experiments jelly companion. The renders go through the same material shader and `RenderTarget::ScreenCapture` path as a screenshot, are bit-identical across runs on one host, can be repeated at two commits in minutes, and freeze motion at an exact instant, which the companion can only approximate. The face predictions and the decoded-identical checks are the spec's, unchanged. The contact sheet stays on the headless Weston host, because it needs a real wallpaper behind the glass.
- `slabSurface` gains `h`, `u` and the across-bevel direction (`acrossDir`), not `w` (§3.1 says `h`, `u` and `w`): no consumer reads `w`, and the reflection needs the direction.
- The contact sheet is a new script, `glass-edge-sheet.sh`, not `glass-parameter-sweep.sh` (§6 names the sweep): the sweep varies one key at a time, and the sheet crosses four keys over three looks; it reuses the smoke library the sweep's siblings use.
- `edge-highlight`'s GGX `alpha = mix(0.04, 0.5, roughness)` is computed in Rust (`bevel::highlight_alpha`) and uploaded as `mat_edge_highlight_alpha`, so the mirror and the shader cannot disagree on it.

## Owner priorities (2026-10-01 note on material-be611b)

"Edges read flat and static: no transparency, no light effects, no glass noise on the chamfer."

- **Transparency:** Task 4. The bevel thins toward the silhouette, so Beer-Lambert there runs over the local height instead of `thickness / cos`: at the live focused glass (thickness 31.2, bevel 15, attenuation 11) the rim transmits about 8 % against 0.8 % on the face, where today the bevel transmits less than the face. Task 7's reflection then carries the wallpaper's colours on the dark edge, untinted.
- **Light effects:** Task 7 (`reflection`) and Task 8 (`edge-highlight`), plus the glint, which now varies across a rounded bevel instead of sitting at `f0`.
- **Motion:** distortion and jelly ripple move the reflection (its direction carries the perturbation) and the highlight (it uses the perturbed normal); `attention "rim-orbit"` sways the highlight with the glint.
- **Glass noise on the chamfer:** not covered here. Grain acts on transmitted light only, so the thinner rim carries more of it after Task 4, but nothing grains the surface light. Follow-up idea `material-1aa3af` (surface grain on the glass edge), to scope after the contact sheet shows how much grain the rim already carries.

## Review Focus

Inputs the spec implies but its tests do not name, most likely first; each line's test is added to the owning task.

1. **A chamfer narrower than one physical pixel** (bevel 1 at scale 2, or a jelly squeeze): every bevel fragment takes the rim branch (`u = 1`); the normal must stay finite and capped, and `u` must not exceed 1. Test: Task 2 `a_band_narrower_than_a_pixel_is_all_rim`.
2. **`bevel 0` and `thickness 0`:** no bevel, or a slab with no height. The face normal and `L = h` must hold, with no division by zero in `u`, the slope or the path. Test: Task 2 `no_chamfer_or_no_thickness_is_a_flat_face`.
3. **A distortion that cancels the across-bevel direction** in the reflection's bent direction (`acrossDir + perturbed.xy - structural.xy` near zero): the sample must fall back to `acrossDir`, never `normalize(0)`. Test: Task 7 `reflection_guards_a_cancelled_direction` (a source assertion on the guard line, since the GLSL cannot run in a unit test) plus the motion render with `distortion 1`.
4. **A translucent client over the face** (kitty at opacity 0.6): `(1 - F)` scales transmitted glass under the window too, so the composite darkens by `(1 - win.a) * f0 * T`, within a code value at ior 1.28. Expected, not a regression. Test: Task 5's live cases render a 0.6-opacity client variant and report the face delta.
5. **Signal light straight down the z axis** (`mat_sig_light.xy` zero): the existing glint already normalizes it, and the highlight would too. Expected: the signal layer never sends a zero light. Test: Task 8 `the_signal_light_is_never_vertical` pins that `SignalUniforms::from_frame` gives a unit-length xy under every attention response, level and breath.

---

## File structure

| Path | Responsibility |
| --- | --- |
| `niri-config/src/material/mod.rs` | `bevel_profile` on `Glass` and `ResolvedGlass`, its default, `resolve`, `core_params` spec; `reflection`, `edge_highlight` fields |
| `niri-config/src/material/optics/reflection.rs`, `edge_highlight.rs` | the two optics' node types, resolved structs, `resolve`, `params` |
| `niri-config/src/material/optics/mod.rs` | `ORDER`, aggregated `params()` |
| `niri-config/src/lib.rs` | re-exports; parse, default and bound tests; the exhaustive `ResolvedGlass` literal |
| `src/render_helpers/material/bevel.rs` | the f64 mirror: coordinate, profile, softened gradient, normal, ray, lift, highlight weight; its tests |
| `src/render_helpers/material/mod.rs` | `pub mod bevel;`, `mat_bevel_profile` upload |
| `src/render_helpers/material/optics/reflection.rs`, `edge_highlight.rs` | `ReflectionOptic`, `EdgeHighlightOptic` |
| `src/render_helpers/material/optics/mod.rs` | `OPTICS` registry |
| `src/render_helpers/shaders/material/prelude.frag` | constants, `lsp`, `tanhs`, `softOuterGrad`, profile functions, `slabSurface`, `rayPath`, `liftTapNormal`, `tap`, `Surface` |
| `src/render_helpers/shaders/material/main.frag` | taps, attenuation, `(1 - F)`, spill on `u`, the `Surface` hook calls |
| `src/render_helpers/shaders/material/iridescence.frag`, `reflection.frag`, `edge_highlight.frag` | specular hooks |
| `src/render_helpers/shaders/mod.rs` | `mat_bevel_profile` uniform name; assembly tests |
| `src/tests/glass_edge.rs`, `src/tests/mod.rs` | the frozen-clock render harness and the motion test |
| `docs/materials/scripts/glass-edge-compare.py` | before/after checks over harness dumps |
| `docs/materials/scripts/glass-edge-sheet.sh` | the contact sheet on the headless Weston host |
| `docs/materials/material-config.md` | generated table; `bevel-profile` prose; `### reflection`, `### edge-highlight` |
| `docs/materials/render-pipeline.md` | stages 1, 3, 4, 5, 6, 8; §5 parameter rows |
| `docs/materials/adding-an-optic.md` | the specular hook signature and `Surface` |
| `docs/materials/<run-date>-glass-edge-optics-evidence.md` | evidence from Tasks 5 to 9 |
| `docs/specs/2026-09-30-glass-edge-optics-design.md` | status header at the end |

---

### Task 1: `bevel-profile` in niri-config

**Files:**
- Modify: `niri-config/src/material/mod.rs` (`Glass` after `bevel`; `ResolvedGlass` after `bevel`; `Default`; `core_params` after the `bevel` spec; `Material::resolve`)
- Modify: `niri-config/src/lib.rs` (tests; the exhaustive `ResolvedGlass` literal in `material_full_glass_parses`, about line 1453)
- Modify: `docs/materials/material-config.md` (generated block)

**Interfaces:**
- Produces: `ResolvedGlass::bevel_profile: f64` (1 to 8, default 1); KDL `glass { bevel-profile <k>; }`.

- [ ] **Step 1: Write the failing tests** in `niri-config/src/lib.rs`, beside `iridescence_resolves_through_its_optic`:

```rust
    #[test]
    fn bevel_profile_parses_and_defaults_to_planar() {
        let written = do_parse(r##"material "edge" { glass { bevel-profile 2.5; }; }"##);
        assert_eq!(written.materials[0].resolve().glass.bevel_profile, 2.5);
        let omitted = do_parse(r##"material "edge" { glass {}; }"##);
        assert_eq!(omitted.materials[0].resolve().glass.bevel_profile, 1.);
        assert_eq!(ResolvedGlass::default().bevel_profile, 1.);
    }

    #[test]
    fn bevel_profile_rejects_values_outside_one_and_eight() {
        for value in ["0.99", "8.01"] {
            let err = do_parse_err(&format!(
                "material \"edge\" {{ glass {{ bevel-profile {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 1 and 8"), "{err}");
        }
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri-config bevel_profile`
Expected: compile error, no field `bevel_profile` on `ResolvedGlass`.

- [ ] **Step 3: Implement.** In `Glass`, after `pub bevel: Option<FloatOrInt<0, 128>>,`:

```rust
    /// Bevel profile exponent: 1 is a planar chamfer, 2 a quarter-round, and
    /// higher values a squircle that stays flat longer and rolls off harder
    /// (docs/specs/2026-09-30-glass-edge-optics-design.md §3.1).
    #[knuffel(child, unwrap(argument))]
    pub bevel_profile: Option<FloatOrInt<1, 8>>,
```

In `ResolvedGlass`, after `pub bevel: f64,`: `pub bevel_profile: f64,`. In `Default`, after `bevel: 12.,`: `bevel_profile: 1.,`. In `Material::resolve`, after the `bevel:` line:

```rust
                bevel_profile: g.bevel_profile.map_or(d.bevel_profile, |x| x.0),
```

In `core_params`, after the `bevel` spec:

```rust
        ParamSpec {
            node: "bevel-profile",
            kind: ParamKind::float::<FloatOrInt<1, 8>>(d.bevel_profile, "—"),
            write: |v| format!("bevel-profile {v}"),
            read: Some(|g| Some(g.bevel_profile)),
        },
```

In the exhaustive literal in `niri-config/src/lib.rs` (`bevel: 20.,` in the `frost` material test), add `bevel_profile: 1.,` after it. `cargo build` names any other exhaustive literal; give each the default.

- [ ] **Step 4: Regenerate the parameter table and run the tests**

Run: `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_parameter_table_matches_the_docs`, then `just test-one -p niri-config bevel_profile`, then `just test-one -p niri-config material_parameter`
Expected: all pass; `git diff docs/materials/material-config.md` shows one new `bevel-profile` row (range 1 to 8, default 1).

- [ ] **Step 5: Commit**

```bash
just test-fast
tasks done material-10e12e "bevel-profile parses, defaults to 1, bounded 1-8; table row generated"
tasks check
git add niri-config/src/material/mod.rs niri-config/src/lib.rs docs/materials/material-config.md tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(config): add the glass bevel-profile parameter (material-be611b)"
```

---

### Task 2: The bevel mirror in f64

**Files:**
- Create: `src/render_helpers/material/bevel.rs`
- Modify: `src/render_helpers/material/mod.rs` (add `pub mod bevel;` after `pub mod optics;`)

**Interfaces:**
- Produces (used by Tasks 4 and 8 and by the shader-line test):
  - constants `BEVEL_SLOPE_CAP = 20.`, `BEVEL_OUTER_SOFTEN = 0.5`, `BEVEL_RIDGE_EPS = 1e-4`, `BEVEL_PROFILE_EPS = 1e-6`, `BEVEL_LSP_SMALL = 1e-3`, `BEVEL_TANH_CLAMP = 10.`, `RAY_PATH_FLOOR = 0.25`, `TAP_MIN_Z = 0.05`;
  - `type Radii = [f64; 4]`; `corner_radius`, `sd_rounded_box`, `sd_rounded_box_grad`, `lsp`, `tanhs`, `soft_outer_grad(p: DVec2, b: DVec2, radii: Radii) -> DVec2` (ZERO on the ridge), `profile(u, k)`, `profile_slope(u, k)`;
  - `struct Slab { center, half, outer_r, face_center, face_half, face_r, chamfer, thickness, profile, aa }` with `Slab::at_rest(half: DVec2, chamfer: f64, face_radius: f64, thickness: f64, profile: f64) -> Slab`, `Slab::deformed(self, shift: DVec2, resize: DVec2) -> Slab`, `Slab::surface(&self, p: DVec2) -> EdgeSample`, `Slab::surface_with(&self, p, outer_grad: fn(DVec2, DVec2, Radii) -> DVec2) -> EdgeSample`;
  - `struct EdgeSample { outer_dist: f64, inner_dist: f64, height: f64, across: f64, across_dir: DVec2, normal: DVec3 }`;
  - `refract(i, n, eta) -> DVec3`, `ray(n: DVec3, ior: f64) -> DVec3`, `ray_path(t: DVec3, h: f64) -> f64`, `lift_tap_normal(n: DVec3) -> DVec3`, `displacement(n: DVec3, ior: f64, h: f64) -> DVec2`.

- [ ] **Step 1: Write the module with its tests first.** Create `src/render_helpers/material/bevel.rs` containing the test module below and the signatures above with `todo!()` bodies, so the tests compile and fail.

```rust
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use glam::Vec2;

    use super::*;

    const IOR: f64 = 1.28;

    /// The point `u` across the right side of a resting 112 px slab, chamfer 12.
    fn right_side(thickness: f64, k: f64, u: f64) -> EdgeSample {
        Slab::at_rest(DVec2::splat(56.), 12., 0., thickness, k).surface(DVec2::new(44. + 12. * u, 0.))
    }

    #[test]
    fn the_reference_table_holds() {
        // Spec §3.1: (thickness, k, u, h, n.z, L, displacement) at chamfer 12, ior 1.28.
        let rows = [
            (6., 1., 0., 6.00, 0.894, 6.03, 0.64),
            (6., 1., 0.9, 0.60, 0.894, 0.60, 0.06),
            (6., 2., 0.9, 2.62, 0.696, 2.67, 0.55),
            (12., 1., 0.5, 6.00, 0.707, 6.12, 1.22),
            (12., 2., 0.9, 5.23, 0.436, 5.55, 1.85),
            (31.2, 1., 1., 19.20, 0.707, 19.59, 3.89),
            (31.2, 2., 0.9, 24.43, 0.436, 25.91, 8.64),
            (31.2, 2., 1., 19.20, 0.050, 23.69, 13.87),
            (6., 2., 1., 0.00, 0.050, 0.00, 0.00),
            (12., 2., 1., 0.00, 0.050, 0.00, 0.00),
            (75.3, 2., 0.9, 68.53, 0.436, 72.69, 24.24),
            (75.3, 2., 1., 63.30, 0.050, 78.10, 45.74),
            (12.1, 2., 0.9, 5.33, 0.436, 5.65, 1.89),
            (12.1, 2., 1., 0.10, 0.050, 0.12, 0.07),
        ];
        for (thickness, k, u, h, nz, path, shift) in rows {
            let s = right_side(thickness, k, u);
            let t = ray(s.normal, IOR);
            for (name, got, want) in [
                ("h", s.height, h),
                ("n.z", s.normal.z, nz),
                ("L", ray_path(t, s.height), path),
                ("shift", displacement(s.normal, IOR, s.height).length(), shift),
            ] {
                assert!((got - want).abs() < 0.01, "{thickness}/{k}/{u}: {name} {got} vs {want}");
            }
        }
    }

    #[test]
    fn the_face_is_flat_and_its_path_is_the_thickness() {
        let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 31.2, 2.);
        let face = slab.surface(DVec2::new(10., -5.));
        assert_eq!(face.normal, DVec3::Z);
        assert_eq!((face.height, face.across), (31.2, 0.));
        assert!((ray_path(ray(face.normal, IOR), face.height) - 31.2).abs() < 1e-12);
        assert!(displacement(face.normal, IOR, face.height).length() < 1e-12);
    }

    #[test]
    fn upward_normals_bound_the_ray_and_the_floor_binds_only_past_index_four() {
        let cap_z = 1. / 401f64.sqrt();
        for ior in [1., 1.28, 3.] {
            for i in 0..=1000 {
                let z = cap_z + (1. - cap_z) * f64::from(i) / 1000.;
                let t = ray(DVec3::new((1. - z * z).sqrt(), 0., z), ior);
                assert!(-t.z >= 1. / ior - 1e-12, "ior {ior}, n.z {z}: -t.z {}", -t.z);
                assert!(-t.z > RAY_PATH_FLOOR);
            }
        }
        let rim = DVec3::new((1f64 - 1. / 401.).sqrt(), 0., cap_z);
        assert!(-ray(rim, 4.).z >= RAY_PATH_FLOOR);
        assert!(-ray(rim, 9.).z < RAY_PATH_FLOOR, "the floor caps aberration taps at index 9");
    }

    /// Today's chamfer normal: one slope across the whole band.
    fn today(slab: &Slab, p: DVec2) -> DVec3 {
        let g = sd_rounded_box_grad(p - slab.face_center, slab.face_half, slab.face_r);
        let rise = slab.chamfer.min(slab.thickness);
        let slope = rise.hypot(slab.chamfer);
        (g * (rise / slope)).extend(slab.chamfer / slope)
    }

    /// Every sample strictly inside the bevel on a `step` grid.
    fn bevel_samples(slab: &Slab, step: f64) -> Vec<(DVec2, EdgeSample)> {
        let reach = slab.half + DVec2::splat(2.);
        let n = (2. * reach / step).ceil();
        let mut out = Vec::new();
        for i in 0..=n.x as usize {
            for j in 0..=n.y as usize {
                let p = slab.center - reach + DVec2::new(i as f64, j as f64) * step;
                let s = slab.surface(p);
                if s.inner_dist > 0. && s.outer_dist < 0. {
                    out.push((p, s));
                }
            }
        }
        out
    }

    #[test]
    fn profile_one_at_rest_is_todays_bevel_away_from_the_corner_junctions() {
        for radius in [0., 8.] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., radius, 20., 1.);
            let samples = bevel_samples(&slab, 0.1);
            let mut over = 0;
            for (p, s) in &samples {
                let dev = (s.normal - today(&slab, *p)).length();
                assert!(dev <= 0.03, "r {radius} at {p}: {dev}");
                if dev > 0.01 {
                    over += 1;
                }
                // q of the outer box: its zero lines are the arc-to-side junctions.
                let q = p.abs() - slab.half + DVec2::splat(radius + slab.chamfer);
                if q.x.abs().min(q.y.abs()) > 1.5 {
                    assert!(dev <= 0.002, "r {radius} at {p}: {dev} away from a junction");
                }
                if q.x.min(q.y) <= -8. {
                    assert!(dev <= 1e-6, "r {radius} at {p}: {dev} on a straight side");
                }
            }
            assert!(over * 100 <= samples.len() * 3, "r {radius}: {over} of {}", samples.len());
        }
    }

    /// The largest normal change between neighbouring samples along `lines`,
    /// strictly inside the bevel.
    fn max_jump(slab: &Slab, lines: &[(DVec2, DVec2)], step: f64, exact: bool) -> f64 {
        let grad: fn(DVec2, DVec2, Radii) -> DVec2 =
            if exact { sd_rounded_box_grad } else { soft_outer_grad };
        let mut worst = 0f64;
        for &(a, b) in lines {
            let n = ((b - a).length() / step).ceil() as usize;
            let mut prev: Option<DVec3> = None;
            for i in 0..=n {
                let s = slab.surface_with(a + (b - a) * (i as f64 / n as f64), grad);
                if s.inner_dist > 0. && s.outer_dist < 0. {
                    if let Some(prev) = prev {
                        worst = worst.max((s.normal - prev).length());
                    }
                    prev = Some(s.normal);
                } else {
                    prev = None;
                }
            }
        }
        worst
    }

    fn lines(points: &[((f64, f64), (f64, f64))]) -> Vec<(DVec2, DVec2)> {
        points.iter().map(|&(a, b)| (DVec2::from(a), DVec2::from(b))).collect()
    }

    /// Lines through the bottom-right corner of a 112 px slab.
    fn corner_lines() -> Vec<(DVec2, DVec2)> {
        lines(&[
            ((36., 52.), (52., 36.)),
            ((40., 56.), (56., 40.)),
            ((38., 50.), (50., 38.)),
            ((30., 47.), (58., 47.)),
            ((47., 30.), (47., 58.)),
            ((44., 44.), (56., 56.)),
            ((35., 48.5), (50., 48.5)),
            ((30., 41.), (58., 41.)),
            ((41., 30.), (41., 58.)),
        ])
    }

    #[test]
    fn the_normal_is_continuous_through_deformed_corners() {
        // Continuity: the largest neighbour jump falls with the step (0.25 is
        // exactly proportional). The face join at k = 1 is a crease by design
        // and lies at inner_dist = 0, which the samples exclude.
        for radius in [0., 8.] {
            for k in [1., 2.] {
                for shift in [(3., 3.), (3., 0.), (-3., -3.), (-3., 0.), (0., -3.)] {
                    let slab = Slab::at_rest(DVec2::splat(56.), 12., radius, 20., k)
                        .deformed(DVec2::from(shift), DVec2::ZERO);
                    let coarse = max_jump(&slab, &corner_lines(), 0.02, false);
                    let fine = max_jump(&slab, &corner_lines(), 0.005, false);
                    assert!(fine <= 0.4 * coarse, "r {radius} k {k} {shift:?}: {coarse} -> {fine}");
                }
            }
        }
    }

    #[test]
    fn the_exact_outer_gradient_creases_where_the_softened_one_does_not() {
        for k in [1., 2.] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 20., k)
                .deformed(DVec2::new(-3., -3.), DVec2::ZERO);
            let coarse = max_jump(&slab, &corner_lines(), 0.02, true);
            let fine = max_jump(&slab, &corner_lines(), 0.005, true);
            assert!(fine >= 0.9 * coarse && coarse > 0.04, "k {k}: {coarse} -> {fine}");
        }
    }

    #[test]
    fn the_normal_is_continuous_across_a_stadiums_centerlines() {
        let crossings = lines(&[
            ((-5., -97.), (5., -97.)),
            ((-5., 97.), (5., 97.)),
            ((7., -5.), (7., 5.)),
            ((-7., -5.), (-7., 5.)),
            ((-5., -95.5), (5., -95.5)),
            ((-5., -98.5), (5., -98.5)),
        ]);
        for shift in [(0., 0.), (2., 2.), (2., 0.), (0., 2.)] {
            for k in [1., 2.] {
                let slab = Slab::at_rest(DVec2::new(10., 100.), 6., 4., 20., k)
                    .deformed(DVec2::from(shift), DVec2::ZERO);
                let coarse = max_jump(&slab, &crossings, 0.02, false);
                let fine = max_jump(&slab, &crossings, 0.005, false);
                assert!(fine <= 0.3 * coarse, "k {k} {shift:?}: {coarse} -> {fine}");
            }
        }
    }

    /// `soft_outer_grad` in f32, as the shader evaluates it; `None` on the ridge.
    fn soft_outer_grad_f32(p: Vec2, b: Vec2, r: f32) -> Option<Vec2> {
        fn lsp(t: f32) -> f32 {
            if t > 0. {
                return (t + (1. + (-t).exp()).ln()).ln();
            }
            let x = t.exp();
            let ratio = if x < BEVEL_LSP_SMALL as f32 { 1. - 0.5 * x } else { (1. + x).ln() / x };
            t + ratio.ln()
        }
        fn tanhs(t: f32) -> f32 {
            let c = BEVEL_TANH_CLAMP as f32;
            let e = (2. * t.clamp(-c, c)).exp();
            (e - 1.) / (e + 1.)
        }
        let s = BEVEL_OUTER_SOFTEN as f32;
        let q = p.abs() - b + Vec2::splat(r);
        let l = Vec2::new(lsp(q.x / s), lsp(q.y / s));
        let m = l.max_element();
        let c = Vec2::new((l.x - m).exp() * tanhs(p.x / s), (l.y - m).exp() * tanhs(p.y / s));
        assert!(c.is_finite(), "non-finite at p {p}, b {b}, r {r}");
        let len = c.length();
        (len >= BEVEL_RIDGE_EPS as f32).then(|| c / len)
    }

    #[test]
    fn the_softened_gradient_holds_in_f32() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        let (lo, hi) = (2f64.ln(), 4000f64.ln());
        for _ in 0..200_000 {
            let b = DVec2::new((lo + (hi - lo) * next()).exp(), (lo + (hi - lo) * next()).exp());
            let r = next() * b.min_element();
            let p = DVec2::new((next() * 2.4 - 1.2) * b.x, (next() * 2.4 - 1.2) * b.y);
            let wide = soft_outer_grad(p, b, [r; 4]);
            match soft_outer_grad_f32(p.as_vec2(), b.as_vec2(), r as f32) {
                Some(narrow) => {
                    let narrow = narrow.as_dvec2();
                    let angle = narrow.perp_dot(wide).atan2(narrow.dot(wide)).abs();
                    assert!(angle < 0.001, "p {p}, b {b}, r {r}: {angle} rad");
                }
                None => {
                    // Only on the ridge between the long parallel sides.
                    let (short, long) = if b.x < b.y {
                        (p.x, p.y.abs() - (b.y - b.x))
                    } else {
                        (p.y, p.x.abs() - (b.x - b.y))
                    };
                    assert!(short.abs() < 1. && long < 1., "guard off the ridge: p {p}, b {b}, r {r}");
                }
            }
        }
    }

    #[test]
    fn lifted_tap_normals_keep_every_ray_downward() {
        let at_cap = DVec3::new((1f64 - 0.05 * 0.05).sqrt(), 0., 0.05);
        assert!((lift_tap_normal(at_cap) - at_cap).length() < 1e-12, "continuous at n.z = 0.05");
        for z in [0.06f64, 0.3, 0.9, 1.] {
            let n = DVec3::new((1. - z * z).sqrt() * 0.6, (1. - z * z).sqrt() * 0.8, z);
            assert!((lift_tap_normal(n) - n).length() < 1e-12, "n.z {z} is above the cap");
        }
        // Normals swung from straight up, through horizontal, to straight down.
        for ior in [1., 1.28, 3.] {
            for i in 0..=200 {
                let a = PI * f64::from(i) / 200.;
                let n = DVec3::new(a.sin() * 0.6, a.sin() * 0.8, a.cos());
                let lifted = lift_tap_normal(n);
                assert!(lifted.z >= 1. / 401f64.sqrt() - 1e-12);
                let t = ray(lifted, ior);
                assert!(-t.z >= 1. / ior - 1e-12, "ior {ior}, angle {a}: -t.z {}", -t.z);
            }
        }
    }

    #[test]
    fn u_spans_the_bevel_on_both_sides_under_motion() {
        for (shift, resize) in [((3., 0.), (0., 0.)), ((0., 0.), (6., 0.)), ((3., 0.), (-6., 0.))] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 20., 2.)
                .deformed(DVec2::from(shift), DVec2::from(resize));
            for side in [1., -1.] {
                let face = slab.face_center.x + side * slab.face_half.x;
                let rim = side * slab.half.x;
                assert_eq!(slab.surface(DVec2::new(face, 0.)).across, 0., "{shift:?} {resize:?} {side}");
                assert_eq!(slab.surface(DVec2::new(rim, 0.)).across, 1., "{shift:?} {resize:?} {side}");
                let mut last = -1.;
                for i in 0..=1000 {
                    let x = face + (rim - face) * f64::from(i) / 1000.;
                    let u = slab.surface(DVec2::new(x, 0.)).across;
                    assert!(u > last, "{shift:?} {resize:?} {side}: u {u} after {last} at x {x}");
                    last = u;
                }
            }
        }
    }

    #[test]
    fn a_band_narrower_than_a_pixel_is_all_rim() {
        let mut slab = Slab::at_rest(DVec2::splat(56.), 0.75, 0., 20., 2.);
        slab.aa = 1.;
        for x in [55.3, 55.6, 55.9] {
            let s = slab.surface(DVec2::new(x, 0.));
            assert_eq!(s.across, 1.);
            assert!(s.normal.is_finite() && s.normal.z >= 1. / 401f64.sqrt() - 1e-12);
            assert!(s.height.is_finite());
        }
    }

    #[test]
    fn no_chamfer_or_no_thickness_is_a_flat_face() {
        let flat = Slab::at_rest(DVec2::splat(56.), 0., 0., 20., 2.);
        for x in [0., 50., 55.9] {
            let s = flat.surface(DVec2::new(x, 0.));
            assert_eq!((s.normal, s.height, s.across), (DVec3::Z, 20., 0.));
        }
        let thin = Slab::at_rest(DVec2::splat(56.), 12., 0., 0., 2.);
        for x in [0., 50., 55.9] {
            let s = thin.surface(DVec2::new(x, 0.));
            assert_eq!(s.normal, DVec3::Z, "x {x}");
            assert_eq!(s.height, 0.);
            assert_eq!(ray_path(ray(s.normal, IOR), s.height), 0.);
        }
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri bevel::tests`
Expected: every test panics at `todo!()`.

- [ ] **Step 3: Implement.** Replace the `todo!()` bodies; the whole non-test part of the file is:

```rust
//! The glass edge in f64: the two-boundary bevel coordinate, the height-field
//! profile, the softened outer gradient, the bevel normal, and the refracted
//! ray's path (docs/specs/2026-09-30-glass-edge-optics-design.md §3.1).
//! `prelude.frag` mirrors these functions, and a test pins the constants the
//! two share. Coordinates are logical px, y down, +z toward the viewer.

use glam::{DVec2, DVec3};

/// The bevel's slope cap, |grad h| <= 20: the structural normal keeps
/// n.z >= 1 / sqrt(401).
pub const BEVEL_SLOPE_CAP: f64 = 20.;
/// The outer gradient's softening length, logical px.
pub const BEVEL_OUTER_SOFTEN: f64 = 0.5;
/// Below this length the softened gradient sits on the ridge between parallel
/// sides, and the inner gradient stands in.
pub const BEVEL_RIDGE_EPS: f64 = 1e-4;
/// Keeps the profile slope's powers off pow(0, y <= 0), which GLSL leaves
/// undefined.
pub const BEVEL_PROFILE_EPS: f64 = 1e-6;
/// Where log(log(1 + x) / x) switches to its series 1 - x / 2.
pub const BEVEL_LSP_SMALL: f64 = 1e-3;
/// `tanhs` clamps its argument here, so exp never sees more than 20.
pub const BEVEL_TANH_CLAMP: f64 = 10.;
/// The ray path's floor on -t.z: it binds only for aberration taps whose
/// effective index passes 4.
pub const RAY_PATH_FLOOR: f64 = 0.25;
/// Tap normals are lifted to at least this z before normalizing.
pub const TAP_MIN_Z: f64 = 0.05;

/// Corner radii in `CornerRadius` order: top-left, top-right, bottom-right,
/// bottom-left.
pub type Radii = [f64; 4];

/// This quadrant's radius; mirrors `cornerRadius`.
pub fn corner_radius(p: DVec2, r: Radii) -> f64 {
    let top = if p.x < 0. { r[0] } else { r[1] };
    let bottom = if p.x < 0. { r[3] } else { r[2] };
    if p.y < 0. {
        top
    } else {
        bottom
    }
}

/// Signed distance to a rounded box of half-size `b`; mirrors `sdRoundedBox`.
pub fn sd_rounded_box(p: DVec2, b: DVec2, radii: Radii) -> f64 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    q.max_element().min(0.) + q.max(DVec2::ZERO).length() - r
}

/// The exact outward gradient; mirrors `sdRoundedBoxGrad`.
pub fn sd_rounded_box_grad(p: DVec2, b: DVec2, radii: Radii) -> DVec2 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    let g = if q.x > 0. && q.y > 0. {
        q / q.length()
    } else if q.x > q.y {
        DVec2::X
    } else {
        DVec2::Y
    };
    g * DVec2::new(
        if p.x >= 0. { 1. } else { -1. },
        if p.y >= 0. { 1. } else { -1. },
    )
}

/// log(softplus(t)), without overflow for large t or loss for very negative t.
pub fn lsp(t: f64) -> f64 {
    if t > 0. {
        return (t + (1. + (-t).exp()).ln()).ln();
    }
    let x = t.exp();
    let ratio = if x < BEVEL_LSP_SMALL {
        1. - 0.5 * x
    } else {
        (1. + x).ln() / x
    };
    t + ratio.ln()
}

/// tanh with a clamped argument, written with exp as GLSL ES 1.00 must.
pub fn tanhs(t: f64) -> f64 {
    let e = (2. * t.clamp(-BEVEL_TANH_CLAMP, BEVEL_TANH_CLAMP)).exp();
    (e - 1.) / (e + 1.)
}

/// The outer box's softened outward gradient (spec §3.1): softplus for each
/// positive part, `tanhs` for each sign, normalized in the log domain. ZERO on
/// the ridge between parallel sides. Mirrors `softOuterGrad`.
pub fn soft_outer_grad(p: DVec2, b: DVec2, radii: Radii) -> DVec2 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    let l = DVec2::new(lsp(q.x / BEVEL_OUTER_SOFTEN), lsp(q.y / BEVEL_OUTER_SOFTEN));
    let m = l.max_element();
    let c = DVec2::new(
        (l.x - m).exp() * tanhs(p.x / BEVEL_OUTER_SOFTEN),
        (l.y - m).exp() * tanhs(p.y / BEVEL_OUTER_SOFTEN),
    );
    let len = c.length();
    if len < BEVEL_RIDGE_EPS {
        DVec2::ZERO
    } else {
        c / len
    }
}

/// f(u) = 1 - (1 - u^k)^(1/k): 0 at the face edge, 1 at the silhouette.
/// Mirrors `bevelProfile`.
pub fn profile(u: f64, k: f64) -> f64 {
    1. - (1. - u.powf(k)).max(0.).powf(1. / k)
}

/// f'(u) = u^(k-1) (1 - u^k)^(1/k - 1), both bases kept off zero. Mirrors
/// `bevelProfileSlope`.
pub fn profile_slope(u: f64, k: f64) -> f64 {
    let a = u.clamp(BEVEL_PROFILE_EPS, 1.);
    let b = (1. - u.powf(k)).clamp(BEVEL_PROFILE_EPS, 1.);
    a.powf(k - 1.) * b.powf(1. / k - 1.)
}

/// The slab's two boxes: the fixed silhouette and the face that trails the
/// jelly. Radii are taken as given (the shader fits them before this point).
#[derive(Debug, Clone, Copy)]
pub struct Slab {
    pub center: DVec2,
    pub half: DVec2,
    pub outer_r: Radii,
    pub face_center: DVec2,
    pub face_half: DVec2,
    pub face_r: Radii,
    pub chamfer: f64,
    pub thickness: f64,
    /// `bevel-profile`, k.
    pub profile: f64,
    /// One physical pixel in logical px, `1 / scale`.
    pub aa: f64,
}

/// What `slabSurface` returns for one fragment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeSample {
    pub outer_dist: f64,
    pub inner_dist: f64,
    /// Local height h: `thickness` on the face.
    pub height: f64,
    /// u: 0 at the face edge, 1 at the silhouette, 0 on the face.
    pub across: f64,
    /// normalize(grad u): outward across the bevel; ZERO on the face.
    pub across_dir: DVec2,
    /// The structural normal.
    pub normal: DVec3,
}

impl Slab {
    /// Concentric boxes at rest: `outer_r = face_r + chamfer`.
    pub fn at_rest(half: DVec2, chamfer: f64, face_radius: f64, thickness: f64, profile: f64) -> Self {
        Self {
            center: DVec2::ZERO,
            half,
            outer_r: [face_radius + chamfer; 4],
            face_center: DVec2::ZERO,
            face_half: half - DVec2::splat(chamfer),
            face_r: [face_radius; 4],
            chamfer,
            thickness,
            profile,
            aa: 1.,
        }
    }

    /// The face moved by `shift` and resized by `resize` px, as `slabSurface`
    /// applies `mat_jelly_move` and `mat_jelly_resize`.
    pub fn deformed(self, shift: DVec2, resize: DVec2) -> Self {
        Self {
            face_center: self.center + shift,
            face_half: (self.half - DVec2::splat(self.chamfer))
                * (DVec2::ONE + resize / (2. * self.half).max(DVec2::ONE)),
            ..self
        }
    }

    /// Mirrors `slabSurface` from the distances on.
    pub fn surface(&self, p: DVec2) -> EdgeSample {
        self.surface_with(p, soft_outer_grad)
    }

    /// `surface` with the outer gradient supplied: the exact one is the
    /// tests' negative control.
    pub fn surface_with(&self, p: DVec2, outer_grad: fn(DVec2, DVec2, Radii) -> DVec2) -> EdgeSample {
        let d = sd_rounded_box(p - self.center, self.half, self.outer_r);
        let di = sd_rounded_box(p - self.face_center, self.face_half, self.face_r);
        let mut s = EdgeSample {
            outer_dist: d,
            inner_dist: di,
            height: self.thickness,
            across: 0.,
            across_dir: DVec2::ZERO,
            normal: DVec3::Z,
        };
        if !(self.chamfer > 0. && di >= 0.) {
            return s;
        }
        let g_in = sd_rounded_box_grad(p - self.face_center, self.face_half, self.face_r);
        let mut g_out = outer_grad(p - self.center, self.half, self.outer_r);
        if g_out == DVec2::ZERO {
            g_out = g_in;
        }
        let w = di - d;
        let grad_u = if w < self.aa {
            s.across = 1.;
            g_in / w.max(self.aa)
        } else {
            s.across = (di / w).clamp(0., 1.);
            (-d * g_in + di * g_out) / (w * w)
        };
        let rise = self.chamfer.min(self.thickness);
        s.height = self.thickness - rise * profile(s.across, self.profile);
        let mut slope = rise * profile_slope(s.across, self.profile) * grad_u;
        let len = slope.length();
        if len > BEVEL_SLOPE_CAP {
            slope *= BEVEL_SLOPE_CAP / len;
        }
        s.normal = slope.extend(1.).normalize();
        s.across_dir = if grad_u.length() > 0. { grad_u.normalize() } else { g_in };
        s
    }
}

/// GLSL's `refract`.
pub fn refract(i: DVec3, n: DVec3, eta: f64) -> DVec3 {
    let d = n.dot(i);
    let k = 1. - eta * eta * (1. - d * d);
    if k < 0. {
        DVec3::ZERO
    } else {
        eta * i - (eta * d + k.sqrt()) * n
    }
}

/// The orthographic view ray refracted into glass of index `ior`.
pub fn ray(n: DVec3, ior: f64) -> DVec3 {
    refract(DVec3::NEG_Z, n, 1. / ior)
}

/// The refracted ray's length to the backdrop plane under local height `h`;
/// mirrors `rayPath`.
pub fn ray_path(t: DVec3, h: f64) -> f64 {
    h / (-t.z).max(RAY_PATH_FLOOR)
}

/// A tap's normal lifted into the structural cap; mirrors `liftTapNormal`.
pub fn lift_tap_normal(n: DVec3) -> DVec3 {
    DVec3::new(n.x, n.y, n.z.max(TAP_MIN_Z)).normalize()
}

/// One tap's in-plane displacement, as `tap` applies it.
pub fn displacement(n: DVec3, ior: f64, h: f64) -> DVec2 {
    let t = ray(lift_tap_normal(n), ior);
    t.truncate() * ray_path(t, h)
}
```

Note on `thickness 0` (Review Focus 2): `rise = 0`, so the slope is 0 and the normal is `Z`; `height` is 0. On `chamfer 0` the early return leaves the face values.

- [ ] **Step 4: Run the tests**

Run: `just test-one -p niri bevel::tests`
Expected: all 12 pass. If `the_softened_gradient_holds_in_f32` reports a guard off the ridge or an angle at or above 0.001 rad, stop and report it with the printed input: the design's f32 claim (spec §3.1, "Ridge guard") would be wrong, which is a spec question, not a threshold to loosen.

- [ ] **Step 5: Commit**

```bash
just test-fast
tasks done material-ce3229 "bevel.rs: two-boundary u, height-field profile, softened outer gradient, capped normal, ray path and tap lift in f64, with the spec §6 tests"
tasks check
git add src/render_helpers/material/bevel.rs src/render_helpers/material/mod.rs tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): mirror the glass edge geometry and ray in f64 (material-be611b)"
```

---

### Task 3: The frozen-clock edge render harness

Lands before any shader change, so the base renders exist at a commit.

**Files:**
- Create: `src/tests/glass_edge.rs`
- Modify: `src/tests/mod.rs` (add `mod glass_edge;` after `mod focus_swap;`)
- Create: `docs/materials/scripts/glass-edge-compare.py`

**Interfaces:**
- Consumes: `super::ring_pair::{render_at, set_time}` (already `pub(super)`).
- Produces: `GLASS_EDGE_DUMP=<dir>` writes, per case, `<case>.rgba`, `<case>.png`, `<case>.json` (`{"size", "window", "face", "slab", "renderer"}`); cases `stock-off`, `stock-on`, `stock-iridescence`, `live-off`, `live-on`, `live-iridescence`, `live-translucent`, `live-opaque`. Helpers later tasks reuse: `Look` (`STOCK`, `LIVE`), `case`, `cases()`, `config(glass, response, top)`, `fixture`, `open`, `window_rects`, `dump(name, pixels, Option<Look>, window, renderer)`. Later tasks append cases at the end of `cases()`; Task 4 adds the motion test. `glass-edge-compare.py` subcommands `identical` (one case across two dirs), `pair` (two cases in one dir), `predict-face`, `predict-within`, `report`; regions `all`, `window`, `face`, `outside`, `bevel`.

- [ ] **Step 1: Write the harness test**, `src/tests/glass_edge.rs`:

```rust
//! Frozen-clock renders of the glass edge (material-be611b): the instrument
//! for the glass edge optics' intended-change, neutrality and motion evidence
//! (docs/plans/2026-10-02-glass-edge-optics.md).
//!
//! One window over a flat backdrop, rendered through the real material shader
//! by the headless backend's GLES renderer, the way a screenshot renders it
//! (see ring_pair.rs). Set `GLASS_EDGE_DUMP=<dir>` to write each case as
//! `<case>.rgba` (raw RGBA), `<case>.png` and `<case>.json` (output size, the
//! window, face and slab rectangles, the renderer), which
//! docs/materials/scripts/glass-edge-compare.py reads.
//!
//! Each fixture gets the next jelly seed from a process-wide counter, and the
//! aurora reads the seed. Dumps compare across commits because nextest runs
//! every test in its own process, so `every_case_renders_frozen` sees the
//! same seed sequence each time as long as `cases()` only grows at its end.
//! Run it through `just test-one`, never `cargo test`.

use std::time::Duration;

use niri_config::Config;
use smithay::utils::{Logical, Rectangle};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::ring_pair::{diff, render_at, set_time};
use super::*;

const OUT_W: u16 = 1280;
const OUT_H: u16 = 720;
const W: u16 = 640;
const H: u16 = 360;
const REST: Duration = Duration::from_secs(2);
/// The smokes' warm-mid backdrop, rgb(140, 115, 90).
const BACKDROP: &str = "#8c735a";
/// Fully transparent content: every pixel shows the glass.
const CLEAR: u32 = 0x0000_0000;
/// Kitty-like content at 0.6 opacity (premultiplied).
const TRANSLUCENT: u32 = 0x990a_0c0e;
/// Opaque content: the shader must pass it through untouched.
const OPAQUE: u32 = 0xff0a_0c0e;

/// A pinned glass look and the geometry the evidence regions need.
#[derive(Clone, Copy)]
struct Look {
    name: &'static str,
    glass: &'static str,
    bevel: f64,
    offset: (f64, f64),
}

/// The stock glass the smokes pin.
const STOCK: Look = Look {
    name: "stock",
    glass: r##"
                ior 1.5
                thickness 20
                attenuation-color "#dfe8ff"
                attenuation-distance 60
                bevel 12
                offset-x 6
                offset-y 6"##,
    bevel: 12.,
    offset: (6., 6.),
};
/// Pinned values modelled on Prism's focused terminal glass (2026-10-02),
/// without grain or blur. Not the live config: that drifts.
const LIVE: Look = Look {
    name: "live",
    glass: r##"
                ior 1.28
                thickness 31.2
                attenuation-color "#2e3034"
                attenuation-distance 11
                chromatic-aberration 0.36
                roughness 0.24
                bevel 15
                offset-x -6
                offset-y -5"##,
    bevel: 15.,
    offset: (-6., -5.),
};
const RING_OFF: &str = r#"focus "none""#;
const RING_ON: &str = r#"focus "ring-light"
                ring-beam-speed 0"#;
const AURORA_ON: &str = "aurora 0.5 { drift-hz 0; }";

struct Case {
    name: String,
    look: Look,
    /// Glass lines after the look's own.
    extra: String,
    response: &'static str,
    content: u32,
}

fn case(look: Look, suffix: &str, extra: &str, response: &'static str) -> Case {
    Case {
        name: format!("{}-{suffix}", look.name),
        look,
        extra: extra.to_owned(),
        response,
        content: CLEAR,
    }
}

/// Every case the evidence steps dump. Each must parse on the commit that
/// dumps it; later tasks append their cases at the end, never in between.
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for look in [STOCK, LIVE] {
        cases.push(case(look, "off", "", RING_OFF));
        cases.push(case(look, "on", AURORA_ON, RING_ON));
        cases.push(case(look, "iridescence", "iridescence 0.8", RING_OFF));
    }
    cases.push(Case {
        content: TRANSLUCENT,
        ..case(LIVE, "translucent", "", RING_OFF)
    });
    cases.push(Case {
        content: OPAQUE,
        ..case(LIVE, "opaque", "", RING_OFF)
    });
    cases
}

/// The output's config. Distortion is left to the glass lines (the default is
/// 0), so a case can set it without a duplicate node.
fn config(glass: &str, response: &str, top: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        {top}
        layout {{
            gaps 40
            center-focused-column "always"
            background-color "{BACKDROP}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "edge" {{
            glass {{
                {glass}
                backdrop-blur false
                jelly-ripple 0
            }}
            response "default" {{
                accent "none"
                {response}
            }}
        }}
        window-rule {{
            material "edge"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16), content: u32) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(content);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

fn fixture(config: Config) -> Fixture {
    let mut f = Fixture::with_config(config);
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    f
}

/// Every tile's animated window rectangle, in logical px (scale 1).
fn window_rects(f: &mut Fixture) -> Vec<Rectangle<f64, Logical>> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    workspace
        .tiles_with_render_positions()
        .map(|(tile, pos, _)| Rectangle::new(pos + tile.window_loc(), tile.animated_window_size()))
        .collect()
}

/// The face and slab at rest, from the window, as `slabSurface`'s comment
/// derives them: the face is the window narrowed on both axes by
/// 2 * max(|offset-x|, |offset-y|) and moved by the offset; the slab is the
/// face grown by the bevel.
fn face_and_slab(look: Look, w: Rectangle<f64, Logical>) -> ([f64; 4], [f64; 4]) {
    let (ox, oy) = look.offset;
    let m = ox.abs().max(oy.abs());
    let face = [w.loc.x + m + ox, w.loc.y + m + oy, w.size.w - 2. * m, w.size.h - 2. * m];
    let b = look.bevel;
    let slab = [face[0] - b, face[1] - b, face[2] + 2. * b, face[3] + 2. * b];
    (face, slab)
}

fn renderer_name(f: &mut Fixture) -> String {
    f.niri_state()
        .backend
        .with_primary_renderer(|r| {
            r.with_context(|gl| unsafe {
                let name = gl.GetString(smithay::backend::renderer::gles::ffi::RENDERER);
                std::ffi::CStr::from_ptr(name.cast())
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap()
        })
        .unwrap()
}

struct Render {
    pixels: Vec<u8>,
    again: Vec<u8>,
    window: Rectangle<f64, Logical>,
    renderer: String,
}

/// `case` at rest, one focused window, rendered twice in one fixture.
fn render_case(case: &Case) -> Render {
    let glass = format!("{}\n{}", case.look.glass, case.extra);
    let mut f = fixture(config(&glass, case.response, ""));
    let id = f.add_client();
    open(&mut f, id, (W, H), case.content);
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    let pixels = render_at(&mut f, REST);
    let again = render_at(&mut f, REST);
    let window = window_rects(&mut f)[0];
    let renderer = renderer_name(&mut f);
    Render {
        pixels,
        again,
        window,
        renderer,
    }
}

fn rect_json(r: [f64; 4]) -> String {
    format!("[{}, {}, {}, {}]", r[0], r[1], r[2], r[3])
}

fn dump(name: &str, pixels: &[u8], look: Option<Look>, window: Rectangle<f64, Logical>, renderer: &str) {
    let Some(dir) = std::env::var_os("GLASS_EDGE_DUMP") else {
        return;
    };
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(format!("{name}.rgba")), pixels).unwrap();
    let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
    let win = [window.loc.x, window.loc.y, window.size.w, window.size.h];
    let regions = look.map_or(String::new(), |look| {
        let (face, slab) = face_and_slab(look, window);
        format!(r#", "face": {}, "slab": {}"#, rect_json(face), rect_json(slab))
    });
    let json = format!(
        r#"{{"size": [{OUT_W}, {OUT_H}], "window": {}{regions}, "renderer": {renderer:?}}}"#,
        rect_json(win)
    );
    std::fs::write(dir.join(format!("{name}.json")), json).unwrap();
}

#[test]
fn every_case_renders_frozen() {
    for case in cases() {
        let r = render_case(&case);
        let (px, max) = diff(&r.pixels, &r.again);
        assert!(px == 0, "{}: a repeat render differs in {px} px, max {max}", case.name);
        dump(&case.name, &r.pixels, Some(case.look), r.window, &r.renderer);
    }
}
```

- [ ] **Step 2: Run it**

Run: `just test-one -p niri glass_edge`
Expected: `every_case_renders_frozen` passes. If a repeat render differs, report the case: a render that does not repeat at a frozen instant invalidates every comparison below. Open one dumped PNG (Step 4) and confirm the face rectangle in its `.json` lands on the visible face edge, a bevel's width inside the slab's outline.

- [ ] **Step 3: Write the comparison script**, `docs/materials/scripts/glass-edge-compare.py` (executable):

```python
#!/usr/bin/env python3
"""Compare glass edge harness dumps (src/tests/glass_edge.rs) across commits.

    glass-edge-compare.py identical A B CASE... [--region R]
    glass-edge-compare.py pair DIR CASE_A CASE_B --region R --expect same|differ
    glass-edge-compare.py predict-face BEFORE AFTER CASE --ior N
    glass-edge-compare.py predict-within BEFORE AFTER OFF ON --ior N
    glass-edge-compare.py report A B CASE [--region R]

`identical` compares one case across two dump dirs; `pair` compares two cases
in one dir. Regions come from <case>.json: `all`; `window`, inside the window;
`face`, the face rectangle inset by 2 px (the ring band included); `outside`,
farther than 2 px outside the slab rectangle; `bevel`, inside the slab and
outside the face. Values are 8-bit sRGB; predictions run in linear light.

predict-face (spec §6, ring and aurora off): AFTER = (1 - f0) * (BEFORE - S) + S
with S = 0.15 * f0, the face's constant glint. predict-within (ring and aurora
on): AFTER_ON = (1 - f0) * (BEFORE_OFF - S) + S + (BEFORE_ON - BEFORE_OFF).
"Within one code value" is |after - round(prediction)| <= 1: the 8-bit inputs
alone put up to about 1.02 code values between the exact prediction and a
correct render. Exit 1 on a failed check, 2 on bad input. Standard library only.
"""
import json
import math
import pathlib
import sys


def load(directory, case):
    base = pathlib.Path(directory) / case
    meta = json.loads(base.with_suffix(".json").read_text())
    pixels = base.with_suffix(".rgba").read_bytes()
    width, height = meta["size"]
    if len(pixels) != width * height * 4:
        sys.exit(f"{base}.rgba: {len(pixels)} bytes for {width}x{height}")
    return meta, pixels


def inside(rect, px, py, pad=0.0):
    x, y, w, h = rect
    cx, cy = px + 0.5, py + 0.5
    return x - pad <= cx < x + w + pad and y - pad <= cy < y + h + pad


def region(meta, name):
    width, height = meta["size"]
    for py in range(height):
        for px in range(width):
            if name == "all":
                hit = True
            elif name == "window":
                hit = inside(meta["window"], px, py)
            elif name == "face":
                hit = inside(meta["face"], px, py, -2.0)
            elif name == "outside":
                hit = not inside(meta["slab"], px, py, 2.0)
            elif name == "bevel":
                hit = inside(meta["slab"], px, py) and not inside(meta["face"], px, py)
            else:
                sys.exit(f"unknown region {name}")
            if hit:
                yield px, py


def rgb(meta, pixels, px, py):
    i = (py * meta["size"][0] + px) * 4
    return pixels[i:i + 3]


def lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def enc(c):
    c = max(c, 0.0)
    c = c * 12.92 if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
    return c * 255


def f0_of(ior):
    return ((ior - 1) / (ior + 1)) ** 2


def option(args, flag, default=None):
    if flag in args:
        at = args.index(flag)
        return args[at + 1], args[:at] + args[at + 2:]
    return default, args


def check(failures, label, got, want):
    worst = max(abs(g - round(w)) for g, w in zip(got, want))
    if worst > 1:
        failures.append(f"{label}: got {tuple(got)}, predicted {tuple(round(v, 2) for v in want)}")
    return worst


def differing(meta, pa, pb, name):
    return sum(1 for px, py in region(meta, name) if rgb(meta, pa, px, py) != rgb(meta, pb, px, py))


def main(argv):
    if len(argv) < 2:
        sys.exit(__doc__)
    cmd, args = argv[1], argv[2:]
    failures = []
    if cmd == "identical":
        name, args = option(args, "--region", "all")
        a, b, cases = args[0], args[1], args[2:]
        for case in cases:
            meta, pa = load(a, case)
            _, pb = load(b, case)
            diff = differing(meta, pa, pb, name)
            print(f"{case} {name}: {diff} differing pixels")
            if diff:
                failures.append(f"{case} {name}: {diff} pixels differ")
    elif cmd == "pair":
        name, args = option(args, "--region")
        expect, args = option(args, "--expect")
        directory, case_a, case_b = args
        meta, pa = load(directory, case_a)
        _, pb = load(directory, case_b)
        diff = differing(meta, pa, pb, name)
        print(f"{case_a} vs {case_b} {name}: {diff} differing pixels (expect {expect})")
        if (diff == 0) != (expect == "same"):
            failures.append(f"{case_a} vs {case_b} {name}: {diff} differing pixels, expected {expect}")
    elif cmd == "predict-face":
        ior, args = option(args, "--ior")
        before, after, case = args
        f0 = f0_of(float(ior))
        s = 0.15 * f0
        meta, pb = load(before, case)
        _, pa = load(after, case)
        worst = 0
        for px, py in region(meta, "face"):
            want = [enc((1 - f0) * (lin(c) - s) + s) for c in rgb(meta, pb, px, py)]
            worst = max(worst, check(failures, f"{case} ({px},{py})", rgb(meta, pa, px, py), want))
        print(f"{case} face, f0 {f0:.5f}: worst {worst} code values from the rounded prediction")
    elif cmd == "predict-within":
        ior, args = option(args, "--ior")
        before, after, off, on = args
        f0 = f0_of(float(ior))
        s = 0.15 * f0
        meta, b_off = load(before, off)
        _, b_on = load(before, on)
        _, a_on = load(after, on)
        worst = 0
        for px, py in region(meta, "face"):
            t_s = [lin(c) for c in rgb(meta, b_off, px, py)]
            w = [lin(c1) - c0 for c1, c0 in zip(rgb(meta, b_on, px, py), t_s)]
            want = [enc((1 - f0) * (c - s) + s + d) for c, d in zip(t_s, w)]
            worst = max(worst, check(failures, f"{on} ({px},{py})", rgb(meta, a_on, px, py), want))
        print(f"{on} face with ring and aurora, f0 {f0:.5f}: worst {worst} code values from the rounded prediction")
    elif cmd == "report":
        name, args = option(args, "--region", "bevel")
        a, b, case = args
        meta, pa = load(a, case)
        _, pb = load(b, case)
        total, count, worst = 0.0, 0, 0
        for px, py in region(meta, name):
            for ca, cb in zip(rgb(meta, pa, px, py), rgb(meta, pb, px, py)):
                total += (ca - cb) ** 2
                count += 1
                worst = max(worst, abs(ca - cb))
        print(f"{case} {name}: rmse {math.sqrt(total / count):.3f}, max {worst} code values")
    else:
        sys.exit(__doc__)
    for line in failures[:20]:
        print("FAIL", line)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
```

- [ ] **Step 4: Dump the base renders and self-check the script**

```bash
EV=/mnt/ssd3/niri-material/material-be611b/$(date +%Y%m%dT%H%M%S)
echo "$EV" > target/glass-edge-ev   # later tasks read the run stamp from here
tasks note material-be611b "glass edge evidence root: $EV"   # survives a cargo clean
GLASS_EDGE_DUMP=$EV/base just test-one -p niri every_case_renders_frozen
RING_LOOK_DUMP=$EV/ring-look-base just test-one -p niri accepted_ring_look
C=docs/materials/scripts/glass-edge-compare.py
python3 $C identical $EV/base $EV/base stock-off live-on
python3 $C pair $EV/base live-off live-opaque --region window --expect differ
python3 $C pair $EV/base live-off live-opaque --region outside --expect same
```
Expected: the tests pass; `$EV/base` holds 8 × 3 files and `$EV/ring-look-base` the owner's accepted ring look (`ring_look.rs` asks for dumps before and after any change to edge rendering); every script call exits 0.

- [ ] **Step 5: Commit**

```bash
just test-fast
tasks done material-f1b307 "glass_edge.rs frozen-clock harness and glass-edge-compare.py; base renders dumped to <EV>/base"
tasks check
git add src/tests/glass_edge.rs src/tests/mod.rs docs/materials/scripts/glass-edge-compare.py tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "test(material): add the frozen-clock glass edge render harness (material-be611b)"
```

---

### Task 4: Height-field bevel in the shader

**Files:**
- Modify: `src/render_helpers/shaders/material/prelude.frag`
- Modify: `src/render_helpers/shaders/material/main.frag`
- Modify: `src/render_helpers/shaders/mod.rs` (uniform name, assembly tests)
- Modify: `src/render_helpers/material/mod.rs` (upload `mat_bevel_profile`)
- Modify: `src/render_helpers/material/bevel.rs` (shader-line test)
- Modify: `src/tests/glass_edge.rs` (motion test)
- Modify: `docs/materials/render-pipeline.md`, `docs/materials/material-config.md`

**Interfaces:**
- Consumes: `ResolvedGlass::bevel_profile` (Task 1); the `bevel.rs` constants and function shapes (Task 2).
- Produces: `slabSurface(vec2 p, out float coverage, out vec3 normal, out float outerDist, out float innerDist, out float chamferOut, out float height, out float across, out vec2 acrossDir)`; `float rayPath(vec3 t, float h)`; `vec3 liftTapNormal(vec3 n)`; `vec3 tap(vec2 v, vec3 n, float ior, float h)`; in `main()`, locals `surfaceHeight`, `bevelAcross`, `acrossDir`, `fresnel` (now computed before `transmitted`). Task 6 builds `Surface` from these.

- [ ] **Step 1: Write the failing tests.** In `bevel.rs`'s test module:

```rust
    #[test]
    fn the_shader_names_the_same_constants_and_formulas() {
        let frag = include_str!("../shaders/material/prelude.frag");
        for (name, value, literal) in [
            ("BEVEL_SLOPE_CAP", BEVEL_SLOPE_CAP, "20.0"),
            ("BEVEL_OUTER_SOFTEN", BEVEL_OUTER_SOFTEN, "0.5"),
            ("BEVEL_RIDGE_EPS", BEVEL_RIDGE_EPS, "0.0001"),
            ("BEVEL_PROFILE_EPS", BEVEL_PROFILE_EPS, "0.000001"),
            ("BEVEL_LSP_SMALL", BEVEL_LSP_SMALL, "0.001"),
            ("BEVEL_TANH_CLAMP", BEVEL_TANH_CLAMP, "10.0"),
            ("RAY_PATH_FLOOR", RAY_PATH_FLOOR, "0.25"),
            ("TAP_MIN_Z", TAP_MIN_Z, "0.05"),
        ] {
            assert_eq!(literal.parse::<f64>().unwrap(), value, "{name}");
            let line = format!("const float {name} = {literal};");
            assert!(frag.contains(&line), "prelude.frag lacks `{line}`");
        }
        for line in [
            "float w = di - d;",
            "gradU = (-d * gIn + di * gOut) / (w * w);",
            "height = mat_thickness - rise * bevelProfile(across, k);",
            "vec2 slope = rise * bevelProfileSlope(across, k) * gradU;",
            "return h / max(-t.z, RAY_PATH_FLOOR);",
            "return normalize(vec3(n.xy, max(n.z, TAP_MIN_Z)));",
        ] {
            assert!(frag.contains(line), "prelude.frag lacks `{line}`");
        }
    }
```

In `src/render_helpers/shaders/mod.rs`, in `material_source_is_prelude_then_optics_in_order_then_main`, replace
`let attenuation = main.find("vec3 transmitted = sampled * att;").unwrap();` with

```rust
        let fresnel = main.find("float fresnel = f0 + ").unwrap();
        let attenuation = main.find("vec3 transmitted = sampled * att * (1.0 - fresnel);").unwrap();
        assert!(fresnel < attenuation);
        assert!(material_uniform_names()
            .iter()
            .any(|name| name.name == "mat_bevel_profile" && name.type_ == UniformType::_1f));
        assert!(main.contains("spill = mat_sig_focus.x * BEAM_BASE * ringGlow * BEAM_SPILL * moving * (1.0 - bevelAcross);"));
        assert!(!main.contains("innerDist / slabChamfer"));
```

In `src/tests/glass_edge.rs`, add the motion test (Review Focus 3's distortion case rides along):

```rust
const MOTION: &str = r#"animations {
            window-resize { duration-ms 1000; curve "linear"; }
            horizontal-view-movement { duration-ms 1000; curve "linear"; }
        }"#;

/// The live look on a rounded bevel at full flex.
fn motion_glass(distortion: &str) -> String {
    format!("{}\nbevel-profile 2\njelly-flex 0.02\n{distortion}", LIVE.glass)
}

fn px(pixels: &[u8], x: i32, y: i32) -> Option<[u8; 3]> {
    if x < 0 || y < 0 || x >= i32::from(OUT_W) || y >= i32::from(OUT_H) {
        return None;
    }
    let i = ((y * i32::from(OUT_W) + x) * 4) as usize;
    Some([pixels[i], pixels[i + 1], pixels[i + 2]])
}

/// On each side of `rect` whose middle lies on the output, scanning outward
/// from 20 px inside the window along its middle row or column to the first
/// three backdrop pixels: the three pixels just inside that outline (the
/// anti-aliased pixel excluded) must not be one flat colour. A bevel
/// coordinate that saturates before the silhouette, or a planar facet, leaves
/// such a run. Returns the sides it checked.
fn assert_no_rim_plateau(tag: &str, pixels: &[u8], rect: Rectangle<f64, Logical>) -> Vec<&'static str> {
    let backdrop = px(pixels, 2, 2).unwrap();
    let cx = (rect.loc.x + rect.size.w / 2.) as i32;
    let cy = (rect.loc.y + rect.size.h / 2.) as i32;
    let sides = [
        ("left", (rect.loc.x as i32 + 20, cy), (-1, 0)),
        ("right", ((rect.loc.x + rect.size.w) as i32 - 20, cy), (1, 0)),
        ("top", (cx, rect.loc.y as i32 + 20), (0, -1)),
        ("bottom", (cx, (rect.loc.y + rect.size.h) as i32 - 20), (0, 1)),
    ];
    let mut checked = Vec::new();
    for (side, (x0, y0), (dx, dy)) in sides {
        let at = |i: i32| px(pixels, x0 + dx * i, y0 + dy * i);
        if at(0).is_none() {
            continue;
        }
        let outline = (0..)
            .take_while(|&i| at(i + 2).is_some())
            .find(|&i| (0..3).all(|j| at(i + j) == Some(backdrop)))
            .unwrap_or_else(|| panic!("{tag} {side}: no outline before the output's edge"));
        let run = [at(outline - 4), at(outline - 3), at(outline - 2)];
        assert!(
            !(run[0] == run[1] && run[1] == run[2]),
            "{tag} {side}: flat run {run:?} before the outline at {outline}"
        );
        checked.push(side);
    }
    checked
}

#[test]
fn the_rounded_rim_has_no_plateau_at_rest_mid_resize_or_mid_scroll() {
    let all = ["left", "right", "top", "bottom"];
    for (tag, distortion) in [("plain", ""), ("distorted", "distortion 1 scale=0.5")] {
        // One centered window, its column 200 px wider, half way through the
        // resize.
        let mut f = fixture(config(&motion_glass(distortion), RING_OFF, MOTION));
        let id = f.add_client();
        let surface = open(&mut f, id, (W, H), CLEAR);
        f.niri_state().update_keyboard_focus();
        f.double_roundtrip(id);
        set_time(&mut f, Duration::ZERO);
        f.niri_complete_animations();
        let rest = render_at(&mut f, Duration::ZERO);
        let rect = window_rects(&mut f)[0];
        dump(&format!("motion-{tag}-rest"), &rest, None, rect, "");
        assert_eq!(assert_no_rim_plateau("rest", &rest, rect), all);

        f.niri().layout.set_column_width(niri_ipc::SizeChange::AdjustFixed(200));
        f.double_roundtrip(id);
        let window = f.client(id).window(&surface);
        window.attach_new_shm_buffer(CLEAR);
        window.set_size(W + 200, H);
        window.ack_last_and_commit();
        f.roundtrip(id);
        let mid = render_at(&mut f, Duration::from_millis(500));
        let rect = window_rects(&mut f)[0];
        dump(&format!("motion-{tag}-resize-mid"), &mid, None, rect, "");
        assert_eq!(assert_no_rim_plateau("mid-resize", &mid, rect), all);

        // Two 800 px columns; focusing the second scrolls the view, so at
        // 500 ms the first tile's right edge trails and the second's left
        // edge leads, both on the output.
        let mut f = fixture(config(&motion_glass(distortion), RING_OFF, MOTION));
        let id = f.add_client();
        let first = open(&mut f, id, (W, H), CLEAR);
        let second = open(&mut f, id, (W, H), CLEAR);
        // The newest window has focus: widen its column, then the first's,
        // each client committing the new width as mid_resize_fixture does.
        for surface in [&second, &first] {
            f.niri().layout.set_column_width(niri_ipc::SizeChange::SetFixed(800));
            f.double_roundtrip(id);
            let window = f.client(id).window(surface);
            window.attach_new_shm_buffer(CLEAR);
            window.set_size(800, H);
            window.ack_last_and_commit();
            f.double_roundtrip(id);
            f.niri().layout.focus_left();
        }
        f.niri_state().update_keyboard_focus();
        f.double_roundtrip(id);
        set_time(&mut f, Duration::ZERO);
        f.niri_complete_animations();
        let _ = render_at(&mut f, Duration::ZERO);
        f.niri().layout.focus_right();
        let mid = render_at(&mut f, Duration::from_millis(500));
        let mut checked = Vec::new();
        for (i, rect) in window_rects(&mut f).into_iter().enumerate() {
            dump(&format!("motion-{tag}-scroll-mid-{i}"), &mid, None, rect, "");
            checked.extend(assert_no_rim_plateau(&format!("mid-scroll {i}"), &mid, rect));
        }
        assert!(
            checked.contains(&"left") && checked.contains(&"right"),
            "mid-scroll checked only {checked:?}: no leading and trailing edge on the output"
        );
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri the_shader_names_the_same` then `just test-one -p niri material_source_is_prelude` then `just test-one -p niri the_rounded_rim`
Expected: all three fail. The first two fail on the missing lines. The motion test fails at `rest` on the old shader, which ignores `bevel-profile` and draws a planar facet: one flat colour up to the outline. If it fails anywhere else (no outline, fewer sides), fix the harness before touching the shader.

- [ ] **Step 3: Implement the prelude.** In `prelude.frag`:

After `uniform float mat_light_ior;` add `uniform float mat_bevel_profile;`.

After the slab globals (`vec4 g_face_r;`), add:

```glsl
// Glass edge constants: bevel.rs holds the same values under the same names,
// and a test there checks these lines (spec 2026-09-30-glass-edge-optics §3.1).
const float BEVEL_SLOPE_CAP = 20.0;
const float BEVEL_OUTER_SOFTEN = 0.5;
const float BEVEL_RIDGE_EPS = 0.0001;
const float BEVEL_PROFILE_EPS = 0.000001;
const float BEVEL_LSP_SMALL = 0.001;
const float BEVEL_TANH_CLAMP = 10.0;
const float RAY_PATH_FLOOR = 0.25;
const float TAP_MIN_Z = 0.05;
```

After `sdRoundedBoxGrad`, add:

```glsl
// log(softplus(t)), without overflow for large t or loss for very negative t.
float lsp(float t) {
    if (t > 0.0)
        return log(t + log(1.0 + exp(-t)));
    float x = exp(t);
    float ratio = x < BEVEL_LSP_SMALL ? 1.0 - 0.5 * x : log(1.0 + x) / x;
    return t + log(ratio);
}

// tanh with a clamped argument: GLSL ES 1.00 has no tanh.
float tanhs(float t) {
    float e = exp(2.0 * clamp(t, -BEVEL_TANH_CLAMP, BEVEL_TANH_CLAMP));
    return (e - 1.0) / (e + 1.0);
}

// The outer box's softened outward gradient: softplus for each positive part,
// tanhs for each sign, normalized in the log domain. vec2(0.0) on the ridge
// between parallel sides, where the caller uses the inner gradient.
vec2 softOuterGrad(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
    vec2 q = abs(p) - b + vec2(r);
    vec2 l = vec2(lsp(q.x / BEVEL_OUTER_SOFTEN), lsp(q.y / BEVEL_OUTER_SOFTEN));
    float m = max(l.x, l.y);
    vec2 c = vec2(exp(l.x - m) * tanhs(p.x / BEVEL_OUTER_SOFTEN),
                  exp(l.y - m) * tanhs(p.y / BEVEL_OUTER_SOFTEN));
    float len = length(c);
    return len < BEVEL_RIDGE_EPS ? vec2(0.0) : c / len;
}

// f(u) = 1 - (1 - u^k)^(1/k): 0 at the face edge, 1 at the silhouette.
float bevelProfile(float u, float k) {
    return 1.0 - pow(max(1.0 - pow(u, k), 0.0), 1.0 / k);
}

// f'(u), both bases kept off zero: GLSL leaves pow(0, y <= 0) undefined.
float bevelProfileSlope(float u, float k) {
    float a = clamp(u, BEVEL_PROFILE_EPS, 1.0);
    float b = clamp(1.0 - pow(u, k), BEVEL_PROFILE_EPS, 1.0);
    return pow(a, k - 1.0) * pow(b, 1.0 / k - 1.0);
}
```

Replace `slabSurface`'s signature, its doc comment's last sentence, and its final `if` block:

```glsl
// The fragment-faked slab at an element-local logical-px position:
// coverage in [0, 1], the structural normal (y-down, +z toward the viewer),
// the local height, and the across-bevel coordinate u with its direction.
// The outer silhouette is the slab back and stays fixed; the face trails
// the jelly. The bevel is a height field measured against both boundaries
// (spec 2026-09-30-glass-edge-optics §3.1); bevel.rs mirrors it.
void slabSurface(vec2 p, out float coverage, out vec3 normal, out float outerDist,
                 out float innerDist, out float chamferOut, out float height,
                 out float across, out vec2 acrossDir) {
```

and, in place of the existing `if (chamfer > 0.0 && di >= 0.0) { ... } else { ... }`:

```glsl
    height = mat_thickness;
    across = 0.0;
    acrossDir = vec2(0.0);
    normal = vec3(0.0, 0.0, 1.0);
    if (chamfer > 0.0 && di >= 0.0) {
        vec2 gIn = sdRoundedBoxGrad(p - inner_center, inner_half, inner_r);
        vec2 gOut = softOuterGrad(p - center, half_ext, outer_r);
        if (gOut == vec2(0.0))
            gOut = gIn;
        float w = di - d;
        vec2 gradU;
        if (w < aa) {
            // Narrower than a physical pixel: rim.
            across = 1.0;
            gradU = gIn / max(w, aa);
        } else {
            across = clamp(di / w, 0.0, 1.0);
            gradU = (-d * gIn + di * gOut) / (w * w);
        }
        float rise = min(chamfer, mat_thickness);
        float k = mat_bevel_profile;
        height = mat_thickness - rise * bevelProfile(across, k);
        vec2 slope = rise * bevelProfileSlope(across, k) * gradU;
        float s = length(slope);
        if (s > BEVEL_SLOPE_CAP)
            slope *= BEVEL_SLOPE_CAP / s;
        normal = normalize(vec3(slope, 1.0));
        acrossDir = length(gradU) > 0.0 ? normalize(gradU) : gIn;
    }
}
```

Replace `tap` with:

```glsl
// The refracted ray's length to the backdrop plane under local height h
// (spec §3.1 "Ray model"). On the face, t = (0, 0, -1) and L = h.
float rayPath(vec3 t, float h) {
    return h / max(-t.z, RAY_PATH_FLOOR);
}

// Distortion and ripple can tip a tap's normal past horizontal; lift it into
// the structural cap so every refracted ray points down.
vec3 liftTapNormal(vec3 n) {
    return normalize(vec3(n.xy, max(n.z, TAP_MIN_Z)));
}

// One refraction tap: bend the orthographic ray at the lifted normal, follow
// it to the backdrop plane under height h, and sample the composed background
// there. Offsets are logical px mapped through the element-UV frame; the
// element and the background buffers share the y-down orientation.
vec3 tap(vec2 v, vec3 n, float ior, float h) {
    vec3 refr = refract(vec3(0.0, 0.0, -1.0), liftTapNormal(n), 1.0 / ior);
    vec2 vv = v + (refr.xy * rayPath(refr, h)) / mat_area_size;
    return srgbToLinear(sampleBackground(vv));
}
```

- [ ] **Step 4: Implement `main.frag`.** The slab call becomes:

```glsl
    float coverage;
    vec3 surfaceNormal;
    float slabDist;
    float innerDist;
    float slabChamfer;
    float surfaceHeight;
    float bevelAcross;
    vec2 acrossDir;
    slabSurface(p, coverage, surfaceNormal, slabDist, innerDist, slabChamfer,
                surfaceHeight, bevelAcross, acrossDir);
```

The taps: `sampled = tap(v, n, mat_ior, surfaceHeight);` in the single-tap branch; in the loop, delete `float smear = mat_thickness * mat_anisotropic_blur;` and replace `float t = mat_thickness + smear * (fi + r1) / count;` with

```glsl
                // Today's smear rule applied to the local height.
                float t = surfaceHeight * (1.0 + mat_anisotropic_blur * (fi + r1) / count);
```

The attenuation block becomes:

```glsl
        // Beer-Lambert along the structural ray to the backdrop plane
        // (spec §3.1): the face path is the thickness exactly; on the bevel
        // the glass thins. Distortion moves taps but does not lengthen it.
        float surfaceCosine = clamp(surfaceNormal.z, 0.0, 1.0);
        vec3 structuralRay = refract(vec3(0.0, 0.0, -1.0), surfaceNormal, 1.0 / mat_ior);
        float opticalDistance = rayPath(structuralRay, surfaceHeight);
        vec3 att = pow(clamp(mat_attenuation_color.rgb, vec3(0.001), vec3(1.0)),
                       vec3(opticalDistance / mat_attenuation_distance));
        // Schlick from the configured IOR on the structural normal. What the
        // surface reflects is not transmitted (spec §3.2).
        float f0 = (mat_ior - 1.0) / (mat_ior + 1.0);
        f0 = f0 * f0;
        float fresnel = f0 + (1.0 - f0) * pow(1.0 - surfaceCosine, 5.0);
        vec3 transmitted = sampled * att * (1.0 - fresnel);
```

In the spill, replace

```glsl
                    float across = clamp(innerDist / slabChamfer, 0.0, 1.0);
                    spill = mat_sig_focus.x * BEAM_BASE * ringGlow * BEAM_SPILL * moving * (1.0 - across);
```

with

```glsl
                    spill = mat_sig_focus.x * BEAM_BASE * ringGlow * BEAM_SPILL * moving * (1.0 - bevelAcross);
```

and update the comment above it to "falling off toward the silhouette along u". In the glint block, delete the now-duplicate `float f0 = ...; f0 = f0 * f0; float fresnel = ...;` lines and change its comment's first line to "Fresnel edge glint, on the Schlick term computed with the attenuation.".

- [ ] **Step 5: Upload the uniform.** In `src/render_helpers/shaders/mod.rs` `material_uniform_names`, after `UniformName::new("mat_light_ior", UniformType::_1f),` add `UniformName::new("mat_bevel_profile", UniformType::_1f),`. In `src/render_helpers/material/mod.rs`, after `Uniform::new("mat_light_ior", g.light_ior as f32),` add `Uniform::new("mat_bevel_profile", g.bevel_profile as f32),`.

- [ ] **Step 6: Validate the shader offline**

```bash
{ echo '#version 100'; cat src/render_helpers/shaders/material/prelude.frag
  for o in saturation noise aurora iridescence; do cat src/render_helpers/shaders/material/$o.frag; done
  cat src/render_helpers/shaders/material/main.frag; } > target/material.frag
glslangValidator -S frag target/material.frag
```
Expected: exit 0, no errors. (The optic list follows `OPTICS`; Tasks 7 and 8 extend it.)

- [ ] **Step 7: Run the tests**

Run: `just test-one -p niri bevel::tests`, `just test-one -p niri material_source_is_prelude`, `just test-one -p niri glass_edge`, then `just test-one -p niri ring_` (the ring render tests must still pass: `ring_cap_keeps_one_core` checks the cap table, which holds at `bevel-profile 1` because the normal on straight sides is unchanged).
Expected: all pass. If a `ring_` test fails, stop and report: the spill or the ring's attenuation changed on the face, which §3.2 rules out.

- [ ] **Step 8: Documentation.** In `docs/materials/render-pipeline.md` stage table:
  - row 1: "Signed distance to the slab silhouette gives anti-aliased coverage. The bevel is a height field between two boundaries, the fixed silhouette and the face that trails the jelly shear and resize: `u = innerDist / (innerDist - outerDist)` runs from 0 at the face edge to 1 at the silhouette on every side, the local height is `thickness - R * f(u)` with `R = min(bevel, thickness)` and `f(u) = 1 - (1 - u^k)^(1/k)`, and the structural normal comes from that height field, its slope capped at 20. `k = 1` is a planar chamfer." Parameters gain `bevel-profile`.
  - row 3: "Refract the orthographic ray at the perturbed normal, lifted to `n.z >= 0.05`, with `ior`, follow it to the backdrop plane under the local height (`L = h / max(-t.z, 0.25)`), and sample the composed background there."
  - row 4: "`attenuation-color ^ (L / attenuation-distance)`, with `L` the structural ray's path under the local height: the face's path is `thickness`; the bevel thins toward the silhouette. Transmitted light is then scaled by `1 - F` (Schlick, stage 6)."
  - row 5: "edge spill on the bevel, fading from the face edge to the silhouette along `u`".
  - row 8: "`glass = linearToSrgb((1 - F) * sampled * att + within + specular + emissive)`".
  - §5 table: add `| bevel-profile | glass.bevelProfile (pending, prism-7024c4) | 1, 3, 4 |` after the `bevel` row.

  In `docs/materials/material-config.md`, after the `light-ior` paragraphs, add:

  ```markdown
  `bevel-profile` shapes the bevel between the face and the silhouette:
  `1` is a planar chamfer, `2` a quarter-round, and larger values a squircle
  that stays flat longer and rolls off harder. The bevel is a height field: the
  glass thins from `thickness` at the face edge to `thickness - min(bevel,
  thickness)` at the silhouette, refraction and attenuation follow the refracted
  ray to the backdrop plane under that local height, and the reflected share
  `F` of the light is no longer transmitted (`docs/specs/2026-09-30-glass-edge-optics-design.md`).
  At `bevel-profile 1` on a straight side the normal, and so the ring cap table
  above, is unchanged; for larger values the tilt varies across the bevel.
  ```

- [ ] **Step 9: Commit**

```bash
just test-fast
tasks done material-d37c1a "height-field bevel in the shader: two-boundary u, profile, softened outer gradient, ray path for taps and attenuation, tap lift, (1 - F), spill on u; mirror test pins prelude lines; motion renders show no rim plateau"
tasks check
git add src/render_helpers docs/materials/render-pipeline.md docs/materials/material-config.md src/tests/glass_edge.rs tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): render the glass bevel as a height field (material-be611b)"
```

---

### Task 5: Intended-change evidence

**Files:**
- Create: `docs/materials/<run-date>-glass-edge-optics-evidence.md`

**Interfaces:**
- Consumes: `$EV/base` (Task 3), the harness and the compare script.

- [ ] **Step 1: Dump the Task 4 renders**

```bash
EV=$(cat target/glass-edge-ev)   # or the root in material-be611b's "evidence root" note
GLASS_EDGE_DUMP=$EV/step1 just test-one -p niri every_case_renders_frozen
RING_LOOK_DUMP=$EV/ring-look-step1 just test-one -p niri accepted_ring_look
```
Expected: both pass; `$EV/step1` holds the eight cases and `$EV/ring-look-step1` the ring look after the change.

- [ ] **Step 2: Run the checks** (each must exit 0; record every line of output):

```bash
C=docs/materials/scripts/glass-edge-compare.py
ALL="stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque"
python3 $C identical $EV/base $EV/step1 $ALL --region outside
python3 $C identical $EV/base $EV/step1 live-opaque --region window
python3 $C predict-face $EV/base $EV/step1 stock-off --ior 1.5
python3 $C predict-face $EV/base $EV/step1 live-off --ior 1.28
python3 $C predict-within $EV/base $EV/step1 stock-off stock-on --ior 1.5
python3 $C predict-within $EV/base $EV/step1 live-off live-on --ior 1.28
for c in stock-off live-off stock-on live-on; do python3 $C report $EV/base $EV/step1 $c --region bevel; done
python3 $C report $EV/base $EV/step1 live-translucent --region face
```
Expected: nothing outside the slab and no opaque window pixel changes; both face predictions and both within predictions hold (worst at most 1 code value from the rounded prediction; the face region includes the ring band, so the within check shows the ring's face attenuation is unchanged); bevel and translucent-face deltas reported, not asserted. If a prediction fails, stop and report the printed pixels: §4 says face attenuation and the ring's face factor are exact, so a failure means the implementation and the spec disagree. Compare `$EV/ring-look-base` with `$EV/ring-look-step1` by eye and describe the edge change.

- [ ] **Step 3: Write the evidence document** `docs/materials/<run-date>-glass-edge-optics-evidence.md`: the commit, renderer string (from any `.json`; if it is llvmpipe, say so: the shader then first compiles on the real driver in Task 9, whose sheet is that check), the `$EV` path, the ring-look comparison, each command and its output verbatim, and a short reading: what changed on the bevel (from the bevel reports and by eye from the PNGs), and the translucent-face delta set against Review Focus 4. Include a "Step 1" heading; Tasks 6 to 9 append theirs.

- [ ] **Step 4: Commit**

```bash
tasks done material-c210a0 "step 1 evidence: outside identical, face and within predictions within one code value, bevel deltas reported"
tasks check
git add docs/materials/*-glass-edge-optics-evidence.md tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "docs(material): record the glass edge step 1 evidence (material-be611b)"
```

---

### Task 6: The `Surface` specular hook

**Files:**
- Modify: `src/render_helpers/shaders/material/prelude.frag` (the struct)
- Modify: `src/render_helpers/shaders/material/main.frag` (build it, pass it)
- Modify: `src/render_helpers/shaders/material/iridescence.frag`
- Modify: `src/render_helpers/shaders/mod.rs` (assembly test)
- Modify: `docs/materials/adding-an-optic.md`, `docs/materials/render-pipeline.md`

**Interfaces:**
- Produces: `struct Surface { vec2 p; vec2 v; vec3 structural; vec3 perturbed; float cosine; float fresnel; float across; vec2 acrossDir; float outerDist; }`; the specular hook signature `vec3 <name>_specular(vec3 specular, Surface s)`; in `main()`, `Surface surf`.

- [ ] **Step 1: Dump the pre-change renders.** `GLASS_EDGE_DUMP=$EV/step2-before just test-one -p niri every_case_renders_frozen` (with `EV=$(cat target/glass-edge-ev)`).

- [ ] **Step 2: Write the failing assertion.** In `material_source_is_prelude_then_optics_in_order_then_main`, add:

```rust
        assert!(source.contains("struct Surface {"));
        assert!(source.contains("vec3 iridescence_specular(vec3 specular, Surface s)"));
        assert!(main.contains("Surface surf = Surface(p, v, surfaceNormal, n, surfaceCosine, fresnel, bevelAcross, acrossDir, slabDist);"));
        assert!(main.contains("specular = iridescence_specular(specular, surf);"));
```

Run: `just test-one -p niri material_source_is_prelude` — Expected: FAIL on `struct Surface {`.

- [ ] **Step 3: Implement.** In `prelude.frag`, after the slab globals and the bevel constants:

```glsl
// What a specular hook sees of the surface at this fragment (spec §3.4).
struct Surface {
    vec2 p;            // element-local logical px
    vec2 v;            // element UV
    vec3 structural;   // the slab's normal
    vec3 perturbed;    // after distortion and jelly ripple
    float cosine;      // structural.z, clamped to [0, 1]
    float fresnel;     // Schlick at the structural normal
    float across;      // u: 0 at the face edge, 1 at the silhouette, 0 on the face
    vec2 acrossDir;    // normalize(grad u), outward across the bevel
    float outerDist;   // signed distance to the silhouette, negative inside
};
```

`iridescence.frag`'s function becomes:

```glsl
vec3 iridescence_specular(vec3 specular, Surface s) {
    if (mat_iridescence <= 0.0)
        return specular;
    const float TAU = 6.28318530718;
    float hue = fract(2.5 * (1.0 - s.cosine));
    vec3 palette = 0.5 + 0.5 * cos(TAU * (hue + vec3(0.0, 1.0 / 3.0, 2.0 / 3.0)));
    return mix(specular, specular * palette * 2.0, mat_iridescence);
}
```

In `main.frag`, before the specular hooks:

```glsl
        Surface surf = Surface(p, v, surfaceNormal, n, surfaceCosine, fresnel, bevelAcross, acrossDir, slabDist);
        // Specular hooks, in OPTICS order (render-pipeline.md stage 6).
        specular = iridescence_specular(specular, surf);
```

- [ ] **Step 4: Validate and test.** Run the Task 4 Step 6 shader command (exit 0), then `just test-one -p niri material_source_is_prelude` and `just test-one -p niri iridescence`.

- [ ] **Step 5: Neutrality evidence.** `GLASS_EDGE_DUMP=$EV/step2 just test-one -p niri every_case_renders_frozen`, then `python3 docs/materials/scripts/glass-edge-compare.py identical $EV/step2-before $EV/step2 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque`. Expected: 0 differing pixels in every case (spec §6: the step 2 build is decoded-identical to step 1 across iridescence). Append the output under a "Step 2" heading in the evidence document.

- [ ] **Step 6: Documentation.** In `adding-an-optic.md`'s hook table, the `specular` row becomes `` `vec3 <name>_specular(vec3 specular, Surface s)` `` with neutral `specular`, and a paragraph below the table lists `Surface`'s fields as in the struct comment. In `render-pipeline.md` row 6, add "Specular hooks take the fragment's `Surface` (position, both normals, cosine, Fresnel, `u` and its direction, silhouette distance)."

- [ ] **Step 7: Commit**

```bash
just test-fast
tasks done material-d63184 "Surface specular hook; iridescence migrated, renders decoded-identical"
tasks check
git add src/render_helpers docs/materials tasks/
git commit -m "refactor(material): pass a Surface to the specular hooks (material-be611b)"
```

---

### Task 7: The `reflection` optic

**Files:**
- Create: `niri-config/src/material/optics/reflection.rs`
- Modify: `niri-config/src/material/optics/mod.rs`, `niri-config/src/material/mod.rs`, `niri-config/src/lib.rs`
- Create: `src/render_helpers/material/optics/reflection.rs`, `src/render_helpers/shaders/material/reflection.frag`
- Modify: `src/render_helpers/material/optics/mod.rs`, `src/render_helpers/shaders/material/main.frag`, `src/render_helpers/shaders/mod.rs`
- Modify: `src/tests/glass_edge.rs` (cases)
- Modify: `docs/materials/material-config.md`, `docs/materials/render-pipeline.md`, `docs/materials/scripts/glass-parameter-sweep.sh`

**Interfaces:**
- Produces: `niri_config::ResolvedReflection { amount: f64 }`, `ResolvedGlass::reflection`; KDL `reflection <0..1>`; `ReflectionOptic` (`NAME = "reflection"`, uniform `mat_reflection`); GLSL `vec3 reflection_specular(vec3 specular, Surface s)`.

- [ ] **Step 1: Dump the pre-change renders** to `$EV/step3-before`, as in Task 6 Step 1.

- [ ] **Step 2: Write the failing tests.** In `niri-config/src/lib.rs`:

```rust
    #[test]
    fn reflection_resolves_through_its_optic() {
        let written = do_parse(r##"material "edge" { glass { reflection 0.6; }; }"##);
        assert_eq!(written.materials[0].resolve().glass.reflection, ResolvedReflection { amount: 0.6 });
        let omitted = do_parse(r##"material "edge" { glass {}; }"##);
        assert_eq!(omitted.materials[0].resolve().glass.reflection.amount, 0.);
    }

    #[test]
    fn glass_reflection_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!("material \"edge\" {{ glass {{ reflection {value}; }}; }}\n"));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }
```

In `src/render_helpers/material/optics/reflection.rs` (test module, with the `frame` helper copied from `iridescence.rs`, `logical_now: Duration::ZERO`):

```rust
    #[test]
    fn the_amount_is_the_only_uniform_and_the_optic_is_static() {
        let glass = ResolvedGlass {
            reflection: ResolvedReflection { amount: 0.6 },
            ..Default::default()
        };
        let blur = Blur::default();
        let values = ReflectionOptic::values(&glass, &frame(&blur));
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "mat_reflection");
        assert_eq!(values[0].value, UniformValue::_1f(0.6));
        assert_eq!(ReflectionOptic::next_change(&glass, &frame(&blur)), None);
    }

    #[test]
    fn reflection_guards_a_cancelled_direction() {
        // Review Focus 3: a perturbation can cancel the across-bevel direction.
        let glsl = ReflectionOptic::GLSL;
        assert!(glsl.contains("vec2 dir = lb > 0.0001 ? bent / lb : s.acrossDir;"));
        assert!(glsl.contains("if (mat_reflection <= 0.0)\n        return specular;"));
    }
```

(the tests module sees `ReflectionOptic` through `use super::*`). In `shaders/mod.rs`, extend the marker list to `"// ---- optic: saturation"`, `"noise"`, `"aurora"`, `"reflection"`, `"iridescence"`, `"// ---- main"` and add `assert!(main.contains("specular = reflection_specular(specular, surf);"));`. In `glass_edge.rs` `cases()`, append:

```rust
    for look in [STOCK, LIVE] {
        cases.push(case(look, "reflection-0", "reflection 0", RING_OFF));
        cases.push(case(look, "reflection", "reflection 0.6", RING_OFF));
    }
```

Run: `just test-one -p niri-config reflection` — Expected: compile failure, no `ResolvedReflection`.

- [ ] **Step 3: Implement the config side.** `niri-config/src/material/optics/reflection.rs`:

```rust
//! `reflection <amount>`: stage 6, the scene just beyond the silhouette,
//! reflected by the bevel and untinted. Neutral at amount 0.

use crate::material::params::{ParamKind, ParamSpec};
use crate::FloatOrInt;

/// The node's scalar type; the field on `Glass` uses it.
pub type Reflection = FloatOrInt<0, 1>;

/// Final reflection state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedReflection {
    /// Gain on the Fresnel-weighted reflection; 0 is none.
    pub amount: f64,
}

pub fn resolve(node: Option<Reflection>) -> ResolvedReflection {
    ResolvedReflection {
        amount: node.map_or(ResolvedReflection::default().amount, |x| x.0),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "reflection",
        kind: ParamKind::float::<Reflection>(ResolvedReflection::default().amount, "—"),
        write: |v| format!("reflection {v}"),
        read: Some(|g| Some(g.reflection.amount)),
    }]
}
```

In `optics/mod.rs`: `pub mod reflection;`, `ORDER = &["saturation", "noise", "aurora", "reflection", "iridescence"]`, and `specs.extend(reflection::params());` before `iridescence::params()`. In `material/mod.rs`: `Glass` gains `#[knuffel(child, unwrap(argument))] pub reflection: Option<optics::reflection::Reflection>,` after `iridescence`; `ResolvedGlass` gains `pub reflection: optics::reflection::ResolvedReflection,` after `iridescence`, defaulted with `optics::reflection::ResolvedReflection::default()` and resolved with `reflection: optics::reflection::resolve(g.reflection),`. In `lib.rs`: `pub use crate::material::optics::reflection::{Reflection, ResolvedReflection};` beside the iridescence re-export, and `reflection: ResolvedReflection::default(),` in the exhaustive literal.

- [ ] **Step 4: Implement the renderer side.** `src/render_helpers/shaders/material/reflection.frag`:

```glsl
// Optic: reflection (render-pipeline.md stage 6). Neutral at amount 0.
uniform float mat_reflection;

// What a pane edge reflects is the room around it; the nearest stand-in is
// the scene just beyond the silhouette, sampled outward along the
// across-bevel direction bent by the perturbation, one thickness further.
// Untinted: reflection happens at the surface, before light enters the
// glass (spec §3.2). Zero on the face, where the direction is undefined.
vec3 reflection_specular(vec3 specular, Surface s) {
    if (mat_reflection <= 0.0)
        return specular;
    float wr = mat_reflection * smoothstep(0.0, 0.1, s.across);
    if (wr <= 0.0)
        return specular;
    vec2 bent = s.acrossDir + s.perturbed.xy - s.structural.xy;
    float lb = length(bent);
    vec2 dir = lb > 0.0001 ? bent / lb : s.acrossDir;
    float reach = -s.outerDist + mat_thickness;
    vec3 scene = srgbToLinear(sampleBackground(s.v + dir * reach / mat_area_size));
    return specular + s.fresnel * wr * scene;
}
```

`src/render_helpers/material/optics/reflection.rs`:

```rust
//! `reflection`: stage 6, neutral at amount 0. Static: the amount is its only
//! uniform.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};

pub struct ReflectionOptic;

impl Optic for ReflectionOptic {
    const NAME: &'static str = "reflection";
    const GLSL: &'static str = include_str!("../../shaders/material/reflection.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[("mat_reflection", UniformType::_1f)];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![Uniform::new("mat_reflection", glass.reflection.amount as f32)]
    }
}
```

In `optics/mod.rs` (renderer): `pub mod reflection;` and `OpticEntry::of::<reflection::ReflectionOptic>(),` between aurora and iridescence. In `main.frag`, before `specular = iridescence_specular(specular, surf);`: `specular = reflection_specular(specular, surf);`.

- [ ] **Step 5: Validate, regenerate, test.** Shader check with the optic list `saturation noise aurora reflection iridescence` (exit 0). `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_parameter_table_matches_the_docs`. Then `just test-one -p niri-config reflection`, `just test-one -p niri reflection`, `just test-one -p niri material_source_is_prelude`, `just test-one -p niri optic_` (the settling tests iterate `OPTICS`: a static optic must hold its values).

- [ ] **Step 6: Evidence.** Dump to `$EV/step3` (`GLASS_EDGE_DUMP=$EV/step3 just test-one -p niri every_case_renders_frozen`), then:

```bash
C=docs/materials/scripts/glass-edge-compare.py
python3 $C identical $EV/step3-before $EV/step3 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque
for l in stock live; do
  python3 $C pair $EV/step3 $l-off $l-reflection-0 --region all --expect same
  python3 $C pair $EV/step3 $l-off $l-reflection --region face --expect same
  python3 $C pair $EV/step3 $l-off $l-reflection --region outside --expect same
  python3 $C pair $EV/step3 $l-off $l-reflection --region bevel --expect differ
done
```

Expected: every command exits 0. That is: every existing case is decoded-identical to step 2 (spec §6, step 3 at default values); `reflection 0` renders exactly as no node; `reflection 0.6` leaves the face and everything outside the slab untouched and changes the bevel. Append the output under "Step 3", with the `live-reflection` PNG's bevel read by eye (the backdrop colour on the dark edge, brightest toward the silhouette).

- [ ] **Step 7: Documentation.** `material-config.md` `## Optics`, after `### iridescence`:

```markdown
### reflection

Stage 6. `reflection <amount>` adds what the bevel reflects: the scene just
beyond the silhouette, sampled outward across the bevel and one thickness
further, weighted by the Fresnel term and faded in over the first tenth of
the bevel. It is not tinted by `attenuation-color` (reflection happens at the
surface), so a dark edge still shows the wallpaper's colours, and it goes
through the same prefilter as the refracted image, so `roughness` softens it.
Distortion and jelly ripple bend its direction. Its explicit neutral is 0, and
omission is 0; nothing inherits.
```

`render-pipeline.md`: row 6 gains "then the `reflection` optic adds the Fresnel-weighted, untinted scene just beyond the silhouette on the bevel"; its Samples column gains `niri_tex_bg[_high]`, `niri_tex_backdrop[_high]` (reflection); §5 gains `| reflection | glass.reflection (pending, prism-7024c4) | 6 |`. In `glass-parameter-sweep.sh`'s allowlist, change `glass:iridescence|glass:aurora) ;;` to `glass:iridescence|glass:aurora|glass:reflection|glass:edge-highlight|glass:bevel-profile) ;;` (Task 8 adds the optic the second key names; the sweep refuses an unknown key before launching, so listing it early is harmless).

- [ ] **Step 8: Commit**

```bash
just test-fast
tasks done material-02b42a "reflection optic: config, renderer, GLSL, docs; neutral byte-identical, face untouched, bevel lit"
tasks check
git add niri-config src/render_helpers src/tests/glass_edge.rs docs/materials tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): add the glass reflection optic (material-be611b)"
```

---

### Task 8: The `edge-highlight` optic

**Files:**
- Create: `niri-config/src/material/optics/edge_highlight.rs`
- Modify: `niri-config/src/material/optics/mod.rs`, `niri-config/src/material/mod.rs`, `niri-config/src/lib.rs`
- Create: `src/render_helpers/material/optics/edge_highlight.rs`, `src/render_helpers/shaders/material/edge_highlight.frag`
- Modify: `src/render_helpers/material/bevel.rs` (highlight weight), `src/render_helpers/material/optics/mod.rs`, `src/render_helpers/shaders/material/main.frag`, `src/render_helpers/shaders/mod.rs`, `src/render_helpers/material/mod.rs` test module (Review Focus 5)
- Modify: `src/tests/glass_edge.rs` (cases), `docs/materials/material-config.md`, `docs/materials/render-pipeline.md`

**Interfaces:**
- Produces: `niri_config::ResolvedEdgeHighlight { amount: f64 }`, `ResolvedGlass::edge_highlight`; KDL `edge-highlight <0..1>`; `bevel::highlight_alpha(roughness: f64) -> f64`, `bevel::ggx_peak_ratio(x: f64, alpha: f64) -> f64`, `bevel::highlight_weight(structural: DVec3, perturbed: DVec3, light: DVec2, roughness: f64) -> f64`; `EdgeHighlightOptic` (`NAME = "edge-highlight"`, uniforms `mat_edge_highlight`, `mat_edge_highlight_alpha`); GLSL `vec3 edge_highlight_specular(vec3 specular, Surface s)`.

- [ ] **Step 1: Dump the pre-change renders** to `$EV/step4-before`.

- [ ] **Step 2: Write the failing tests.** In `bevel.rs` tests:

```rust
    /// The light from the top left, as `mat_sig_light` carries it at rest.
    const LIGHT: DVec2 = DVec2::new(-1., -1.);

    fn toward_light(tilt_deg: f64) -> DVec3 {
        let a = tilt_deg.to_radians();
        (LIGHT.normalize() * a.sin()).extend(a.cos())
    }

    #[test]
    fn the_highlight_shader_names_the_same_formulas() {
        let frag = include_str!("../shaders/material/edge_highlight.frag");
        for line in [
            "return a2 * a2 / (t * t);",
            "vec3 l = normalize(vec3(normalize(mat_sig_light.xy), 1.0));",
            "vec3 h = normalize(l + vec3(0.0, 0.0, 1.0));",
            "float tilt = smoothstep(0.0, 1.0, (1.0 - s.structural.z) / (1.0 - h.z));",
            "float lobe = ggxPeakRatio(max(dot(s.perturbed, h), 0.0), mat_edge_highlight_alpha);",
        ] {
            assert!(frag.contains(line), "edge_highlight.frag lacks `{line}`");
        }
    }

    #[test]
    fn the_highlight_vanishes_at_a_rounded_face_join() {
        for roughness in [0., 1.] {
            let s = Slab::at_rest(DVec2::splat(56.), 12., 0., 20., 2.).surface(DVec2::new(44.001, 0.));
            let w = highlight_weight(s.normal, s.normal, DVec2::new(1., 0.), roughness);
            assert!(w < 1e-6, "roughness {roughness}: {w}");
        }
    }

    #[test]
    fn a_facet_at_the_half_angle_lights_at_full_gain() {
        // R / chamfer = tan 22.5°: the planar facet's normal is H.
        for roughness in [0., 1.] {
            let n = toward_light(22.5);
            assert!((highlight_weight(n, n, LIGHT, roughness) - 1.).abs() < 1e-9);
        }
    }

    #[test]
    fn a_45_degree_facet_takes_the_lobe_tail() {
        let n = toward_light(45.);
        assert!((highlight_weight(n, n, LIGHT, 0.) - 0.000117).abs() < 2e-6);
        assert!((highlight_weight(n, n, LIGHT, 1.) - 0.4827).abs() < 1e-3);
    }

    #[test]
    fn far_side_facets_take_only_the_lobe_tail() {
        // A 45 degree facet facing away: 67.5 degrees from H.
        let n = toward_light(-45.);
        let x = 67.5f64.to_radians().cos();
        for roughness in [0., 1.] {
            let w = highlight_weight(n, n, LIGHT, roughness);
            assert!((w - ggx_peak_ratio(x, highlight_alpha(roughness))).abs() < 1e-12);
        }
        assert!(highlight_weight(n, n, LIGHT, 0.) < 1e-5);
        // Past 90 degrees from H the clamp leaves only D(0).
        let under = toward_light(-89.);
        let past = (under + DVec3::new(0., 0., -0.5)).normalize();
        assert!(past.dot((LIGHT.normalize().extend(1.).normalize() + DVec3::Z).normalize()) < 0.);
        assert_eq!(highlight_weight(under, past, LIGHT, 1.), ggx_peak_ratio(0., highlight_alpha(1.)));
    }
```

In `edge_highlight.rs` (renderer) tests:

```rust
    #[test]
    fn uniforms_are_the_gain_and_the_roughness_alpha() {
        let glass = ResolvedGlass {
            edge_highlight: ResolvedEdgeHighlight { amount: 0.5 },
            roughness: 1.,
            ..Default::default()
        };
        let blur = Blur::default();
        let values = EdgeHighlightOptic::values(&glass, &frame(&blur));
        assert_eq!(values[0].name, "mat_edge_highlight");
        assert_eq!(values[0].value, UniformValue::_1f(0.5));
        assert_eq!(values[1].name, "mat_edge_highlight_alpha");
        assert_eq!(values[1].value, UniformValue::_1f(0.5));
        assert_eq!(EdgeHighlightOptic::next_change(&glass, &frame(&blur)), None);
    }
```

In `src/render_helpers/material/mod.rs` tests (Review Focus 5; `SignalUniforms::from_frame` builds `light` as `[-1, -1, 0]`, or a unit `(cos, sin)` under `rim-orbit`):

```rust
    #[test]
    fn the_signal_light_is_never_vertical() {
        // The glint and edge-highlight normalize the light's xy.
        for attention in [niri_config::AttentionResponse::None, niri_config::AttentionResponse::RimOrbit] {
            let r = niri_config::ResolvedResponse {
                attention,
                ..niri_config::ResolvedResponse::default()
            };
            for level in [0., 0.5, 1.] {
                for breath in [0., 0.5, 1.] {
                    let frame = SignalFrame {
                        accent: None,
                        level,
                        breath,
                        impulses: Default::default(),
                        presence: 0.,
                        focus: 0.,
                        beam: BeamFrame::REST,
                    };
                    let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
                    let [x, y, _] = SignalUniforms::from_frame(&frame, &g, &r).light;
                    assert!(x.hypot(y) >= 0.99, "{attention:?} {level} {breath}: ({x}, {y})");
                }
            }
        }
    }
```

`SignalFrame` has no `Default`; the fields above are the ones the neighbouring `from_frame` tests spell out. Import `BeamFrame` the way those tests do if the module does not already.

In `niri-config/src/lib.rs`, the two tests from Task 7 Step 2 with `edge-highlight` / `edge_highlight` / `ResolvedEdgeHighlight` and value 0.5. In `shaders/mod.rs`: markers gain `"// ---- optic: edge-highlight"` between reflection and iridescence; `assert!(main.contains("specular = edge_highlight_specular(specular, surf);"));`; and in `every_optic_declares_its_uniforms_in_its_glsl` the hook check becomes

```rust
            let prefix = format!("{}_", entry.name.replace('-', "_"));
            assert!(entry.glsl.contains(&prefix), "{}: no hook function", entry.name);
```

In `glass_edge.rs` `cases()`:

```rust
    for look in [STOCK, LIVE] {
        cases.push(case(look, "highlight-0", "edge-highlight 0", RING_OFF));
        cases.push(case(look, "k2", "bevel-profile 2", RING_OFF));
        cases.push(case(look, "highlight", "bevel-profile 2\nedge-highlight 0.5", RING_OFF));
    }
```

Run: `just test-one -p niri bevel::tests::the_highlight` — Expected: compile failure, no `highlight_weight`.

- [ ] **Step 3: Implement the mirror.** Append to `bevel.rs` (non-test part):

```rust
/// GGX alpha for the edge highlight: mix(0.04, 0.5, roughness).
pub fn highlight_alpha(roughness: f64) -> f64 {
    0.04 + (0.5 - 0.04) * roughness
}

/// D(x) / D(1) for GGX (Trowbridge-Reitz): 1 where the normal meets H.
/// Mirrors `ggxPeakRatio`.
pub fn ggx_peak_ratio(x: f64, alpha: f64) -> f64 {
    let a2 = alpha * alpha;
    let t = x * x * (a2 - 1.) + 1.;
    a2 * a2 / (t * t)
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// The edge highlight's weight before its gain (spec §3.3): a GGX lobe where
/// the perturbed normal bisects the view and a key light at 45 degrees'
/// elevation, faded in by the structural tilt. Mirrors
/// `edge_highlight_specular`.
pub fn highlight_weight(structural: DVec3, perturbed: DVec3, light: DVec2, roughness: f64) -> f64 {
    let l = light.normalize().extend(1.).normalize();
    let h = (l + DVec3::Z).normalize();
    let tilt = smoothstep(0., 1., (1. - structural.z) / (1. - h.z));
    tilt * ggx_peak_ratio(perturbed.dot(h).max(0.), highlight_alpha(roughness))
}
```

- [ ] **Step 4: Implement the config side**: `niri-config/src/material/optics/edge_highlight.rs` is Task 7 Step 3's file with `Reflection` → `EdgeHighlight`, `ResolvedReflection` → `ResolvedEdgeHighlight`, the doc lines "`edge-highlight <amount>`: stage 6, a key-light lobe on the bevel. Neutral at amount 0." and "Peak added linear brightness of the lobe; 0 is none.", node `"edge-highlight"`, write `format!("edge-highlight {v}")`, read `g.edge_highlight.amount`. Register it: `pub mod edge_highlight;`, `ORDER` = `["saturation", "noise", "aurora", "reflection", "edge-highlight", "iridescence"]`, `specs.extend(edge_highlight::params());` after reflection; `Glass` field `pub edge_highlight: Option<optics::edge_highlight::EdgeHighlight>,` (knuffel names it `edge-highlight`); `ResolvedGlass` field, default and resolve as in Task 7; re-export and the exhaustive literal.

- [ ] **Step 5: Implement the renderer side.** `src/render_helpers/shaders/material/edge_highlight.frag`:

```glsl
// Optic: edge-highlight (render-pipeline.md stage 6). Neutral at amount 0.
uniform float mat_edge_highlight;
uniform float mat_edge_highlight_alpha;

// D(x) / D(1) for GGX (Trowbridge-Reitz): 1 where the normal meets H.
float ggxPeakRatio(float x, float alpha) {
    float a2 = alpha * alpha;
    float t = x * x * (a2 - 1.0) + 1.0;
    return a2 * a2 / (t * t);
}

// A key-light lobe where the bevel's normal bisects the view and the light
// (spec §3.3): the light sits at 45 degrees' elevation in the signal light's
// direction, so H is 22.5 degrees off vertical. `tilt` fades it in from a
// flat surface, which keeps a rounded face join dark at every roughness.
// Not multiplied by F: it stands for a bright source. bevel.rs mirrors it.
vec3 edge_highlight_specular(vec3 specular, Surface s) {
    if (mat_edge_highlight <= 0.0)
        return specular;
    vec3 l = normalize(vec3(normalize(mat_sig_light.xy), 1.0));
    vec3 h = normalize(l + vec3(0.0, 0.0, 1.0));
    float tilt = smoothstep(0.0, 1.0, (1.0 - s.structural.z) / (1.0 - h.z));
    float lobe = ggxPeakRatio(max(dot(s.perturbed, h), 0.0), mat_edge_highlight_alpha);
    return specular + vec3(mat_edge_highlight * tilt * lobe);
}
```

`src/render_helpers/material/optics/edge_highlight.rs`:

```rust
//! `edge-highlight`: stage 6, neutral at amount 0. Static: its gain and the
//! roughness-derived lobe width are its uniforms.

use niri_config::ResolvedGlass;
use smithay::backend::renderer::gles::{Uniform, UniformType};

use super::{Optic, OpticFrame};
use crate::render_helpers::material::bevel::highlight_alpha;

pub struct EdgeHighlightOptic;

impl Optic for EdgeHighlightOptic {
    const NAME: &'static str = "edge-highlight";
    const GLSL: &'static str = include_str!("../../shaders/material/edge_highlight.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_edge_highlight", UniformType::_1f),
        ("mat_edge_highlight_alpha", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, _ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        vec![
            Uniform::new("mat_edge_highlight", glass.edge_highlight.amount as f32),
            Uniform::new("mat_edge_highlight_alpha", highlight_alpha(glass.roughness) as f32),
        ]
    }
}
```

Register `OpticEntry::of::<edge_highlight::EdgeHighlightOptic>()` between reflection and iridescence; in `main.frag`, `specular = edge_highlight_specular(specular, surf);` between the reflection and iridescence calls.

- [ ] **Step 6: Validate, regenerate, test.** Shader check with `saturation noise aurora reflection edge_highlight iridescence` (exit 0). Table regeneration. `just test-one -p niri bevel::tests`, `just test-one -p niri edge_highlight`, `just test-one -p niri-config edge_highlight`, `just test-one -p niri material_source_is_prelude`, `just test-one -p niri every_optic_declares`, `just test-one -p niri optic_`, `just test-one -p niri the_signal_light`.

- [ ] **Step 7: Evidence.** Dump to `$EV/step4`, then:

```bash
C=docs/materials/scripts/glass-edge-compare.py
python3 $C identical $EV/step4-before $EV/step4 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque stock-reflection-0 stock-reflection live-reflection-0 live-reflection
for l in stock live; do
  python3 $C pair $EV/step4 $l-off $l-highlight-0 --region all --expect same
  python3 $C pair $EV/step4 $l-k2 $l-highlight --region face --expect same
  python3 $C pair $EV/step4 $l-k2 $l-highlight --region outside --expect same
  python3 $C pair $EV/step4 $l-k2 $l-highlight --region bevel --expect differ
done
```

Expected: every command exits 0 (spec §6: step 4 at default values is decoded-identical to step 3; `edge-highlight 0` is the unconfigured render; on a rounded bevel the highlight leaves the face, whose tilt is 0, and everything outside the slab untouched and lights the bevel). By eye from `live-highlight.png`: the lobe sits on the top-left bevel. Append under "Step 4".

- [ ] **Step 8: Documentation.** `material-config.md`:

```markdown
### edge-highlight

Stage 6. `edge-highlight <amount>` adds a key-light lobe on the bevel where
its normal bisects the view and a light at 45 degrees' elevation in the signal
light's direction (top left at rest; `attention "rim-orbit"` sways it with the
glint). The lobe is GGX, peak-normalized so the amount is the peak added
linear brightness, and `roughness` widens it. It fades in with the bevel's
tilt, so a rounded face join stays dark; a planar facet lights uniformly and
flashes when its slope matches the light's half-angle (`R / bevel = 0.414`).
Its explicit neutral is 0, and omission is 0; nothing inherits.
```

In `adding-an-optic.md` (and the `Optic::NAME` doc comment), state that a hyphenated optic name keeps its hyphen in `NAME`, `ORDER` and the KDL node, and becomes underscores in file, module, hook and uniform names (`edge-highlight`: `edge_highlight.frag`, `edge_highlight_specular`, `mat_edge_highlight`). `render-pipeline.md` row 6 gains "then the `edge-highlight` optic adds a GGX key-light lobe on the bevel, before `iridescence` hues all of it"; §5 gains `| edge-highlight | glass.edgeHighlight (pending, prism-7024c4) | 6 |`; the `roughness` row's Stage column becomes "source selection before 3; 5 scattering; 6 highlight width".

- [ ] **Step 9: Commit**

```bash
just test-fast
tasks done material-5e64ef "edge-highlight optic with its f64 weight; neutral byte-identical; lobe on the light-facing bevel"
tasks check
git add niri-config src/render_helpers src/tests/glass_edge.rs docs/materials tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): add the glass edge-highlight optic (material-be611b)"
```

---

### Task 9: Contact sheet for the owner's review

**Files:**
- Create: `docs/materials/scripts/glass-edge-sheet.sh`
- Modify: the evidence document; `docs/specs/2026-09-30-glass-edge-optics-design.md` (status header)

**Interfaces:**
- Consumes: `glass-optic-smoke-lib.sh` (`capture_preflight`, `build_binaries`, `capture_identity`, `calibrate_probe_rect`, `write_config`, `start_nested`, `spawn_probe`, `probe_rect`, `shot`, `stop_nested`, `finish`, `GLASS_EXTRA`, `GLASS_BASELINE`, `WALL`, `PX`, `PY`).

- [ ] **Step 1: Write the script**, `docs/materials/scripts/glass-edge-sheet.sh` (executable):

```bash
#!/usr/bin/env bash
# Glass edge contact sheet (material-be611b, spec §6 "Visual judgement"): the
# probe's top-left corner over a real wallpaper, for bevel-profile {1, 2, 4}
# crossed with three looks, each at reflection {0, 0.6} x edge-highlight
# {0, 0.5} x roughness {0, 1}, plus a planar facet row at R / bevel = 0.414.
# The owner judges the look and Prism's starting values from it.
#
# Env: OUT (fresh artifact dir), NIRI_MATERIAL_WORK_ROOT, CAPTURE_TASK (task id
# authorizing this run), SHEET_WALL (wallpaper image; the owner's current one),
# SHEET_PILOT=1 for one lit cell (focused, k 2, reflection 0.6, edge-highlight
# 0.5, roughness 0) to check the crop before the full run.
set -eu
: "${CAPTURE_TASK:?task id authorizing this run}"
: "${SHEET_WALL:?wallpaper image to put behind the glass}"
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
capture_preflight headless
build_binaries
capture_identity --config preset=glass-edge-sheet --config output=1280x720 --config scale=1 --config vrr=off
WALL=$OUT/wall.png
magick "$SHEET_WALL" -resize 1280x720^ -gravity center -extent 1280x720 "$WALL"
calibrate_probe_rect "$NIRI" 0

# The looks, pinned. live-*: Prism's terminal glass on 2026-10-02.
declare -A LOOK=(
    [focused]=$'ior 1.28\nthickness 31.2\nattenuation-color "#2e3034"\nattenuation-distance 11\nchromatic-aberration 0.36\nbevel 15\noffset-x -6\noffset-y -5'
    [inactive]=$'ior 1.2\nthickness 62.3\nattenuation-color "#2e3034"\nattenuation-distance 42\nchromatic-aberration 0.32\ndistortion 0.2 scale=0.08\nbevel 15\noffset-x -6\noffset-y -5'
    [weak-thin]=$'ior 1.5\nthickness 6\nattenuation-color "#dfe8ff"\nattenuation-distance 60\nbevel 12\noffset-x 6\noffset-y 6'
)
corner_roi() { echo "160x160+$((PX - 40))+$((PY - 40))"; }

cell() {   # $1 name, $2 glass lines
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$NIRI" "$OUT/$1.kdl"
    spawn_probe "$NIRI" "$IDLE"
    sleep 2
    probe_rect "$NIRI"
    shot "$NIRI" "$1"
    magick "$OUT/$1.png" -crop "$(corner_roi)" +repage "$OUT/cell-$1.png"
    stop_nested
}

ROWS=()
if [ "${SHEET_PILOT:-0}" = 1 ]; then
    LOOKS=(focused); KS=(2); REFLS=(0.6); HLS=(0.5); ROUGHS=(0)
else
    LOOKS=(focused inactive weak-thin); KS=(1 2 4); REFLS=(0 0.6); HLS=(0 0.5); ROUGHS=(0 1)
fi
for look in "${LOOKS[@]}"; do
    for k in "${KS[@]}"; do
        row=()
        for refl in "${REFLS[@]}"; do
            for hl in "${HLS[@]}"; do
                for rough in "${ROUGHS[@]}"; do
                    name="$look-k$k-r$refl-h$hl-g$rough"
                    cell "$name" "${LOOK[$look]}"$'\n'"bevel-profile $k"$'\n'"reflection $refl"$'\n'"edge-highlight $hl"$'\n'"roughness $rough"
                    row+=("$OUT/cell-$name.png")
                done
            done
        done
        ROWS+=("$look k$k:${row[*]}")
    done
done
# The planar flash: R / bevel = tan 22.5 degrees.
row=()
[ "${SHEET_PILOT:-0}" = 1 ] && ROUGHS=() || ROUGHS=(0 1)
for rough in "${ROUGHS[@]}"; do
    for hl in 0 0.5; do
        name="facet-h$hl-g$rough"
        cell "$name" $'ior 1.5\nthickness 4.97\nbevel 12\noffset-x 6\noffset-y 6\nbevel-profile 1'$'\n'"edge-highlight $hl"$'\n'"roughness $rough"
        row+=("$OUT/cell-$name.png")
    done
done
[ ${#row[@]} -gt 0 ] && ROWS+=("facet:${row[*]}")

args=()
for entry in "${ROWS[@]}"; do
    for f in ${entry#*:}; do
        args+=(-label "$(basename "$f" .png | sed 's/^cell-//')" "$f")
    done
done
magick montage "${args[@]}" -tile 8x -geometry +4+4 -pointsize 10 "$OUT/glass-edge-sheet.png"
echo "sheet: $OUT/glass-edge-sheet.png"
finish
```

- [ ] **Step 2: Pilot one cell.** `OUT=$EV/sheet-pilot CAPTURE_TASK=material-124f1f SHEET_PILOT=1 SHEET_WALL=<owner's wallpaper> NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material docs/materials/scripts/glass-edge-sheet.sh`. Read `$EV/sheet-pilot/glass-edge-sheet.png`: the probe's top-left corner with the lit bevel (the highlight's lobe) and the wallpaper must be inside the crop; the smoke library notes the headless output may be transformed, so if the lit corner is elsewhere, fix `corner_roi` before the full run. If `capture_preflight headless` refuses, park as Global Constraints says.

- [ ] **Step 3: The full sheet.** `OUT=$EV/sheet CAPTURE_TASK=material-124f1f SHEET_WALL=<wallpaper> NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material docs/materials/scripts/glass-edge-sheet.sh` (about 76 cells, 8 to 10 minutes after the build). Record `run: <actual> min (est 25, headless); build <m>, cells <m>; <outcome>` on the task.

- [ ] **Step 4: Hand the sheet to the owner.** `tasks attach material-be611b $EV/sheet/glass-edge-sheet.png --caption "glass edge contact sheet: bevel-profile x looks x reflection/edge-highlight/roughness, planar facet row"`. Append a "Step 5: contact sheet" section to the evidence document (command, `$EV/sheet`, identity from `capture.json`). Add the new scripts and the evidence document to `docs/materials/README.md`'s index, in its existing format. Update the spec's status header to "implemented on `glass-edges` (steps 1 to 4); contact sheet awaiting the owner's review". Commit:

```bash
tasks done material-124f1f "contact sheet script and sheet; attached to material-be611b for the owner's review"
tasks check
git add docs/materials docs/specs/2026-09-30-glass-edge-optics-design.md tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "docs(material): add the glass edge contact sheet for review (material-be611b)"
```

Then `tasks park material-be611b "Owner reviews the contact sheet attached to material-be611b and picks Prism starting values for bevel-profile, reflection and edge-highlight; then the branch gets its final review and prism-7024c4 is unblocked" --waiting-on user --reason review`.

---

## Self-review

- **Spec coverage.** §3.1 coordinate, profile, normal, softened gradient, ridge guard, slope cap, ray model, tap lift, bound: Tasks 2 and 4. §3.2 `(1 - F)`, interior-light attenuation on the new path (unchanged code, new `att`), reflection: Tasks 4 and 7. §3.3: Task 8. §3.4 hook and order: Tasks 6 to 8. §4 default changes: Task 5's predictions. §5 cost: no measurement is planned; the spec states an operation count, not a budget, and the contact sheet's capture lane records no GPU time. If a reviewer wants a cost figure, Task 9's script can add the lib's `trace_run`/`gpu_median_ns` round for `{reflection 0.6, edge-highlight 0.5}` against the default. §6 verification: Tasks 1, 2, 4 to 9 (with the harness deviation stated above). §7 delivery 1 to 4: Tasks 1 to 9; 5: `prism-7024c4`, filed and blocked on this task.
- **Placeholders.** None: task ids are filled, and every code step carries its code. `<run-date>` and `$EV` are run-time values each step says how to set.
- **Type consistency.** `EdgeSample` (Rust) and `Surface` (GLSL) are different on purpose: the Rust struct mirrors `slabSurface`'s outputs, the GLSL struct is the hook argument. `bevelAcross`, `acrossDir`, `surfaceHeight`, `fresnel` are introduced in Task 4 and consumed in Task 6.
