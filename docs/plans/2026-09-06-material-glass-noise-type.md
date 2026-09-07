# Glass Noise Type Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the glass `noise` node an optional `type` property with three
grains (`white`, `fine`, `lightness`), rendered by the material shader, so
Prism can offer a finer grain than today's white noise.

**Architecture:** The config crate parses `noise <amount> type=<type>` into a
`Noise` struct shaped like `Distortion`, resolves the type into
`ResolvedGlass.noise_type` with `White` as the omitted default, and leaves the
amount's inheritance rule untouched. The resolved glass already travels inside
`MaterialRenderConfig` to the render element, so the type reaches the shader
as one new float uniform read from `ResolvedGlass`, and the existing
config-inequality commit bump already redraws on a type change. The shader's
noise branch becomes a three-way switch; the `white` branch is the existing
code unchanged.

**Tech Stack:** Rust, knuffel (KDL), smithay GLES2, GLSL ES 1.00, Cargo,
bash, Weston headless GL, ImageMagick 7.

**Spec:** `docs/specs/2026-09-06-material-glass-noise-type-design.md`

## Global Constraints

- An omitted `type` and an explicit `type=white` render byte-identical to
  today's build; the `white` shader branch is the existing two lines,
  character for character.
- `fine` is `centre - mean(8 neighbours)` scaled by `sqrt(8/9)` so its
  standard deviation matches `white` at the same amount; it is achromatic and
  added in sRGB-encoded space.
- `lightness` applies the `fine` value to Oklab L of the post-saturation
  colour; chroma and hue hold except where the result leaves the sRGB gamut
  and clamps.
- The amount keeps its `Option<f64>` and its inheritance rule from
  `docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md`;
  the type has no inheritance.
- Discrete shader uniforms are floats in this codebase (`mat_samples`); the
  type uniform is `float mat_noise_type` with 0, 1, 2.
- `just check` and `just test` are the gates. Bare `cargo test` does not
  enforce suite selection, timing records or the commit checks.
- The nested GLES smoke runs on the headless Weston unit, never the desktop
  session.
- Conventional commits, no AI attribution trailers.

---

### Task 1: Parse and resolve the noise type property

**Files:**
- Modify: `niri-config/src/material.rs` (the `Glass` struct at ~line 406,
  `ResolvedGlass` at ~448, `Material::resolve` at ~540, and the enum block
  with `response_from_str!` at ~205-260)
- Modify: `niri-config/src/lib.rs:58-61` (the `pub use crate::material::{…}`
  list) and its test module after
  `glass_saturation_rejects_values_outside_zero_and_three` (~line 1350)
- Modify: `docs/materials/material-config.md` (the parameter table at ~line
  49 and the noise paragraph at ~line 87)

**Interfaces:**
- Consumes: `FloatOrInt<0, 1>`, `Distortion` (the amount-plus-property
  idiom), the `response_from_str!` macro.
- Produces:
  - `pub enum NoiseType { White, Fine, Lightness }` in `niri_config::material`,
    re-exported at the crate root, `Copy`, `Default` (`White`), `FromStr`
    from `"white" | "fine" | "lightness"`, `#[repr(u8)]` with 0, 1, 2.
  - `pub struct Noise { pub amount: FloatOrInt<0, 1>, pub kind: Option<NoiseType> }`.
  - `Glass.noise: Option<Noise>`.
  - `ResolvedGlass.noise_type: NoiseType`, default `White`; `ResolvedGlass.noise`
    stays `Option<f64>`.

- [ ] **Step 1: Write the failing parse tests**

Add to the test module of `niri-config/src/lib.rs`, directly after
`glass_saturation_rejects_values_outside_zero_and_three`:

