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

- [x] **Step 1: Write the failing policy and commit tests**

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

- [x] **Step 2: Run the focused tests and verify they fail for the missing interface**

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

- [x] **Step 3: Add the render wrapper and resolve the effective pair once**

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

- [x] **Step 4: Bind and apply the two shader values**

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

- [x] **Step 5: Run focused, shader, formatting, and workspace checks**

Run:

```bash
glslangValidator -S frag src/render_helpers/shaders/material.frag
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri material_postprocess_follows_effective_backdrop_blur -- --nocapture
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test -p niri postprocess_change_advances_the_commit_in_place -- --nocapture
rustfmt --edition 2021 --check src/layout/tile.rs \
  src/render_helpers/material.rs src/render_helpers/shaders/mod.rs
CARGO_TARGET_DIR=/mnt/ssd3/tmp/niri-material-cad932-impl-target \
  cargo test --workspace --all-targets
git diff --check
tasks check
```

Expected: every command exits zero. Existing warnings may remain, but the
change introduces no warning. Inspect the diff to confirm there are still
five material samplers and no changed file under `niri-config`,
`effect_buffer`, `blur`, or `postprocess.frag`.

- [x] **Step 6: Commit the native implementation**

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

- [x] **Step 1: Add the failing Prism description assertion**

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

- [x] **Step 2: Clarify Prism without changing generated KDL**

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
and test:

```bash
git add defs/glass.yaml test/glass-defs.test.js
git commit -m "docs(glass): clarify backdrop blur inheritance"
```

Name the worktree branch `docs/material-cad932`. After the commit passes, land
it in the requested clean `~/d/prism` checkout by fast-forward only:

```bash
git -C ~/d/prism status --short --branch
git -C ~/d/prism merge --ff-only docs/material-cad932
git -C ~/d/prism status --short --branch
```

Expected: the first status reports clean `main`, the merge fast-forwards, and
the final status remains clean. Do not cherry-pick or create a merge commit.

- [x] **Step 3: Run the nested GLES acceptance matrix**

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

Use the dedicated 1280×720 Weston headless-GL lifecycle and cleanup discipline
from `docs/materials/2026-08-29-material-backdrop-blur-evidence.md`: never run
`niri --session`, record every owned PID/socket, stop only owned processes,
and prove the socket is absent afterward.

Do **not** reuse `/mnt/ssd3/tmp/material-cad932-config.kdl`: it has no material
and exercises `background-effect { xray true; blur true }`, so it cannot prove
this change. Create each runtime KDL from this exact material fixture instead:

```kdl
prefer-no-csd

layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.6; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}

animations { off; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }

blur {
    passes 1
    offset 8
    noise 0
    saturation 1
}

material "cad932-probe" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        roughness 0
        backdrop-blur true
        jelly-flex 0
        jelly-ripple 0
        bevel 12
        offset-x 6
        offset-y 6
    }
}

window-rule {
    match app-id="^material-cad932-probe$"
    material "cad932-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
```

The per-window block is Prism's exact inert contract: it contains no `xray`
and its explicit values make `BackgroundEffect::Options::is_visible` false.
Validate every saved KDL with the binary used for its capture, and reject any
runtime KDL containing `xray true` or a bare `blur true` child:

```bash
baseline_binary=/mnt/ssd3/tmp/niri-material-cad932-target/debug/niri
implementation_binary=/mnt/ssd3/tmp/niri-material-cad932-impl-target/debug/niri
"$baseline_binary" validate \
  --config "$CAD932_ARTIFACT_DIR/default-off-before.kdl"
for capture in default-off-after saturation-0 noise-0 noise-008 \
  opaque-neutral opaque-active blur-off material-opt-out; do
  "$implementation_binary" validate \
    --config "$CAD932_ARTIFACT_DIR/$capture.kdl"
done
for capture_kdl in "$CAD932_ARTIFACT_DIR"/*.kdl; do
  if rg -n '^\s*(xray|blur) true\s*$' "$capture_kdl"; then
    exit 1
  fi
done
```

Use this exact value matrix; “global off” means adding the child `off` inside
the `blur` block, and all other rows omit that child:

| Capture | Binary | `backdrop-blur` | global off | global noise | global saturation | Client |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `default-off-before` | recorded pre-change | false | no | 0 | 1 | transparent |
| `default-off-after` | implementation | false | no | 0 | 1 | transparent |
| `saturation-0` | implementation | true | no | 0 | 0 | transparent |
| `noise-0` | implementation | true | no | 0 | 1 | transparent |
| `noise-008` | implementation | true | no | 0.08 | 1 | transparent |
| `opaque-neutral` | implementation | true | no | 0 | 1 | opaque |
| `opaque-active` | implementation | true | no | 0.08 | 0 | opaque |
| `blur-off` | implementation | true | yes | 0.08 | 0 | transparent |
| `material-opt-out` | implementation | false | no | 0.08 | 0 | transparent |

