# Glass Config Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the glass parameter set inherited from the Quickshell
prototype with the twelve-parameter surface in the spec, before v1 acceptance
makes it a released compatibility surface.

**Architecture:** Six sequential tasks, each ending green. Tasks 1–4 change
the config surface one parameter group at a time, each rewriting its
consumers so the tree compiles at every commit. Task 5 adds the corner-radius
input, which is additive rather than a rename. Task 6 reconciles the docs.
Every parameter rename is a clean break: no aliases, no compatibility layer.

**Tech Stack:** Rust, knuffel (KDL decoding), smithay, GLSL 1.0 ES fragment
shaders, `cargo test`.

**Spec:** `docs/materials/2026-08-24-glass-config-surface-design.md`

## Global Constraints

- Branch from `v1-acceptance` at `c89f31c8`. This work lands **after** the v1
  parity pass; do not start it while that pass is unexecuted unless
  explicitly told to.
- No compatibility layer, no aliases, no deprecation shims for the removed
  names. `lip`, `shift-x`, `shift-y`, `samples`, and `distortion-scale` become
  unknown nodes that knuffel rejects.
- `cargo test --workspace` passes at the end of every task. CI runs
  `cargo test --all --exclude niri-visual-tests`, so `niri-visual-tests` must
  still **build** but is never asserted against.
- Conventional commits, no AI-attribution trailers. Stage named paths, never
  `git add .` or `git add -A`.
- Parameter ranges, verbatim from the spec: `bevel` 0–128, `offset-x` and
  `offset-y` −64–64, `thickness` 0–200, `distortion` 0–1, `distortion`'s
  `scale` 0–2. Defaults: `bevel 12`, `offset-x 6`, `offset-y 6`,
  `thickness 20`, `distortion 0 scale=0.5`.
- The one cross-parameter rule: `max(abs(offset-x), abs(offset-y)) <= bevel`,
  rejected with the message `offset must not exceed bevel`.
- Rendered appearance has **no check of any kind** in this repository,
  automated or manual. `niri-visual-tests` cannot show a material at all: it
  builds its `RenderCtx` with `xray: None`, and `Tile::render_inner` gates the
  material on `self.material.is_some() && ctx.xray.is_some()`. Never claim a
  rendered result was verified.

## File Structure

| File | Responsibility | Task |
| --- | --- | --- |
| `niri-config/src/material.rs` | `Glass`/`ResolvedGlass` fields, defaults, `resolve`, `Distortion`, `Material::validate` | 1, 3, 4 |
| `niri-config/src/lib.rs` | Call `Material::validate` in the `"material"` arm; config tests | 1, 3, 4 |
| `src/render_helpers/material.rs` | `material_frame` geometry, `bevel_depth`, `tap_count`, `InputFingerprint`, uniforms; slab constants deleted | 1, 2, 3, 5 |
| `src/render_helpers/shaders/material.frag` | Bevel depth from `mat_thickness`; per-corner SDF; radius derivation | 2, 5 |
| `src/render_helpers/shaders/mod.rs` | `mat_corner_radius` uniform declaration | 5 |
| `src/layout/tile.rs` | Jelly clamp via `bevel_depth`; feed the rendered corner radius at both element sites | 2, 5 |
| `docs/materials/material-config.md` | User-facing parameter reference | 6 |
| `docs/materials/2026-08-22-v1-design.md` | §4 parameter table | 6 |
| `docs/materials/2026-08-24-glass-config-surface-design.md` | Status header | 6 |
| `docs/materials/2026-08-24-v1-parity-design.md` | Mark “Config surface review” resolved | 6 |

---

### Task 1: `bevel` and `offset` replace `lip` and `shift`

The visible band becomes the parameter and the uniform inflation becomes
derived. This is a pure re-parameterization: at the defaults the computed
frame is identical to today's.

**Files:**
- Modify: `niri-config/src/material.rs`
- Modify: `niri-config/src/lib.rs`
- Modify: `src/render_helpers/material.rs`
- Test: `niri-config/src/lib.rs` (inline `mod tests`)
- Test: `src/render_helpers/material.rs` (inline `mod tests`)

**Interfaces:**
- Produces: `ResolvedGlass { bevel: f64, offset_x: f64, offset_y: f64, .. }`
  replacing `lip`, `shift_x`, `shift_y`. `pub(crate) fn
  Material::validate(&self) -> Result<(), String>` — its only caller is the
  `"material"` arm in the same crate. `material_frame`'s signature is
  unchanged.

- [ ] **Step 1: Write the failing config tests**

Add to the `mod tests` block in `niri-config/src/lib.rs`, beside
`material_rejects_out_of_range_parameter`:

```rust
    #[test]
    fn material_accepts_bevel_and_offset() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    bevel 20
                    offset-x -8
                    offset-y 4
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.bevel, 20.);
        assert_eq!(g.offset_x, -8.);
        assert_eq!(g.offset_y, 4.);
    }

    #[test]
    fn material_defaults_bevel_and_offset() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {}
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.bevel, 12.);
        assert_eq!(g.offset_x, 6.);
        assert_eq!(g.offset_y, 6.);
    }

    #[test]
    fn material_rejects_removed_lip_and_shift() {
        for node in ["lip 6", "shift-x 6", "shift-y 6"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{\n    glass {{\n        {node}\n    }}\n}}\n"
            ));
            assert!(err.contains("unexpected node"), "{node}: {err}");
        }
    }

    #[test]
    fn material_rejects_out_of_range_bevel() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    bevel 200
                }
            }
            "##,
        );
        assert!(err.contains("value must be between 0 and 128"), "{err}");
    }

    #[test]
    fn material_rejects_offset_exceeding_bevel() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    bevel 10
                    offset-y -12
                }
            }
            "##,
        );
        assert!(err.contains("offset must not exceed bevel"), "{err}");
    }

    #[test]
    fn material_offset_rule_applies_inside_an_include() {
        // Includes decode through the same `"material"` arm, so the check
        // must fire there rather than only on the root file.
        let err = parse_files_err(&[
            ("main.kdl", "include \"other.kdl\"\n"),
            (
                "other.kdl",
                "material \"frost\" { glass { bevel 4; offset-x 9; }; }\n",
            ),
        ]);
        assert!(err.contains("offset must not exceed bevel"), "{err}");
    }
```

`do_parse` and `do_parse_err` already exist in that `mod tests` block —
`material_parses_full_glass_block` uses `do_parse` — so no helper is needed.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri-config material_ -- --nocapture`
Expected: FAIL — `bevel`, `offset_x`, `offset_y` are not fields of
`ResolvedGlass`, so the new tests do not compile.

- [ ] **Step 3: Rename the config fields**

In `niri-config/src/material.rs`, in `struct Glass`, replace the three
`lip`/`shift_x`/`shift_y` fields with:

```rust
    #[knuffel(child, unwrap(argument))]
    pub bevel: Option<FloatOrInt<0, 128>>,
    #[knuffel(child, unwrap(argument))]
    pub offset_x: Option<FloatOrInt<-64, 64>>,
    #[knuffel(child, unwrap(argument))]
    pub offset_y: Option<FloatOrInt<-64, 64>>,
```

In `struct ResolvedGlass`, replace `lip`, `shift_x`, `shift_y` with:

```rust
    pub bevel: f64,
    pub offset_x: f64,
    pub offset_y: f64,
```

In `impl Default for ResolvedGlass`, replace the three corresponding lines
with:

```rust
            bevel: 12.,
            offset_x: 6.,
            offset_y: 6.,
```

In `Material::resolve`, replace the three corresponding lines with:

```rust
                bevel: g.bevel.map_or(d.bevel, |x| x.0),
                offset_x: g.offset_x.map_or(d.offset_x, |x| x.0),
                offset_y: g.offset_y.map_or(d.offset_y, |x| x.0),
```

- [ ] **Step 4: Add the cross-parameter validation**

Append to `impl Material` in `niri-config/src/material.rs`:

```rust
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
```

- [ ] **Step 5: Call it from the decode arm**

In `niri-config/src/lib.rs`, in the `"material" =>` arm, immediately after
`let part = Material::decode_node(node, ctx)?;` and before the duplicate-name
check, insert:

```rust
                    // `resolve` is infallible and runs after the config is
                    // already accepted, so a cross-parameter rule has to be
                    // checked here, where a decode error can still be
                    // emitted. Unlike `validate_material_refs` this needs no
                    // post-include deferral: the rule is per-definition.
                    if let Err(message) = part.validate() {
                        match node.arguments.first() {
                            Some(arg) => ctx.emit_error(DecodeError::unexpected(
                                &arg.literal,
                                "material",
                                message,
                            )),
                            None => {
                                ctx.emit_error(DecodeError::unexpected(node, "material", message))
                            }
                        }
                    }
```

- [ ] **Step 6: Rewrite `material_frame` to derive the inflation**

In `src/render_helpers/material.rs`, replace the body of `material_frame`
between the `round` closure and the `MaterialFrame` literal with:

```rust
    let offset_max = glass.offset_x.abs().max(glass.offset_y.abs());
    let inflate = ceil(glass.bevel - offset_max);
    let offset = Point::<f64, Logical>::from((round(glass.offset_x), round(glass.offset_y)));
    let slab = Rectangle::new(
        win_geo.loc - Point::from((inflate, inflate)) + offset,
        win_geo.size + Size::from((inflate * 2., inflate * 2.)),
    );
    let area = tex_geo.merge(slab);
```

and set the chamfer field to the bevel directly:

```rust
        chamfer: glass.bevel as f32,
