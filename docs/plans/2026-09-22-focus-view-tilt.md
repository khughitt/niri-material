# Focus View Tilt Implementation Plan

**Status:** executed on branch material-77db8a, not merged: the design was rejected in review on 2026-10-06 (see the spec's status).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the glass shader's fixed orthographic view ray with a view vector: a static perspective by screen position, plus a transient arrival swing on focus gain that settles to exactly straight-on.

**Architecture:** Four new config keys resolve into `ResolvedGlass` and `ResolvedResponse`. A small pure module (`render_helpers/material/view.rs`) owns the swing curve and directions. The tile holds a `ViewSwing` run and sends its slope as a uniform. The `Layout` observes the one focused window per update pass (`layout/focus_origin.rs`) and starts the swing. The shader builds the incident ray once per fragment and routes every orthographic `refract` and cosine through it, keeping today's code path when the view is off.

**Tech Stack:** Rust (niri fork, smithay), GLSL ES 1.00, knuffel config, bash smokes under the capture protocol.

**Spec:** `docs/specs/2026-09-22-focus-view-tilt-design.md`. Read it first; section numbers below refer to it. Read `docs/materials/render-pipeline.md` before touching the shader (AGENTS.md).

## Global Constraints

- Worktree: `.worktrees/material-77db8a`, branch `material-77db8a`. Every path below is relative to that worktree root.
- Task: `material-77db8a` (already started). Every commit message ends with `Refs material-77db8a`. Conventional commits. No AI attribution trailers of any kind.
- Tests run through `tools/tt`, never bare `cargo`: the inner loop is `python3 tools/tt test-fast -- python3 tools/test-affected <nextest filter>`; the full suite is `just test`; `just check` runs at pre-commit.
- Keys, blocks, ranges and defaults, verbatim from spec §4:
  - `view-perspective`: material (`glass`) block, default 0, range 0–1.
  - `view-parallax`: material (`glass`) block, default 1, range 0–8.
  - `view-tilt`: `response` block, default 0, range 0–45 (degrees).
  - `view-tilt-settle`: `response` block, default 900, range 100–5000 (ms).
- With every view key at its default the output is pixel-identical to the renderer at the branch's base commit `1c978f75`.
- The swing curve is `θ(u) = θ0 · (1 − u)² · cos(1.5π · u)`, `u = t / settle`, exactly 0 for `u ≥ 1`.
- The swing slope is `tan(radians(θ)) · d`, never `θ · d`.
- Displacement gain: `shift = shift(I0) + gain · (shift(I) − shift(I0))`; the gain never touches cosines.
- The swing obeys `motion_allowed` (`signal motion full`, animations on) only, not `focus ring-light`. Perspective ignores the motion policy.
- Only the `Layout` starts swings, for the window `Layout::focus_with_output` names; tiles never start one from their own `active` flag.
- Shader: no identifier named `flat` (it is a GLSL keyword in later versions); keep GLSL ES 1.00 syntax.
- Before removing the worktree at the very end, run `tt-report` (AGENTS.md).
- Each plan task has a child task. Run `tasks start <child>` before its first step, and put `tasks done <child>` in its commit. If `start` or `done` reports the claim is held by this session's own `sid:<pid>`, rerun it with `--force`.

  | Plan task | Child |
  | --- | --- |
  | 1 | `material-098a7a` |
  | 2 | `material-3ccefe` |
  | 3 | `material-5abc50` |
  | 4 | `material-4752ef` |
  | 5 | `material-4bcd82` |
  | 6 | `material-cd0e1d` |
  | 7 | `material-dd1ad8` |

- **Commit protocol.** `tools/upstream-report --check` in the pre-commit hook reads the index, so the report is regenerated after staging, never before. Every commit step below runs, in this order:

  ```bash
  tasks done <child>
  git add <the task's paths> tasks/
  just upstream-report
  git add docs/materials/upstream-divergence.md
  git commit -m "<message>"
  ```

  If the hook still reports the report stale, repeat the last three lines. Never commit with `--no-verify`.

## Review Focus

1. **A multi-monitor focus move** (focus goes to a window on another output): the new window swings in the seeded fallback direction, and the origin records the new output. Pinned in Task 5 (`a_focus_on_another_output_takes_the_fallback`).
2. **A window closing while it holds the origin**: the next focus observes a window that differs from the stale origin, so it swings from the closed window's last centre with no panic and no lookup of the dead window. Pinned in Task 5 (`closing_the_origin_window_is_harmless`).
3. **A config reload that sets `view-tilt 0` mid-swing**: the running swing keeps its snapshot and settles; the next gain starts nothing. Pinned in Task 4 (`a_reload_mid_swing_keeps_the_snapshot`).
4. **Perspective on a rotated or fractional-scale output**: the lean follows the backdrop's logical frame, so a portrait output leans across its own long axis. `mat_view_output` must be the backdrop's logical size, not the mode size. Pinned in Task 3 (`the_view_uniforms_carry_the_swing_and_the_backdrop_logical_size`).
5. **`view-parallax 0`**: the view adds nothing to displacement but still steepens the cosines. This is legal and must not divide by the gain. Pinned in Task 3's source assertions (the gain only multiplies).

---

### Task 1: Config keys

**Files:**
- Modify: `niri-config/src/material/mod.rs` (the `Glass` struct near line 485, `ResolvedGlass` near 532, its `Default` near 558, `core_params()` near 580, `Material::resolve` near 733, the `Response` struct near 286, `ResolvedResponse` near 321, its `Default` near 342, `ResolvedResponse::with_overrides` near 364)
- Modify: `niri-config/src/lib.rs` (the exhaustive `ResolvedGlass` literal near line 1360; the new test goes beside `ring_beam_noise_defaults_off_bounds_and_inherits`)
- Modify: `docs/materials/material-config.md` (regenerated glass table; hand-written response table and prose)

**Interfaces:**
- Produces: `ResolvedGlass::view_perspective: f64` (default 0), `ResolvedGlass::view_parallax: f64` (default 1), `ResolvedResponse::view_tilt: f64` (degrees, default 0), `ResolvedResponse::view_tilt_settle: f64` (ms, default 900).

- [ ] **Step 1: Write the failing test** in `niri-config/src/lib.rs`, directly after `ring_beam_noise_defaults_off_bounds_and_inherits`:

```rust
    #[test]
    fn view_keys_default_off_bound_and_inherit() {
        let parsed = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)]).unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!(
            (m.glass.view_perspective, m.glass.view_parallax),
            (0., 1.),
            "an orthographic camera and physical parallax unless asked"
        );
        let d = m.response(None);
        assert_eq!((d.view_tilt, d.view_tilt_settle), (0., 900.), "no swing unless asked");

        let parsed = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass { view-perspective 0.36; view-parallax 3; }; response "default" { view-tilt 20; view-tilt-settle 1200; }; response "still" {}; }"#,
        )])
        .unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!((m.glass.view_perspective, m.glass.view_parallax), (0.36, 3.));
        let d = m.response(None);
        assert_eq!((d.view_tilt, d.view_tilt_settle), (20., 1200.));
        let still = m.response(Some("still"));
        assert_eq!(
            (still.view_tilt, still.view_tilt_settle),
            (20., 1200.),
            "inherits from default"
        );

        for (config, message) in [
            (
                r#"material "tg" { glass { view-perspective 1.5; }; }"#,
                "value must be between 0 and 1",
            ),
            (
                r#"material "tg" { glass { view-parallax 9; }; }"#,
                "value must be between 0 and 8",
            ),
            (
                r#"material "tg" { glass {}; response "default" { view-tilt 46; }; }"#,
                "value must be between 0 and 45",
            ),
            (
                r#"material "tg" { glass {}; response "default" { view-tilt-settle 50; }; }"#,
                "value must be between 100 and 5000",
            ),
        ] {
            let err = parse_files_err(&[("config.kdl", config)]);
            assert!(err.contains(message), "{config}: {err}");
        }
    }
```

- [ ] **Step 2: Run it and confirm it fails to compile** (unknown fields)

Run: `python3 tools/tt test-fast -- python3 tools/test-affected view_keys_default_off_bound_and_inherit`
Expected: compile error, `no field view_perspective on type ResolvedGlass`.

- [ ] **Step 3: Add the glass keys.** In the `Glass` struct, after `light_ior`:

```rust
    /// Slope of the view ray at the output's corners: 0 is the orthographic
    /// camera, 0.36 leans 20° there (design 2026-09-22 §1).
    #[knuffel(child, unwrap(argument))]
    pub view_perspective: Option<FloatOrInt<0, 1>>,
    /// Gain on the displacement the view adds to refraction and to the
    /// ring's landing; 1 is physical (design 2026-09-22 §2).
    #[knuffel(child, unwrap(argument))]
    pub view_parallax: Option<FloatOrInt<0, 8>>,
```

In `ResolvedGlass`, after `pub light_ior: f64,`:

```rust
    pub view_perspective: f64,
    pub view_parallax: f64,
```

In `impl Default for ResolvedGlass`, after `light_ior: 6.,`:

```rust
            view_perspective: 0.,
            view_parallax: 1.,
```

In `core_params()`, after the `light-ior` entry:

```rust
        ParamSpec {
            node: "view-perspective",
            kind: ParamKind::float::<FloatOrInt<0, 1>>(d.view_perspective, "—"),
            write: |v| format!("view-perspective {v}"),
            read: Some(|g| Some(g.view_perspective)),
        },
        ParamSpec {
            node: "view-parallax",
            kind: ParamKind::float::<FloatOrInt<0, 8>>(d.view_parallax, "—"),
            write: |v| format!("view-parallax {v}"),
            read: Some(|g| Some(g.view_parallax)),
        },
```

In `Material::resolve`, after `light_ior: …`:

```rust
                view_perspective: g.view_perspective.map_or(d.view_perspective, |x| x.0),
                view_parallax: g.view_parallax.map_or(d.view_parallax, |x| x.0),
```

- [ ] **Step 4: Add the response keys.** In the `Response` struct, after `ring_beam_noise_hz`:

```rust
    #[knuffel(child, unwrap(argument))]
    pub view_tilt: Option<FloatOrInt<0, 45>>,
    #[knuffel(child, unwrap(argument))]
    pub view_tilt_settle: Option<FloatOrInt<100, 5000>>,
```

In `ResolvedResponse`, after `ring_beam_noise_hz`:

```rust
    /// Peak angle of the arrival swing on focus gain, in degrees; zero skips
    /// the swing (design 2026-09-22 §3).
    pub view_tilt: f64,
    /// How long the swing takes to settle, in ms.
    pub view_tilt_settle: f64,
```

Defaults `view_tilt: 0.,` and `view_tilt_settle: 900.,`. In `with_overrides`:

```rust
            view_tilt: response.view_tilt.map_or(base.view_tilt, |x| x.0),
            view_tilt_settle: response
                .view_tilt_settle
                .map_or(base.view_tilt_settle, |x| x.0),
```

- [ ] **Step 5: Fix the exhaustive literals.** Add `view_perspective: 0., view_parallax: 1.,` to the `ResolvedGlass { … }` literal in `niri-config/src/lib.rs` near line 1371. Also run `git grep -n "light_ior: 6\.,"`: every other exhaustive `ResolvedGlass` literal it finds gets the same two fields. `ResolvedResponse` literals elsewhere use `..Default::default()`; fix any that do not, as the compiler reports them.

- [ ] **Step 6: Regenerate the glass parameter table**

Run: `MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- python3 tools/test-affected params`
Then: `git diff --stat docs/materials/material-config.md`
Expected: the table in `material-config.md` gains `view-perspective` and `view-parallax` rows.

- [ ] **Step 7: Document the response keys by hand** in `docs/materials/material-config.md`:
  - In the response table, after the `ring-beam-noise-hz` row, add:

```markdown
| `view-tilt` | 0–45 degrees | 0 |
| `view-tilt-settle` | 100–5000 ms | 900 |
```

  - In the KDL example's `response "default"` block, after `ring-beam-noise-hz 3`, add `view-tilt 0` and `view-tilt-settle 900`.
  - After the paragraph on `ring-beam-noise-hz`, add:

```markdown
`view-tilt` swings the view on focus gain: the glass is seen tipped by up
to this many degrees, as if the camera had just come over from the window
that had focus before, and the view settles back with one soft overshoot.
Only the optics move (refraction, the glint on the chamfer, where the ring
sits under the face); window pixels never skew. `view-tilt-settle` is how
long the swing takes to come to rest, exactly. Zero skips the swing; it also
never runs under `signal { motion "reduced" }`, `"off"`, or `animations { off }`.

`view-perspective` (glass) leans the view by each fragment's place on the
output, like a camera over the screen's centre: 0 is straight down
everywhere, 0.36 leans 20° at the corners. `view-parallax` (glass) scales
only what the view adds to the backdrop's refraction and the ring's landing.
At 1 it is physical, which on a thin slab is a pixel or two. The ring's
landing is still capped at half of `ring-gap`, so a large gain on a narrow
gap stops at the cap. Neither changes the glint, which follows the view's
angle.
```

- [ ] **Step 8: Run the test and the package**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected view_keys_default_off_bound_and_inherit`
Expected: PASS.
Run: `python3 tools/tt test-fast -- python3 tools/test-affected`
Expected: all affected tests pass, including the parameter-table staleness test.

- [ ] **Step 9: Commit**

```bash
tasks done material-098a7a
git add niri-config docs/materials/material-config.md tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(config): view-perspective, view-parallax, view-tilt and view-tilt-settle

Four keys for the focus view tilt, all defaulting to today's orthographic
view: perspective and parallax on the glass, the arrival swing's peak angle
and settle time on the response block, inheriting like the ring keys.

Refs material-77db8a"
```

---

### Task 2: The swing curve and directions

**Files:**
- Create: `src/render_helpers/material/view.rs`
- Modify: `src/render_helpers/material/mod.rs` (add `pub mod view;` beside `pub mod ring;`)
- Modify: `src/render_helpers/material/ring.rs:206` (`fn hash01` becomes `pub(crate) fn hash01`)

**Interfaces:**
- Produces:
  - `pub fn swing_angle(theta0: f64, u: f64) -> f64`
  - `pub fn swing_slope(theta0: f64, settle: Duration, elapsed: Duration, direction: [f64; 2]) -> [f32; 2]`
  - `pub enum SwingFrom { Origin([f64; 2]), Fallback }` (derives `Debug, Clone, Copy, PartialEq`)
  - `pub fn direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom`
  - `pub fn seeded_direction(seed: u32) -> [f64; 2]`
  - `pub const COINCIDENT_PX: f64 = 1.`

- [ ] **Step 1: Write the module with its tests, bodies stubbed.** Create `src/render_helpers/material/view.rs`:

```rust
//! The view ray's moving part: the arrival swing on focus gain
//! (docs/specs/2026-09-22-focus-view-tilt-design.md §3). The static part,
//! the perspective, is computed per fragment in the shader; the curve lives
//! here only, and the shader receives its slope.

use std::f64::consts::PI;
use std::time::Duration;

use super::ring::hash01;

/// Half-turns of the cosine over one settle: 1.5 puts the single zero
/// crossing at u = 1/3 and a cosine zero at u = 1, where `(1 − u)²` also
/// vanishes, so the swing ends with zero value and zero slope.
const SWING_HALF_TURNS: f64 = 1.5;

/// Centres closer than this give no direction to swing along.
pub const COINCIDENT_PX: f64 = 1.;

/// Where a swing comes from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwingFrom {
    /// From the previous focus: the unit vector from its centre to the new
    /// focus's centre, in output-local logical coordinates.
    Origin([f64; 2]),
    /// No usable origin: the tile seeds a direction of its own.
    Fallback,
}

