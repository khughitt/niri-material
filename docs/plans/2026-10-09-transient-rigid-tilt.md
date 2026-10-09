# Transient Rigid Tilt Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** draft, revised after plan review rounds 1–3 (2026-10-09). Design task `material-abc08c`; implementation goal `material-6901e0`.

**Goal:** On focus gain, swing the pane once in depth and settle it back to exactly face-on, with window content and glass moving as one rigid body whose rounded edge and side wall show while it turns, and with today's pixels and code path at rest.

**Architecture:** One f64 mirror, `src/render_helpers/material/view/` (`mod.rs`: curve, pose, camera, projection, near clip, footprint, wall rule; `solid.rs`: the traced convex slab, bracketing and outline distance), is the reference that `prelude.frag` follows line for line, and the source of the element's footprint. The `Tile` owns a snapshotted `ViewSwing`, started once per focus gain by the `Layout`'s `FocusOrigin` (ported from `material-77db8a`), and hands a per-frame `ViewPose` to the material element. The element grows its geometry to the footprint, cropped to the render view, and uploads five view uniforms. The shader takes one uniform branch (`mat_view_on`) that computes the slab-local ray, the decal, the hit and the outline coverage, then feeds today's stages their inputs; every view quantity is today's value plus a correction that is exactly zero at the identity.

**Tech Stack:** Rust (workspace crates `niri-config`, `niri`), knuffel (KDL), glam (`DVec2`, `DVec3`), smithay GLES renderer, GLSL ES 1.00, the headless test backend (`src/tests/`, surfaceless EGL), Python 3 with Pillow for the still sheet, bash on the `glass-optic-smoke-lib.sh` capture library for the cost A/B and the clip, `just` front door.

**Spec:** `docs/specs/2026-10-09-transient-rigid-tilt-design.md` (accepted at spec review round 4). Read it alongside this plan: section numbers below are the spec's. Read `docs/materials/render-pipeline.md` before touching the shaders (AGENTS.md).

## Global Constraints

- Branch `material-abc08c`, worktree `.worktrees/material-abc08c` (it exists; reuse it). Run every command from the worktree; paths shown to the user are prefixed `.worktrees/material-abc08c/`. Execution's setup runs the worktree skill's baseline (`just setup`, then `just test-fast`) before the first code change; the first build is a full one.
- Conventional commits; no AI-attribution or session trailers. Each plan task is a child of `material-6901e0`, in order; the ids are in the task table below. `tasks start <id>` before a task, `tasks done <id> "<what landed>"` in the same commit as its code, `tasks check` before every commit.
- Run `cargo fmt --all` before every `git add` of Rust: the plan's code blocks are not rustfmt-formatted, and the hook runs `cargo fmt --all -- --check`.
- Every commit passes the pre-commit hook (`just check`). Never bypass it. A new file counts as a fork path: after `git add`, run `python3 tools/upstream-report && git add docs/materials/upstream-divergence.md` before `git commit`.
- Every test run goes through `just`: `just test-one -p <crate> <filter>` while working (nextest arguments; an ignored test runs as `just test-one -p <crate> --run-ignored only <filter>`), `just test-fast` before each commit. Never call cargo test or nextest directly.
- **The rest path stays textually today's.** Every view-dependent expression in the shader lives in a branch on `g_view_on` (or `mat_view_on == 1`) that replaces or adds to today's value; today's expression is left as written. In the view branch, every quantity is today's value plus a correction written so that it is exactly zero at θ = 0 and w = 0 (`R − I` as `cos θ − 1 = −2 sin²(θ/2)` and `sin θ`, `f − 1`, `dir.xy`, the quad offset). Never recompute a rest quantity from a corrected one (for example `p = v * mat_area_size` after `v` was shifted); carry `v` and `p` side by side and add the shift to each.
- Constants shared by `view/` and `prelude.frag` carry the same names (`VIEW_NEAR_CLIP`, `VIEW_GOLDEN`, `VIEW_GOLDEN_STEPS`, `VIEW_BISECT_STEPS`); `view::solid::tests::the_shader_names_the_same_constants_and_formulas` pins them and the mirrored lines. Change both together.
- After any `.frag` change, validate both assembled programs offline (expect exit 0 for each):
  ```bash
  S=src/render_helpers/shaders
  { echo '#version 100'; echo 'precision highp float;'; cat $S/material/common.frag $S/material/prelude.frag
    for o in saturation noise aurora reflection edge_highlight iridescence; do cat $S/material/$o.frag; done
    cat $S/material/main.frag; } > target/material.frag && glslangValidator -S frag target/material.frag
  { echo '#version 100'; echo 'precision highp float;'; echo 'varying vec2 v_coords;'; echo 'uniform sampler2D tex;'
    cat $S/material/common.frag $S/material/noise.frag $S/grain.frag; } > target/grain.frag && glslangValidator -S frag target/grain.frag
  ```
- Keys: `view-tilt` is `FloatOrInt<0, 20>` (degrees, default 0, off), `view-tilt-settle` is `FloatOrInt<100, 5000>` (ms, default 900), `view-distance` is `Milli<1500, 50000>` (κ, default 6), all in the response block, inherited from `default`. The niri default for `view-tilt` stays 0; Prism writes starting values after the clip (`prism-2a8fea`).
- Uniforms: `mat_view_on` (int), `mat_view` (vec4: cos θ − 1, sin θ, λ = w / (κE), wall spill gain), `mat_view_dir` (vec2: d), `mat_view_quad` (vec4: drawn-quad UV to area UV, offset then scale), `mat_view_seed` (vec4: framebuffer px per logical px, row-major 2×2). At rest: 0, (0, 0, 0, 1), (1, 0), (0, 0, 1, 1), the frame's basis.
- Evidence artifacts (dumps, sheets, captures) go under `EV=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/<run-stamp>`, never into the repository. Evidence documents that cite them are committed under `docs/materials/`.
- Host use: Tasks 13 and 15 produce artifacts the owner must judge, and stop for that review. Tasks 14 and 15 need an idle host on the headless lane: on a `capture_preflight` refusal, record a `run:` note for the attempt and park with `--reason quiet` (the exact commands are in those tasks). Nothing in this plan opens a window on the live desktop.
- The test renderer may be llvmpipe. Task 8 records the renderer string; if it is not the NVIDIA driver, Task 14's capture lane is the view path's first compile on the real driver, and a compile failure there stops the task.

## Task table

| Task | Id | Deliverable |
| --- | --- | --- |
| 1 | `material-1971fd` | The three response keys in niri-config |
| 2 | `material-44d5be` | Swing curve, weight, pose and direction (`view/mod.rs`) |
| 3 | `material-d7ebec` | Camera, projection, near clip and footprint (`view/mod.rs`) |
| 4 | `material-d74c02` | The traced slab and its bracketing; the wall-reveal rule (`view/solid.rs`) |
| 5 | `material-526ae1` | Outline distance and its bound (`view/solid.rs`) |
| 6 | `material-e44538` | Swing state on the tile, focus origin, layout tests |
| 7 | `material-392fbc` | Element footprint, view crop, quad mapping, uniforms, fingerprint, culling |
| 8 | `material-a0e3fb` | Shader: camera ray, decal, face hits, projection into every sample, `Surface.view`, seeds |
| 9 | `material-564316` | Shader: bracketing, outline coverage, wall surface |
| 10 | `material-4b1449` | Frozen renders: lifecycle, geometry, containment, view crop |
| 11 | `material-99c7e8` | Frozen renders: walls, grazing edges, interaction sequences |
| 12 | `material-c73cf0` | Pre-change baseline identity; docs |
| 13 | `material-7e3447` | Owner stills (stops for the owner's verdict) |
| 14 | `material-2283b8` | GPU cost A/B on the capture lane (and the conditional contingency) |
| 15 | `material-c5f9d0` | Owner clip on the nested lane (stops for the owner's verdict) |

Each task is a child of `material-6901e0` filed with this plan's step heading, and depends on the one before it and on `material-abc08c`, the design task, which closes when this plan is accepted. None is ready before then.

## Deviations from the spec, decided here

- **`view.rs` is a directory module** (`view/mod.rs` and `view/solid.rs`), split by responsibility: the motion and camera, which the tile and the element use, and the traced solid, which only the mirror tests and the shader use. §7's unit table is otherwise unchanged.
- **The depth parameter.** The ray's segment is parameterised by depth, `q(t) = h₀ + t·T·ρ` with `ρ = r / (−r.z)`, so `z = −tT`. Fixed `t` is then a fixed plane `z = −tT`, and §4.5's interior form and its face- and back-plane end form share one Jacobian (`∂q/∂s` along a `z` plane). Only a minimiser at a centre-line end uses the `x = cx` or `y = cy` plane.
- **Each piece evaluates `Ψ` with its own quadrant's radius**, chosen from the piece's midpoint, rather than from the sign of each sample. That makes every piece exactly the single-radius convex function of §4.1 up to and including its ends, so a minimiser at a seam end belongs to its piece. At the identity no seam is crossed and the midpoint is the pixel itself, so the radius is today's.
- **Top or wall** is decided by `Ψ` at the rim depth as well as by the entry depth: a hit is a wall hit when its entry lies below the rim and the ray is outside at the rim depth. At the identity `Ψ` at the rim depth is exactly `σ`, so a covered pixel is never a wall hit there, whatever the bisection's last step. A ray whose pieces all stay above 0 shades at its minimiser, classified the same way, so a miss inside the coverage band takes the colour of the surface it nearly hit.
- **A sure miss skips the bracketing**: since `σ` is 1-Lipschitz and the bevel term is non-negative, `Ψ(q(t)) ≥ σ(h₀) − T·|ρ.xy|`; when that bound is positive the ray misses with coverage exactly 0. §6 asks that bracketing run only near the outline; this is the test that makes it so. At the identity the bound is `σ(s)`, today's coverage gate.
- **A face hit near the outline takes the outline coverage** (§4.1 step 4, §4.5): the flat-face shortcut keeps the shading at `h₀`, but within 4 physical px of the face plane's cross-section it traces the ray for `m`. With bevel 0 the face reaches the silhouette, and the slab-local coverage there errs past the side bound (plan review round 1: 0.087 px at a 100 px slab, 20°, κ 1.5). At the identity the traced `m` is exactly `σ`, so the rest pixels are today's.
- **The swing's visibility uses its footprint** (§5.1 "Hidden panes"): the run is reported ongoing while the resting slab or the swing's footprint is in view, so a lifted wall that alone reaches the view keeps its frames.
- **The intersection reference is certified** (§8.1 "Bracketing"): from Ψ's convexity, a sample at or below 0 proves a hit and chord extensions that stay above 0 prove a miss; what neither proves is reported unresolved, never counted as a miss. Closed-form entries (a planar chamfer, a box) control both the mirror and the reference.
- **The background effect's neutrality** (§5) is read from the window's surface state at its last render (`BackgroundEffect::is_visible`), through a new `LayoutElement::background_effect_visible`, rather than from the rule alone: a client's blur region turns an unset rule visible.
- **The grain seed's framebuffer orientation** (§4.4) is the sign pattern of the frame's projection, uploaded per draw as `mat_view_seed`. smithay's projection is a signed permutation scaled to NDC, so its signs are the framebuffer's axes.
- **The wall spill** (§4.3) is on, at gain 1, and `mat_view.w` carries the gain so the still sheet can turn it off through a test hook. The owner's pick (§8.3) then fixes it; no key is added.
- **Two test probes** ride on the wall spill uniform: a gain of −1 paints wall hits magenta and −2 writes the coverage, so the frozen renders read the shader's wall classification and coverage directly (§8.2 "Walls", "Grazing edges"). Only the test hook uploads a negative gain.
- **The close snapshot's crop** (§8.2 "View crop") is checked through a real unmap mid-swing: the close animation's first frame must match the frame before it, which a wrongly cropped snapshot (an output rect taken in the snapshot's (0, 0) coordinates) would cut.
- **The pinned pre-change baseline** (§8.2 "Rest identity") is the `glass_edge` and `ring_look` dumps taken at the branch's base and at the branch head, compared byte for byte (Task 12). Those harnesses carry the beam, aberration, reflection, grain, opaque and translucent content. Within the branch, Task 10 compares a frame after a swing with a never-swung render.

## Review Focus

Inputs the spec implies but its tests do not name, most likely first; each line's test is added to the owning task.

1. **A swing on a pane whose decal reaches the near clip on a later frame only** (a CSD shadow or a subsurface far past the slab, at 20°, κ = 1.5): the footprint must stay finite on every frame of the run, not only at the peak. Test: Task 3 `the_footprint_is_finite_on_every_frame_of_a_run`.
2. **A focus gain while the pane's open animation is still running and the swing would start on its last frame**: no swing, and none started later for the same gain. Test: Task 6 `an_opening_pane_takes_no_swing_for_that_gain`.
3. **Fractional scale** (1.25, 1.5): the footprint and the crop round out to physical pixels, and the quad mapping is computed from the physical rects, so the drawn quad's UV covers the resting area exactly. Test: Task 7 `the_drawn_quad_maps_the_resting_area_at_fractional_scale`.
4. **A 90°-rotated or flipped output**: the grain seed basis follows the framebuffer, so the grain still moves with the glass. Test: Task 7 `the_seed_basis_follows_the_projection`.
5. **`view-tilt` set on an inactive material only** (the `is-focused` rule picks a different material at the gain): the swing reads the response of the material the tile holds when the `Layout` observes the gain, and a material swap mid-run keeps the snapshot. Test: Task 6 `a_material_swap_mid_swing_keeps_the_snapshot`.

---

## File structure

| Path | Responsibility |
| --- | --- |
| `niri-config/src/material/mod.rs` | `Response.view_tilt`, `view_tilt_settle`, `view_distance`; `ResolvedResponse` fields, defaults, overrides; `response_fields()` |
| `niri-config/src/material/pipeline.rs` | `VIEW_RESPONSES` on the `slab` stage; the destructuring guard |
| `resources/materials/pipeline.json` | regenerated |
| `niri-config/src/lib.rs` | parse, default, bound and inheritance tests |
| `src/render_helpers/material/view/mod.rs` | curve, `ViewPose`, directions, `Camera`, projection, near clip, footprint, wall rule, test hooks |
| `src/render_helpers/material/view/solid.rs` | `Solid`, `Segment`, pieces, bracketing, `Trace`, outline distance, shader pins |
| `src/render_helpers/material/ring.rs` | `hash01` made `pub(super)` |
| `src/render_helpers/material/mod.rs` | `pub mod view;`, `MaterialFrame.slab`/`tex`, `ElementView`, drawn area, quad, view uniforms, seed basis, fingerprint field |
| `src/render_helpers/background_effect.rs` | `surface_effect_visible` |
| `src/layout/mod.rs` | `mod focus_origin;`, `focus_origin` field, the observation in `update_render_elements`, `LayoutElement::background_effect_visible` |
| `src/layout/focus_origin.rs` | ported from `material-77db8a` |
| `src/layout/monitor.rs` | `tile_centre_on_output`, `tile_mut` (ported) |
| `src/layout/tile.rs` | `ViewSwing`, eligibility, cut, `view_pose`, dynamics, element wiring, offscreen view shift, culling, tests |
| `src/layout/tests.rs`, `src/layout/tests/view_swing.rs` | the ported layout tests |
| `src/window/mapped.rs` | `background_effect_visible` |
| `src/render_helpers/shaders/material/common.frag` | `g_seed_glide` |
| `src/render_helpers/shaders/material/prelude.frag` | view uniforms, globals, rotation and projection helpers, `slabFrame`, the traced solid, `viewBegin`, `viewTrace`, `viewOutline`, `Surface.view` |
| `src/render_helpers/shaders/material/main.frag` | the uniform branch, the shading point, wall overrides, outline coverage, seeds |
| `src/render_helpers/shaders/material/noise.frag`, `reflection.frag`, `edge_highlight.frag` | glide, projected reflection sample, half-vector |
| `src/render_helpers/shaders/mod.rs` | uniform names; assembly and pin tests |
| `src/tests/view_tilt.rs`, `src/tests/mod.rs` | the frozen-render harness and its cases |
| `src/tests/client.rs` | subsurfaces, popups and pixel buffers, ported from the spike's fixture commit |
| `src/tests/glass_edge.rs`, `src/tests/ring_look.rs` | `renderer_name` and `ACCEPTED` shared with the harness |
| `docs/materials/scripts/view-tilt-sheet.py` | the owner's still sheet |
| `docs/materials/scripts/view-tilt-cost.sh` | the GPU cost A/B |
| `docs/materials/scripts/ring-motion-clips.sh` | the owner's clip: four tilt sequences |
| `docs/materials/material-config.md`, `render-pipeline.md`, `performance.md` | the keys, the view path, its cost |
| `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md` | identity, cost, stills and clip evidence |

---
### Task 1: The three response keys in niri-config

**Files:**
- Modify: `niri-config/src/material/mod.rs` (`Response`, `response_fields`, `ResolvedResponse`, its `Default`, `with_overrides`)
- Modify: `niri-config/src/material/pipeline.rs` (`VIEW_RESPONSES`, the `slab` stage, `response_fields_match_the_struct`)
- Modify: `resources/materials/pipeline.json` (regenerated)
- Modify: `docs/materials/material-config.md` (the response table and a `### Focus tilt` section)
- Test: `niri-config/src/lib.rs`

**Interfaces:**
- Consumes: `Milli<MIN_MILLI, MAX_MILLI>` and `FloatOrInt<MIN, MAX>` (existing).
- Produces: `ResolvedResponse { view_tilt: f64 /* degrees */, view_tilt_settle: f64 /* ms */, view_distance: f64 /* κ */, .. }`, defaults 0, 900, 6.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module of `niri-config/src/lib.rs`, next to `ring_beam_decay_defaults_off_bounds_and_inherits`:

```rust
#[test]
fn view_tilt_keys_default_off_bound_and_inherit() {
    let parsed = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)]).unwrap();
    let d = parsed.materials[0].resolve().response(None);
    assert_eq!(
        (d.view_tilt, d.view_tilt_settle, d.view_distance),
        (0., 900., 6.),
        "no swing unless asked"
    );

    let parsed = parse_files(&[(
        "config.kdl",
        r#"material "tg" { glass {}; response "default" { view-tilt 10; view-tilt-settle 1200; view-distance 2.5; }; response "still" {}; }"#,
    )])
    .unwrap();
    let m = parsed.materials[0].resolve();
    let d = m.response(None);
    assert_eq!((d.view_tilt, d.view_tilt_settle, d.view_distance), (10., 1200., 2.5));
    let still = m.response(Some("still"));
    assert_eq!(
        (still.view_tilt, still.view_tilt_settle, still.view_distance),
        (10., 1200., 2.5),
        "inherits from default"
    );

    for (line, message) in [
        ("view-tilt 21;", "value must be between 0 and 20"),
        ("view-tilt -1;", "value must be between 0 and 20"),
        ("view-tilt-settle 50;", "value must be between 100 and 5000"),
        ("view-distance 1.4;", "value must be between 1.5 and 50"),
        ("view-distance 51;", "value must be between 1.5 and 50"),
    ] {
        let config = format!(r#"material "tg" {{ glass {{}}; response "default" {{ {line} }}; }}"#);
        let err = parse_files_err(&[("config.kdl", &config)]);
        assert!(err.contains(message), "{line}: {err}");
    }
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `just test-one -p niri-config view_tilt_keys_default_off_bound_and_inherit`
Expected: FAIL to compile: `no field view_tilt on type ResolvedResponse`.

- [ ] **Step 3: Add the fields**

In `niri-config/src/material/mod.rs`, in `struct Response`, directly after `ring_beam_decay`:

```rust
    #[knuffel(child, unwrap(argument))]
    pub view_tilt: Option<FloatOrInt<0, 20>>,
    #[knuffel(child, unwrap(argument))]
    pub view_tilt_settle: Option<FloatOrInt<100, 5000>>,
    #[knuffel(child, unwrap(argument))]
    pub view_distance: Option<Milli<1500, 50000>>,
```

In `response_fields()`, after `"ring-beam-decay",`:

```rust
        "view-tilt",
        "view-tilt-settle",
        "view-distance",
```

In `struct ResolvedResponse`, after `ring_beam_decay`:

```rust
    /// Peak angle of the rigid tilt on focus gain, degrees; zero is no swing
    /// (design 2026-10-09 §3.1).
    pub view_tilt: f64,
    /// How long the swing runs, ms.
    pub view_tilt_settle: f64,
    /// κ: the camera's distance in multiples of the pane's extent along the
    /// swing (design 2026-10-09 §3.2).
    pub view_distance: f64,
```

In `Default for ResolvedResponse`, after `ring_beam_decay: 0.,`:

```rust
            view_tilt: 0.,
            view_tilt_settle: 900.,
            view_distance: 6.,
```

In `with_overrides`, after the `ring_beam_decay` field:

```rust
            view_tilt: response.view_tilt.map_or(base.view_tilt, |x| x.0),
            view_tilt_settle: response
                .view_tilt_settle
                .map_or(base.view_tilt_settle, |x| x.0),
            view_distance: response.view_distance.map_or(base.view_distance, |x| x.0),
```

- [ ] **Step 4: Register them in the pipeline schema**

In `niri-config/src/material/pipeline.rs`, after `IMPULSE_RESPONSES`:

```rust
/// The rigid tilt's keys: they move the slab's frame, so the slab stage reads
/// them (design 2026-10-09 §3.3).
const VIEW_RESPONSES: &[&str] = &["view-tilt", "view-tilt-settle", "view-distance"];
```

In the `slab` stage, replace its `responses` argument `&[],` (the line after the `reads` list that ends in `"thickness",\n        ],`) with `VIEW_RESPONSES,`.

In `response_fields_match_the_struct`, add to the destructuring pattern after `ring_beam_decay: _,`:

```rust
            view_tilt: _,
            view_tilt_settle: _,
            view_distance: _,
```

and to `live`, after the `ring-beam-decay` entry:

```rust
            ("view-tilt", "view-tilt 10"),
            ("view-tilt-settle", "view-tilt-settle 900"),
            ("view-distance", "view-distance 6"),
```

- [ ] **Step 5: Regenerate the schema file and run the tests**

Run: `MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file`, then `just test-one -p niri-config view_tilt_keys_default_off_bound_and_inherit`, then `just test-one -p niri-config pipeline`.
Expected: all PASS; `git diff resources/materials/pipeline.json` shows the three keys in the `slab` stage's `responses`.

- [ ] **Step 6: Document the keys**

In `docs/materials/material-config.md`, add three rows to the response table after `ring-beam-decay`:

```markdown
| `view-tilt` | 0–20 degrees | 0 (off) |
| `view-tilt-settle` | 100–5000 ms | 900 |
| `view-distance` | 1.5–50 | 6 |
```

and, before `## Signal motion and animation`, this section:

```markdown
### Focus tilt

`view-tilt` turns the pane briefly in depth when it gains focus: window
content and glass swing as one body about the pane's centre, across the
direction focus came from, and settle back to exactly face-on after
`view-tilt-settle` ms. The side the focus came from presses in first; one
small overshoot follows. While it turns, perspective engages with the angle
and the slab's rounded edge and side wall come into view. `view-distance`
is the camera's distance in multiples of the pane's extent along the swing:
smaller is stronger perspective, larger shows more wall. On a horizontal
swing the advancing wall shows on every moving frame when
`sin(view-tilt) > 1 / (2 · view-distance)`.

A swing starts only when the window's border and focus ring are off, its
`background-effect` is neutral (`blur false; noise 0; saturation 1`, as
Prism writes), it is not fullscreen or opening, and `signal { motion "full"; }`
with animations on. A pane refocused mid-swing finishes the swing it has.
The policy cuts a running swing to rest on the next frame. At rest nothing
changes. Design: `docs/specs/2026-10-09-transient-rigid-tilt-design.md`.
```

- [ ] **Step 7: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-1971fd "view-tilt, view-tilt-settle, view-distance in the response block; slab stage reads them"
tasks check
git add niri-config/src/material/mod.rs niri-config/src/material/pipeline.rs niri-config/src/lib.rs resources/materials/pipeline.json docs/materials/material-config.md tasks/
git commit -m "feat(config): rigid tilt response keys"
```

### Task 2: Swing curve, weight, pose and direction

**Files:**
- Create: `src/render_helpers/material/view/mod.rs`
- Modify: `src/render_helpers/material/mod.rs` (`pub mod view;`)
- Modify: `src/render_helpers/material/ring.rs` (`hash01` becomes `pub(super)`)

**Interfaces:**
- Consumes: `ring::hash01(u64) -> f64`.
- Produces: `view::SWING_PEAK: f64`, `view::COINCIDENT_PX: f64`, `view::swing_shape(u: f64) -> f64`, `view::ViewPose { theta: f64, dir: DVec2, weight: f64, kappa: f64 }` with `ViewPose::at(peak_deg: f64, settle: Duration, elapsed: Duration, dir: DVec2, kappa: f64) -> Option<ViewPose>` and `ViewPose::identity(dir: DVec2, kappa: f64) -> ViewPose`, `view::SwingFrom::{Origin([f64; 2]), Fallback}`, `view::direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom`, `view::seeded_direction(seed: u32) -> [f64; 2]`.

- [ ] **Step 1: Write the module with its tests, curve body stubbed**

Create `src/render_helpers/material/view/mod.rs`:

```rust
//! The transient rigid tilt in f64 (docs/specs/2026-10-09-transient-rigid-tilt-design.md):
//! the swing curve and its weight (§3.1), the pose and its direction (§3.2), the camera and
//! the slab-local ray, the projection back to the screen and its near clip (§4.1), and the
//! element footprint (§6). `prelude.frag` mirrors the camera and the projection under the same
//! names. Coordinates are element-local logical px, y down, +z toward the viewer.
//!
//! Every map is the identity plus a correction that is exactly zero at θ = 0 and w = 0, so the
//! identity reproduces its inputs bit for bit (§4.1).

use std::f64::consts::PI;
use std::time::Duration;

use glam::DVec2;

use super::ring::hash01;

/// The peak of g(u) = (1 − u)² sin 2πu over [0, 1], at u = 0.190381.
pub const SWING_PEAK: f64 = 0.610_028_508_853_155_6;
/// Centres closer than this give no direction to swing along.
pub const COINCIDENT_PX: f64 = 1.;

/// The swing's shape at `u` = elapsed / settle, g(u) / G: exactly 0 at u = 0, a peak of 1 at
/// u = 0.1904, one zero crossing at u = 0.5, an overshoot to −0.1638 at u = 0.636, and exactly
/// 0 from u = 1 on (§3.1).
pub fn swing_shape(u: f64) -> f64 {
    let _ = u;
    unimplemented!("Task 2 Step 3")
}

/// The swing's pose on one frame (§3.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewPose {
    /// Signed angle, radians: the side along +`dir` lifts while it is positive.
    pub theta: f64,
    /// Unit direction in the glass plane, from the focus origin toward this pane.
    pub dir: DVec2,
    /// The perspective weight |θ| / θp, in [0, 1].
    pub weight: f64,
    /// κ: the camera's distance in multiples of the slab's extent along `dir`.
    pub kappa: f64,
}

impl ViewPose {
    /// The pose `elapsed` into a swing of `settle` that peaks at `peak_deg`. `None` where the
    /// curve is exactly zero: the first and the settled frames are rest.
    pub fn at(
        peak_deg: f64,
        settle: Duration,
        elapsed: Duration,
        dir: DVec2,
        kappa: f64,
    ) -> Option<Self> {
        let shape = swing_shape(elapsed.as_secs_f64() / settle.as_secs_f64());
        (shape != 0.).then(|| Self {
            theta: peak_deg.to_radians() * shape,
            dir,
            weight: shape.abs(),
            kappa,
        })
    }

    /// θ = 0 and w = 0 with the view path on: the identity hook of §8.2.
    pub fn identity(dir: DVec2, kappa: f64) -> Self {
        Self {
            theta: 0.,
            dir,
            weight: 0.,
            kappa,
        }
    }
}

/// Where a swing comes from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwingFrom {
    /// From the previous focus: the unit vector from its centre to the new focus's centre, in
    /// output-local logical coordinates.
    Origin([f64; 2]),
    /// No usable origin: the tile seeds a direction of its own.
    Fallback,
}

/// The swing's direction from the previous focus's centre to the new one.
pub fn direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    if len < COINCIDENT_PX {
        SwingFrom::Fallback
    } else {
        SwingFrom::Origin([d[0] / len, d[1] / len])
    }
}