```

Update the doc comment above `material_frame` to read:

```rust
/// The slab is the window rect inflated by `bevel - max(|offset-x|,
/// |offset-y|)` on every side, then slid by (`offset-x`, `offset-y`), so the
/// chamfer spans the whole widest visible band. The inflation is aligned to
/// physical pixels the way Shadow's is, so the element geometry and damage
/// stay exact.
```

Update `MaterialFrame::chamfer`'s doc comment to `/// Chamfer width in
logical px: the `bevel` parameter.`

- [ ] **Step 7: Rewrite the geometry tests**

In `src/render_helpers/material.rs`'s `mod tests`, replace
`material_frame_inflates_by_lip_and_shift` and
`material_frame_shift_slides_the_slab` with:

```rust
    #[test]
    fn material_frame_inflates_by_bevel_less_offset() {
        let mut glass = ResolvedGlass::default();
        glass.bevel = 10.;
        glass.offset_x = 0.;
        glass.offset_y = 0.;
        let win = Rectangle::new(Point::new(0., 0.), Size::new(200., 100.));

        let frame = material_frame(win, win, &glass, 1.);

        // Zero offset: the whole bevel becomes uniform inflation.
        assert_eq!(
            frame.area,
            Rectangle::new(Point::new(-10., -10.), Size::new(220., 120.))
        );
        assert_eq!(frame.slab_rect, [0., 0., 1., 1.]);
        assert_eq!(
            frame.geo_rect,
            [10. / 220., 10. / 120., 200. / 220., 100. / 120.]
        );
        assert_eq!(frame.area_size, [220., 120.]);
        assert_eq!(frame.chamfer, 10.);
    }

    #[test]
    fn material_frame_offset_slides_the_slab() {
        let mut glass = ResolvedGlass::default();
        glass.bevel = 12.;
        glass.offset_x = 6.;
        glass.offset_y = 6.;
        let win = Rectangle::new(Point::new(100., 50.), Size::new(200., 100.));

        let frame = material_frame(win, win, &glass, 1.);

        // inflate = 12 - 6 = 6, then slid by 6: the slab's top-left lands on
        // the window's top-left and the band is entirely right/bottom.
        assert_eq!(
            frame.area,
            Rectangle::new(Point::new(100., 50.), Size::new(212., 112.))
        );
        assert_eq!(frame.chamfer, 12.);
    }

    #[test]
    fn material_frame_defaults_match_the_pre_change_geometry() {
        // Pins the spec's re-parameterization claim: the shipped defaults
        // produce exactly the frame the old lip 6 / shift 6 constants did.
        let glass = ResolvedGlass::default();
        let win = Rectangle::new(Point::new(100., 50.), Size::new(200., 100.));

        let frame = material_frame(win, win, &glass, 1.);

        assert_eq!(
            frame.area,
            Rectangle::new(Point::new(100., 50.), Size::new(212., 112.))
        );
        assert_eq!(frame.chamfer, 12.);
    }
```

In `material_frame_aligns_to_physical_pixels`, replace the three assignments
with `glass.bevel = 5.3; glass.offset_x = 0.; glass.offset_y = 0.;` and add a
chamfer assertion, because this is the one case where the new parameterization
is deliberately not identical to the old one:

```rust
        // The inflation is still snapped up to the physical grid, but the
        // chamfer is the bevel exactly. The old code reused its rounded lip
        // for both, so `lip 5.3` at scale 2 reported a 5.5 chamfer; the band
        // width has no reason to be pixel-aligned.
        assert_eq!(frame.chamfer, 5.3);
``` In
`material_frame_merges_an_oversized_texture_footprint`, replace them with
`glass.bevel = 6.; glass.offset_x = 0.; glass.offset_y = 0.;`.

- [ ] **Step 8: Update the existing full-block test**

`material_parses_full_glass_block` at `niri-config/src/lib.rs:868` writes
every parameter and asserts against a whole `ResolvedGlass` literal, so it
holds the removed names and will not compile. In its KDL block replace

```kdl
                    lip 12
                    shift-x -8
                    shift-y 4
```

with

```kdl
                    bevel 20
                    offset-x -8
                    offset-y 4
```

`bevel 20` is the value the old pair produced — `lip 12 + max(8, 4)` — so the
case still describes the same slab, and `max(8, 4) <= 20` satisfies the new
rule. In the `ResolvedGlass` literal replace `lip: 12., shift_x: -8.,
shift_y: 4.,` with `bevel: 20., offset_x: -8., offset_y: 4.,`.

- [ ] **Step 9: Run the whole suite**

Run: `cargo test --workspace`
Expected: PASS. No other file reads `lip`, `shift_x`, or `shift_y` — the only
readers are `material_frame` and the tests, both edited above.

- [ ] **Step 10: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs \
    src/render_helpers/material.rs
git commit -m "feat(material): set the glass bevel directly"
```

---

### Task 2: `thickness` becomes the slab depth

`SLAB_DEPTH` disappears. The bevel depth becomes `min(bevel, thickness)`
everywhere it is used, in one shared function.

