# Material Roughness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native `roughness` glass parameter whose visible effect comes
from damage-aware sharp and blurred prefilter pyramids owned by each render
target's `EffectBuffer`.

**Architecture:** Keep level calculation and dirty/failure transitions in a
GPU-free `PrefilterState`; keep texture allocation and the downsample draw in
the existing effect-buffer/blur boundary. The material draw asks each buffer
for two adjacent levels and a blend, while roughness zero reuses level zero and
performs no allocation or downsample work.

**Tech Stack:** Rust, knuffel, smithay GLES2, GLSL ES 1.00, Tracy, `cargo test`.

**Spec:** `docs/materials/2026-09-01-material-roughness-design.md`

## Global Constraints

- Native `roughness` uses `FloatOrInt<0, 1>` and defaults to `0`; Prism owns
  its separate `0.08` default.
- `backdrop-blur` chooses the sharp or globally blurred source first;
  roughness then filters that selected source.
- Level zero is the existing source. Derived levels halve each dimension,
  clamp each to one, and stop at `1 x 1`.
- Derived storage is strictly less than one additional source for every
  positive source size. Do not encode the normal-output one-third
  approximation as an invariant.
- Roughness zero allocates no pyramid and does not execute a second texture
  fetch. Bind the unused high sampler to level zero.
- A failed preparation warns once per dirty period and uses the existing plain
  window fallback; damage, option, size, or context changes permit one retry.
- Do not add a new shader, dependency, compatibility alias, general benchmark
  harness, or region-damage propagation.
- Conventional commits only, with named paths staged and no AI attribution.
- Use a writable target directory when the configured Cargo target is not
  writable, for example `CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-c854bd-target`.

## File Structure

| File | Responsibility | Task |
| --- | --- | --- |
| `niri-config/src/material.rs` | Parse, default, and resolve `roughness` | 1 |
| `niri-config/src/lib.rs` | Config acceptance/default/range tests | 1 |
| `src/render_helpers/effect_buffer.rs` | GPU-free state, per-source cache ownership, invalidation, failure suppression | 2, 3 |
| `src/render_helpers/blur.rs` | Reusable downsample-only GL execution | 3 |
| `src/render_helpers/material.rs` | Request level pairs, bind textures and blend uniforms | 4 |
| `src/render_helpers/shaders/material.frag` | Blend each source's two explicit levels | 4 |
| `src/render_helpers/shaders/mod.rs` | Declare two samplers and two uniforms | 4 |
| `docs/materials/material-config.md` | User-facing roughness contract | 5 |
| `docs/materials/2026-09-01-material-roughness-design.md` | Landed status and evidence links | 5 |
| `README.md` | Keep the material documentation index current | 5 |

---

### Task 1: Add the native roughness configuration contract

**Files:**

- Modify: `niri-config/src/material.rs`
- Modify: `niri-config/src/lib.rs`

**Interfaces:**

- Produces: `Glass::roughness: Option<FloatOrInt<0, 1>>` and
  `ResolvedGlass::roughness: f64` with default `0.0`.
- Consumed by: Task 4 passes `ResolvedGlass::roughness` to both effect
  buffers.

- [ ] **Step 1: Write the failing config tests**

In `niri-config/src/lib.rs`, add `roughness 0.08` to
`material_parses_full_glass_block`, add `roughness: 0.08` to its expected
`ResolvedGlass`, and add:

```rust
    #[test]
    fn roughness_defaults_to_zero() {
        assert_eq!(ResolvedGlass::default().roughness, 0.);
    }

    #[test]
    fn roughness_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ roughness {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }
```

- [ ] **Step 2: Run the focused tests and confirm the new field is missing**

Run:

```bash
cargo test -p niri-config roughness -- --nocapture
```

Expected: FAIL because `ResolvedGlass` has no `roughness` field.

- [ ] **Step 3: Add the minimal parser and resolver fields**

In `Glass`, immediately after `anisotropic_blur`, add:

```rust
    #[knuffel(child, unwrap(argument))]
    pub roughness: Option<FloatOrInt<0, 1>>,
```