/// A unit direction from `seed`, for a swing with no usable origin.
pub fn seeded_direction(seed: u32) -> [f64; 2] {
    let angle = hash01(u64::from(seed) ^ 0x5EED_D1EC) * 2. * PI;
    [angle.cos(), angle.sin()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_starts_at_rest_peaks_overshoots_once_and_ends_still() {
        assert_eq!(swing_shape(0.), 0.);
        let steps = 100_000;
        let (mut peak, mut peak_u) = (f64::NEG_INFINITY, 0.);
        let (mut low, mut low_u) = (f64::INFINITY, 0.);
        let mut crossings = Vec::new();
        let mut prev = swing_shape(1. / steps as f64);
        for i in 1..steps {
            let u = i as f64 / steps as f64;
            let g = swing_shape(u);
            if g > peak {
                (peak, peak_u) = (g, u);
            }
            if g < low {
                (low, low_u) = (g, u);
            }
            if g.signum() != prev.signum() {
                crossings.push(u);
            }
            prev = g;
        }
        assert!((peak - 1.).abs() < 1e-8, "peak {peak}");
        assert!((peak_u - 0.1904).abs() < 1e-3, "peak at {peak_u}");
        assert_eq!(crossings.len(), 1, "one zero crossing: {crossings:?}");
        assert!((crossings[0] - 0.5).abs() < 1e-4, "crossing at {}", crossings[0]);
        assert!((low + 0.1638).abs() < 1e-3, "overshoot {low}");
        assert!((low_u - 0.636).abs() < 1e-3, "overshoot at {low_u}");
        assert_eq!(swing_shape(1.), 0.);
        assert_eq!(swing_shape(1.5), 0.);
        assert!(swing_shape(1. - 1e-4).abs() < 1e-8, "the last moving frame is still");
        // A struck body: the initial angular velocity is 2π / G.
        let slope = swing_shape(1e-9) / 1e-9;
        assert!((slope - 10.30).abs() < 0.01, "initial slope {slope}");
    }

    #[test]
    fn the_peak_constant_is_the_maximum() {
        let g = |u: f64| (1. - u).powi(2) * (2. * PI * u).sin();
        // tan 2πu = π(1 − u) at the peak; Newton from 0.19.
        let mut u = 0.19;
        for _ in 0..50 {
            let f = (2. * PI * u).tan() - PI * (1. - u);
            let df = 2. * PI / (2. * PI * u).cos().powi(2) + PI;
            u -= f / df;
        }
        assert!((g(u) - SWING_PEAK).abs() < 1e-15, "{} vs {SWING_PEAK}", g(u));
    }

    #[test]
    fn the_weight_is_the_shape_magnitude_and_rest_is_none() {
        let settle = Duration::from_millis(900);
        let dir = DVec2::X;
        assert_eq!(ViewPose::at(10., settle, Duration::ZERO, dir, 6.), None);
        assert_eq!(ViewPose::at(10., settle, settle, dir, 6.), None);
        assert_eq!(ViewPose::at(10., settle, Duration::from_secs(5), dir, 6.), None);
        for ms in (1..900).step_by(7) {
            let pose = ViewPose::at(10., settle, Duration::from_millis(ms), dir, 6.).unwrap();
            assert!((0. ..=1.).contains(&pose.weight), "{ms} ms: {}", pose.weight);
            let ratio = pose.theta.abs() / 10f64.to_radians();
            assert!((ratio - pose.weight).abs() < 1e-12, "{ms} ms");
        }
        let peak = ViewPose::at(10., settle, Duration::from_secs_f64(0.9 * 0.190381), dir, 6.)
            .unwrap();
        assert!((peak.weight - 1.).abs() < 1e-6 && peak.theta > 0.);
        let back = ViewPose::at(10., settle, Duration::from_millis(572), dir, 6.).unwrap();
        assert!(back.theta < 0., "the overshoot turns the other way");
        let id = ViewPose::identity(dir, 6.);
        assert_eq!((id.theta, id.weight), (0., 0.));
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

In `src/render_helpers/material/mod.rs`, after `pub mod tint;`, add `pub mod view;`. In `src/render_helpers/material/ring.rs`, change `fn hash01(mut x: u64) -> f64 {` to `pub(super) fn hash01(mut x: u64) -> f64 {`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri view::tests`
Expected: FAIL: `the_curve_starts_at_rest...` and `the_weight_is_the_shape...` panic with `not implemented: Task 2 Step 3`; `the_peak_constant_is_the_maximum` and the two direction tests PASS.

- [ ] **Step 3: Implement the curve**

Replace the body of `swing_shape`:

```rust
pub fn swing_shape(u: f64) -> f64 {
    if u >= 1. {
        return 0.;
    }
    let u = u.max(0.);
    (1. - u).powi(2) * (2. * PI * u).sin() / SWING_PEAK
}
```

- [ ] **Step 4: Run the tests**

Run: `just test-one -p niri view::tests`
Expected: 5 PASS.

- [ ] **Step 5: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-44d5be "view module: swing curve, weight, pose, directions"
tasks check
git add src/render_helpers/material/view/mod.rs src/render_helpers/material/mod.rs src/render_helpers/material/ring.rs tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): rigid tilt swing curve and pose"
```

### Task 3: Camera, projection, near clip and footprint

**Files:**
- Modify: `src/render_helpers/material/view/mod.rs`

**Interfaces:**
- Consumes: `ViewPose` (Task 2).
- Produces: `view::VIEW_NEAR_CLIP: f64`; `view::Camera { cos_m1: f64, sin: f64, dir: DVec2, lambda: f64, pivot: DVec2 }` with `Camera::new(pose: &ViewPose, slab_min: DVec2, slab_size: DVec2) -> Camera`, `unrotate_delta(DVec3) -> DVec3`, `rotate_delta(DVec3) -> DVec3`, `ray(s: DVec2) -> (DVec3, DVec3)` (origin, unnormalised direction), `to_world(q: DVec3) -> DVec3`, `zeta(e: DVec3) -> f64` (of a world point), `screen_shift(q: DVec3) -> DVec2`, `to_screen(q: DVec3) -> DVec2`, `project(e: DVec3) -> DVec2` (of a world point); `view::Bounds = (DVec2, DVec2)` (min, max); `view::footprint(camera: &Camera, slab: Bounds, tex: Bounds, thickness: f64) -> Bounds`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module of `view/mod.rs` (and `use glam::DVec3;` at the top of the module):

```rust
    /// A deterministic stream of uniforms in [0, 1).
    pub(super) fn uniforms(seed: u64) -> impl FnMut() -> f64 {
        let mut i = seed << 20;
        move || {
            i += 1;
            hash01(i)
        }
    }

    fn slab(min: (f64, f64), size: (f64, f64)) -> (DVec2, DVec2) {
        (DVec2::new(min.0, min.1), DVec2::new(size.0, size.1))
    }

    fn peak(peak_deg: f64, dir: DVec2, kappa: f64) -> ViewPose {
        ViewPose {
            theta: peak_deg.to_radians(),
            dir,
            weight: 1.,
            kappa,
        }
    }

    const DIRS: [DVec2; 4] = [
        DVec2::new(1., 0.),
        DVec2::new(0., 1.),
        DVec2::new(0.6, 0.8),
        DVec2::new(-0.8, 0.6),
    ];

    #[test]
    fn the_identity_reproduces_its_inputs() {
        let (min, size) = slab((10., 20.), (640., 360.));
        let mut rnd = uniforms(1);
        for dir in DIRS {
            let camera = Camera::new(&ViewPose::identity(dir, 6.), min, size);
            for _ in 0..2000 {
                let s = DVec2::new(rnd() * 900. - 100., rnd() * 600. - 100.);
                let (o, r) = camera.ray(s);
                assert_eq!(o, s.extend(0.));
                assert_eq!(r, DVec3::new(0., 0., -1.));
                let q = DVec3::new(s.x, s.y, -rnd() * 200.);
                assert_eq!(camera.to_world(q), q);
                assert_eq!(camera.screen_shift(q), DVec2::ZERO);
                assert_eq!(camera.to_screen(q), q.truncate());
                assert_eq!(camera.zeta(q), 0.);
            }
        }
    }

    #[test]
    fn the_positive_lobe_lifts_the_side_along_dir() {
        let (min, size) = slab((0., 0.), (400., 300.));
        let camera = Camera::new(&peak(10., DVec2::X, 6.), min, size);
        let right = camera.to_world(DVec3::new(400., 150., 0.));
        let left = camera.to_world(DVec3::new(0., 150., 0.));
        assert!(right.z > 0. && left.z < 0., "{right} {left}");
        // The projection is the pinhole's: a raised point moves away from the pivot.
        let shift = camera.to_screen(DVec3::new(400., 150., 0.)) - DVec2::new(400., 150.);
        assert!(shift.x < 0., "foreshortened toward the pivot: {shift}");
    }

    /// Points on the slab box's faces and on the texture rect, the drawn ones only.
    fn drawn_points(camera: &Camera, s: (DVec2, DVec2), t: (DVec2, DVec2), thickness: f64) -> Vec<DVec2> {
        let mut out = Vec::new();
        let n = 24;
        let mut push = |q: DVec3| {
            let e = camera.to_world(q);
            if camera.zeta(e) <= VIEW_NEAR_CLIP {
                out.push(camera.project(e));
            }
        };
        for i in 0..=n {
            for j in 0..=n {
                let (a, b) = (i as f64 / n as f64, j as f64 / n as f64);
                let slab = s.0 + (s.1 - s.0) * DVec2::new(a, b);
                for z in [0., -thickness, -thickness * a] {
                    push(slab.extend(z));
                }
                for z in [0., -thickness * b] {
                    push(DVec3::new(s.0.x, slab.y, z));
                    push(DVec3::new(s.1.x, slab.y, z));
                    push(DVec3::new(slab.x, s.0.y, z));
                    push(DVec3::new(slab.x, s.1.y, z));
                }
                push((t.0 + (t.1 - t.0) * DVec2::new(a, b)).extend(0.));
            }
        }
        out
    }

    fn assert_contains(fp: Bounds, points: &[DVec2], what: &str) {
        assert!(fp.0.is_finite() && fp.1.is_finite(), "{what}: {fp:?}");
        for p in points {
            assert!(
                p.x >= fp.0.x - 1e-9 && p.y >= fp.0.y - 1e-9 && p.x <= fp.1.x + 1e-9 && p.y <= fp.1.y + 1e-9,
                "{what}: {p} outside {fp:?}"
            );
        }
    }

    #[test]
    fn the_footprint_contains_every_drawn_point() {
        let thickness = 31.2;
        for (slab_min, slab_size, tex) in [
            // A texture 150 px past the slab on every side.
            ((0., 0.), (640., 360.), ((-150., -150.), (790., 510.))),
            // The (250, 250) texture corner of a 100 px slab (§6).
            ((-50., -50.), (100., 100.), ((-50., -50.), (250., 250.))),
        ] {
            let s = (DVec2::from(slab_min), DVec2::from(slab_min) + DVec2::from(slab_size));
            let t = (DVec2::from(tex.0), DVec2::from(tex.1));
            for peak_deg in [2., 10., 20.] {
                for kappa in [1.5, 6., 50.] {
                    for dir in DIRS {
                        for sign in [1., -1.] {
                            let pose = ViewPose { theta: sign * peak_deg.to_radians(), ..peak(peak_deg, dir, kappa) };
                            let camera = Camera::new(&pose, s.0, slab_size.into());
                            let fp = footprint(&camera, s, t, thickness);
                            let what = format!("{peak_deg}° κ {kappa} {dir} {sign}");
                            assert_contains(fp, &drawn_points(&camera, s, t, thickness), &what);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_decal_past_the_near_clip_has_a_finite_footprint() {
        // A texture reaching x = 500 from the pivot of a 100 px slab at 20°, κ = 1.5 (§8.1).
        let s = (DVec2::new(-50., -50.), DVec2::new(50., 50.));
        let t = (DVec2::new(-50., -50.), DVec2::new(500., 50.));
        let camera = Camera::new(&peak(20., DVec2::X, 1.5), s.0, DVec2::splat(100.));
        let far = camera.to_world(DVec3::new(500., 0., 0.));
        assert!(camera.zeta(far) > VIEW_NEAR_CLIP, "the case must cross the clip");
        let fp = footprint(&camera, s, t, 31.2);
        assert_contains(fp, &drawn_points(&camera, s, t, 31.2), "past the clip");
        // No point is magnified more than 4× about the pivot.
        assert!(fp.1.x < 4. * 500. && fp.1.x > 50., "{fp:?}");
    }

    #[test]
    fn the_footprint_is_finite_on_every_frame_of_a_run() {
        let s = (DVec2::new(-50., -50.), DVec2::new(50., 50.));
        let t = (DVec2::new(-400., -400.), DVec2::new(900., 900.));
        let settle = Duration::from_millis(900);
        for ms in 0..=900 {
            let Some(pose) = ViewPose::at(20., settle, Duration::from_millis(ms), DVec2::new(0.6, 0.8), 1.5) else {
                continue;
            };
            let camera = Camera::new(&pose, s.0, DVec2::splat(100.));
            let fp = footprint(&camera, s, t, 200.);
            assert_contains(fp, &drawn_points(&camera, s, t, 200.), &format!("{ms} ms"));
        }
    }

    #[test]
    fn the_slab_never_reaches_the_near_clip() {
        let mut rnd = uniforms(2);
        for _ in 0..20_000 {
            let peak_deg = rnd() * 20.;
            let kappa = 1.5 + rnd() * 48.5;
            let angle = rnd() * 2. * PI;
            let dir = DVec2::new(angle.cos(), angle.sin());
            let size = DVec2::new(2. + rnd() * 4000., 2. + rnd() * 3000.);
            let thickness = rnd() * 200.;
            let camera = Camera::new(&peak(peak_deg, dir, kappa), DVec2::ZERO, size);
            let bound = peak_deg.to_radians().sin() / (2. * kappa);
            for (x, y) in [(0., 0.), (1., 0.), (1., 1.), (0., 1.)] {
                for z in [0., -thickness] {
                    let q = DVec3::new(x * size.x, y * size.y, z);
                    let zeta = camera.zeta(camera.to_world(q));
                    assert!(zeta <= bound + 1e-12, "ζ {zeta} > {bound}");
                    assert!(bound <= 0.114, "{bound}");
                }
            }
        }
    }

    #[test]
    fn rays_through_the_slab_point_down() {
        let mut rnd = uniforms(3);
        for _ in 0..20_000 {
            let peak_deg = rnd() * 20.;
            let kappa = 1.5 + rnd() * 48.5;
            let angle = rnd() * 2. * PI;
            let size = DVec2::new(2. + rnd() * 4000., 2. + rnd() * 3000.);
            let camera = Camera::new(&peak(peak_deg, DVec2::new(angle.cos(), angle.sin()), kappa), DVec2::ZERO, size);
            let fp = footprint(&camera, (DVec2::ZERO, size), (DVec2::ZERO, size), rnd() * 200.);
            let s = fp.0 + (fp.1 - fp.0) * DVec2::new(rnd(), rnd());
            assert!(camera.ray(s).1.z < 0., "r.z ≥ 0 at {s} in the slab's footprint");
        }
    }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri view::tests`
Expected: FAIL to compile: `cannot find type Camera`, `cannot find function footprint`.

- [ ] **Step 3: Implement the camera and the footprint**

In `view/mod.rs`, change the import to `use glam::{DVec2, DVec3};` and add after `seeded_direction`:

```rust
/// No point drawn during a swing is nearer the camera than this fraction of its distance: at
/// most 4× magnification (§4.1).
pub const VIEW_NEAR_CLIP: f64 = 0.75;

/// One frame's rotation about the pivot and its camera (§3.2, §4.1). R turns by θ in the
/// (dir, z) plane, so the side along +dir lifts while θ > 0. The camera sits on the pivot's
/// normal at D = κE / w.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// cos θ − 1, written −2 sin²(θ/2): exactly 0 at θ = 0, and accurate near it.
    pub cos_m1: f64,
    pub sin: f64,
    pub dir: DVec2,
    /// λ = w / (κE): the camera ray's slope per px from the pivot, and ζ per px of height.
    pub lambda: f64,
    /// The slab's centre on its front face, as `slabSurface` computes it.
    pub pivot: DVec2,
}

/// An axis-aligned box as (min, max).
pub type Bounds = (DVec2, DVec2);

impl Camera {
    pub fn new(pose: &ViewPose, slab_min: DVec2, slab_size: DVec2) -> Self {
        let extent = pose.dir.x.abs() * slab_size.x + pose.dir.y.abs() * slab_size.y;
        let half = 0.5 * pose.theta;
        Self {
            cos_m1: -2. * half.sin() * half.sin(),
            sin: pose.theta.sin(),
            dir: pose.dir,
            lambda: pose.weight / (pose.kappa * extent),
            pivot: slab_min + slab_size * 0.5,
        }
    }

    /// (R⁻¹ − I) v: the turn by −θ, less the identity. Mirrors `viewUnrotateDelta`.
    pub fn unrotate_delta(&self, v: DVec3) -> DVec3 {
        let a = v.truncate().dot(self.dir);
        let da = a * self.cos_m1 + v.z * self.sin;
        let dz = -a * self.sin + v.z * self.cos_m1;
        (self.dir * da).extend(dz)
    }

    /// (R − I) v. Mirrors `viewRotateDelta`.
    pub fn rotate_delta(&self, v: DVec3) -> DVec3 {
        let a = v.truncate().dot(self.dir);
        let da = a * self.cos_m1 - v.z * self.sin;
        let dz = a * self.sin + v.z * self.cos_m1;
        (self.dir * da).extend(dz)
    }

    /// The camera ray through screen point `s` on z = 0, slab-local: origin
    /// `o = s + (R⁻¹ − I)(s − P)` and direction `r = R⁻¹ ((s − P) λ, −1)`, unnormalised.
    pub fn ray(&self, s: DVec2) -> (DVec3, DVec3) {
        let rel = s - self.pivot;
        let o = s.extend(0.) + self.unrotate_delta(rel.extend(0.));
        let dir = (rel * self.lambda).extend(-1.);
        (o, dir + self.unrotate_delta(dir))
    }

    /// A slab-local point in the world: `q + (R − I)(q − P)`.
    pub fn to_world(&self, q: DVec3) -> DVec3 {
        q + self.rotate_delta(q - self.pivot.extend(0.))
    }

    /// ζ of a world point: its height toward the camera as a fraction of the camera distance.
    pub fn zeta(&self, e: DVec3) -> f64 {
        e.z * self.lambda
    }

    /// Where a world point projects on the screen along its camera ray:
    /// `e.xy + (e.xy − P)(f − 1)`, `f − 1 = ζ / (1 − ζ)`.
    pub fn project(&self, e: DVec3) -> DVec2 {
        let zeta = self.zeta(e);
        e.truncate() + (e.truncate() - self.pivot) * (zeta / (1. - zeta))
    }

    /// `toScreen(q) − q.xy` for a slab-local point, exactly zero at the identity. Mirrors
    /// `viewScreenShift`.
    pub fn screen_shift(&self, q: DVec3) -> DVec2 {
        let dq = self.rotate_delta(q - self.pivot.extend(0.));
        let exy = q.truncate() + dq.truncate();
        let zeta = (q.z + dq.z) * self.lambda;
        dq.truncate() + (exy - self.pivot) * (zeta / (1. - zeta))
    }

    /// `toScreen` (§4.1 step 5).
    pub fn to_screen(&self, q: DVec3) -> DVec2 {
        q.truncate() + self.screen_shift(q)
    }
}

/// The element footprint during a swing (§6): the bounding box of the projections of the slab
/// box's eight corners and the texture rect's four, each set first clipped at ζ = 3/4. The kept
/// vertices are the corners at or below the plane plus each edge's crossing of it.
pub fn footprint(camera: &Camera, slab: Bounds, tex: Bounds, thickness: f64) -> Bounds {
    let corners = |(lo, hi): Bounds, z: f64| {
        [
            DVec3::new(lo.x, lo.y, z),
            DVec3::new(hi.x, lo.y, z),
            DVec3::new(hi.x, hi.y, z),
            DVec3::new(lo.x, hi.y, z),
        ]
        .map(|q| camera.to_world(q))
    };
    let front = corners(slab, 0.);
    let back = corners(slab, -thickness);
    let decal = corners(tex, 0.);
    let mut edges = Vec::with_capacity(16);
    for i in 0..4 {
        let j = (i + 1) % 4;
        edges.extend([(front[i], front[j]), (back[i], back[j]), (front[i], back[i])]);
        edges.push((decal[i], decal[j]));
    }
    let mut lo = DVec2::INFINITY;
    let mut hi = DVec2::NEG_INFINITY;
    let mut keep = |e: DVec3| {
        let s = camera.project(e);
        lo = lo.min(s);
        hi = hi.max(s);
    };
    for e in front.into_iter().chain(back).chain(decal) {
        if camera.zeta(e) <= VIEW_NEAR_CLIP {
            keep(e);
        }
    }
    for (a, b) in edges {
        let (za, zb) = (camera.zeta(a), camera.zeta(b));
        if (za - VIEW_NEAR_CLIP) * (zb - VIEW_NEAR_CLIP) < 0. {
            keep(a + (b - a) * ((VIEW_NEAR_CLIP - za) / (zb - za)));
        }
    }
    (lo, hi)
}
```

- [ ] **Step 4: Run the tests**

Run: `just test-one -p niri view::tests`
Expected: 12 PASS.

- [ ] **Step 5: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-d7ebec "view camera, projection, near clip, footprint"
tasks check
git add src/render_helpers/material/view/mod.rs tasks/
git commit -m "feat(material): rigid tilt camera and footprint"
```

### Task 4: The traced slab and its bracketing; the wall-reveal rule

**Files:**
- Create: `src/render_helpers/material/view/solid.rs`
- Modify: `src/render_helpers/material/view/mod.rs` (`pub mod solid;`, `wall_faces_camera`)

**Interfaces:**
- Consumes: `Camera`, `ViewPose`, `VIEW_NEAR_CLIP` (Task 3); `bevel::{corner_radius, Radii, BEVEL_PROFILE_EPS}`, `bevel::Slab` (tests), `ring::fit_radii` (tests). The tests' reference, `certified_entry`, proves a hit or a miss from Ψ's convexity, or reports the ray unresolved.
- Produces: `solid::{VIEW_GOLDEN, VIEW_GOLDEN_STEPS, VIEW_BISECT_STEPS}`; `solid::Solid { center: DVec2, half: DVec2, outer_r: Radii, chamfer: f64, thickness: f64, profile: f64 }` with `rise()`, `sigma(xy, sel)`, `sigma_grad(xy, sel)`, `psi(q: DVec3, sel: DVec2)`, `psi_dz(q)`; `solid::Segment { r, h0, h0_shift, rho, tau0 }` with `Segment::new(&Camera, s: DVec2) -> Option<Segment>` and `point(t, thickness) -> DVec3`; `solid::Trace { hit: bool, wall: bool, t: f64, psi_min: f64, min_t: f64, min_plane: DVec3, min_sel: DVec2 }`; `solid::trace(&Solid, &Segment) -> Trace`; `view::wall_faces_camera(pose: &ViewPose, extent: f64, n0: DVec2, h: f64) -> bool`.

- [ ] **Step 1: Write the module with its tests, the trace stubbed**

Create `src/render_helpers/material/view/solid.rs`:

```rust
//! The traced slab under a tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md §4.1
//! step 4, §4.5): the resting slab as a convex solid Ψ ≤ 0, the ray's segment split where it
//! crosses the silhouette's centre lines, golden-section and bisection bracketing per piece, and
//! the outline distance that gives coverage. `prelude.frag` mirrors it line for line, and a test
//! here pins the constants and lines the two share.
//!
//! The segment runs from the window plane z = 0 to the back plane z = −T and is parameterised by
//! depth, q(t) = h₀ + t·T·ρ with ρ = r / (−r.z), so z = −tT.

use glam::{DVec2, DVec3};

use super::Camera;
use crate::render_helpers::material::bevel::{corner_radius, Radii, BEVEL_PROFILE_EPS};

/// The golden-section ratio, (√5 − 1) / 2.
pub const VIEW_GOLDEN: f64 = 0.618_033_988_749_895;
/// Golden-section steps per piece: the bracket shrinks to 0.618^28 ≈ 1.4e-6 of the piece.
pub const VIEW_GOLDEN_STEPS: usize = 28;
/// Bisection steps to the entry: the longest piece in the configured ranges (thickness 200,
/// |ρ| ≤ 1.2, scale 4) ends within 1/64 physical px after 16; 24 leaves a margin.
pub const VIEW_BISECT_STEPS: usize = 24;

/// The resting slab as `slabSurface` frames it, without jelly (§4.1 "Jelly").
#[derive(Debug, Clone, Copy)]
pub struct Solid {
    pub center: DVec2,
    pub half: DVec2,
    /// Outer radii, fitted, in `CornerRadius` order.
    pub outer_r: Radii,
    /// The chamfer actually drawn (the tiny-slab guard applied).
    pub chamfer: f64,
    pub thickness: f64,
    /// `bevel-profile`, k.
    pub profile: f64,
}

impl Solid {
    /// min(chamfer, thickness): the bevel's height.
    pub fn rise(&self) -> f64 {
        self.chamfer.min(self.thickness)
    }

    /// σ, the silhouette's signed distance, with the radius of the quadrant `sel` (relative to
    /// the centre) lies in. Mirrors `viewSigma`.
    pub fn sigma(&self, xy: DVec2, sel: DVec2) -> f64 {
        let r = corner_radius(sel, self.outer_r);
        let q = (xy - self.center).abs() - self.half + DVec2::splat(r);
        q.x.max(q.y).min(0.) + q.max(DVec2::ZERO).length() - r
    }

    /// ∇σ in the same quadrant, as `sdRoundedBoxGrad`. Mirrors `viewSigmaGrad`.
    pub fn sigma_grad(&self, xy: DVec2, sel: DVec2) -> DVec2 {
        let p = xy - self.center;
        let r = corner_radius(sel, self.outer_r);
        let q = p.abs() - self.half + DVec2::splat(r);
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

    /// Ψ (§4.1): ≤ 0 inside. `X` inverts today's `bevelProfile`, so Ψ ≤ 0 is the set
    /// z ≤ height − T. Without a rise (bevel 0 or thickness 0) Ψ is σ. Mirrors `viewPsi`.
    pub fn psi(&self, q: DVec3, sel: DVec2) -> f64 {
        let sigma = self.sigma(q.truncate(), sel);
        let rise = self.rise();
        if rise <= 0. {
            return sigma;
        }
        let y = -q.z / rise;
        let x = if y >= 1. {
            1.
        } else {
            (1. - (1. - y).max(0.).powf(self.profile))
                .max(0.)
                .powf(1. / self.profile)
        };
        sigma + self.chamfer * (1. - x)
    }

    /// ∂Ψ/∂z = c X'(y) / rise, X'(y) = (1 − (1 − y)^k)^(1/k − 1) (1 − y)^(k − 1), both bases
    /// kept off zero as `bevelProfileSlope` keeps them. Mirrors `viewPsiDz`.
    pub fn psi_dz(&self, q: DVec3) -> f64 {
        let rise = self.rise();
        if rise <= 0. {
            return 0.;
        }
        let y = -q.z / rise;
        if y >= 1. {
            return 0.;
        }
        let k = self.profile;
        let b = (1. - y).clamp(BEVEL_PROFILE_EPS, 1.);
        let a = (1. - b.powf(k)).clamp(BEVEL_PROFILE_EPS, 1.);
        self.chamfer * a.powf(1. / k - 1.) * b.powf(k - 1.) / rise
    }
}

/// The ray through one screen point, set up as `viewBegin` sets it up (§4.1 steps 1–3).
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// The slab-local direction, unnormalised.
    pub r: DVec3,
    /// The window-plane hit, z exactly 0, and its shift from the screen point.
    pub h0: DVec3,
    pub h0_shift: DVec2,
    /// r / (−r.z): the step per px of depth.
    pub rho: DVec3,
    /// The ray parameter at h₀, from the origin along r.
    pub tau0: f64,
}

impl Segment {
    /// `None` when the ray does not point down (r.z ≥ 0): no plane hit, no glass.
    pub fn new(camera: &Camera, s: DVec2) -> Option<Self> {
        let rel = s - camera.pivot;
        let o_shift = camera.unrotate_delta(rel.extend(0.));
        let dir = (rel * camera.lambda).extend(-1.);
        let r = dir + camera.unrotate_delta(dir);
        if r.z >= 0. {
            return None;
        }
        let tau0 = -o_shift.z / r.z;
        let h0_shift = o_shift.truncate() + r.truncate() * tau0;
        Some(Self {
            r,
            h0: (s + h0_shift).extend(0.),
            h0_shift,
            rho: r / -r.z,
            tau0,
        })
    }

    /// The point `t` of the way down to the back plane.
    pub fn point(&self, t: f64, thickness: f64) -> DVec3 {
        self.h0 + self.rho * (t * thickness)
    }
}

/// One piece of the segment between centre-line crossings (§4.1 "Quadrant seams"): its ends,
/// each end's plane normal, and the quadrant its midpoint lies in, whose radius the whole piece
/// takes, so the piece is one convex function up to its ends.
#[derive(Debug, Clone, Copy)]
struct Piece {
    a: f64,
    b: f64,
    na: DVec3,
    nb: DVec3,
    sel: DVec2,
}

fn pieces(solid: &Solid, seg: &Segment) -> Vec<Piece> {
    let thickness = solid.thickness;
    let mut cuts = vec![(0., DVec3::Z)];
    if thickness > 0. {
        for (axis, normal) in [(0, DVec3::X), (1, DVec3::Y)] {
            let step = thickness * seg.rho[axis];
            if step != 0. {
                let at = (solid.center[axis] - seg.h0[axis]) / step;
                if at > 0. && at < 1. {
                    cuts.push((at, normal));
                }
            }
        }
    }
    cuts.sort_by(|x, y| x.0.total_cmp(&y.0));
    cuts.push((1., DVec3::Z));
    // Two crossings at one depth leave an empty piece between them; `viewTrace` skips it too.
    cuts.windows(2)
        .filter(|w| w[0].0 < w[1].0)
        .map(|w| {
            let ((a, na), (b, nb)) = (w[0], w[1]);
            let mid = seg.point(0.5 * (a + b), thickness);
            Piece {
                a,
                b,
                na,
                nb,
                sel: mid.truncate() - solid.center,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    A,
    B,
    Interior,
}

/// The minimum of `f` on [a, b], its ends included; a tie goes to an end, the start first
/// (§4.5 "Ties"). Mirrors `viewMinimise`.
fn minimise(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> (f64, f64, End) {
    let (mut lo, mut hi) = (a, b);
    let mut c = hi - VIEW_GOLDEN * (hi - lo);
    let mut d = lo + VIEW_GOLDEN * (hi - lo);
    let (mut fc, mut fd) = (f(c), f(d));
    for _ in 0..VIEW_GOLDEN_STEPS {
        if fc < fd {
            hi = d;
            d = c;
            fd = fc;
            c = hi - VIEW_GOLDEN * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + VIEW_GOLDEN * (hi - lo);
            fd = f(d);
        }
    }
    let (mut t, mut psi, mut end) = if fc < fd {
        (c, fc, End::Interior)
    } else {
        (d, fd, End::Interior)
    };
    let (fa, fb) = (f(a), f(b));
    if fb <= psi {
        (t, psi, end) = (b, fb, End::B);
    }
    if fa <= psi {
        (t, psi, end) = (a, fa, End::A);
    }
    (t, psi, end)
}

/// The entry on [lo, hi], where f(lo) > 0 ≥ f(hi). Mirrors the bisection in `viewTrace`.
fn bisect(f: &impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    let (mut lo, mut hi) = (lo, hi);
    for _ in 0..VIEW_BISECT_STEPS {
        let mid = 0.5 * (lo + hi);
        if f(mid) <= 0. {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// What one ray finds (§4.1 "Kinds of hit", §4.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trace {
    pub hit: bool,
    /// Shade the point as the wall: the entry (or, on a miss, the minimiser) lies below the rim
    /// and the ray is outside at the rim depth.
    pub wall: bool,
    /// The shading point's depth parameter: the entry on a hit, the minimiser on a miss.
    pub t: f64,
    /// Ψmin over every piece, its depth parameter, its end plane's normal (z for the face and
    /// back planes and for an interior minimiser), and the quadrant it was evaluated in.
    pub psi_min: f64,
    pub min_t: f64,
    pub min_plane: DVec3,
    pub min_sel: DVec2,
}

/// Brackets the ray's entry into the slab (§4.1 "Bracketing"). Mirrors `viewTrace`.
pub fn trace(solid: &Solid, seg: &Segment) -> Trace {
    let _ = (solid, seg);
    unimplemented!("Task 4 Step 3")
}

#[cfg(test)]
pub(super) mod tests {
    use std::f64::consts::PI;
    use std::time::Duration;

    use super::*;
    use crate::render_helpers::material::bevel::Slab;
    use crate::render_helpers::material::ring::fit_radii;
    use crate::render_helpers::material::view::tests::uniforms;
    use crate::render_helpers::material::view::{wall_faces_camera, ViewPose};

    /// A slab at the origin of `half` with face radii `face_r`, fitted as `slabSurface` fits
    /// them, and its camera for `pose`.
    pub(in crate::render_helpers::material::view) fn setup(
        half: DVec2,
        chamfer: f64,
        face_r: Radii,
        thickness: f64,
        profile: f64,
        pose: &ViewPose,
    ) -> (Solid, Camera) {
        let chamfer = chamfer.min((half.x.min(half.y) - 1.).max(0.));
        let face_half = half - DVec2::splat(chamfer);
        let fitted = fit_radii(face_r, [face_half.x, face_half.y]);
        let solid = Solid {
            center: DVec2::ZERO,
            half,
            outer_r: fitted.map(|r| r + chamfer),
            chamfer,
            thickness,
            profile,
        };
        (solid, Camera::new(pose, -half, 2. * half))
    }

    fn peak(peak_deg: f64, dir: DVec2, kappa: f64) -> ViewPose {
        ViewPose {
            theta: peak_deg.to_radians(),
            dir,
            weight: 1.,
            kappa,
        }
    }

    /// The silhouette point along `angle` from the centre, at rest.
    fn silhouette(solid: &Solid, angle: f64) -> DVec2 {
        let u = DVec2::new(angle.cos(), angle.sin());
        let (mut lo, mut hi) = (0., solid.half.length() + 1.);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if solid.sigma(u * mid, u * mid) <= 0. {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        u * lo
    }

    /// What the certified reference says of one ray.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(in crate::render_helpers::material::view) enum Certified {
        /// The entry's depth parameter, bisected to 1e-15 of the piece.
        Hit(f64),
        Miss,
        /// Neither proved within the refinement budget: a ray tangent to the solid.
        Unresolved,
    }

    /// The first entry, certified by convexity rather than by sampling alone. Ψ is convex on
    /// each piece (`bracketing_matches_the_certified_intersection` asserts it), so:
    /// - a sample at or below 0 proves a hit, and the entry lies between it and the last
    ///   positive sample before it (the sublevel set of a convex function is an interval);
    /// - on an interval between samples, the chords through the two samples on either side,
    ///   extended across it, bound a convex function from below; where both stay above 0
    ///   the interval holds no hit.
    /// Intervals neither cleared nor hit are halved, up to 48 times; what is left is
    /// unresolved. A thin dip between two samples is therefore found or reported, never
    /// passed over.
    pub(in crate::render_helpers::material::view) fn certified_entry(
        solid: &Solid,
        seg: &Segment,
    ) -> Certified {
        for piece in pieces(solid, seg) {
            let f = |t: f64| solid.psi(seg.point(t, solid.thickness), piece.sel);
            match certify(&f, piece.a, piece.b) {
                Certified::Miss => continue,
                other => return other,
            }
        }
        Certified::Miss
    }

    fn certify(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> Certified {
        let mut pts: Vec<(f64, f64)> = (0..=64)
            .map(|i| a + (b - a) * f64::from(i) / 64.)
            .map(|t| (t, f(t)))
            .collect();
        for _ in 0..48 {
            if let Some(i) = pts.iter().position(|&(_, v)| v <= 0.) {
                if i == 0 {
                    return Certified::Hit(a);
                }
                let (mut lo, mut hi) = (pts[i - 1].0, pts[i].0);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    if f(mid) <= 0. {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                return Certified::Hit(hi);
            }
            let chord = |i: usize, j: usize, t: f64| {
                let ((ti, fi), (tj, fj)) = (pts[i], pts[j]);
                fi + (fj - fi) * (t - ti) / (tj - ti)
            };
            let mut next = vec![pts[0]];
            let mut open = false;
            for i in 0..pts.len() - 1 {
                let (l, r) = (pts[i].0, pts[i + 1].0);
                // Lazy: `then_some` would evaluate `i - 1` at i = 0 and underflow.
                let left = (i >= 1).then(|| (i - 1, i));
                let right = (i + 2 < pts.len()).then(|| (i + 1, i + 2));
                let bound = |t: f64| {
                    let mut v = f64::NEG_INFINITY;
                    if let Some((p, q)) = left {
                        v = v.max(chord(p, q, t));
                    }
                    if let Some((p, q)) = right {
                        v = v.max(chord(p, q, t));
                    }
                    v
                };
                // The max of two lines is lowest at an end or where they cross.
                let mut low = bound(l).min(bound(r));
                if let (Some((p, q)), Some((u, w))) = (left, right) {
                    let (s1, s2) = (
                        (pts[q].1 - pts[p].1) / (pts[q].0 - pts[p].0),
                        (pts[w].1 - pts[u].1) / (pts[w].0 - pts[u].0),
                    );
                    if s1 != s2 {
                        let t = (pts[u].1 - s2 * pts[u].0 - pts[p].1 + s1 * pts[p].0) / (s1 - s2);
                        if t > l && t < r {
                            low = low.min(bound(t));
                        }
                    }
                }
                if low <= 0. {
                    open = true;
                    let mid = 0.5 * (l + r);
                    next.push((mid, f(mid)));
                }
                next.push(pts[i + 1]);
            }
            if !open {
                return Certified::Miss;
            }
            pts = next;
        }
        Certified::Unresolved
    }

    #[test]
    fn the_certifier_proves_misses_and_hits_on_known_functions() {
        // Plan review round 2: a function positive everywhere reaches the chord bounds on the
        // first pass, so a miss must come back certified, not panic.
        assert_eq!(certify(&|t: f64| 1. + t * t, 0., 1.), Certified::Miss);
        assert_eq!(certify(&|t: f64| (t - 0.5).powi(2) + 1e-3, 0., 1.), Certified::Miss);
        // A dip below 0 narrower than the first sample spacing (1/64): found, and bisected.
        let dip = |t: f64| (t - 0.5004).abs() - 1e-4;
        match certify(&dip, 0., 1.) {
            Certified::Hit(t) => assert!((t - 0.5003).abs() < 1e-12, "entry {t}"),
            other => panic!("{other:?}"),
        }
        assert_eq!(certify(&|t: f64| t - 0.25, 0., 1.), Certified::Hit(0.), "inside at the start");
        // Tangent to 0: neither proved.
        assert_eq!(certify(&|t: f64| (t - 0.3).powi(2), 0., 1.), Certified::Unresolved);
    }

    /// A random slab, look and frame from the configured ranges.
    fn random_case(rnd: &mut impl FnMut() -> f64) -> (Solid, Camera) {
        let half = DVec2::new(20. + rnd() * 780., 20. + rnd() * 580.);
        let chamfer = if rnd() < 0.15 { 0. } else { rnd() * 20. };
        let thickness = if rnd() < 0.15 { 0. } else { rnd() * 200. };
        let face_r: Radii = if rnd() < 0.3 {
            [88., 12., 12., 12.]
        } else {
            std::array::from_fn(|_| rnd() * 30.)
        };
        let profile = 1. + rnd() * 7.;
        let angle = rnd() * 2. * PI;
        let pose = ViewPose::at(
            1. + rnd() * 19.,
            Duration::from_millis(900),
            Duration::from_secs_f64(0.9 * (0.02 + rnd() * 0.96)),
            DVec2::new(angle.cos(), angle.sin()),
            1.5 + rnd() * 48.5,
        )
        .unwrap();
        setup(half, chamfer, face_r, thickness, profile, &pose)
    }

    /// Screen points within about a pixel of the projected outline.
    fn near_outline(solid: &Solid, camera: &Camera, rnd: &mut impl FnMut() -> f64) -> DVec2 {
        let edge = silhouette(solid, rnd() * 2. * PI);
        let z = -rnd() * solid.thickness;
        camera.to_screen(edge.extend(z)) + DVec2::new(rnd() * 3. - 1.5, rnd() * 3. - 1.5)
    }

    #[test]
    fn psi_is_the_resting_height_field() {
        let mut rnd = uniforms(10);
        for _ in 0..200 {
            let k = 1. + rnd() * 7.;
            let chamfer = 2. + rnd() * 18.;
            let thickness = rnd() * 200.;
            let half = DVec2::new(100., 80.);
            let face_r = rnd() * 30.;
            let (solid, _) = setup(half, chamfer, [face_r; 4], thickness, k, &peak(0., DVec2::X, 6.));
            let slab = Slab::at_rest(half, chamfer, solid.outer_r[0] - chamfer, thickness, k);
            for _ in 0..200 {
                let p = DVec2::new(rnd() * 240. - 120., rnd() * 200. - 100.);
                let z = -rnd() * thickness;
                let surface = slab.surface(p);
                if surface.outer_dist > 0. {
                    assert!(solid.psi(p.extend(z), p) > 0., "outside the silhouette at {p}");
                    continue;
                }
                let top = surface.height - thickness;
                if (z - top).abs() < 1e-6 {
                    continue;
                }
                assert_eq!(solid.psi(p.extend(z), p) <= 0., z <= top, "{p} z {z} top {top}");
            }
        }
    }

    #[test]
    fn bracketing_matches_the_certified_intersection() {
        let mut rnd = uniforms(11);
        let (mut rays, mut later, mut skipped) = (0, 0, 0);
        for _ in 0..120 {
            let (solid, camera) = random_case(&mut rnd);
            for _ in 0..40 {
                let s = near_outline(&solid, &camera, &mut rnd);
                let Some(seg) = Segment::new(&camera, s) else {
                    panic!("a ray near the slab points up at {s}");
                };
                // Convexity of Ψ along every piece.
                for piece in pieces(&solid, &seg) {
                    let f = |t: f64| solid.psi(seg.point(t, solid.thickness), piece.sel);
                    for i in 1..63 {
                        let t = |j: usize| piece.a + (piece.b - piece.a) * j as f64 / 63.;
                        let second = f(t(i - 1)) + f(t(i + 1)) - 2. * f(t(i));
                        assert!(second >= -1e-7 * (1. + f(t(i)).abs()), "not convex: {second}");
                    }
                }
                let got = trace(&solid, &seg);
                assert!(got.psi_min.is_finite() && got.t.is_finite(), "NaN at {s}");
                let exact = match certified_entry(&solid, &seg) {
                    Certified::Unresolved => {
                        skipped += 1;
                        continue;
                    }
                    Certified::Hit(t) => Some(t),
                    Certified::Miss => None,
                };
                rays += 1;
                assert_eq!(got.hit, exact.is_some(), "hit or miss at {s}: {got:?}");
                if let Some(t) = exact {
                    let depth = (got.t - t).abs() * solid.thickness;
                    assert!(depth <= 1. / 128., "entry depth off by {depth} px at {s}");
                    if pieces(&solid, &seg).len() > 1 && t > pieces(&solid, &seg)[0].b {
                        later += 1;
                    }
                }
            }
        }
        assert!(skipped * 100 < rays, "{skipped} unresolved (tangent) rays of {rays}");
        assert!(later > 0, "no ray's first hit lay in a later piece");
    }

    #[test]
    fn asymmetric_radii_find_the_hit_in_a_later_piece() {
        // (88, 12, 12, 12) on a 100 px slab (§4.1 "Quadrant seams"): at y = −49 the big
        // top-left corner leaves x < 0 outside (σ ≈ +6.9) while x > 0 is inside (σ ≈ −1). In the
        // overshoot the rays drift toward +x as they descend, so a ray that starts at x < 0 can
        // enter only after it crosses x = 0, in its second piece.
        let pose = ViewPose {
            theta: -10f64.to_radians(),
            ..peak(10., DVec2::X, 6.)
        };
        let (solid, camera) = setup(DVec2::splat(50.), 0., [88., 12., 12., 12.], 31.2, 2., &pose);
        let mut found = 0;
        for i in 0..4000 {
            let s = DVec2::new(-8. + i as f64 * 0.002, -49.);
            let seg = Segment::new(&camera, s).unwrap();
            let ps = pieces(&solid, &seg);
            let got = trace(&solid, &seg);
            let exact = match certified_entry(&solid, &seg) {
                Certified::Unresolved => continue,
                Certified::Hit(t) => Some(t),
                Certified::Miss => None,
            };
            assert_eq!(got.hit, exact.is_some(), "at {s}");
            if let Some(t) = exact {
                assert!((got.t - t).abs() * solid.thickness <= 1. / 128., "at {s}");
                if ps.len() > 1 && t >= ps[1].a {
                    found += 1;
                }
            }
        }
        assert!(found > 0, "no ray entered through the second piece");
    }

    #[test]
    fn zero_rise_is_the_box_or_the_plane() {
        let mut rnd = uniforms(12);
        for (chamfer, thickness) in [(0., 31.2), (12., 0.), (0., 0.)] {
            let (solid, camera) = setup(DVec2::new(200., 150.), chamfer, [8.; 4], thickness, 2., &peak(14., DVec2::new(0.6, 0.8), 2.5));
            for _ in 0..2000 {
                let s = near_outline(&solid, &camera, &mut rnd);
                let seg = Segment::new(&camera, s).unwrap();
                let got = trace(&solid, &seg);
                assert!(got.psi_min.is_finite() && got.t.is_finite(), "NaN at {s}");
                if thickness == 0. {
                    let sigma = solid.sigma(seg.h0.truncate(), seg.h0.truncate());
                    assert_eq!(got.hit, sigma <= 0.);
                    assert_eq!((got.t, got.wall), (0., false), "a plane has no wall");
                } else {
                    match certified_entry(&solid, &seg) {
                        Certified::Hit(_) => assert!(got.hit, "box at {s}"),
                        Certified::Miss => assert!(!got.hit, "box at {s}"),
                        Certified::Unresolved => {}
                    }
                }
            }
        }
    }

    /// The entry of an affine Ψ(t) = a + b t on [lo, hi], if any.
    fn affine_entry(a: f64, b: f64, lo: f64, hi: f64) -> Option<f64> {
        let at = |t: f64| a + b * t;
        if at(lo) <= 0. {
            Some(lo)
        } else if b < 0. && at(hi) <= 0. {
            Some(-a / b)
        } else {
            None
        }
    }

    #[test]
    fn analytic_entries_match_closed_form() {
        // Profile 1 is a planar chamfer, X(y) = y: along the row through a straight side's
        // middle, σ = x − half.x and the bevel term are both affine in t, with one kink at the
        // rim depth, so the entry has a closed form. Bevel 0 is a box. Both are analytic
        // controls for the mirror and for the certified reference.
        let mut checked = 0;
        for (chamfer, thickness, deg, kappa) in [(12., 31.2, 10., 6.), (0., 31.2, 14., 2.5), (20., 10., 20., 1.5), (12., 31.2, -10., 6.)] {
            let pose = ViewPose { theta: f64::to_radians(deg), ..peak(10., DVec2::X, kappa) };
            let (solid, camera) = setup(DVec2::new(400., 300.), chamfer, [0.; 4], thickness, 1., &pose);
            let rise = solid.rise();
            for i in 0..4000 {
                let s = DVec2::new(370. + f64::from(i) * 0.015, 0.);
                let seg = Segment::new(&camera, s).unwrap();
                // x(t) = h0.x + ρ.x T t; above the rim the bevel adds c (1 − T t / rise).
                let (x0, dx) = (seg.h0.x - solid.half.x, seg.rho.x * thickness);
                let rim = if rise > 0. { (rise / thickness).min(1.) } else { 0. };
                let exact = if rise > 0. {
                    affine_entry(x0 + chamfer, dx - chamfer * thickness / rise, 0., rim)
                } else {
                    None
                }
                .or_else(|| affine_entry(x0, dx, rim, 1.));
                let got = trace(&solid, &seg);
                assert_eq!(got.hit, exact.is_some(), "{chamfer} {thickness} {deg}° at {s}: {got:?}");
                if let Some(t) = exact {
                    assert!((got.t - t).abs() * thickness <= 1. / 128., "{deg}° at {s}: {} vs {t}", got.t);
                    match certified_entry(&solid, &seg) {
                        Certified::Hit(c) => assert!((c - t).abs() * thickness <= 1e-9, "the reference at {s}"),
                        other => panic!("the reference says {other:?} at {s}"),
                    }
                    checked += 1;
                } else {
                    assert!(!matches!(certified_entry(&solid, &seg), Certified::Hit(_)), "the reference hits at {s}");
                }
            }
        }
        assert!(checked > 1000, "{checked} analytic hits");
    }

    #[test]
    fn the_rim_graze_ray_hits_where_a_fixed_march_misses() {
        // §4.1: a peak ray of an 800 px slab at 10°, κ 6, profile 2, bevel 12, thickness 31.2 is
        // inside only between depths −8.93 and −9.02 px on the receding edge.
        let (solid, camera) = setup(DVec2::new(400., 300.), 12., [0.; 4], 31.2, 2., &peak(10., DVec2::X, 6.));
        let s = DVec2::new(-385.69676, 0.);
        let seg = Segment::new(&camera, s).unwrap();
        let got = trace(&solid, &seg);
        assert!(got.hit, "{got:?}");
        let depth = got.t * solid.thickness;
        assert!((depth - 8.93).abs() < 0.01, "entry depth {depth}");
        let marched = (0..=64).any(|i| solid.psi(seg.point(i as f64 / 64., 31.2), s) <= 0.);
        assert!(!marched, "the negative control: a 64-step march must miss");
    }

    /// Wall hits along a screen line across the edge whose outward normal is `n0`, between the
    /// projections of its rim and back edges, at `step` px.
    fn wall_width(solid: &Solid, camera: &Camera, edge: DVec2, n0: DVec2, step: f64) -> f64 {
        let a = camera.to_screen(edge.extend(-solid.rise())) - n0 * 2.;
        let b = camera.to_screen(edge.extend(-solid.thickness)) + n0 * 2.;
        let len = (b - a).dot(n0).abs().max(4.);
        let n = (len / step) as usize;
        let walls = (0..n)
            .filter(|&i| {
                let s = a + n0 * (i as f64 * step) * (b - a).dot(n0).signum();
                Segment::new(camera, s).is_some_and(|seg| {
                    let t = trace(solid, &seg);
                    t.hit && t.wall
                })
            })
            .count();
        walls as f64 * step
    }

    #[test]
    fn peak_wall_widths_match_the_spec_table() {
        // §2.1: the active slab, thickness 31.2 and bevel 12, so a wall 19.2 px deep below the
        // rim, at the swing's peak.
        for (peak_deg, widths) in [(10., [0.96, 1.77, 2.40]), (14., [2.36, 3.14, 3.76])] {
            for (kappa, want) in [4., 6., 10.].into_iter().zip(widths) {
                let (solid, camera) = setup(DVec2::new(400., 300.), 12., [0.; 4], 31.2, 2., &peak(peak_deg, DVec2::X, kappa));
                let got = wall_width(&solid, &camera, DVec2::new(400., 0.), DVec2::X, 0.002);
                assert!((got - want).abs() <= 0.1, "{peak_deg}° κ {kappa}: {got} px, want {want}");
            }
        }
    }

    #[test]
    fn the_wall_rule_agrees_with_the_ray_cast() {
        let settle = Duration::from_millis(900);
        let half = DVec2::new(400., 300.);
        // (peak, κ, dir, rounded corner radius): every frame, no frame, and the corner cases.
        for (peak_deg, kappa, dir, radius) in [
            (10., 6., DVec2::X, 0.),
            (12., 2.5, DVec2::X, 0.),
            (11., 2.5, DVec2::X, 0.),
            (10., 6., DVec2::Y, 0.),
            (14., 6., DVec2::new(0.6, 0.8), 60.),
        ] {
            let mut shown = 0;
            for frame in 1..54 {
                let elapsed = Duration::from_secs_f64(frame as f64 / 60.);
                let Some(pose) = ViewPose::at(peak_deg, settle, elapsed, dir, kappa) else {
                    continue;
                };
                let (solid, camera) = setup(half, 12., [radius; 4], 31.2, 2., &pose);
                let s_hat = dir * pose.theta.signum();
                // The wall point whose outward normal is ŝ, and its distance from the pivot.
                let r = solid.outer_r[0];
                let core = (half - DVec2::splat(r)) * s_hat.signum();
                let edge = if radius > 0. { core + s_hat * r } else { half * s_hat.signum() * s_hat.abs().ceil() };
                let h = edge.dot(s_hat);
                let extent = dir.x.abs() * 2. * half.x + dir.y.abs() * 2. * half.y;
                let rule = wall_faces_camera(&pose, extent, s_hat, h);
                let width = wall_width(&solid, &camera, edge, s_hat, 1. / 256.);
                if pose.theta.abs() < 0.5f64.to_radians() {
                    continue;
                }
                assert_eq!(width > 0., rule, "{peak_deg}° κ {kappa} {dir} frame {frame}: width {width}");
                shown += usize::from(rule);
            }
            let all = peak_deg.to_radians().sin() > 1. / (2. * kappa);
            let none = peak_deg.to_radians() <= 1. / (2. * kappa);
            if dir == DVec2::X && none {
                assert_eq!(shown, 0, "{peak_deg}° κ {kappa}: no frame may show a wall");
            }
            if dir == DVec2::X && all {
                assert!(shown > 30, "{peak_deg}° κ {kappa}: every moving frame shows a wall");
            }
        }
    }
}
```

In `view/mod.rs`, add `pub mod solid;` after the `use` lines, and after `footprint`:

```rust
/// Whether a wall with in-plane outward normal `n0`, `h` px from the pivot, faces the camera
/// under `pose` (§2.1): D sin|θ| (n₀ · ŝ) > h, with D = κE / w and ŝ = sign(θ) dir.
pub fn wall_faces_camera(pose: &ViewPose, extent: f64, n0: DVec2, h: f64) -> bool {
    if pose.theta == 0. {
        return false;
    }
    let s_hat = pose.dir * pose.theta.signum();
    pose.kappa * extent / pose.weight * pose.theta.abs().sin() * n0.dot(s_hat) > h
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `just test-one -p niri view::solid::tests`
Expected: `psi_is_the_resting_height_field` PASS; every other test panics with `not implemented: Task 4 Step 3`.

- [ ] **Step 3: Implement the trace**

Replace the body of `trace`:

```rust
pub fn trace(solid: &Solid, seg: &Segment) -> Trace {
    let thickness = solid.thickness;
    let mut best = Trace {
        hit: false,
        wall: false,
        t: 0.,
        psi_min: f64::INFINITY,
        min_t: 0.,
        min_plane: DVec3::Z,
        min_sel: DVec2::ZERO,
    };
    let mut entry = None;
    for piece in pieces(solid, seg) {
        let f = |t: f64| solid.psi(seg.point(t, thickness), piece.sel);
        let (t, psi, end) = minimise(&f, piece.a, piece.b);
        if psi < best.psi_min {
            best.psi_min = psi;
            best.min_t = t;
            best.min_sel = piece.sel;
            best.min_plane = match end {
                End::A => piece.na,
                End::B => piece.nb,
                End::Interior => DVec3::Z,
            };
        }
        if entry.is_none() && psi <= 0. {
            let at = if f(piece.a) <= 0. {
                piece.a
            } else {
                bisect(&f, piece.a, t)
            };
            entry = Some((at, piece.sel));
        }
    }
    best.hit = entry.is_some();
    let (t, sel) = entry.unwrap_or((best.min_t, best.min_sel));
    let rim = if thickness > 0. {
        (solid.rise() / thickness).min(1.)
    } else {
        0.
    };
    best.wall = t > rim && solid.psi(seg.point(rim, thickness), sel) > 0.;
    best.t = t;
    best
}
```

- [ ] **Step 4: Run the tests**

Run: `just test-one -p niri view::`
Expected: every `view::` test PASS. If `the_wall_rule_agrees_with_the_ray_cast` fails only on frames whose predicted width is under 1/256 px (check by printing `width` for the failing frame at a step of 1/4096), raise the skip angle from 0.5° to the smallest angle that clears them and say so in the task's done note; any other failure is a mirror defect.

- [ ] **Step 5: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-d74c02 "traced slab, bracketing, wall-reveal rule; matches the certified intersection and the closed forms"
tasks check
git add src/render_helpers/material/view/ tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): rigid tilt traced slab and bracketing"
```

### Task 5: Outline distance and its bound

**Files:**
- Modify: `src/render_helpers/material/view/solid.rs`

**Interfaces:**
- Consumes: `Solid`, `Segment`, `Trace`, `trace` (Task 4); `Camera` (Task 3).
- Produces: `solid::outline(solid: &Solid, camera: &Camera, seg: &Segment, tr: &Trace) -> f64`, the signed screen distance `m`, negative inside, in logical px.

The bound (§4.5): within one physical px of the outline, `|m − exact| ≤ 1/16` physical px on straight sides and `≤ 1/2` physical px at corners (the region within `r + 1.5` px of a projected corner arc centre on the face or back plane), for any swing direction and any thickness, zero included. The tests run at scale 1, so physical and logical px agree. "Exact" is the signed distance to the zero set of the mirror's Ψmin, which Task 4 checked against a dense intersection.

- [ ] **Step 1: Write the failing tests**

Add to `solid.rs`, after `trace`, the stub:

```rust
/// The outline distance m (§4.5): Ψmin over the norm of its screen gradient, taken at the
/// minimiser on the plane it is held to, and written as a correction to Ψmin that is exactly
/// zero at the identity. Negative inside. Mirrors `viewOutline`.
pub fn outline(solid: &Solid, camera: &Camera, seg: &Segment, tr: &Trace) -> f64 {
    let _ = (solid, camera, seg, tr);
    unimplemented!("Task 5 Step 3")
}
```

and to its `tests` module:

```rust
    /// The trace and m at screen point `s`.
    fn outline_at(solid: &Solid, camera: &Camera, s: DVec2) -> (Trace, f64) {
        let seg = Segment::new(camera, s).expect("a downward ray");
        let tr = trace(solid, &seg);
        let m = outline(solid, camera, &seg, &tr);
        (tr, m)
    }

    /// Where Ψmin changes sign between `inside` and `outside`: Illinois regula falsi on the
    /// segment, to 1e-9 px.
    fn crossing(solid: &Solid, camera: &Camera, inside: DVec2, outside: DVec2) -> DVec2 {
        let f = |x: f64| outline_at(solid, camera, inside.lerp(outside, x)).0.psi_min;
        let (mut a, mut b) = (0., 1.);
        let (mut fa, mut fb) = (f(a), f(b));
        assert!(fa <= 0. && fb > 0., "not a bracket: {fa} {fb}");
        let tol = 1e-9 / (outside - inside).length();
        let mut side = 0;
        for _ in 0..100 {
            let c = (a * fb - b * fa) / (fb - fa);
            let fc = f(c);
            if fc <= 0. {
                (a, fa) = (c, fc);
                if side == -1 {
                    fb *= 0.5;
                }
                side = -1;
            } else {
                (b, fb) = (c, fc);
                if side == 1 {
                    fa *= 0.5;
                }
                side = 1;
            }
            if b - a < tol {
                break;
            }
        }
        inside.lerp(outside, 0.5 * (a + b))
    }

    #[test]
    fn the_outline_distance_is_sigma_at_the_identity() {
        let mut rnd = uniforms(20);
        for (chamfer, thickness) in [(12., 31.2), (0., 31.2), (12., 0.), (0., 0.), (0.8, 2.), (15., 200.)] {
            for dir in [DVec2::X, DVec2::new(0.6, 0.8)] {
                let (solid, camera) = setup(DVec2::new(300., 200.), chamfer, [10., 2., 30., 0.], thickness, 2.5, &ViewPose::identity(dir, 6.));
                for _ in 0..4000 {
                    let s = DVec2::new(rnd() * 640. - 320., rnd() * 440. - 220.);
                    let (_, m) = outline_at(&solid, &camera, s);
                    assert_eq!(m, solid.sigma(s, s), "at {s}");
                }
            }
        }
    }

    /// The cases of §8.1: outer radius 0 (bevel 0, corner radius 0), 0.8 (bevel 0.8, corner
    /// radius 0) and 2 (bevel 0.8, corner radius 1.2); thickness 0, 0.01, 2, 31.2 and 200;
    /// horizontal, vertical and diagonal swings at 20° and κ 1.5, plus the diagonal at 6°, κ 6.
    fn bound_cases() -> Vec<(String, Solid, Camera)> {
        let mut out = Vec::new();
        let swings = [
            (20., 1.5, DVec2::X),
            (20., 1.5, DVec2::Y),
            (20., 1.5, DVec2::new(0.6, 0.8)),
            (6., 6., DVec2::new(0.6, 0.8)),
        ];
        for (outer, chamfer, face_r) in [(0., 0., 0.), (0.8, 0.8, 0.), (2., 0.8, 1.2)] {
            for thickness in [0., 0.01, 2., 31.2, 200.] {
                for (i, (peak_deg, kappa, dir)) in swings.into_iter().enumerate() {
                    if i == 3 && thickness != 31.2 {
                        continue;
                    }
                    let (solid, camera) = setup(DVec2::splat(50.), chamfer, [face_r; 4], thickness, 2., &peak(peak_deg, dir, kappa));
                    out.push((format!("r {outer} T {thickness} {peak_deg}° κ {kappa} {dir}"), solid, camera));
                }
            }
        }
        out
    }

    #[test]
    fn the_outline_distance_meets_its_bound_on_straight_sides() {
        for (name, solid, camera) in bound_cases() {
            let centre = camera.to_screen(solid.center.extend(-0.5 * solid.thickness));
            for side in [DVec2::X, DVec2::NEG_X, DVec2::Y, DVec2::NEG_Y] {
                let tangent = side.perp();
                // Two boundary points 4 px either side of the side's midpoint give its line.
                let cross = |offset: f64| {
                    let inside = centre + tangent * offset;
                    crossing(&solid, &camera, inside, inside + side * 200.)
                };
                let (a, b) = (cross(-4.), cross(4.));
                let normal = (b - a).perp().normalize();
                let normal = if normal.dot(side) < 0. { -normal } else { normal };
                let mid = cross(0.);
                for i in -96..=96 {
                    let s = mid + normal * (i as f64 / 64.);
                    let (tr, m) = outline_at(&solid, &camera, s);
                    let exact = (s - a).dot(normal);
                    let exact = if tr.psi_min <= 0. { -exact.abs() } else { exact.abs() };
                    if exact.abs() > 1. {
                        continue;
                    }
                    assert!((m - exact).abs() <= 1. / 16., "{name} side {side}: m {m} exact {exact} at {s}");
                }
            }
        }
    }

    /// The corner check of §8.1: a screen grid at `spacing` over ±(r + 1.5) px around the
    /// projected arc centre on the face plane and on the back plane, against the signed distance
    /// to the boundary polyline traced from an interior anchor at `chord` spacing.
    fn check_corner(name: &str, solid: &Solid, camera: &Camera, corner: DVec2, spacing: f64, chord: f64) -> f64 {
        let r = corner_radius(corner, solid.outer_r);
        let arc = solid.center + corner * (solid.half - DVec2::splat(r));
        let reach = r + 1.5;
        let anchor = camera.to_screen((arc - corner * 3.).extend(-0.5 * solid.thickness));
        let centres = [camera.to_screen(arc.extend(0.)), camera.to_screen(arc.extend(-solid.thickness))];
        // The boundary, by angle from the anchor, over every grid point's direction.
        let mut far = 0f64;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let base = corner.normalize();
        for c in centres {
            for (x, y) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
                let v = c + DVec2::new(x, y) * reach - anchor;
                far = far.max(v.length());
                let a = base.perp_dot(v).atan2(base.dot(v));
                (lo, hi) = (lo.min(a), hi.max(a));
            }
        }
        let radius = far + 2.;
        let steps = ((hi - lo + 0.4) * radius / chord).ceil() as usize;
        let mut boundary = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let a = lo - 0.2 + (hi - lo + 0.4) * i as f64 / steps as f64;
            let u = base.rotate(DVec2::new(a.cos(), a.sin()));
            if outline_at(solid, camera, anchor + u * radius).0.psi_min <= 0. {
                continue;
            }
            boundary.push(crossing(solid, camera, anchor, anchor + u * radius));
        }
        // Segments bucketed by cell, so each grid point looks only near itself.
        let cell = 0.5;
        let key = |p: DVec2| ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64);
        let mut buckets: std::collections::HashMap<(i64, i64), Vec<usize>> = Default::default();
        for i in 1..boundary.len() {
            buckets.entry(key(0.5 * (boundary[i - 1] + boundary[i]))).or_default().push(i);
        }
        let distance = |p: DVec2| {
            let (kx, ky) = key(p);
            let mut best = f64::INFINITY;
            for dx in -4..=4 {
                for dy in -4..=4 {
                    for &i in buckets.get(&(kx + dx, ky + dy)).into_iter().flatten() {
                        let (a, b) = (boundary[i - 1], boundary[i]);
                        let t = ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.);
                        best = best.min((a + (b - a) * t - p).length());
                    }
                }
            }
            best
        };
        let mut worst = 0f64;
        let n = (reach / spacing).ceil() as i64;
        for c in centres {
            for i in -n..=n {
                for j in -n..=n {
                    let s = c + DVec2::new(i as f64, j as f64) * spacing;
                    let d = distance(s);
                    if d > 1. {
                        continue;
                    }
                    let (tr, m) = outline_at(solid, camera, s);
                    let exact = if tr.psi_min <= 0. { -d } else { d };
                    let err = (m - exact).abs();
                    assert!(err <= 0.5, "{name} corner {corner}: m {m} exact {exact} at {s}");
                    worst = worst.max(err);
                }
            }
        }
        worst
    }

    const CORNERS: [DVec2; 4] = [
        DVec2::new(-1., -1.),
        DVec2::new(1., -1.),
        DVec2::new(1., 1.),
        DVec2::new(-1., 1.),
    ];

    #[test]
    fn the_outline_distance_meets_its_bound_at_corners_coarse() {
        for (name, solid, camera) in bound_cases() {
            let quick = (name.starts_with("r 0.8 T 0 ") && name.ends_with("[1, 0]"))
                || (name.starts_with("r 0 T 31.2 20") && name.ends_with("[0.6, 0.8]"))
                || (name.starts_with("r 0.8 T 200 ") && name.ends_with("[0.6, 0.8]"));
            if !quick {
                continue;
            }
            for corner in CORNERS {
                check_corner(&name, &solid, &camera, corner, 1. / 8., 1. / 32.);
            }
        }
    }

    #[test]
    #[ignore = "the dense corner grid of §8.1, minutes in a debug build: run with --run-ignored only"]
    fn the_outline_distance_meets_its_bound_at_corners() {
        let mut worst = 0f64;
        for (name, solid, camera) in bound_cases() {
            for corner in CORNERS {
                let w = check_corner(&name, &solid, &camera, corner, 1. / 32., 1. / 64.);
                eprintln!("{name} corner {corner}: worst {w:.4} px");
                worst = worst.max(w);
            }
        }
        eprintln!("worst corner error over every case: {worst:.4} px (bound 0.5)");
    }

    #[test]
    fn a_seam_step_keeps_coverage_in_range_and_continuous_within_each_piece() {
        // §4.5 "Exception": (88, 12, 12, 12) steps 8.6 px at the top seam, already at rest.
        let (solid, camera) = setup(DVec2::splat(50.), 0.8, [88., 12., 12., 12.], 31.2, 2., &peak(14., DVec2::X, 2.5));
        let coverage = |m: f64| {
            let x = ((m + 1.) / 1.).clamp(0., 1.);
            1. - x * x * (3. - 2. * x)
        };
        for y in [-50.5, -50., -49.5, -49.] {
            let mut prev: Option<(DVec2, f64)> = None;
            for i in 0..4000 {
                let s = DVec2::new(-20. + i as f64 * 0.01, y);
                let (tr, m) = outline_at(&solid, &camera, s);
                let c = coverage(m);
                assert!((0. ..=1.).contains(&c), "{c} at {s}");
                if let Some((sel, pc)) = prev {
                    if sel.x.signum() == tr.min_sel.x.signum() {
                        assert!((c - pc).abs() <= 0.1, "coverage jumps {pc} → {c} within a piece at {s}");
                    }
                }
                prev = Some((tr.min_sel, c));
            }
        }
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri view::solid::tests::the_outline`
Expected: FAIL: `not implemented: Task 5 Step 3`.

- [ ] **Step 3: Implement the outline distance**

Replace the body of `outline`:

```rust
pub fn outline(solid: &Solid, camera: &Camera, seg: &Segment, tr: &Trace) -> f64 {
    let q = seg.point(tr.min_t, solid.thickness);
    let g0 = solid.sigma_grad(q.truncate(), tr.min_sel);
    let gq = g0.extend(solid.psi_dz(q));
    // ∂q/∂s_j for a minimiser held to the plane of normal n:
    // (1 + τλ)(R⁻¹e_j − r (n · R⁻¹e_j) / (n · r)), τ the ray parameter at q.
    let n = tr.min_plane;
    let tau = seg.tau0 + tr.min_t * solid.thickness / -seg.r.z;
    let scale = 1. + tau * camera.lambda;
    let column = |e: DVec3| {
        let u = e + camera.unrotate_delta(e);
        scale * (u - seg.r * (n.dot(u) / n.dot(seg.r)))
    };
    let g = DVec2::new(gq.dot(column(DVec3::X)), gq.dot(column(DVec3::Y)));
    let len = g.length();
    if len == 0. {
        return tr.psi_min;
    }
    tr.psi_min + tr.psi_min * (g0.length() - len) / len
}
```

- [ ] **Step 4: Run the tests, then the dense grid**

Run: `just test-one -p niri view::`, then `just test-one -p niri --run-ignored only view::solid::tests::the_outline_distance_meets_its_bound_at_corners`.
Expected: all PASS. The ignored run prints each case's worst corner error and the overall worst; the design model's worst was 0.32 px (diagonal, sharp corner, thickness 31.2 or 200). Record the printed overall worst in the done note. A corner case over 0.5 is a defect in the outline distance, not a reason to raise the bound: stop and report the case and the point.

- [ ] **Step 5: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-526ae1 "outline distance; sides within 1/16 px, corners within 1/2 px (dense grid worst <w> px)"
tasks check
git add src/render_helpers/material/view/solid.rs tasks/
git commit -m "feat(material): rigid tilt outline distance"
```

### Task 6: Swing state on the tile, focus origin, layout tests

**Files:**
- Create: `src/layout/focus_origin.rs` (from `material-77db8a`, `d51825f9`)
- Create: `src/layout/tests/view_swing.rs` (from `d51825f9`, adapted)
- Modify: `src/layout/mod.rs` (`mod focus_origin;`, the `focus_origin` field and its two initialisers, the observation in `update_render_elements`, `LayoutElement::background_effect_visible`)
- Modify: `src/layout/monitor.rs` (`tile_centre_on_output`, `tile_mut`)
- Modify: `src/layout/tests.rs` (`mod view_swing;`, `TestWindow::background_effect_visible`)
- Modify: `src/window/mapped.rs` (`background_effect_visible`)
- Modify: `src/render_helpers/background_effect.rs` (`surface_effect_visible`)
- Modify: `src/render_helpers/material/view/mod.rs` (`hooks`)
- Modify: `src/layout/tile.rs` (`ViewSwing`, the field, `motion_allowed`, `start_view_swing`, eligibility, the cut, `view_pose`, `MaterialDynamics.view`, `advance_animations`, `are_animations_ongoing`, tests)

**Interfaces:**
- Consumes: `view::{ViewPose, SwingFrom, direction_between, seeded_direction}` (Task 2); `ResolvedResponse.{view_tilt, view_tilt_settle, view_distance}` (Task 1).
- Produces: `Tile::start_view_swing(&mut self, from: SwingFrom)`; `Tile::view_pose(&self) -> Option<ViewPose>`; `MaterialDynamics.view: Option<ViewPose>`; `#[cfg(test)] Tile::view_swing_start(&self) -> Option<(Duration, [f64; 2])>`, `Tile::fallback_direction_at(&self, Duration) -> [f64; 2]`; `LayoutElement::background_effect_visible(&self) -> bool`; `background_effect::surface_effect_visible(&WlSurface) -> bool`; `#[cfg(test)] view::hooks::POSE: LocalKey<Cell<Option<ViewPose>>>`.

- [ ] **Step 1: Port the focus origin, the monitor lookups and the layout observation**

Run `git show d51825f9:src/layout/focus_origin.rs > src/layout/focus_origin.rs`. In it, replace the module doc's first two lines with:

```rust
//! Where the rigid tilt's swing comes from (docs/specs/2026-10-09-transient-rigid-tilt-design.md
//! §3.2, §5): the last focus an update pass observed. The `Layout` observes the one
```

The `use crate::render_helpers::material::view::{direction_between, SwingFrom};` line already names the Task 2 items.

In `src/layout/mod.rs`, after `pub mod floating;` add `mod focus_origin;`; in `struct Layout`, after `overview_progress`, add:

```rust
    /// The last focus an update pass observed, where the next focus's swing comes from
    /// (design 2026-10-09 §5).
    focus_origin: Option<focus_origin::FocusOrigin<W::Id, Output>>,
```

and `focus_origin: None,` after `overview_progress: None,` in both constructors. In `Layout::update_render_elements`, after `self.update_insert_hint(output);`:

```rust
        // The focus the swing observes (design 2026-10-09 §5): the one globally focused window,
        // never a tile an interactive move carries.
        let focused = if matches!(self.interactive_move, Some(InteractiveMoveState::Moving(_))) {
            None
        } else {
            self.focus_with_output()
                .map(|(win, output)| (win.id().clone(), output.clone()))
        };
```

and in the monitor loop, between `mon.set_overview_progress(...)` and `mon.update_render_elements(...)`:

```rust
                if let Some((window, focused_output)) = &focused {
                    if mon.output == *focused_output {
                        focus_origin::observe(&mut self.focus_origin, mon, window);
                    }
                }
```

In `trait LayoutElement`, after `fn is_ignoring_opacity_window_rule(&self) -> bool;`:

```rust
    /// Whether the window's background effect drew at its last render (design 2026-10-09 §5:
    /// a swing needs it neutral).
    fn background_effect_visible(&self) -> bool;
```

In `src/layout/tests.rs`, add `mod view_swing;` after `mod fullscreen;`, and to `impl LayoutElement for TestWindow`, after `is_ignoring_opacity_window_rule`:

```rust
    fn background_effect_visible(&self) -> bool {
        false
    }
```

In `src/render_helpers/background_effect.rs`, after `render_for_tile`:

```rust
/// Whether the surface's background effect drew at its last render: its rule and any blur
/// region the client set, resolved as `render_for_tile` resolves them.
pub fn surface_effect_visible(surface: &WlSurface) -> bool {
    with_states(surface, |states| {
        SurfaceBackgroundEffect::get(states)
            .0
            .lock()
            .unwrap()
            .is_visible()
    })
}
```

In `src/window/mapped.rs`, in `impl LayoutElement for Mapped`, after `is_ignoring_opacity_window_rule`:

```rust
    fn background_effect_visible(&self) -> bool {
        crate::render_helpers::background_effect::surface_effect_visible(self.toplevel().wl_surface())
    }
```

In `src/layout/monitor.rs`, before `pub fn workspace_under(`:

```rust
    /// The centre of `window`'s tile in output-local logical coordinates, as it renders now:
    /// the tile's animated position and size in its workspace, then the workspace's render
    /// position and the overview zoom, as `window_under` inverts. `None` if its workspace is
    /// culled.
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

    /// The tile of `window` on any of this monitor's workspaces.
    pub fn tile_mut(&mut self, window: &W::Id) -> Option<&mut Tile<W>> {
        self.workspaces
            .iter_mut()
            .flat_map(Workspace::tiles_mut)
            .find(|tile| tile.window().id() == window)
    }
```

In `view/mod.rs`, after `wall_faces_camera`:

```rust
/// Hooks for the frozen renders (`src/tests/view_tilt.rs`); none exists outside tests.
#[cfg(test)]
pub mod hooks {
    use std::cell::Cell;

    use super::ViewPose;

    thread_local! {
        /// Forces this pose on every glass tile in place of its swing.
        pub static POSE: Cell<Option<ViewPose>> = const { Cell::new(None) };
    }
}
```

- [ ] **Step 2: Write the tile tests**

Add at the end of the `tests` module of `src/layout/tile.rs`:

```rust
    fn swing_response() -> niri_config::ResolvedResponse {
        niri_config::ResolvedResponse {
            view_tilt: 10.,
            view_tilt_settle: 900.,
            view_distance: 6.,
            ring_beam_speed: 0.,
            ..Default::default()
        }
    }

    /// `beam_tile` with the focus ring off, which a swing needs (design 2026-10-09 §5).
    fn swing_tile(response: niri_config::ResolvedResponse, clock: Clock) -> Tile<TestWindow> {
        swing_tile_with(flat_glass(), response, clock)
    }

    fn swing_tile_with(
        glass: niri_config::ResolvedGlass,
        response: niri_config::ResolvedResponse,
        clock: Clock,
    ) -> Tile<TestWindow> {
        let mut tile = beam_tile(glass, response, clock);
        let mut options = (*tile.options).clone();
        options.layout.focus_ring.off = true;
        tile.update_config(tile.view_size, tile.scale, Rc::new(options));
        tile
    }

    fn pose_of(tile: &Tile<TestWindow>) -> Option<crate::render_helpers::material::view::ViewPose> {
        render_dynamics(tile, 400., 300.).view
    }

    #[test]
    fn the_swing_runs_its_curve_and_settles_to_exact_rest() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = swing_tile(swing_response(), clock.clone());
        tile.update_render_elements(false, true, Some(view), view);
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(pose_of(&tile), None, "the first frame is rest: the curve starts at 0");
        assert!(tile.are_animations_ongoing());
        assert!(!tile.are_transitions_ongoing(), "a swing is not a layout transition");

        clock.set_unadjusted(secs(0.9 * 0.190381));
        tile.update_render_elements(false, true, Some(view), view);
        let peak = pose_of(&tile).unwrap();
        assert!((peak.theta - 10f64.to_radians()).abs() < 1e-6 && (peak.weight - 1.).abs() < 1e-6);
        assert_eq!((peak.dir, peak.kappa), (DVec2::X, 6.));

        clock.set_unadjusted(secs(0.9 * 0.636));
        tile.update_render_elements(false, true, Some(view), view);
        assert!(pose_of(&tile).unwrap().theta < 0., "the overshoot");

        clock.set_unadjusted(secs(0.9));
        tile.update_render_elements(false, true, Some(view), view);
        assert_eq!(pose_of(&tile), None);
        tile.advance_animations();
        assert_eq!(tile.view_swing_start(), None);

        // Settled: no clock, no deadline, and the same fingerprint inputs at every later
        // instant (design §8.2).
        let settled = |tile: &Tile<TestWindow>| {
            let d = render_dynamics(tile, 400., 300.);
            (d.jelly_fingerprint, d.signal_fingerprint, d.glass_signal_fingerprint, d.optics, d.view)
        };
        let first = settled(&tile);
        for t in [1.0, 1.5, 3.0, 10.0] {
            clock.set_unadjusted(secs(t));
            tile.update_render_elements(false, true, Some(view), view);
            tile.advance_animations();
            assert!(!tile.are_animations_ongoing(), "at {t}");
            assert_eq!(tile.tick_deadline(Point::default(), view, secs(t)), None, "at {t}");
            assert_eq!(settled(&tile), first, "fingerprint inputs moved at {t}");
        }
    }

    #[test]
    fn a_refocus_mid_swing_finishes_the_swing_it_has() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = swing_tile(swing_response(), clock.clone());
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        clock.set_unadjusted(secs(0.5));
        tile.start_view_swing(SwingFrom::Origin([0., 1.]));
        assert_eq!(tile.view_swing_start(), Some((Duration::ZERO, [1., 0.])), "no restart");
        clock.set_unadjusted(secs(1.0));
        tile.start_view_swing(SwingFrom::Origin([0., 1.]));
        assert_eq!(tile.view_swing_start(), Some((secs(1.0), [0., 1.])), "a new gain after the end");
    }

    #[test]
    fn focus_loss_lets_the_swing_finish() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let mut tile = swing_tile(swing_response(), Clock::with_time(Duration::ZERO));
        tile.update_render_elements(true, true, Some(view), view);
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        tile.update_render_elements(false, true, Some(view), view);
        assert!(tile.view_swing_start().is_some());
    }

    #[test]
    fn a_material_with_no_ring_still_swings() {
        let response = niri_config::ResolvedResponse {
            focus: niri_config::FocusResponse::None,
            ..swing_response()
        };
        let mut tile = swing_tile(response, Clock::with_time(Duration::ZERO));
        tile.start_view_swing(SwingFrom::Fallback);
        assert!(tile.view_swing_start().is_some());
    }

    #[test]
    fn no_swing_at_zero_tilt_or_on_an_ineligible_tile() {
        let zero = niri_config::ResolvedResponse {
            view_tilt: 0.,
            ..swing_response()
        };
        let mut tile = swing_tile(zero, Clock::with_time(Duration::ZERO));
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(tile.view_swing_start(), None, "view-tilt 0");

        let ring_on = beam_tile(flat_glass(), swing_response(), Clock::with_time(Duration::ZERO));
        let mut ring_on = ring_on;
        ring_on.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(ring_on.view_swing_start(), None, "the focus ring is on");

        let mut bordered = swing_tile(swing_response(), Clock::with_time(Duration::ZERO));
        let mut options = (*bordered.options).clone();
        options.layout.border.off = false;
        bordered.update_config(bordered.view_size, bordered.scale, Rc::new(options));
        bordered.start_view_swing(SwingFrom::Origin([1., 0.]));
        assert_eq!(bordered.view_swing_start(), None, "the border is on");
    }

    #[test]
    fn an_opening_tile_takes_no_swing() {
        let mut tile = swing_tile(swing_response(), Clock::with_time(Duration::ZERO));
        tile.start_open_animation();
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
            let mut clock = Clock::with_time(Duration::ZERO);
            let mut tile = swing_tile(swing_response(), clock.clone());
            tile.start_view_swing(SwingFrom::Origin([1., 0.]));
            assert!(tile.view_swing_start().is_some(), "{label}: starts under full");
            clock.set_unadjusted(secs(0.1));

            let mut options = (*tile.options).clone();
            options.signal.motion = policy;
            options.animations.off = animations_off;
            tile.update_config(tile.view_size, tile.scale, Rc::new(options));
            assert_eq!(tile.view_swing_start(), None, "{label}: cut to rest");
            assert_eq!(pose_of(&tile), None, "{label}");
            tile.start_view_swing(SwingFrom::Origin([1., 0.]));
            assert_eq!(tile.view_swing_start(), None, "{label}: not started");
        }
    }

    #[test]
    fn a_reload_mid_swing_keeps_the_snapshot() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = swing_tile(swing_response(), clock.clone());
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        let mut options = (*tile.options).clone();
        let mut material = options.materials["frost"].clone();
        material.responses[0].1.view_tilt = 0.;
        material.responses[0].1.view_distance = 50.;
        options.materials = Rc::new(HashMap::from([(String::from("frost"), material)]));
        tile.update_config(tile.view_size, tile.scale, Rc::new(options));
        clock.set_unadjusted(secs(0.9 * 0.190381));
        let pose = pose_of(&tile).unwrap();
        assert!((pose.theta - 10f64.to_radians()).abs() < 1e-6 && pose.kappa == 6., "{pose:?}");
    }

    #[test]
    fn a_material_swap_mid_swing_keeps_the_snapshot() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = swing_tile(swing_response(), clock.clone());
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        let before = tile.material.as_ref().unwrap().id().clone();
        // Another definition under the same rule, as an is-focused rule swap gives: a new
        // MaterialState (apply_resolved rebuilds on a name change), here with no tilt at all.
        let mut options = (*tile.options).clone();
        let mut other = options.materials["frost"].clone();
        other.name = String::from("ice");
        other.responses[0].1.view_tilt = 0.;
        options.materials = Rc::new(HashMap::from([(String::from("frost"), other)]));
        tile.update_config(tile.view_size, tile.scale, Rc::new(options));
        assert_ne!(tile.material.as_ref().unwrap().id(), &before, "the state was rebuilt");
        clock.set_unadjusted(secs(0.9 * 0.190381));
        let pose = pose_of(&tile).unwrap();
        assert!((pose.theta - 10f64.to_radians()).abs() < 1e-6, "{pose:?}");
    }

    #[test]
    fn fallback_directions_are_unit_and_differ_between_panes() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut a = swing_tile(swing_response(), clock.clone());
        let mut b = swing_tile(swing_response(), clock);
        a.start_view_swing(SwingFrom::Fallback);
        b.start_view_swing(SwingFrom::Fallback);
        let (_, da) = a.view_swing_start().unwrap();
        let (_, db) = b.view_swing_start().unwrap();
        assert!((da[0].hypot(da[1]) - 1.).abs() < 1e-12);
        assert_ne!(da, db, "each pane seeds its own direction");
        assert_eq!(da, a.fallback_direction_at(Duration::ZERO));
    }

    #[test]
    fn the_pose_hook_overrides_the_swing() {
        use crate::render_helpers::material::view::{hooks, ViewPose};
        let tile = swing_tile(swing_response(), Clock::with_time(Duration::ZERO));
        let pose = ViewPose::identity(DVec2::X, 6.);
        hooks::POSE.with(|hook| hook.set(Some(pose)));
        assert_eq!(pose_of(&tile), Some(pose));
        hooks::POSE.with(|hook| hook.set(None));
        assert_eq!(pose_of(&tile), None);
    }
```

Add `use glam::DVec2;` and `use crate::render_helpers::material::view::SwingFrom;` to the tests module's imports.

- [ ] **Step 3: Port the layout tests**

Run `git show d51825f9:src/layout/tests/view_swing.rs > src/layout/tests/view_swing.rs`, then:

1. Replace the module doc with `//! The Layout starts the rigid tilt's swing from the focus origin
//! (docs/specs/2026-10-09-transient-rigid-tilt-design.md §5).`
2. In `swing_options`, replace the response with `view_tilt: 10., view_tilt_settle: 900., view_distance: 6., ring_beam_speed: 0., ..Default::default()`, and the `Options` literal with:
   ```rust
   let mut layout = niri_config::Layout::default();
   layout.focus_ring.off = true;
   Options {
       layout,
       materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
       ..Default::default()
   }
   ```
3. Where a test refocuses a window whose swing started less than 900 ms of clock earlier and expects a new start, insert `advance(&mut layout, 1000);` before the refocus: a refocus mid-swing now finishes the swing it has (§5). Leave every expectation about origins, directions and fallbacks as it is.
4. Append:
   ```rust
   #[test]
   fn a_refocus_mid_swing_does_not_restart() {
       let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
       layout.update_render_elements(None);
       advance(&mut layout, 16);
       Op::FocusColumnLeft.apply(&mut layout);
       layout.update_render_elements(None);
       assert_eq!(started_now(&layout), vec![1]);
       advance(&mut layout, 100);
       Op::FocusColumnRight.apply(&mut layout);
       layout.update_render_elements(None);
       let before = swings(&layout);
       advance(&mut layout, 100);
       Op::FocusColumnLeft.apply(&mut layout);
       layout.update_render_elements(None);
       let one = |s: &[(usize, Duration, [f64; 2])]| s.iter().find(|(id, ..)| *id == 1).cloned();
       assert_eq!(one(&swings(&layout)), one(&before), "window 1 keeps the swing it has");
   }

   #[test]
   fn an_opening_pane_takes_no_swing_for_that_gain() {
       let mut layout = swing_layout([Op::AddOutput(1), glass_window(1)]);
       layout.update_render_elements(None);
       glass_window(2).apply(&mut layout);
       layout.start_open_animation_for_window(&2);
       layout.update_render_elements(None);
       assert!(!started_now(&layout).contains(&2), "no swing while it opens");
       advance(&mut layout, 2000);
       layout.update_render_elements(None);
       assert!(swings(&layout).iter().all(|(id, ..)| *id != 2), "and none later for that gain");
   }
   ```

- [ ] **Step 4: Run them to see them fail**

Run: `just test-one -p niri view_swing`, then `just test-one -p niri tile::tests::a_`
Expected: FAIL to compile: `no method named start_view_swing`, `no field view on MaterialDynamics`.

- [ ] **Step 5: Implement the tile state**

In `src/layout/tile.rs` add the imports `use glam::DVec2;` and `use crate::render_helpers::material::view::{self, SwingFrom, ViewPose};`. Add to `struct Tile`, after `focus_beam`: `view_swing: Option<ViewSwing>,` and `view_swing: None,` after `focus_beam: None,` in `Tile::new`. Add to `struct MaterialDynamics`:

```rust
    /// This frame's pose of the rigid tilt; `None` at rest (design 2026-10-09 §6 "Commit").
    view: Option<ViewPose>,
```

After `impl FocusBeam { ... }`:

```rust
/// The rigid tilt's swing on focus gain (design 2026-10-09 §3, §5). Everything is snapshotted
/// at the start, so a reload or a material swap mid-run does not move the pane; the `Layout`
/// starts it, never the tile's own focus flag.
#[derive(Debug)]
struct ViewSwing {
    /// Start instant on the unadjusted clock.
    started: Duration,
    /// Peak angle, degrees.
    peak: f64,
    settle: Duration,
    /// κ.
    kappa: f64,
    /// Unit direction in output-local logical coordinates.
    dir: DVec2,
}

impl ViewSwing {
    fn elapsed(&self, clock: &Clock) -> Duration {
        clock.now_unadjusted().saturating_sub(self.started)
    }

    fn is_done(&self, clock: &Clock) -> bool {
        clock.should_complete_instantly() || self.elapsed(clock) >= self.settle
    }

    /// The pose now; `None` once done, and on the frames where the curve is exactly zero.
    fn pose(&self, clock: &Clock) -> Option<ViewPose> {
        if self.is_done(clock) {
            return None;
        }
        ViewPose::at(self.peak, self.settle, self.elapsed(clock), self.dir, self.kappa)
    }
}

/// A swing's direction with no usable origin: the pane's own seed keeps two panes apart, and
/// the start instant keeps one pane's successive fallbacks from repeating.
fn fallback_direction(material: &MaterialState, started: Duration) -> [f64; 2] {
    view::seeded_direction(material.jelly_seed()[0].to_bits() ^ started.as_millis() as u32)
}
```

Replace `beam_allowed` and add `motion_allowed` before it:

```rust
    /// Whether the policy allows motion at all: `motion full`, animations on.
    fn motion_allowed(&self) -> bool {
        self.options.signal.motion == niri_config::SignalMotionPolicy::Full
            && !self.options.animations.off
    }

    /// Whether the policy and response allow the ring's beam to move at all: `focus
    /// ring-light` and `motion_allowed`. This is the cut rule's test (design §1); it says
    /// nothing about the speed, which is snapshotted per beam.
    fn beam_allowed(&self, response: &ResolvedResponse) -> bool {
        response.focus == niri_config::FocusResponse::RingLight && self.motion_allowed()
    }
```

After `cut_beam_if_forbidden`:

```rust
    /// Starts the swing for a focus gain the `Layout` observed (design 2026-10-09 §5). Nothing
    /// happens without a material, at `view-tilt 0`, when motion is not allowed, on an
    /// ineligible tile, or while a swing runs: a pane refocused mid-swing finishes the swing it
    /// has.
    pub fn start_view_swing(&mut self, from: SwingFrom) {
        if self
            .view_swing
            .as_ref()
            .is_some_and(|swing| !swing.is_done(&self.clock))
        {
            return;
        }
        let Some(material) = &self.material else {
            return;
        };
        let response = material.material().response(None);
        if response.view_tilt <= 0. || !self.motion_allowed() || !self.view_swing_eligible() {
            return;
        }
        let started = self.clock.now_unadjusted();
        let dir = match from {
            SwingFrom::Origin(dir) => dir,
            SwingFrom::Fallback => fallback_direction(material, started),
        };
        self.view_swing = Some(ViewSwing {
            started,
            peak: response.view_tilt,
            settle: Duration::from_secs_f64(response.view_tilt_settle / 1000.),
            kappa: response.view_distance,
            dir: DVec2::from(dir),
        });
    }

    /// What §5 asks of the tile itself: not fullscreen or opening, border and focus ring off,
    /// and a background effect that does not draw.
    fn view_swing_eligible(&self) -> bool {
        !self.sizing_mode.is_fullscreen()
            && self.fullscreen_progress() == 0.
            && self.open_animation.is_none()
            && self.border.is_off()
            && self.focus_ring.is_off()
            && !self.window.background_effect_visible()
    }

    /// Cut rule for the swing: it ends at rest the moment motion is no longer allowed. A reload
    /// that only changes `view-tilt` or `view-distance` leaves a running swing on its snapshot.
    fn cut_view_swing_if_forbidden(&mut self) {
        if !self.motion_allowed() {
            self.view_swing = None;
        }
    }

    /// This frame's pose: the frozen renders' forced one, else the running swing's.
    pub(crate) fn view_pose(&self) -> Option<ViewPose> {
        #[cfg(test)]
        if let Some(pose) = view::hooks::POSE.with(std::cell::Cell::get) {
            return Some(pose);
        }
        self.view_swing
            .as_ref()
            .and_then(|swing| swing.pose(&self.clock))
    }

    #[cfg(test)]
    pub(crate) fn view_swing_start(&self) -> Option<(Duration, [f64; 2])> {
        self.view_swing
            .as_ref()
            .map(|swing| (swing.started, swing.dir.to_array()))
    }

    /// The direction a fallback swing started at `started` takes on this tile, so layout tests
    /// can tell a fallback from an origin direction.
    #[cfg(test)]
    pub(crate) fn fallback_direction_at(&self, started: Duration) -> [f64; 2] {
        fallback_direction(self.material.as_ref().expect("a glass tile"), started)
    }
```

Call `self.cut_view_swing_if_forbidden();` after each of the two `self.cut_beam_if_forbidden();` calls (in `update_config` and in `refresh_material`). In `material_dynamics`, add `view: self.view_pose(),` to the returned `MaterialDynamics`. In `advance_animations`, after the `focus_beam` block:

```rust
        if self
            .view_swing
            .as_ref()
            .is_some_and(|swing| swing.is_done(&self.clock))
        {
            self.view_swing = None;
        }
```

In `are_animations_ongoing`, replace the trailing beam term with:

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

and add to its doc comment: `The swing lives here for the same reason (design 2026-10-09 §5.1 "Hidden panes").`

- [ ] **Step 6: Run the tests**

Run: `just test-one -p niri view_swing`, `just test-one -p niri tile::tests`, `just test-one -p niri focus_origin`
Expected: all PASS. Then `just test-fast`: the existing `ring_` and `signal` suites must stay green, since the beam's rule is unchanged.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
tasks done material-e44538 "swing state on the tile, focus origin, eligibility, cut; layout tests ported"
tasks check
git add src/layout/ src/window/mapped.rs src/render_helpers/background_effect.rs src/render_helpers/material/view/mod.rs tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(layout): rigid tilt swing on focus gain"
```

### Task 7: Element footprint, view crop, quad mapping, uniforms, fingerprint, culling

**Files:**
- Modify: `src/render_helpers/material/mod.rs` (`MaterialFrame.slab`, `.tex`; `ElementView`; `drawn_area`; `view_quad`; `seed_basis`; `view_uniforms`; `InputFingerprint.view`; `MaterialState::element`; `MaterialRenderElement`; `geometry`; `draw`; tests)
- Modify: `src/render_helpers/material/view/mod.rs` (`wall_spill`, `quad_grow`, the `GROW` and `WALL_SPILL` hooks)
- Modify: `src/layout/tile.rs` (both element sites, `Tile::render`'s offscreen paths, `swing_footprint`, `material_out_of_view`, the swing's visibility in `update_render_elements` and `are_animations_ongoing`, tests)
- Modify: `src/render_helpers/shaders/material/prelude.frag` (the view uniforms), `main.frag` (the quad remap)
- Modify: `src/render_helpers/shaders/mod.rs` (uniform names, a test)

**Interfaces:**
- Consumes: `Camera`, `footprint`, `ViewPose` (Tasks 2–3); `Tile::view_pose`, `MaterialDynamics.view` (Task 6).
- Produces: `Tile::swing_footprint(&self, location: Point<f64, Logical>, pose: ViewPose) -> Option<Rectangle<f64, Logical>>` and the field `Tile::view_swing_visible: bool`, which gates the swing in `are_animations_ongoing`; `material::ElementView { pose: ViewPose, crop: Option<Rectangle<f64, Logical>>, wall_spill: f32, grow: f64 }`; `material::drawn_area(frame: &MaterialFrame, view: Option<&ElementView>, thickness: f64, scale: f64) -> Rectangle<f64, Logical>`; `material::view_quad(area, drawn, scale: f64) -> [f32; 4]`; `material::seed_basis(projection: &[f32; 9], scale: f64) -> [f32; 4]`; `InputFingerprint.view: Option<ElementView>`; `MaterialState::element(.., optics, view: Option<ElementView>, scale, ..)`; GLSL `uniform int mat_view_on; uniform vec4 mat_view; uniform vec2 mat_view_dir; uniform vec4 mat_view_quad; uniform vec4 mat_view_seed;`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module of `src/render_helpers/material/mod.rs` (with `use super::view::{ViewPose, Camera, footprint};` and `use glam::DVec2;`):

```rust
    fn swing_view(pose: ViewPose, crop: Option<Rectangle<f64, Logical>>) -> ElementView {
        ElementView {
            pose,
            crop,
            wall_spill: 1.,
            grow: 0.,
        }
    }

    fn peak_pose() -> ViewPose {
        ViewPose {
            theta: 20f64.to_radians(),
            dir: DVec2::new(0.6, 0.8),
            weight: 1.,
            kappa: 1.5,
        }
    }

    #[test]
    fn the_drawn_area_is_the_resting_area_at_rest_and_the_footprint_in_a_swing() {
        let glass = ResolvedGlass::default();
        let win = Rectangle::new(Point::new(100., 80.), Size::new(400., 300.));
        let tex = Rectangle::new(Point::new(60., 40.), Size::new(480., 380.));
        let frame = material_frame(win, tex, &glass, 1.);
        assert_eq!(drawn_area(&frame, None, glass.thickness, 1.), frame.area);
        let identity = swing_view(ViewPose::identity(DVec2::X, 6.), None);
        assert_eq!(drawn_area(&frame, Some(&identity), glass.thickness, 1.), frame.area);

        let swing = swing_view(peak_pose(), None);
        let drawn = drawn_area(&frame, Some(&swing), glass.thickness, 1.);
        let b = |r: Rectangle<f64, Logical>| {
            (DVec2::new(r.loc.x, r.loc.y), DVec2::new(r.loc.x + r.size.w, r.loc.y + r.size.h))
        };
        let slab = b(frame.slab);
        let camera = Camera::new(&peak_pose(), slab.0, slab.1 - slab.0);
        let (lo, hi) = footprint(&camera, slab, b(frame.tex), glass.thickness);
        assert!(drawn.contains_rect(frame.area), "{drawn:?}");
        assert!(drawn.loc.x <= lo.x && drawn.loc.y <= lo.y);
        assert!(drawn.loc.x + drawn.size.w >= hi.x && drawn.loc.y + drawn.size.h >= hi.y);

        let crop = Rectangle::new(Point::new(0., 0.), Size::new(300., 1000.));
        let cropped = drawn_area(&frame, Some(&swing_view(peak_pose(), Some(crop))), glass.thickness, 1.);
        assert_eq!(cropped.loc.x, drawn.loc.x.max(-1.));
        assert_eq!(cropped.loc.x + cropped.size.w, 301., "the crop grown by one physical px");
        let away = Rectangle::new(Point::new(5000., 0.), Size::new(10., 10.));
        let gone = drawn_area(&frame, Some(&swing_view(peak_pose(), Some(away))), glass.thickness, 1.);
        assert!(gone.is_empty(), "{gone:?}");
    }

    #[test]
    fn the_drawn_quad_maps_the_resting_area_at_fractional_scale() {
        let glass = ResolvedGlass::default();
        for scale in [1., 1.25, 1.5, 2.] {
            let win = Rectangle::new(Point::new(100.4, 80.8), Size::new(400., 300.));
            let frame = material_frame(win, win, &glass, scale);
            let drawn = drawn_area(&frame, Some(&swing_view(peak_pose(), None)), glass.thickness, scale);
            for v in [drawn.loc.x, drawn.loc.y, drawn.size.w, drawn.size.h] {
                assert!(((v * scale) - (v * scale).round()).abs() < 1e-9, "{scale}: {drawn:?}");
            }
            let q = view_quad(frame.area, drawn, scale);
            let s = Scale::from(scale);
            let a = frame.area.to_physical_precise_round(s);
            let d = drawn.to_physical_precise_round(s);
            // The drawn quad's UV at the area's physical corners is the area's UV 0 and 1.
            for (corner, want) in [(a.loc, 0.), (a.loc + a.size.to_point(), 1.)] {
                let u = f64::from(corner.x - d.loc.x) / f64::from(d.size.w);
                let got = f64::from(q[0]) + u * f64::from(q[2]);
                assert!((got - want).abs() < 1e-6, "{scale}: {got} vs {want}");
            }
            assert_eq!(view_quad(frame.area, frame.area, scale), [0., 0., 1., 1.]);
        }
    }

    #[test]
    fn the_seed_basis_follows_the_projection() {
        // Column-major, as smithay's `projection()` returns it: Normal, 90° and flipped.
        let normal = [2. / 1280., 0., 0., 0., 2. / 720., 0., -1., -1., 1.];
        assert_eq!(seed_basis(&normal, 1.5), [1.5, 0., 0., 1.5]);
        let turned = [0., -2. / 720., 0., 2. / 1280., 0., 0., -1., 1., 1.];
        assert_eq!(seed_basis(&turned, 1.), [0., 1., -1., 0.]);
        let flipped = [-2. / 1280., 0., 0., 0., -2. / 720., 0., 1., 1., 1.];
        assert_eq!(seed_basis(&flipped, 2.), [-2., 0., 0., -2.]);
    }

    #[test]
    fn the_view_joins_the_fingerprint() {
        let background_id = Id::new();
        let backdrop_id = Id::new();
        let at_rest = fingerprint(1, 1, &background_id, &backdrop_id);
        assert_eq!(at_rest.view, None);
        let swinging = InputFingerprint {
            view: Some(swing_view(peak_pose(), None)),
            ..at_rest.clone()
        };
        assert_ne!(at_rest, swinging);
    }

    #[test]
    fn the_view_uniforms_are_the_identity_at_rest() {
        let rest = view_uniforms(None, [0., 0., 1., 1.], [1., 0., 0., 1.]);
        let expect = [
            Uniform::new("mat_view_on", UniformValue::_1i(0)),
            Uniform::new("mat_view", [0f32, 0., 0., 1.]),
            Uniform::new("mat_view_dir", [1f32, 0.]),
            Uniform::new("mat_view_quad", [0f32, 0., 1., 1.]),
            Uniform::new("mat_view_seed", [1f32, 0., 0., 1.]),
        ];
        assert_eq!(rest, expect);
        let id = swing_view(ViewPose::identity(DVec2::new(0.6, 0.8), 6.), None);
        let camera = Camera::new(&id.pose, DVec2::ZERO, DVec2::splat(100.));
        let on = view_uniforms(Some((&id, &camera)), [0., 0., 1., 1.], [1., 0., 0., 1.]);
        assert_eq!(on[0], Uniform::new("mat_view_on", UniformValue::_1i(1)));
        assert_eq!(on[1], Uniform::new("mat_view", [-0f32, 0., 0., 1.]), "every correction is zero");
    }
```

In the `fingerprint` helper of the same module, add `view: None,` to the literal. Add to the `tests` module of `src/layout/tile.rs`:

```rust
    #[test]
    fn culling_uses_the_footprint_during_a_swing() {
        use crate::render_helpers::material::view::{hooks, ViewPose};
        let glass = niri_config::ResolvedGlass {
            thickness: 31.2,
            ..Default::default()
        };
        let tile = beam_tile(glass, swing_response(), Clock::with_time(Duration::ZERO));
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        // Slide the tile left until its body is culled at rest: it is then within 0.25 px of
        // the view.
        let mut x = 0.;
        while !tile.material_out_of_view(Point::from((x, 100.)), view) {
            x -= 0.25;
        }
        // The positive lobe lifts the right edge toward the camera, and its projection reaches
        // a few percent of the half-width past its resting place.
        hooks::POSE.with(|hook| {
            hook.set(Some(ViewPose {
                theta: 20f64.to_radians(),
                dir: DVec2::X,
                weight: 1.,
                kappa: 1.5,
            }))
        });
        let culled = tile.material_out_of_view(Point::from((x, 100.)), view);
        hooks::POSE.with(|hook| hook.set(None));
        assert!(!culled, "the swing's footprint reaches into the view");
    }
```

and, after it:

```rust
    #[test]
    fn a_swing_whose_wall_alone_is_in_view_keeps_animating() {
        // Plan review round 1: a 100 px pane with bevel 0 and thickness 200 at 20°, κ 6. Its
        // resting right edge sits 10 px left of the view; the lifted wall reaches 29.8 px into
        // it. The swing must keep requesting frames while only the wall is visible.
        let glass = niri_config::ResolvedGlass {
            bevel: 0.,
            offset_x: 0.,
            offset_y: 0.,
            thickness: 200.,
            ..Default::default()
        };
        let response = niri_config::ResolvedResponse {
            view_tilt: 20.,
            view_distance: 6.,
            ..swing_response()
        };
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = swing_tile_with(glass, response, clock.clone());
        let size = tile.tile_size();
        assert_eq!(size.w, 100., "the case's pane");
        let view = Rectangle::new(Point::from((size.w + 10., 0.)), Size::from((1280., 720.)));
        tile.update_render_elements(true, true, Some(view), view);
        tile.start_view_swing(SwingFrom::Origin([1., 0.]));
        clock.set_unadjusted(secs(0.9 * 0.190381));
        tile.update_render_elements(true, true, Some(view), view);
        let pose = tile.view_pose().unwrap();
        let reach = tile.swing_footprint(Point::default(), pose).unwrap();
        let into = reach.loc.x + reach.size.w - view.loc.x;
        assert!(into > 29., "the case: the wall reaches {into} px into the view");
        assert!(!tile.signal_render_visible, "the case: the resting slab is out of view");
        assert!(tile.are_animations_ongoing(), "the swing stopped scheduling frames");
        // At rest the same tile is out of view and schedules nothing.
        clock.set_unadjusted(secs(1.));
        tile.update_render_elements(true, true, Some(view), view);
        tile.advance_animations();
        assert!(!tile.are_animations_ongoing());
    }
```

and to the `tests` module of `src/render_helpers/shaders/mod.rs`:

```rust
    #[test]
    fn the_view_uniforms_are_declared() {
        let source = material_source();
        for (name, glsl, ty) in [
            ("mat_view_on", "uniform int mat_view_on;", UniformType::_1i),
            ("mat_view", "uniform vec4 mat_view;", UniformType::_4f),
            ("mat_view_dir", "uniform vec2 mat_view_dir;", UniformType::_2f),
            ("mat_view_quad", "uniform vec4 mat_view_quad;", UniformType::_4f),
            ("mat_view_seed", "uniform vec4 mat_view_seed;", UniformType::_4f),
        ] {
            assert!(source.contains(glsl), "{glsl}");
            assert!(material_uniform_names().iter().any(|n| n.name == name && n.type_ == ty), "{name}");
        }
        let main = source.split_once("void main()").unwrap().1;
        assert!(main.contains("if (mat_view_on == 1)\n        v = mat_view_quad.xy + niri_v_coords * mat_view_quad.zw;"));
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri drawn_` then `just test-one -p niri the_view_uniforms_are_declared`
Expected: FAIL to compile: `cannot find type ElementView`, `cannot find function drawn_area`.

- [ ] **Step 3: Implement the element side**

In `src/render_helpers/material/view/mod.rs`, extend `hooks` and add the two readers after it:

```rust
#[cfg(test)]
pub mod hooks {
    use std::cell::Cell;

    use super::ViewPose;

    thread_local! {
        /// Forces this pose on every glass tile in place of its swing.
        pub static POSE: Cell<Option<ViewPose>> = const { Cell::new(None) };
        /// Draws the quad this many logical px past the computed geometry (§8.2 "Containment").
        pub static GROW: Cell<f64> = const { Cell::new(0.) };
        /// The wall's beam spill gain (§4.3); the still sheet turns it off (§8.3).
        pub static WALL_SPILL: Cell<f32> = const { Cell::new(1.) };
    }
}

/// The wall's beam spill gain (§4.3): 1, or the still sheet's hook.
pub fn wall_spill() -> f32 {
    #[cfg(test)]
    return hooks::WALL_SPILL.with(std::cell::Cell::get);
    #[cfg(not(test))]
    1.
}

/// How far past the computed geometry the quad is drawn: 0, or the containment test's hook.
pub fn quad_grow() -> f64 {
    #[cfg(test)]
    return hooks::GROW.with(std::cell::Cell::get);
    #[cfg(not(test))]
    0.
}
```

In `src/render_helpers/material/mod.rs`, add `use glam::DVec2;` and `use self::view::{footprint, Camera, ViewPose};`. Add to `MaterialFrame` after `chamfer`:

```rust
    /// The slab silhouette and the window-texture footprint, absolute logical: the rects
    /// `slab_rect` and `geo_rect` express, which the swing's footprint projects (design
    /// 2026-10-09 §6).
    pub slab: Rectangle<f64, Logical>,
    pub tex: Rectangle<f64, Logical>,
```

and `slab, tex: tex_geo,` to the literal in `material_frame` (before `area,`). After `material_frame`:

```rust
/// The rigid tilt's per-frame inputs to the element (design 2026-10-09 §6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementView {
    pub pose: ViewPose,
    /// The render view in the coordinates the tile draws in, when the render has one.
    pub crop: Option<Rectangle<f64, Logical>>,
    /// The wall's beam spill gain (§4.3).
    pub wall_spill: f32,
    /// How far past the computed geometry the quad is drawn, logical px; 0 outside the
    /// containment test.
    pub grow: f64,
}

/// The element's drawn area (design 2026-10-09 §6): `frame.area` at rest; during a swing the
/// footprint merged with it, cropped to the render view grown by one physical px, and rounded
/// out to physical px.
pub fn drawn_area(
    frame: &MaterialFrame,
    view: Option<&ElementView>,
    thickness: f64,
    scale: f64,
) -> Rectangle<f64, Logical> {
    let Some(view) = view else {
        return frame.area;
    };
    let bounds = |r: Rectangle<f64, Logical>| {
        (
            DVec2::new(r.loc.x, r.loc.y),
            DVec2::new(r.loc.x + r.size.w, r.loc.y + r.size.h),
        )
    };
    let slab = bounds(frame.slab);
    let camera = Camera::new(&view.pose, slab.0, slab.1 - slab.0);
    let (lo, hi) = footprint(&camera, slab, bounds(frame.tex), thickness);
    let mut area = Rectangle::from_extremities((lo.x, lo.y), (hi.x, hi.y)).merge(frame.area);
    if let Some(crop) = view.crop {
        let px = 1. / scale;
        let grown = Rectangle::new(
            crop.loc - Point::from((px, px)),
            crop.size + Size::from((2. * px, 2. * px)),
        );
        area = area
            .intersection(grown)
            .unwrap_or_else(|| Rectangle::new(area.loc, Size::default()));
    }
    let g = view.grow;
    let area = Rectangle::new(area.loc - Point::from((g, g)), area.size + Size::from((2. * g, 2. * g)));
    let lo = Point::from(((area.loc.x * scale).floor() / scale, (area.loc.y * scale).floor() / scale));
    let hi = Point::from((
        ((area.loc.x + area.size.w) * scale).ceil() / scale,
        ((area.loc.y + area.size.h) * scale).ceil() / scale,
    ));
    Rectangle::from_extremities(lo, hi)
}

/// `mat_view_quad`: the drawn quad's UV mapped to the resting area's UV, offset then scale,
/// from both rects as drawn (rounded to physical px), so the resting area's UV is what today's
/// path samples. (0, 0, 1, 1) when the two agree.
pub fn view_quad(
    area: Rectangle<f64, Logical>,
    drawn: Rectangle<f64, Logical>,
    scale: f64,
) -> [f32; 4] {
    let scale = Scale::from(scale);
    let a: Rectangle<i32, Physical> = area.to_physical_precise_round(scale);
    let d: Rectangle<i32, Physical> = drawn.to_physical_precise_round(scale);
    let (aw, ah) = (f64::from(a.size.w), f64::from(a.size.h));
    [
        (f64::from(d.loc.x - a.loc.x) / aw) as f32,
        (f64::from(d.loc.y - a.loc.y) / ah) as f32,
        (f64::from(d.size.w) / aw) as f32,
        (f64::from(d.size.h) / ah) as f32,
    ]
}

/// `mat_view_seed`: framebuffer px per element-local logical px, row-major (§4.4). smithay's
/// projection is a signed permutation scaled to NDC, so its signs are the framebuffer's axes,
/// and `gl_FragCoord` grows with NDC on both. `projection` is column-major.
pub fn seed_basis(projection: &[f32; 9], scale: f64) -> [f32; 4] {
    let sign = |v: f32| if v.abs() < 1e-12 { 0. } else { v.signum() };
    let s = scale as f32;
    [
        sign(projection[0]) * s,
        sign(projection[3]) * s,
        sign(projection[1]) * s,
        sign(projection[4]) * s,
    ]
}

/// The view uniforms (design 2026-10-09 §4.1). At rest every value is the identity.
fn view_uniforms(
    view: Option<(&ElementView, &Camera)>,
    quad: [f32; 4],
    seed: [f32; 4],
) -> [Uniform<'static>; 5] {
    let (on, values, dir) = match view {
        Some((view, camera)) => (
            1,
            [
                camera.cos_m1 as f32,
                camera.sin as f32,
                camera.lambda as f32,
                view.wall_spill,
            ],
            [camera.dir.x as f32, camera.dir.y as f32],
        ),
        None => (0, [0., 0., 0., 1.], [1., 0.]),
    };
    [
        Uniform::new("mat_view_on", UniformValue::_1i(on)),
        Uniform::new("mat_view", values),
        Uniform::new("mat_view_dir", dir),
        Uniform::new("mat_view_quad", quad),
        Uniform::new("mat_view_seed", seed),
    ]
}
```

Add to `InputFingerprint`, after `optics`:

```rust
    /// The rigid tilt's inputs this frame; `None` at rest, so a settled pane's fingerprint is
    /// today's (design 2026-10-09 §6 "Commit").
    pub view: Option<ElementView>,
```

In `MaterialState::element`, add the parameter `view: Option<ElementView>,` after `optics`, and in the returned literal `drawn: drawn_area(&frame, view.as_ref(), self.config.material.glass.thickness, scale),` before `frame,` and `view,` after `optics,`. Add to `struct MaterialRenderElement`, after `optics`:

```rust
    /// The rigid tilt's inputs this frame; `None` at rest.
    view: Option<ElementView>,
    /// The area drawn: `frame.area` at rest, the cropped footprint during a swing.
    drawn: Rectangle<f64, Logical>,
```

Change `geometry` to `self.drawn.to_physical_precise_round(scale)`. In `draw`, replace the fallback's `self.frame.geo_rect` argument with:

```rust
                if self.view.is_none() {
                    self.frame.geo_rect
                } else {
                    // The window rect in the drawn quad's UV: the fallback draws it flat.
                    let a = self.frame.area;
                    let g = self.frame.geo_rect;
                    let x = a.loc.x + f64::from(g[0]) * a.size.w;
                    let y = a.loc.y + f64::from(g[1]) * a.size.h;
                    let d = self.drawn;
                    [
                        ((x - d.loc.x) / d.size.w) as f32,
                        ((y - d.loc.y) / d.size.h) as f32,
                        (f64::from(g[2]) * a.size.w / d.size.w) as f32,
                        (f64::from(g[3]) * a.size.h / d.size.h) as f32,
                    ]
                },
```

After `uniforms.extend(self.optics.iter().cloned());` (before the `Rc` conversion), add:

```rust
        let camera = self.view.as_ref().map(|view| {
            let slab = self.frame.slab;
            Camera::new(
                &view.pose,
                DVec2::new(slab.loc.x, slab.loc.y),
                DVec2::new(slab.size.w, slab.size.h),
            )
        });
        let quad = view_quad(self.frame.area, self.drawn, self.scale);
        let seed = seed_basis(frame.projection(), self.scale);
        uniforms.extend(view_uniforms(self.view.as_ref().zip(camera.as_ref()), quad, seed));
```

and pass `self.drawn.size` instead of `self.frame.area.size` to `ShaderRenderElement::new`.

In `src/render_helpers/shaders/mod.rs`, add to `material_uniform_names` after `mat_bevel_profile`:

```rust
        UniformName::new("mat_view_on", UniformType::_1i),
        UniformName::new("mat_view", UniformType::_4f),
        UniformName::new("mat_view_dir", UniformType::_2f),
        UniformName::new("mat_view_quad", UniformType::_4f),
        UniformName::new("mat_view_seed", UniformType::_4f),
```

In `prelude.frag`, after `uniform float mat_bevel_profile;`:

```glsl
// The rigid tilt (spec 2026-10-09-transient-rigid-tilt §4): 1 while a swing runs. mat_view
// carries cos θ − 1, sin θ, λ = w / (κE) and the wall spill gain; mat_view_dir is d;
// mat_view_quad maps the drawn quad's UV to the resting area's UV (offset, scale);
// mat_view_seed is framebuffer px per logical px, row-major. At rest: 0, (0, 0, 0, 1),
// (1, 0), (0, 0, 1, 1).
uniform int mat_view_on;
uniform vec4 mat_view;
uniform vec2 mat_view_dir;
uniform vec4 mat_view_quad;
uniform vec4 mat_view_seed;
```

In `main.frag`, after `vec2 v = niri_v_coords;`:

```glsl
    // During a swing the drawn quad is the footprint; v stays the resting area's UV.
    if (mat_view_on == 1)
        v = mat_view_quad.xy + niri_v_coords * mat_view_quad.zw;
```

- [ ] **Step 4: Wire the tile**

In `src/layout/tile.rs`, import `ElementView` from `crate::render_helpers::material`. In both element sites of `render_inner` (the resize path and the normal path), before `let inputs = InputFingerprint {`:

```rust
let view = dynamics.view.map(|pose| ElementView {
    pose,
    crop: ctx.signal_ticks.as_ref().map(|sink| sink.view),
    wall_spill: view::wall_spill(),
    grow: view::quad_grow(),
});
```

add `view,` to the `InputFingerprint` literal and pass `view` after `dynamics.optics` to `material.element(..)`. In `Tile::render`, in both the open-animation and alpha-animation branches, right after `let mut ctx = ctx.as_gles();`:

```rust
            // The offscreen draws the tile at (0, 0): move the view the crop takes with it
            // (design 2026-10-09 §6 "View crop").
            if let Some(sink) = &mut ctx.signal_ticks {
                sink.view.loc -= location;
            }
```

Add, next to `material_out_of_view`:

```rust
    /// The swing's drawn area for a tile at `location` (design 2026-10-09 §6), from the
    /// window's buffers, as `material_out_of_view` bounds the resting body. `None` without a
    /// material.
    fn swing_footprint(
        &self,
        location: Point<f64, Logical>,
        pose: ViewPose,
    ) -> Option<Rectangle<f64, Logical>> {
        let material = self.material.as_ref()?;
        let bobbed = location + self.bob_offset();
        let bbox = self.window.buf_bbox().to_f64();
        let buffers = Rectangle::new(bobbed + self.window_loc() + bbox.loc, bbox.size);
        let win = Rectangle::new(bobbed + self.window_loc(), self.animated_window_size());
        let glass = &material.material().glass;
        let frame = material_frame(win, buffers, glass, self.scale);
        let view = ElementView {
            pose,
            crop: None,
            wall_spill: 1.,
            grow: 0.,
        };
        Some(drawn_area(&frame, Some(&view), glass.thickness, self.scale))
    }
```

In `material_out_of_view`, after the margin-grown `view` is computed:

```rust
        // A swing draws its footprint, which contains the buffers (design 2026-10-09 §6).
        if let Some(pose) = self.view_pose() {
            return !self
                .swing_footprint(location, pose)
                .is_some_and(|drawn| drawn.overlaps(view));
        }
```

Add the field `view_swing_visible: bool,` to `struct Tile` after `signal_render_visible` (`false` in `Tile::new`), and in `update_render_elements`, right after `self.signal_render_visible = ...;`:

```rust
        // A swing draws its footprint, which can reach the view while the resting slab does
        // not: the lifted wall of a pane just off the edge (design 2026-10-09 §5.1 "Hidden
        // panes"). The resting test still covers the swing's first frame, whose pose is rest.
        self.view_swing_visible = self.signal_render_visible
            || screen_view.is_some_and(|view| {
                self.view_pose()
                    .and_then(|pose| self.swing_footprint(Point::default(), pose))
                    .is_some_and(|drawn| drawn.overlaps(view))
            });
```

In `are_animations_ongoing`, move the swing out of the `signal_render_visible` group of Task 6 into a term of its own:

```rust
            || self.signal_render_visible
                && self
                    .focus_beam
                    .as_ref()
                    .is_some_and(|beam| !beam.is_done(&self.clock))
            || self.view_swing_visible
                && self
                    .view_swing
                    .as_ref()
                    .is_some_and(|swing| !swing.is_done(&self.clock))
```

and in `clear_signal_render_visibility` also set `self.view_swing_visible = false;`.

importing `drawn_area` and `material_frame` from `crate::render_helpers::material` where they are not yet imported.

- [ ] **Step 5: Run the tests**

Run: `just test-one -p niri material::tests`, `just test-one -p niri culling_uses_the_footprint`, `just test-one -p niri a_swing_whose_wall_alone`, `just test-one -p niri the_view_uniforms_are_declared`, `just test-one -p niri material_source`, then the offline validation of Global Constraints.
Expected: all PASS; both programs validate. Then `just test-one -p niri ring_` and `just test-one -p niri glass_edge`: the frozen renders are unchanged, since no tile swings and every uniform is its rest value.

- [ ] **Step 6: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-392fbc "element footprint, view crop with offscreen shift, quad mapping, view uniforms, fingerprint, culling"
tasks check
git add src/render_helpers/ src/layout/tile.rs tasks/
git commit -m "feat(material): rigid tilt footprint, crop and view uniforms"
```

### Task 8: Shader — camera ray, decal, projection into every sample, `Surface.view`, seeds

This task makes the view branch tilt the window plane and route every sample through the camera; the glass is shaded at the window-plane hit, as on the spike. Task 9 replaces that hit with the traced solid. The identity hook test lands here and must stay green through Task 9.

**Files:**
- Modify: `src/render_helpers/shaders/material/common.frag` (`g_seed_glide`)
- Modify: `src/render_helpers/shaders/material/prelude.frag` (view globals and helpers, `slabFrame`, `tap`, `lightShift`, `Surface.view`)
- Modify: `src/render_helpers/shaders/material/main.frag`
- Modify: `src/render_helpers/shaders/material/noise.frag`, `reflection.frag`, `edge_highlight.frag`
- Modify: `src/render_helpers/shaders/mod.rs` (the `Surface` pin)
- Create: `src/tests/view_tilt.rs`
- Modify: `src/tests/mod.rs` (`mod view_tilt;`), `src/tests/glass_edge.rs` (`renderer_name` becomes `pub(super)`)

**Interfaces:**
- Consumes: the five view uniforms (Task 7); `view::hooks::POSE` (Task 6); `ring_pair::{render_at, set_time, diff}`, `glass_edge::renderer_name`.
- Produces: Rust test helpers `view_tilt::{set_nodes, bevel0, thickness0}`; GLSL `bool g_view_on; vec3 g_ray; vec3 g_view; vec3 g_r; vec3 g_h0; vec2 g_h0_shift; vec3 g_rho; float g_tau0; bool g_plane_ok; bool g_decal_ok; vec2 g_seed_glide;`, `void slabFrame(out float chamfer)`, `vec3 viewUnrotateDelta(vec3)`, `vec3 viewRotateDelta(vec3)`, `vec2 viewScreenShift(vec3 q)`, `void viewBegin(vec2 p)`, `vec2 viewHit(vec2 p)`, `Surface.view`; Rust test harness `view_tilt::{Scene, renders, PoseHook, GAIN, REST}`.

- [ ] **Step 1: Write the identity test**

Create `src/tests/view_tilt.rs`:

```rust
//! Frozen-clock renders of the rigid tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md
//! §8.2), through the real material shader in the headless backend, as ring_pair.rs renders.
//! Set `VIEW_TILT_DUMP=<dir>` to write each render as `<name>.png`. Every fixture gets the next
//! jelly seed; nextest runs each test in its own process, so dumps compare across commits.
//! Run through `just test-one`, never `cargo test`.

use std::time::Duration;

use glam::DVec2;
use niri_config::Config;
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::glass_edge::renderer_name;
use super::ring_pair::{diff, render_at, set_time};
use super::*;
use crate::render_helpers::material::view::{hooks, ViewPose};

pub(super) const OUT_W: u16 = 1280;
pub(super) const OUT_H: u16 = 720;
/// The instant of the focus gain, and the swing's and the beam's start.
pub(super) const GAIN: Duration = Duration::from_secs(1);
/// Long after any run.
pub(super) const REST: Duration = Duration::from_secs(4);
const BACKDROP: &str = "#8c735a";
pub(super) const CLEAR: u32 = 0x0000_0000;
pub(super) const TRANSLUCENT: u32 = 0x990a_0c0e;
pub(super) const OPAQUE: u32 = 0xff0a_0c0e;

/// The stock light glass with every optic §8.2 names on: reflection, the edge highlight, film
/// and glass grain, aberration.
pub(super) const STOCK: &str = r#"
                ior 1.5
                thickness 20
                attenuation-color "#dfe8ff"
                attenuation-distance 60
                bevel 12
                offset-x 6
                offset-y 6
                bevel-profile 2
                chromatic-aberration 0.3
                reflection 0.6
                edge-highlight 0.5
                noise 0.15 type="fine" site="film"
                noise 0.1 type="white" site="glass""#;
/// The accepted dark look's values (ring_look.rs), with grain at scale 8 so value noise runs.
pub(super) const LIVE: &str = r#"
                ior 1.28
                light-ior 4.5
                thickness 31.2
                attenuation-color "#0D1D1E"
                attenuation-distance 11
                chromatic-aberration 0.36
                roughness 0.24
                bevel 12
                bevel-profile 2
                offset-x -6
                offset-y -5
                noise 0.2 type="fine" site="film" scale=8"#;
/// The beam mid-run 150 ms after the gain.
pub(super) const BEAM: &str = r#"focus "ring-light"
                ring-beam-speed 600"#;

/// One case: the glass lines, the response lines, extra window-rule lines, the pane's content
/// and size.
#[derive(Clone)]
pub(super) struct Scene {
    pub glass: String,
    pub response: String,
    pub rules: String,
    pub content: u32,
    pub size: (u16, u16),
}

impl Scene {
    pub fn new(glass: &str, response: &str) -> Self {
        Self {
            glass: glass.to_owned(),
            response: response.to_owned(),
            rules: String::new(),
            content: TRANSLUCENT,
            size: (640, 480),
        }
    }
}

fn config(scene: &Scene) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 40
            center-focused-column "never"
            background-color "{BACKDROP}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "tilt" {{
            glass {{
                {glass}
                backdrop-blur false
            }}
            response "default" {{
                accent "none"
                {response}
            }}
        }}
        window-rule {{
            material "tilt"
            background-effect {{ blur false; noise 0; saturation 1; }}
            {rules}
        }}
        "##,
        glass = scene.glass,
        response = scene.response,
        rules = scene.rules,
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

/// Sets the pose hook for its lifetime.
pub(super) struct PoseHook;

impl PoseHook {
    pub fn set(pose: Option<ViewPose>) -> Self {
        hooks::POSE.with(|hook| hook.set(pose));
        Self
    }
}

impl Drop for PoseHook {
    fn drop(&mut self) {
        hooks::POSE.with(|hook| hook.set(None));
    }
}

/// The fixture for `scene`: the pane, then a second window that holds focus until the gain.
pub(super) fn fixture(scene: &Scene) -> Fixture {
    let mut f = Fixture::with_config(config(scene));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    open(&mut f, id, scene.size, scene.content);
    open(&mut f, id, (400, 300), scene.content);
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

/// The gain: focus moves back to the pane. The swing and the beam start on the first render
/// after it, frozen at its instant (see ring_look.rs).
pub(super) fn gain(f: &mut Fixture) {
    f.niri().layout.focus_left();
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_states();
    f.niri().refresh_layout();
    f.niri().refresh_window_rules();
}

/// Renders `scene` at `frames` after a gain at `GAIN`, under the pose hook `pose`.
pub(super) fn renders(scene: &Scene, pose: Option<ViewPose>, frames: &[Duration]) -> Vec<Vec<u8>> {
    let _hook = PoseHook::set(pose);
    let mut f = fixture(scene);
    gain(&mut f);
    frames.iter().map(|t| render_at(&mut f, *t)).collect()
}

pub(super) fn dump(name: &str, pixels: &[u8], (w, h): (u16, u16)) {
    let Some(dir) = std::env::var_os("VIEW_TILT_DUMP") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("{name}.png"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    crate::utils::write_png_rgba8(file, w.into(), h.into(), pixels).unwrap();
}

/// `glass` with each of `nodes` in place of the line naming the same node, or appended. KDL
/// rejects a repeated single node, so a variant cannot simply append `bevel 0` to a look
/// that already sets `bevel`. Only for single nodes: `noise` may repeat and is not replaced.
pub(super) fn set_nodes(glass: &str, nodes: &[&str]) -> String {
    let key = |line: &str| line.split_whitespace().next().unwrap_or("").to_owned();
    let mut lines: Vec<String> = glass.lines().map(str::to_owned).collect();
    for node in nodes {
        let line = format!("                {node}");
        match lines.iter_mut().find(|l| key(l) == key(node)) {
            Some(existing) => *existing = line,
            None => lines.push(line),
        }
    }
    lines.join("\n")
}

/// No bevel and no offset (the offsets may not exceed the bevel).
pub(super) fn bevel0(glass: &str) -> String {
    set_nodes(glass, &["bevel 0", "offset-x 0", "offset-y 0"])
}

pub(super) fn thickness0(glass: &str) -> String {
    set_nodes(glass, &["thickness 0"])
}

/// The identity cases of §8.2: the beam, aberration, reflection and grain on, and again for
/// bevel 0, thickness 0 and asymmetric corner radii, on translucent, clear and opaque content.
fn identity_scenes() -> Vec<(String, Scene)> {
    let mut out = Vec::new();
    for (look, glass) in [("stock", STOCK), ("live", LIVE)] {
        for (content, px) in [("translucent", TRANSLUCENT), ("clear", CLEAR), ("opaque", OPAQUE)] {
            out.push((format!("{look}-{content}"), Scene { content: px, ..Scene::new(glass, BEAM) }));
        }
        out.push((format!("{look}-bevel0"), Scene::new(&bevel0(glass), BEAM)));
        out.push((format!("{look}-thickness0"), Scene::new(&thickness0(glass), BEAM)));
        out.push((
            format!("{look}-radii"),
            Scene {
                rules: "geometry-corner-radius 88 12 12 12\n            clip-to-geometry true".to_owned(),
                ..Scene::new(glass, BEAM)
            },
        ));
        out.push((
            format!("{look}-radii-small"),
            Scene {
                rules: "geometry-corner-radius 88 12 12 12\n            clip-to-geometry true".to_owned(),
                size: (100, 100),
                ..Scene::new(glass, BEAM)
            },
        ));
    }
    out
}

#[test]
fn every_identity_scene_parses_and_replaces_its_nodes() {
    for (name, scene) in identity_scenes() {
        for node in ["bevel", "thickness", "offset-x", "offset-y"] {
            let count = scene.glass.lines().filter(|l| l.split_whitespace().next() == Some(node)).count();
            assert!(count <= 1, "{name}: {count} `{node}` nodes");
        }
        let _ = config(&scene);
    }
    assert!(bevel0(STOCK).contains("bevel 0") && !bevel0(STOCK).contains("bevel 12"));
}

#[test]
fn the_view_branch_at_the_identity_is_byte_identical_to_rest() {
    eprintln!("renderer: {}", renderer_name(&mut fixture(&Scene::new(STOCK, BEAM))));
    let frames = [GAIN, GAIN + Duration::from_millis(150), REST];
    let identity = ViewPose::identity(DVec2::new(0.6, 0.8), 6.);
    let mut failures = Vec::new();
    for (name, scene) in identity_scenes() {
        let rest = renders(&scene, None, &frames);
        let forced = renders(&scene, Some(identity), &frames);
        for (i, (a, b)) in rest.iter().zip(&forced).enumerate() {
            dump(&format!("identity/{name}-{i}-rest"), a, (OUT_W, OUT_H));
            dump(&format!("identity/{name}-{i}-forced"), b, (OUT_W, OUT_H));
            let (px, max) = diff(a, b);
            if px != 0 {
                failures.push(format!("{name} frame {i}: {px} px differ, max {max}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
```

In `src/tests/mod.rs`, add `mod view_tilt;` after `mod transactions;`. In `src/tests/glass_edge.rs`, make `fn renderer_name` `pub(super) fn renderer_name`. Check how `glass_edge.rs` and `ring_look.rs` are declared there and match (`mod glass_edge;`): `view_tilt` uses `super::glass_edge::renderer_name`, so `glass_edge` must be a sibling module, which it is.

- [ ] **Step 2: Run it to see what it shows before the shader changes**

Run: `just test-one -p niri view_tilt::every_identity_scene_parses`, then `just test-one -p niri view_tilt::the_view_branch_at_the_identity`
Expected: both PASS already: Task 7's branch only remaps `v` by the identity quad. Record the printed renderer string in the task's notes (`tasks note material-a0e3fb "renderer: <string>"`). This run is the control for Step 5.

- [ ] **Step 3: Write the shader**

`common.frag`, at the end:

```glsl
// The sub-pixel part of the grain seed's shift under a tilt (spec 2026-10-09 §4.4): the
// material program's main sets it so value-noise grain glides with the glass. The grain pass
// never sets it, so it stays zero there, as it does at rest.
vec2 g_seed_glide = vec2(0.0);
```

`noise.frag`, in `noiseValue`, replace `vec2 p = fragCoord / scale;` with `vec2 p = (fragCoord + g_seed_glide) / scale;`.

`prelude.frag`, after the view uniforms of Task 7:

```glsl
// The view path's state for this fragment (spec 2026-10-09 §4.1). main sets it while
// mat_view_on is 1; at rest it keeps these values and no stage reads it.
bool g_view_on = false;
vec3 g_ray = vec3(0.0, 0.0, -1.0); // slab-local incident direction, unit
vec3 g_view = vec3(0.0, 0.0, 1.0); // toward the viewer: -g_ray
vec3 g_r;                          // the incident direction, unnormalised
vec3 g_h0;                         // the window-plane hit, z = 0
vec2 g_h0_shift;                   // g_h0.xy minus the screen point
vec3 g_rho;                        // g_r / -g_r.z: the step per px of depth
float g_tau0;                      // the ray parameter at g_h0
bool g_plane_ok;                   // the ray points down
bool g_decal_ok;                   // and its plane hit lies inside the near clip

// view/mod.rs holds the same value under the same name.
const float VIEW_NEAR_CLIP = 0.75;
```

Replace `slabSurface`'s opening, from `vec2 slab_min = mat_slab_rect.xy * mat_area_size;` through `chamferOut = chamfer;`, by a call to a new function holding those lines unchanged. The new function, placed before `slabSurface`:

```glsl
// The slab's frame, independent of the fragment: the silhouette and the face that trails the
// jelly, published for the filament and the traced solid. slabSurface starts with it.
void slabFrame(out float chamfer) {
    vec2 slab_min = mat_slab_rect.xy * mat_area_size;
    vec2 slab_size = mat_slab_rect.zw * mat_area_size;
    vec2 half_ext = slab_size * 0.5;
    vec2 center = slab_min + half_ext;

    // <the tiny-slab guard comment, unchanged>
    float max_chamfer = max(min(half_ext.x, half_ext.y) - 1.0, 0.0);
    chamfer = clamp(mat_chamfer, 0.0, max_chamfer);

    vec2 inner_half = (half_ext - vec2(chamfer))
        * (vec2(1.0) + mat_jelly_resize / max(2.0 * half_ext, vec2(1.0)));
    vec2 inner_center = center + mat_jelly_move;

    // <the radius-fitting comment, unchanged>
    vec2 radius_half = min(inner_half, half_ext - vec2(chamfer));
    vec4 inner_r = fitRadii(mat_corner_radius, radius_half);
    vec4 outer_r = inner_r + vec4(chamfer);

    g_center = center;
    g_half = half_ext;
    g_outer_r = outer_r;
    g_face_center = inner_center;
    g_face_half = inner_half;
    g_face_r = inner_r;
}
```

(move the two comments verbatim; `<…>` marks where they go, not new text). `slabSurface` then begins:

```glsl
    float chamfer;
    slabFrame(chamfer);
    chamferOut = chamfer;
    vec2 center = g_center;
    vec2 half_ext = g_half;
    vec2 inner_center = g_face_center;
    vec2 inner_half = g_face_half;
    vec4 inner_r = g_face_r;
    vec4 outer_r = g_outer_r;
```

and continues unchanged from `float d = sdRoundedBox(p - center, half_ext, outer_r);`.

After `slabSurface`, add the view helpers:

```glsl
// (R⁻¹ − I) v and (R − I) v: the turn by ∓θ in the (d, z) plane less the identity, exactly
// zero at θ = 0. view/mod.rs mirrors both.
vec3 viewUnrotateDelta(vec3 v) {
    float a = dot(v.xy, mat_view_dir);
    float da = a * mat_view.x + v.z * mat_view.y;
    float dz = -a * mat_view.y + v.z * mat_view.x;
    return vec3(mat_view_dir * da, dz);
}

vec3 viewRotateDelta(vec3 v) {
    float a = dot(v.xy, mat_view_dir);
    float da = a * mat_view.x - v.z * mat_view.y;
    float dz = a * mat_view.y + v.z * mat_view.x;
    return vec3(mat_view_dir * da, dz);
}

// toScreen(q) − q.xy for a slab-local point (§4.1 step 5): into the world, then onto z = 0
// along its camera ray. Exactly zero at the identity.
vec2 viewScreenShift(vec3 q) {
    vec3 dq = viewRotateDelta(q - vec3(g_center, 0.0));
    vec2 exy = q.xy + dq.xy;
    float zeta = (q.z + dq.z) * mat_view.z;
    return dq.xy + (exy - g_center) * (zeta / (1.0 - zeta));
}

// The camera ray through screen point p, slab-local, and its window-plane hit (§4.1 steps
// 1–3). slabFrame must have run.
void viewBegin(vec2 p) {
    vec2 rel = p - g_center;
    vec3 oShift = viewUnrotateDelta(vec3(rel, 0.0));
    vec3 dir = vec3(rel * mat_view.z, -1.0);
    g_r = dir + viewUnrotateDelta(dir);
    g_ray = normalize(g_r);
    g_view = -g_ray;
    g_plane_ok = g_r.z < 0.0;
    g_tau0 = g_plane_ok ? -oShift.z / g_r.z : 0.0;
    g_h0_shift = oShift.xy + g_r.xy * g_tau0;
    g_h0 = vec3(p + g_h0_shift, 0.0);
    g_rho = g_plane_ok ? g_r / -g_r.z : vec3(0.0, 0.0, -1.0);
    g_decal_ok = g_plane_ok
        && viewRotateDelta(g_h0 - vec3(g_center, 0.0)).z * mat_view.z <= VIEW_NEAR_CLIP;
}

// The shading point's shift from the screen point p. Here the window-plane hit; Task 9 traces
// the solid.
vec2 viewHit(vec2 p) {
    return g_h0_shift;
}
```

Change `tap`:

```glsl
vec3 tap(vec2 v, vec3 n, float ior, float h) {
    vec3 refr = refract(vec3(0.0, 0.0, -1.0), liftTapNormal(n), 1.0 / ior);
    if (g_view_on)
        refr = refract(g_ray, liftTapNormal(n), 1.0 / ior);
    vec2 vv = v + (refr.xy * rayPath(refr, h)) / mat_area_size;
    // Under a tilt the exit point on the back plane goes through toScreen (§4.2).
    if (g_view_on)
        vv += viewScreenShift(vec3(v * mat_area_size + refr.xy * rayPath(refr, h), -mat_thickness))
            / mat_area_size;
    return srgbToLinear(sampleBackground(vv));
}
```

Change `lightShift`:

```glsl
vec2 lightShift(vec3 n, float ior, float depth) {
    if (g_view_on)
        return refract(g_ray, n, 1.0 / ior).xy * depth;
    return refract(vec3(0.0, 0.0, -1.0), n, 1.0 / ior).xy * depth;
}
```

Add to `struct Surface`, after `outerDist`: `vec3 view;          // toward the viewer: +z at rest`.

`reflection.frag`, replace the `vec3 scene = ...` line with:

```glsl
    vec2 at = s.v + dir * reach / mat_area_size;
    if (g_view_on)
        at += viewScreenShift(vec3(s.p + dir * reach, -mat_thickness)) / mat_area_size;
    vec3 scene = srgbToLinear(sampleBackground(at));
```

`edge_highlight.frag`, after `vec3 h = normalize(l + vec3(0.0, 0.0, 1.0));`:

```glsl
    if (g_view_on)
        h = normalize(l + s.view);
```

`main.frag`: after the Task 7 remap and `vec2 p = v * mat_area_size;`, before the window sample:

```glsl
    // The rigid tilt (spec 2026-10-09 §4.1): the slab-local ray and its window-plane hit. The
    // decal is sampled there, and the glass is shaded at the hit; every shift is exactly zero
    // at the identity, and the seed stays on the pixel lattice (§4.4).
    vec2 winV = v;
    bool hasDecal = true;
    vec2 seed = gl_FragCoord.xy;
    if (mat_view_on == 1) {
        g_view_on = true;
        float frameChamfer;
        slabFrame(frameChamfer);
        viewBegin(p);
        winV = v + g_h0_shift / mat_area_size;
        hasDecal = g_decal_ok;
    }
```

change the window sample to `if (hasDecal && inRect(winV, mat_geo_rect)) { vec2 wv = (winV - mat_geo_rect.xy) / mat_geo_rect.zw; ... }`, and before `slabSurface(p, ...)`:

```glsl
    if (g_view_on) {
        vec2 hitShift = viewHit(p);
        p = p + hitShift;
        v = v + hitShift / mat_area_size;
        vec2 fb = vec2(dot(mat_view_seed.xy, hitShift), dot(mat_view_seed.zw, hitShift));
        vec2 whole = floor(fb + 0.5);
        seed = gl_FragCoord.xy + whole;
        g_seed_glide = fb - whole;
    }
```

and after `slabSurface(...)`: `if (g_view_on && !g_plane_ok) coverage = 0.0;`. In the rest of `main`:
- `hash12(gl_FragCoord.xy + fi * 17.0)` → `hash12(seed + fi * 17.0)`, `hash12(gl_FragCoord.xy + fi * 71.0 + 3.7)` → `hash12(seed + fi * 71.0 + 3.7)`;
- `saturation_behind(sampled, gl_FragCoord.xy)`, `noise_behind(sampled, gl_FragCoord.xy)`, `noise_post(glassColor, gl_FragCoord.xy)` take `seed`;
- after `float surfaceCosine = clamp(surfaceNormal.z, 0.0, 1.0);` add `if (g_view_on) surfaceCosine = clamp(dot(g_view, surfaceNormal), 0.0, 1.0);`;
- after `vec3 structuralRay = refract(vec3(0.0, 0.0, -1.0), surfaceNormal, 1.0 / mat_ior);` add `if (g_view_on) structuralRay = refract(g_ray, surfaceNormal, 1.0 / mat_ior);`;
- the `Surface` constructor gains `g_view` as its last argument.

In `src/render_helpers/shaders/mod.rs`, update the pinned constructor line to `Surface surf = Surface(p, v, surfaceNormal, n, surfaceCosine, fresnel, bevelAcross, acrossDir, slabDist, g_view);`.

- [ ] **Step 4: Validate the programs offline**

Run the two `glslangValidator` commands of Global Constraints.
Expected: exit 0 for both.

- [ ] **Step 5: Run the tests**

Run: `just test-one -p niri view_tilt::the_view_branch_at_the_identity`, then `just test-one -p niri material_source`, `just test-one -p niri pipeline_`, `just test-one -p niri bevel::tests`, `just test-one -p niri glass_edge`, `just test-one -p niri ring_`, `just test-one -p niri noise_`.
Expected: all PASS. The identity test must report 0 differing pixels in every frame of every case. A difference confined to the frames where the view branch runs means a correction is not exactly zero at the identity: find it by setting each correction's uniform input to zero in turn, fix it, and rerun. If it persists only through `refract`'s constant folding (the rest path's `refract(vec3(0.0, 0.0, -1.0), ...)` against the view branch's `refract(g_ray, ...)`), record the renderer, the case and the count in a task note and stop: that is the spec's §6 contingency (a second program variant), which needs the owner's go-ahead, not an edit to the test.

- [ ] **Step 6: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-a0e3fb "view branch: camera ray, decal, toScreen in taps and reflection, Surface.view, slab-anchored seeds; identity byte-identical"
tasks check
git add src/render_helpers/shaders/ src/tests/ tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "feat(material): rigid tilt view branch in the shader"
```

### Task 9: Shader — bracketing, outline coverage, wall surface

**Files:**
- Modify: `src/render_helpers/shaders/material/prelude.frag` (the traced solid, `viewTrace`, `viewOutline`, `viewHit`)
- Modify: `src/render_helpers/shaders/material/main.frag` (the solid's frame, coverage, wall overrides, wall spill)
- Modify: `src/render_helpers/material/view/solid.rs` (`sure_miss`, the shader pins, a test)
- Modify: `src/tests/view_tilt.rs` (the zero-rise swing)

**Interfaces:**
- Consumes: the Task 8 globals and helpers; `solid::{Solid, Segment, trace, outline}` (Tasks 4–5).
- Produces: GLSL `float g_rise; float g_chamfer; bool g_wall; bool g_face_hit; bool g_face_near; float g_hit_z; float g_view_coverage;`, `viewSigmaR`, `viewSigma`, `viewSigmaGrad`, `viewPsi`, `viewPsiDz`, `viewPoint`, `viewMinimise`, `viewTrace`, `viewOutline`; Rust `solid::sure_miss(&Solid, &Segment) -> bool`.

- [ ] **Step 1: Write the failing tests**

Add to `solid.rs`, after `outline`:

```rust
/// The sure miss of `viewHit` (design 2026-10-09 §6): σ with the smallest radius is 1-Lipschitz
/// and no larger than any quadrant's σ, and the bevel term is non-negative, so
/// Ψ(q(t)) ≥ σ_min(h₀) − T·|ρ.xy| on the whole segment. Mirrors the test in `viewHit`.
pub fn sure_miss(solid: &Solid, seg: &Segment) -> bool {
    let r = solid.outer_r.iter().copied().fold(f64::INFINITY, f64::min);
    let q = (seg.h0.truncate() - solid.center).abs() - solid.half + DVec2::splat(r);
    let sigma = q.x.max(q.y).min(0.) + q.max(DVec2::ZERO).length() - r;
    sigma - solid.thickness * seg.rho.truncate().length() > 0.
}
```

and to its `tests` module:

```rust
    #[test]
    fn a_sure_miss_misses() {
        let mut rnd = uniforms(30);
        let mut skipped = 0;
        for _ in 0..200 {
            let (solid, camera) = random_case(&mut rnd);
            for _ in 0..100 {
                let s = solid.center + DVec2::new(rnd() - 0.5, rnd() - 0.5) * solid.half * 3.;
                let Some(seg) = Segment::new(&camera, s) else { continue };
                if sure_miss(&solid, &seg) {
                    skipped += 1;
                    let got = trace(&solid, &seg);
                    assert!(!got.hit && got.psi_min > 0., "a sure miss hit at {s}: {got:?}");
                }
            }
        }
        assert!(skipped > 1000, "the bound skipped only {skipped} rays");
    }

    #[test]
    fn the_shader_names_the_same_constants_and_formulas() {
        let frag = include_str!("../../shaders/material/prelude.frag");
        for (name, literal, value) in [
            ("VIEW_GOLDEN", "0.618033988749895", VIEW_GOLDEN),
            ("VIEW_NEAR_CLIP", "0.75", super::super::VIEW_NEAR_CLIP),
        ] {
            assert_eq!(literal.parse::<f64>().unwrap(), value, "{name}");
            let line = format!("const float {name} = {literal};");
            assert!(frag.contains(&line), "prelude.frag lacks `{line}`");
        }
        for (name, value) in [
            ("VIEW_GOLDEN_STEPS", VIEW_GOLDEN_STEPS),
            ("VIEW_BISECT_STEPS", VIEW_BISECT_STEPS),
        ] {
            let line = format!("const int {name} = {value};");
            assert!(frag.contains(&line), "prelude.frag lacks `{line}`");
        }
        for line in [
            "float da = a * mat_view.x + v.z * mat_view.y;",
            "float dz = -a * mat_view.y + v.z * mat_view.x;",
            "float da = a * mat_view.x - v.z * mat_view.y;",
            "float dz = a * mat_view.y + v.z * mat_view.x;",
            "return dq.xy + (exy - g_center) * (zeta / (1.0 - zeta));",
            "vec3 dir = vec3(rel * mat_view.z, -1.0);",
            "g_tau0 = g_plane_ok ? -oShift.z / g_r.z : 0.0;",
            "g_h0_shift = oShift.xy + g_r.xy * g_tau0;",
            "return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;",
            "return sigma + g_chamfer * (1.0 - x);",
            "return g_chamfer * pow(a, 1.0 / k - 1.0) * pow(b, k - 1.0) / g_rise;",
            "return g_h0 + g_rho * (t * mat_thickness);",
            "if (fb <= psi) {",
            "if (fa <= psi) {",
            "if (psi < g_psi_min) {",
            "g_wall = g_trace_t > rim && viewPsi(viewPoint(rim), entrySel) > 0.0;",
            "float tau = g_tau0 + g_min_t * mat_thickness / -g_r.z;",
            "float scale = 1.0 + tau * mat_view.z;",
            "vec3 jx = scale * (ux - g_r * (dot(n, ux) / dot(n, g_r)));",
            "return g_psi_min + g_psi_min * (length(g0) - len) / len;",
            "if (viewSigmaR(g_h0.xy, rMin) - mat_thickness * length(g_rho.xy) > 0.0)",
            "if (viewSigma(g_h0.xy, g_h0.xy - g_center) + g_chamfer > -4.0 * aa) {",
        ] {
            assert!(frag.contains(line), "prelude.frag lacks `{line}`");
        }
    }
```

Add to `src/tests/view_tilt.rs`:

```rust
/// A real swing through the gain: the moving frames of §8.2's sequence.
pub(super) const SWING_MS: [u64; 6] = [50, 171, 300, 450, 572, 800];

pub(super) fn swing_frames() -> Vec<Duration> {
    let mut frames = vec![GAIN];
    frames.extend(SWING_MS.map(|ms| GAIN + Duration::from_millis(ms)));
    frames.push(REST);
    frames
}

/// Pixels that are pure black: what a NaN reaching the output shows as on llvmpipe, and what
/// no glass over this backdrop is.
fn black(pixels: &[u8]) -> usize {
    pixels.chunks_exact(4).filter(|p| p[0] == 0 && p[1] == 0 && p[2] == 0).count()
}

#[test]
fn a_swing_with_zero_rise_draws_no_nan() {
    for (name, glass) in [
        ("bevel0", bevel0(STOCK)),
        ("thickness0", thickness0(STOCK)),
    ] {
        let scene = Scene::new(&glass, "view-tilt 14\n                view-distance 2.5");
        let frames = renders(&scene, None, &swing_frames());
        let rest = frames.last().unwrap();
        for (i, f) in frames.iter().enumerate() {
            dump(&format!("zero-rise/{name}-{i}"), f, (OUT_W, OUT_H));
            assert_eq!(black(f), 0, "{name} frame {i}: black pixels");
        }
        assert!(diff(&frames[2], rest).0 > 0, "{name}: the peak frame did not move");
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `just test-one -p niri view::solid::tests::the_shader_names`, `just test-one -p niri view::solid::tests::a_sure_miss`, `just test-one -p niri view_tilt::a_swing_with_zero_rise`
Expected: the pin test FAILS (`prelude.frag lacks const float VIEW_GOLDEN`); `a_sure_miss_misses` PASSES (it checks the mirror alone); the render test PASSES or FAILS depending on the planar shading of Task 8, and is the regression check for this task.

- [ ] **Step 3: Write the traced solid in the prelude**

Replace Task 8's `viewHit` in `prelude.frag` with:

```glsl
// The traced slab (spec 2026-10-09 §4.1 step 4, §4.5). view/solid.rs mirrors these line for
// line under the same names.
const float VIEW_GOLDEN = 0.618033988749895;
const int VIEW_GOLDEN_STEPS = 28;
const int VIEW_BISECT_STEPS = 24;

float g_rise;          // min(chamfer, thickness): the bevel's height
float g_chamfer;       // the chamfer drawn
bool g_wall;           // shade this fragment as the wall
bool g_face_hit;       // the window-plane hit is on the flat face: it is shaded there
bool g_face_near;      // and lies near the outline, so its coverage is the outline's
float g_hit_z;         // the shading point's depth
float g_view_coverage; // the outline coverage (§4.5)
float g_psi_min;       // Ψmin over the segment, where, on which plane, in which quadrant
float g_min_t;
vec3 g_min_plane;
vec2 g_min_sel;
float g_trace_t;       // the entry on a hit, the minimiser on a miss
bool g_trace_hit;

// σ for corner radius r, as sdRoundedBox computes it about the slab's centre.
float viewSigmaR(vec2 xy, float r) {
    vec2 q = abs(xy - g_center) - g_half + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;
}

// σ with the radius of the quadrant `sel` (relative to the centre) lies in.
float viewSigma(vec2 xy, vec2 sel) {
    return viewSigmaR(xy, cornerRadius(sel, g_outer_r));
}

vec2 viewSigmaGrad(vec2 xy, vec2 sel) {
    vec2 p = xy - g_center;
    float r = cornerRadius(sel, g_outer_r);
    vec2 q = abs(p) - g_half + vec2(r);
    vec2 g;
    if (q.x > 0.0 && q.y > 0.0)
        g = q / length(q);
    else if (q.x > q.y)
        g = vec2(1.0, 0.0);
    else
        g = vec2(0.0, 1.0);
    return g * vec2(p.x >= 0.0 ? 1.0 : -1.0, p.y >= 0.0 ? 1.0 : -1.0);
}

// Ψ (§4.1): <= 0 inside; X inverts bevelProfile. Without a rise Ψ is σ.
float viewPsi(vec3 q, vec2 sel) {
    float sigma = viewSigma(q.xy, sel);
    if (g_rise <= 0.0)
        return sigma;
    float y = -q.z / g_rise;
    float x = y >= 1.0
        ? 1.0
        : pow(max(1.0 - pow(max(1.0 - y, 0.0), mat_bevel_profile), 0.0), 1.0 / mat_bevel_profile);
    return sigma + g_chamfer * (1.0 - x);
}

// ∂Ψ/∂z = c X'(y) / rise, both bases kept off zero as bevelProfileSlope keeps them.
float viewPsiDz(vec3 q) {
    if (g_rise <= 0.0)
        return 0.0;
    float y = -q.z / g_rise;
    if (y >= 1.0)
        return 0.0;
    float k = mat_bevel_profile;
    float b = clamp(1.0 - y, BEVEL_PROFILE_EPS, 1.0);
    float a = clamp(1.0 - pow(b, k), BEVEL_PROFILE_EPS, 1.0);
    return g_chamfer * pow(a, 1.0 / k - 1.0) * pow(b, k - 1.0) / g_rise;
}

// The point t of the way down the segment: z = -t * thickness.
vec3 viewPoint(float t) {
    return g_h0 + g_rho * (t * mat_thickness);
}

// The minimum of Ψ on [a, b], its ends included; a tie goes to an end, the start first
// (§4.5 "Ties"). `end` is 0 for the interior, 1 at a, 2 at b.
float viewMinimise(float a, float b, vec2 sel, out float psi, out float end) {
    float lo = a;
    float hi = b;
    float c = hi - VIEW_GOLDEN * (hi - lo);
    float d = lo + VIEW_GOLDEN * (hi - lo);
    float fc = viewPsi(viewPoint(c), sel);
    float fd = viewPsi(viewPoint(d), sel);
    for (int i = 0; i < VIEW_GOLDEN_STEPS; ++i) {
        if (fc < fd) {
            hi = d;
            d = c;
            fd = fc;
            c = hi - VIEW_GOLDEN * (hi - lo);
            fc = viewPsi(viewPoint(c), sel);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + VIEW_GOLDEN * (hi - lo);
            fd = viewPsi(viewPoint(d), sel);
        }
    }
    float t = fc < fd ? c : d;
    psi = fc < fd ? fc : fd;
    end = 0.0;
    float fa = viewPsi(viewPoint(a), sel);
    float fb = viewPsi(viewPoint(b), sel);
    if (fb <= psi) {
        t = b;
        psi = fb;
        end = 2.0;
    }
    if (fa <= psi) {
        t = a;
        psi = fa;
        end = 1.0;
    }
    return t;
}

// Brackets the ray's entry (§4.1 "Bracketing"): the segment split where it crosses the
// centre lines, each piece minimised in its own quadrant, the first piece that reaches 0
// bisected. Then top or wall: below the rim and outside at the rim depth is the wall.
void viewTrace() {
    float T = mat_thickness;
    float c1 = 2.0;
    float c2 = 2.0;
    vec3 n1 = vec3(1.0, 0.0, 0.0);
    vec3 n2 = vec3(0.0, 1.0, 0.0);
    if (T > 0.0 && g_rho.x != 0.0) {
        float at = (g_center.x - g_h0.x) / (T * g_rho.x);
        if (at > 0.0 && at < 1.0)
            c1 = at;
    }
    if (T > 0.0 && g_rho.y != 0.0) {
        float at = (g_center.y - g_h0.y) / (T * g_rho.y);
        if (at > 0.0 && at < 1.0)
            c2 = at;
    }
    if (c2 < c1) {
        float s = c1;
        c1 = c2;
        c2 = s;
        vec3 m = n1;
        n1 = n2;
        n2 = m;
    }
    float e1 = min(c1, 1.0);
    float e2 = min(c2, 1.0);
    vec3 m1 = c1 < 1.0 ? n1 : vec3(0.0, 0.0, 1.0);
    vec3 m2 = c2 < 1.0 ? n2 : vec3(0.0, 0.0, 1.0);
    g_psi_min = 1e30;
    g_min_t = 0.0;
    g_min_plane = vec3(0.0, 0.0, 1.0);
    g_min_sel = vec2(0.0);
    g_trace_hit = false;
    g_trace_t = 0.0;
    vec2 entrySel = vec2(0.0);
    for (int i = 0; i < 3; ++i) {
        float a = i == 0 ? 0.0 : i == 1 ? e1 : e2;
        float b = i == 0 ? e1 : i == 1 ? e2 : 1.0;
        vec3 na = i == 0 ? vec3(0.0, 0.0, 1.0) : i == 1 ? m1 : m2;
        vec3 nb = i == 0 ? m1 : i == 1 ? m2 : vec3(0.0, 0.0, 1.0);
        if (b <= a)
            continue;
        vec2 sel = viewPoint(0.5 * (a + b)).xy - g_center;
        float psi;
        float end;
        float t = viewMinimise(a, b, sel, psi, end);
        if (psi < g_psi_min) {
            g_psi_min = psi;
            g_min_t = t;
            g_min_sel = sel;
            g_min_plane = end == 1.0 ? na : end == 2.0 ? nb : vec3(0.0, 0.0, 1.0);
        }
        if (!g_trace_hit && psi <= 0.0) {
            g_trace_hit = true;
            entrySel = sel;
            if (viewPsi(viewPoint(a), sel) <= 0.0) {
                g_trace_t = a;
            } else {
                float lo = a;
                float hi = t;
                for (int j = 0; j < VIEW_BISECT_STEPS; ++j) {
                    float mid = 0.5 * (lo + hi);
                    if (viewPsi(viewPoint(mid), sel) <= 0.0)
                        hi = mid;
                    else
                        lo = mid;
                }
                g_trace_t = hi;
            }
        }
    }
    if (!g_trace_hit) {
        g_trace_t = g_min_t;
        entrySel = g_min_sel;
    }
    float rim = T > 0.0 ? min(g_rise / T, 1.0) : 0.0;
    g_wall = g_trace_t > rim && viewPsi(viewPoint(rim), entrySel) > 0.0;
}

// The outline distance m (§4.5): Ψmin over the norm of its screen gradient at the minimiser,
// held to its plane, as a correction to Ψmin that is exactly zero at the identity.
float viewOutline() {
    vec3 q = viewPoint(g_min_t);
    vec2 g0 = viewSigmaGrad(q.xy, g_min_sel);
    vec3 gq = vec3(g0, viewPsiDz(q));
    vec3 n = g_min_plane;
    float tau = g_tau0 + g_min_t * mat_thickness / -g_r.z;
    float scale = 1.0 + tau * mat_view.z;
    vec3 ux = vec3(1.0, 0.0, 0.0) + viewUnrotateDelta(vec3(1.0, 0.0, 0.0));
    vec3 uy = vec3(0.0, 1.0, 0.0) + viewUnrotateDelta(vec3(0.0, 1.0, 0.0));
    vec3 jx = scale * (ux - g_r * (dot(n, ux) / dot(n, g_r)));
    vec3 jy = scale * (uy - g_r * (dot(n, uy) / dot(n, g_r)));
    vec2 g = vec2(dot(gq, jx), dot(gq, jy));
    float len = length(g);
    if (len == 0.0)
        return g_psi_min;
    return g_psi_min + g_psi_min * (length(g0) - len) / len;
}

// The shading point's shift from the screen point (§4.1 step 4): the window-plane hit on the
// flat face; else, past a sure miss, the traced entry, or on a miss the minimiser. Sets the
// coverage and whether to shade the wall. slabFrame and viewBegin must have run, and main sets
// g_chamfer and g_rise.
vec2 viewHit(vec2 p) {
    g_wall = false;
    g_face_hit = false;
    g_face_near = false;
    g_view_coverage = 0.0;
    g_hit_z = 0.0;
    if (!g_plane_ok)
        return vec2(0.0);
    float aa = 1.0 / niri_scale;
    if (sdRoundedBox(g_h0.xy - g_face_center, g_face_half, g_face_r) < 0.0) {
        g_face_hit = true;
        // Far from the outline the coverage is 1 either way. Within 4 physical px of the
        // face plane's cross-section (sigma + chamfer), the projected outline can be nearer
        // than the slab-local one: with bevel 0 the face reaches the silhouette, and a
        // slab-local coverage errs past the side bound there (plan review round 1). The
        // plane-to-screen map shrinks no distance by more than 2x over the configured
        // ranges, so 4 px slab-local is at least 2 px on screen.
        if (viewSigma(g_h0.xy, g_h0.xy - g_center) + g_chamfer > -4.0 * aa) {
            g_face_near = true;
            viewTrace();
            g_wall = false;
            g_view_coverage = 1.0 - smoothstep(-aa, 0.0, viewOutline());
        }
        return g_h0_shift;
    }
    // A sure miss (design 2026-10-09 §6): coverage exactly 0, no bracketing.
    float rMin = min(min(g_outer_r.x, g_outer_r.y), min(g_outer_r.z, g_outer_r.w));
    if (viewSigmaR(g_h0.xy, rMin) - mat_thickness * length(g_rho.xy) > 0.0)
        return g_h0_shift;
    viewTrace();
    g_view_coverage = 1.0 - smoothstep(-aa, 0.0, viewOutline());
    g_hit_z = -g_trace_t * mat_thickness;
    return g_h0_shift + g_rho.xy * (g_trace_t * mat_thickness);
}
```

- [ ] **Step 4: Use it in main**

In `main.frag`, in the `mat_view_on == 1` block of Task 8, after `slabFrame(frameChamfer);`:

```glsl
        g_chamfer = frameChamfer;
        g_rise = min(frameChamfer, mat_thickness);
```

Remove Task 8's `if (g_view_on && !g_plane_ok) coverage = 0.0;` and put after `slabSurface(...)`:

```glsl
    // The outline coverage and the wall (§4.3, §4.5). A face hit far from the outline keeps
    // the coverage slabSurface gave it, which is 1 there.
    if (g_view_on && (!g_face_hit || g_face_near))
        coverage = g_view_coverage;
    if (g_view_on && !g_face_hit) {
        if (g_wall) {
            vec2 gOut = softOuterGrad(p - g_center, g_half, g_outer_r);
            if (gOut == vec2(0.0))
                gOut = sdRoundedBoxGrad(p - g_center, g_half, g_outer_r);
            surfaceNormal = vec3(gOut, 0.0);
            surfaceHeight = g_hit_z + mat_thickness;
            bevelAcross = 1.0;
            acrossDir = gOut;
            slabDist = 0.0;
        }
    }
```

In the beam block, after the `spill = ... * (1.0 - bevelAcross);` statement's closing brace:

```glsl
                // Guided light leaves the slab at the wall at the face-edge strength (§4.3).
                if (g_wall)
                    spill = mat_sig_focus.x * BEAM_BASE * ringGlow * BEAM_SPILL * moving * mat_view.w;
```

The face shortcut's coverage near the outline is checked against the mirror by Task 11's `the_face_coverage_matches_the_mirror_at_bevel_0`, which needs Task 11's coverage probe.

- [ ] **Step 5: Validate and run**

Run the two `glslangValidator` commands, then `just test-one -p niri view::`, `just test-one -p niri view_tilt`, `just test-one -p niri material_source`, `just test-one -p niri ring_`, `just test-one -p niri glass_edge`.
Expected: all PASS, the identity test of Task 8 included (the bracketing at the identity returns the pixel itself and `m = σ`).

- [ ] **Step 6: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-564316 "traced solid in the shader: bracketing, outline coverage, wall surface and spill"
tasks check
git add src/render_helpers/ src/tests/view_tilt.rs tasks/
git commit -m "feat(material): rigid tilt traced slab in the shader"
```

### Task 10: Frozen renders — lifecycle, geometry, containment, view crop

**Files:**
- Modify: `src/tests/client.rs` (the spike baseline's test-client helpers: subsurfaces, popups, shm pixel buffers, from `69431712`)
- Modify: `src/tests/view_tilt.rs` (`Scene` gains `backdrop`, `subsurface`, `output` and `second`; render helpers; tests)
- Modify: `src/layout/tile.rs` (`#[cfg(test)] Tile::material_id`)

**Interfaces:**
- Consumes: the harness of Tasks 8–9; `view::hooks::{POSE, GROW}`; `material::{drawn_area, material_frame, ElementView}`; `view::{Camera, footprint, VIEW_NEAR_CLIP, ViewPose}`.
- Produces: `view_tilt::{render_output_at, pane_geometry, pane_rect, slab_frame}` for Task 11; `#[cfg(test)] Tile::material_id(&self) -> Option<Id>`.

- [ ] **Step 1: Extend the harness**

Port the test-client helpers the spike's fixture commit added (`State::create_subsurface`, `create_popup`, `ack_and_attach_popup`, `Window::attach_new_shm_pixels`, `LayerSurface::attach_new_shm_pixels`, the free `attach_shm_pixels`, and the `wl_subcompositor` binding with its dispatch impls):

```bash
git diff materials-26.04...69431712 -- src/tests/client.rs | git apply -3
```

Expected: applies cleanly or with a three-way merge; resolve any conflict in favour of keeping both sides' additions. `69431712` changed only test code ("fixture only, renderer as on main").

In `src/layout/tile.rs`, next to `view_swing_start`:

```rust
    /// The material element's id, which its render element carries.
    #[cfg(test)]
    pub(crate) fn material_id(&self) -> Option<smithay::backend::renderer::element::Id> {
        self.material.as_ref().map(|material| material.id().clone())
    }
```

In `src/tests/view_tilt.rs`, add to `Scene`:

```rust
    /// The workspace background behind the pane.
    pub backdrop: &'static str,
    /// An opaque red subsurface of the pane at (x, y), w × h, relative to its top-left: the
    /// texture then reaches past the slab (§6, §8.2 "Containment").
    pub subsurface: Option<[i32; 4]>,
    /// The output's size.
    pub output: (u16, u16),
    /// Whether a second window opens to hold focus until the gain. A pose-hook case without a
    /// gain can leave it out, so nothing but backdrop lies beside the pane.
    pub second: bool,
```

with `backdrop: BACKDROP, subsurface: None, output: (OUT_W, OUT_H), second: true,` in `Scene::new`; use `scene.backdrop` for `background-color` in `config`, and `f.add_output(1, scene.output)` in `fixture`. Change `open` to take the subsurface:

```rust
pub(super) const RED: u32 = 0xffff_0000;

fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16), content: u32, subsurface: Option<[i32; 4]>) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(content);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    if let Some([x, y, sw, sh]) = subsurface {
        // Sync mode: the subsurface lands with the parent's next commit.
        let client = f.client(id);
        let sub = client.state.create_subsurface(&surface, x, y);
        let (shm, qh) = (client.state.shm.clone().unwrap(), client.qh.clone());
        super::client::attach_shm_pixels(&shm, &qh, &sub, sw, sh, &vec![RED; (sw * sh) as usize]);
        sub.commit();
        f.client(id).window(&surface).commit();
        f.double_roundtrip(id);
    }
    surface
}
```

(the second window passes `None`, and `fixture` opens it only when `scene.second` is true). Add:

```rust
/// Like `render_at`, through the output path: the render creates its signal sink, so the view
/// crop applies (§6).
pub(super) fn render_output_at(f: &mut Fixture, time: Duration) -> Vec<u8> {
    render_with(f, time, RenderTarget::Output)
}

fn render_with(f: &mut Fixture, time: Duration, target: RenderTarget) -> Vec<u8> {
    set_time(f, time);
    f.niri().advance_animations();
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let size = output.current_mode().unwrap().size;
    backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target,
                xray: None,
                signal_ticks: None,
            };
            let elements = niri.render_to_vec(ctx, &output, false);
            render_to_vec(renderer, size, Scale::from(1.), Transform::Normal, Fourcc::Abgr8888, elements.iter().rev())
                .unwrap()
        })
        .unwrap()
}

/// The pane's material element geometry in a render at `time` through `target`.
pub(super) fn pane_geometry(f: &mut Fixture, time: Duration, target: RenderTarget) -> Rectangle<i32, Physical> {
    set_time(f, time);
    f.niri().advance_animations();
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, _, _) = workspace.tiles_with_render_positions().next().unwrap();
    let id = tile.material_id().expect("a glass pane");
    backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target,
                xray: None,
                signal_ticks: None,
            };
            let elements = niri.render_to_vec(ctx, &output, false);
            elements
                .iter()
                .find(|e| e.id() == &id)
                .map(|e| e.geometry(Scale::from(1.)))
                .expect("the pane's material element")
        })
        .unwrap()
}

/// The pane's window rect on the output, logical px.
pub(super) fn pane_rect(f: &mut Fixture) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

/// The pane's material frame as the element computes it, at scale 1, with the texture
/// covering the window and its subsurface.
pub(super) fn slab_frame(f: &mut Fixture, glass: &niri_config::ResolvedGlass, subsurface: Option<[i32; 4]>) -> MaterialFrame {
    let win = pane_rect(f);
    let tex = subsurface.map_or(win, |[x, y, w, h]| {
        win.merge(Rectangle::new(
            win.loc + Point::from((f64::from(x), f64::from(y))),
            Size::from((f64::from(w), f64::from(h))),
        ))
    });
    material_frame(win, tex, glass, 1.)
}

/// The resolved glass of the pane's material.
pub(super) fn pane_glass(f: &mut Fixture) -> niri_config::ResolvedGlass {
    f.niri().config.borrow().materials[0].resolve().glass
}
```

with `use smithay::backend::allocator::Fourcc; use smithay::utils::{Logical, Physical, Point, Rectangle, Scale, Size, Transform}; use crate::render_helpers::{render_to_vec, RenderCtx, RenderTarget}; use crate::render_helpers::material::{drawn_area, material_frame, ElementView, MaterialFrame};`.

- [ ] **Step 2: Write the tests**

```rust
/// The swing at the middle cell of the still sheet: 10° at κ 6, with no focus light, so a
/// frame shows the swing alone.
pub(super) const SWING: &str = "focus \"none\"\n                view-tilt 10\n                view-distance 6";
/// The same response without the swing.
pub(super) const STILL: &str = "focus \"none\"";

#[test]
fn a_swing_settles_to_the_rest_render_and_matches_view_tilt_zero_after() {
    let frames = vec![
        GAIN,
        GAIN + Duration::from_millis(171),
        GAIN + Duration::from_millis(900),
        GAIN + Duration::from_millis(1000),
        REST,
    ];
    let tilted = renders(&Scene::new(LIVE, SWING), None, &frames);
    let still = renders(&Scene::new(LIVE, STILL), None, &frames);
    for (i, f) in tilted.iter().enumerate() {
        dump(&format!("settle/{i}"), f, (OUT_W, OUT_H));
    }
    assert!(diff(&tilted[1], &still[1]).0 > 0, "the peak frame did not move");
    for i in [2, 3, 4] {
        assert_eq!(diff(&tilted[i], &still[i]), (0, 0), "frame {i} after the run differs from view-tilt 0");
    }
    assert_eq!(diff(&tilted[2], &tilted[4]), (0, 0), "the frame at settle is not rest");
}

#[test]
fn after_the_run_nothing_animates_and_the_geometry_returns() {
    let scene = Scene::new(LIVE, SWING);
    let mut f = fixture(&scene);
    gain(&mut f);
    let glass = pane_glass(&mut f);
    let settle = Duration::from_millis(900);
    for ms in [0u64, 50, 171, 300, 450, 572, 800, 900, 1500, 3000, 10_000] {
        let at = GAIN + Duration::from_millis(ms);
        let got = pane_geometry(&mut f, at, RenderTarget::ScreenCapture);
        let frame = slab_frame(&mut f, &glass, None);
        let view = ViewPose::at(10., settle, Duration::from_millis(ms), DVec2::NEG_X, 6.).map(|pose| ElementView {
            pose,
            crop: None,
            wall_spill: 1.,
            grow: 0.,
        });
        let want = drawn_area(&frame, view.as_ref(), glass.thickness, 1.).to_physical_precise_round(1.);
        assert_eq!(got, want, "{ms} ms");
        if ms >= 900 {
            assert_eq!(got, frame.area.to_physical_precise_round(1.), "{ms} ms: back to the resting area");
            let niri = f.niri();
            let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
            let (tile, _, _) = workspace.tiles_with_render_positions().next().unwrap();
            assert!(!tile.are_animations_ongoing(), "{ms} ms");
        }
    }
}

#[test]
fn nothing_draws_outside_the_computed_geometry() {
    // §8.2 "Containment": a subsurface outside the slab, every projected pixel of it drawn; then
    // a subsurface past the near clip, drawn up to the clip and nowhere past it.
    let peak = |deg: f64, kappa: f64| ViewPose { theta: deg.to_radians(), dir: DVec2::X, weight: 1., kappa };
    for (name, scene, pose) in [
        (
            "subsurface",
            Scene { subsurface: Some([60, 340, 120, 80]), size: (400, 300), second: false, ..Scene::new(STOCK, "") },
            peak(10., 6.),
        ),
        (
            "near-clip",
            Scene {
                subsurface: Some([110, 20, 500, 60]),
                size: (100, 100),
                output: (2560, 1440),
                second: false,
                ..Scene::new(STOCK, "")
            },
            peak(20., 1.5),
        ),
    ] {
        let _hook = PoseHook::set(Some(pose));
        let mut f = fixture(&scene);
        let glass = pane_glass(&mut f);
        let exact = render_output_at(&mut f, REST);
        let geometry = pane_geometry(&mut f, REST, RenderTarget::Output);
        hooks::GROW.with(|g| g.set(64.));
        let grown = render_output_at(&mut f, REST);
        hooks::GROW.with(|g| g.set(0.));
        dump(&format!("containment/{name}-exact"), &exact, scene.output);
        dump(&format!("containment/{name}-grown"), &grown, scene.output);
        let (w, h) = (i32::from(scene.output.0), i32::from(scene.output.1));
        let at = |pixels: &[u8], x: i32, y: i32| {
            let i = ((y * w + x) * 4) as usize;
            [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
        };
        assert!(
            geometry.loc.x >= -1 && geometry.loc.y >= -1
                && geometry.loc.x + geometry.size.w <= w + 1
                && geometry.loc.y + geometry.size.h <= h + 1,
            "{name}: geometry {geometry:?} leaves the output"
        );
        let mut outside = 0;
        for y in 0..h {
            for x in 0..w {
                if !geometry.contains((x, y)) && at(&exact, x, y) != at(&grown, x, y) {
                    outside += 1;
                }
            }
        }
        assert_eq!(outside, 0, "{name}: the oversized quad drew outside the geometry");

        let frame = slab_frame(&mut f, &glass, scene.subsurface);
        let slab = frame.slab;
        let camera = Camera::new(&pose, DVec2::new(slab.loc.x, slab.loc.y), DVec2::new(slab.size.w, slab.size.h));
        let win = pane_rect(&mut f);
        let [sx, sy, sw, sh] = scene.subsurface.unwrap().map(f64::from);
        let sub = (DVec2::new(win.loc.x + sx, win.loc.y + sy), DVec2::new(win.loc.x + sx + sw, win.loc.y + sy + sh));
        let red = [255, 0, 0, 255];
        if name == "subsurface" {
            // Every pixel whose centre lies inside the subsurface's projection by 1.5 px is red.
            let corners = [sub.0, DVec2::new(sub.1.x, sub.0.y), sub.1, DVec2::new(sub.0.x, sub.1.y)]
                .map(|c| camera.to_screen(c.extend(0.)));
            let inside = |p: DVec2| {
                (0..4).all(|i| {
                    let (a, b) = (corners[i], corners[(i + 1) % 4]);
                    let edge = (b - a).normalize();
                    edge.perp_dot(p - a) >= 1.5
                })
            };
            let mut checked = 0;
            for y in 0..h {
                for x in 0..w {
                    if inside(DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5)) {
                        assert_eq!(at(&exact, x, y), red, "the subsurface is not drawn at ({x}, {y})");
                        checked += 1;
                    }
                }
            }
            assert!(checked > 1000, "{checked} subsurface pixels checked");
        } else {
            // The decal's projection ends where its depth reaches 3/4 of the camera distance.
            // The row through the pivot projects to one screen row at every magnification.
            let row = camera.pivot.y;
            assert!(row > sub.0.y && row < sub.1.y, "the case must put the pivot's row in the subsurface");
            let mut x_clip = f64::NEG_INFINITY;
            let mut x = sub.0.x;
            while x <= sub.1.x {
                let e = camera.to_world(glam::DVec3::new(x, row, 0.));
                if camera.zeta(e) <= VIEW_NEAR_CLIP {
                    x_clip = x_clip.max(camera.project(e).x);
                }
                x += 0.25;
            }
            let y = row as i32;
            let start = camera.to_screen(DVec2::new(sub.0.x, row).extend(0.)).x.ceil() as i32 + 2;
            for px in start..(x_clip.floor() as i32 - 2) {
                assert_eq!(at(&exact, px, y), red, "not drawn up to the clip at x {px} (clip {x_clip})");
            }
            for px in (x_clip.ceil() as i32 + 2)..w {
                assert_ne!(at(&exact, px, y), red, "drawn past the clip at x {px} (clip {x_clip})");
            }
        }
    }
}

#[test]
fn the_view_crop_follows_the_tile_into_an_offscreen() {
    // §8.2 "View crop": a swing during an alpha animation, well away from the output's edges,
    // matches the same frame rendered directly; the bevel at negative offscreen coordinates is
    // drawn in full.
    let scene = Scene { size: (500, 400), ..Scene::new(STOCK, "") };
    let pose = ViewPose { theta: 10f64.to_radians(), dir: DVec2::new(0.6, 0.8), weight: 1., kappa: 6. };
    let _hook = PoseHook::set(Some(pose));
    let mut direct = fixture(&scene);
    let want = render_output_at(&mut direct, REST);
    let mut through = fixture(&scene);
    {
        let anim = through.niri().config.borrow().animations.window_movement.0;
        let niri = through.niri();
        let ws = niri.layout.workspaces_mut().next().unwrap();
        let tile = ws.tiles_mut().next().unwrap();
        tile.animate_alpha(1., 1., anim);
        tile.hold_alpha_animation_after_done();
    }
    let got = render_output_at(&mut through, REST);
    dump("crop/direct", &want, (OUT_W, OUT_H));
    dump("crop/alpha", &got, (OUT_W, OUT_H));
    let (px, max) = diff(&want, &got);
    assert!(max <= 1, "{px} px differ, by up to {max} levels");
}

#[test]
fn a_tile_partly_off_the_left_edge_is_cropped_there_only() {
    let scene = Scene { size: (1000, 400), ..Scene::new(STOCK, "") };
    let pose = ViewPose { theta: -14f64.to_radians(), dir: DVec2::X, weight: 1., kappa: 2.5 };
    let _hook = PoseHook::set(Some(pose));
    let mut f = fixture(&scene);
    // Focus stays on the second window, so the view scrolls the pane partly off the left.
    let full = pane_geometry(&mut f, REST, RenderTarget::ScreenCapture);
    let cropped = pane_geometry(&mut f, REST, RenderTarget::Output);
    assert!(full.loc.x < -1, "the case must reach past the left edge: {full:?}");
    assert_eq!(cropped.loc.x, -1, "cropped one physical px past the edge");
    assert_eq!(cropped.loc.y, full.loc.y);
    assert_eq!(cropped.loc.y + cropped.size.h, full.loc.y + full.size.h);
    assert_eq!(cropped.loc.x + cropped.size.w, full.loc.x + full.size.w, "and nowhere else");
}
```

- [ ] **Step 3: Run them**

Run: `just test-one -p niri view_tilt`
Expected: all PASS. If `a_tile_partly_off_the_left_edge...` fails its precondition (`full.loc.x < -1`), the layout did not scroll the pane off the left: widen the pane (`size: (1100, 400)`) until it does, and say so in the done note. If `the_view_crop_follows...` reports differences above one level, check the open and alpha branches' view shift in `Tile::render` first: an unshifted view crops the offscreen at the output's rect.

- [ ] **Step 4: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-4b1449 "frozen renders: settle to rest, geometry = footprint, containment incl. near clip, view crop in offscreens"
tasks check
git add src/tests/view_tilt.rs src/layout/tile.rs tasks/
git commit -m "test(material): rigid tilt lifecycle, containment and crop renders"
```

### Task 11: Frozen renders — walls, grazing edges, interaction sequences

**Files:**
- Modify: `src/render_helpers/shaders/material/main.frag` (the two test probes)
- Modify: `src/tests/view_tilt.rs` (`Scene.top`, `mirror`, probes, tests)

**Interfaces:**
- Consumes: the harness of Task 10; `view::solid::{Solid, Segment, trace, outline}`; `view::hooks::WALL_SPILL`.
- Produces: GLSL probes on a negative `mat_view.w` (−1: wall hits magenta, everything else clear; −2: coverage as grey); `view_tilt::mirror(frame, glass, pose) -> (Solid, Camera)`.

- [ ] **Step 1: Add the probes**

In `main.frag`, right after the outline-coverage and wall block of Task 9:

```glsl
    // The frozen renders' probes (src/tests/view_tilt.rs): a spill gain of -1 paints wall hits
    // magenta and leaves the rest clear; -2 writes the coverage as grey. Only the test hook
    // uploads a negative gain.
    if (g_view_on && mat_view.w < 0.0) {
        if (mat_view.w < -1.5)
            gl_FragColor = vec4(coverage);
        else
            gl_FragColor = g_wall && coverage > 0.0 ? vec4(1.0, 0.0, 1.0, 1.0) : vec4(0.0);
        return;
    }
```

Validate both programs offline.

- [ ] **Step 2: Extend the harness**

Add `pub top: String,` and `pub after: String,` to `Scene` (top-level config lines placed after `hotkey-overlay` and after the window rule, so a later rule can override it; both `String::new()` in `Scene::new`), and `{top}` and `{after}` in `config`. Split `fixture` into `fixture_pane(scene: &Scene) -> (Fixture, ClientId, WlSurface)`, the body returning the pane's client and surface as `open` gave them, and `fixture(scene)`, which calls it and keeps the `Fixture`. Add:

```rust
/// The pane's slab as the mirror sees it: a window with square corners, so the face radius is
/// 0 and the outer radius is the chamfer.
pub(super) fn mirror(frame: &MaterialFrame, glass: &niri_config::ResolvedGlass, pose: &ViewPose) -> (Solid, Camera) {
    let slab = frame.slab;
    let half = DVec2::new(slab.size.w, slab.size.h) * 0.5;
    let center = DVec2::new(slab.loc.x, slab.loc.y) + half;
    let chamfer = f64::from(frame.chamfer);
    let solid = Solid {
        center,
        half,
        outer_r: [chamfer; 4],
        chamfer,
        thickness: glass.thickness,
        profile: glass.bevel_profile,
    };
    (solid, Camera::new(pose, center - half, 2. * half))
}

/// Renders the pane under `pose` with the probe `gain` (−1 walls, −2 coverage), clear content
/// over a black backdrop and no second window.
fn probe(look: &str, pose: ViewPose, gain: f32) -> (Vec<u8>, MaterialFrame, niri_config::ResolvedGlass) {
    probe_sized(look, (640, 480), pose, gain)
}

fn probe_sized(look: &str, size: (u16, u16), pose: ViewPose, gain: f32) -> (Vec<u8>, MaterialFrame, niri_config::ResolvedGlass) {
    let scene = Scene {
        content: CLEAR,
        backdrop: "#000000",
        second: false,
        size,
        ..Scene::new(look, "")
    };
    let _hook = PoseHook::set(Some(pose));
    hooks::WALL_SPILL.with(|g| g.set(gain));
    let mut f = fixture(&scene);
    let pixels = render_at(&mut f, REST);
    hooks::WALL_SPILL.with(|g| g.set(1.));
    let glass = pane_glass(&mut f);
    let frame = slab_frame(&mut f, &glass, None);
    (pixels, frame, glass)
}

fn px(pixels: &[u8], x: i32, y: i32) -> [u8; 4] {
    let i = ((y * i32::from(OUT_W) + x) * 4) as usize;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

const MAGENTA: [u8; 4] = [255, 0, 255, 255];
```

with `use crate::render_helpers::material::view::solid::{outline, trace, Segment, Solid};` and `use crate::render_helpers::material::view::Camera;`.

- [ ] **Step 3: Write the tests**

```rust
#[test]
fn walls_show_where_the_mirror_predicts() {
    // §8.2 "Walls": at the peak, at 2° of a 10° swing at κ 6, and in the overshoot, the wall's
    // width along the row through the pivot, on each side edge, against the mirror ±1 px.
    let (peak_deg, kappa) = (10., 6.);
    for (name, shape) in [("peak", 1.), ("small", 0.2), ("overshoot", -0.1638)] {
        let pose = ViewPose { theta: peak_deg.to_radians() * shape, dir: DVec2::X, weight: f64::abs(shape), kappa };
        let (pixels, frame, glass) = probe(LIVE, pose, -1.);
        dump(&format!("walls/{name}"), &pixels, (OUT_W, OUT_H));
        let (solid, camera) = mirror(&frame, &glass, &pose);
        let y = solid.center.y as i32;
        for sign in [1., -1.] {
            let edge = camera.to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.));
            let (lo, hi) = (edge.x as i32 - 40, edge.x as i32 + 40);
            let rendered = (lo..hi).filter(|&x| px(&pixels, x, y) == MAGENTA).count() as f64;
            let mut predicted = 0.;
            let mut s = f64::from(lo);
            while s < f64::from(hi) {
                if let Some(seg) = Segment::new(&camera, DVec2::new(s, f64::from(y) + 0.5)) {
                    let tr = trace(&solid, &seg);
                    if tr.wall && outline(&solid, &camera, &seg, &tr) < 0. {
                        predicted += 1. / 64.;
                    }
                }
                s += 1. / 64.;
            }
            assert!((rendered - predicted).abs() <= 1., "{name} side {sign}: {rendered} px rendered, {predicted} predicted");
            if name == "peak" && sign < 0. {
                assert_eq!(rendered, 0., "the receding edge shows no wall at the peak");
            }
        }
    }
}

#[test]
fn a_swing_below_the_reveal_threshold_shows_no_wall() {
    // 11° at κ 2.5: θp ≤ 1 / (2κ), so no frame shows a wall (§3.2).
    for i in 1..20 {
        let shape = crate::render_helpers::material::view::swing_shape(f64::from(i) / 20.);
        let pose = ViewPose { theta: 11f64.to_radians() * shape, dir: DVec2::X, weight: shape.abs(), kappa: 2.5 };
        let (pixels, _, _) = probe(LIVE, pose, -1.);
        let walls = pixels.chunks_exact(4).filter(|p| *p == MAGENTA).count();
        assert_eq!(walls, 0, "frame {i}");
    }
}

#[test]
fn a_diagonal_swing_shows_the_corner_wall_where_the_mirror_does() {
    let pose = ViewPose { theta: 14f64.to_radians(), dir: DVec2::new(0.6, 0.8), weight: 1., kappa: 6. };
    let (pixels, frame, glass) = probe(LIVE, pose, -1.);
    dump("walls/diagonal", &pixels, (OUT_W, OUT_H));
    let (solid, camera) = mirror(&frame, &glass, &pose);
    let corner = camera.to_screen((solid.center + solid.half).extend(0.));
    let (cx, cy) = (corner.x as i32, corner.y as i32);
    let mut rendered = 0;
    let mut predicted = 0;
    for y in cy - 30..cy + 30 {
        for x in cx - 30..cx + 30 {
            rendered += usize::from(px(&pixels, x, y) == MAGENTA);
            if let Some(seg) = Segment::new(&camera, DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5)) {
                let tr = trace(&solid, &seg);
                predicted += usize::from(tr.wall && outline(&solid, &camera, &seg, &tr) < 0.);
            }
        }
    }
    assert!(predicted > 0, "the case must show a corner wall");
    assert!(rendered.abs_diff(predicted) * 10 <= predicted.max(10), "{rendered} rendered, {predicted} predicted");
}

#[test]
fn the_face_coverage_matches_the_mirror_at_bevel_0() {
    // Plan review round 1: with bevel 0 the flat face reaches the silhouette, so face hits sit
    // on the outline. A 100 px slab at 20°, κ 1.5: at x = 52.5 the slab-local distance is
    // −0.4436 px against the projected −0.5305 px. The rendered coverage must follow the mirror
    // within the side bound (1/16 px of outline distance, times the smoothstep's slope 1.5).
    for glass in [thickness0(&bevel0(STOCK)), bevel0(STOCK)] {
        let pose = ViewPose { theta: 20f64.to_radians(), dir: DVec2::X, weight: 1., kappa: 1.5 };
        let (pixels, frame, glass_r) = probe_sized(&glass, (100, 100), pose, -2.);
        let (solid, camera) = mirror(&frame, &glass_r, &pose);
        let y = solid.center.y as i32;
        let coverage = |m: f64| {
            let x = (m + 1.).clamp(0., 1.);
            1. - x * x * (3. - 2. * x)
        };
        for sign in [1., -1.] {
            let edge = camera.to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.)).x as i32;
            for x in edge - 4..edge + 4 {
                let s = DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
                let Some(seg) = Segment::new(&camera, s) else { continue };
                let want = coverage(outline(&solid, &camera, &seg, &trace(&solid, &seg)));
                let got = f64::from(px(&pixels, x, y)[0]) / 255.;
                assert!(
                    (got - want).abs() <= 1.5 / 16. + 2. / 255.,
                    "side {sign} x {x}: rendered coverage {got}, mirror {want}"
                );
            }
        }
    }
}

#[test]
fn grazing_edges_change_coverage_smoothly() {
    // §8.2 "Grazing edges": across a slow sweep of θ, each outline pixel's coverage moves by no
    // more than 1.5 per px of the outline's motion plus twice the side bound (1/16 px), here
    // along rows through the middle of the straight sides.
    let mut prev: Option<(Vec<u8>, ViewPose)> = None;
    for step in 2..=56 {
        let theta = 0.25 * f64::from(step);
        let pose = ViewPose { theta: theta.to_radians(), dir: DVec2::X, weight: theta / 14., kappa: 6. };
        let (pixels, frame, glass) = probe(LIVE, pose, -2.);
        let (solid, camera) = mirror(&frame, &glass, &pose);
        let m_at = |pose: &ViewPose, x: i32, y: i32| {
            let (solid, camera) = mirror(&frame, &glass, pose);
            Segment::new(&camera, DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5))
                .map(|seg| outline(&solid, &camera, &seg, &trace(&solid, &seg)))
                .unwrap_or(f64::INFINITY)
        };
        let rows = (solid.center.y - 0.25 * solid.half.y) as i32..(solid.center.y + 0.25 * solid.half.y) as i32;
        for y in rows.step_by(8) {
            for sign in [1., -1.] {
                let edge = camera.to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.)).x as i32;
                for x in edge - 20..edge + 20 {
                    let c = f64::from(px(&pixels, x, y)[0]) / 255.;
                    let m = m_at(&pose, x, y);
                    if m < -1. {
                        assert!(c > 0., "zero coverage inside the outline at ({x}, {y}), θ {theta}");
                    }
                    if let Some((before, last)) = &prev {
                        let c0 = f64::from(px(before, x, y)[0]) / 255.;
                        let moved = (m - m_at(last, x, y)).abs();
                        assert!(
                            (c - c0).abs() <= 1.5 * moved + 0.125 + 2. / 255.,
                            "({x}, {y}) θ {theta}: coverage {c0} → {c}, outline moved {moved} px"
                        );
                    }
                }
            }
        }
        prev = Some((pixels, pose));
    }
}

/// The pane's tile, to read its swing.
fn pane_swing(f: &mut Fixture) -> Option<(Duration, [f64; 2])> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, _, _) = workspace.tiles_with_render_positions().next().unwrap();
    tile.view_swing_start()
}

/// An inactive material twice as thick, picked by an is-focused rule after the main one: the
/// focus loss mid-swing is a hard cut of the slab (§5.1).
const INACTIVE: &str = r##"
        material "tilt-inactive" {
            glass {
                ior 1.28
                thickness 58.1
                attenuation-color "#0D1D1E"
                attenuation-distance 11
                bevel 12
                bevel-profile 2
                backdrop-blur false
            }
        }
        window-rule {
            match is-focused=false
            material "tilt-inactive"
        }"##;

#[test]
fn interaction_sequences_finish_at_rest() {
    let ms = |m: u64| GAIN + Duration::from_millis(m);
    let mut black_frames = Vec::new();
    let mut check = |name: &str, frames: &[Vec<u8>]| {
        for (i, f) in frames.iter().enumerate() {
            dump(&format!("sequences/{name}-{i}"), f, (OUT_W, OUT_H));
            if black(f) > 0 {
                black_frames.push(format!("{name} frame {i}"));
            }
        }
    };

    // Focus loss mid-swing: the material swap is the accepted hard cut, then the swing
    // finishes on the unfocused pane (§5.1).
    let scene = Scene { after: INACTIVE.to_owned(), ..Scene::new(LIVE, SWING) };
    let mut f = fixture(&scene);
    gain(&mut f);
    let mut frames = vec![render_at(&mut f, ms(0)), render_at(&mut f, ms(100))];
    f.niri().layout.focus_right();
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    frames.extend([ms(171), ms(300), ms(900), REST].map(|t| render_at(&mut f, t)));
    assert!(diff(&frames[2], &frames[5]).0 > 0, "the swing went on after the loss");
    assert_eq!(diff(&frames[4], &frames[5]), (0, 0), "the swing did not finish at settle");
    check("focus-loss", &frames);

    // A refocus mid-swing keeps the swing it has: the frames match an uninterrupted run.
    let plain = Scene::new(LIVE, SWING);
    let straight = renders(&plain, None, &[ms(0), ms(450), ms(600)]);
    let mut f = fixture(&plain);
    gain(&mut f);
    render_at(&mut f, ms(0));
    render_at(&mut f, ms(100));
    f.niri().layout.focus_right();
    f.niri_state().update_keyboard_focus();
    render_at(&mut f, ms(150));
    gain(&mut f);
    let again = [render_at(&mut f, ms(450)), render_at(&mut f, ms(600))];
    assert_eq!(diff(&again[0], &straight[1]), (0, 0), "a restart moved the 450 ms frame");
    assert_eq!(diff(&again[1], &straight[2]), (0, 0), "a restart moved the 600 ms frame");

    // A resize mid-swing runs through (§5.1).
    let (mut f, id, surface) = fixture_pane(&plain);
    gain(&mut f);
    let mut frames = vec![render_at(&mut f, ms(0)), render_at(&mut f, ms(100))];
    let window = f.client(id).window(&surface);
    window.set_size(560, 420);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    frames.extend([ms(171), ms(600), REST, REST + Duration::from_secs(1)].map(|t| render_at(&mut f, t)));
    assert_eq!(diff(&frames[4], &frames[5]), (0, 0), "not at rest after a resize mid-swing");
    check("resize", &frames);

    // The policy cut: motion "reduced" mid-swing ends it at rest on the next frame.
    let mut f = fixture(&plain);
    gain(&mut f);
    render_at(&mut f, ms(0));
    let peak = render_at(&mut f, ms(100));
    let reduced = Scene { top: r#"signal { motion "reduced"; }"#.to_owned(), ..plain.clone() };
    f.niri_state().reload_config(Ok(config(&reduced)));
    let cut = render_at(&mut f, ms(171));
    let rest = renders(&reduced, None, &[ms(171)]);
    assert!(diff(&peak, &cut).0 > 0);
    assert_eq!(diff(&cut, &rest[0]), (0, 0), "the cut is not rest");

    // Animations off: no swing at all.
    let off = Scene { top: "animations { off; }".to_owned(), ..plain.clone() };
    let still = Scene { top: "animations { off; }".to_owned(), ..Scene::new(LIVE, STILL) };
    let a = renders(&off, None, &[ms(171)]);
    let b = renders(&still, None, &[ms(171)]);
    assert_eq!(diff(&a[0], &b[0]), (0, 0), "a swing ran with animations off");

    // Ineligible panes take no swing (§5): border on, fullscreen, a visible background effect.
    for (name, rules) in [
        ("border", "border { on; }"),
        ("fullscreen", "open-fullscreen true"),
        ("background-effect", "background-effect { blur true; }"),
    ] {
        let scene = Scene { rules: rules.to_owned(), ..plain.clone() };
        let mut f = fixture(&scene);
        gain(&mut f);
        render_at(&mut f, ms(0));
        render_at(&mut f, ms(50));
        assert_eq!(pane_swing(&mut f), None, "{name}: a swing started");
    }

    // Close mid-swing: the unmap snapshot captures the pose, and the close animation starts
    // from it (§5.1).
    let (mut f, id, surface) = fixture_pane(&plain);
    gain(&mut f);
    render_at(&mut f, ms(0));
    let before = render_at(&mut f, ms(171));
    let window = f.client(id).window(&surface);
    window.attach_null();
    window.commit();
    f.double_roundtrip(id);
    let after = render_at(&mut f, ms(171));
    dump("sequences/close-before", &before, (OUT_W, OUT_H));
    dump("sequences/close-after", &after, (OUT_W, OUT_H));
    let (count, max) = diff(&before, &after);
    assert!(max <= 2, "the close animation does not start from the pose: {count} px, up to {max}");

    assert!(black_frames.is_empty(), "black pixels in {black_frames:?}");
}
```

- [ ] **Step 4: Run them**

Run: `just test-one -p niri view_tilt`
Expected: all PASS. A wall width outside ±1 px, or a coverage step past the bound, is a shader–mirror disagreement: compare the shader's lines with the pins of Task 9 first.

- [ ] **Step 5: Commit**

```bash
just test-fast
cargo fmt --all
tasks done material-99c7e8 "frozen renders: walls against the mirror, grazing edges, interaction sequences"
tasks check
git add src/render_helpers/shaders/material/main.frag src/tests/view_tilt.rs tasks/
git commit -m "test(material): rigid tilt walls, grazing edges and sequences"
```

### Task 12: Pre-change baseline identity; docs

**Files:**
- Create: `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md` (`<dd>` is the day of the run)
- Modify: `docs/materials/render-pipeline.md` (§1 step 3, a new §3a, §5 rows)
- Modify: `docs/materials/performance.md` (§4 and §7 rows)

**Interfaces:**
- Consumes: the `glass_edge` (`GLASS_EDGE_DUMP`) and `ring_look` (`RING_LOOK_DUMP`) dump harnesses, which exist at the branch's base.
- Produces: the evidence document's *Rest identity* section, which Tasks 13–15 extend.

- [ ] **Step 1: Dump the rest renders at the base and at the head**

The base worktree follows the global worktree rules (work-link, lock, setup), and goes once the comparison is recorded:

```bash
BASE=$(git merge-base HEAD materials-26.04)
EV=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/baseline-$(date +%Y%m%dT%H%M)
mkdir -p "$EV/base" "$EV/head"
MAIN=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")   # the main checkout
cd "$MAIN"
work-link --ensure .worktrees
git worktree add --detach .worktrees/material-abc08c-base "$BASE"
git worktree lock --reason "on WORK_ROOT storage (host: $(uname -n))" .worktrees/material-abc08c-base
(cd .worktrees/material-abc08c-base && just setup \
  && GLASS_EDGE_DUMP="$EV/base" just test-one -p niri glass_edge \
  && RING_LOOK_DUMP="$EV/base" just test-one -p niri ring_look)
(cd .worktrees/material-abc08c \
  && GLASS_EDGE_DUMP="$EV/head" just test-one -p niri glass_edge \
  && RING_LOOK_DUMP="$EV/head" just test-one -p niri ring_look)
```

The base build is a full one (about 10 minutes). Task 14's cost run needs the base's Tracy build as its kept baseline; build it here, before the worktree goes:

```bash
(cd .worktrees/material-abc08c-base && cargo build --release --features profile-with-tracy \
  && cp target/release/niri "$NIRI_MATERIAL_WORK_ROOT/material-6901e0/niri-tracy-$(git rev-parse --short HEAD)")
```

- [ ] **Step 2: Compare byte for byte**

```bash
cd "$EV" && (cd base && find . -type f | sort) > base.list && (cd head && find . -type f | sort) > head.list
diff base.list head.list && while read -r f; do cmp -s "base/$f" "head/$f" || echo "DIFFERS $f"; done < base.list | tee compare.txt
wc -l < compare.txt
```

Expected: identical file lists and `0` lines in `compare.txt`. Keep `$EV` (its `base/` dumps are Task 14's reference if the contingency there splits the program) and record its path in the task's done note. Any `DIFFERS` line fails the spec's rest identity (§8.2): stop, report the files, and do not continue to Step 3. A difference there with `view-tilt` unset means the rest path changed; the usual cause is an expression moved out of its rest form (Global Constraints), and the fix is in the shader, not in the comparison.

- [ ] **Step 3: Remove the base worktree**

```bash
cd "$MAIN"
git -C .worktrees/material-abc08c-base status --ignored --porcelain   # nothing to keep: the dumps are in $EV
(cd .worktrees/material-abc08c-base && tt-report)
git worktree unlock .worktrees/material-abc08c-base
git worktree remove .worktrees/material-abc08c-base
```

Expected: `git worktree remove` succeeds without `--force`. If it refuses (the base holds a submodule), stop and report rather than forcing.

- [ ] **Step 4: Write the evidence document and the docs**

Create `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md`:

```markdown
# Rigid tilt evidence

Design: [transient rigid tilt](../specs/2026-10-09-transient-rigid-tilt-design.md).
Plan: [implementation plan](../plans/2026-10-09-transient-rigid-tilt.md).
Task: `material-6901e0`.

## Rest identity

The `glass_edge` and `ring_look` frozen renders (beam, aberration, reflection,
grain, opaque and translucent content) were dumped at the branch's base
`<BASE>` and at `<HEAD>` on `<renderer string>`, `<n>` files: byte-identical
(`$EV` = `<run-stamp>`). Within the branch, `view_tilt` checks that the
view branch forced on at θ = 0 and w = 0 renders byte-identically to rest,
and that a frame after a swing equals a never-swung render.
```

with the placeholders filled from the run. In `docs/materials/render-pipeline.md`, add to §1 step 3 after the push order: `During a rigid-tilt swing the material element's geometry is its footprint (section 3a); the other elements keep theirs.` Add after §3's table:

```markdown
## 3a. Under a swing (the rigid tilt)

While a swing runs (`view-tilt`, design `2026-10-09-transient-rigid-tilt`),
`mat_view_on` is 1 and the shader takes one branch before stage 0:

1. The drawn quad is the swing's footprint; `mat_view_quad` maps it back to
   the resting area's UV.
2. The camera ray through the pixel, turned into the slab's frame, meets the
   window plane at `h₀`; the window texture (stage 0) is sampled there, up to
   the near clip.
3. On the flat face the glass is hit at `h₀`. Elsewhere the ray is traced
   through the resting slab as a convex solid (golden-section and bisection
   per quadrant piece). The hit is a top hit, shaded by stages 1–11 at the
   hit point, or a wall hit below the rim, shaded with the wall's normal,
   Fresnel and, while the beam runs, its spill.
4. Every backdrop sample (stage 3's taps, stage 6's reflection) projects
   its exit point back to the screen. The Fresnel cosine, the iridescence
   angle and the highlight's half-vector use the slab-local view direction.
   Grain and tap jitter are seeded from the hit, rounded to whole pixels.
5. Coverage (stage 10) is the outline coverage: the signed screen distance
   to the traced solid's outline.

At rest the branch is one uniform test and every stage reads today's inputs.
```

and to §5's table:

```markdown
| response `view-tilt`, `view-tilt-settle`, `view-distance` | (pending, prism-2a8fea) | 3a |
```

In `docs/materials/performance.md` §4, add a row after `Ring beam`:

```markdown
| Rigid tilt | Refresh rate for `view-tilt-settle` (900 ms default) after a focus gain, while the slab band is in view | 0 once the run ends | [rigid tilt evidence](2026-10-<dd>-rigid-tilt-evidence.md) |
```

and to §7:

```markdown
| `view-tilt` | During a swing only: the footprint quad (a few percent larger than the slab) and the traced solid on the bevel band and wall, for `view-tilt-settle` per focus gain; nothing at rest (cost in the evidence document) | 0 |
```

- [ ] **Step 5: Commit**

```bash
just test-fast
tasks done material-c73cf0 "rest renders byte-identical to the base (<n> files); render-pipeline and performance docs"
tasks check
git add docs/materials/ tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "docs(material): rigid tilt rest identity and pipeline notes"
```

### Task 13: Owner stills

This task ends at a gate: the owner picks the peak angle, the camera distance and the wall spill from the sheet. Nothing after it sets a default.

**Files:**
- Modify: `src/tests/ring_look.rs` (`ACCEPTED` becomes `pub(super)`)
- Modify: `src/tests/view_tilt.rs` (the sheet scene and the ignored render)
- Create: `docs/materials/scripts/view-tilt-sheet.py`
- Modify: `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md` (the *Stills* section)

**Interfaces:**
- Consumes: the harness of Tasks 8–11 (`pane_swing` from Task 11); `ring_look::ACCEPTED`; the test-client helpers ported in Task 10; `view::hooks::{POSE, WALL_SPILL}`.
- Produces: `VIEW_TILT_DUMP/sheet/*.png` and `sheet/geometry.json`; `view-tilt-sheet.py <dump dir> <out dir>` writing `sheet.png`, `crops.png`, `filmstrip.png`.

- [ ] **Step 1: Write the sheet render**

In `src/tests/ring_look.rs`, change `const ACCEPTED: &str` to `pub(super) const ACCEPTED: &str`. In `src/tests/view_tilt.rs`, copy the functions `backdrop`, `hash` and `glyphs`, and the constants `FILL` and `GLYPH`, verbatim from `git show 69431712:src/tests/tilt_spike.rs` (the spike's patterned backdrop and terminal-like content: rows of 1 px opaque strokes over a translucent fill), with `OUT_W`/`OUT_H` in `backdrop` replaced by `SHEET_OUT.0`/`SHEET_OUT.1`. Then add:

```rust
const SHEET_OUT: (u16, u16) = (2560, 1440);
const SHEET_PANE: (u16, u16) = (1400, 900);

/// The accepted dark material pair (ring_look.rs, with its is-active rules) with `extra` in
/// every response, or the lighter look beside it: the active glass with the stock smokes'
/// light attenuation.
fn sheet_material(light: bool, extra: &str) -> String {
    let m = super::ring_look::ACCEPTED.replace("ring-glow 1.2\n", &format!("ring-glow 1.2\n        {extra}\n"));
    if light {
        m.replace(r##"attenuation-color "#0D1D1E""##, r##"attenuation-color "#dfe8ff""##)
            .replace("attenuation-distance 11", "attenuation-distance 60")
    } else {
        m
    }
}

fn sheet_fixture(material: &str) -> Fixture {
    let config = Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 31
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        {material}
        "##
    ))
    .unwrap();
    // ACCEPTED carries its own is-active rules with a neutral background effect.
    let mut f = Fixture::with_config(config);
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, SHEET_OUT);
    // The patterned Background layer, so refraction and the wall's reflection show.
    let bg = f.add_client();
    let layer = f.client(bg).create_layer(None, Layer::Background, "");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Top | Anchor::Left | Anchor::Right | Anchor::Bottom),
        exclusive_zone: Some(-1),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(bg);
    let layer = f.client(bg).layer(&surface);
    layer.attach_new_shm_pixels(i32::from(SHEET_OUT.0), i32::from(SHEET_OUT.1), &backdrop());
    layer.ack_last_and_commit();
    f.double_roundtrip(bg);
    let id = f.add_client();
    for (size, pixels) in [
        (SHEET_PANE, glyphs(SHEET_PANE.0, SHEET_PANE.1)),
        ((400, 300), vec![FILL; 400 * 300]),
    ] {
        let window = f.client(id).create_window();
        let s = window.surface.clone();
        window.commit();
        f.roundtrip(id);
        let window = f.client(id).window(&s);
        window.attach_new_shm_pixels(i32::from(size.0), i32::from(size.1), &pixels);
        window.ack_last_and_commit();
        f.double_roundtrip(id);
    }
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

/// One frame of the sheet: a gain at `GAIN`, rendered `ms` after it, under the hooks given.
fn sheet_frame(light: bool, extra: &str, pose: Option<ViewPose>, spill: f32, ms: u64) -> Vec<u8> {
    let _hook = PoseHook::set(pose);
    hooks::WALL_SPILL.with(|g| g.set(spill));
    let mut f = sheet_fixture(&sheet_material(light, extra));
    gain(&mut f);
    render_at(&mut f, GAIN);
    let pixels = render_at(&mut f, GAIN + Duration::from_millis(ms));
    hooks::WALL_SPILL.with(|g| g.set(1.));
    pixels
}

/// The peak of a 900 ms swing.
const PEAK_MS: u64 = 171;

#[test]
#[ignore = "the owner's still sheet (§8.3): run with VIEW_TILT_DUMP set, then view-tilt-sheet.py"]
fn still_sheet_frames() {
    assert!(std::env::var_os("VIEW_TILT_DUMP").is_some(), "set VIEW_TILT_DUMP");
    let swing = |t: f64, k: f64| format!("view-tilt {t}\n        view-distance {k}");
    for light in [false, true] {
        let look = if light { "light" } else { "dark" };
        for t in [6., 10., 14.] {
            for k in [2.5, 6., 10.] {
                let pixels = sheet_frame(light, &swing(t, k), None, 1., PEAK_MS);
                dump(&format!("sheet/cell-t{t}-k{k}-{look}"), &pixels, SHEET_OUT);
            }
        }
        let middle = swing(10., 6.);
        dump(&format!("sheet/spill-off-{look}"), &sheet_frame(light, &middle, None, 0., PEAK_MS), SHEET_OUT);
        let diagonal = ViewPose { theta: 10f64.to_radians(), dir: DVec2::new(0.6, 0.8), weight: 1., kappa: 6. };
        dump(&format!("sheet/diagonal-{look}"), &sheet_frame(light, &middle, Some(diagonal), 1., PEAK_MS), SHEET_OUT);
        // The spike's control: 4° about the vertical axis with the camera at 2000 px.
        let mut f = sheet_fixture(&sheet_material(light, ""));
        let glass = pane_glass(&mut f);
        let slab = slab_frame(&mut f, &glass, None).slab;
        let control = ViewPose { theta: 4f64.to_radians(), dir: DVec2::X, weight: 1., kappa: 2000. / slab.size.w };
        dump(&format!("sheet/control-{look}"), &sheet_frame(light, "", Some(control), 1., PEAK_MS), SHEET_OUT);
        dump(&format!("sheet/rest-{look}"), &sheet_frame(light, &middle, None, 1., 2000), SHEET_OUT);
    }
    for ms in (0..=900).step_by(50) {
        dump(&format!("sheet/film-{ms:03}"), &sheet_frame(false, &swing(10., 6.), None, 1., ms), SHEET_OUT);
    }
    // The cells' swing direction: the positive lobe lifts the edge along it, which is the
    // advancing edge the crops show.
    let mut f = sheet_fixture(&sheet_material(false, &swing(10., 6.)));
    gain(&mut f);
    render_at(&mut f, GAIN);
    let (_, advancing) = pane_swing(&mut f).expect("the gain starts a swing");
    let pane = pane_rect(&mut f);
    let dir = std::env::var_os("VIEW_TILT_DUMP").unwrap();
    std::fs::write(
        std::path::Path::new(&dir).join("sheet/geometry.json"),
        format!(
            r#"{{"size": [{}, {}], "pane": [{}, {}, {}, {}], "advancing": [{}, {}]}}"#,
            SHEET_OUT.0, SHEET_OUT.1, pane.loc.x, pane.loc.y, pane.size.w, pane.size.h, advancing[0], advancing[1]
        ),
    )
    .unwrap();
}
```

with `use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;`, `use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;` and `use super::client::LayerConfigureProps;`. The gain direction is the pane's leftward focus move, so the cells swing horizontally; the diagonal crop forces its pose through the hook while the beam still starts on the gain.

- [ ] **Step 2: Write the sheet script**

Create `docs/materials/scripts/view-tilt-sheet.py` (mode 0755):

```python
#!/usr/bin/env python3
"""The owner's still sheet for the rigid tilt (design 2026-10-09 §8.3).

Usage: view-tilt-sheet.py <VIEW_TILT_DUMP dir> <out dir>

Reads <dump>/sheet/*.png and geometry.json from `still_sheet_frames` and
writes sheet.png (the 3x3 grid, dark and light in each cell, the control,
the spill row), crops.png (1:1 crops of the advancing edge, of text, and of
a corner on a diagonal swing) and filmstrip.png (one swing at the middle
cell). Needs Pillow and the DejaVu fonts.
"""
import json
import os
import sys

from PIL import Image, ImageDraw, ImageFont

FONTS = '/usr/share/fonts/TTF'
GOLD, TEXT, DIM, BG = '#d4a62a', '#ffffff', '#cccccc', '#1e1e1e'
SCALE, GAP, MARGIN = 0.28, 10, 16
TS, KS = [6, 10, 14], [2.5, 6, 10]


def font(size, bold=False):
    return ImageFont.truetype(os.path.join(FONTS, 'DejaVuSans-Bold.ttf' if bold else 'DejaVuSans.ttf'), size)


def num(v):
    return f'{v:g}'


def main(dump, out):
    d = os.path.join(dump, 'sheet')
    geo = json.load(open(os.path.join(d, 'geometry.json')))
    x, y, w, h = geo['pane']
    pad = 80
    box = (int(x - pad), int(y - pad), int(x + w + pad), int(y + h + pad))

    def cell(name):
        im = Image.open(os.path.join(d, f'{name}.png')).convert('RGB').crop(box)
        return im.resize((int(im.width * SCALE), int(im.height * SCALE)), Image.Resampling.LANCZOS)

    tw, th = cell('rest-dark').size
    pair = 2 * tw + GAP
    head, side = 150, 210
    rows = len(TS) + 1
    W = MARGIN * 2 + side + len(KS) * (pair + GAP)
    H = MARGIN * 2 + head + rows * (th + 46 + GAP) + 60
    sheet = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(sheet)
    draw.text((MARGIN, MARGIN), 'How far, and how much perspective, for the focus tilt?', TEXT, font(30, True))
    draw.text((MARGIN, MARGIN + 44), 'Each cell: the peak of one focus-gain swing (171 ms of 900 ms), the accepted dark look (left) and a lighter look (right).', DIM, font(17))
    draw.text((MARGIN, MARGIN + 70), 'Look for depth you can read at the peak: the rounded rim turning toward you, the wall on the advancing edge, the keystone.', DIM, font(17))
    draw.text((MARGIN, MARGIN + 96), 'Wall: the slab side below the rim. Rim: the rounded bevel. Keystone: the near edge drawn taller than the far one.', DIM, font(17))
    for j, k in enumerate(KS):
        cx = MARGIN + side + j * (pair + GAP)
        draw.text((cx, MARGIN + head - 34), f'camera distance κ = {num(k)} pane widths', TEXT, font(19, True))
    for i, t in enumerate(TS):
        ry = MARGIN + head + i * (th + 46 + GAP)
        draw.text((MARGIN, ry + th // 2 - 12), f'peak angle {t}°', TEXT, font(21, True))
        for j, k in enumerate(KS):
            cx = MARGIN + side + j * (pair + GAP)
            cid = f'{"ABC"[i]}{j + 1}'
            for n, look in enumerate(['dark', 'light']):
                sheet.paste(cell(f'cell-t{num(t)}-k{num(k)}-{look}'), (cx + n * (tw + GAP), ry))
            draw.text((cx, ry + th + 6), f'{cid}   (dark | light)', DIM, font(16))
    ry = MARGIN + head + len(TS) * (th + 46 + GAP)
    draw.text((MARGIN, ry + th // 2 - 24), 'B2, wall light', TEXT, font(21, True))
    draw.text((MARGIN, ry + th // 2 + 4), 'off | on', DIM, font(17))
    for j, (name, label) in enumerate([('spill-off-dark', 'S1  wall spill off'), ('cell-t10-k6-dark', 'S2  wall spill on (as B2)')]):
        cx = MARGIN + side + j * (tw + GAP)
        sheet.paste(cell(name), (cx, ry))
        draw.text((cx, ry + th + 6), label, DIM, font(16))
    cx = MARGIN + side + 2 * (pair + GAP)
    sheet.paste(cell('control-dark'), (cx, ry))
    draw.rectangle((cx - 3, ry - 3, cx + tw + 2, ry + th + 2), outline=GOLD, width=3)
    draw.text((cx, ry + th + 6), 'K  control: the spike, 4° at 2000 px', GOLD, font(16, True))
    draw.text((MARGIN, H - MARGIN - 26), 'Reply with a cell id (A1–C3), the spill (S1 or S2), and anything that reads wrong.', TEXT, font(19, True))
    sheet.save(os.path.join(out, 'sheet.png'))

    # The positive lobe lifts the edge along the swing's direction: the advancing edge.
    if geo['advancing'][0] < 0:
        edge = (x - 100, y + h / 2 - 160, x + 220, y + h / 2 + 160)
        text = (x + 120, y + 60, x + 520, y + 260)
    else:
        edge = (x + w - 220, y + h / 2 - 160, x + w + 100, y + h / 2 + 160)
        text = (x + w - 520, y + 60, x + w - 120, y + 260)
    # The diagonal crop's pose lifts the corner along (0.6, 0.8): bottom right.
    corner = (x + w - 200, y + h - 200, x + w + 100, y + h + 100)
    crops = [
        ('advancing edge, B2 dark', 'cell-t10-k6-dark', edge),
        ('advancing edge, B2 light', 'cell-t10-k6-light', edge),
        ('text near the advancing edge, B2 dark', 'cell-t10-k6-dark', text),
        ('the same text at rest', 'rest-dark', text),
        ('corner, diagonal swing, dark', 'diagonal-dark', corner),
        ('corner, diagonal swing, light', 'diagonal-light', corner),
    ]
    tiles = [(label, Image.open(os.path.join(d, f'{name}.png')).convert('RGB').crop(tuple(int(v) for v in rect))) for label, name, rect in crops]
    rows = list(zip(tiles[::2], tiles[1::2]))
    CW = MARGIN * 2 + max(a.width + GAP + b.width for (_, a), (_, b) in rows)
    CH = MARGIN * 2 + 60 + sum(max(a.height, b.height) + 40 for (_, a), (_, b) in rows)
    page = Image.new('RGB', (CW, CH), BG)
    draw = ImageDraw.Draw(page)
    draw.text((MARGIN, MARGIN), 'Native-resolution crops (1:1) at the peak', TEXT, font(24, True))
    yy = MARGIN + 50
    for (la, a), (lb, b) in rows:
        page.paste(a, (MARGIN, yy))
        page.paste(b, (MARGIN + a.width + GAP, yy))
        draw.text((MARGIN, yy + a.height + 6), la, DIM, font(15))
        draw.text((MARGIN + a.width + GAP, yy + b.height + 6), lb, DIM, font(15))
        yy += max(a.height, b.height) + 40
    page.save(os.path.join(out, 'crops.png'))

    frames = [ms for ms in range(0, 901, 50)]
    fs = [cell(f'film-{ms:03d}') for ms in frames]
    FW = MARGIN * 2 + 5 * (tw + GAP)
    FH = MARGIN * 2 + 50 + ((len(fs) + 4) // 5) * (th + 34)
    film = Image.new('RGB', (FW, FH), BG)
    draw = ImageDraw.Draw(film)
    draw.text((MARGIN, MARGIN), 'One swing at B2 (10°, κ 6), every 50 ms after the focus gain', TEXT, font(24, True))
    for n, (ms, im) in enumerate(zip(frames, fs)):
        fx, fy = MARGIN + (n % 5) * (tw + GAP), MARGIN + 50 + (n // 5) * (th + 34)
        film.paste(im, (fx, fy))
        draw.text((fx, fy + th + 4), f'{ms} ms', DIM, font(15))
    film.save(os.path.join(out, 'filmstrip.png'))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
```

- [ ] **Step 3: Render and assemble**

```bash
EV=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/stills-$(date +%Y%m%dT%H%M)
mkdir -p "$EV"
VIEW_TILT_DUMP="$EV" just test-one -p niri --run-ignored only view_tilt::still_sheet_frames
python3 docs/materials/scripts/view-tilt-sheet.py "$EV" "$EV"
```

Expected: the test passes; `sheet.png`, `crops.png` and `filmstrip.png` exist under `$EV`.

- [ ] **Step 4: Look at the sheets before handing them over**

Open each PNG at the size the owner will see it (Read tool on the file). Check that the title, the row and column labels, every cell id and the legend are legible, that the control carries its gold border and label, that the edge crop shows the lifted edge with its wall (the side `geometry.json`'s `advancing` names), and that both text crops show whole lines, uncut at the canvas edge. Fix the script's offsets or scale and rerun Step 3 until they do.

- [ ] **Step 5: Record and stop for the owner**

Append to the evidence document:

```markdown
## Stills

Sheet, crops and filmstrip: `$EV` = `<run-stamp>` (`sheet.png`, `crops.png`,
`filmstrip.png`), rendered by `view_tilt::still_sheet_frames` on
`<renderer string>`. Cells A1–C3 are θp ∈ {6, 10, 14}° by κ ∈ {2.5, 6, 10};
S1/S2 the wall spill off and on at B2; K the spike's control.

Owner's verdict: pending.
```

Commit, then park for the owner:

```bash
cargo fmt --all
tasks note material-7e3447 "stills: $EV (sheet.png, crops.png, filmstrip.png)"
tasks check
git add src/tests/ docs/materials/ tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "test(material): rigid tilt owner stills"
tasks park material-7e3447 "Owner picks a cell (θp, κ) and the wall spill (S1/S2) from $EV/sheet.png with crops.png and filmstrip.png; then the agent records the verdict in the evidence doc, sets the spill (removing the hook's 'off' path if S2), and starts Task 14" --waiting-on user --reason review
git add tasks/ && git commit -m "chore(tasks): park material-7e3447 for the owner's stills verdict"
```

When the verdict arrives: record it under *Stills* (`Owner's verdict (<date>): <cell>, <spill>; <comments>`), and if the owner picks S1 (spill off), change `view::wall_spill`'s non-test value to `0.` with a comment citing the verdict; then `tasks done material-7e3447 "owner picked <cell>, <spill>"` in that commit. If no cell shows enough depth, record that and stop: §8.3 names the remaining levers (a larger angle, a thicker active slab), and the slab is a material choice outside this plan.

### Task 14: GPU cost A/B on the capture lane

**Files:**
- Create: `docs/materials/scripts/view-tilt-cost.sh`
- Modify: `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md` (the *Cost* section), `docs/materials/performance.md` (the §7 row's figures)

**Interfaces:**
- Consumes: `glass-optic-smoke-lib.sh` (`build_binaries`, `capture_preflight`, `capture_identity`, `write_config`, `start_nested`, `spawn_probe`, `steal_focus`, the wallpaper and Tracy helpers, `finish`); the baseline `niri-tracy` kept by Task 12.
- Produces: `$OUT/metrics.txt` with `<case>-<side><round>_MaterialRenderElement::draw_median_ms`, `<case>_verdict=PASS|FAIL|INCONCLUSIVE`, and the swing lines `swing_frame_median_ms`, `swing_frame_max_ms`, `swing_frames`, `swing_extra_ms_per_gain`.

- [ ] **Step 1: Write the script**

Create `docs/materials/scripts/view-tilt-cost.sh` (mode 0755):

```bash
#!/usr/bin/env bash
# Rigid tilt GPU cost (design 2026-10-09-transient-rigid-tilt §8.5): a same-session A/B of
# this tree's niri-tracy (a) against a kept baseline built from the branch's base (b),
# alternating a, b, a, b so a shift between sessions cannot pass for one between binaries.
# Rest cases, each with the live look and the beam off: `view-tilt 0`, `view-tilt 10` with no
# swing running, and one fine noise layer. Each replaces the wallpaper DAMAGE_STEPS times at
# 1 Hz, so every step redraws resting glass. Bound: a's median material draw within 1 % of
# b's or 0.002 ms, whichever is larger; a/a or b/b repeats further apart than that make the
# case inconclusive. The swing case (a only) repeats focus gains at 10°, κ 6 and records the
# swing frames' draws beside the rest median, and the extra GPU time per swing: no bound.
# VIEW_TILT_COST_AB=<baseline niri-tracy> is required; VIEW_TILT_COST_PILOT=1 runs three
# damage steps per case and two gains.
set -eu
PILOT=${VIEW_TILT_COST_PILOT:-0}
case "$PILOT" in 0|1) ;; *) echo "VIEW_TILT_COST_PILOT must be 0 or 1" >&2; exit 2 ;; esac
AB=${VIEW_TILT_COST_AB:-}
[ -n "$AB" ] && [ -x "$AB" ] || { echo "VIEW_TILT_COST_AB must name an executable" >&2; exit 2; }
source "$(dirname "$0")/glass-optic-smoke-lib.sh"
trap cleanup_cost EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
build_binaries
await_load
capture_preflight headless
cp "$AB" "$OUT/niri-tracy-b"
capture_identity --binary "$OUT/niri-tracy-b" --config "pilot=$PILOT" --config ab=1
reserve_tracy_port
tools_ready
DAMAGE_STEPS=20; GAINS=8
[ "$PILOT" = 0 ] || { DAMAGE_STEPS=3; GAINS=2; }
LIVE=$'ior 1.28\nthickness 31.2\nattenuation-color "#0D1D1E"\nattenuation-distance 11\nchromatic-aberration 0.36\nroughness 0.24\nbevel 12\nbevel-profile 2\noffset-x -6\noffset-y -5'
declare -A GLASS=([tilt0]="$LIVE" [tilt10]="$LIVE" [fine]="$LIVE"$'\nnoise 0.3 type="fine"')
declare -A VIEW=([tilt0]='view-tilt 0' [tilt10]=$'view-tilt 10\n        view-distance 6' [fine]='view-tilt 0')
ORDER=()
declare -A BIN=() BASE=() SIDE=()
for base in tilt0 tilt10 fine; do for round in 1 2; do for side in a b; do
    name=$base-$side$round; ORDER+=("$name"); BASE[$name]=$base; SIDE[$name]=$side
    BIN[$name]=$NIRI_TRACY; [ "$side" = a ] || BIN[$name]=$OUT/niri-tracy-b
done; done; done
write_cost_config() {   # $1 path, $2 case, $3 side
    local view=${VIEW[$2]}
    # The baseline predates the keys, so its configs carry none: b is today's rest path.
    [ "$3" = a ] || view=
    GLASS_EXTRA=${GLASS[$2]} RESPONSE_EXTRA=$view FOCUS_RESPONSE=none \
        TOP_EXTRA='blur { passes 3; offset 3; noise 0; saturation 1; }' write_config "$1.tmp"
    # This script owns and reaps every wallpaper PID.
    sed '/^spawn-at-startup "swaybg" /d' "$1.tmp" > "$1"
    rm "$1.tmp"
}
for name in "${ORDER[@]}"; do
    cfg=$OUT/$name.kdl
    write_cost_config "$cfg" "${BASE[$name]}" "${SIDE[$name]}"
    start_nested "${BIN[$name]}" "$cfg"
    capture_bg "$name"; capture_ready "$name"
    start_wall "$WALL"
    sleep 0.5
    spawn_probe "$NIRI" "$IDLE"
    # The probe's first focus starts a swing under view-tilt 10; let it end before measuring.
    sleep 1.5
    start=$(now_ns)
    reload_marker "$cfg"
    for i in $(seq 1 "$DAMAGE_STEPS"); do
        magick -size 1280x720 xc:"rgb($((i * 10)),100,120)" "$OUT/$name-wall-$i.png"
        stop_walls; wait_for_wall_removal; start_wall "$OUT/$name-wall-$i.png"
        sleep_until "$((start + i * 1000000000))" 500000000
        reload_marker "$cfg"
    done
    capture_wait
    [ "$(wall_count)" -eq 1 ] || fail "wallpaper layer disappeared during $name"
    stop_walls; stop_nested
    csvexport --gpu "$OUT/$name.tracy" "$OUT/$name.gpu.csv"
    export_cpu "$name"
    python3 - "$OUT" "$name" "$DAMAGE_STEPS" <<'PYREPORT' >> "$OUT/metrics.txt"
import csv, statistics, sys, pathlib
root, name, steps = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
def zones(path, t, d):
    with path.open() as f:
        return [(next(iter(r.values())), float(r[t]), float(r[d])) for r in csv.DictReader(f)]
cpu = zones(root / f'{name}.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / f'{name}.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted((t, d) for n, t, d in cpu if n == 'State::reload_config')
assert len(markers) == steps + 1, f'{name}: {len(markers)} reload markers, expected {steps + 1}'
begin, end = markers[0][0], markers[-1][0] + markers[-1][1]
values = [d for n, t, d in gpu if n == 'MaterialRenderElement::draw' and begin <= t <= end]
assert len(values) >= steps, f'{name}: {len(values)} material draws for {steps} damages'
print(f'{name}_MaterialRenderElement::draw_median_ms={statistics.median(values) / 1e6:.6f}')
PYREPORT
done
python3 - "$OUT/metrics.txt" <<'PYVERDICT' >> "$OUT/metrics.txt"
import sys
m = dict(line.strip().split('=', 1) for line in open(sys.argv[1]) if '=' in line)
get = lambda n: float(m[f'{n}_MaterialRenderElement::draw_median_ms'])
for base in ['tilt0', 'tilt10', 'fine']:
    a1, a2, b1, b2 = (get(f'{base}-{s}') for s in ['a1', 'a2', 'b1', 'b2'])
    b = (b1 + b2) / 2
    a = (a1 + a2) / 2
    bound = max(0.01 * b, 0.002)
    if abs(a1 - a2) > bound or abs(b1 - b2) > bound:
        verdict = 'INCONCLUSIVE'
    else:
        verdict = 'PASS' if a - b <= bound else 'FAIL'
    print(f'{base}_a_ms={a:.6f}')
    print(f'{base}_b_ms={b:.6f}')
    print(f'{base}_bound_ms={bound:.6f}')
    print(f'{base}_verdict={verdict}')
PYVERDICT

# The swing case, on a only: two windows, focus gains on the probe every 3 s.
cfg=$OUT/swing.kdl
write_cost_config "$cfg" tilt10 a
start_nested "$NIRI_TRACY" "$cfg"
capture_bg swing; capture_ready swing
start_wall "$WALL"
sleep 0.5
spawn_probe "$NIRI" "$IDLE"
steal_focus "$NIRI"
probe=$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-probe") | .id')
other=$(msg "$NIRI" -j windows | jq -r '.[] | select(.app_id=="gos-other") | .id')
sleep 1.5
reload_marker "$cfg"
for i in $(seq 1 "$GAINS"); do
    reload_marker "$cfg"
    msg "$NIRI" action focus-window --id "$probe"
    sleep 1.5
    msg "$NIRI" action focus-window --id "$other"
    sleep 1.5
done
capture_wait
stop_walls; stop_nested
csvexport --gpu "$OUT/swing.tracy" "$OUT/swing.gpu.csv"
export_cpu swing
python3 - "$OUT" "$GAINS" <<'PYSWING' >> "$OUT/metrics.txt"
import csv, statistics, sys, pathlib
root, gains = pathlib.Path(sys.argv[1]), int(sys.argv[2])
def zones(path, t, d):
    with path.open() as f:
        return [(next(iter(r.values())), float(r[t]), float(r[d])) for r in csv.DictReader(f)]
cpu = zones(root / 'swing.csv', 'ns_since_start', 'exec_time_ns')
gpu = zones(root / 'swing.gpu.csv', 'Time from start of program', 'GPU execution time')
markers = sorted(t for n, t, _ in cpu if n == 'State::reload_config')[1:]
assert len(markers) == gains, f'{len(markers)} gain markers, expected {gains}'
m = dict(line.strip().split('=', 1) for line in open(root / 'metrics.txt') if '=' in line)
rest = float(m['tilt10_a_ms']) * 1e6
frames = [d for t0 in markers for n, t, d in gpu if n == 'MaterialRenderElement::draw' and t0 <= t <= t0 + 0.9e9]
assert len(frames) >= 6 * gains, f'{len(frames)} swing frames for {gains} gains'
print(f'swing_frames={len(frames)}')
print(f'swing_frame_median_ms={statistics.median(frames) / 1e6:.6f}')
print(f'swing_frame_max_ms={max(frames) / 1e6:.6f}')
print(f'swing_extra_ms_per_gain={sum(f - rest for f in frames) / gains / 1e6:.6f}')
PYSWING
finish
printf 'PASS: view tilt cost run complete (pilot=%s); verdicts in %s/metrics.txt\n' "$PILOT" "$OUT"
```

Check it: `bash -n docs/materials/scripts/view-tilt-cost.sh` (expect no output), and `shellcheck docs/materials/scripts/view-tilt-cost.sh` if it is installed (no errors; the sourced lib's variables are reported as unassigned, which is expected).

- [ ] **Step 2: Pilot it**

Run (on a quiet host; the script preflights `headless` itself):

```bash
B=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/niri-tracy-<base short sha>
OUT=$NIRI_MATERIAL_WORK_ROOT/material-6901e0/cost-pilot-$(date +%s) CAPTURE_TASK=material-2283b8 \
  VIEW_TILT_COST_AB=$B VIEW_TILT_COST_PILOT=1 docs/materials/scripts/view-tilt-cost.sh
```

Expected: about 12 minutes (build 4 with Tracy, twelve rest sub-runs and the swing case at about 40 s each); the last line `PASS: view tilt cost run complete (pilot=1)`; `metrics.txt` carries a verdict per case and the four swing lines. Read them: a pilot is too short to judge the bound, but every gate must have run. Record `run:` for the attempt.

On a preflight refusal: `tasks note material-2283b8 "run: <m> min (est 12, headless); preflight <m>; refused: <cause>"`, then `tasks park material-2283b8 "Rerun view-tilt-cost.sh with VIEW_TILT_COST_PILOT=1 (~12 min: build 4, cases 8), then the full run (~20 min: cases 20); last attempt refused by the headless preflight" --reason quiet --waiting-on user --minutes 32 --needs headless`.

- [ ] **Step 3: Run it in full and judge**

The same command with `VIEW_TILT_COST_PILOT=0` (about 20 minutes). Record its `run:` note.

- If every rest verdict is `PASS`: record the medians, bounds and verdicts in the evidence document's *Cost* section, with the swing lines, and fill the §7 row in `performance.md` with the swing figures.
- If a verdict is `INCONCLUSIVE`: rerun; do not judge it.
- If a verdict is `FAIL`: the rest path regressed past the bound. Apply the contingency (Step 4), then rerun Steps 2–3.

- [ ] **Step 4 (conditional, only on a `FAIL`): the second program variant**

Per §6 "Contingency": the rest program is built from today's source and the view variant is bound only while a swing runs.

1. Guard the view path with `#ifdef MATERIAL_VIEW` (`#ifdef`, not `#if`: the grain program defines nothing, and GLSL ES leaves an undefined macro in `#if` an error). Two kinds of lines:
   - **Added lines** (the view uniforms and globals, `slabFrame`'s callers in the view block, `viewUnrotateDelta` through `viewHit`, every `if (g_view_on ...)` and `if (mat_view_on == 1)` statement, the probes, the wall spill line, `Surface.view`): inside `#ifdef MATERIAL_VIEW … #endif`.
   - **Changed rest lines**: `#ifdef MATERIAL_VIEW <the branch's line> #else <the line as it reads at the branch's base> #endif`, taking the base text from `git show <BASE>:<file>`. These are, by file:
     - `main.frag`: the window sample (`if (hasDecal && inRect(winV, ...))` and its `wv` line, base: `if (inRect(v, mat_geo_rect))` and `vec2 wv = (v - mat_geo_rect.xy) / mat_geo_rect.zw;`); the two `hash12(seed + ...)` jitter lines (base: `gl_FragCoord.xy`); `saturation_behind`, `noise_behind`, `noise_post` (base: `gl_FragCoord.xy`); the `Surface surf = Surface(...)` constructor (base: without `g_view`); and the coverage line moved by Task 9 if its text differs from the base.
     - `prelude.frag`: `slabSurface`'s opening (base: the lines Task 8 moved into `slabFrame`, verbatim), with `slabFrame` itself inside `#ifdef`; `tap`'s body if its text differs from the base (Task 8 kept the base lines and only added branches; check with `git diff <BASE> -- prelude.frag`).
     - `reflection.frag`: the `vec2 at` lines (base: `vec3 scene = srgbToLinear(sampleBackground(s.v + dir * reach / mat_area_size));`).
     - `noise.frag`: `vec2 p = (fragCoord + g_seed_glide) / scale;` (base: `vec2 p = fragCoord / scale;`); `common.frag`'s `g_seed_glide` inside `#ifdef`.
   Then check that the rest variant is the base text (no `#ifdef` nests inside another in these files):
   ```bash
   BASE=<the base commit in Task 12's done note>
   rm -rf target/rest-variant target/base-shaders && mkdir -p target/rest-variant target/base-shaders
   git archive "$BASE" src/render_helpers/shaders/material | tar -x -C target/base-shaders
   python3 - <<'PY'
   import pathlib
   src, out = pathlib.Path('src/render_helpers/shaders/material'), pathlib.Path('target/rest-variant')
   for f in src.glob('*.frag'):
       keep, state = [], 'rest'
       for line in f.read_text().splitlines():
           t = line.strip()
           if t == '#ifdef MATERIAL_VIEW':
               state = 'view'; continue
           if t == '#else' and state == 'view':
               state = 'else'; continue
           if t == '#endif' and state != 'rest':
               state = 'rest'; continue
           if state != 'view':
               keep.append(line)
       (out / f.name).write_text('\n'.join(keep) + '\n')
   PY
   diff -r target/base-shaders/src/render_helpers/shaders/material target/rest-variant
   ```
   Expected: no output. Any difference is a rest line that changed; put the base text under `#else`.
2. In `src/render_helpers/shaders/mod.rs`, add `material_view_source()` that inserts `#define MATERIAL_VIEW\n` after the precision line, leave `material_source()` without it, compile both, and add `ProgramType::MaterialView` beside `ProgramType::Material`. Split the uniform list: `material_uniform_names()` returns the base's list (no `mat_view*`), and `material_view_uniform_names()` returns it followed by the five view uniforms of Task 7; the view program compiles with the second.
   Retarget Task 7's `the_view_uniforms_are_declared` to the view program: it reads `material_view_source()` and `material_view_uniform_names()`, and gains the converse for the rest program: `material_uniform_names()` names no `mat_view*` uniform. It makes no claim about `material_source()`'s text: that string is the raw files concatenated, so the view declarations stay in it inside `#ifdef MATERIAL_VIEW`, and only the compiler drops them. That the rest program is the base's text is what Step 4 item 1's stripped-source comparison shows. Any other test that asserts on view lines in `material_source()` (Task 8's `Surface` pin, Task 9's prelude pins in `view::solid::tests`) reads the files, not the assembled rest source, and stays as it is; check with `just test-one -p niri material_source` and `just test-one -p niri the_view_uniforms_are_declared`.
3. In `MaterialRenderElement::draw`, pick `ProgramType::MaterialView` and append the view uniforms only when `self.view.is_some()`; at rest bind `ProgramType::Material` with the base's uniform list.
4. Retarget `the_view_branch_at_the_identity_is_byte_identical_to_rest` to compare the two programs (the forced identity pose binds the view variant), run every `view_tilt`, `material_source` and `the_view_uniforms_are_declared` test, then `just test-fast`, and validate both variants offline with `glslangValidator` (the Global Constraints command, and again with `#define MATERIAL_VIEW` inserted after the precision line).
5. Repeat Task 12's pinned comparison: dump `glass_edge` and `ring_look` at the new head into `$EV_HEAD2` and compare them byte for byte against the base dumps Task 12 kept (`$EV/base` of its run; Task 12 Step 2's `cmp` loop with `head` replaced by the new directory). Expected: no `DIFFERS` line. Record it in the evidence document's *Rest identity* section as a second row.
6. Commit as `perf(material): rigid tilt view path as a second program variant`, then rerun Steps 2–3.

- [ ] **Step 5: Commit**

```bash
tasks done material-2283b8 "rest cost within bound (tilt0 <a>/<b>, tilt10 <a>/<b>, fine <a>/<b> ms); swing <median> ms per frame, <extra> ms per gain"
tasks check
git add docs/materials/ tasks/
python3 tools/upstream-report && git add docs/materials/upstream-divergence.md
git commit -m "docs(material): rigid tilt GPU cost"
```

### Task 15: Owner clip on the nested lane

This task ends at the second owner gate: the motion, the sign and `settle`. Prism's starting values (`prism-2a8fea`) follow from it.

**Files:**
- Modify: `docs/materials/scripts/ring-motion-clips.sh` (four tilt sequences, the burst clock, the swing crops)
- Modify: `docs/materials/2026-10-<dd>-rigid-tilt-evidence.md` (the *Clip* section)

**Interfaces:**
- Consumes: the owner's pick from Task 13 (θp `T`, κ `K`); the script's `start_nested`, `spawn_kitty_to`, `kitty_ids`, `settle`, `burst_start`, `burst_wait`, `scratch_build`, `wants_scratch`.
- Produces: `$OUT/tilt-swing*/` bursts and `$OUT/tilt-swing*-swing/at-*.png` crops, `clips.txt`, and GIFs as the beam sequences write them.

- [ ] **Step 1: Add the sequences**

In `docs/materials/scripts/ring-motion-clips.sh`:

1. Extend the header's sequence list:
   ```bash
   #   tilt-swing          the rigid tilt alone (beam off) at `view-tilt $TILT;
   #                       view-distance $KAPPA`: a 2.5 s burst from before the gain
   #                       and 4x crops of the gaining pane's corner through the swing
   #   tilt-swing-beam     the same with the beam on: the look the owner judges
   #   tilt-swing-slow     `view-tilt-settle 1400`, beside the 900 ms default
   #   tilt-swing-flipped  a scratch build with the swing's sign flipped: the
   #                       origin's edge comes forward first
   ```
   and the count in its first lines to match.
2. After `SEQUENCES=...`, add:
   ```bash
   TILT=${TILT:-10}; KAPPA=${KAPPA:-6}   # the owner's pick from the stills (Task 13)
   TILT_SWINGS="tilt-swing tilt-swing-beam tilt-swing-slow tilt-swing-flipped"
   SWING_S=2.5   # view-tilt-settle is 900 ms; the rest of the burst shows the pane at rest
   ```
3. In `write_config`, add to the response line `view-tilt ${VTILT:-0}; view-distance ${VKAPPA:-6}; view-tilt-settle ${SETTLE:-900};`, and list `VTILT, VKAPPA, SETTLE` in its comment.
4. After the `GLASS='bevel 0; ...' write_config` line:
   ```bash
   SPEED=0 VTILT=$TILT VKAPPA=$KAPPA write_config "$OUT/tilt-swing.kdl"
   VTILT=$TILT VKAPPA=$KAPPA write_config "$OUT/tilt-swing-beam.kdl"
   SPEED=0 VTILT=$TILT VKAPPA=$KAPPA SETTLE=1400 write_config "$OUT/tilt-swing-slow.kdl"
   ```
5. After the `beam-splash` scratch block:
   ```bash
   NIRI_FLIPPED=
   if wants_scratch tilt-swing-flipped; then
       NIRI_FLIPPED=$(scratch_build flipped \
           'src/render_helpers/material/view/mod.rs|s/^\( *\)theta: peak_deg.to_radians() \* shape,$/\1theta: -peak_deg.to_radians() * shape,/|theta: -peak_deg.to_radians() * shape,')
       BINARIES+=(--binary "$NIRI_FLIPPED")
   fi
   ```
6. Add `--input "$OUT/tilt-swing.kdl" --input "$OUT/tilt-swing-beam.kdl" --input "$OUT/tilt-swing-slow.kdl"` to the `capture_meta identity` call.
7. Replace `BURST_PID=` and the start of `burst_start` with `material-77db8a`'s shared burst clock, and add its `swing_crops` after `seq_beam_bevel0`, both verbatim from `git diff materials-26.04...d51825f9 -- docs/materials/scripts/ring-motion-clips.sh` (the `BURST_T0` hunk and the `swing_crops` function). Then add the sequence, which takes its binary as `beam_sequence` does:
   ```bash
   # One focus gain on a two-pane scene over SWING_S, as beam_sequence. The gain instant is
   # taken on the burst's clock just before the focus message, and the frames are chosen from
   # it; a host too slow to take six frames inside the settle fails rather than showing rest.
   swing_sequence() {   # $1 = label, $2 = config, $3 = binary
       local ids gain early saved=$NIRI
       NIRI=$3
       start_nested "$2" "$1"
       spawn_kitty_to 2
       ids=($(kitty_ids))
       msg action focus-window --id "${ids[1]}"; settle
       burst_start "$1" "$SWING_S"
       gain=$(date +%s.%N)
       msg action focus-window --id "${ids[0]}"
       gain=$(awk -v a="$gain" -v b="$BURST_T0" 'BEGIN { printf "%.3f", a - b }')
       burst_wait "$1"
       echo "$1: gain at ${gain} s into the burst" >> "$OUT/clips.txt"
       early=$(awk -v g="$gain" '$2 - g >= 0 && $2 - g < 0.9' "$OUT/$1/frames.txt" | wc -l)
       [ "$early" -ge 6 ] || fail "$1: only $early frames inside the 900 ms settle after the gain"
       swing_crops "$1" "$gain"
       stop_nested
       finish_sub_run "$1"
       NIRI=$saved
   }
   seq_tilt_swing()         { swing_sequence tilt-swing         "$OUT/tilt-swing.kdl"      "$NIRI"; }
   seq_tilt_swing_beam()    { swing_sequence tilt-swing-beam    "$OUT/tilt-swing-beam.kdl" "$NIRI"; }
   seq_tilt_swing_slow()    { swing_sequence tilt-swing-slow    "$OUT/tilt-swing-slow.kdl" "$NIRI"; }
   seq_tilt_swing_flipped() { swing_sequence tilt-swing-flipped "$OUT/tilt-swing.kdl"      "$NIRI_FLIPPED"; }
   ```
8. Add the four cases to the dispatch `case`, the flipped one guarded like `beam-nospill`.

Check: `bash -n docs/materials/scripts/ring-motion-clips.sh`.

- [ ] **Step 2: Pilot it**

On the headless Weston lane (never the desktop), the smallest run through its verdict first:

```bash
CAPTURE_TASK=material-c5f9d0 TILT=<T> KAPPA=<K> SEQUENCES=tilt-swing BEAM_SCRATCH=0 \
  docs/materials/scripts/ring-motion-clips.sh
```

Expected: about 8 minutes (build 4, one sequence); `clips: OK`, and `clips.txt` names the gain offset and the four crops; the swing sequence's own check (six frames inside the settle) passed. Record the `run:` note. On a preflight refusal, note the attempt and `tasks park material-c5f9d0 "Run ring-motion-clips.sh with SEQUENCES=tilt-swing (pilot, ~8 min), then the four tilt sequences (~25 min: build 4, scratch build 10, sequences 11); TILT=<T> KAPPA=<K>" --reason quiet --waiting-on user --minutes 33 --needs headless`.

- [ ] **Step 3: Run the four sequences**

The same command with `SEQUENCES="$TILT_SWINGS"` spelled out (`tilt-swing tilt-swing-beam tilt-swing-slow tilt-swing-flipped`) and `BEAM_SCRATCH=1`. Record its `run:` note.

- [ ] **Step 4: Record and stop for the owner**

Append to the evidence document:

```markdown
## Clip

Four swings at the owner's pick (`view-tilt <T>; view-distance <K>`) on the
nested lane: `$EV` = `<run-stamp>` (`tilt-swing`, `tilt-swing-beam`,
`tilt-swing-slow` at 1400 ms, and `tilt-swing-flipped` with the sign
reversed), each a 2.5 s burst from before the gain with 4x corner crops at
0.15, 0.3, 0.45 and 0.7 s.

Owner's verdict: pending.
```

```bash
tasks note material-c5f9d0 "clips: $OUT (tilt-swing, -beam, -slow, -flipped)"
tasks check
git add docs/materials/ tasks/
git commit -m "test(material): rigid tilt owner clips"
tasks park material-c5f9d0 "Owner judges the swing's motion, sign and settle from the clips in $OUT; then the agent records the verdict, flips the sign if asked (one line in view/mod.rs, its tests' signs), sets view-tilt-settle's default if asked, notes prism-2a8fea with the starting values, and closes material-6901e0's plan" --waiting-on user --reason review
git add tasks/ && git commit -m "chore(tasks): park material-c5f9d0 for the owner's clip verdict"
```

When the verdict arrives: record it under *Clip*; if the owner wants the other sign, change `theta: peak_deg.to_radians() * shape,` to its negation and the sign assertions of Tasks 2, 3 and 6 with it (a RED→GREEN pair: flip the assertions first); if the owner wants another default settle, change `view_tilt_settle: 900.` in niri-config and the docs' table; then `tasks note prism-2a8fea "starting values from the owner's clip review (<date>): view-tilt <T>, view-distance <K>, view-tilt-settle <S>"` and `tasks done material-c5f9d0 "<verdict>"` in that commit.