**Files:**
- Modify: `src/render_helpers/material.rs`
- Modify: `src/render_helpers/shaders/material.frag`
- Modify: `src/layout/tile.rs:1299`, `src/layout/tile.rs:1476`
- Test: `src/render_helpers/material.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `ResolvedGlass::bevel` from Task 1.
- Produces: `pub(crate) fn bevel_depth(chamfer: f64, thickness: f64) -> f64`,
  used by the jelly clamp in `tile.rs` and mirrored by the shader. Crate-
  visible, not `pub`: nothing outside `niri` calls it.

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `src/render_helpers/material.rs`:

```rust
    #[test]
    fn bevel_depth_is_the_shallower_of_bevel_and_thickness() {
        // Default: bevel 12, thickness 20 — the old constant was also 12, so
        // the shipped look is unchanged.
        assert_eq!(bevel_depth(12., 20.), 12.);
        // A bevel deeper than the old constant but shallower than the slab
        // is now honoured; the old code capped it at 12.
        assert_eq!(bevel_depth(15., 20.), 15.);
        // A slab shallower than the bevel now limits it; the old code
        // returned 12 regardless of thickness.
        assert_eq!(bevel_depth(12., 5.), 5.);
        // Both directions collapse to the bevel when it is the shallower.
        assert_eq!(bevel_depth(4., 20.), 4.);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p niri bevel_depth_is_the_shallower -- --nocapture`
Expected: FAIL — `bevel_depth` is not defined.

- [ ] **Step 3: Replace the constant with the function**

In `src/render_helpers/material.rs`, delete these two lines and the doc
comment above them:

```rust
pub const SLAB_DEPTH: f64 = 12.;
pub const SLAB_CORNER_RADIUS: f64 = 28.;
```

`SLAB_CORNER_RADIUS` has no reader anywhere in Rust — the shader carries its
own separate `SLAB_RADIUS`, which Task 5 deletes — so removing the Rust
constant now is safe and touches no shader line. In their place add:

```rust
/// The slab's bevel depth: the shallower of the chamfer and the slab.
///
/// A bevel cannot be deeper than the glass it is cut into. This mirrors the
/// shader's `min(mat_chamfer, mat_thickness)`; both must change together.
/// Pass `MaterialFrame::chamfer`, which is already clamped to what the shader
/// will draw — passing the configured bevel would disagree on small windows.
pub(crate) fn bevel_depth(chamfer: f64, thickness: f64) -> f64 {
    chamfer.min(thickness)
}
```

- [ ] **Step 4: Make `frame.chamfer` the effective chamfer**

`bevel_depth` is only a mirror of the shader if both start from the same
chamfer, and today they do not: the shader first clamps it to
`max(min(half_ext.x, half_ext.y) - 1.0, 0.0)` for windows too small to carry a
band (`src/render_helpers/shaders/material.frag:211`), while Rust hands it the
configured value. On a small window the jelly clamp would then be derived from
a bevel depth the shader never uses.

Apply the same clamp once, in `material_frame`, so the field means the
chamfer that is actually drawn. Replace the `chamfer:` line with:

```rust
        // Mirrors the shader's tiny-slab guard: a window too small to carry
        // the configured band gets whatever fits, and the field is the
        // chamfer actually drawn so `bevel_depth` and the shader agree.
        chamfer: {
            let max_chamfer = (slab.size.w.min(slab.size.h) / 2. - 1.).max(0.);
            glass.bevel.min(max_chamfer) as f32
        },
```

The shader's own clamp stays: it is now a no-op for every value Rust sends,
but it still guards the uniform against being set from anywhere else.

Add the case to `mod tests`:

```rust
    #[test]
    fn material_frame_clamps_the_chamfer_on_a_tiny_window() {
        let mut glass = ResolvedGlass::default();
        glass.bevel = 12.;
        glass.offset_x = 0.;
        glass.offset_y = 0.;
        // 8x8 window, inflated by the full bevel, is a 32x32 slab: half is
        // 16, so the band the shader will draw is 15, not the configured 12.
        // Larger windows are unaffected.
        let tiny = Rectangle::new(Point::new(0., 0.), Size::new(8., 8.));
        assert_eq!(material_frame(tiny, tiny, &glass, 1.).chamfer, 12.);

        glass.bevel = 40.;
        let frame = material_frame(tiny, tiny, &glass, 1.);
        // slab 88x88, half 44, so the band is capped at 43.
        assert_eq!(frame.chamfer, 43.);
    }
```

- [ ] **Step 5: Point the jelly clamp at it**

In `src/layout/tile.rs`, change the import on line 28 from `MaterialState,
SLAB_DEPTH,` to `MaterialState,` and add `bevel_depth,` to the alphabetical
import list on line 27. Then replace both occurrences of

```rust
let max_flex = 0.25 * SLAB_DEPTH.min(f64::from(frame.chamfer));
```

with

```rust
let max_flex = 0.25 * bevel_depth(f64::from(frame.chamfer), glass.thickness);
```

Both sites already bind `glass` on the line immediately above — `let glass =
&material.material().glass;` at `src/layout/tile.rs:1298` and `:1475` — so no
new binding is needed at either.

- [ ] **Step 6: Point the shader at the uniform**

In `src/render_helpers/shaders/material.frag`, delete **only** the
`SLAB_DEPTH` line and narrow the comment above it:

```glsl
// Slab corner radius in logical px, matching the legacy slab mesh. Task 5
// replaces this with the window's own radius.
const float SLAB_RADIUS = 28.0;
```

Leave `SLAB_RADIUS` and the `float r = min(SLAB_RADIUS, ...)` line untouched;
Task 5 owns that derivation and deletes the constant with it. Do not
substitute a bare `28.0` literal here — rewriting that line twice is the
churn the spec's sequencing section objects to.

Then change the bevel line to read from the thickness uniform:

```glsl
        float bevel = min(chamfer, mat_thickness);
```

- [ ] **Step 7: Validate the shader as GLSL**

`cargo test` and `cargo build` embed `material.frag` as a string; neither
compiles it. Validate it explicitly, the way the slice 2 plan does:

Run:
```sh
{ echo '#version 100'; cat src/render_helpers/shaders/material.frag; } \
    > target/material_check.frag \
    && glslangValidator -S frag target/material_check.frag
```
Expected: clean, no errors.

- [ ] **Step 8: Run the suite**

Run: `cargo test --workspace`
Expected: PASS, including the new `bevel_depth` and chamfer-clamp tests.

- [ ] **Step 9: Commit**

```bash
git add src/render_helpers/material.rs \
    src/render_helpers/shaders/material.frag src/layout/tile.rs
git commit -m "feat(material): make thickness the slab depth"
```

---

### Task 3: `samples` is derived from effect strength

**Files:**
- Modify: `niri-config/src/material.rs`
- Modify: `src/render_helpers/material.rs`
- Test: `niri-config/src/lib.rs` (inline `mod tests`)
- Test: `src/render_helpers/material.rs` (inline `mod tests`)

**Interfaces:**
- Produces: `fn tap_count(anisotropic_blur: f64, chromatic_aberration: f64)
  -> u8`, private to `src/render_helpers/material.rs` — its only caller is the
  uniform array in the same file, and its test is in the same module.
  `ResolvedGlass::samples` is removed; the `mat_samples` uniform stays and is
  fed from this function.

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `src/render_helpers/material.rs`:

```rust
    #[test]
    fn tap_count_follows_the_strongest_multi_tap_effect() {
        // Neutral: the shader's single-tap fast path.
        assert_eq!(tap_count(0., 0.), 1);
        // Below one tap's worth still gets a real average, not one sample.
        assert_eq!(tap_count(0.125, 0.), 2);
        // The prototype shipped `samples 4`; mid strength reproduces it.
        assert_eq!(tap_count(0.5, 0.), 4);
        assert_eq!(tap_count(0., 0.5), 4);
        // Full strength is the old maximum.
        assert_eq!(tap_count(1., 0.), 8);
        // The stronger effect wins.
        assert_eq!(tap_count(0.125, 1.), 8);
    }
```

Add to `mod tests` in `niri-config/src/lib.rs`:

```rust
    #[test]
    fn material_rejects_removed_samples() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    samples 8
                }
            }
            "##,
        );
        assert!(err.contains("unexpected node"), "{err}");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

