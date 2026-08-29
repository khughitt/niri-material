# Material Backdrop Blur Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a glass material refract the blurred backdrop instead of the sharp one, so blur and refraction compose into frost rather than competing as two images.

**Architecture:** `EffectBuffer` already produces and caches a blurred texture beside the sharp one, and the xray buffers already receive blur options from the global config every frame. The material is the one consumer hardcoded to ask for sharp. This adds a `backdrop-blur` boolean to the `glass` block, gates it against global `blur { off }` inside `resolve_material` so a single already-gated value exists, and threads that one value to the two `prepare` sites and the two `render` sites.

**Tech Stack:** Rust, knuffel (KDL derive), smithay/GLES, `cargo test`.

**Spec:** `docs/materials/2026-08-29-material-backdrop-blur-design.md`

## Global Constraints

- The parameter is `backdrop-blur`, a bool, default `false`. An existing material with no `backdrop-blur` key must render exactly as it does today.
- Strength is not per-material. It comes from the global `blur { passes; offset }` block.
- `blur { off }` disables blur globally and overrides the material. The effective value is `glass.backdrop_blur && !blur.off`.
- Preparation and drawing must read the same value. After Task 2, no call to `EffectBuffer::prepare` or `EffectBuffer::render` on a material path may pass a bool literal.
- Both the `background` and the `backdrop` buffer follow the one flag; they must never disagree.
- Do not remove or alter `anisotropic-blur`.

---

### Task 1: Add `backdrop-blur` to the glass config

**Files:**
- Modify: `niri-config/src/material.rs:167-192` (the `Glass` parse struct)
- Modify: `niri-config/src/material.rs:203-217` (`ResolvedGlass`)
- Modify: `niri-config/src/material.rs:219-238` (`ResolvedGlass::default`)
- Modify: `niri-config/src/material.rs:243-272` (`Material::resolve`)
- Modify: `niri-config/src/lib.rs:912-923` (an exhaustive `ResolvedGlass` literal in an existing test)
- Test: `niri-config/src/material.rs` (its `mod tests`)

**Interfaces:**
- Produces: `Glass::backdrop_blur: Option<bool>` and `ResolvedGlass::backdrop_blur: bool` (default `false`). Task 2 reads `ResolvedGlass::backdrop_blur`.

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block in `niri-config/src/material.rs`:

```rust
#[test]
fn backdrop_blur_defaults_to_off() {
    assert!(!ResolvedGlass::default().backdrop_blur);
}

#[test]
fn backdrop_blur_parses_and_resolves() {
    let config = Config::parse_mem(
        r#"
        material "frost" {
            glass {
                backdrop-blur true
            }
        }
        "#,
    )
    .unwrap();
    assert!(config.materials[0].resolve().glass.backdrop_blur);
}

#[test]
fn an_omitted_backdrop_blur_resolves_to_the_default() {
    let config = Config::parse_mem(
        r#"
        material "frost" {
            glass {}
        }
        "#,
    )
    .unwrap();
    assert!(!config.materials[0].resolve().glass.backdrop_blur);
}

#[test]
fn backdrop_blur_rejects_a_non_boolean() {
    assert!(Config::parse_mem(
        r#"
        material "frost" {
            glass {
                backdrop-blur 0.5
            }
        }
        "#,
    )
    .is_err());
}
```

If `Config` is not already in scope in that test module, add `use crate::Config;` at the top of `mod tests`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri-config backdrop_blur`
Expected: FAIL — `no field 'backdrop_blur' on type 'ResolvedGlass'` (a compile error is the expected failure here).

- [ ] **Step 3: Add the parse field**

In the `Glass` struct, after the `anisotropic_blur` field at `niri-config/src/material.rs:181`:

```rust
    #[knuffel(child, unwrap(argument))]
    pub backdrop_blur: Option<bool>,
```

- [ ] **Step 4: Add the resolved field and its default**

In `ResolvedGlass`, after `anisotropic_blur: f64,`:

```rust
    /// Whether the material samples the blurred backdrop. Strength comes from
    /// the global `blur` block; `blur { off }` overrides this.
    pub backdrop_blur: bool,