```rust
    #[test]
    fn glass_noise_type_parses_each_value() {
        for (written, expected) in [
            ("white", NoiseType::White),
            ("fine", NoiseType::Fine),
            ("lightness", NoiseType::Lightness),
        ] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.02 type={written}; }}; }}\n"
            ));
            let glass = parsed.materials[0].resolve().glass;
            assert_eq!(glass.noise, Some(0.02), "{written}");
            assert_eq!(glass.noise_type, expected, "{written}");
        }
    }

    #[test]
    fn glass_noise_type_rejects_an_unknown_value() {
        let err = do_parse_err("material \"frost\" { glass { noise 0.02 type=blue; }; }\n");
        assert!(err.contains("unknown NoiseType value: blue"), "{err}");
    }

    #[test]
    fn glass_noise_type_cannot_be_written_without_an_amount() {
        let err = do_parse_err("material \"frost\" { glass { noise type=fine; }; }\n");
        assert!(!err.is_empty());
    }

    #[test]
    fn an_omitted_noise_type_resolves_to_white_and_keeps_the_amount_rule() {
        let written = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let glass = written.materials[0].resolve().glass;
        assert_eq!(glass.noise, Some(0.5));
        assert_eq!(glass.noise_type, NoiseType::White);

        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        let glass = omitted.materials[0].resolve().glass;
        assert_eq!(glass.noise, None);
        assert_eq!(glass.noise_type, NoiseType::White);
        assert_eq!(ResolvedGlass::default().noise_type, NoiseType::White);
    }
```

The test module starts with `use super::*;`, so `NoiseType` is in scope as
soon as Step 3 adds it to the crate root's `pub use crate::material::{…}`
list; no import line is needed.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p niri-config glass_noise_type`
Expected: compile error, `NoiseType` not found.

- [ ] **Step 3: Add the enum, the struct and the field**

In `niri-config/src/material.rs`, after the `FocusResponse` enum and before
the `response_from_str!` macro:

```rust
/// The grain pattern `noise` renders with. `White` is the original per-pixel
/// uniform hash; `Fine` is its high-pass, bell-shaped form; `Lightness`
/// applies `Fine` to Oklab lightness so chroma and hue hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NoiseType {
    #[default]
    White = 0,
    Fine = 1,
    Lightness = 2,
}
```

After the four existing `response_from_str!` invocations:

```rust
response_from_str!(NoiseType, "white" => White, "fine" => Fine, "lightness" => Lightness);
```

Directly after the `Distortion` struct:

```rust
/// `noise <amount> type=<type>`.
///
/// The type does nothing while the amount is zero, so it rides the node it
/// depends on, as `distortion` carries `scale=`. An omitted type is `white`,
/// which renders exactly as the node did before the property existed.
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
}
```

In `Glass`, replace

```rust
    #[knuffel(child, unwrap(argument))]
    pub noise: Option<FloatOrInt<0, 1>>,
```

with

```rust
    #[knuffel(child)]
    pub noise: Option<Noise>,
```

In `ResolvedGlass`, after the `saturation` field:

```rust
    /// Grain pattern for `noise`. No inheritance: the global `blur` block
    /// has no notion of it, so omission is `White` regardless of backdrop
    /// blur.
    pub noise_type: NoiseType,
```

In `impl Default for ResolvedGlass`, after `saturation: None,`:

```rust
            noise_type: NoiseType::White,
```

In `Material::resolve`, replace `noise: g.noise.map(|x| x.0),` with

```rust
                noise: g.noise.map(|x| x.amount.0),
                noise_type: g.noise.and_then(|x| x.kind).unwrap_or_default(),
```

In `niri-config/src/lib.rs`, add `Noise` and `NoiseType` to the
`pub use crate::material::{…}` list, keeping it alphabetical.

- [ ] **Step 4: Run the config crate's tests**

Run: `cargo test -p niri-config`
Expected: all pass, including the four new tests and the untouched
`glass_noise_rejects_values_outside_zero_and_one` (the amount's range error
text is unchanged because `Noise.amount` is the same `FloatOrInt<0, 1>`).

If `glass_noise_type_cannot_be_written_without_an_amount` fails because
knuffel accepted the node, that means the argument was made optional; it must
stay a required `#[knuffel(argument)]`.

- [ ] **Step 5: Document the grammar**

In `docs/materials/material-config.md`, after the `| noise | float | inherit | 0–1 | — |`
row add:

```markdown
| `noise` `type=` | `white` / `fine` / `lightness` | `white` | — | — |
```

After the paragraph that begins "`noise` and `saturation` are applied after
the glass optics", add:

```markdown
`noise` takes an optional `type=` property naming its grain. `white` is the
original per-pixel uniform hash added equally to the three channels. `fine`
subtracts each pixel's eight-neighbour mean from that hash and rescales it, so
the grain loses its low-frequency clumps and its hard extremes while keeping
the same strength for the same amount; it is still achromatic. `lightness`
applies the `fine` value to Oklab lightness instead, so the backdrop's chroma
and hue hold except where the result leaves the sRGB gamut and clamps. The
type has no inheritance and an omitted type is `white`.
```