Add `pub roughness: f64` in the same position in `ResolvedGlass`, set
`roughness: 0.` in `Default`, and resolve it with:

```rust
                roughness: g.roughness.map_or(d.roughness, |x| x.0),
```

- [ ] **Step 4: Run the config suite**

Run:

```bash
cargo test -p niri-config material_ -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs
git commit -m "feat(config): add glass roughness"
```

### Task 2: Add GPU-free prefilter state

**Files:**

- Modify: `src/render_helpers/effect_buffer.rs`
- Test: `src/render_helpers/effect_buffer.rs` inline `mod tests`

**Interfaces:**

- Produces: private `PrefilterState`, `PrefilterStatus`, and
  `PrefilterSelection { low: usize, high: usize, mix: f32 }`.
- Produces methods used by Task 3:
  `new(Size<i32, Buffer>)`, `invalidate()`, `needs_prepare()`,
  `mark_clean()`, `mark_failed()`, `selection(f64, f64)` and
  `level_sizes()`.

- [ ] **Step 1: Add failing state tests**

Append an inline test module to `effect_buffer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilter_levels_reach_one_by_one_and_stay_below_the_source() {
        for w in 1..=64 {
            for h in 1..=64 {
                let state = PrefilterState::new(Size::from((w, h)));
                let pixels: i32 = state.level_sizes().iter().map(|s| s.w * s.h).sum();
                assert!(pixels < w * h || w * h == 1, "{w} x {h}: {pixels}");
                if w * h == 1 {
                    assert!(state.level_sizes().is_empty());
                } else {
                    assert_eq!(state.level_sizes().last(), Some(&Size::from((1, 1))));
                }
            }
        }
    }

    #[test]
    fn prefilter_selection_matches_reference_formula() {
        let state = PrefilterState::new(Size::from((1920, 1080)));
        assert_eq!(state.selection(0., 1.5), PrefilterSelection::level_zero());
        let selected = state.selection(0.08, 1.5);
        assert_eq!((selected.low, selected.high), (0, 1));
        assert!((selected.mix - 0.8).abs() < f32::EPSILON);
        assert_eq!(state.selection(1., 1.), PrefilterSelection::level_zero());
    }

    #[test]
    fn failure_retries_only_after_invalidation() {
        let mut state = PrefilterState::new(Size::from((1920, 1080)));
        assert!(state.needs_prepare());
        state.mark_failed();
        assert!(!state.needs_prepare());
        state.invalidate();
        assert!(state.needs_prepare());
        state.mark_clean();
        assert!(!state.needs_prepare());
    }
}
```

- [ ] **Step 2: Run the tests and confirm the types are missing**

Run:

```bash
cargo test -p niri prefilter_ -- --nocapture
```

Expected: FAIL because `PrefilterState` is undefined.

- [ ] **Step 3: Implement the state machine and exact level math**