```

In `impl Default for ResolvedGlass`, after `anisotropic_blur: 0.,`:

```rust
            backdrop_blur: false,
```

- [ ] **Step 5: Resolve it**

In `Material::resolve`, after the `anisotropic_blur` line:

```rust
                backdrop_blur: g.backdrop_blur.unwrap_or(d.backdrop_blur),
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p niri-config backdrop_blur`
Expected: PASS, 4 tests.

- [ ] **Step 7: Fix the known struct literal and run the whole config suite**

`ResolvedGlass` gained a field, which breaks the exhaustive struct literal in
`niri-config/src/lib.rs:912-923` (`material_parameters_resolve`). Add the field
to it, after `anisotropic_blur: 0.75,`:

```rust
                backdrop_blur: false,
```

That test asserts a fully-specified material resolves exactly, so `false` is
correct: its KDL does not set `backdrop-blur`.

Run: `cargo test -p niri-config`
Expected: PASS, including `material_omitted_parameters_take_design_defaults`, which compares against `ResolvedGlass::default()` and needs no edit.

- [ ] **Step 8: Commit**

```bash
git add niri-config/src/material.rs
git commit -m "feat(materials): add backdrop-blur to the glass config"
```

---

### Task 2: Gate the flag once in `resolve_material`

**Files:**
- Modify: `src/layout/tile.rs:151-163` (`resolve_material`)
- Test: `src/layout/tile.rs` (its `mod tests` at 1944)

**Interfaces:**
- Consumes: `ResolvedGlass::backdrop_blur` from Task 1; `Options::blur` (`niri_config::Blur`, field `off: bool`) and `Options::materials` from `src/layout/mod.rs:394-398`.
- Produces: `resolve_material` returns a `ResolvedMaterial` whose `glass.backdrop_blur` is already gated against global blur. Task 3 reads it and must not re-gate.

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `src/layout/tile.rs`:

```rust
    fn options_with(name: &str, backdrop_blur: bool, blur_off: bool) -> Options {
        let mut glass = niri_config::ResolvedGlass::default();
        glass.backdrop_blur = backdrop_blur;
        let material = ResolvedMaterial {
            name: String::from(name),
            glass,
        };
        let mut options = Options::default();
        options.materials = Rc::new(HashMap::from([(String::from(name), material)]));
        options.blur.off = blur_off;
        options
    }

    #[test]
    fn backdrop_blur_survives_resolution_when_global_blur_is_on() {
        let options = options_with("frost", true, false);
        let resolved = resolve_material(Some("frost"), &options).unwrap();
        assert!(resolved.glass.backdrop_blur);
    }

    #[test]
    fn global_blur_off_overrides_the_material() {
        // `blur { off }` means off. BlurOptions carries only passes and offset,
        // so nothing downstream would honor `off` if it were not applied here.
        let options = options_with("frost", true, true);
        let resolved = resolve_material(Some("frost"), &options).unwrap();
        assert!(!resolved.glass.backdrop_blur);
    }

    #[test]
    fn a_material_that_opts_out_stays_out_with_global_blur_on() {
        let options = options_with("frost", false, false);
        let resolved = resolve_material(Some("frost"), &options).unwrap();
        assert!(!resolved.glass.backdrop_blur);
    }
```

The module's `use super::*` already brings in `Rc` (imported at `src/layout/tile.rs:2`) and `ResolvedMaterial` (line 5). Add the one that is missing, at the top of `mod tests`:

```rust
    use std::collections::HashMap;
```

`ResolvedGlass` is not imported in `tile.rs`, which is why the helper above names it as `niri_config::ResolvedGlass`. It is re-exported from the crate root (`niri-config/src/lib.rs:57`).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri backdrop_blur`
Expected: FAIL — `backdrop_blur_survives_resolution_when_global_blur_is_on` passes trivially, but `global_blur_off_overrides_the_material` FAILS with `assertion failed: !resolved.glass.backdrop_blur`. That one failing test is the point of this task.

