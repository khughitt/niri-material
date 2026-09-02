# Glass Noise and Saturation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make glass with effective `backdrop-blur` inherit the global blur
block's noise and saturation inside the existing material pass, while keeping
opt-out, global-off, opaque-window, and Prism output behavior unchanged.

**Architecture:** Resolve one render-side `MaterialRenderConfig` containing
the public `ResolvedMaterial` plus two effective scalar values. Store that
wrapper in `MaterialState`, include it in in-place damage decisions, and apply
the scalars after glass optics in `material.frag`; no public grammar, cache,
texture, or render-pass change is involved.

**Tech Stack:** Rust, smithay GLES2, GLSL ES 1.00, Cargo, Node.js test runner,
YAML, Weston headless GL, ImageMagick.

**Spec:** `docs/specs/2026-09-02-material-noise-saturation-design.md`

## Global Constraints

- Effective backdrop blur remains exactly
  `glass.backdrop-blur && !blur.off`.
- Effective values are `(noise = 0, saturation = 1)` when that gate is false;
  otherwise they are the global `blur.noise` and `blur.saturation` values.
- Apply saturation, then screen-space noise, after transmission, specular, and
  linear-to-sRGB conversion but before slab coverage and window composition.
- Opaque application pixels remain untouched. Per-window
  `background-effect { noise; saturation }` remains independent.
- Mirror `postprocess.frag`'s neutral branches and seed material noise with
  `gl_FragCoord.xy + vec2(47.0, 113.0)`.
- Do not change the public material grammar, `EffectBuffer`, texture caches,
  blur/postprocess shaders, sample counts, render passes, or dependencies.
- Prism gains no new controls and no renderer change. Its generated niri KDL
  must remain byte-identical; only the `glass.backdropBlur` description and an
  explicit description assertion change.
- Land the verified Prism commit in the requested `~/d/prism` checkout before
  closing the native parent task.
- Do not add compatibility aliases, general postprocess abstractions, shared
  GLSL helpers, or a performance benchmark.
- Use conventional commits, stage named paths, and add no AI attribution.
- Use a writable Cargo target, for example
  `CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target`.

## File Structure

| File | Responsibility | Task |
| --- | --- | --- |
| `src/layout/tile.rs` | Resolve the effective gate and inherited pair | 1 |
| `src/render_helpers/material.rs` | Own the render wrapper, update damage, and bind uniforms | 1 |
| `src/render_helpers/shaders/material.frag` | Apply saturation and noise at the approved composition point | 1 |
| `src/render_helpers/shaders/mod.rs` | Register the two material uniforms | 1 |
| `docs/materials/material-config.md` | Document inheritance and override independence | 2 |
| `docs/materials/2026-09-02-material-noise-saturation-evidence.md` | Record automated and nested-GLES acceptance | 2 |
| `docs/specs/2026-09-02-material-noise-saturation-design.md` | Record landed status and evidence | 2 |
| Prism `defs/glass.yaml` | Clarify the existing backdrop-blur contract | 2 |
| Prism `test/glass-defs.test.js` | Pin the description contract | 2 |
| `tasks/*.md` via `tasks` CLI | Track and close the two deliverables | 1, 2 |

---

### Task 1: Compose inherited postprocess in material rendering

**Files:**

- Modify: `src/layout/tile.rs:153-181, 1960-2014`
- Modify: `src/render_helpers/material.rs:396-546, 620-657, 709-1073`
- Modify: `src/render_helpers/shaders/material.frag:13-40, 383-409`
- Modify: `src/render_helpers/shaders/mod.rs:153-192`
- Test: inline modules in `src/layout/tile.rs` and
  `src/render_helpers/material.rs`

**Interfaces:**

- Produces in `material.rs`:
  `MaterialRenderConfig { material: ResolvedMaterial, noise: f32,
  saturation: f32 }`.
- Produces in `tile.rs`:
  `resolve_material(Option<&str>, &Options) -> Option<MaterialRenderConfig>`.
- Changes `MaterialState::new` and `apply_resolved` to consume the wrapper;
  keeps `MaterialState::material() -> &ResolvedMaterial` for existing tile
  call sites.
