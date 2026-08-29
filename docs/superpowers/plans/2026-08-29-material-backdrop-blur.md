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
- Preparation and drawing must read the same value. After Task 3, no call to `EffectBuffer::prepare` or `EffectBuffer::render` on a material path may pass a bool literal.
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
- Test: `niri-config/src/lib.rs`, its `mod tests` — `material.rs` has no test module, and the existing material parsing tests (`material_parameters_resolve`, `material_omitted_parameters_take_design_defaults`) already live there.

**Interfaces:**
- Produces: `Glass::backdrop_blur: Option<bool>` and `ResolvedGlass::backdrop_blur: bool` (default `false`). Task 2 reads `ResolvedGlass::backdrop_blur`.

- [x] **Step 1: Write the failing tests**

Add to `mod tests` in `niri-config/src/lib.rs`, beside the existing material tests. They use the module's existing `do_parse` and `do_parse_err` helpers (`niri-config/src/lib.rs:724` and `:731`):

```rust
    #[test]
    fn backdrop_blur_defaults_to_off() {
        assert!(!ResolvedGlass::default().backdrop_blur);
    }

    #[test]
    fn backdrop_blur_parses_and_resolves() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    backdrop-blur true
                }
            }
            "##,
        );
        assert!(parsed.materials[0].resolve().glass.backdrop_blur);
    }

    #[test]
    fn an_omitted_backdrop_blur_resolves_to_the_default() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {}
            }
            "##,
        );
        assert!(!parsed.materials[0].resolve().glass.backdrop_blur);
    }

    #[test]
    fn backdrop_blur_rejects_a_non_boolean() {
        do_parse_err(
            r##"
            material "frost" {
                glass {
                    backdrop-blur 0.5
                }
            }
            "##,
        );
    }
```

`ResolvedGlass` is already in scope in that module — it is used by the existing literal at line 912.

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri-config backdrop_blur`
Expected: FAIL — `no field 'backdrop_blur' on type 'ResolvedGlass'`. A compile error is the expected failure here.

- [x] **Step 3: Add the parse field**

In the `Glass` struct, after the `anisotropic_blur` field at `niri-config/src/material.rs:181`:

```rust
    #[knuffel(child, unwrap(argument))]
    pub backdrop_blur: Option<bool>,
```

- [x] **Step 4: Add the resolved field and its default**

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

- [x] **Step 5: Resolve it**

In `Material::resolve`, after the `anisotropic_blur` line:

```rust
                backdrop_blur: g.backdrop_blur.unwrap_or(d.backdrop_blur),
```

- [x] **Step 6: Fix the exhaustive literal the new field breaks**

`ResolvedGlass` gained a field, which breaks the exhaustive struct literal in
`material_parameters_resolve` at `niri-config/src/lib.rs:912-923`. This must be
done before the tests can compile, not after. Add, after `anisotropic_blur: 0.75,`:

```rust
                backdrop_blur: false,
```

`false` is correct there: that test's KDL does not set `backdrop-blur`.
`material_omitted_parameters_take_design_defaults` compares against
`ResolvedGlass::default()` and needs no edit.

- [x] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p niri-config backdrop_blur`
Expected: PASS, 4 tests.

- [x] **Step 8: Run the whole config suite**

Run: `cargo test -p niri-config`
Expected: PASS, including the two pre-existing material tests.