`cargo test` accepts a single `TESTNAME` filter, so run two commands:

Run: `cargo test -p niri tap_count`
Expected: FAIL — `tap_count` is not defined.

Run: `cargo test -p niri-config material_rejects_removed_samples`
Expected: FAIL — `samples 8` still parses.

- [ ] **Step 3: Remove the parameter**

In `niri-config/src/material.rs`: delete the `samples` field from `Glass`,
the `samples` field from `ResolvedGlass`, the `samples: 4,` line from
`impl Default`, the `samples:` line from `Material::resolve`, and the whole
`Samples` newtype together with its `Default` and `DecodeScalar` impls.

- [ ] **Step 4: Add the derivation**

In `src/render_helpers/material.rs`, beside `bevel_depth`:

```rust
/// Refraction taps for the shader's multi-tap loop.
///
/// This is a cost dial, not an appearance one, so it is derived rather than
/// configured: a stronger effect needs more samples to stay smooth, and
/// dialing an effect down lowers its cost without a second knob. Zero
/// strength keeps the shader's single-tap fast path. Strength 0.5 yields 4,
/// the count the Quickshell prototype shipped as its default.
fn tap_count(anisotropic_blur: f64, chromatic_aberration: f64) -> u8 {
    let strength = anisotropic_blur.max(chromatic_aberration);
    if strength <= 0. {
        return 1;
    }
    (8. * strength).ceil().clamp(2., 8.) as u8
}
```

- [ ] **Step 5: Feed the uniform from it**

In `src/render_helpers/material.rs`, replace

```rust
            Uniform::new("mat_samples", f32::from(g.samples)),
```

with

```rust
            Uniform::new(
                "mat_samples",
                f32::from(tap_count(g.anisotropic_blur, g.chromatic_aberration)),
            ),
```

- [ ] **Step 6: Clear the tests that assert on the removed parameter**

These do not survive the removal, so they must go before the suite can be
green — not reactively afterwards.

In `niri-config/src/lib.rs`, delete `material_rejects_out_of_range_samples`
and `material_rejects_non_integer_samples` outright;
`material_rejects_removed_samples` from Step 1 replaces both. In
`material_parses_full_glass_block`, delete the `samples 8` line from the KDL
block and the `samples: 8,` line from the `ResolvedGlass` literal.

- [ ] **Step 7: Run the suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs \
    src/render_helpers/material.rs
git commit -m "feat(material): derive glass sample count"
```

---

### Task 4: `distortion`'s scale becomes a property

**Files:**
- Modify: `niri-config/src/material.rs`
- Test: `niri-config/src/lib.rs` (inline `mod tests`)

**Interfaces:**
- Produces: `pub struct Distortion { pub amount: FloatOrInt<0, 1>, pub scale:
  Option<FloatOrInt<0, 2>> }`. `ResolvedGlass::distortion` and
  `::distortion_scale` keep their existing `f64` types and uniform path.

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `niri-config/src/lib.rs`:

```rust
    #[test]
    fn material_accepts_distortion_with_scale() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    distortion 0.5 scale=1.5
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.distortion, 0.5);
        assert_eq!(g.distortion_scale, 1.5);
    }

    #[test]
    fn material_distortion_scale_defaults_without_the_property() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    distortion 0.5
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.distortion_scale, 0.5);
    }

    #[test]
    fn material_rejects_removed_distortion_scale_node() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    distortion-scale 1.5
                }
            }
            "##,
        );
        assert!(err.contains("unexpected node"), "{err}");
    }

    #[test]
    fn material_rejects_a_scale_without_an_amplitude() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    distortion scale=1.5
                }
            }
            "##,
        );
        // The amplitude is a required argument, so an orphan scale is a
        // parse error rather than an accepted no-op.
        assert!(!err.is_empty(), "{err}");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri-config distortion`
Expected: FAIL — `distortion 0.5 scale=1.5` does not parse; `scale` is not a
known property.

- [ ] **Step 3: Add the node type**

In `niri-config/src/material.rs`, above `struct Glass`:

```rust
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
```

- [ ] **Step 4: Use it**

In `struct Glass`, replace the `distortion` and `distortion_scale` fields
with:

```rust
    #[knuffel(child)]
    pub distortion: Option<Distortion>,