- Produces shader uniforms `mat_noise: float` and
  `mat_saturation: float`.

- [ ] **Step 1: Write the failing policy and commit tests**

Start the plan-step task before editing:

```bash
tasks start material-0b6d06
```

In `tile.rs`, add one table-driven policy test and update the existing
resolution assertions to read through `.material.glass`:

```rust
    #[test]
    fn material_postprocess_follows_effective_backdrop_blur() {
        for (material_blur, global_off, expected) in [
            (true, false, (0.07, 0.8)),
            (false, false, (0.0, 1.0)),
            (true, true, (0.0, 1.0)),
        ] {
            let mut options = options_with("frost", material_blur, global_off);
            options.blur.noise = 0.07;
            options.blur.saturation = 0.8;

            let resolved = resolve_material(Some("frost"), &options).unwrap();
            assert_eq!((resolved.noise, resolved.saturation), expected);
        }
    }
```

In `material.rs`, replace the test helper with a neutral render wrapper and
mechanically update existing `MaterialState::new`/`apply_resolved` test calls
to use it:

```rust
    fn render_config(name: &str) -> MaterialRenderConfig {
        MaterialRenderConfig {
            material: ResolvedMaterial {
                name: String::from(name),
                glass: ResolvedGlass::default(),
            },
            noise: 0.,
            saturation: 1.,
        }
    }
```

Add a focused test that isolates both inherited fields from glass changes:

```rust
    #[test]
    fn postprocess_change_advances_the_commit_in_place() {
        let mut slot = Some(MaterialState::new(render_config("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();
        let initial_commit = slot.as_ref().unwrap().commit.get();

        let mut changed = render_config("frost");
        changed.noise = 0.04;
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        let noise_commit = slot.as_ref().unwrap().commit.get();
        assert_ne!(noise_commit, initial_commit);

        changed.saturation = 0.8;
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        assert_ne!(slot.as_ref().unwrap().commit.get(), noise_commit);
    }
```

In the existing `parameter_change_advances_the_commit_in_place` test, mutate
`changed.material.glass.roughness` and assert through
`state.material().glass.roughness`; keep all identity and per-target commit
assertions.

- [ ] **Step 2: Run the focused tests and verify they fail for the missing interface**

Run:

```bash
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri material_postprocess_follows_effective_backdrop_blur -- --nocapture
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri postprocess_change_advances_the_commit_in_place -- --nocapture
```

Expected: compilation fails because `resolve_material` still returns
`ResolvedMaterial` and `MaterialRenderConfig` does not exist. A pre-existing
unrelated failure is not an acceptable red phase.

- [ ] **Step 3: Add the render wrapper and resolve the effective pair once**

In `material.rs`, add immediately before `MaterialState`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialRenderConfig {
    pub material: ResolvedMaterial,
    pub noise: f32,
    pub saturation: f32,
}
```

Store it as `config: MaterialRenderConfig` in `MaterialState`. Keep existing
call sites stable with:

```rust
    pub fn material(&self) -> &ResolvedMaterial {
        &self.config.material
    }
```

Change `MaterialState::new` and `apply_resolved` to accept the wrapper. The
same-name branch must compare and replace the whole wrapper so present and
future render-only fields cannot evade damage:

```rust
pub fn apply_resolved(
    slot: &mut Option<MaterialState>,
    resolved: Option<&MaterialRenderConfig>,
) -> bool {
    match (slot.as_mut(), resolved) {
        (None, None) => false,
        (Some(_), None) => {
            *slot = None;
            true
        }
        (Some(state), Some(resolved))
            if state.config.material.name == resolved.material.name =>
        {
            if &state.config != resolved {
                state.config = resolved.clone();
                state.bump();
            }
            false
        }
        (_, Some(resolved)) => {
            *slot = Some(MaterialState::new(resolved.clone()));
            true
        }
    }
}
```

Copy `config.material.glass`, `config.noise`, and `config.saturation` onto
each `MaterialRenderElement`. In `tile.rs`, import `MaterialRenderConfig` and
replace `resolve_material`'s return construction with:

```rust
fn resolve_material(name: Option<&str>, options: &Options) -> Option<MaterialRenderConfig> {
    let mut material = options.materials.get(name?).cloned()?;
    let backdrop_blur = backdrop_blur_enabled(&material.glass, &options.blur);
    material.glass.backdrop_blur = backdrop_blur;
    let (noise, saturation) = if backdrop_blur {
        (options.blur.noise as f32, options.blur.saturation as f32)
    } else {
        (0., 1.)
    };
    Some(MaterialRenderConfig {
        material,
        noise,
        saturation,
    })
}
```

- [ ] **Step 4: Bind and apply the two shader values**

Add the two uniforms to `MaterialRenderElement::draw` immediately after the
prefilter mix uniforms:

```rust
            Uniform::new("mat_noise", self.noise),
            Uniform::new("mat_saturation", self.saturation),