For each row, retain its complete KDL beside the PNG. Change only the table's
four KDL values and the named client opacity between rows; keep the same
process, output, position, wallpaper, and capture timing for each matched
implementation pair.

Pin the color-bars wallpaper by hash and content. It is three 1280×720 thirds:
red `(255,32,32)` through x=425, green `(32,255,32)` through x=852, then blue
`(32,32,255)`:

```bash
printf '%s  %s\n' \
  fa81fd9535e1a4873b7c36b759d564418b12a341d13f4cdcaf08687d67a5370a \
  /mnt/ssd3/tmp/material-cad932-color-bars.png | sha256sum -c -
```

Pin the probe using the blank client pattern from
`docs/materials/plans/2026-08-24-v1-parity.md:430-441`. The transparent
variant is:

```bash
env -u DISPLAY XDG_RUNTIME_DIR="$runtime_dir" \
  WAYLAND_DISPLAY="$nested_display" \
  kitty --config NONE --class material-cad932-probe \
  --title "material cad932 probe" \
  -o background_opacity=0 -o cursor_blink_interval=0 \
  sh -c 'printf "\033[?25l"; exec sleep 600'
```

The `printf` hides the cursor and emits no printable character; `sleep` keeps
the otherwise blank client mapped. The opaque variant changes only
`background_opacity=0` to `background_opacity=1`. Require exactly one nested
window with app ID `material-cad932-probe`, place its 704×640 surface at
`(40,40)`, and capture only after it and the wallpaper have remained unchanged
for the fixed settle interval.

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
magick "$CAD932_ARTIFACT_DIR/noise-0.png" \
  -crop 200x400+100+160 +repage \
  "$CAD932_ARTIFACT_DIR/noise-0-roi.png"
magick "$CAD932_ARTIFACT_DIR/noise-008.png" \
  -crop 200x400+100+160 +repage \
  "$CAD932_ARTIFACT_DIR/noise-008-roi.png"
set +e
noise_rmse=$(magick compare -metric RMSE \
  "$CAD932_ARTIFACT_DIR/noise-0-roi.png" \
  "$CAD932_ARTIFACT_DIR/noise-008-roi.png" null: 2>&1)
noise_compare_status=$?
set -e
test "$noise_compare_status" -eq 1
printf 'noise_roi_rmse=%s\n' "$noise_rmse"
noise_0_sd=$(magick "$CAD932_ARTIFACT_DIR/noise-0-roi.png" \
  -colorspace Gray -format '%[fx:standard_deviation]' info:)
noise_008_sd=$(magick "$CAD932_ARTIFACT_DIR/noise-008-roi.png" \
  -colorspace Gray -format '%[fx:standard_deviation]' info:)
printf 'noise_0_sd=%s\nnoise_008_sd=%s\n' "$noise_0_sd" "$noise_008_sd"
awk -v quiet="$noise_0_sd" -v active="$noise_008_sd" \
  'BEGIN { exit !(active > quiet) }'
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
test -s "$CAD932_ARTIFACT_DIR/niri.log"
set +e
log_hits=$(rg -n --glob '*.log' \
  "material.*(error|fallback)|error compiling material shader|panic" \
  "$CAD932_ARTIFACT_DIR")
log_status=$?
set -e
case "$log_status" in
  0) printf '%s\n' "$log_hits"; exit 1 ;;
  1) ;;
  *) exit "$log_status" ;;
esac
```

Run the metric block under `set -e`; only the noise `compare` is bracketed by
`set +e` because ImageMagick returns status 1 for the required non-identical
pair while writing its metric to stderr. Acceptance is: every AE metric is
`0`; both saturation channel comparisons are `0`; noise ROI RMSE reports a
nonzero value with status 1 and its grayscale standard deviation increases;
and the final log gate finds nothing. Record the
Weston renderer/version, GLES version, source commit, binary hash, exact KDL,
capture hashes, ROI statistics, commands, and cleanup proof.

- [x] **Step 4: Update native documentation and landed status**

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

- [x] **Step 5: Re-run final checks, close tasks, and commit acceptance**

Run fresh verification from the native worktree:

```bash
glslangValidator -S frag src/render_helpers/shaders/material.frag
rustfmt --edition 2021 --check src/layout/tile.rs \
  src/render_helpers/material.rs src/render_helpers/shaders/mod.rs
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
