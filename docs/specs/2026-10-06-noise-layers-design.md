# Noise layers: up to four grain generators per material, each with a size

**Status:** implemented 2026-10-08 (merged at 1ae20983; owner's second look:
"scale 8 looks better"; the B-spline's idle cost on every material draw is
material-03fdf3). Amended 2026-10-07 after the owner's look: §4's lattice is
reshaped from Hermite to cubic B-spline, with cost, §8's correlation test
and a lattice-visibility check to match (amendment review round 3, claude:
accept, four wording fixes applied).
Accepted for planning 2026-10-06 after spec review rounds 1 and
2 (codex) and the owner's go-ahead; amended while planning (§3: the
validation message's form and resolution's bound on an invalid material;
§7.2 (6) and §8: a per-slot check replaces the order check, which seeds
confound, and a fixed-seed hook-ordered reference checks order; plan review
round 1); plan `docs/plans/2026-10-06-noise-layers.md`. Round 2 (2026-10-06: the
`material-config.md` instruction still carried the unqualified collapse
claim). Round 1 (2026-10-06: the fine lattice's anticorrelated corners need
their covariance in the norm, checked by position within the cell; stacked
layers match one layer's variance, not its distribution).
**Task:** `material-3fcba2`, in the material library lane (`material-3aa1f2`).
Folds in `material-829590` (one site per layer), which the noise placement
design filed as this task's follow-up. Wakes `prism-85f63a` (stacked noise on
the rack), whose brainstorm decides how the rack and the bus carry layers.
**Baseline:** `4a8b2072` (the task-start commit; it changes only task records).
**Builds on:** `2026-10-05-noise-placement-design.md` (the `site` property,
the backdrop grain pass, the film hook, the schema's selectors) and
`2026-09-06-material-glass-noise-type-design.md` (the three grain kinds).

## 1. Intent and boundary

A material may write the glass `noise` node up to four times. Each node is
one layer with its own amount, type, site and a new `scale=`: the grain's
cell size in pixels. One layer of fine grain over one of coarse white
mottling, or lightness grain at the glass under white film grain, become
expressible in one material.

The owner chose type *and* size (2026-10-06) because a stack of
independent additive layers of one type at one size and one site, with
nothing clipping, only approximates a single layer: its variance is that of
one layer of amount `sqrt(a² + b²)`, but not its distribution (two uniform
white grains sum to a triangular one; summed fine grains change shape too;
repeated lightness layers clamp to the gamut in between). That difference is
small next to a change of type, site or size, so those are what layers are
for, and size is the axis that makes stacking worth having.

Configs that write at most one `noise` node and no `scale=` render
byte-identical to the baseline at every site. Opaque client pixels never
change. The shipped KDL keeps one node: prism writes one until
`prism-85f63a` designs the rack side.

## 2. What the code establishes

- **One node, one optional field.** `Glass.noise: Option<Noise>` with
  `Noise { amount, kind, site }` (`niri-config/src/material/optics/noise.rs`);
  `resolve` yields `ResolvedNoise { amount: Option<f64>, kind, site }`, and
  `ResolvedGlass` is `Copy`. A material is defined once; includes add whole
  materials and reject a duplicate name (`niri-config/src/lib.rs`), so no
  merge rule touches the node.
- **Repeated children decode already.** `#[knuffel(children(name = "…"))]`
  collects every node of a name into a `Vec` (aurora's `color`, the
  material's `response`).
- **The inherit rule lives in the optic.** `NoiseOptic::values`
  (`src/render_helpers/material/optics/noise.rs`) uses the written amount,
  else `blur { noise }` while backdrop blur is effective, else 0. A written
  node never inherits.
- **Three hooks share one file.** `noise.frag` holds `noise_behind` (glass,
  stage 3b), `noise_post` (film, stage 9) and `noise_source` (backdrop, the
  effect program's grain pass). All seed `fragCoord + (47, 113)`; white is
  `hash12 - 0.5`, fine is `fineGrain` (nine hashes, unit-normalised to
  white's deviation), lightness moves Oklab L by the fine value.
  `noise_behind` keeps the pre-site arithmetic byte for byte, including the
  lightness kind's direct linear return.
- **The backdrop is one grain per output.** `backdrop_grain` checks that
  every material placing noise at the backdrop agrees on `(amount, kind)`
  and returns the agreed `BackdropGrain`; `niri.rs` hands it to every effect
  buffer as `GrainOptions`, whose change invalidates the grain, the blur and
  both pyramids and bumps the commit counter (`effect_buffer.rs`).
- **The fan-out precedent is four slots with constant bounds.** The signal
  impulses pack four slots into `vec4` uniforms and walk them with ternary
  selection under a constant bound (`main.frag`, `prelude.frag`), which
  GLSL ES 1.00 accepts on every driver niri targets.
- **The schema has no parameter list and no multiplicity.**
  `resources/materials/pipeline.json` carries sites, stages and interactions.
  Stages name parameters by node (`noise`, `noise type=`, `noise site=`); the
  three noise stages are picked by `noise site=` selectors. Prism's loader
  ignores unknown keys, but its schema design says a new field bumps the
  version and revises that design first.
- **The pixel tests and smokes exist.** `src/tests/noise_site.rs` renders
  the material program in process under a frozen clock;
  `docs/materials/scripts/glass-noise-site-smoke.sh` compares against a
  baseline binary; `noise-placement-cost.sh` counts and times the grain and
  material spans under Tracy.

## 3. Configuration

```kdl
material "film-stock" {
    glass {
        noise 0.04 type="lightness"
        noise 0.03 type="white" scale=4
        noise 0.02 type="fine" site="film"
    }
}
```

- **Repeated node.** `Glass.noise` becomes `Vec<Noise>` decoded with
  `children(name = "noise")`. `NOISE_LAYERS = 4`. A fifth node fails
  `Material::validate` with
  `material film-stock: at most 4 noise layers, found 5` (the file's
  `material <name>:` form). The invalid material still reaches the
  post-include backdrop check, so resolution takes at most four nodes rather
  than assuming validation passed.
- **`scale=`.** `Noise` gains `#[knuffel(property)] scale:
  Option<FloatOrInt<1, 16>>`. Omitted is 1. The unit is the carrier's pixel:
  physical output pixels at every site, as today's grain is (so `scale=4` on
  a 2× output is a two-logical-pixel cell).
- **Resolution.** `ResolvedNoise { layers: [Option<ResolvedNoiseLayer>;
  NOISE_LAYERS] }` with `ResolvedNoiseLayer { amount: f64, kind: NoiseType,
  site: NoiseSite, scale: f64 }`, in config order; it stays `Copy`. All
  slots `None` means no node was written, and only then does the inherit
  rule apply: slot 0 becomes white grain at the glass site, scale 1, with
  the inherited amount. A written layer has a definite amount and never
  inherits, as today.
- **Amount 0 is a layer.** A node at amount 0 keeps its slot and does
  nothing, so a rack can mute a layer without renumbering the rest.
- **Order.** Each site applies its layers in config order; sites run in
  pipeline order (backdrop, glass, film). White and fine layers commute (they
  add in encoding, unclamped); a lightness layer does not, so its position
  among the layers at its site matters.
- **Seeds.** Slot `k` seeds `fragCoord + O_k`, with `O_0 = (47, 113)`
  (today's), `O_1 = (1301, 2659)`, `O_2 = (3709, 977)`,
  `O_3 = (2381, 3917)`; every offset stays below 4096 so `hash12`'s
  precision at the far edge of an 8K output is no worse than today's. Glass
  and film layers use their material slot. Backdrop layers use their
  position in the backdrop list (§5), so the output's grain does not depend
  on where a material wrote its backdrop node. A single node in slot 0 seeds
  as today at every site, which keeps the placement design's identities. The
  coincidence left: backdrop layer k and a material's glass or film layer in
  slot k share a seed, so their patterns coincide. For k = 0 that is the
  placement design's identity case (backdrop grain lands where glass grain
  would at `ior 1`); for any k, refraction and blur decorrelate them, and
  without either the two add coherently rather than in quadrature.
- **Backdrop agreement over lists.** The materials that place any layer at
  the backdrop must agree on the ordered list of their backdrop layers'
  `(amount, kind, scale)`. The check stays beside `validate_material_refs`
  at `recursion == 0`. Its error names both materials and both lists:

  ```
  materials "a" and "b" place different noise at the backdrop
  ([0.3 fine], [0.3 fine, 0.1 white scale 4]); backdrop grain is one setting per output
  ```

  `Config::backdrop_grain()` returns `Option<BackdropGrain { layers:
  [Option<BackdropLayer>; NOISE_LAYERS] }>`: `None` when no material places
  noise there. Validation guarantees agreement, so the accessor takes the
  first match.
- **`material-config.md`.** The noise paragraph says: up to four nodes, the
  order rule, the seed rule, `scale=` and its unit, and §1's qualified
  equivalence: independent additive layers of one type and size at one site,
  with nothing clipping, have the variance of one layer of amount
  `sqrt(a² + b²)` but not its distribution, so they are close to redundant.

## 4. The material shader

- **Uniforms.** `NoiseOptic` declares four `vec4` uniforms, one component
  per slot: `mat_noise` (amount), `mat_noise_type`, `mat_noise_site`,
  `mat_noise_scale`. An empty slot is amount 0, scale 1. `values` fills slot
  0 from the inherit rule when no node is written. The effect program's
  grain pass sets the same uniforms from `GrainOptions` (§5).
- **One grain value per layer.** `float noiseValue(vec2 fragCoord, float
  type, float scale, vec2 offset)` returns the signed, amount-free grain:
  - `scale <= 1.0`: today's per-pixel path, unchanged: `hash12(fragCoord +
    offset) - 0.5` for white, `fineGrain(fragCoord + offset)` for fine and
    lightness.
  - `scale > 1.0`: value noise on a lattice of cell `scale`, reconstructed
    with the uniform cubic B-spline over the 4×4 lattice points around the
    pixel. `p = fragCoord / scale`, `i = floor(p)`, `t = fract(p)`, weights
    per axis `b(t) = ((1-t)³, 3t³-6t²+4, -3t³+3t²+3t+1, t³) / 6` for the
    points `i - 1` to `i + 2`, `w = bx ⊗ by`. The sixteen values `g` at those
    points plus `offset` are white (`hash12 - 0.5`) or fine (the lattice's
    own high-pass: each point's hash minus the mean of its eight neighbours,
    times 0.94280904); the fine values share one 6×6 block of thirty-six
    hashes. The value is `Σ w g / sqrt(wᵀ C w)`, where `C` is the values'
    correlation matrix: dividing by the reconstruction's own deviation keeps
    the grain's deviation constant across the cell, so `amount` means the
    same deviation at every scale and every position in the cell. Because
    `w` is separable and `C` depends only on the offset between two points,
    `wᵀ C w = Σ_d C(d) X(|dx|) Y(|dy|)`, with `X(k) = Σ_j bx_j bx_{j+k}` and
    `Y` the same over `by`. White values are independent, so
    `wᵀ C w = X0 Y0`. Fine values correlate up to two points apart: `-1/6`
    adjacent, `-7/36` diagonal, `1/24` at (2, 0), `1/36` at (2, 1), `1/72` at
    (2, 2) (`var = 9/8` of a hash's before the 0.94280904). So for fine and
    lightness
    `wᵀ C w = X0 Y0 - (1/3)(X1 Y0 + X0 Y1) - (7/9) X1 Y1 + (1/12)(X2 Y0 + X0 Y2)
    + (1/9)(X2 Y1 + X1 Y2) + (1/18) X2 Y2`, each coefficient the
    correlation times the number of offsets that share it. `C` is positive
    definite (smallest eigenvalue 0.113): the smallest `wᵀ C w` is about
    0.089 for fine and 0.21 for white, so the denominator never falls below
    about 0.30.
  - *Why the B-spline (owner's look, 2026-10-07).* The first lattice used
    Hermite (`smoothstep`) interpolation of the 2×2 corners. Its derivative
    is zero at every lattice point, so the grain went flat at the points and
    steep along the cell edges: the owner saw scale-8 white grain as blocky,
    a grid of squares. Rotating that lattice only tilts the squares. The
    cubic B-spline is C², and its weights stay within a narrow band across
    the cell. In an offline prototype of this arithmetic, the mean squared
    gradient by position in the cell stayed within 0.54 to 1.59 of its mean
    at scales 2 to 8 for both types, against 0.00 to 4.00 for Hermite. §8
    pins that with the lattice-visibility check. Blobs come out rounder and
    somewhat larger at the same `scale`; `scale` remains the lattice cell,
    not a blob diameter.
  - Crossing `scale` 1 re-rolls the pattern (the per-pixel path seeds pixel
    centres, the lattice seeds integer lattice points) and coarsens it in one
    step: the B-spline approximates rather than interpolates, so just above 1
    the grain is visibly smoother than the per-pixel path. Dragging the scale
    across 1 shows that once; nothing above 1 re-rolls.
- **Applying a layer.** `vec3 noiseApply(vec3 encoded, float grain, float
  type)`: white and fine add `grain` in encoding; lightness goes through
  today's `noiseLightness`. This is the existing `noiseEncoded` split at the
  seam between the value and its application, so the film and backdrop
  paths keep their arithmetic.
- **Four explicit calls, no indexing.** Each hook calls a per-slot step four
  times with `.x`, `.y`, `.z`, `.w` and `O_0` to `O_3`, each gated by
  `amount > 0.0 && site == S`. No dynamic vector indexing and no loop: the
  impulse precedent's constant bound, written out.
- **`noise_behind` (glass).** Returns `color` untouched when no slot is
  active at the glass, as today. Otherwise it encodes once and applies the
  active layers in slot order. A white or fine layer leaves the value
  encoded; a lightness layer produces today's clamped linear result, which
  is re-encoded only if another glass layer follows. The function returns
  linear light: decoded after a white or fine last layer, directly after a
  lightness last layer. With one active layer this is the baseline
  arithmetic, lightness's direct return included; but the NVIDIA driver
  compiles the layered lightness path one code off the baseline at one pixel
  in the smoke (`2026-10-06-noise-layers-evidence.md`), so when slot 0 is the
  only active slot at any site and is a glass layer at scale 1,
  `noise_behind` runs the baseline body verbatim. A lone glass layer beside
  backdrop or film layers takes the layered path; that config has no
  baseline to match.
- **`noise_post` (film).** Applies the film-site layers in slot order to the
  encoded glass, as today's single call does.
- **Cost.** Per active layer per fragment: one hash (white, scale 1), nine
  (fine or lightness, scale 1), sixteen (white, scale above 1), thirty-six
  (fine or lightness, scale above 1), plus an Oklab round trip for
  lightness. Four fine layers above scale 1 cost 144 hashes per glass
  fragment where today costs at most nine. Inactive slots cost a uniform compare. §7.3 measures
  it; the material renders only when damaged, so settled glass pays
  nothing.

## 5. The backdrop site

- `GrainOptions` becomes `{ layers: [Option<GrainLayer { amount: f32, kind:
  NoiseType, scale: f32 }>; NOISE_LAYERS] }`, filled in backdrop-list order
  from `BackdropGrain`. Change detection compares the whole array, so adding,
  removing or editing any backdrop layer runs the existing cascade (grain,
  blur, both pyramids, commit counter). Nothing else in `effect_buffer.rs`
  changes.
- `noise_source` applies the slots in order to the straight colour through
  `noiseValue` and `noiseApply`, then clamps once, as today's single layer
  does before re-premultiplying. With one slot it is the baseline arithmetic.
  `mat_noise_site` stays unconsulted in the effect program: only backdrop
  layers reach it.

## 6. The schema

- **One new parameter.** `ParamSpec` node `"noise scale="`,
  `ParamKind::Float { default: 1, min: 1, max: 16, unit: "px" }`, write
  `noise 0.5 scale=<v>`, read taken from slot 0 like the `noise` row. The
  generated table in `material-config.md` gains its row.
- **Stages.** `noise` owns `noise scale=` beside its other three; all three
  noise stages list it under `reads`. `every_parameter_is_owned_by_exactly_one_stage`
  holds. The GLSL-reads pin maps `mat_noise_scale` to `"noise scale="`.
  The hook pins are unchanged: each hook is still called exactly once per
  program; the four slots live inside the hook.
- **No multiplicity field.** How many times a node may repeat is a fact of
  the configuration, not of the pipeline's sites and stages, and prism's
  schema design requires a version bump and its own revision before the file
  gains a field. `pipeline.json` stays at version 1 with only the new
  parameter in the noise stages' lists. The bound is `NOISE_LAYERS` in
  `niri-config` and the `material-config.md` paragraph. `prism-85f63a`
  decides whether the rack needs it in the schema; that decision gets a
  note on the task when this design is accepted.
- **Generated file and prism's copy.** `pipeline.json` regenerates under
  `MATERIAL_DOCS_UPDATE=1`; prism's vendored `defs/rack/pipeline.json` is
  refreshed at merge in a one-line commit with the rack unchanged, as the
  placement work did. The rack reads `owns` and `reads` as lists of strings,
  so the new name passes its validator.

## 7. Evidence

Written into one evidence document under `docs/materials/`, dated the day
the captures run, with `tools/capture-meta show` output for each lane.

### 7.1 The look

A contact sheet from the headless Weston lane, built on
`glass-optic-smoke-lib.sh` like the noise-type smoke, over its flat warm
backdrop with one textured region and a transparent kitty at `ior 1`:

- fine at scale 1, 2, 4, 8 (amount 0.3);
- white at scale 1, 2, 4, 8 (amount 0.3);
- lightness at scale 1 and 4 (amount 0.3);
- stacks: white scale 4 under fine scale 1; lightness at the glass under
  white film grain; four fine layers at scale 1 and amount 0.15 beside one
  fine layer at amount 0.3. These two have the same variance and different
  distributions (§1); the cell shows whether that difference is visible, which
  decides how the rack explains redundant layers.

The owner looks once and records the verdict in the evidence document's
"Owner's look" line, judging above all whether the lattice shows at scale 8.
A visible lattice is a finding that reshapes `scale > 1`, not a reason to
drop sizes. The first look (2026-10-07) found the Hermite lattice blocky at
scale 8; §4 now uses the B-spline, and the owner looks again at the reshaped
sheet.

### 7.2 Captures and statistics

The same lane, the difference against the amount-0 capture of the same
fixture, the noise-type smoke's statistics (`sd`, downsampled `sd`,
low-frequency ratio, determinism).

1. Determinism: two captures of each cell are identical.
2. Byte identity against a baseline binary built from `4a8b2072`, same
   config, absolute error 0: one node of each kind at each site, no
   `scale=`; and no node at all with `blur { noise }` inheriting.
   Since the merge (`1ae20983`) the baseline is its first parent, `37de154e`:
   glass edges (material-be611b) landed beside this work and change bevel
   pixels by design, so a `4a8b2072` build no longer matches the merged
   branch (material-a8232f).
3. `scale=1` written equals `scale` omitted (absolute error 0).
4. Independence: two fine layers at amount 0.2 each, scale 1, give a grain
   `sd` within 5 % of `sqrt(2)` times one such layer's.
5. Normalisation: for white and fine, the grain `sd` at scale 2, 4 and 8 is
   within 10 % of scale 1's, while the low-frequency ratio rises with scale.
   The aggregate alone cannot see a contrast grid (the independent-corner
   norm on fine grain passes it at about 91 % while its cell centres sit at
   69 %), so the same cells are also binned by position within the lattice
   cell, `fract(fragCoord / scale)` on the pixel centres (64 bins at scale
   8), and every bin's `sd` must be within 10 % of the cell's aggregate.
6. Every slot is applied and seeded apart: fine grain at amount 0.3 alone
   in slot `k` (earlier slots `noise 0`), for `k` 1 to 3, has a grain `sd`
   within 5 % of slot 0's, and the `sd` of its difference from slot 0's
   grain is at least slot 0's `sd` (an identical pattern would give 0).
   Swapping two layers' order cannot test application: it swaps their seeds
   too, so the result differs even when one layer is never applied. Order
   is checked in process instead (§8) against a fixed-seed reference.

### 7.3 Cost

`noise-placement-cost.sh`'s Tracy tooling on the headless lane, timing the
material program's GPU span over a fixed number of damaged frames for: no
noise; one fine layer at scale 1 (today's case); four fine layers at
scale 1; four fine layers at scale 8; one backdrop fine layer at scale 8
against one at scale 1 (the grain pass span). Per-fragment cost is the
span difference divided by the glass area. The numbers go in the evidence
document and, as prose, into prism's interaction document under the cost
class when `prism-85f63a` lands.

## 8. Verification

- **Config (`niri-config`).** One to four nodes parse in order; a fifth
  fails with the §3 message; `scale=` parses within 1 to 16, omits to 1 and
  rejects 0.5 and 17; no node resolves to all-`None` slots and inherits;
  amount-0 layers keep their slots. Backdrop agreement: equal lists pass,
  lists differing in length, order, amount, kind or scale fail naming both
  materials; a material with only glass or film layers is not consulted;
  `backdrop_grain()` returns the list or `None`. The parameter table test
  regenerates with the `noise scale=` row.
- **Optic.** `NoiseOptic::values` packs slots in config order, inherits
  only into slot 0 and only with no node, and leaves empty slots at amount 0
  and scale 1.
- **Effect buffer.** `GrainOptions` equality over the array: an edit to any
  slot triggers the cascade, an equal array does not.
- **Fine-lattice correlation.** A unit test derives the fine values'
  correlations by enumerating the high-pass definition's hash coefficients
  (`-1/6`, `-7/36`, `1/24`, `1/36`, `1/72` at offsets (1, 0), (1, 1),
  (2, 0), (2, 1), (2, 2), and 0 beyond) and checks that the coefficients in
  `noise.frag`'s norm (`-1/3`, `-7/9`, `1/12`, `1/9`, `1/18`) are each
  correlation times its offset count, and that the shader's `x1` and `x2`
  sums with those coefficients equal the brute-force `wᵀ C w` over the
  sixteen values at sampled cell positions, so the shader and the
  definition cannot drift apart.
- **Schema.** The existing pins run against the new tables, including the
  GLSL-reads map with `mat_noise_scale`; the generated file is fresh.
- **Pixels, in process (`src/tests/noise_layers.rs`).** Frozen-clock
  renders: one layer equals the same layer followed by three amount-0 layers
  (absolute error 0) at each site; `scale=1` equals omitted; the per-slot
  check of §7.2 (6); slot order at each site, against a reference whose
  order the hooks fix (lightness in slot 0 at the glass, white in slot 1 on
  the film, amount 0.5 each): the same two layers stacked at the glass, on
  the film and at the backdrop must match it, and reversing a hook's slot
  order must fail; the
  independence and normalisation statistics of §7.2 (4 and 5), the
  per-position bins included, and a lattice-visibility check: at scales 2,
  4 and 8 for white and fine, the mean squared difference between
  horizontally and vertically adjacent grain pixels, grouped by the
  position in the cell of the pair's left or upper pixel, stays within 0.4 to 2.0 of its mean (the Hermite lattice the
  owner saw as blocky fails it by far: about 0.01 and 2.6 at white scale 8);
  all computed over the glass area of an in-process render, so they gate every build and the smoke repeats them on real
  hardware; a two-material backdrop config with
  two layers renders and its effect buffer's commit counter advances when one
  layer's scale changes on reload.
- **Smoke and cost.** §7.1 to §7.3 run once on the headless lane; the
  scripts stay in `docs/materials/scripts` for reruns. Byte identity against
  the baseline is the smoke's (§7.2, 2), since one build cannot hold both.
- **Gates.** `just test-fast` while working, `just check` before commits,
  `just gate` before the merge. No live desktop is used.

Acceptance: every §8 test green; the evidence document holds the sheet, the
owner's verdict, the six assertions and the cost table; `material-config.md`
and `render-pipeline.md` (stage 3b, stage 9, §1 step 2 and §4's noise bullet
name layers and `scale=`, §5's table gains `noise scale=`) match the code;
the generated schema is fresh and prism's copy is refreshed at merge.

## 9. Follow-ups

- `material-829590` is folded in here and dropped with a pointer to this
  document when the spec is accepted.
- `prism-85f63a`: a note that the renderer accepts four layers with
  `scale=`, that the schema carries no multiplicity, and that its brainstorm
  owns the bus and rack representation.
- The owner found the first (Hermite) lattice visible at scale 8 (§7.1,
  2026-10-07); §4's B-spline reshape replaced it on this branch, so no
  follow-up task.

## Out of scope

More than four layers; a user-facing seed; grain with a time term; blend
modes other than additive-in-encoding and lightness; precomputed grain
textures; a schema multiplicity field and any prism change beyond the
vendored file's refresh; film-site saturation (`material-a7e72e`); the
window-site `blur { noise }`.