/// The swing angle in degrees at `u` = elapsed / settle: θ0 at u = 0, one
/// overshoot to about −0.1777 θ0 near u = 0.514, exactly 0 from u = 1 on.
pub fn swing_angle(theta0: f64, u: f64) -> f64 {
    todo!()
}

/// The swing's slope in the glass plane, as the shader takes it:
/// `tan(θ)` along `direction`. Exactly zero once the swing has settled.
pub fn swing_slope(
    theta0: f64,
    settle: Duration,
    elapsed: Duration,
    direction: [f64; 2],
) -> [f32; 2] {
    todo!()
}

/// The swing's direction from the previous focus's centre to the new one.
pub fn direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom {
    todo!()
}

/// A unit direction from `seed`, for a swing with no usable origin.
pub fn seeded_direction(seed: u32) -> [f64; 2] {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_swing_starts_at_its_peak_overshoots_once_and_ends_still() {
        let theta0 = 20.;
        assert_eq!(swing_angle(theta0, 0.), theta0);
        let steps = 100_000;
        let mut crossings = 0;
        let mut min = f64::INFINITY;
        let mut prev = swing_angle(theta0, 0.);
        for i in 1..steps {
            let a = swing_angle(theta0, i as f64 / steps as f64);
            if a.signum() != prev.signum() && a != 0. && prev != 0. {
                crossings += 1;
            }
            min = min.min(a);
            prev = a;
        }
        assert_eq!(crossings, 1, "one overshoot, no second crossing");
        assert!(
            (min / theta0 + 0.1777).abs() < 1e-3,
            "overshoot {} of θ0",
            min / theta0
        );
        assert_eq!(swing_angle(theta0, 1.), 0.);
        assert_eq!(swing_angle(theta0, 1.5), 0.);
        // A triple root at u = 1: the last step is still.
        assert!(swing_angle(theta0, 1. - 1e-4).abs() < 1e-10);
    }

    #[test]
    fn the_slope_is_the_tangent_along_the_direction_and_zero_when_settled() {
        let settle = Duration::from_millis(900);
        let s = swing_slope(20., settle, Duration::ZERO, [0.6, -0.8]);
        let t = 20f64.to_radians().tan();
        assert!((f64::from(s[0]) - 0.6 * t).abs() < 1e-6);
        assert!((f64::from(s[1]) + 0.8 * t).abs() < 1e-6);
        assert_eq!(swing_slope(20., settle, settle, [0.6, -0.8]), [0., 0.]);
        assert_eq!(
            swing_slope(20., settle, Duration::from_secs(5), [0.6, -0.8]),
            [0., 0.]
        );
    }

    #[test]
    fn the_direction_points_from_the_origin_and_coincident_centres_fall_back() {
        assert_eq!(
            direction_between([100., 50.], [400., 50.]),
            SwingFrom::Origin([1., 0.])
        );
        let SwingFrom::Origin(d) = direction_between([0., 0.], [3., -4.]) else {
            panic!("expected an origin direction");
        };
        assert!((d[0] - 0.6).abs() < 1e-12 && (d[1] + 0.8).abs() < 1e-12);
        assert_eq!(
            direction_between([10., 10.], [10.5, 10.5]),
            SwingFrom::Fallback
        );
    }

    #[test]
    fn seeded_directions_are_unit_and_differ_between_seeds() {
        let a = seeded_direction(1);
        let b = seeded_direction(2);
        for d in [a, b] {
            assert!((d[0].hypot(d[1]) - 1.).abs() < 1e-12);
        }
        assert_ne!(a, b);
        assert_eq!(seeded_direction(7), seeded_direction(7));
    }
}
```

In `src/render_helpers/material/mod.rs`, add `pub mod view;` directly after `pub mod ring;`. In `ring.rs`, change `fn hash01(mut x: u64) -> f64 {` to `pub(crate) fn hash01(mut x: u64) -> f64 {`.

- [ ] **Step 2: Run the tests and confirm they fail**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected render_helpers::material::view`
Expected: four FAILs, each panicking at `not yet implemented`.

- [ ] **Step 3: Implement the four bodies**

```rust
pub fn swing_angle(theta0: f64, u: f64) -> f64 {
    if u >= 1. {
        return 0.;
    }
    let u = u.max(0.);
    theta0 * (1. - u).powi(2) * (SWING_HALF_TURNS * PI * u).cos()
}

pub fn swing_slope(
    theta0: f64,
    settle: Duration,
    elapsed: Duration,
    direction: [f64; 2],
) -> [f32; 2] {
    let theta = swing_angle(theta0, elapsed.as_secs_f64() / settle.as_secs_f64());
    if theta == 0. {
        return [0., 0.];
    }
    let t = theta.to_radians().tan();
    [(t * direction[0]) as f32, (t * direction[1]) as f32]
}

pub fn direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    if len < COINCIDENT_PX {
        SwingFrom::Fallback
    } else {
        SwingFrom::Origin([d[0] / len, d[1] / len])
    }
}

pub fn seeded_direction(seed: u32) -> [f64; 2] {
    let angle = hash01(u64::from(seed) ^ 0x5EED_D1EC) * 2. * PI;
    [angle.cos(), angle.sin()]
}
```

- [ ] **Step 4: Run the tests and confirm they pass**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected render_helpers::material::view`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
tasks done material-3ccefe
git add src/render_helpers/material/view.rs src/render_helpers/material/mod.rs src/render_helpers/material/ring.rs tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(material): the view swing's curve, slope and directions

A polynomial envelope under a cosine that starts at the peak, overshoots
once to about -0.178 of it, and ends at exactly zero with zero slope, so
the run has a finite, still end. Directions come from the previous focus
or, with none usable, from a seed.

Refs material-77db8a"
```

---

### Task 3: The view in the shader, uniforms and fingerprint

The swing slope is `[0, 0]` everywhere in this task; Task 4 fills it in. At the end of this task `view-perspective` and `view-parallax` work end to end.

**Files:**
- Modify: `src/render_helpers/shaders/material/prelude.frag` (uniforms after line 64, globals after line 74, `tap` at 388, `lightShift` at 396)
- Modify: `src/render_helpers/shaders/material/main.frag` (after `slabSurface(...)`, the `surfaceCosine` line, the two comments on the orthographic ray)
- Modify: `src/render_helpers/shaders/mod.rs` (`material_uniform_names`, the source test, a new glslang test)
- Modify: `src/render_helpers/material/mod.rs` (`InputFingerprint`, `MaterialState::element`, `MaterialRenderElement`, the uniform list in `draw`, the test helper `fingerprint2`)
- Modify: `src/layout/tile.rs` (`MaterialDynamics`, `material_dynamics`, both element build sites near lines 1840 and 2020)

**Interfaces:**
- Consumes: `ResolvedGlass::view_perspective`, `ResolvedGlass::view_parallax` (Task 1).
- Produces:
  - `MaterialDynamics::view_swing: [f32; 2]`
  - `InputFingerprint::view_swing: [f32; 2]`
  - `MaterialState::element(…, optics, view_swing: [f32; 2], scale, …)`: the new parameter sits directly after `optics`.
  - uniforms `mat_view` (`vec4`: swing x, swing y, perspective, parallax) and `mat_view_output` (`vec2`: backdrop logical size).

- [ ] **Step 1: Add the glslang compile test and see it pass on today's shader.** In `src/render_helpers/shaders/mod.rs`'s test module:

```rust
    /// The assembled program parses and type-checks as GLSL ES 1.00. Needs
    /// `glslangValidator` on PATH, so it runs on request:
    /// `python3 tools/tt test-fast -- python3 tools/test-affected --run-ignored only material_source_compiles_under_glslang`
    #[test]
    #[ignore = "needs glslangValidator on PATH"]
    fn material_source_compiles_under_glslang() {
        let dir = std::env::temp_dir().join(format!("niri-material-glsl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("material.frag");
        // The header `shader_element.rs` prepends at compile time.
        std::fs::write(&path, format!("#version 100\n{}", material_source())).unwrap();
        let out = std::process::Command::new("glslangValidator")
            .arg(&path)
            .output()
            .expect("glslangValidator on PATH");
        assert!(
            out.status.success(),
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
```

Run: `python3 tools/tt test-fast -- python3 tools/test-affected --run-ignored only material_source_compiles_under_glslang`
Expected: PASS on the unchanged shader. If it fails on today's source, the fault is the harness, not this work. Fix the header (for example, a missing precision or varying declaration that the smithay program adds). Record what you changed in the test's comment, and get it passing before going on.

- [ ] **Step 2: Write the failing source assertions.** In `material_source_is_prelude_then_optics_in_order_then_main`, after the `mat_sig_ring` assertion, add:

```rust
        assert!(source.contains("uniform vec4 mat_view;"));
        assert!(source.contains("uniform vec2 mat_view_output;"));
        assert!(material_uniform_names()
            .iter()
            .any(|name| name.name == "mat_view" && name.type_ == UniformType::_4f));
        assert!(material_uniform_names()
            .iter()
            .any(|name| name.name == "mat_view_output" && name.type_ == UniformType::_2f));
        // Every orthographic refraction goes through the one view helper:
        // its two uses are the view-off path and the gain's reference.
        assert_eq!(source.matches("refract(vec3(0.0, 0.0, -1.0)").count(), 2);
        let helper = source.split_once("vec2 viewRefract(").unwrap().1;
        let helper = helper.split_once("\n}\n").unwrap().0;
        assert_eq!(helper.matches("refract(vec3(0.0, 0.0, -1.0)").count(), 2);
        // The gain only multiplies: `view-parallax 0` is legal.
        assert!(!helper.contains("/ mat_view.w"));
        assert!(!source.contains(" flat "));
        let main = source.split_once("void main()").unwrap().1;
        assert!(main.contains("g_view_ray = g_view_on ? viewRay(v) : vec3(0.0, 0.0, -1.0);"));
        assert!(main.contains("clamp(dot(-g_view_ray, surfaceNormal), 0.0, 1.0)"));
```

Run: `python3 tools/tt test-fast -- python3 tools/test-affected material_source_is_prelude_then_optics_in_order_then_main`
Expected: FAIL at `uniform vec4 mat_view;`.

- [ ] **Step 3: Shader: uniforms, globals and helpers.** In `prelude.frag`, after `uniform float mat_light_ior;`:

```glsl
// The view ray (docs/specs/2026-09-22-focus-view-tilt-design.md): the
// swing's slope in xy, the perspective slope at the output's corners in z,
// the parallax gain in w; and the output's logical size, for the fragment's
// place on it.
uniform vec4 mat_view;
uniform vec2 mat_view_output;
```

After `vec4 g_face_r;`:

```glsl
// Set once per fragment in main(): whether any view is configured, and the
// incident ray. With the view off every stage takes the orthographic path.
bool g_view_on;
vec3 g_view_ray;
```

Replace `tap` and `lightShift` (lines 388–398) with:

```glsl
// The incident ray at element UV `v` (spec §1): the perspective by the
// fragment's place on the output, plus the swing. The backdrop mapping
// covers the whole output in logical space after its transform, so it
// gives the output-local position.
vec3 viewRay(vec2 v) {
    vec2 c = (mat_backdrop_rect.xy + v * mat_backdrop_rect.zw) * mat_view_output;
    vec2 o = 0.5 * mat_view_output;
    vec2 s = mat_view.z * (c - o) / length(o) + mat_view.xy;
    return normalize(vec3(s, -1.0));
}

// The in-plane part of the refracted ray (spec §2). The parallax gain scales
// only what the view adds to the orthographic refraction, never today's.
vec2 viewRefract(vec3 n, float eta) {
    if (!g_view_on)
        return refract(vec3(0.0, 0.0, -1.0), n, eta).xy;
    vec2 tilted = refract(g_view_ray, n, eta).xy;
    if (mat_view.w == 1.0)
        return tilted;
    vec2 ortho = refract(vec3(0.0, 0.0, -1.0), n, eta).xy;
    return ortho + mat_view.w * (tilted - ortho);
}

vec3 tap(vec2 v, vec3 n, float ior, float thickness) {
    vec2 vv = v + (viewRefract(n, 1.0 / ior) * thickness) / mat_area_size;
    return srgbToLinear(sampleBackground(vv));
}

// Interior light follows the perturbed normal through the light-path index
// and lands `depth` px into the slab; this is that in-plane displacement,
// along the view ray.
vec2 lightShift(vec3 n, float ior, float depth) {
    return viewRefract(n, 1.0 / ior) * depth;
}
```

`viewRay` must come after `mat_backdrop_rect`'s declaration, which is near the top of the prelude, so placing it here is fine.

- [ ] **Step 4: Shader: main.** In `main.frag`, directly after `slabSurface(p, coverage, surfaceNormal, slabDist, innerDist, slabChamfer);`:

```glsl
    g_view_on = mat_view.x != 0.0 || mat_view.y != 0.0 || mat_view.z > 0.0;
    g_view_ray = g_view_on ? viewRay(v) : vec3(0.0, 0.0, -1.0);
```

Replace the Beer–Lambert comment and the `surfaceCosine` line:

```glsl
        // Beer-Lambert over the view-lengthened slab path: the cosine of the
        // incident ray on the structural normal, which for the orthographic
        // ray is the normal's z, so the chamfer tints more strongly than the
        // face. The 0.25 floor bounds near-edge-on facets at four times the
        // configured thickness.
        float surfaceCosine = g_view_on
            ? clamp(dot(-g_view_ray, surfaceNormal), 0.0, 1.0)
            : clamp(surfaceNormal.z, 0.0, 1.0);
```

The Fresnel block and `iridescence_specular` read `surfaceCosine` already; leave them as they are. `facing` uses `mat_sig_light` and is unchanged.

- [ ] **Step 5: Uniform names.** In `src/render_helpers/shaders/mod.rs`, after `UniformName::new("mat_light_ior", UniformType::_1f),`:

```rust
        UniformName::new("mat_view", UniformType::_4f),
        UniformName::new("mat_view_output", UniformType::_2f),
```

- [ ] **Step 6: Run the source test and the glslang test**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected material_source_is_prelude_then_optics_in_order_then_main`
Expected: PASS.
Run: `python3 tools/tt test-fast -- python3 tools/test-affected --run-ignored only material_source_compiles_under_glslang`
Expected: PASS.

- [ ] **Step 7: Write the failing element test.** In `src/render_helpers/material/mod.rs`'s test module, next to the existing fingerprint tests:

```rust
    #[test]
    fn the_view_swing_joins_the_fingerprint_and_a_move_under_perspective_changes_it() {
        let background_id = Id::new();
        let backdrop_id = Id::new();
        let at_rest = fingerprint(1, 1, &background_id, &backdrop_id);
        assert_eq!(at_rest.view_swing, [0., 0.]);
        let swinging = InputFingerprint {
            view_swing: [0.1, 0.],
            ..at_rest.clone()
        };
        assert_ne!(at_rest, swinging);

        // Perspective reads the element's place on the output, which the
        // backdrop mapping already carries.
        let ws = Rectangle::from_size(Size::new(1920., 1080.));
        let color = Color32F::from([0., 0., 0., 1.]);
        let here = background_mapping(
            Rectangle::new(Point::new(100., 100.), Size::new(400., 300.)),
            &[(ws, color)],
            Size::new(1920., 1080.),
        );
        let there = background_mapping(
            Rectangle::new(Point::new(900., 100.), Size::new(400., 300.)),
            &[(ws, color)],
            Size::new(1920., 1080.),
        );
        assert_ne!(
            InputFingerprint { mapping: here, ..at_rest.clone() },
            InputFingerprint { mapping: there, ..at_rest }
        );
    }
```

Check how the existing `background_mapping` tests near line 1404 build `color`, and build it the same way if `Color32F::from([f32; 4])` differs from their form.

Run: `python3 tools/tt test-fast -- python3 tools/test-affected the_view_swing_joins_the_fingerprint`
Expected: compile error, `no field view_swing`.

- [ ] **Step 8: Plumb `view_swing` through the element.**
  - In `InputFingerprint`, after `optics`:

```rust
    /// The view swing's slope this frame. Exactly `[0, 0]` at rest, so a
    /// settled pane's fingerprint is constant (design 2026-09-22 §5).
    pub view_swing: [f32; 2],
```

  - In the test helper `fingerprint2`, add `view_swing: [0.; 2],`.
  - In `MaterialRenderElement`, after `optics`: `view_swing: [f32; 2],`.
  - In `MaterialState::element`, add the parameter `view_swing: [f32; 2],` directly after `optics: Vec<Uniform<'static>>,`, and store it in the struct literal.
  - In `draw`, before `let mut uniforms`:

```rust
        let view_output = self.backdrop.borrow().logical_size();
```

   and after `Uniform::new("mat_light_ior", g.light_ior as f32),`:

```rust
            Uniform::new(
                "mat_view",
                [
                    self.view_swing[0],
                    self.view_swing[1],
                    g.view_perspective as f32,
                    g.view_parallax as f32,
                ],
            ),
            Uniform::new(
                "mat_view_output",
                [view_output.w as f32, view_output.h as f32],
            ),
```

  - In `src/layout/tile.rs`, add `view_swing: [f32; 2],` to `MaterialDynamics`. In `material_dynamics`'s returned literal, add `view_swing: [0., 0.],`, which Task 4 replaces. At both build sites (near 1851 and 2028), add `view_swing: dynamics.view_swing,` to the `InputFingerprint` literal and pass `dynamics.view_swing` to `material.element(…)` directly after `dynamics.optics`.
  - Fix any other `material.element(` or `InputFingerprint {` callers the compiler reports.

- [ ] **Step 9: Pin the uniform values (Review Focus 4).** Extract the two values into a helper on `MaterialRenderElement` so a test can check them without a GL context:

```rust
    /// `mat_view` and `mat_view_output`: the swing, the glass's perspective
    /// and parallax, and the backdrop's logical size, which follows the
    /// output's transform and scale rather than its mode.
    fn view_uniforms(&self) -> ([f32; 4], [f32; 2]) {
        let output = self.backdrop.borrow().logical_size();
        (
            [
                self.view_swing[0],
                self.view_swing[1],
                self.glass.view_perspective as f32,
                self.glass.view_parallax as f32,
            ],
            [output.w as f32, output.h as f32],
        )
    }
```

Use it in `draw` in place of the inline values from Step 8 (`let (view, view_output) = self.view_uniforms();` then `Uniform::new("mat_view", view)` and `Uniform::new("mat_view_output", view_output)`). Then write `the_view_uniforms_carry_the_swing_and_the_backdrop_logical_size`. The element tests near line 1846 already build `MaterialRenderElement`s; copy their setup (effect buffers included). Give the backdrop `EffectBuffer` a logical size of 1080×1920, as a portrait-rotated output has. Set `view_swing: [0.25, -0.5]` and a glass with `view_perspective: 0.36, view_parallax: 3.`. Assert `view_uniforms() == ([0.25, -0.5, 0.36, 3.], [1080., 1920.])`. If building an `EffectBuffer` needs a renderer those tests do not have, assert on the pure part instead: make `view_uniforms` a free function `view_uniforms(view_swing, &glass, output: Size<f64, Logical>)` and test that.

- [ ] **Step 10: Run the tests**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected`
Expected: all affected tests pass, including the two new ones.

- [ ] **Step 11: Commit**

```bash
tasks done material-5abc50
git add src/render_helpers src/layout/tile.rs tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(material): the view ray in the glass shader

material.frag builds the incident ray once per fragment from a perspective
by the fragment's place on the output and the swing's slope, and routes
refraction, the ring's landing, Beer-Lambert, the Fresnel glint and the
iridescence cosine through it. The parallax gain scales only what the view
adds. With the view off every stage takes the orthographic path unchanged.
The swing's slope joins the material fingerprint; it is zero until the
tile drives it.

Refs material-77db8a"
```

---

### Task 4: The swing run on the tile

**Files:**
- Modify: `src/layout/tile.rs` (the `ViewSwing` struct beside `FocusBeam` near line 386; the `Tile` field beside `focus_beam` at 145 and its initializer at 467; `beam_allowed` at 540; `cut_beam_if_forbidden`'s two call sites at 489 and 525; `material_dynamics`; `advance_animations` near 955; `are_animations_ongoing` near 970; tests)

**Interfaces:**
- Consumes: `view::{swing_slope, seeded_direction, SwingFrom}` (Task 2), `MaterialDynamics::view_swing` (Task 3), `ResolvedResponse::{view_tilt, view_tilt_settle}` (Task 1).
- Produces:
  - `pub fn start_view_swing(&mut self, from: SwingFrom)` on `Tile<W>`
  - `#[cfg(test)] pub(crate) fn view_swing_start(&self) -> Option<(Duration, [f64; 2])>` on `Tile<W>`, which returns the start instant and direction.
  - `#[cfg(test)] pub(crate) fn fallback_direction_at(&self, started: Duration) -> [f64; 2]` on `Tile<W>`.

- [ ] **Step 1: Write the failing tests** in `tile.rs`'s test module, after the beam tests. Use the existing helpers `beam_tile`, `flat_glass`, `render_dynamics`, `secs`:

```rust
    fn swing_response() -> niri_config::ResolvedResponse {
        niri_config::ResolvedResponse {
            view_tilt: 20.,
            view_tilt_settle: 900.,
            ring_beam_speed: 0.,
            ..Default::default()
        }
    }

    #[test]
    fn the_view_swing_runs_its_curve_and_settles_to_exact_rest() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), swing_response(), clock.clone());
        tile.update_render_elements(false, true, true, view);
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));

        let t = 20f64.to_radians().tan() as f32;
        assert_eq!(render_dynamics(&tile, 400., 300.).view_swing, [t, 0.]);
        assert!(tile.are_animations_ongoing());
        assert!(!tile.are_transitions_ongoing(), "a swing is not a layout transition");

        clock.set_unadjusted(secs(0.45));
        tile.update_render_elements(false, true, true, view);
        let mid = render_dynamics(&tile, 400., 300.).view_swing;
        assert!(mid[0] < 0., "past the crossing the view overshoots: {mid:?}");
        assert_eq!(mid[1], 0.);

        clock.set_unadjusted(secs(0.9));
        tile.update_render_elements(false, true, true, view);
        assert_eq!(render_dynamics(&tile, 400., 300.).view_swing, [0., 0.]);
        tile.advance_animations();
        assert_eq!(tile.view_swing_start(), None);

        // Settled: no clock, no deadline, and the same fingerprint inputs at
        // every later instant (design §3 lifecycle, §5).
        let settled = |tile: &Tile<TestWindow>| {
            let d = render_dynamics(tile, 400., 300.);
            (
                d.jelly_fingerprint,
                d.signal_fingerprint,
                d.glass_signal_fingerprint,
                d.optics,
                d.view_swing,
            )
        };
        let first = settled(&tile);
        assert_eq!(first.4, [0., 0.]);
        for t in [1.0, 1.5, 3.0, 10.0] {
            clock.set_unadjusted(secs(t));
            tile.update_render_elements(false, true, true, view);
            tile.advance_animations();
            assert!(!tile.are_animations_ongoing(), "at {t}");
            assert_eq!(tile.tick_deadline(Point::default(), view, secs(t)), None, "at {t}");
            assert_eq!(settled(&tile), first, "fingerprint inputs moved at {t}");
        }
    }

    #[test]
    fn a_second_start_replaces_the_run_with_a_fresh_snapshot() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), swing_response(), clock.clone());
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        clock.set_unadjusted(secs(0.5));
        tile.start_view_swing(SwingFrom::Origin([0., 1.]));
        assert_eq!(tile.view_swing_start(), Some((secs(0.5), [0., 1.])));
        let t = 20f64.to_radians().tan() as f32;
        assert_eq!(render_dynamics(&tile, 400., 300.).view_swing, [0., t]);
    }

    #[test]
    fn focus_loss_lets_the_swing_finish() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), swing_response(), clock);
        tile.update_render_elements(true, true, true, view);
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        tile.update_render_elements(false, true, true, view);
        assert!(tile.view_swing_start().is_some());
    }

    #[test]
    fn a_material_with_no_ring_still_swings() {
        let response = niri_config::ResolvedResponse {
            focus: niri_config::FocusResponse::None,
            ..swing_response()
        };
        let mut tile = beam_tile(flat_glass(), response, Clock::with_time(Duration::ZERO));
        tile.start_view_swing(SwingFrom::Fallback);
        assert!(tile.view_swing_start().is_some());
    }

    #[test]
    fn no_swing_at_zero_tilt() {
        let response = niri_config::ResolvedResponse {
            view_tilt: 0.,
            ..swing_response()
        };
        let mut tile = beam_tile(flat_glass(), response, Clock::with_time(Duration::ZERO));
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(tile.view_swing_start(), None);
    }

    #[test]
    fn reduced_motion_and_animations_off_forbid_and_cut_the_swing() {
        for (policy, animations_off) in [
            (niri_config::SignalMotionPolicy::Reduced, false),
            (niri_config::SignalMotionPolicy::Off, false),
            (niri_config::SignalMotionPolicy::Full, true),
        ] {
            let label = format!("{policy:?} animations_off={animations_off}");
            let mut tile =
                beam_tile(flat_glass(), swing_response(), Clock::with_time(Duration::ZERO));
            tile.start_view_swing(SwingFrom::Origin([1., 0.]));
            assert!(tile.view_swing_start().is_some(), "{label}: starts under full");

            let mut options = (*tile.options).clone();
            options.signal.motion = policy;
            options.animations.off = animations_off;
            tile.update_config(tile.view_size, tile.scale, Rc::new(options));
            assert_eq!(tile.view_swing_start(), None, "{label}: cut at rest");
            assert_eq!(render_dynamics(&tile, 400., 300.).view_swing, [0., 0.], "{label}");
            tile.start_view_swing(SwingFrom::Origin([1., 0.]));
            assert_eq!(tile.view_swing_start(), None, "{label}: not started");
        }
    }

    #[test]
    fn a_reload_mid_swing_keeps_the_snapshot() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), swing_response(), clock.clone());
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        let mut options = (*tile.options).clone();
        let mut material = options.materials["frost"].clone();
        material.responses[0].1.view_tilt = 0.;
        options.materials = Rc::new(HashMap::from([(String::from("frost"), material)]));
        tile.update_config(tile.view_size, tile.scale, Rc::new(options));
        clock.set_unadjusted(secs(0.1));
        let s = render_dynamics(&tile, 400., 300.).view_swing;
        assert!(s[0] > 0., "the running swing keeps its 20° snapshot: {s:?}");
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(tile.view_swing_start().map(|s| s.0), Some(Duration::ZERO), "no new run at view-tilt 0");
    }

    #[test]
    fn fallback_directions_are_unit_and_differ_between_panes() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut a = beam_tile(flat_glass(), swing_response(), clock.clone());
        let mut b = beam_tile(flat_glass(), swing_response(), clock);
        a.start_view_swing(SwingFrom::Fallback);
        b.start_view_swing(SwingFrom::Fallback);
        let (_, da) = a.view_swing_start().unwrap();
        let (_, db) = b.view_swing_start().unwrap();
        assert!((da[0].hypot(da[1]) - 1.).abs() < 1e-12);
        assert_ne!(da, db, "each pane seeds its own direction");
        assert_eq!(da, a.fallback_direction_at(Duration::ZERO));
    }
```

Add `use crate::render_helpers::material::view::SwingFrom;` to the test module's imports. `tick_deadline` is the same call `settled_focus_reports_no_deadline_and_no_transition` uses. If one of the fingerprint types in `settled` lacks `PartialEq` or `Debug`, compare its fields the way `InputFingerprint` does; `InputFingerprint` derives `PartialEq`, so each of them already implements it. If `tile.view_size` and `tile.scale` are named differently, use the fields `update_config` assigns (`self.view_size = view_size; self.scale = scale;`, near line 486).

Run: `python3 tools/tt test-fast -- python3 tools/test-affected swing`
Expected: compile error, `no method named start_view_swing`.

- [ ] **Step 2: The run.** Beside `FocusBeam`:

```rust
/// The arrival swing of the view on focus gain (design 2026-09-22 §3).
/// Everything is snapshotted at the start, so a reload mid-run does not
/// jump the view; the `Layout` starts it, never the tile's own focus flag.
#[derive(Debug)]
struct ViewSwing {
    /// Start instant on the unadjusted clock.
    started: Duration,
    /// Peak angle, degrees.
    theta0: f64,
    settle: Duration,
    /// Unit direction in output-local logical coordinates.
    direction: [f64; 2],
}

impl ViewSwing {
    fn elapsed(&self, clock: &Clock) -> Duration {
        clock.now_unadjusted().saturating_sub(self.started)
    }

    fn is_done(&self, clock: &Clock) -> bool {
        clock.should_complete_instantly() || self.elapsed(clock) >= self.settle
    }

    /// The slope the shader takes; exactly zero once done.
    fn slope(&self, clock: &Clock) -> [f32; 2] {
        if self.is_done(clock) {
            return [0., 0.];
        }
        view::swing_slope(self.theta0, self.settle, self.elapsed(clock), self.direction)
    }
}
```

Add `view_swing: Option<ViewSwing>,` to `Tile` after `focus_beam`, initialize it to `None` beside `focus_beam: None`, and import `crate::render_helpers::material::view::{self, SwingFrom}`.

- [ ] **Step 3: Policy, start, cut.** Replace `beam_allowed` with the pair:

```rust
    /// Whether the policy allows motion at all: `motion full`, animations
    /// on. The view swing needs only this (design 2026-09-22 §3).
    fn motion_allowed(&self) -> bool {
        self.options.signal.motion == niri_config::SignalMotionPolicy::Full
            && !self.options.animations.off
    }

    /// Whether the policy and response allow the ring's beam to move at
    /// all: `focus ring-light` and `motion_allowed`. This is the cut rule's
    /// test (design §1); it says nothing about the speed, which is
    /// snapshotted per beam.
    fn beam_allowed(&self, response: &ResolvedResponse) -> bool {
        response.focus == niri_config::FocusResponse::RingLight && self.motion_allowed()
    }
```

After `cut_beam_if_forbidden`:

```rust
    /// Starts the arrival swing, replacing a running one. The `Layout` calls
    /// this for the one focused window (design 2026-09-22 §3). Nothing
    /// happens without a material, when motion is not allowed, or at
    /// `view-tilt 0`.
    pub fn start_view_swing(&mut self, from: SwingFrom) {
        let Some(material) = &self.material else {
            return;
        };
        let response = material.material().response(None);
        if !self.motion_allowed() || response.view_tilt <= 0. {
            return;
        }
        let started = self.clock.now_unadjusted();
        let direction = match from {
            SwingFrom::Origin(direction) => direction,
            SwingFrom::Fallback => fallback_direction(material, started),
        };
        self.view_swing = Some(ViewSwing {
            started,
            theta0: response.view_tilt,
            settle: Duration::from_secs_f64(response.view_tilt_settle / 1000.),
            direction,
        });
    }

    /// Cut rule for the swing: it ends at rest the moment motion is no
    /// longer allowed. A reload that only changes `view-tilt` leaves a
    /// running swing on its snapshot.
    fn cut_view_swing_if_forbidden(&mut self) {
        if !self.motion_allowed() {
            self.view_swing = None;
        }
    }

    #[cfg(test)]
    pub(crate) fn view_swing_start(&self) -> Option<(Duration, [f64; 2])> {
        self.view_swing
            .as_ref()
            .map(|swing| (swing.started, swing.direction))
    }

    /// The direction a fallback swing started at `started` takes on this
    /// tile, so layout tests can tell a fallback from an origin direction.
    #[cfg(test)]
    pub(crate) fn fallback_direction_at(&self, started: Duration) -> [f64; 2] {
        fallback_direction(self.material.as_ref().expect("a glass tile"), started)
    }
```

As a free function beside `ViewSwing`:

```rust
/// A swing's direction when there is no usable origin: the pane's own seed
/// keeps two panes apart, and the start instant keeps one pane's successive
/// fallbacks from repeating.
fn fallback_direction(material: &MaterialState, started: Duration) -> [f64; 2] {
    view::seeded_direction(material.jelly_seed()[0].to_bits() ^ started.as_millis() as u32)
}
```

Call `self.cut_view_swing_if_forbidden();` directly after each of the two `self.cut_beam_if_forbidden();` calls (in `update_config` and `refresh_material`).

- [ ] **Step 4: Lifecycle and slope.**
  - In `advance_animations`, next to where the done beam is dropped:

```rust
        if self
            .view_swing
            .as_ref()
            .is_some_and(|swing| swing.is_done(&self.clock))
        {
            self.view_swing = None;
        }
```

  - In `are_animations_ongoing`, extend the final disjunct:

```rust
            || self.signal_render_visible
                && (self
                    .focus_beam
                    .as_ref()
                    .is_some_and(|beam| !beam.is_done(&self.clock))
                    || self
                        .view_swing
                        .as_ref()
                        .is_some_and(|swing| !swing.is_done(&self.clock)))
```

  and add one sentence to its doc comment: the view swing lives here for the same reason as the beam.
  - In `material_dynamics`, replace `view_swing: [0., 0.],` with:

```rust
            view_swing: self
                .view_swing
                .as_ref()
                .map_or([0., 0.], |swing| swing.slope(&self.clock)),
```

- [ ] **Step 5: Run the tests**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected swing`
Expected: all new tests pass. Then run `python3 tools/tt test-fast -- python3 tools/test-affected tile` and expect the beam tests to still pass.

- [ ] **Step 6: Commit**

```bash
tasks done material-4752ef
git add src/layout/tile.rs tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(tile): the view swing run

A tile holds one arrival swing, snapshotted at its start, reported on the
animation loop like the beam and dropped once settled, when its slope is
exactly zero and the fingerprint constant again. It obeys the motion policy
alone, so glass with no ring swings too, and a focus loss lets it finish.

Refs material-77db8a"
```

---

### Task 5: The Layout starts the swing from the focus origin

**Files:**
- Create: `src/layout/focus_origin.rs`
- Create: `src/layout/tests/view_swing.rs`
- Modify: `src/layout/mod.rs` (the `mod` list, the `Layout` struct near line 340, every `Self {` constructor near 717 and 743, `update_render_elements` near 2821)
- Modify: `src/layout/monitor.rs` (two helpers after `workspaces_with_render_geo_mut`)
- Modify: `src/layout/tests.rs` (`mod view_swing;` beside `mod animations;`)

**Interfaces:**
- Consumes: `Tile::start_view_swing`, `Tile::view_swing_start` (Task 4); `view::{direction_between, SwingFrom}` (Task 2).
- Produces:
  - `pub struct FocusOrigin<Id, O> { pub window: Id, pub output: O, pub centre: Point<f64, Logical> }`
  - `pub fn swing_from<Id: PartialEq, O: PartialEq>(origin: Option<&FocusOrigin<Id, O>>, window: &Id, output: &O, centre: Point<f64, Logical>) -> Option<SwingFrom>`
  - `pub fn observe<W: LayoutElement>(origin: &mut Option<FocusOrigin<W::Id, Output>>, mon: &mut Monitor<W>, window: &W::Id)`
  - `Monitor::tile_centre_on_output(&self, window: &W::Id) -> Option<Point<f64, Logical>>`
  - `Monitor::tile_mut(&mut self, window: &W::Id) -> Option<&mut Tile<W>>`
  - the `Layout` field `focus_origin: Option<FocusOrigin<W::Id, Output>>`

- [ ] **Step 1: The pure decision, with its tests first.** Create `src/layout/focus_origin.rs`:

```rust
//! Where the view swing comes from (docs/specs/2026-09-22-focus-view-tilt-design.md
//! §3): the last focus an update pass observed. The `Layout` observes the one
//! globally focused window, on its monitor, once per pass that visits it.

use smithay::output::Output;
use smithay::utils::{Logical, Point};

use super::monitor::Monitor;
use super::LayoutElement;
use crate::render_helpers::material::view::{direction_between, SwingFrom};

/// The last observed focus: the window, its output, and its tile's centre in
/// output-local logical coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct FocusOrigin<Id, O> {
    pub window: Id,
    pub output: O,
    pub centre: Point<f64, Logical>,
}

/// What observing `window` on `output` at `centre` means for the swing:
/// `None` when the focus is the origin's window, otherwise where the new
/// focus swings from.
pub fn swing_from<Id: PartialEq, O: PartialEq>(
    origin: Option<&FocusOrigin<Id, O>>,
    window: &Id,
    output: &O,
    centre: Point<f64, Logical>,
) -> Option<SwingFrom> {
    todo!()
}

/// One observation on the focused window's monitor, after its overview
/// state is synced: start the swing on a changed focus, then record the
/// focus as the origin. A focused window whose workspace is culled off the
/// output leaves the origin untouched.
pub fn observe<W: LayoutElement>(
    origin: &mut Option<FocusOrigin<W::Id, Output>>,
    mon: &mut Monitor<W>,
    window: &W::Id,
) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64, y: f64) -> Point<f64, Logical> {
        Point::from((x, y))
    }

    fn origin(window: u32, output: &'static str, x: f64) -> FocusOrigin<u32, &'static str> {
        FocusOrigin {
            window,
            output,
            centre: at(x, 100.),
        }
    }

    #[test]
    fn no_origin_falls_back() {
        assert_eq!(
            swing_from::<u32, &str>(None, &1, &"a", at(0., 0.)),
            Some(SwingFrom::Fallback)
        );
    }

    #[test]
    fn the_same_window_starts_nothing() {
        let o = origin(1, "a", 100.);
        assert_eq!(swing_from(Some(&o), &1, &"a", at(900., 100.)), None);
    }

    #[test]
    fn another_window_on_the_same_output_swings_from_the_origin() {
        let o = origin(1, "a", 100.);
        assert_eq!(
            swing_from(Some(&o), &2, &"a", at(900., 100.)),
            Some(SwingFrom::Origin([1., 0.]))
        );
    }

    #[test]
    fn another_output_or_a_coincident_centre_falls_back() {
        let o = origin(1, "a", 100.);
        assert_eq!(
            swing_from(Some(&o), &2, &"b", at(900., 100.)),
            Some(SwingFrom::Fallback)
        );
        assert_eq!(
            swing_from(Some(&o), &2, &"a", at(100.4, 100.)),
            Some(SwingFrom::Fallback)
        );
    }
}
```

Add `mod focus_origin;` to `src/layout/mod.rs` beside the other `mod` lines.

Run: `python3 tools/tt test-fast -- python3 tools/test-affected layout::focus_origin`
Expected: four FAILs at `not yet implemented`.

- [ ] **Step 2: Implement `swing_from`**

```rust
    match origin {
        Some(origin) if origin.window == *window => None,
        Some(origin) if origin.output == *output => Some(direction_between(
            [origin.centre.x, origin.centre.y],
            [centre.x, centre.y],
        )),
        _ => Some(SwingFrom::Fallback),
    }
```

Run the same filter again. Expected: 4 passed.

- [ ] **Step 3: The monitor helpers.** In `src/layout/monitor.rs`, after `workspaces_with_render_geo_mut`:

```rust
    /// The centre of `window`'s tile in output-local logical coordinates, as
    /// it renders now: the tile's animated position and size in its
    /// workspace, then the workspace's render position and the overview
    /// zoom, as `window_under` inverts. `None` if its workspace is culled.
    pub fn tile_centre_on_output(&self, window: &W::Id) -> Option<Point<f64, Logical>> {
        let zoom = self.overview_zoom();
        self.workspaces_with_render_geo().find_map(|(ws, geo)| {
            ws.tiles_with_render_positions()
                .find(|(tile, _, _)| tile.window().id() == window)
                .map(|(tile, pos, _)| {
                    let centre = pos + tile.animated_tile_size().downscale(2.).to_point();
                    geo.loc + centre.upscale(zoom)
                })
        })
    }

    pub fn tile_mut(&mut self, window: &W::Id) -> Option<&mut Tile<W>> {
        self.workspaces
            .iter_mut()
            .flat_map(Workspace::tiles_mut)
            .find(|tile| tile.window().id() == window)
    }
```

Before relying on it, confirm in `workspace.rs` (`tiles_with_render_positions`, line 1589) and in `window_under` (monitor.rs, line 1566) that tile positions are workspace-local and unzoomed. `window_under` computes `(pos_within_output - geo.loc).downscale(zoom)` in the overview, which is the inverse used here.

- [ ] **Step 4: Implement `observe`**

```rust
    let Some(centre) = mon.tile_centre_on_output(window) else {
        return;
    };
    if let Some(from) = swing_from(origin.as_ref(), window, &mon.output, centre) {
        if let Some(tile) = mon.tile_mut(window) {
            tile.start_view_swing(from);
        }
    }
    *origin = Some(FocusOrigin {
        window: window.clone(),
        output: mon.output.clone(),
        centre,
    });
```

- [ ] **Step 5: Wire it into the Layout.**
  - Add the field to `Layout` after `overview_progress`:

```rust
    /// The last focus an update pass observed, where the next focus's view
    /// swing comes from (design 2026-09-22 §3).
    focus_origin: Option<focus_origin::FocusOrigin<W::Id, Output>>,
```

  - Add `focus_origin: None,` to every `Layout` constructor literal (`grep -n "overview_progress: None" src/layout/mod.rs` finds them).
  - In `update_render_elements`, directly before `let MonitorSet::Normal {`:

```rust
        // The focus the view swing observes (design 2026-09-22 §3): the one
        // globally focused window, never a tile an interactive move carries.
        let focused = if matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_))) {
            None
        } else {
            self.focus_with_output()
                .map(|(win, output)| (win.id().clone(), output.clone()))
        };