```

Register both as `_1f` uniforms in `Shaders::compile` in the same order. In
`material.frag`, declare them beside the prefilter uniforms, then replace the
single `glassed = ...` assignment with:

```glsl
        vec3 glassColor = linearToSrgb(transmitted + specular);
        if (mat_saturation != 1.0) {
            const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
            glassColor = mix(vec3(dot(glassColor, luma)), glassColor, mat_saturation);
        }
        if (mat_noise > 0.0) {
            vec2 noiseSeed = gl_FragCoord.xy + vec2(47.0, 113.0);
            glassColor += (hash12(noiseSeed) - 0.5) * mat_noise;
        }
        glassed = vec4(glassColor, 1.0) * coverage;
```

Do not move the final `win + (1.0 - win.a) * glassed` composition and do not
touch `postprocess.frag`.

- [ ] **Step 5: Run focused, shader, formatting, and workspace checks**

Run:

```bash
glslangValidator -S frag src/render_helpers/shaders/material.frag
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri material_postprocess_follows_effective_backdrop_blur -- --nocapture
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri postprocess_change_advances_the_commit_in_place -- --nocapture
cargo fmt --all -- --check
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test --workspace --all-targets
git diff --check
tasks check
```

Expected: every command exits zero. Existing warnings may remain, but the
change introduces no warning. Inspect the diff to confirm there are still
five material samplers and no changed file under `niri-config`,
`effect_buffer`, `blur`, or `postprocess.frag`.

- [ ] **Step 6: Commit the native implementation**

```bash
tasks done material-0b6d06 \
  "Inherited the effective global noise/saturation pair, damaged in-place edits, and applied both values in the existing material shader pass."
tasks check
git add src/layout/tile.rs src/render_helpers/material.rs \
  src/render_helpers/shaders/material.frag src/render_helpers/shaders/mod.rs \
  tasks/material-0b6d06.md