- [ ] **Step 6: Run the repository gates**

Run: `just check && just test`
Expected: both pass. `just check` includes `cargo fmt --check` and clippy; fix
formatting with `cargo fmt --all` if it complains.

- [ ] **Step 7: Commit**

```bash
git add niri-config/src/material.rs niri-config/src/lib.rs docs/materials/material-config.md
git commit -m "feat(config): parse a type property on glass noise

noise <amount> type=white|fine|lightness, resolved to
ResolvedGlass.noise_type with white as the omitted default. The amount
keeps its Option and inheritance rule."
```

The pre-commit hook regenerates nothing itself; if it reports
`docs/materials/upstream-divergence.md is stale`, run `just upstream-report`,
`git add docs/materials/upstream-divergence.md`, and commit again.

---

### Task 2: Render the type in the material shader

**Files:**
- Modify: `src/render_helpers/shaders/mod.rs:175` (the `UniformName` list,
  after `mat_noise`)
- Modify: `src/render_helpers/material.rs:840` (the `Uniform::new` list) and
  its test module (`postprocess_change_advances_the_commit_in_place` at ~1253)
- Modify: `src/layout/tile.rs` test module (after
  `written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur`,
  ~line 2452)
- Modify: `src/render_helpers/shaders/material.frag` (uniform block at ~line
  31, helpers after `hash12` at ~line 220, noise branch at ~line 535)

**Interfaces:**
- Consumes: `niri_config::NoiseType` and `ResolvedGlass.noise_type` from
  Task 1; `MaterialRenderElement.glass: ResolvedGlass` (already present);
  `hash12`, `srgbToLinear`, `linearToSrgb` in `material.frag`.
- Produces: uniform `float mat_noise_type` (0 white, 1 fine, 2 lightness);
  GLSL `float fineGrain(vec2 p)`, `vec3 linearToOklab(vec3 c)`,
  `vec3 oklabToLinear(vec3 lab)`.

- [ ] **Step 1: Write the failing damage test**

In `src/render_helpers/material.rs`'s test module, after
`postprocess_change_advances_the_commit_in_place`:

```rust
    #[test]
    fn noise_type_change_advances_the_commit_in_place() {
        let mut slot = Some(MaterialState::new(render_config("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();
        let initial_commit = slot.as_ref().unwrap().commit.get();

        let mut changed = render_config("frost");
        changed.material.glass.noise_type = niri_config::NoiseType::Fine;
        assert!(!apply_resolved(&mut slot, Some(&changed)));
        assert_eq!(slot.as_ref().unwrap().id(), &id_before);
        assert_ne!(slot.as_ref().unwrap().commit.get(), initial_commit);
    }
```

- [ ] **Step 2: Write the failing resolver test**

In `src/layout/tile.rs`'s test module, after
`written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur`:

```rust
    #[test]
    fn noise_type_reaches_the_render_config_regardless_of_blur() {
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let global_off = niri_config::Blur {
            off: true,
            ..Default::default()
        };
        for (noise_type, backdrop_blur, blur) in [
            (niri_config::NoiseType::Fine, false, niri_config::Blur::default()),
            (niri_config::NoiseType::Lightness, true, global_off),
            (niri_config::NoiseType::White, true, niri_config::Blur::default()),
        ] {
            let options = options_for(
                niri_config::ResolvedGlass {
                    noise: Some(0.3),
                    noise_type,
                    backdrop_blur,
                    ..Default::default()
                },
                blur,
            );
            let resolved = resolve_material(Some(&reference), &options).unwrap();
            assert_eq!(resolved.material.glass.noise_type, noise_type);
            assert_eq!(resolved.noise, 0.3);
        }
    }
```

- [ ] **Step 3: Run both tests**

Run: `cargo test --all --exclude niri-visual-tests noise_type`
Expected: PASS for both. They cannot fail once Task 1 has landed, because
`ResolvedGlass` derives `PartialEq`, rides inside `MaterialRenderConfig`, and
`apply_resolved` already bumps the commit on any config inequality. They are
regression pins for behaviour the spec requires, not red-green steps; a
failure here means Task 1 is incomplete.

- [ ] **Step 4: Register and set the uniform**

In `src/render_helpers/shaders/mod.rs`, after
`UniformName::new("mat_noise", UniformType::_1f),` add:

```rust
                UniformName::new("mat_noise_type", UniformType::_1f),
```