- [ ] **Step 3: Apply the gate**

Replace the body of `resolve_material` at `src/layout/tile.rs:161-163`. Keep the existing doc comment above it unchanged and append the second paragraph:

```rust
/// The global `blur { off }` switch is applied here, so every consumer reads
/// one already-gated value. `BlurOptions` carries only `passes` and `offset`
/// and drops `off`, so a consumer that re-derived this would silently ignore
/// it — the same reason the background effect gates separately.
fn resolve_material(name: Option<&str>, options: &Options) -> Option<ResolvedMaterial> {
    let mut material = options.materials.get(name?).cloned()?;
    material.glass.backdrop_blur &= !options.blur.off;
    Some(material)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p niri backdrop_blur`
Expected: PASS, 3 tests. Also run `cargo test -p niri a_name_the_config_does_not_define` and expect PASS — the early return on an unknown name must still work.

- [ ] **Step 5: Commit**

```bash
git add src/layout/tile.rs
git commit -m "feat(materials): gate backdrop blur against the global blur switch"
```

---

### Task 3: Prepare and draw the blurred texture

**Files:**
- Modify: `src/layout/tile.rs:1275-1276` (first `prepare` pair)
- Modify: `src/layout/tile.rs:1460-1461` (second `prepare` pair)
- Modify: `src/render_helpers/material.rs:450-461` (element constructor parameters)
- Modify: `src/render_helpers/material.rs:528-538` (`MaterialRenderElement` fields)
- Modify: `src/render_helpers/material.rs:576-577` (the two `render` calls)

**Interfaces:**
- Consumes: the gated `ResolvedGlass::backdrop_blur` from Task 2, reachable at both tile sites as `material.material().glass.backdrop_blur` (`material` is bound at `src/layout/tile.rs:1269` and `:1428`).
- Produces: a `MaterialRenderElement` that samples the blurred textures when the flag is set.

**Why both sites:** `EffectBuffer::prepare(renderer, blur)` allocates the blur textures only when its flag is set (`src/render_helpers/effect_buffer.rs:148`). If `prepare` gets `false` while `render` gets `true`, `render` returns `Err`, the existing `.ok()` swallows it, and the window falls through to `plain_window_subdraw` — the glass disappears entirely, with only a `warn!`. Changing `render` without `prepare` produces that bug.

- [ ] **Step 1: Add the field to the render element**

In the `MaterialRenderElement` struct at `src/render_helpers/material.rs:535-537`, after `backdrop: Rc<RefCell<EffectBuffer>>,`:

```rust
    /// Whether to sample the blurred backdrop. Already gated against global
    /// `blur { off }` by `resolve_material`; never re-derive it here.
    backdrop_blur: bool,
```

- [ ] **Step 2: Thread it through the constructor**

In the constructor parameter list at `src/render_helpers/material.rs:458-460`, after `backdrop: Rc<RefCell<EffectBuffer>>,`:

```rust
        backdrop_blur: bool,
```

And in the `MaterialRenderElement { .. }` literal it returns, alongside the other fields:

```rust
            backdrop_blur,
```

- [ ] **Step 3: Use it at the two draw calls**

Replace `src/render_helpers/material.rs:576-577`:

```rust
        let bg_texture = self.bg.borrow_mut().render(frame, self.backdrop_blur).ok();
        let backdrop_texture = self
            .backdrop
            .borrow_mut()
            .render(frame, self.backdrop_blur)
            .ok();
```

- [ ] **Step 4: Use it at the two prepare calls**

At `src/layout/tile.rs:1275-1276`, replace the two `false` literals:

```rust
                            let backdrop_blur = material.material().glass.backdrop_blur;
                            if MaterialState::has_program(ctx.renderer)
                                && bg.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                                && backdrop.borrow_mut().prepare(ctx.renderer, backdrop_blur)
```

At `src/layout/tile.rs:1460-1461`, the same:

```rust
                    let backdrop_blur = material.material().glass.backdrop_blur;
                    if bg.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                        && backdrop.borrow_mut().prepare(ctx.renderer, backdrop_blur)
```

Then pass `backdrop_blur` as the new constructor argument at each of the two element-construction call sites that follow these blocks.

- [ ] **Step 5: Verify no literals remain on the material path**

Run:

```bash
rg -n 'prepare\(ctx\.renderer, (true|false)\)|render\(frame, (true|false)\)' src/
```

Expected: no matches in `src/layout/tile.rs` or `src/render_helpers/material.rs`. A match in either is the Task 3 bug reappearing. `src/render_helpers/xray.rs:310` passes `self.blur`, which is correct and not a literal.

- [ ] **Step 6: Build and run the full suite**

Run: `cargo test`
Expected: PASS. No behavior changes for existing materials, because Task 1 defaults the flag to `false`.

- [ ] **Step 7: Commit**

```bash
git add src/layout/tile.rs src/render_helpers/material.rs
git commit -m "feat(materials): sample the blurred backdrop when backdrop-blur is set"
```

---

### Task 4: Document the parameter

**Files:**
- Modify: `docs/materials/material-config.md:35-48` (the parameter table and the prose under it)

**Interfaces:**
- Consumes: the final parameter name and semantics from Tasks 1-3.

- [ ] **Step 1: Add the table row**

In the parameter table, after the `anisotropic-blur` row at `docs/materials/material-config.md:44`:

```markdown
| `backdrop-blur` | bool | false | true / false | — |
```

- [ ] **Step 2: Explain the parameter**

After the `jelly-flex` / `jelly-ripple` paragraph that follows the table, add:

```markdown
`backdrop-blur` makes the glass refract the blurred backdrop rather than the
sharp one, which is what produces a frosted appearance: blur and refraction
compose into one image instead of being drawn as two. It is a switch, not a
strength — the amount of blur comes from the global `blur` block's `passes` and
`offset`, shared with every other blur consumer. Setting `blur { off }`
disables it along with all other blur, regardless of this parameter.

It is unrelated to `anisotropic-blur`, which smears the refraction itself along
one axis and does not soften the backdrop.
```

- [ ] **Step 3: Commit**

```bash
git add docs/materials/material-config.md
git commit -m "docs(materials): document backdrop-blur"
```

---

### Task 5: Verify on hardware and record the result

**Files:**
- Modify: `docs/materials/2026-08-29-material-backdrop-blur-design.md:3` (status header)

**Interfaces:**
- Consumes: a built compositor carrying Tasks 1-4.

This task is manual and needs the operator. The unit suite cannot prove which texture `draw()` samples — that decision runs inside the concrete `GlesFrame` path — so the visual checks below are the only evidence that the feature works at all.

- [ ] **Step 1: Build and install, then enable the Prism debug backdrop**

The checkerboard makes softening legible in a way a photograph cannot. Enable it with `prism set debug.backdrop true`.

- [ ] **Step 2: Run the visual checks**

Add `backdrop-blur true` to the `terminal-glass` material and reload. Confirm each:

- the checkerboard seen through a terminal softens, and raising `blur { passes }` deepens it;
- **the terminal still renders with glass at all.** A `prepare`/`render` mismatch shows up precisely as the glass disappearing, so this check is what catches the Task 3 bug;
- `blur { off }` removes the softening while leaving the material otherwise intact;
- removing `backdrop-blur` restores the current appearance exactly;
- the overview shows the same treatment, confirming both textures agree.

- [ ] **Step 3: Record the outcome**

Update the design doc's status header to implemented with the commit range and the visual result, or record what failed. Commit:

```bash
git commit -m "docs(materials): record backdrop blur implementation"
```

- [ ] **Step 4: Note the deferred pixel check**

Default-preserves-v1 is verified by the DRM acceptance harness
(`docs/materials/2026-08-27-v1-drm-acceptance-design.md`), which gates on
byte-identical paired settled frames, by running its existing scenarios against
a build carrying this parameter. That run is not part of this plan; record in
the status header that it remains outstanding.