git commit -m "feat(material): compose global glass postprocess"
```

### Task 2: Document and verify cross-repository delivery

**Files:**

- Modify: `docs/materials/material-config.md:56-75`
- Create: `docs/materials/2026-09-02-material-noise-saturation-evidence.md`
- Modify: `docs/specs/2026-09-02-material-noise-saturation-design.md:3`
- Verify/correct only if stale: `README.md`, user-facing files under `docs/`
- Modify in Prism: `defs/glass.yaml`
- Test in Prism: `test/glass-defs.test.js`
- Modify through CLI: `tasks/*.md`

**Interfaces:**

- Consumes: Task 1's complete native implementation and its effective gate.
- Produces: a user-facing native contract, an explicit Prism description
  contract, retained acceptance evidence, landed design status, and a closed
  task chain.
- Does not change Prism's render function, manifest, generated KDL fixtures,
  or public control surface.

- [ ] **Step 1: Add the failing Prism description assertion**

From the native worktree, start the final delivery task:

```bash
tasks start material-7b9ddb
```

In a fresh Prism worktree created with `superpowers:using-git-worktrees`, add:

```javascript
test('backdrop blur documents inherited global effects', () => {
  const description = loadDefs(defsDir()).get('glass.backdropBlur').description;

  assert.match(description, /global blur block/);
  assert.match(description, /saturation/);
  assert.match(description, /noise/);
});
```

Run:

```bash
node --test test/glass-defs.test.js
```

Expected: FAIL because the current description mentions only blur strength.

- [ ] **Step 2: Clarify Prism without changing generated KDL**

Set the `glass.backdropBlur` description in `defs/glass.yaml` to:

```yaml
  description: Refract the blurred backdrop for a frosted look; niri's global blur block supplies blur strength, saturation, and noise
```

Run from the Prism repository:

```bash
node --test test/glass-defs.test.js
npm test
git diff --check
git diff --exit-code -- integrations/niri/render.js integrations/niri/manifest.yaml \
  test/niri-render.test.js test/niri-apply.test.js
```

Expected: all tests pass, and the last command confirms the renderer,
manifest, and exact-KDL contracts are untouched. Commit only the definition
and test, then land that commit in the requested Prism checkout:

```bash
git add defs/glass.yaml test/glass-defs.test.js
git commit -m "docs(glass): clarify backdrop blur inheritance"
```

- [ ] **Step 3: Run the nested GLES acceptance matrix**

First build a fresh implementation binary without overwriting the recorded
pre-change binary:

```bash
printf '%s  %s\n' \
  8edc614ce17217ba55af3f89be0e83437c708bcbe32820df34f60a5b8ad8792d \
  /mnt/ssd3/tmp/niri-material-cad932-target/debug/niri | sha256sum -c -
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo build --workspace --all-targets
CAD932_ARTIFACT_DIR=$(mktemp -d /mnt/ssd3/tmp/material-cad932-acceptance.XXXXXX)
export CAD932_ARTIFACT_DIR
sha256sum /mnt/ssd3/tmp/niri-material-cad932-impl-target/debug/niri \
  > "$CAD932_ARTIFACT_DIR/implementation-binary.sha256"
```

Use the dedicated 1280×720 Weston headless-GL fixture and cleanup discipline
from `docs/materials/2026-08-29-material-backdrop-blur-evidence.md`: never run
`niri --session`, record every owned PID/socket, stop only owned processes,
and prove the socket is absent afterward. Use the same red/green color-bars
wallpaper, a 704×640 kitty at `(40, 40)`, fixed capture timing after all
animations settle, and one unchanged scene for every matched pair.

Capture these fixtures with both noise and saturation set explicitly unless
the fixture is testing one of them:

- pre-change and implementation binaries with `backdrop-blur false`,
  `noise 0`, `saturation 1`;
- effective blur with `noise 0`, `saturation 0`;
- effective blur with `noise 0`, `saturation 1`, then `noise 0.08`,
  `saturation 1`;
- an opaque kitty with neutral values, then `noise 0.08`, `saturation 0`;
- `blur { off }` with material opt-in, then global blur on with material
  opt-out.

Use these exact checks after naming the captures accordingly:

```bash
magick compare -metric AE \
  "$CAD932_ARTIFACT_DIR/default-off-before.png" \
  "$CAD932_ARTIFACT_DIR/default-off-after.png" null:
magick "$CAD932_ARTIFACT_DIR/saturation-0.png" \
  -crop 200x400+100+160 +repage -channel R -separate \
  "$CAD932_ARTIFACT_DIR/saturation-r.png"
magick "$CAD932_ARTIFACT_DIR/saturation-0.png" \
  -crop 200x400+100+160 +repage -channel G -separate \
  "$CAD932_ARTIFACT_DIR/saturation-g.png"
magick "$CAD932_ARTIFACT_DIR/saturation-0.png" \
  -crop 200x400+100+160 +repage -channel B -separate \
  "$CAD932_ARTIFACT_DIR/saturation-b.png"
magick compare -metric AE "$CAD932_ARTIFACT_DIR/saturation-r.png" \
  "$CAD932_ARTIFACT_DIR/saturation-g.png" null:
magick compare -metric AE "$CAD932_ARTIFACT_DIR/saturation-r.png" \
  "$CAD932_ARTIFACT_DIR/saturation-b.png" null:
magick compare -metric RMSE \
  "$CAD932_ARTIFACT_DIR/noise-0.png" \
  "$CAD932_ARTIFACT_DIR/noise-008.png" null:
magick "$CAD932_ARTIFACT_DIR/noise-0.png" \
  -crop 200x400+100+160 +repage -colorspace Gray \
  -format 'noise_0_sd=%[fx:standard_deviation]\n' info:
magick "$CAD932_ARTIFACT_DIR/noise-008.png" \
  -crop 200x400+100+160 +repage -colorspace Gray \
  -format 'noise_008_sd=%[fx:standard_deviation]\n' info:
magick "$CAD932_ARTIFACT_DIR/opaque-neutral.png" \
  -crop 400x300+160+200 +repage \
  "$CAD932_ARTIFACT_DIR/opaque-neutral-roi.png"
magick "$CAD932_ARTIFACT_DIR/opaque-active.png" \
  -crop 400x300+160+200 +repage \
  "$CAD932_ARTIFACT_DIR/opaque-active-roi.png"
magick compare -metric AE \
  "$CAD932_ARTIFACT_DIR/opaque-neutral-roi.png" \
  "$CAD932_ARTIFACT_DIR/opaque-active-roi.png" null:
magick compare -metric AE \
  "$CAD932_ARTIFACT_DIR/blur-off.png" \
  "$CAD932_ARTIFACT_DIR/material-opt-out.png" null:
if rg -n "material.*(error|fallback)|error compiling material shader|panic" \
  "$CAD932_ARTIFACT_DIR"/*.log; then
  exit 1
fi
```

Acceptance is: every AE metric is `0`; both saturation channel comparisons
are `0`; noise RMSE is nonzero and the grayscale standard deviation of the
same solid-color ROI increases; and the final `rg` finds nothing. Record the
Weston renderer/version, GLES version, source commit, binary hash, exact KDL,
capture hashes, ROI statistics, commands, and cleanup proof.

- [ ] **Step 4: Update native documentation and landed status**

After the existing `backdrop-blur` paragraph in
`docs/materials/material-config.md`, add:

```markdown
When `backdrop-blur` is effective, glass also inherits `noise` and
`saturation` from the global `blur` block. The material applies saturation
then screen-space noise after its glass optics. `blur { off }` or material
opt-out makes both values neutral. Per-window `background-effect` overrides
remain independent and do not alter the material.
```

Create the evidence document with the complete results from Step 3. Change
the design status from “implementation not started” to the exact native and
Prism commits plus the evidence link. Verify outward drift:

```bash
rg -n "noise|saturation|backdrop-blur|implementation not started|independent blur defects" \
  README.md docs
```

Correct only current user-facing claims made stale by delivery. Preserve
historical measurements as history, including the already-corrected follow-up
in `docs/materials/2026-08-29-material-backdrop-blur-design.md`.

- [ ] **Step 5: Re-run final checks, close tasks, and commit acceptance**

Run fresh verification from the native worktree:

```bash
glslangValidator -S frag src/render_helpers/shaders/material.frag
cargo fmt --all -- --check
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test --workspace --all-targets
tasks check
git diff --check
```

Verify the Prism commit is an ancestor of the requested checkout before
closing the parent. Task 1's record was closed with its implementation commit;
close Task 2 and then `material-cad932` in dependency order without editing
task markdown by hand:

```bash
tasks done material-7b9ddb \
  "Landed the Prism contract update, documented inheritance, and recorded passing automated and nested GLES acceptance."
tasks done material-cad932 \
  "Glass now inherits global noise and saturation under effective backdrop blur; native and Prism delivery and acceptance are recorded."
tasks check
git add docs/materials/material-config.md \
  docs/materials/2026-09-02-material-noise-saturation-evidence.md \
  docs/specs/2026-09-02-material-noise-saturation-design.md \
  tasks/material-7b9ddb.md tasks/material-cad932.md
git commit -m "docs(material): record glass postprocess acceptance"
```

Expected final state: both repositories are clean; all automated and nested
GLES gates pass; the native design and task record name the landed commits;
and Prism's generated KDL is unchanged.