```

  - In the monitor loop, between `mon.set_overview_progress(…)` and `mon.update_render_elements(…)`:

```rust
                if let Some((window, focused_output)) = &focused {
                    if mon.output == *focused_output {
                        focus_origin::observe(&mut self.focus_origin, mon, window);
                    }
                }
```

`monitors` is borrowed from `self.monitor_set` and `self.focus_origin` is a separate field, so the two borrows are disjoint, just as the existing `self.overview_progress` read in that loop is.

- [ ] **Step 6: Write the layout tests.** Create `src/layout/tests/view_swing.rs` and add `mod view_swing;` after `mod fullscreen;` in `src/layout/tests.rs`:

```rust
//! The Layout starts the view swing from the focus origin
//! (docs/specs/2026-09-22-focus-view-tilt-design.md §3).

use super::*;

fn swing_options() -> Options {
    let material = niri_config::ResolvedMaterial {
        name: String::from("frost"),
        glass: niri_config::ResolvedGlass::default(),
        responses: vec![(
            String::from("default"),
            niri_config::ResolvedResponse {
                view_tilt: 20.,
                ring_beam_speed: 0.,
                ..Default::default()
            },
        )],
    };
    Options {
        materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
        ..Default::default()
    }
}

fn glass_window(id: usize) -> Op {
    let mut params = TestWindowParams::new(id);
    params.rules = Some(ResolvedWindowRules {
        material: Some(niri_config::MaterialRef {
            name: String::from("frost"),
            response: None,
        }),
        ..Default::default()
    });
    Op::AddWindow { params }
}