- [x] **Step 9: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs
git commit -m "feat(materials): add backdrop-blur to the glass config"
```

---

### Task 2: Gate the flag once in `resolve_material`

**Files:**
- Modify: `src/layout/tile.rs:151-163` (`resolve_material`)
- Test: `src/layout/tile.rs` (its `mod tests` at 1944)

**Interfaces:**
- Consumes: `ResolvedGlass::backdrop_blur` from Task 1; `Options::blur` (`niri_config::Blur`, field `off: bool`) and `Options::materials` from `src/layout/mod.rs:394-398`.
- Produces: `backdrop_blur_enabled(&ResolvedGlass, &niri_config::Blur) -> bool`, and a `resolve_material` whose returned `glass.backdrop_blur` is already gated against global blur. Task 3 reads that value and must not re-gate or re-derive it.

- [x] **Step 1: Write the failing tests**

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
    fn backdrop_blur_is_off_when_the_global_switch_is_off() {
        // `blur { off }` means off. BlurOptions carries only passes and offset,
        // so nothing downstream would honor `off` if it were not applied here.
        let options = options_with("frost", true, true);
        let resolved = resolve_material(Some("frost"), &options).unwrap();
        assert!(!resolved.glass.backdrop_blur);
    }

    #[test]
    fn backdrop_blur_stays_off_when_the_material_opts_out() {
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

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri backdrop_blur_`

All three names begin with `backdrop_blur_` so this filter runs all three; a
name that does not share the prefix would be silently skipped and the red step
would pass for the wrong reason.

Expected: 3 tests run. `backdrop_blur_survives_resolution_when_global_blur_is_on`
and `backdrop_blur_stays_off_when_the_material_opts_out` pass trivially, and
`backdrop_blur_is_off_when_the_global_switch_is_off` FAILS with
`assertion failed: !resolved.glass.backdrop_blur`. That one failing test is the
point of this task.

- [x] **Step 3: Apply the gate**

Replace the body of `resolve_material` at `src/layout/tile.rs:161-163`. Keep the existing doc comment above it unchanged and append the second paragraph:

```rust
/// Whether a material's backdrop blur is actually on, given the global switch.
///
/// `BlurOptions` carries only `passes` and `offset` and drops `off`, so any
/// consumer that re-derived this would silently ignore `blur { off }` — the
/// same reason the background effect gates separately at
/// `src/render_helpers/background_effect.rs:172`.
fn backdrop_blur_enabled(glass: &ResolvedGlass, blur: &niri_config::Blur) -> bool {
    glass.backdrop_blur && !blur.off
}
```

Then apply it in `resolve_material`, keeping its existing doc comment and
appending a second paragraph:

```rust
/// The global `blur { off }` switch is applied here, so every consumer
/// downstream reads one already-gated value rather than re-deriving it.
fn resolve_material(name: Option<&str>, options: &Options) -> Option<ResolvedMaterial> {
    let mut material = options.materials.get(name?).cloned()?;
    material.glass.backdrop_blur = backdrop_blur_enabled(&material.glass, &options.blur);
    Some(material)
}
```

`ResolvedGlass` is not imported in `tile.rs`; add it to the existing
`niri_config` import at `src/layout/tile.rs:5`.

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p niri backdrop_blur_`
Expected: PASS, 3 tests. Also run `cargo test -p niri a_name_the_config_does_not_define` and expect PASS — the early return on an unknown name must still work.

- [x] **Step 5: Commit**

```bash
git add src/layout/tile.rs
git commit -m "feat(materials): gate backdrop blur against the global blur switch"
```

---

### Task 3: Prepare and draw the blurred texture

**Files:**
- Modify: `src/layout/tile.rs:1275-1276` (first `prepare` pair)
- Modify: `src/layout/tile.rs:1460-1461` (second `prepare` pair)
- Modify: `src/render_helpers/material.rs:576-577` (the two `render` calls)

**Interfaces:**
- Consumes: the gated `ResolvedGlass::backdrop_blur` from Task 2. In `tile.rs` it is reachable as `material.material().glass.backdrop_blur` (`material` is bound at `src/layout/tile.rs:1269` and `:1428`). In `draw()` it is reachable as `self.glass.backdrop_blur`.
- Produces: a material that samples the blurred textures when the flag is set. No new struct field, constructor parameter, or call-site argument.

**Why no new field:** `MaterialRenderElement` already owns the resolved
parameters as `glass: ResolvedGlass` (`src/render_helpers/material.rs:526`),
populated directly from the same `MaterialState` the preparation sites read
(`glass: self.material.glass`, `material.rs:472`). Adding a separate
`backdrop_blur` bool would create a second copy of one value that could drift
from the first — exactly the divergence this design exists to prevent.

**Why both prepare sites:** `EffectBuffer::prepare(renderer, blur)` allocates
the blur textures only when its flag is set
(`src/render_helpers/effect_buffer.rs:148`). If `prepare` gets `false` while
`render` gets `true`, `render` returns `Err`, the existing `.ok()` swallows it,
and the window falls through to `plain_window_subdraw` — the glass disappears
entirely, with only a `warn!`. Changing `render` without `prepare` produces
that bug.

- [x] **Step 1: Use the existing glass field at the two draw calls**

Replace `src/render_helpers/material.rs:576-577`:

```rust
        let bg_texture = self
            .bg
            .borrow_mut()
            .render(frame, self.glass.backdrop_blur)
            .ok();
        let backdrop_texture = self
            .backdrop
            .borrow_mut()
            .render(frame, self.glass.backdrop_blur)
            .ok();