Add above `EffectBuffer`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrefilterStatus {
    Dirty,
    Clean,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PrefilterSelection {
    low: usize,
    high: usize,
    mix: f32,
}

impl PrefilterSelection {
    const fn level_zero() -> Self {
        Self { low: 0, high: 0, mix: 0. }
    }
}

#[derive(Debug)]
struct PrefilterState {
    level_sizes: Vec<Size<i32, Buffer>>,
    status: PrefilterStatus,
    highest_prepared_level: usize,
}

impl PrefilterState {
    fn new(size: Size<i32, Buffer>) -> Self {
        let mut level_sizes = Vec::new();
        let (mut w, mut h) = (size.w, size.h);
        while w > 1 || h > 1 {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            level_sizes.push(Size::from((w, h)));
        }
        Self { level_sizes, status: PrefilterStatus::Dirty, highest_prepared_level: 0 }
    }

    fn level_sizes(&self) -> &[Size<i32, Buffer>] { &self.level_sizes }
    fn needs_prepare(&self) -> bool { self.status == PrefilterStatus::Dirty }
    fn invalidate(&mut self) { self.status = PrefilterStatus::Dirty; self.highest_prepared_level = 0; }
    fn mark_failed(&mut self) { self.status = PrefilterStatus::Failed; self.highest_prepared_level = 0; }
    fn mark_clean(&mut self) { self.status = PrefilterStatus::Clean; self.highest_prepared_level = self.level_sizes.len(); }

    fn selection(&self, roughness: f64, ior: f64) -> PrefilterSelection {
        let max = self.level_sizes.len();
        if max == 0 || roughness == 0. {
            return PrefilterSelection::level_zero();
        }
        let lod = max as f64 * roughness * (ior * 2. - 2.).clamp(0., 1.);
        let low = lod.floor() as usize;
        let high = (low + 1).min(max);
        PrefilterSelection { low, high, mix: (lod - low as f64) as f32 }
    }
}
```

- [ ] **Step 4: Run the focused tests**

Run:

```bash
cargo test -p niri prefilter_ -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/render_helpers/effect_buffer.rs
git commit -m "feat(render): model roughness prefilter state"
```

### Task 3: Cache and regenerate effect-buffer pyramids

**Files:**

- Modify: `src/render_helpers/blur.rs`
- Modify: `src/render_helpers/effect_buffer.rs`

**Interfaces:**

- Produces in `blur.rs`: `BlurProgram::render_downsample(renderer,
  source, targets, offset) -> anyhow::Result<()>`, reusing the existing down
  program and raw GL draw loop.
- Produces in `effect_buffer.rs`: public
  `PrefilteredTexture { low: GlesTexture, high: GlesTexture, mix: f32 }` and
  `EffectBuffer::render_prefiltered(&mut self, &mut GlesFrame, bool, f64,
  f64) -> Option<PrefilteredTexture>`.
- Consumed by: Task 4 material drawing.

- [ ] **Step 1: Extract the existing downsample draw without changing blur output**

Move the downsample half of `Blur::render` into one private helper used by
both callers. Expose this thin wrapper on `BlurProgram`:

```rust
    pub fn render_downsample(
        &self,
        renderer: &mut GlesRenderer,
        source: &GlesTexture,
        targets: &mut [GlesTexture],
        offset: f32,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("Prefilter::downsample");
        ensure!(targets.iter().all(GlesTexture::is_unique_reference),
            "prefilter texture has a non-unique reference");
        renderer.with_profiled_context(gpu_span_location!("Prefilter::downsample"), |gl| unsafe {
            draw_downsample(gl, &self.0.down, source, targets, offset);
        })?;
        Ok(())
    }
```

The extracted `draw_downsample` must retain the current GLES2 setup, linear
clamp filtering, destination-sized `half_pixel`, and one draw per target.
`Blur::render` calls it with `&mut self.textures[1..]` and
`options.offset as f32`; prefilter calls it with `1.0`.

- [ ] **Step 2: Run the existing blur build checks**

Run:

```bash
cargo test -p niri render_helpers::blur --no-run
```

Expected: PASS; extraction has not changed the full blur path.

- [ ] **Step 3: Add two states and two texture vectors to each offscreen**

Add these fields to `Offscreen`, initialized from the new texture's size:

```rust
    sharp_prefilter: PrefilterState,
    sharp_prefilter_textures: Vec<GlesTexture>,
    blurred_prefilter: PrefilterState,
    blurred_prefilter_textures: Vec<GlesTexture>,
```

Add the return type:

```rust
#[derive(Debug, Clone)]
pub struct PrefilteredTexture {
    pub low: GlesTexture,
    pub high: GlesTexture,
    pub mix: f32,
}
```

Use one helper to map level zero to the selected source and positive levels to
`textures[level - 1]`. Never clone a texture before downsampling finishes,
because `is_unique_reference` is the allocation guard.

- [ ] **Step 4: Implement lazy preparation and once-per-dirty failure**

`render_prefiltered` first calls the existing `render(frame, blur)` to obtain
the selected source, computes `selection`, and immediately returns the source
twice when `(low, high) == (0, 0)`. For non-zero selection:

```rust
        if state.needs_prepare() {
            let result = prepare_prefilter(renderer, program, source, state, textures);
            if let Err(err) = result {
                state.mark_failed();
                warn!("material prefilter failed: {err:?}");
                return None;
            }
            state.mark_clean();
        } else if state.status == PrefilterStatus::Failed {
            return None;
        }
```

`prepare_prefilter` creates only missing `Abgr8888` textures at
`state.level_sizes()`, emits a trace allocation reason, then calls
`BlurProgram::render_downsample(..., 1.0)`. Put Tracy spans around the whole
preparation and downsample call.

On actual level-zero damage, call `sharp_prefilter.invalidate()`, clear the
full-size blurred texture, and invalidate `blurred_prefilter`. When a new
blurred texture is produced, invalidate `blurred_prefilter`. Blur option
changes invalidate only `blurred_prefilter`; size or context replacement
recreates the whole `Offscreen`. Add the accepted ponytail comment above the
full-chain regeneration.

- [ ] **Step 5: Build and run state/config tests**

Run:

```bash
cargo test -p niri prefilter_ -- --nocapture
cargo test -p niri-config material_ -- --nocapture
cargo build --workspace --all-targets
```

Expected: all PASS. Inspect the new method to confirm the clean and failed
branches allocate nothing and issue no GL draw.

- [ ] **Step 6: Commit**

```bash
git add src/render_helpers/blur.rs src/render_helpers/effect_buffer.rs
git commit -m "feat(render): cache roughness prefilter pyramids"
```

### Task 4: Blend explicit prefilter levels in glass

**Files:**

- Modify: `src/render_helpers/material.rs`
- Modify: `src/render_helpers/shaders/material.frag`
- Modify: `src/render_helpers/shaders/mod.rs`
- Test: `src/render_helpers/material.rs` inline `mod tests`

**Interfaces:**

- Consumes: `ResolvedGlass::roughness` and
  `EffectBuffer::render_prefiltered` from Tasks 1 and 3.
- Produces: samplers `niri_tex_bg_high`, `niri_tex_backdrop_high` and uniforms
  `mat_bg_prefilter_mix`, `mat_backdrop_prefilter_mix`.

- [ ] **Step 1: Extend the in-place parameter commit test**

In `parameter_change_advances_the_commit_in_place`, change only roughness on
the second resolved material before `apply_resolved`:

```rust
        changed.glass.roughness = 0.08;
```

Keep the assertions that the slot identity is preserved and the commit
advances.

- [ ] **Step 2: Request level pairs and preserve the plain fallback**

Replace both `EffectBuffer::render(...).ok()` calls with:

```rust
        let bg_texture = self.bg.borrow_mut().render_prefiltered(
            frame,
            self.glass.backdrop_blur,
            self.glass.roughness,
            self.glass.ior,
        );
```

and the corresponding backdrop call. Keep the existing plain-window subdraw
on either `None`, but remove its repeated generic warning because the
effect-buffer transition emits the one actionable warning.

Add each pair's `mix` to the uniform array and bind five textures total. At
mix zero, bind the selected level-zero clone for both low and high.

- [ ] **Step 3: Add the GLSL pair sampler**

Declare the two new samplers and mix uniforms, then add:

```glsl
vec4 samplePrefilter(
    sampler2D low_tex,
    sampler2D high_tex,
    float amount,
    vec2 uv
) {
    vec4 low = texture2D(low_tex, clamp(uv, 0.0, 1.0));
    if (amount == 0.0)
        return low;
    return mix(low, texture2D(high_tex, clamp(uv, 0.0, 1.0)), amount);
}
```

Use it in `sampleBackdrop` and the workspace branch of `sampleBackground`.
Do not change composition order or any optical tap math.

- [ ] **Step 4: Register the uniforms and samplers**

In `Shaders::compile`, add two `_1f` `UniformName`s and replace the material
sampler slice with:

```rust
            &[
                "niri_tex_win",
                "niri_tex_bg",
                "niri_tex_bg_high",
                "niri_tex_backdrop",
                "niri_tex_backdrop_high",
            ],
```

- [ ] **Step 5: Run shader and Rust verification**

Run:

```bash
glslangValidator -S frag src/render_helpers/shaders/material.frag
cargo test -p niri parameter_change_advances_the_commit_in_place -- --nocapture
cargo test --workspace --all-targets
```

Expected: all PASS. The compiled material program declares exactly five
samplers, below the GLES2 minimum of eight.

- [ ] **Step 6: Commit**

```bash
git add src/render_helpers/material.rs src/render_helpers/shaders/material.frag src/render_helpers/shaders/mod.rs
git commit -m "feat(material): render glass roughness"
```

### Task 5: Verify and document roughness delivery

**Files:**

- Modify: `docs/materials/material-config.md`
- Modify: `docs/materials/2026-09-01-material-roughness-design.md`
- Modify only if its material link text is stale: `README.md`
- Modify through CLI: `tasks/*.md`

**Interfaces:**

- Consumes: the complete native implementation from Tasks 1–4.
- Produces: user reference, recorded evidence, landed design status, and a
  closed task chain. Prism is a separate repository delivery and must not be
  claimed complete by this task.

- [ ] **Step 1: Run the complete automated suite**

Run:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo build --workspace --all-targets
tasks check
git diff --check
```

Expected: all exit zero. Existing unrelated warnings may remain, but no new
warning is accepted.

- [ ] **Step 2: Record focused GPU evidence**

Use the established nested Weston material fixture and the same output size,
driver, binary profile, window layout, and capture timing for every run. Save
raw traces and images outside the source tree. Record:

- native roughness `0`, Prism-equivalent `0.08`, and worst-case `0.08` with
  anisotropic blur `1` plus chromatic aberration `1` on a large window;
- a 60-second unchanged interval after warm-up with zero prefilter allocation
  and downsample spans;
- one induced background change with one regeneration per selected source and
  target followed by reuse;
- settled and overview captures at roughness `0.5`, including recorded zoom;
  measure the 10%–90% checkerboard edge width and require the source-pixel
  difference to be no more than `1 / zoom` source pixels;
- matched GPU `render` samples only when counts differ by less than 10%, and
  report the observed delta without inventing a threshold.

If the default path regresses beyond run-to-run noise for an unexplained
reason, stop before changing status.

- [ ] **Step 3: Update the user reference and evidence record**

Add `roughness` to the example and parameter table in
`docs/materials/material-config.md`, documenting native default `0`, range
`0..1`, source selection order, and lazy allocation. In the design document,
replace the status with the exact implementation commits and link the raw
evidence location/commit. Grep outward claims:

```bash
rg -n "roughness|mipmap|prefilter|implementation has not started" README.md docs
```

Correct only claims made stale by this delivery; preserve historical design
records as historical.

- [ ] **Step 4: Close the native task records and commit**

Use `tasks done` for each completed plan step, then close `material-c854bd`
only after the separate Prism commit exists and the accepted cross-repository
scope is satisfied. Run `tasks check`, stage named documentation/task paths,
and commit:

```bash
git commit -m "docs(material): record roughness acceptance"
```

## Separate Prism delivery

Execute after native grammar exists. In the Prism repository, create its own
task and plan, then make one test-driven conventional commit touching only:

- `defs/glass.yaml`: restore `glass.roughness`, range `0..1`, default `0.08`,
  percent presentation, historical Glass blur label;
- `integrations/niri/manifest.yaml`: bind it with reload liveness;
- `integrations/niri/render.js`: emit `roughness` beside the other glass
  optics;
- `test/glass-defs.test.js`: remove roughness from the retired set and assert
  the public definition;
- `test/niri-render.test.js` and `test/niri-apply.test.js`: assert exact KDL;
- `docs/superpowers/specs/2026-08-28-niri-native-material-sink-design.md` and
  `docs/plans/2026-08-31-prism-tasks-migration.md`: correct the now-stale
  statements that Prism does not expose roughness.

Run Prism's repository-defined formatting and test commands before the commit.
Do not add an alias or a second parameter. Link its commit in the native
evidence/status update before closing `material-c854bd`.