fn swing_layout(ops: impl IntoIterator<Item = Op>) -> Layout<TestWindow> {
    check_ops_with_options(swing_options(), ops)
}

/// Every tile's swing, by window id: (start, direction).
fn swings(layout: &Layout<TestWindow>) -> Vec<(usize, Duration, [f64; 2])> {
    layout
        .monitors()
        .flat_map(|mon| mon.workspaces.iter())
        .flat_map(|ws| ws.tiles())
        .filter_map(|tile| {
            tile.view_swing_start()
                .map(|(t, d)| (*tile.window().id(), t, d))
        })
        .collect()
}

/// The windows whose swing started at the clock's current instant.
fn started_now(layout: &Layout<TestWindow>) -> Vec<usize> {
    let now = layout.clock.now_unadjusted();
    swings(layout)
        .into_iter()
        .filter(|(_, t, _)| *t == now)
        .map(|(id, _, _)| id)
        .collect()
}

fn tile_of(layout: &Layout<TestWindow>, id: usize) -> &Tile<TestWindow> {
    layout
        .monitors()
        .flat_map(|mon| mon.workspaces.iter())
        .flat_map(|ws| ws.tiles())
        .find(|tile| *tile.window().id() == id)
        .unwrap_or_else(|| panic!("no tile for window {id}"))
}