```

- [x] **Step 2: Use the same value at the two prepare sites**

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

No other call site changes: the element constructor already carries the glass.

- [x] **Step 3: Verify no literals remain on the material path**

Run:

```bash
rg -n 'prepare\(ctx\.renderer, (true|false)\)|render\(frame, (true|false)\)' src/
```

Expected: no matches in `src/layout/tile.rs` or `src/render_helpers/material.rs`.
A match in either is the preparation/draw mismatch reappearing.
`src/render_helpers/xray.rs:310` passes `self.blur`, which is correct and not a
literal.

- [x] **Step 4: Run the test suite as CI does**

Run: `cargo test --all --exclude niri-visual-tests`
Expected: PASS. No behavior changes for existing materials, because Task 1
defaults the flag to `false`.

- [x] **Step 5: Run the lint gates CI enforces**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets
```

Expected: both exit zero and this change introduces no warnings. These are
separate CI jobs (`.github/workflows/ci.yml:229` and `:214`); a red either one
blocks the merge, so fix formatting and lints before committing rather than
after. Warnings already present with a newer local toolchain are not part of
this feature.

- [x] **Step 6: Commit**

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

- [x] **Step 1: Add the table row**

In the parameter table, after the `anisotropic-blur` row at `docs/materials/material-config.md:44`:

```markdown
| `backdrop-blur` | bool | false | true / false | — |
```

- [x] **Step 2: Explain the parameter**

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

- [x] **Step 3: Commit**

```bash
git add docs/materials/material-config.md
git commit -m "docs(materials): document backdrop-blur"
```

---

### Task 5: Verify on hardware and record the result