In `src/render_helpers/material.rs`, after
`Uniform::new("mat_noise", self.noise),` add:

```rust
            Uniform::new("mat_noise_type", g.noise_type as u8 as f32),
```

`g` is the `let g = &self.glass;` binding at line 818 of the same function,
already used by the `mat_ior` line a few lines below.

- [ ] **Step 5: Change the shader**

In `src/render_helpers/shaders/material.frag`, after `uniform float mat_noise;`:

```glsl
uniform float mat_noise_type;
```

After the `hash12` function:

```glsl
// High-pass hash grain: the fragment's hash minus the mean of its eight
// unit-offset neighbours. A weighted sum of nine independent uniforms is
// bell-shaped and zero-mean, and the subtraction removes the low-frequency
// energy that reads as clumps. Var(centre - mean8) = (1/12)(1 + 1/8), so
// sqrt(8/9) brings the standard deviation back to a single hash's.
float fineGrain(vec2 p) {
    float mean = (hash12(p + vec2(-1.0, -1.0)) + hash12(p + vec2(0.0, -1.0))
                + hash12(p + vec2(1.0, -1.0)) + hash12(p + vec2(-1.0, 0.0))
                + hash12(p + vec2(1.0, 0.0)) + hash12(p + vec2(-1.0, 1.0))
                + hash12(p + vec2(0.0, 1.0)) + hash12(p + vec2(1.0, 1.0))) / 8.0;
    return (hash12(p) - mean) * 0.94280904;
}

// Oklab (Björn Ottosson, public domain), linear sRGB in and out.
vec3 linearToOklab(vec3 c) {
    float l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    float m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    float s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    float l_ = pow(max(l, 0.0), 1.0 / 3.0);
    float m_ = pow(max(m, 0.0), 1.0 / 3.0);
    float s_ = pow(max(s, 0.0), 1.0 / 3.0);
    return vec3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_);
}

vec3 oklabToLinear(vec3 lab) {
    float l_ = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
    float m_ = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
    float s_ = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;
    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;
    return vec3(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s);
}
```

Replace the noise branch

```glsl
        if (mat_noise > 0.0) {
            vec2 noiseSeed = gl_FragCoord.xy + vec2(47.0, 113.0);
            glassColor += (hash12(noiseSeed) - 0.5) * mat_noise;
        }
```

with

```glsl
        if (mat_noise > 0.0) {
            vec2 noiseSeed = gl_FragCoord.xy + vec2(47.0, 113.0);
            if (mat_noise_type < 0.5) {
                glassColor += (hash12(noiseSeed) - 0.5) * mat_noise;
            } else {
                float grain = fineGrain(noiseSeed) * mat_noise;
                if (mat_noise_type < 1.5) {
                    glassColor += vec3(grain);
                } else {
                    // Clamp before re-encoding: a lightness pushed past the
                    // gamut yields negative linear values, and pow() of a
                    // negative is undefined.
                    vec3 lab = linearToOklab(srgbToLinear(clamp(glassColor, 0.0, 1.0)));
                    lab.x += grain;
                    glassColor = linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
                }
            }
        }
```

The `white` branch keeps the original seed and the original expression.

- [ ] **Step 6: Build, run the unit tests and the gates**

Run: `cargo build && just check && just test`
Expected: the build succeeds (the shader is a string compiled at runtime, so
`cargo build` only proves the Rust side; the shader compiles on the GPU in
Task 3), clippy is clean, all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/render_helpers/shaders/mod.rs src/render_helpers/material.rs src/layout/tile.rs src/render_helpers/shaders/material.frag
git commit -m "feat(material): render the glass noise type