/// Asserts that window `id` started a fallback swing at the current instant.
fn assert_fallback(layout: &Layout<TestWindow>, id: usize) {
    let now = layout.clock.now_unadjusted();
    assert_eq!(
        direction_of(layout, id),
        tile_of(layout, id).fallback_direction_at(now),
        "window {id} should swing in its seeded direction"
    );
}

/// The centre of window `id`'s hit area on the output, found by walking out
/// from `inside` until `window_under` stops naming it. This is independent of
/// how `tile_centre_on_output` computes its centre: hit-testing is the other
/// consumer of the render geometry.
fn hit_centre(mon: &Monitor<TestWindow>, id: usize, inside: Point<f64, Logical>) -> Point<f64, Logical> {
    let hits = |p: Point<f64, Logical>| mon.window_under(p).is_some_and(|(w, _)| *w.id() == id);
    assert!(hits(inside), "the computed centre is not over window {id}");
    let edge = |dx: f64, dy: f64| {
        let step = Point::from((dx, dy));
        let mut p = inside;
        while hits(p + step) {
            p += step;
        }
        p
    };
    let (l, r) = (edge(-0.25, 0.).x, edge(0.25, 0.).x);
    let (t, b) = (edge(0., -0.25).y, edge(0., 0.25).y);
    Point::from(((l + r) / 2., (t + b) / 2.))
}

