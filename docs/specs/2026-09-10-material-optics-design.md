# Material optics: design

**Status:** designed 2026-09-10; not implemented.

**Task:** `material-397fcb` (the API), with first users `material-bb3fe5`
(ice) and `material-f0fc7b` (aurora and rainbow). Prism pieces are filed in
the Prism project and depend on the material tasks; they are listed in
section 9.

## Context

A material is a named parameter set for one shader. v1 fixed the shape as
`material "name" { glass { ... } }` with exactly one `glass` block
([`2026-08-24-glass-config-surface-design.md`](../materials/2026-08-24-glass-config-surface-design.md)),
and every addition since has been a parameter on that block. The shape is
right: the slab, its bevel, the jelly motion, the ring of light, and the
signal responses are what make a window read as a pane of something, and
every material anyone has asked for keeps them. Ice, aurora, rainbow, and
particles all add a stage to the glass pipeline; none replaces the slab.

What is wrong is the cost of a stage. Each parameter is hand-copied across
six lists that must stay in lockstep: the knuffel `Glass` struct, the
`ResolvedGlass` struct and its defaults, the field-by-field `resolve()`
mapping in `niri-config/src/material.rs`, the `UniformName` registry in
`src/render_helpers/shaders/mod.rs`, the `Uniform::new` upload in
`src/render_helpers/material.rs`, and the `uniform` declarations in
`material.frag`, plus the parameter table in `material-config.md`, the stage
table in `render-pipeline.md`, and Prism's renderer. The last parameter
added, `noise type=` (one three-valued enum and one float uniform), touched
13 files and about 1,700 lines (`f0370f52`). A second material *kind* would
be worse: `Material.glass` is a required single child whose single-kind
assumption is documented as load-bearing (`niri-config/src/material.rs`,
"when a second material type is added, this field becomes a hand-written
dispatcher"), and there is one `ProgramType::Material` program with one
inline uniform list.

Prism, meanwhile, already presents the glass as an ordered rack of devices
in shader order, each with a mix, a bypass, and detail rows
(Prism `docs/specs/2026-09-08-device-chain-rack-design.md`), and a saved
profile is a full snapshot of every glass value
(Prism `docs/specs/2026-09-05-prism-context-layers-design.md`). A "material
a user can choose" is therefore a profile on the Prism side and a parameter
bundle on the niri side; neither needs a new kind.

## Decision

The glass pipeline becomes a slab plus an ordered list of **optics**. An
optic is one stage of the pipeline, self-contained: it owns its KDL node, its
resolved values and defaults, its uniforms, one GLSL file holding its stage
functions, and one section of the user docs. The material shape, the window
rule, the `response` blocks, and every existing parameter name are
unchanged. Only `saturation` and `noise` migrate into optic modules, to
prove the pattern against a byte-identical rendering check; the slab, jelly,
refraction, tint, ring, and signal code stays where it is.

Adding an optic is then a fixed recipe: four new files, three one-line
registrations, and a docs section. The recipe is written down in
`docs/materials/adding-an-optic.md` with the noise optic as the worked
example, and the parameter table in `material-config.md` is rendered from
typed specs whose defaults and ranges come from the resolved types, with a
test that drives the parser at those ranges, so the table, the specs, and
the parser are checked against each other.

Three optics are the first users: `cracks` for ice, `iridescence` for
rainbow glass, and `aurora` for a slow colour field inside the glass. Each
ships with a preset material under `resources/materials/`, and each gets a
Prism device and a starter profile on the Prism side.

Rejected: a **declaration macro** generating the six lists from one table.
It would have to express `distortion scale=`, `noise type=`,
inherit-or-neutral options, colours, bools, and the `Milli` type, so it
becomes a small language, and it does nothing for the GLSL, which is where
the work of a new material lives. Rejected: **material kinds** with their
own programs. Every requested material keeps the slab, so a kind duplicates
the slab, jelly, ring, signal, and damage code or shares it through an
abstraction nothing else needs. Rejected: **drift guards only** (tests
pinning the docs and a manifest to the structs). A contributor would still
edit six places, and a new stage would still hit the single-program wall.
Rejected: **migrating every existing parameter now**. Nothing new needs the
slab or refraction parameters moved, and proving byte-identical rendering
for seventeen parameters is a large diff for no new capability.

## 1. What an optic is

An optic acts at one or more of four hook points in the fragment shader,
named for the stage of `render-pipeline.md` they sit in:

| Hook | Pipeline stage | Signature | Acts on |
| --- | --- | --- | --- |
| `normal` | 2, after distortion and ripple, before the taps | `vec3 <optic>_normal(vec3 n, vec2 p)` | the perturbed normal the refraction taps use |
| `specular` | 5, after the Schlick term, before the signal accent mix | `vec3 <optic>_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)` | the Fresnel glint |
| `emissive` | 6, after the ring of light, before the sweeps | `vec3 <optic>_emissive(vec2 p, vec3 n, vec3 att, float innerDist)` | returns light to add; `main` accumulates it into `emissive` |
| `post` | 9 and 10, on the encoded sRGB glass colour | `vec3 <optic>_post(vec3 color, vec2 fragCoord)` | the finished glass colour |

An optic defines one function per hook it acts at, named
`<optic>_<hook>`, in its own GLSL file. `main.frag` calls the functions in
`OPTICS` order at each hook; a contributor adds one call per hook the new
optic uses. The `normal`, `specular`, and `post` hooks transform a value
and return it; the `emissive` hook is additive and returns a contribution.

Every optic has a **neutral** configuration, the one a material that never
names the optic resolves to, and the optic's docs section states it. At its
neutral, a transforming hook returns its input unchanged and an emissive
hook returns `vec3(0.0)`, behind one uniform branch, so an unconfigured
optic costs that branch and nothing else. The neutral is not always zero:
`saturation`'s is 1 (0 is grayscale), `noise`'s is an amount of 0 or an
inherited amount of 0, and the three new optics' is an amount of 0.

Every optic may read the shared inputs the prelude declares: the element
position `p`, the slab geometry (`g_center`, `g_half`, `g_outer_r`), the
per-window seed `mat_jelly_seed` (named for its first user; it is the seed
any optic should use for per-window variation), `mat_thickness`, and
`mat_ior`. It declares its own uniforms with the prefix `mat_<optic>_`.

## 2. Where an optic lives

| File | Holds |
| --- | --- |
| `niri-config/src/material/optics/<name>.rs` | the knuffel node struct; the resolved struct with `Default`; `resolve(node) -> Resolved`; `validate(&self) -> Result<(), String>` when the optic has a cross-parameter rule; `pub fn params() -> Vec<ParamSpec>` |
| `src/render_helpers/material/optics/<name>.rs` | `impl Optic for Resolved<Name>`: the uniform names, `values(&self, ctx)`, and `next_change(&self, ctx)` |
| `src/render_helpers/shaders/material/<name>.frag` | its `uniform` declarations and its `<name>_<hook>` functions |
| `docs/materials/material-config.md` | one `### <name>` section under `## Optics` describing what the optic does and which stage it acts at; the parameter rows are generated (section 6) |

And three one-line registrations:

- `#[knuffel(child)] pub <name>: Option<<Name>>` on `Glass`, and
  `<name>: optics::<name>::resolve(g.<name>)` in `Material::resolve()`.
  `Material::validate()` calls each optic's `validate()` after the
  resolved-response checks it makes today.
- one `OpticEntry` in the ordered `OPTICS` table in
  `src/render_helpers/material/optics/mod.rs`;
- one call per hook in `src/render_helpers/shaders/material/main.frag`.

`niri-config/src/material.rs` keeps `Material`, `Glass`, `ResolvedGlass`,
the response types, `MaterialRef`, and the core parameters. The file moves
to `niri-config/src/material/mod.rs` so `optics/` can sit beside it; no
item is renamed.

The crate split is deliberate: `niri-config` parses and resolves, `niri`
renders. An optic therefore has one file in each crate. The alternative,
carrying uniform-shaped values in `niri-config`, would put render contract
into the config crate to save one file.

## 3. The renderer contract

```rust
/// One pipeline stage, self-contained. Implemented on the resolved config
/// type from niri-config; the trait is local to niri.
pub trait Optic {
    /// The uniforms this optic's GLSL declares, in declaration order.
    const UNIFORMS: &'static [UniformName<'static>];
    /// This frame's uniforms, one per entry of `UNIFORMS`, in the same order.
    fn values(&self, ctx: &OpticFrame) -> Vec<Uniform<'static>>;
    /// The next instant `values` changes with no config change, or `None`
    /// for a static optic. The default is static.
    fn next_change(&self, _ctx: &OpticFrame) -> Option<Duration> { None }
}

/// What an optic may depend on beyond its own configuration.
pub struct OpticFrame<'a> {
    pub now: Duration,                  // the tile clock, unadjusted
    pub motion: SignalMotionPolicy,     // full | reduced | off
    pub animations_off: bool,
    pub backdrop_blur: bool,            // effective: configured && !blur.off
    pub blur: &'a niri_config::Blur,    // the global block, for inheritance
    pub seed: f32,                      // the window's jelly seed, component 0
}

pub struct OpticEntry {
    pub glsl: &'static str,
    pub uniforms: &'static [UniformName<'static>],
    pub values: fn(&ResolvedGlass, &OpticFrame) -> Vec<Uniform<'static>>,
    pub next_change: fn(&ResolvedGlass, &OpticFrame) -> Option<Duration>,
}

pub static OPTICS: &[OpticEntry] = &[ /* in render order */ ];
```

`Uniform` and `UniformValue` are smithay's own types; at the pinned revision
(`ff5fa7df`) both derive `PartialEq`, so optic values are fingerprinted as
they are, with no value type of niri's own and no conversion at draw time.

Two things that a new parameter had to do by hand become automatic:

- **Damage.** `InputFingerprint` gains `optics: Vec<Uniform<'static>>`, the
  concatenation of every optic's `values`. `MaterialState::advance_commit`
  already compares the fingerprint between frames, so an optic whose values
  change registers damage without touching the fingerprint type.
- **Redraw scheduling.** `Tile::signal_tick_deadline` today returns `None`
  as soon as `signal_frame_cache` is empty, which is the normal state of a
  window whose response lights no focus filament and that carries no
  signal. Optic deadlines must not sit behind that gate. The method becomes
  `tick_deadline`: it computes the optic deadline from `OPTICS` whenever the
  tile has a material and its slab band is in view, computes the signal
  deadline as today only when the cache is present, and returns the minimum
  of whichever exist. An animated optic never touches `signal.rs`, and a
  visible, unfocused, signal-free aurora window keeps redrawing at its
  `drift-hz`. Section 11 names the integration check for that case.

`MaterialRenderConfig` loses its `noise` and `saturation` fields. The
inherit-or-neutral rule that `resolve_material` in `src/layout/tile.rs`
applies today moves into the noise and saturation optics' `values`, which
read `ctx.backdrop_blur` and `ctx.blur`. The documented rule, "each
parameter decides on its own", becomes literal. `backdrop_blur_enabled`
stays in `tile.rs`: it settles the gate once per frame and the result
travels in `OpticFrame`.

## 4. Shader assembly

`src/render_helpers/shaders/material.frag` splits into
`shaders/material/prelude.frag` (the precision header, the `varying`, the
core uniforms, the shared globals, and the helper library: sRGB and Oklab
conversions, simplex noise, `hash12`, `fineGrain`, the rounded-box SDF,
`slabSurface`, `tap`, `lightShift`, `filamentBand`) and
`shaders/material/main.frag` (`void main()`, with the hook calls).

`Shaders::compile` builds the source as the prelude, each `OPTICS` entry's
`glsl` in order, then `main`, separated by `// ---- optic: <name>` comment
lines so a compile error's line number is locatable by hand. The
`UniformName` list is the core list that exists today minus the migrated
`mat_noise`, `mat_noise_type`, and `mat_saturation`, followed by each
entry's `uniforms`. One program still serves every material; the order of
`OPTICS` is the render order, the order of the docs, and the order of
Prism's rack.

`compile_program` prepends `#version 100`, so the prelude must not carry its
own version line; `material.frag` does not today.

## 5. Migration of saturation and noise

`ResolvedGlass.noise: Option<f64>`, `saturation: Option<f64>`, and
`noise_type` become `noise: ResolvedNoise { amount: Option<f64>, kind:
NoiseType }` and `saturation: ResolvedSaturation { amount: Option<f64> }`.
The KDL nodes, `noise <amount> type=<type>` and `saturation <amount>`, are
unchanged, and the `Noise` node struct moves into the optic module. The
optics act at the `post` hook, saturation first, and their GLSL is the
existing stage 9 and 10 code moved verbatim.

`material-3fcba2` (stacked noise layers) is unaffected in scope: it widens
the noise node and becomes a change inside the noise optic's two Rust files
and one GLSL file. Its "single-node configs render byte-identical"
requirement is the same check this migration uses.

**Byte-identical.** Before and after the migration, the smoke scripts
`docs/materials/scripts/glass-noise-saturation-smoke.sh` and
`glass-noise-type-smoke.sh` are run on the headless Weston host with the
same fixture, and every capture is compared with `cmp`. Any differing pixel
fails the migration. The run is recorded in an
evidence doc under `docs/materials/`, dated the day of the run and named
`material-optics-evidence`, with the binary and capture hashes, following
the existing evidence docs.

## 6. Docs tied to the parser

A hand-written row can agree with the table and disagree with the code. The
parameter table is therefore rendered from typed specs whose defaults come
from the resolved `Default` values and whose ranges come from the bound
types, and a test drives the parser with those ranges.

```rust
pub enum ParamKind {
    Float { default: f64, min: f64, max: f64, unit: &'static str },
    /// A float whose omitted value inherits (noise and saturation).
    FloatOrInherit { min: f64, max: f64 },
    Color { default: Color },
    Bool { default: bool },
    Enum { default: &'static str, variants: &'static [&'static str] },
}

pub struct ParamSpec {
    pub node: &'static str,  // "noise", "noise type=", "aurora drift-hz"
    pub kind: ParamKind,
}

/// The parse-time bounds of a scalar type, as the type declares them.
pub trait Bounded { const MIN: f64; const MAX: f64; }
impl<const MIN: i32, const MAX: i32> Bounded for FloatOrInt<MIN, MAX> { /* MIN, MAX */ }
impl<const MIN: i32, const MAX: i32> Bounded for Milli<MIN, MAX> { /* MIN / 1000, MAX / 1000 */ }
impl<const MAX: i32> Bounded for Positive<MAX> { /* 0 exclusive, MAX */ }
```

Each optic exports `pub fn params() -> Vec<ParamSpec>`, a function rather
than a const so a default is read from `Resolved<Name>::default()` and a
range from `<FieldType as Bounded>::MIN` and `MAX`, where `FieldType` is the
bound type of the node struct's field. `niri-config/src/material/mod.rs`
exports `core_params()` the same way for the fifteen parameters that stay
on the core, in the order the table lists them today. `render_param_table()`
renders both as the Markdown table `material-config.md` carries now, between
`<!-- params:begin -->` and `<!-- params:end -->` markers.

Two tests in `niri-config` tie the three things together:

- **The table matches the specs.** The test reads the file relative to
  `CARGO_MANIFEST_DIR`, compares the block, and on mismatch fails with the
  expected block in its message. With `MATERIAL_DOCS_UPDATE=1` set, it
  rewrites the block instead.
- **The specs match the parser.** For every `Float` and `FloatOrInherit`
  spec, the test writes a material whose node carries `min`, then `max`,
  and asserts both parse and resolve to that value; then `max + 0.001` and
  `min - 0.001` (skipping `min - 0.001` for `Positive`, whose lower bound is
  exclusive and is tested with `0`) and asserts the parse fails with the
  type's range error. For every `Enum` spec it parses each variant and one
  unknown name. For every `Float` spec it also asserts that a material
  omitting the node resolves to `default`. A spec whose field type was
  changed without changing the spec, or whose default moved, fails here.

What this does not catch: a spec that names the wrong node string for a
field that exists. The parse test would then pass on a different node, so
the node strings are the one hand-maintained fact, and the worked example in
`adding-an-optic.md` says so.

`just check` runs the crate's tests through `cargo clippy --all-targets`
only; both tests run in `just test`, which the pre-push hook runs.

The per-parameter prose under the table, the stage table in
`render-pipeline.md`, and the Prism column there stay hand-written: they
explain rather than list.

## 7. The three optics

Every amount below is in `0–1` and defaults to `0`, so a material that does
not name the optic renders exactly as today.

### 7.1 `cracks` (ice)

```kdl
cracks 0.6 scale=1
```

| Parameter | Type | Default | Range |
| --- | --- | --- | --- |
| `cracks` | float | 0 | 0–1 |
| `cracks` `scale=` | float | 1 | 0.1–4 |

Hooks: `normal` and `specular`. A cellular pattern on element position,
`p * (0.02 / scale)` offset by the per-window seed, yields the two nearest
feature distances F1 and F2 from a 3 × 3 cell search. The crack mask is
`1 - smoothstep(0, 0.08, F2 - F1)`, one line per cell edge. `cracks_normal`
bends the normal along the mask's gradient by `amount * mask * 0.6`, so the
refracted image breaks along the lines; `cracks_specular` adds
`amount * mask * 0.15` to the glint, a faint bright filament. Static; no
clock. Cost is nine hashes per fragment, the same class as `fine` grain,
behind a uniform branch on the amount. `scale` is a property of the `cracks`
node for the reason `distortion scale=` is: it does nothing while the amount
is zero.

### 7.2 `iridescence` (rainbow)

```kdl
iridescence 0.8
```

| Parameter | Type | Default | Range |
| --- | --- | --- | --- |
| `iridescence` | float | 0 | 0–1 |

Hook: `specular`. A thin-film hue from the view angle:
`hue = fract(2.5 * (1 - surfaceCosine))`, through the cosine palette
`0.5 + 0.5 * cos(TAU * (hue + vec3(0, 1/3, 2/3)))`. The glint becomes
`mix(specular, specular * palette * 2, amount)`. It runs before the signal
accent mix, so an accent still tints the result. Static and cheap. The
rainbow preset pairs it with `chromatic-aberration`, which is the dispersion
the refracted image carries; iridescence colours the edge light.

### 7.3 `aurora`

```kdl
aurora 0.5 {
    drift-hz 4
    color "#3dffb0"
    color "#7a5cff"
}
```

| Parameter | Type | Default | Range |
| --- | --- | --- | --- |
| `aurora` | float | 0 | 0–1 |
| `aurora` `drift-hz` | float | 4 | 0, or 1–30 |
| `aurora` `color` (× 2) | color | `#3dffb0`, `#7a5cff` | any color |

A block node, because it carries two colours; `color` appears zero times
(both defaults) or exactly twice, and any other count is the validation
error `aurora: expected two color nodes`. `drift-hz` follows the
`ring-drift-hz` rule: `0` pins the field, otherwise at least 1, or the error
`aurora drift-hz must be 0 or at least 1`.

Hook: `emissive`. Two octaves of simplex noise on `p * 0.004 + seed` give a
field `n` in `0–1`; the lookup point also traces a circle of radius 40 noise
units over the period, so the loop is seamless. The colour is
`mix(color_a, color_b, n)` and the emissive term gains
`amount * 0.35 * color * (0.5 + 0.5 * n2) * pow(att, vec3(0.2))`, where
`n2` is a second, coarser octave; the `att` factor is the one the ring uses
so the field sits inside the glass rather than on it.

**Time.** `signal.rs` gains `phase_on(period, hz, now, seed)` and
`next_boundary_on(period, hz, now)`, the existing `drift` and
`drift_next_boundary` generalised over their period; `drift` keeps its 10 s
period and becomes a call to them. Aurora's period is 600 s, so `drift-hz`
steps per second give `hz × 600` buckets anchored to the absolute clock, the
phase is constant within a bucket, and `next_change` is the next bucket
boundary. Redraws are bounded to `drift-hz` per second per visible aurora
window. The motion policy applies through the existing `drift_rate`:
`reduced` halves the rate, `off` and `animations { off }` pin the field at
phase 0. The uniform is `mat_aurora_phase` in radians.

### Out of scope

Particles are not an optic here. They are a lighting question (the lighting
spike `material-1c5a30`, fireflies `material-54bcac`) and a signal question,
and they need a sprite or point pass rather than a per-fragment stage.
`material-f0fc7b` records that its particle clause is deferred to those.

## 8. Presets

Three files, each defining one material named after the file:

`resources/materials/ice.kdl`

```kdl
material "ice" {
    glass {
        ior 1.31
        thickness 28
        attenuation-color "#d6ecff"
        attenuation-distance 80
        backdrop-blur true
        roughness 0.35
        noise 0.06 type="lightness"
        cracks 0.6 scale=1
    }
}
```

`resources/materials/aurora.kdl`

```kdl
material "aurora" {
    glass {
        attenuation-color "#cfe0ff"
        aurora 0.5 {
            drift-hz 4
            color "#3dffb0"
            color "#7a5cff"
        }
    }
}
```

`resources/materials/rainbow.kdl`

```kdl
material "rainbow" {
    glass {
        ior 1.7
        chromatic-aberration 0.5
        iridescence 0.8
    }
}
```

The values are starting points. Each preset is tuned on the headless
harness with `docs/materials/scripts/glass-parameter-sweep.sh` before its
task closes, and the tuned values land in the file and in the evidence doc.

The files install to `/usr/share/niri/materials/` through
`packaging/arch/PKGBUILD`'s `package()` and the `generate-rpm` and `deb`
asset lists in `Cargo.toml`. A user writes

```kdl
include "/usr/share/niri/materials/ice.kdl"

window-rule {
    match app-id="^kitty$"
    material "ice"
}
```

The `include` node accepts absolute paths. `ice` refracts the blurred
backdrop, so it frosts only while the global `blur` block is on; the preset
file says so in a comment. A preset is one material; the focus split is
Prism's, as today.

## 9. Prism

Nothing in this design changes what Prism emits today. Two Prism pieces are
filed, both depending on the material tasks they map:

- **Rack devices for the three optics.** `cracks` (category geometry, after
  Distortion; mix `glass.cracks`, detail `glass.cracksScale`),
  `iridescence` (optic, after Fringing; mix `glass.iridescence`), and
  `aurora` (optic, after Tint; mix `glass.aurora`, details
  `glass.auroraDriftHz`, `glass.auroraColorA`, `glass.auroraColorB`), with
  their bypass keys and whichever `glass.inactive.*` twins the focus matrix
  rule calls for. The sink renders the new nodes; the manifest binds the
  keys; `probe-material` is updated. The deployment rule in
  `.agents/AGENTS.md` applies: Prism emits a node only once the installed
  niri parses it.
- **Starter profiles** named Ice, Aurora, and Rainbow, carrying the preset
  values of section 8, so loading a profile is the material choice. How
  Prism ships a profile it did not save is a Prism design question.

## 10. Tasks

`material-397fcb` becomes a goal with two children:

1. **Optic trait, registry, assembly, and the saturation and noise
   migration** (sections 1 to 5). Done when `just test` passes and the
   byte-identical evidence is recorded.
2. **Parameter table, drift test, and the contributor guide** (section 6
   and `docs/materials/adding-an-optic.md`), plus the `render-pipeline.md`
   and `README.md` updates.

`material-bb3fe5` (ice) depends on child 1: the `cracks` optic, the `ice`
preset, the install plumbing for `resources/materials/`, its docs section,
and a smoke recording its cost.

`material-f0fc7b` becomes a goal with two children, each depending on
child 1: the `aurora` optic with the clock generalisation and its preset,
and the `iridescence` optic with its preset. Each records cost; aurora also
records its redraw rate against `drift-hz`.

The two Prism pieces of section 9 depend on `material-bb3fe5` and
`material-f0fc7b`.

## 11. Verification

- Unit tests in `niri-config` for each optic: parse, defaults, range
  rejection, and every validation error string above.
- Unit tests in `niri` for each optic's `values` and `next_change`,
  including the inherit-or-neutral rule for noise and saturation under all
  four combinations of written value and effective backdrop blur, and
  aurora's phase and boundary under each motion policy.
- A layout test in `src/layout/tests.rs` for the scheduling gate of
  section 3: a tile with an aurora material, unfocused, with a response that
  lights no focus filament and no signal, whose slab band is in view,
  reports a tick deadline equal to aurora's next bucket boundary; the same
  tile with `drift-hz 0` reports none. The aurora smoke repeats the check
  live: two captures of an unfocused, signal-free aurora window one bucket
  apart must differ, and two captures within one bucket must not.
- The byte-identical migration check of section 5.
- A smoke per new optic on the headless host, in the form of the existing
  `docs/materials/scripts/*-smoke.sh`: amount 0 renders identically to the
  unconfigured material; the optic's claimed effect is measurable; frame
  cost is recorded.
- Both section 6 tests stay green: the table matches the specs, and the
  specs match the parser at every range bound and default.
- `tools/upstream-report` is regenerated when files under seams change;
  `src/render_helpers/shaders/mod.rs` is a class B seam and its hunk
  changes.

## Related designs

- [`2026-08-22-v1-design.md`](../materials/2026-08-22-v1-design.md): the
  slab and the compositing contract every optic works inside.
- [`render-pipeline.md`](../materials/render-pipeline.md): the stage order
  the hooks are named for.
- [`2026-09-05-material-glass-noise-saturation-params-design.md`](2026-09-05-material-glass-noise-saturation-params-design.md)
  and [`2026-09-06-material-glass-noise-type-design.md`](2026-09-06-material-glass-noise-type-design.md):
  the two optics that migrate.
- [`2026-09-05-ring-light-focus-response-design.md`](2026-09-05-ring-light-focus-response-design.md):
  the drift clock aurora generalises.
- Prism `docs/specs/2026-09-08-device-chain-rack-design.md`: the rack the
  `OPTICS` order mirrors.