```

`ResolvedGlass` is unchanged. In `Material::resolve`, replace the two
corresponding lines with:

```rust
                distortion: g.distortion.map_or(d.distortion, |x| x.amount.0),
                distortion_scale: g
                    .distortion
                    .and_then(|x| x.scale)
                    .map_or(d.distortion_scale, |x| x.0),
```

- [ ] **Step 5: Fold the full-block test's distortion pair**

In `material_parses_full_glass_block` in `niri-config/src/lib.rs`, replace

```kdl
                    distortion 0.5
                    distortion-scale 1.5
```

with

```kdl
                    distortion 0.5 scale=1.5
```

The `ResolvedGlass` literal is unchanged: `distortion` and `distortion_scale`
keep their fields and values.

- [ ] **Step 6: Run the suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs
git commit -m "feat(material): make distortion scale a property"
```

---

### Task 5: The corner radius follows the window

The last surface change and the only one that is additive rather than a
rename. The radius is a shader input that changes without any `glass`
parameter changing — during an expand animation, and on a reload that edits
only `geometry-corner-radius` — so it must join `InputFingerprint` or the
element's pixels go stale.

**Files:**
- Modify: `src/render_helpers/shaders/material.frag`
- Modify: `src/render_helpers/shaders/mod.rs`
- Modify: `src/render_helpers/material.rs`
- Modify: `src/layout/tile.rs`
- Test: `src/render_helpers/material.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `bevel_depth` from Task 2, `ResolvedGlass::bevel` from Task 1.
- Produces: `InputFingerprint::corner_radius: CornerRadius`. `element`'s
  signature does not change — it already receives the fingerprint and reads
  the radius from it, so the radius has exactly one source.

- [ ] **Step 1: Write the failing damage test**

Add to `mod tests` in `src/render_helpers/material.rs`:

```rust
    #[test]
    fn corner_radius_change_advances_the_commit() {
        // The radius is a uniform: smithay's damage tracker cannot see it,
        // and `apply_resolved` bumps only on a `glass` diff, so a config
        // reload that edits `geometry-corner-radius` alone would otherwise
        // leave the element's pixels stale.
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let mut f = fingerprint2(1, 1, &background_id, 1, &backdrop_id);
        let a = state.advance_commit(RenderTarget::Output, f.clone());
        f.corner_radius = CornerRadius {
            top_left: 16.,
            top_right: 16.,
            bottom_right: 16.,
            bottom_left: 16.,
        };
        let b = state.advance_commit(RenderTarget::Output, f);

        assert_ne!(a, b);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p niri corner_radius_change_advances_the_commit`
Expected: FAIL — `InputFingerprint` has no `corner_radius` field.

- [ ] **Step 3: Carry the radius through the Rust side**

In `src/render_helpers/material.rs`, widen the existing import on line 6 to
`use niri_config::{CornerRadius, ResolvedGlass, ResolvedMaterial};` —
`CornerRadius` is re-exported at the crate root, as
`niri-visual-tests/src/cases/gradient_area.rs` already relies on. Then add to
`InputFingerprint`:

```rust
    /// The window's rendered corner radius, which the shader rounds the slab
    /// to. Animated by the tile, so it changes with no config change.
    pub corner_radius: CornerRadius,
```

Add `corner_radius: CornerRadius::default(),` to the `fingerprint2` test
helper's `InputFingerprint` literal.

`element` already takes `inputs: InputFingerprint`, so it needs no new
parameter — read the radius before `advance_commit` consumes the fingerprint.
In `MaterialState::element`, above the `MaterialRenderElement` literal:

```rust
        let corner_radius = inputs.corner_radius;
```

and add `corner_radius,` to that literal. Add the matching field to
`struct MaterialRenderElement`, beside `frame`:

```rust
    /// The window's rendered corner radius, in `CornerRadius` order.
    corner_radius: CornerRadius,
```

Then add the uniform beside `mat_chamfer` in the uniform array:

```rust
            Uniform::new("mat_corner_radius", <[f32; 4]>::from(self.corner_radius)),
```

- [ ] **Step 4: Declare the uniform**

In `src/render_helpers/shaders/mod.rs`, after the `mat_chamfer` line:

```rust
                UniformName::new("mat_corner_radius", UniformType::_4f),
```

- [ ] **Step 5: Feed the rendered radius from the tile**

In `src/layout/tile.rs`, add a `corner_radius` line to both
`InputFingerprint` literals (near lines 1346 and 1523 before this task's
edits). The `material.element(` calls are unchanged.

The two branches fit the radius against different geometry, and the material
must use whatever its own branch renders with. In the **resize** branch's
literal (near 1346):

```rust
                                            corner_radius: radius
                                                .fit_to(area.size.w as f32, area.size.h as f32),
```

In the **normal** branch's literal (near 1523), reuse the binding the sibling
element already computed — `clip_radius` is declared at
`src/layout/tile.rs:1427` and is still in scope there, and sharing it makes
drift between the two consumers impossible:

```rust
                                    corner_radius: clip_radius,
```

Two things make this the right value and both are load-bearing:

`radius` is the binding declared at `src/layout/tile.rs:1195` as
`self.window.geometry_corner_radius().scaled_by(1. - expanded_progress as
f32)`. Do **not** substitute `self.window.geometry_corner_radius()` — the
unscaled value leaves glass rounded while the window squares off into
fullscreen.

`fit_to` then applies the CSS corner-overlap rule
(`niri-config/src/appearance.rs:174`), a single proportional reduction across
all four corners derived from adjacent-pair sums — not a per-corner clamp. On
a 100 px edge, radii 80 and 40 become 66.7 and 33.3, because
`100 / (80 + 40)` scales both. The sibling elements already do exactly this:
the normal branch computes `clip_radius = radius.fit_to(window_size.w as f32,
window_size.h as f32)` at `src/layout/tile.rs:1427` — the binding reused
above — and
`ResizeRenderElement::new` fits against its `area` argument, which it holds as
`curr_geo` (`src/render_helpers/resize.rs:37, 85`) — the same `area` passed at
`src/layout/tile.rs:1251`. Passing the unfitted `radius` would round the glass
differently from the window it sits under, which is the whole defect this task
exists to remove.

- [ ] **Step 6: Make the shader per-corner**

In `src/render_helpers/shaders/material.frag`, add the uniform beside
`mat_chamfer`:

```glsl
uniform vec4 mat_corner_radius;
```

Replace `sdRoundedBox` and `sdRoundedBoxGrad` with per-corner forms. The
vec4 is in niri's `CornerRadius` order — top-left, top-right, bottom-right,
bottom-left — and `p` is centred with +y down:

```glsl
// Selects this quadrant's radius from a CornerRadius-ordered vec4
// (top-left, top-right, bottom-right, bottom-left).
float cornerRadius(vec2 p, vec4 r) {
    float top = p.x < 0.0 ? r.x : r.y;
    float bottom = p.x < 0.0 ? r.w : r.z;
    return p.y < 0.0 ? top : bottom;
}

float sdRoundedBox(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;
}

// Outward gradient of sdRoundedBox — the silhouette's edge direction.
vec2 sdRoundedBoxGrad(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
    vec2 q = abs(p) - b + vec2(r);
    vec2 g;
    if (q.x > 0.0 && q.y > 0.0)
        g = q / length(q);
    else if (q.x > q.y)
        g = vec2(1.0, 0.0);
    else
        g = vec2(0.0, 1.0);
    return g * vec2(p.x >= 0.0 ? 1.0 : -1.0, p.y >= 0.0 ? 1.0 : -1.0);
}
```

Delete the `SLAB_RADIUS` constant and the comment above it, which Task 2 left
in place for this task. Then in `slabSurface` replace the
`float r = min(SLAB_RADIUS, min(half_ext.x, half_ext.y));` line and the
`float ri = max(r - chamfer, 1.0);` line with the inverted derivation — the
inner face takes the window's radius and the outer follows:

```glsl
    // `mat_corner_radius` is fitted to the *window*, but neither SDF box is
    // the window: the inner face is the window narrowed by twice the offset
    // (defaults, 100 px window: slab 112, inner face 88). Refit to the box
    // actually being drawn. The reduction is proportional across all four
    // corners, never per-corner clamping, which would shrink a large radius
    // whose neighbour is small.
    vec4 inner_r = fitRadii(mat_corner_radius, inner_half);
    // The outer ring then inherits non-overlap for free: adjacent outer radii
    // sum to (inner_a + inner_b + 2 * chamfer) against an outer edge of
    // (inner_edge + 2 * chamfer), and the fit above gives
    // inner_a + inner_b <= inner_edge.
    vec4 outer_r = inner_r + vec4(chamfer);
```

Move those two lines below the existing `inner_half` computation, and update
the three call sites to pass `outer_r` for the outer box and `inner_r` for
the inner box and its gradient.

`fitRadii` is new. Add it beside `cornerRadius`, above `sdRoundedBox`. It is
`CornerRadius::fit_to` (`niri-config/src/appearance.rs:174`) transcribed —
the same four adjacent pairs in the same `CornerRadius` order, with guards so
a zero pair cannot divide by zero:

```glsl
// CSS corner-overlap fitting, mirroring CornerRadius::fit_to: one
// proportional reduction across all four corners, taken from the tightest
// adjacent-pair sum. Radii are (top-left, top-right, bottom-right,
// bottom-left); `half_ext` is half the box being fitted to.
vec4 fitRadii(vec4 r, vec2 half_ext) {
    vec2 edge = 2.0 * half_ext;
    float k = min(min(edge.x / max(r.x + r.y, 1e-6),
                      edge.x / max(r.w + r.z, 1e-6)),
                  min(edge.y / max(r.x + r.w, 1e-6),
                      edge.y / max(r.y + r.z, 1e-6)));
    return r * min(1.0, k);
}
```

Fitting twice — once to the window in Rust, once to the inner face here — is
not a conflict. Proportional reduction composes: fitting to the window and
then to the smaller inner face yields exactly what fitting to the inner face
alone would. The Rust-side fit stays because the fingerprint must carry the
radius the window renders with.

- [ ] **Step 7: Validate the shader as GLSL**

This task rewrites `sdRoundedBox`, its gradient, and the radius derivation,
and adds `fitRadii` — none of which any Rust build checks.

Run:
```sh
{ echo '#version 100'; cat src/render_helpers/shaders/material.frag; } \
    > target/material_check.frag \
    && glslangValidator -S frag target/material_check.frag
```
Expected: clean, no errors.

- [ ] **Step 8: Run the suite**

Run: `cargo test --workspace`
Expected: PASS, including `corner_radius_change_advances_the_commit`.

- [ ] **Step 9: Confirm the viewer still builds**

The shader and element changes must not break the excluded package.

Run: `cargo build --package niri-visual-tests`
Expected: builds clean.

- [ ] **Step 10: Commit**

```bash
git add src/render_helpers/shaders/material.frag \
    src/render_helpers/shaders/mod.rs src/render_helpers/material.rs \
    src/layout/tile.rs
git commit -m "feat(material): follow the window's corner radius"
```

---

### Task 6: Reconcile the documentation

**Files:**
- Modify: `docs/materials/material-config.md`
- Modify: `docs/materials/2026-08-22-v1-design.md`
- Modify: `docs/materials/2026-08-24-glass-config-surface-design.md`
- Modify: `docs/materials/2026-08-24-v1-parity-design.md`

**Interfaces:**
- Consumes: the implementing commits from Tasks 1–5.

- [ ] **Step 1: Rewrite the user-facing reference**

In `docs/materials/material-config.md`: replace the parameter table with the
spec's twelve-row table; delete the provisional caveat added in `1ae1859f`
(the paragraph beginning “This surface is provisional”); change the example's
`thickness 20` block to also show `bevel` and `offset-x`/`offset-y`; replace
the “fixed 12 px depth and 28 px corner radius” paragraph with the
window-following radius rule and the `bevel - max(abs(offset))` inflation;
and add `offset must not exceed bevel` to the validation-errors list.

Add `geometry-corner-radius 16` to the example's `window-rule`, beside its
`match` and `material` lines. The spec states that this example sets it —
without the line, the claim that users get the prototype's rounded look by
setting it on the same rule has no worked example to point at.

- [ ] **Step 2: Update the design's parameter table**

In `docs/materials/2026-08-22-v1-design.md` §4, update the parameter table to
match. `niri-config/src/material.rs`'s module comment cites this section as
the surface's specification, so the two must agree.

- [ ] **Step 3: Update the spec's own status**

`docs/materials/2026-08-24-glass-config-surface-design.md`'s header records
the plan as drafted and execution as pending, gated behind the v1 parity pass.
Replace that with the implemented status and the literal commits from Tasks
1–5. A design doc's status is a claim about the past; the merge is the moment
it goes stale.

- [ ] **Step 4: Mark the review resolved**

In `docs/materials/2026-08-24-v1-parity-design.md`, in the “Config surface
review” section, replace the sentence naming the design as drafted with one
recording it as implemented, citing the literal commits from Tasks 1–5.
Note there that the rendered radii remain without an automated check, so the
gap is recorded where the review is closed rather than only in the plan.

- [ ] **Step 5: Grep for drift**

```sh
rg -nw 'lip|shift-x|shift-y|samples|distortion-scale|SLAB_DEPTH|SLAB_CORNER_RADIUS' \
    docs/materials niri-config/src src/render_helpers src/layout \
    --glob '!docs/materials/plans/2026-08-23-slice2.md' \
    --glob '!docs/materials/plans/2026-08-24-v1-parity.md' \
    --glob '!docs/materials/2026-08-24-v1-parity-design.md'
```

`-w` is load-bearing: without it a bare `lip` also matches `clip`,
`clipboard`, and `clipPath` for more than 800 hits, and "read every result"
becomes unfollowable. The excluded files are the parity pass's records, which
describe the surface as it stood when they were written and must keep the old
names. Read every remaining hit: current reference and status surfaces must
use the new names.

- [ ] **Step 6: Final verification and commit**

```bash
cargo test --workspace
cargo build --package niri-visual-tests
git add docs/materials/material-config.md \
    docs/materials/2026-08-22-v1-design.md \
    docs/materials/2026-08-24-glass-config-surface-design.md \
    docs/materials/2026-08-24-v1-parity-design.md
git commit -m "docs(material): document the glass config surface"
```

Expected: the workspace suite passes and the viewer builds.

---

## Known gaps

Stated so an executor does not mistake them for oversights:

- The shader's quadrant selection and the rendered slab radii are **never
  seen** by this plan. There is no automated check, and there is no cheap
  manual one either: `niri-visual-tests` builds its `RenderCtx` with
  `xray: None`, and `Tile::render_inner` gates the material on
  `ctx.xray.is_some()`, so a case added there renders a plain tile. Showing a
  material in the viewer would mean constructing an `Xray` with background and
  backdrop buffers — more than inspection scaffolding is worth. Closing this
  properly means a headless capture harness in the style of the v1 parity
  pass, which is out of scope here.
- `bevel_depth` in Rust and `min(mat_chamfer, mat_thickness)` in GLSL are the
  same rule expressed twice. The Rust side is tested; the GLSL side is held
  correct by the comment on `bevel_depth` naming its twin. A single source
  would mean passing the depth as a uniform, which is a reasonable follow-up
  but adds a uniform for one `min`.