fn assert_near(a: Point<f64, Logical>, b: Point<f64, Logical>, what: &str) {
    assert!((a.x - b.x).abs() <= 1. && (a.y - b.y).abs() <= 1., "{what}: {a:?} vs {b:?}");
}

fn direction_of(layout: &Layout<TestWindow>, id: usize) -> [f64; 2] {
    swings(layout)
        .into_iter()
        .find(|(w, _, _)| *w == id)
        .map(|(_, _, d)| d)
        .unwrap_or_else(|| panic!("window {id} has no swing"))
}

fn advance(layout: &mut Layout<TestWindow>, msec: i64) {
    Op::AdvanceAnimations { msec_delta: msec }.apply(layout);
}

fn output_named(layout: &Layout<TestWindow>, name: &str) -> Output {
    layout.outputs().find(|o| o.name() == name).unwrap().clone()
}

#[test]
fn a_focus_change_swings_from_the_old_centre() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![2], "the first focus falls back");
    assert_fallback(&layout, 2);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
    let d = direction_of(&layout, 1);
    assert!(d[0] < -0.99, "from window 2 on the right to window 1: {d:?}");
}

#[test]
fn a_b_c_without_a_pass_swings_c_from_a() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        glass_window(2),
        glass_window(3),
        Op::FocusColumn(1),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumn(2).apply(&mut layout);
    Op::FocusColumn(3).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![3], "B was never observed");
    assert!(direction_of(&layout, 3)[0] > 0.99, "from A on the left");
}

#[test]
fn a_b_a_without_a_pass_starts_no_swing() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    Op::FocusColumnRight.apply(&mut layout);
    layout.update_render_elements(None);
    assert!(started_now(&layout).is_empty());
}

#[test]
fn a_pass_over_an_inactive_output_leaves_the_origin() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        Op::AddOutput(2),
        Op::FocusOutput(1),
        glass_window(1),
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    let before = layout.focus_origin.clone();
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    let inactive = output_named(&layout, "output2");
    layout.update_render_elements(Some(&inactive));
    assert_eq!(layout.focus_origin, before);
    assert!(started_now(&layout).is_empty());
}

#[test]
fn a_pass_for_a_capture_of_the_focused_output_counts() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    // Screencopy and screenshots call this with their output.
    let focused = output_named(&layout, "output1");
    layout.update_render_elements(Some(&focused));
    assert_eq!(started_now(&layout), vec![1]);
    advance(&mut layout, 16);
    layout.update_render_elements(None);
    assert!(started_now(&layout).is_empty(), "observed once, not twice");
}

#[test]
fn a_focus_on_another_output_takes_the_fallback() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        Op::AddOutput(2),
        Op::FocusOutput(1),
        glass_window(1),
        Op::FocusOutput(2),
        glass_window(2),
        Op::FocusOutput(1),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusOutput(2).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![2]);
    assert_fallback(&layout, 2);
    assert_eq!(
        layout.focus_origin.as_ref().map(|o| o.output.name()),
        Some(String::from("output2"))
    );
}

#[test]
fn closing_the_origin_window_is_harmless() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::CloseWindow(2).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
}

#[test]
fn one_swinger_mid_workspace_switch_and_its_centre_is_on_screen() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        Op::FocusWorkspaceDown,
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusWorkspaceUp.apply(&mut layout);
    advance(&mut layout, 60); // mid switch
    layout.update_render_elements(None);
    let mon = layout.monitors().next().unwrap();
    assert!(mon.workspace_switch.is_some(), "the switch is still running");
    assert_eq!(
        mon.workspaces_with_render_geo().count(),
        2,
        "both workspaces are on screen"
    );
    assert_eq!(started_now(&layout), vec![1], "only the focused window swings");
    let centre = mon.tile_centre_on_output(&1).unwrap();
    assert_near(centre, hit_centre(mon, 1, centre), "computed vs hit-tested centre");
    let origin = layout.focus_origin.as_ref().unwrap();
    assert_eq!(origin.window, 1);
    assert_near(origin.centre, hit_centre(mon, 1, centre), "recorded origin");
}

#[test]
fn one_swinger_in_the_overview_and_its_centre_is_on_screen() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        Op::FocusWorkspaceDown,
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    Op::ToggleOverview.apply(&mut layout);
    advance(&mut layout, 100);
    Op::FocusWorkspaceUp.apply(&mut layout);
    advance(&mut layout, 30);
    layout.update_render_elements(None);
    let mon = layout.monitors().next().unwrap();
    assert!(mon.overview_zoom() < 1., "the overview is zoomed out");
    assert!(
        mon.workspaces_with_render_geo().count() >= 2,
        "both workspaces are on screen"
    );
    assert_eq!(started_now(&layout), vec![1]);
    let centre = mon.tile_centre_on_output(&1).unwrap();
    assert_near(centre, hit_centre(mon, 1, centre), "computed vs hit-tested centre");
    assert_near(
        layout.focus_origin.as_ref().unwrap().centre,
        hit_centre(mon, 1, centre),
        "recorded origin",
    );
}
```

`hit_centre` measures the window's hit area. The tile's centre coincides with it because borders and insets are symmetric; if the default `Options` puts an asymmetric decoration on the tile, measure against the window's centre instead and say so in the test. Adjust only the harness plumbing if names differ: `Op::CloseWindow` (check the `Op` enum; the window-removal op may be named `CloseWindow` or `RemoveWindow`), `layout.outputs()`, `o.name()`, `Workspace::tiles`. The assertions stay as written. If `FocusOutput(1)` is not needed because the first output is active by default, keep it anyway; it is harmless.

- [ ] **Step 7: Run the layout tests**

Run: `python3 tools/tt test-fast -- python3 tools/test-affected view_swing`
Expected: all pass. If `one_swinger_mid_workspace_switch…` finds the switch finished or one workspace off screen at 60 ms, pick another delay inside the switch animation (it is a spring; 30–120 ms is mid). Keep both visibility asserts, so the test proves what it claims.
Then run: `python3 tools/tt test-fast -- python3 tools/test-affected layout`
Expected: the whole layout suite passes, including the randomized ops tests (`verify_invariants`).

- [ ] **Step 8: Commit** (the protocol regenerates the upstream report after staging)

```bash
tasks done material-4bcd82
git add src/layout tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "feat(layout): start the view swing from the focus origin

The Layout observes the one focused window per update pass, on its monitor
after the overview state is synced, maps its tile's animated centre into
output space, and starts the swing on that tile when the focus changed,
from the last observed centre, or a seeded direction when there is none on
this output. Tiles never start one from their own active flag, which every
visible workspace shares.