One float uniform read from ResolvedGlass selects white, fine (high-pass
bell-shaped hash grain) or lightness (the same grain on Oklab L). The
white branch is unchanged."
```

---

### Task 3: Nested GLES evidence, index, spec status

**Files:**
- Create: `docs/materials/scripts/glass-noise-type-smoke.sh`
- Create: `docs/materials/2026-09-06-material-glass-noise-type-evidence.md`
- Modify: `docs/materials/README.md` (index, after the
  `scripts/glass-noise-saturation-smoke.sh` entry at ~line 30)
- Modify: `docs/specs/2026-09-06-material-glass-noise-type-design.md`
  (status header)

**Interfaces:**
- Consumes: the `type=` grammar from Task 1 and the shader from Task 2; the
  harness idioms in `docs/materials/scripts/glass-noise-saturation-smoke.sh`
  (`write_config`, `start_nested`, `capture`, `compare_metric`, the
  assertion helpers), which this script copies rather than sources so each
  script stays self-contained like the existing ones.
- Produces: `metrics.txt` with the names listed in Step 2, and the evidence
  document.

- [ ] **Step 1: Copy the harness scaffolding**

Copy `docs/materials/scripts/glass-noise-saturation-smoke.sh` to
`docs/materials/scripts/glass-noise-type-smoke.sh`. Keep everything from
`set -eu` through the `assert_greater` helper unchanged except:

- the header comment, which becomes:

```bash
#!/usr/bin/env bash
# Glass noise type smoke (material-6e7352): capture a blank transparent kitty
# over a glass material on a headless Weston host, once per noise type at the
# same amount, and assert the properties the design claims for each type
# against an amount-0 capture of the same fixture. Exit 0 means every
# assertion held; any failure exits non-zero with a FAIL line and the trap
# preserves that status.
#
# The backdrop is a flat warm mid-tone rather than the colour bars of
# glass-noise-saturation-smoke.sh: the statistics below need the grain to
# stay inside the gamut (white reaches ±0.25 of full scale at amount 0.5,
# fine ±0.47), and the chroma-invariance check needs a backdrop with chroma
# to hold. Every difference image below is against the amount-0 capture of
# the same session type, so static content cancels exactly and only the
# grain remains; that relies on the same determinism the omitted check
# asserts first.
#
# Env: IMPL (niri binary under test), OUT (artifact dir).
# Requires: weston, kitty, swaybg, jq, rg, ImageMagick 7 with Oklab.
```

- the wallpaper line, which becomes a flat mid-tone:

```bash
WALL=$OUT/warm-mid.png
magick -size 1280x720 xc:'rgb(140,115,90)' "$WALL"
```

- the runtime dir prefix `gns.XXXXXX` becomes `gnt.XXXXXX`, and the probe
  app-id `gns-probe` becomes `gnt-probe` everywhere (config, window rule,
  `probe_count`, the kitty class).

Add one helper after `sd`:

```bash
# Mean of a 0/1 mask image: the share of pixels the expression selected.
share() {   # $1 image, $2 fx expression over u; result in METRIC
    local out
    out=$(magick "$1" -colorspace Gray -fx "$2" -format '%[fx:mean]' info:) || fail "magick share on $1 failed"
    is_number "$out" || fail "share of $1 is not numeric: $out"
    METRIC=$out
}
# Signed difference of two captures offset by 0.5, as a 16-bit gray image,
# for statistics that need the sign (compose Mathematics computes
# source - destination + 0.5; which operand is which does not matter to a
# standard deviation). abs difference is for statistics that do not.
signed_diff() {   # $1 a, $2 b, $3 out
    magick "$1" "$2" -compose Mathematics -define compose:args=0,1,-1,0.5 -composite -colorspace Gray -depth 16 "$3" || fail "signed diff $3 failed"
}
ratio() {   # $1 numerator, $2 denominator, printed
    awk -v a="$1" -v b="$2" 'BEGIN { if (b == 0) print "nan"; else print a / b }'
}
abs_diff() {   # $1 a, $2 b, $3 out
    magick "$1" "$2" -compose difference -composite -colorspace Gray -depth 16 "$3" || fail "abs diff $3 failed"
}
# Oklab a and b only, L pinned, frozen as raw channels in a 16-bit image.
oklab_ab() {   # $1 in, $2 out
    magick "$1" -colorspace Oklab -channel R -evaluate set 50% +channel -set colorspace sRGB -depth 16 "$2" || fail "oklab ab $2 failed"
}
assert_less() {
    is_number "$2" && is_number "$3" || fail "$1 is not numeric: $2 vs $3"
    awk -v a="$2" -v b="$3" 'BEGIN { exit !(a < b) }' || fail "$1 expected $2 < $3"
}
```

- [ ] **Step 2: Write the captures, metrics and assertions**

Replace everything from `sha256sum "$IMPL"` to the final `echo "PASS…"` with:

```bash
sha256sum "$IMPL" > "$OUT/binaries.sha256"
"$IMPL" --version > "$OUT/impl.version"