**Files:**
- Modify: `docs/materials/2026-08-29-material-backdrop-blur-design.md:3` (status header)
- Create: `docs/materials/2026-08-29-material-backdrop-blur-evidence.md` (retained evidence manifest and reproduction record)
- Modify: `docs/superpowers/plans/2026-08-29-material-backdrop-blur.md` (this plan's checkboxes)

**Interfaces:**
- Consumes: a build carrying Tasks 1-4.

This task is manual and needs the operator. The unit suite cannot prove which
texture `draw()` samples — that decision runs inside the concrete `GlesFrame`
path — so these checks are the only evidence the feature works at all.

**Packaging note:** `packaging/arch/PKGBUILD` does not exist on this branch. It
lives on `v1-post-acceptance-planning` and pins
`#commit=7f6e69c303ab2e72970679b8068763cb42b1f375`, so a package built from it
would **not** contain this work. Installing this to the daily driver is a
separate rollout decision requiring a PKGBUILD pin update and explicit operator
approval; it is out of scope here. Verify from a local build instead.

- [x] **Step 1: Pin and record what is being tested**

```bash
git rev-parse HEAD
cargo build --release --features default
target_dir=$(cargo metadata --no-deps --format-version 1 | jq -r .target_directory)
sha256sum "$target_dir/release/niri"
```

This checkout may use a shared Cargo target directory, so resolve it through
Cargo rather than assuming it lives under the worktree. Record the source
commit, successful build, and binary hash in the status header written in Step
4. A visual result that does not name the binary it came from cannot authorize
an implemented status.

- [x] **Step 2: Run the checks**

Use a nested niri on the dedicated headless host, per this project's convention
that live nested runs never touch the desktop session:

```bash
weston_unit=niri-material-backdrop-blur-weston
systemd-run --user --unit="$weston_unit" --collect \
  --setenv=XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
  weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
  --width=1280 --height=720 --socket=backdrop-blur
trap 'systemctl --user stop "$weston_unit.service" >/dev/null 2>&1 || true' EXIT HUP INT TERM
```

Run `"$target_dir/release/niri"` against that socket with a config defining a
material that sets `backdrop-blur true`, a Background-layer surface carrying
high-contrast content, and a terminal using the material.

Confirm each:

- the backdrop seen through the terminal softens, and raising `blur { passes }`
  deepens it;
- **the terminal still renders with glass at all.** A `prepare`/`render`
  mismatch shows up precisely as the glass disappearing into the plain-window
  fallback, so this check is what catches that bug;
- `blur { off }` removes the softening while leaving the material otherwise
  intact;
- removing `backdrop-blur` restores the current appearance exactly;
- the overview shows the same treatment, confirming both textures agree.

Capture a frame for each state so the result is evidence rather than
recollection.

- [x] **Step 3: Restore the operator's environment**

Required whether the checks passed or failed, and before recording anything:

```bash
systemctl --user stop "$weston_unit.service"
```

If any check was instead performed on the daily-driver session, undo it there
too — `backdrop-blur` is not a Prism parameter, so it can only have been
hand-added to the generated fragment:

```bash
prism set debug.backdrop false
prism apply niri          # regenerates prism.kdl, dropping any hand edit
git -C ~/d/dotfiles diff --quiet -- niri/config.kdl   # any blur{} edits reverted
```

Expected: the last command exits zero, `niri msg layers` shows no
`prism-debug-backdrop`, and `prism doctor` reports ok.

- [x] **Step 4: Update the records**

All edits happen here, before anything is staged:

1. In `docs/materials/2026-08-29-material-backdrop-blur-design.md:3`, replace the
   status header with the outcome — implemented, naming the commit, the binary
   sha256 from Step 1, and the visual result; or what failed. State explicitly
   that the DRM acceptance run below remains outstanding, so the doc does not
   claim pixel verification it has not had.
2. Record the deferred pixel check in that same header: default-preserves-v1 is
   verified by the DRM acceptance harness
   (`docs/materials/2026-08-27-v1-drm-acceptance-design.md`), which gates on
   byte-identical paired settled frames, by running its existing scenarios
   against a build carrying this parameter. That run is not part of this plan.
3. Confirm the design's Goals section still says at most two blur computations
   per changed source per render target, matching Interactions — not "one blur
   per output per frame". (Corrected 2026-08-29; verify it stayed corrected.)
4. In this plan, tick every completed checkbox **including this step and Step 5**,
   which are about to be completed by the act of committing. Leaving them
   unchecked commits a plan that understates its own execution.
5. Record the retained capture and input manifests, measurement procedure, and
   cleanup proof in `docs/materials/2026-08-29-material-backdrop-blur-evidence.md`.

- [x] **Step 5: Stage the records and commit**

```bash
git add docs/materials/2026-08-29-material-backdrop-blur-design.md \
        docs/materials/2026-08-29-material-backdrop-blur-evidence.md \
        docs/superpowers/plans/2026-08-29-material-backdrop-blur.md
git commit -m "docs(materials): record backdrop blur implementation"
```

If a box was missed, fix it and `git commit --amend` rather than leaving the
recorded plan inconsistent with what was done.