Refs material-77db8a"
```

---

### Task 6: Identity, visibility and cost smoke; review clips

**Files:**
- Create: `docs/materials/scripts/glass-view-tilt-smoke.sh`
- Modify: `docs/materials/scripts/ring-motion-clips.sh` (header list, `SEQUENCES` default, `write_config` keys, the grid configs, two sequence functions, the dispatch `case`, the grid sheets)

**Interfaces:**
- Consumes: the built branch and a baseline binary built from `1c978f75`.

- [ ] **Step 1: Write the smoke.** Create `docs/materials/scripts/glass-view-tilt-smoke.sh` (mode 755). Two fixture choices matter:
  - **A patterned backdrop.** Refraction only moves the backdrop, and a uniform wallpaper moved by any amount looks identical. Every capture here, including the baseline identity, runs over a fine checkerboard, so a broken or shifted refraction is visible.
  - **llvmpipe, pinned and verified per launch.** The spec records cost on llvmpipe, as the optics did. The library starts Weston with `--renderer=gl` and forces nothing, so this smoke forces Mesa's software rasterizer. After every launch it reads the GL renderer each process reported, in the part of its log that launch wrote, and fails unless both name llvmpipe. Weston logs `GL renderer: llvmpipe (LLVM …)`. Smithay logs `GL Renderer: "llvmpipe (LLVM …)"` at info, which niri's default log filter drops (`smithay::backend::renderer::gles=error`), so the smoke sets `RUST_LOG` to keep it.

```bash
#!/usr/bin/env bash
# Glass view tilt smoke (material-77db8a,
# docs/specs/2026-09-22-focus-view-tilt-design.md §6): with every view key
# omitted the candidate renders identically to the baseline built from the
# branch's base commit; explicit neutral keys match omitted ones; a
# perspective changes the chamfer; the parallax gain moves the refracted
# backdrop; and the frame cost of the view is recorded on llvmpipe.
#
# Every capture runs over a fine checkerboard: refraction only displaces the
# backdrop, and a uniform wallpaper displaced by any amount looks identical.
#
# Env: OUT (artifact dir), NIRI_MATERIAL_WORK_ROOT, CAPTURE_TASK (task id
# authorizing this run), BASE_NIRI (release build of the base commit).
set -eu
: "${CAPTURE_TASK:?task id authorizing this run}"
: "${BASE_NIRI:?release build of the base commit}"
[ -x "$BASE_NIRI" ] || { echo "FAIL: BASE_NIRI is not executable: $BASE_NIRI" >&2; exit 1; }

# llvmpipe for Weston and the nested niri: Mesa's software rasterizer, and
# Mesa's EGL vendor only, so a host NVIDIA EGL cannot take over.
MESA_EGL=/usr/share/glvnd/egl_vendor.d/50_mesa.json
[ -f "$MESA_EGL" ] || { echo "FAIL: no Mesa EGL vendor file at $MESA_EGL" >&2; exit 1; }
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe __EGL_VENDOR_LIBRARY_FILENAMES=$MESA_EGL
# niri's default filter drops smithay's renderer report; keep it.
export RUST_LOG="niri=debug,smithay::backend::renderer::gles=info"

HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"

# Every launch checks its own processes: the GL renderer Weston and niri
# reported, read from the bytes each log gained during this launch only, so
# an earlier launch's line cannot satisfy it.
log_size() { if [ -f "$1" ]; then stat -c %s "$1"; else echo 0; fi; }
log_since() { tail -c "+$(($2 + 1))" "$1" 2>/dev/null || true; }   # file, byte offset
LAUNCHES=0
eval "$(declare -f start_nested | sed '1s/^start_nested/lib_start_nested/')"
start_nested() {
    local weston_at niri_at
    weston_at=$(log_size "$OUT/weston.log"); niri_at=$(log_size "$OUT/niri.log")
    lib_start_nested "$@"
    log_since "$OUT/weston.log" "$weston_at" | grep -q 'GL renderer: llvmpipe' \
        || fail "this launch's Weston did not report llvmpipe (see $OUT/weston.log)"
    for _ in $(seq 50); do
        log_since "$OUT/niri.log" "$niri_at" | grep -q 'GL Renderer: "llvmpipe' && break
        sleep 0.1
    done
    log_since "$OUT/niri.log" "$niri_at" | grep -q 'GL Renderer: "llvmpipe' \
        || fail "this launch's niri did not report llvmpipe (see $OUT/niri.log)"
    LAUNCHES=$((LAUNCHES + 1))
}

capture_preflight headless
build_binaries
capture_identity --binary "$BASE_NIRI" --config preset=view-tilt --config output=1280x720 \
    --config scale=1 --config vrr=off --config renderer=llvmpipe --config backdrop=checker-8px
calibrate_probe_rect "$NIRI" 0

# The checkerboard replaces the library's uniform wall after calibration,
# which measures the probe on its own geometry config: an explicit 16 px tile
# of two 8 px squares, repeated over the output.
WALL=$OUT/checker.png
magick -size 16x16 xc:white -fill black -draw "rectangle 0,0 7,7" -draw "rectangle 8,8 15,15" \
    -write mpr:tile +delete -size 1280x720 tile:mpr:tile "$WALL"

capture() {   # name, glass lines, binary
    GLASS_EXTRA=$2
    write_config "$OUT/$1.kdl"
    start_nested "$3" "$OUT/$1.kdl"
    spawn_probe "$3" "$IDLE"
    sleep 2
    probe_rect "$3"
    shot "$3" "$1"
    roi "$1" "$(face_roi)" face
    roi "$1" "$(chamfer_roi)" chamfer
    stop_nested
}
capture base-plain "" "$BASE_NIRI"
capture plain "" "$NIRI"
capture neutral $'view-perspective 0\nview-parallax 1' "$NIRI"
capture persp "view-perspective 0.5" "$NIRI"
capture persp-deep $'view-perspective 0.5\nview-parallax 4' "$NIRI"

ae "$OUT/base-plain.png" "$OUT/plain.png";      base_vs_plain_ae=$METRIC
ae "$OUT/plain.png" "$OUT/neutral.png";         neutral_vs_plain_ae=$METRIC
rmse "$OUT/persp-chamfer.png" "$OUT/plain-chamfer.png";      persp_chamfer_rmse=$METRIC
rmse "$OUT/persp-face.png" "$OUT/plain-face.png";            persp_face_rmse=$METRIC
rmse "$OUT/persp-deep-face.png" "$OUT/persp-face.png";       parallax_face_rmse=$METRIC
one_code=$(one_code)
assert_zero base_vs_plain_ae "$base_vs_plain_ae"
assert_zero neutral_vs_plain_ae "$neutral_vs_plain_ae"
assert_greater persp_chamfer_rmse_over_one_code "$persp_chamfer_rmse" "$one_code"
assert_greater persp_face_rmse_over_one_code "$persp_face_rmse" "$one_code"
assert_greater parallax_face_rmse_over_one_code "$parallax_face_rmse" "$one_code"

# Cost: three rounds, order rotated to balance host drift.
tools_ready; reserve_tracy_port
gpu_case() {
    case $1 in plain) GLASS_EXTRA= ;; persp) GLASS_EXTRA="view-perspective 0.5" ;;
        deep) GLASS_EXTRA=$'view-perspective 0.5\nview-parallax 4' ;; esac
    trace_run "gpu-$1-$2" "$TICK" 0
    gpu_median_ns "$OUT/gpu-$1-$2.tracy" >> "$OUT/gpu-$1.medians"
}
for c in plain persp deep; do gpu_case "$c" 1; done
for c in persp deep plain; do gpu_case "$c" 2; done
for c in deep plain persp; do gpu_case "$c" 3; done
plain_ns=$(median3 "$OUT/gpu-plain.medians")
persp_ns=$(median3 "$OUT/gpu-persp.medians")
deep_ns=$(median3 "$OUT/gpu-deep.medians")

{
    printf '%s=%s\n' renderer llvmpipe renderer_verified_launches "$LAUNCHES" \
        base_vs_plain_ae "$base_vs_plain_ae" neutral_vs_plain_ae "$neutral_vs_plain_ae" \
        persp_chamfer_rmse "$persp_chamfer_rmse" persp_face_rmse "$persp_face_rmse" \
        parallax_face_rmse "$parallax_face_rmse" one_code "$one_code" \
        gpu_plain_ms "$(ns_to_ms "$plain_ns")" gpu_persp_ms "$(ns_to_ms "$persp_ns")" \
        gpu_deep_ms "$(ns_to_ms "$deep_ns")" \
        gpu_persp_vs_plain_pct "$(pct_delta "$plain_ns" "$persp_ns")" \
        gpu_deep_vs_plain_pct "$(pct_delta "$plain_ns" "$deep_ns")"
} | tee "$OUT/metrics.txt"

finish
```

Before running, read `glass-optic-smoke-lib.sh` and check the following, adjusting only plumbing:
- `capture_identity` accepts `--binary` alongside the ones `build_binaries` records.
- `start_nested`, `spawn_probe`, `probe_rect` and `shot` take the binary first, as the iridescence smoke passes `"$NIRI"`.
- `write_config` reads `$WALL` when it runs (it interpolates it into `spawn-at-startup "swaybg"`), so reassigning `WALL` after calibration takes effect.
- `trace_run` goes through `start_nested`, so the renderer check covers the cost runs.
- `start_nested` appends each launch's output to `$OUT/weston.log` and `$OUT/niri.log`. The byte offsets taken before the launch depend on that. If the library redirects differently, read the logs it writes.

The checker command was verified to give exact 8 px runs along both axes. If `persp_face_rmse` sits right at one code, the perspective's shift is smaller than expected. Look at the crops before touching the fixture; do not loosen the asserts.

- [ ] **Step 2: Build the baseline binary** from the base commit in a detached, locked worktree:

```bash
cd "$(git rev-parse --path-format=absolute --git-common-dir)/.."   # the main checkout
git worktree add --detach .worktrees/material-77db8a-baseline 1c978f75
git worktree lock --reason "on WORK_ROOT storage" .worktrees/material-77db8a-baseline
mkdir -p .worktrees/material-77db8a-baseline/.cargo
cp .cargo/config.toml .worktrees/material-77db8a-baseline/.cargo/
setfattr -n user.com.dropbox.ignored -v 1 .worktrees/material-77db8a-baseline/.cargo
(cd .worktrees/material-77db8a-baseline && cargo build --release)
cp "$(cd .worktrees/material-77db8a-baseline && cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/release/niri" \
   "$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri"