capture zero-first "$IMPL" $'noise 0\n        saturation 1' ""
capture zero-after "$IMPL" $'noise 0\n        saturation 1' ""
capture untyped    "$IMPL" $'noise 0.5\n        saturation 1' ""
capture white      "$IMPL" $'noise 0.5 type=white\n        saturation 1' ""
capture fine       "$IMPL" $'noise 0.5 type=fine\n        saturation 1' ""
capture lightness  "$IMPL" $'noise 0.5 type=lightness\n        saturation 1' ""

ae "$OUT/zero-first.png" "$OUT/zero-after.png";  zero_determinism_ae=$METRIC
ae "$OUT/untyped.png" "$OUT/white.png";          untyped_vs_white_ae=$METRIC

sd "$OUT/zero-after-roi.png"; zero_sd=$METRIC
oklab_ab "$OUT/zero-after-roi.png" "$OUT/zero-ab.png"
for t in white fine lightness; do
    sd "$OUT/$t-roi.png"; printf -v "${t}_sd" '%s' "$METRIC"
    abs_diff "$OUT/$t-roi.png" "$OUT/zero-after-roi.png" "$OUT/$t-absdiff.png"
    signed_diff "$OUT/$t-roi.png" "$OUT/zero-after-roi.png" "$OUT/$t-signed.png"
    # Deviation statistics in units of the amount, which is 0.5 of full
    # scale: white's bound 0.5*amount is 0.25 of full scale (0.2549 allows
    # for 8-bit rounding), white's top band starts at 0.4*amount = 0.2, and
    # white's median 0.25*amount is 0.125.
    share "$OUT/$t-absdiff.png" 'u>=0.2 && u<=0.2549 ? 1 : 0'; printf -v "${t}_top_band" '%s' "$METRIC"
    share "$OUT/$t-absdiff.png" 'u<0.125 ? 1 : 0';             printf -v "${t}_below_median" '%s' "$METRIC"
    share "$OUT/$t-absdiff.png" 'u>0.2549 ? 1 : 0';            printf -v "${t}_beyond_white" '%s' "$METRIC"
    sd "$OUT/$t-signed.png"; printf -v "${t}_full_sd" '%s' "$METRIC"
    magick "$OUT/$t-signed.png" -filter box -resize 25% "$OUT/$t-signed-down.png"
    sd "$OUT/$t-signed-down.png"; printf -v "${t}_down_sd" '%s' "$METRIC"
    down=${t}_down_sd; full=${t}_full_sd
    printf -v "${t}_lowfreq_ratio" '%s' "$(ratio "${!down}" "${!full}")"
    oklab_ab "$OUT/$t-roi.png" "$OUT/$t-ab.png"
    rmse "$OUT/$t-ab.png" "$OUT/zero-ab.png"; printf -v "${t}_ab_rmse" '%s' "$METRIC"
done

for v in zero_determinism_ae untyped_vs_white_ae zero_sd \
         white_sd fine_sd lightness_sd \
         white_top_band fine_top_band white_below_median fine_below_median \
         white_beyond_white fine_beyond_white \
         white_lowfreq_ratio fine_lowfreq_ratio \
         white_ab_rmse fine_ab_rmse lightness_ab_rmse; do
    printf '%s=%s\n' "$v" "${!v}"
done | tee "$OUT/metrics.txt"

assert_zero zero_determinism_ae "$zero_determinism_ae"        # two sessions of the amount-0 fixture are byte-identical: the precondition of every difference below
assert_zero untyped_vs_white_ae "$untyped_vs_white_ae"        # an omitted type is white, byte for byte
assert_greater white_sd "$white_sd" "$zero_sd"                # every type renders grain
assert_greater fine_sd "$fine_sd" "$zero_sd"
assert_greater lightness_sd "$lightness_sd" "$zero_sd"
assert_less fine_lowfreq_ratio "$fine_lowfreq_ratio" "$white_lowfreq_ratio"   # fine has less low-frequency energy (expected ≈0.12 vs ≈0.25)
assert_less fine_top_band "$fine_top_band" "$white_top_band"                  # fine thins white's top band (expected ≈0.12 vs ≈0.20)
assert_greater fine_below_median "$fine_below_median" "$white_below_median"   # and lifts the share below white's median (expected ≈0.53 vs ≈0.50)
assert_less lightness_ab_rmse "$lightness_ab_rmse" "$white_ab_rmse"           # lightness holds Oklab chroma where white shifts it
assert_less lightness_ab_rmse "$lightness_ab_rmse" "$fine_ab_rmse"
# fine_beyond_white is recorded, not asserted: a few percent of fine's pixels
# exceed white's bound by design (expected ≈0.05).