sha256sum "$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri"
```

The target directory is shared, so the next branch build overwrites `release/niri`. Copy the binary out immediately, as above.

- [ ] **Step 3: Run the smoke** on the headless verification host, never the desktop session:

```bash
cd .worktrees/material-77db8a
OUT=$NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-$(git rev-parse --short HEAD) \
CAPTURE_TASK=material-77db8a BASE_NIRI=$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri \
docs/materials/scripts/glass-view-tilt-smoke.sh
```

Expected: `renderer=llvmpipe`, with `renderer_verified_launches` equal to the number of launches (5 captures plus 9 trace runs, so 14), `base_vs_plain_ae=0`, `neutral_vs_plain_ae=0`, the three RMSE asserts pass, and `metrics.txt` records the GPU cost. If `base_vs_plain_ae` is not zero, stop. The view-off path is not identical, and the difference is in `viewRefract`'s off branch or the `surfaceCosine` ternary. Fix it there, rebuild, and rerun; do not weaken the assert. If the pin check fails, the host cannot give an llvmpipe number: record that and stop, rather than timing on the GPU. A host-GPU cost can be added later as a separate run labelled as such.

- [ ] **Step 4: Add the review grid** to `ring-motion-clips.sh`. The spec asks for a small grid of `view-tilt` × `view-parallax` for the swing and `view-perspective` × `view-parallax` for the static lean, physical gain 1 included, plus crops inside the swing's 900 ms.
  - Header list, after `beam-bevel0`:

```bash
#   view-swing-tT-pG  the view's arrival swing alone (beam off) at
#                     `view-tilt T; view-parallax G`, T in {10, 25}, G in {1, 3}:
#                     a 2.5 s burst from before the gain, and 4x crops of the
#                     gaining pane's corner at 0.15, 0.3, 0.45 and 0.7 s
#   view-swing-beam   `view-tilt 25; view-parallax 3` with the beam on: the
#                     combined look, over the full beam run
#   view-persp-kK-pG  a still at `view-perspective 0.K; view-parallax G`, K in
#                     {25, 50}, G in {1, 3}: the static lean by screen position
```

  - Near the other settings:

```bash
VIEW_SWINGS="view-swing-t10-p1 view-swing-t10-p3 view-swing-t25-p1 view-swing-t25-p3"
VIEW_PERSPS="view-persp-k25-p1 view-persp-k25-p3 view-persp-k50-p1 view-persp-k50-p3"
SWING_S=2.5   # view-tilt-settle is 900 ms; the rest shows the view at rest
```

   Change the `SEQUENCES` default to append ` $VIEW_SWINGS view-swing-beam $VIEW_PERSPS`, and place these two lines before it.
  - `write_config`: change `glass { ${GLASS:-bevel 10;} }` to `glass { ${GLASS:-bevel 10;} ${VIEW_GLASS:-} }`, and append `view-tilt ${TILT:-0}; view-tilt-settle ${SETTLE:-900};` to the response block. Add `VIEW_GLASS, TILT, SETTLE` to its comment.
  - Configs, after the `beam-bevel0` line:

```bash
for name in $VIEW_SWINGS; do   # view-swing-t<T>-p<G>
    t=${name#view-swing-t}; t=${t%%-*}; g=${name##*-p}
    SPEED=0 TILT=$t VIEW_GLASS="view-parallax $g;" write_config "$OUT/$name.kdl"
done
for name in $VIEW_PERSPS; do   # view-persp-k<K>-p<G>, perspective 0.K
    k=${name#view-persp-k}; k=0.${k%%-*}; g=${name##*-p}
    VIEW_GLASS="view-perspective $k; view-parallax $g;" write_config "$OUT/$name.kdl"
done
TILT=25 VIEW_GLASS='view-parallax 3;' write_config "$OUT/view-swing-beam.kdl"
```

  - The `capture_identity` call: build its `--input` list for the view configs rather than listing eight by hand:

```bash
VIEW_INPUTS=()
for name in $VIEW_SWINGS view-swing-beam $VIEW_PERSPS; do VIEW_INPUTS+=(--input "$OUT/$name.kdl"); done
```

   Then pass `"${VIEW_INPUTS[@]}"` to `capture_identity` beside the existing `--input` arguments.
  - Sequences, after `seq_beam_bevel0`:

```bash
# 4x crops of the gaining pane's top-left corner through the swing: before,
# at and after the overshoot (the crossing is at a third of the settle).
swing_crops() {   # $1 = label
    local d=$OUT/$1-swing b=$OUT/$1 at f
    mkdir -p "$d"
    for at in 0.15 0.3 0.45 0.7; do
        f=$(awk -v at="$at" '{ d = $2 - at; if (d < 0) d = -d; if (best == "" || d < bd) { best = $1; bd = d } } END { print best }' "$b/frames.txt")
        [ -n "$f" ] || fail "$1: no frame near $at s in $b/frames.txt"
        magick "$b/$f" -crop 320x240+0+0 +repage -scale 400% "$d/at-$at.png"
        echo "$1-swing: at-$at is $f" >> "$OUT/clips.txt"
    done
}
# One focus gain on a two-pane scene, as beam_sequence, over SWING_S. The
# swing is judged from the frames inside its settle, so a host too slow to
# take six of them fails rather than producing a sheet of rest.
swing_sequence() {   # $1 = label, $2 = config
    local ids early
    start_nested "$2" "$1"
    spawn_kitty_to 2
    ids=($(kitty_ids))
    msg action focus-window --id "${ids[1]}"; settle
    burst_start "$1" "$SWING_S"
    msg action focus-window --id "${ids[0]}"
    burst_wait "$1"
    early=$(awk '$2 < 0.9' "$OUT/$1/frames.txt" | wc -l)
    [ "$early" -ge 6 ] || fail "$1: only $early frames inside the 900 ms settle"
    swing_crops "$1"
    stop_nested
}
# A still of the static lean: both panes settled, focus on the left one, and
# 4x crops of the left and right panes' outer top corners, where the lean is
# strongest.
view_still() {   # $1 = label, $2 = config
    local ids
    start_nested "$2" "$1"
    spawn_kitty_to 2
    ids=($(kitty_ids))
    msg action focus-window --id "${ids[0]}"; settle
    shot "$OUT/$1.png"
    magick "$OUT/$1.png" -crop 320x240+0+0 +repage -scale 400% "$OUT/$1-corner-left.png"
    magick "$OUT/$1.png" -gravity NorthEast -crop 320x240+0+0 +repage -scale 400% "$OUT/$1-corner-right.png"
    echo "$1: $OUT/$1.png" | tee -a "$OUT/clips.txt"
    stop_nested
}
```

  - Dispatch, before `*)`:

```bash
        view-swing-beam) beam_sequence view-swing-beam "$OUT/view-swing-beam.kdl" "$NIRI" ;;
        view-swing-t*)   swing_sequence "$s" "$OUT/$s.kdl" ;;
        view-persp-k*)   view_still "$s" "$OUT/$s.kdl" ;;
```

  - After the sequence loop, before the final `echo`, add the two grid sheets, one row per configuration:

```bash
if ls "$OUT"/view-swing-t*-swing/at-0.15.png >/dev/null 2>&1; then
    magick montage "$OUT"/view-swing-t*-swing/at-*.png -tile 4x -geometry 320x240+2+2 \
        -background '#111' "$OUT/view-swing-grid.png"
    echo "view-swing grid: $OUT/view-swing-grid.png (rows t10-p1, t10-p3, t25-p1, t25-p3; columns 0.15, 0.3, 0.45, 0.7 s)" | tee -a "$OUT/clips.txt"
fi
if ls "$OUT"/view-persp-k*-corner-left.png >/dev/null 2>&1; then
    magick montage $(for n in $VIEW_PERSPS; do echo "$OUT/$n-corner-left.png" "$OUT/$n-corner-right.png"; done) \
        -tile 2x -geometry 320x240+2+2 -background '#111' "$OUT/view-persp-grid.png"
    echo "view-persp grid: $OUT/view-persp-grid.png (rows k25-p1, k25-p3, k50-p1, k50-p3; left and right corners)" | tee -a "$OUT/clips.txt"
fi
```

- [ ] **Step 5: Render the clips** on the headless verification host:

```bash
NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT CAPTURE_TASK=material-77db8a \
SEQUENCES="view-swing-t10-p1 view-swing-t10-p3 view-swing-t25-p1 view-swing-t25-p3 view-swing-beam view-persp-k25-p1 view-persp-k25-p3 view-persp-k50-p1 view-persp-k50-p3" \
docs/materials/scripts/ring-motion-clips.sh
```

Expected: `clips: OK`. `clips.txt` lists `view-swing-grid.png`, `view-persp-grid.png`, the `view-swing-beam` gif and sheet, and one gif and sheet per swing configuration. If a swing sequence fails the six-frame check, the host is too loaded to judge the swing. Rerun it when the host is quiet, rather than lowering the bar.

- [ ] **Step 6: Commit**

```bash
tasks done material-cd0e1d
git add docs/materials/scripts/glass-view-tilt-smoke.sh docs/materials/scripts/ring-motion-clips.sh tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "test(material): view tilt identity smoke and review grid

The smoke pins the view-off renderer to the base commit's pixels over a
checkerboard, where a broken refraction would show, and records what
perspective and parallax change and cost on a verified llvmpipe. The clip
script gains a tilt by parallax grid of the arrival swing, with crops
inside its settle, and a perspective by parallax grid of stills.

Refs material-77db8a"
```

---

### Task 7: Documentation, evidence, Prism follow-up, and parking for review

**Files:**
- Modify: `docs/materials/render-pipeline.md` (§2 lifecycle paragraph, §3 table rows 3–6, §4 bullets, §5 table)
- Create: `docs/materials/2026-09-22-focus-view-tilt-evidence.md`
- Modify: `docs/specs/2026-09-22-focus-view-tilt-design.md` (status line)
- Modify: `tasks/material-77db8a.md` (through the `tasks` CLI only)

- [ ] **Step 1: Update `render-pipeline.md`.**
  - §2, after the beam paragraph, add: "The view swing (`view-tilt`) is another run on the animation loop, with no bucket clock and no deadline. The `Layout` starts it for the one focused window when an update pass observes a changed focus. It ends at exactly zero slope at `view-tilt-settle`, after which the fingerprint is constant."
  - §3 row 3: append "along the view ray (`view-perspective`, the swing), with the view's added displacement scaled by `view-parallax`" and add `view-perspective`, `view-parallax` to its parameters.
  - Row 4: "optical distance is `thickness / cos`, the cosine of the view ray on the structural normal".
  - Row 5: the ring lands "through the light-path index along the view ray".
  - Row 6: the glint's cosine is the view ray's.
  - Add a sentence under the table: "With every view key at its default the view is orthographic and each stage takes its original code path."
  - §5 table: add a row `| view-perspective, view-parallax, view-tilt, view-tilt-settle | (pending Prism task) | 3–6 |`, with the Prism task id from Step 3.

- [ ] **Step 2: Write the evidence doc** `docs/materials/2026-09-22-focus-view-tilt-evidence.md`. Record:
  - the smoke's `metrics.txt` values and run directory;
  - the clips' run directory, and the two grid sheets (`view-swing-grid.png`, `view-persp-grid.png`) with the configuration of each row;
  - the base commit `1c978f75` and the baseline binary's sha256;
  - the llvmpipe cost deltas (the smoke verified the pin), and any host-GPU run labelled as such;
  - one line saying the defaults stay off pending the owner's review of the sheets.

- [ ] **Step 3: File the Prism task** (from the main checkout, since the prism project owns it):

```bash
tasks add --project prism --status idea -p 2 --tag material --tag camera \
  "Expose the glass view keys: glass.view.perspective, parallax, tilt, tiltSettle" \
  -b "niri-material material-77db8a adds view-perspective (glass, 0-1, default 0), view-parallax (glass, 0-8, default 1), view-tilt (response, 0-45 degrees, default 0) and view-tilt-settle (response, 100-5000 ms, default 900). Map them as glass.view.perspective, glass.view.parallax, glass.view.tilt and glass.view.tiltSettle. Defaults for presets come from the owner's review of the view-swing and view-persp sheets; see docs/specs/2026-09-22-focus-view-tilt-design.md in niri-material."
```

Put the printed id in `render-pipeline.md` §5 and in a note on the task: `tasks note material-77db8a "Prism keys: <id>"`.

- [ ] **Step 4: Update the spec status line** to: `**Status:** implemented on material-77db8a (<first commit>…<last commit>); evidence in [2026-09-22-focus-view-tilt-evidence.md](../materials/2026-09-22-focus-view-tilt-evidence.md); defaults off pending the owner's review of the sheets.` Keep the revision history sentence.

- [ ] **Step 5: Full gate**

Run: `just gate`
Expected: check and test both pass.

- [ ] **Step 6: Commit**

```bash
tasks done material-dd1ad8
git add docs/materials/render-pipeline.md docs/materials/2026-09-22-focus-view-tilt-evidence.md docs/specs/2026-09-22-focus-view-tilt-design.md tasks/
just upstream-report
git add docs/materials/upstream-divergence.md
git commit -m "docs(material): the view ray in the pipeline, evidence for the view tilt

Refs material-77db8a"
```

- [ ] **Step 7: Park for the owner's look.** The task is not done until the owner has judged the sheets and the live result:

```bash
tasks park material-77db8a --waiting-on user --reason review \
  "Owner reviews the view-swing and view-persp grids (paths in the evidence doc), then live: merge material-77db8a to materials-26.04, package and install per the arch-package-pin rollout, set view-tilt about 20 and view-perspective about 0.3 in the material, judge on a focus change. Defaults and Prism preset values follow from that review."
```

Remove the baseline worktree once its binary has been copied out:

```bash
cd "$(git rev-parse --path-format=absolute --git-common-dir)/.."   # the main checkout
git worktree unlock .worktrees/material-77db8a-baseline
git worktree remove .worktrees/material-77db8a-baseline
```