sha256sum "$OUT"/*.png "$OUT"/*.kdl >> "$OUT/SHA256SUMS"
if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
    fail "material error, fallback or panic in niri.log"
fi
echo "PASS: artifacts in $OUT"
```

`chmod +x` the script.

- [ ] **Step 3: Build the binary and run the smoke**

Run:

```bash
cargo build
IMPL=target/debug/niri OUT=/mnt/ssd3/tmp/material-6e7352-smoke docs/materials/scripts/glass-noise-type-smoke.sh
```

Expected: `PASS: artifacts in …` and a `metrics.txt` whose values sit near the
expected figures in the assertion comments. If a shader compile error appears
in `niri.log`, the GLSL from Task 2 is at fault; fix the shader, rebuild, and
rerun. If an assertion fails while the metrics are near their expected
values, the threshold is wrong and the executor stops and reports the metrics
rather than loosening the assertion; if the metrics are far off, the shader
or the harness is wrong and the plan needs a revision.

- [ ] **Step 4: Record the evidence**

Create `docs/materials/2026-09-06-material-glass-noise-type-evidence.md` in
the form of `2026-09-05-material-glass-noise-saturation-params-evidence.md`:

```markdown
# Glass noise type: verification evidence

**Result:** PASS or FAIL, <date>, headless Weston <version>. <One paragraph on
the run: no renderer errors, fallbacks or panics in `niri.log` for any of the
six captures, or what appeared.>

## Pinned revisions

- Implementation source commit `<sha>`, binary `<path>`, SHA-256 `<hash>`.

## Metrics

<Paste `metrics.txt` as a two-column table, and after it one sentence per
assertion group: determinism and omitted-equals-white; each type renders;
fine's low-frequency ratio and band figures against white; lightness's
Oklab a/b RMSE against white and fine; fine's recorded share beyond white's
bound.>

## Reading the figures

<Two or three sentences: what the low-frequency ratio and the band shares say
about coarseness, and what the chroma RMSE says about lightness. Note the
expected-versus-measured gap if any.>
```

Fill every angle-bracket field from the run; leave none.

- [ ] **Step 5: Index and status**

In `docs/materials/README.md`, after the
`scripts/glass-noise-saturation-smoke.sh` entry, add:

```markdown
- `../specs/2026-09-06-material-glass-noise-type-design.md`: optional `type=` on glass `noise`: white, fine (high-pass bell-shaped hash grain) and lightness (the same grain on Oklab L).
- `2026-09-06-material-glass-noise-type-evidence.md`: nested GLES evidence for the three types: each renders, fine has less low-frequency energy and a thinner top band than white, lightness holds Oklab chroma.
- `scripts/glass-noise-type-smoke.sh`: headless harness for those captures.
```

In `docs/specs/2026-09-06-material-glass-noise-type-design.md`, replace the
status line with:

```markdown
**Status:** implemented on branch `glass-noise-type` at `<sha of Task 2's commit>`;
`just check` and `just test` passing; nested GLES evidence in
[`2026-09-06-material-glass-noise-type-evidence.md`](../materials/2026-09-06-material-glass-noise-type-evidence.md).
Desktop acceptance (the `lightness` gate) pending installation.
```

- [ ] **Step 6: Gates and commit**

Run: `just check && just test`
Expected: pass.

```bash
git add docs/materials/scripts/glass-noise-type-smoke.sh docs/materials/2026-09-06-material-glass-noise-type-evidence.md docs/materials/README.md docs/specs/2026-09-06-material-glass-noise-type-design.md
git commit -m "docs(materials): record the glass noise type evidence

Per-type nested GLES smoke: determinism, omitted equals white, each type
renders, fine's low-frequency ratio and band shares against white,
lightness's Oklab chroma invariance."
```

Then `tasks done material-6e7352 "<one line: what landed and the metrics'
verdict>"` in the same commit or an immediately following `chore(tasks)`
commit, and run `tasks check`.

---

## After the plan

Desktop acceptance is not a plan task: it needs the package installed on the
desktop and a pair of eyes. It compares `white`, `fine` and `lightness` on an
unfocused pane at Prism's default amount and decides whether `lightness`
stays. If it is dropped, both specs lose the value and the Prism piece's enum
shrinks before it lands; the plan for the Prism piece (`prism-51f23b`) is
written after that decision.
