# Noise placement: a site attribute selecting backdrop, glass, or film grain

**Status:** accepted for planning 2026-10-05 after spec review rounds 1 and 2 (codex) and the owner's go-ahead; §7.2 assertion 5 amended 2026-10-05 during the quiet capture run (monotone down to the 8-bit floor); plan `docs/plans/2026-10-05-noise-placement.md`. Round 2 (codex, 2026-10-05: the
damage test asserts the commit counters rather than pixels of a fresh
render; the ring fixture waits for the comet's full run, tail included).
Round 1 (codex, 2026-10-05: the
grain change must publish damage through the commit counter; the blurred
branch must blur the grained source; the lightness arithmetic is kept
byte-for-byte and checked against a baseline binary; roughness evidence
needs an `ior` above 1; the ring-band expectation is derived from the
transfer curve). Design approved in conversation 2026-10-05: explorable
placement is the goal; a soft backdrop look is a finding, not a failure.
The §7.1 simulation has run; its table is in §7.1 and its sheets await the
owner's look.
**Task:** `material-cf32e5`; the first device that can move. Wakes
`prism-be5abe` (the rack side) and `material-3fcba2` (noise layers) after
design and plan review.
**Baseline:** `b261ad1a` (the task-start commit; it changes only the task).
**Contract:** prism `docs/specs/2026-10-04-pipeline-schema-design.md`
(accepted 2026-10-05), Section 5, which already names the parameter, the
three stages and their selectors. This design fills in the renderer.

## 1. Intent and boundary

The glass `noise` node gains a `site` property with three values. `glass`
is today's placement: grain on the averaged backdrop taps, before
attenuation, per material. `backdrop` grains the per-output effect-buffer
texture before the Kawase blur and the prefilter pyramids, so the blur and
the roughness filter act on grained content and every glass window on that
output refracts the same grained backdrop. `film` grains the encoded glass
after ring, aurora, glint and sweeps, at the post hook that has been empty
since the render-order work.

The goal is that noise can sit at each legal site and that the renderer, its
schema and the rack tell the truth about what each site does: scope, cost
class, what the grain covers and what softens it. The frosted blur averages
hundreds of pixels at its default three passes, so backdrop grain under a
frosted window survives as faint low-frequency mottling rather than grain
(predicted; the 2026-10-05 captures measured none above the 8-bit floor at
three passes or at any roughness above 0, see the evidence document).
That is recorded as evidence (§7) and as a conditional dependency in the
interaction document, not treated as a defect. A site is dropped before the
contract only if its look is useless at every blur setting; backdrop grain
under a sharp window is ordinary grain that moves with the image, so no
site is expected to fail that test.

Configs that name no site render byte-identical to today. Opaque client
pixels never change at any site.

This task delivers the reviewed design and then an implementation plan for
separate review. The renderer work lands in this repository; the prism side
(`glass.noiseSite`, the rack device moving between sites, the panel's
explanations) is `prism-be5abe` and is out of scope here except for the
schema file prism vendors (§6).

## 2. What the code establishes

- **The effect buffer is per output and shared.** Each output owns an
  `Xray` with two `EffectBuffer`s per render target
  (`src/render_helpers/xray.rs`). Every material element on the output
  samples them, and so does the background-effect element under windows
  whose rule asks for one. Their options are the global `blur { passes;
  offset }`, handed down in `update_xray_render_elements` (`src/niri.rs`)
  through `EffectBuffer::update_blur_options`. Nothing per window reaches
  the buffer. A grain applied there has one amount and one type per output.
- **The sharp texture is the root of three consumers.** `prepare_offscreen`
  renders the background layers into the sharp texture; damage there clears
  the blurred texture and invalidates both prefilter pyramids
  (`effect_buffer.rs`, `prepare_offscreen`). `render` returns the sharp
  texture or the cached blur; `render_prefiltered` builds a pyramid over
  whichever the material's source gate selects and picks levels by
  `roughness * clamp(ior * 2 - 2, 0, 1)`. Grain on the sharp texture is
  therefore seen by sharp windows, frosted windows and rough windows alike.
- **The blur and the pyramid are the same program family.**
  `BlurProgram` compiles `blur_down.frag` and `blur_up.frag` under
  `blur.vert`; the prefilter downsample reuses the down pass
  (`blur.rs`, `render_downsample`). An effect-program shader is a `#version
  100` fragment program over `v_coords` and one sampler.
- **Today's grain is screen-seeded and static.** `noise_behind`
  (`shaders/material/noise.frag`) hashes `gl_FragCoord.xy + (47, 113)`
  through `hash12` or `fineGrain` from the prelude, adds the grain in sRGB
  encoding and returns linear light; the lightness kind goes through Oklab
  with a gamut clamp. No time term. The amount follows the inherit-or-neutral
  rule in `NoiseOptic::values` (`material/optics/noise.rs`): an omitted
  node inherits `blur { noise }` while backdrop blur is effective and is 0
  otherwise; a written node applies regardless.
- **The node's amount is a required argument.** `Noise { amount, kind:
  Option<NoiseType> }` (`niri-config/src/material/optics/noise.rs`). A
  property on the node can only appear beside a written amount, so a site
  never meets the inherit rule.
- **The post hook is empty and already a site.** `main.frag` encodes
  `transmitted + within + specular + emissive` and carries a comment where
  post hooks go, before the coverage multiply. The schema lists `post` as a
  site (carrier `encoded`, law `sequence`, coverage `glass`, cost
  `fragment`) with no stage; `pipeline_post_site_is_empty_until_a_stage_lands`
  (`shaders/mod.rs`) pins that.
- **The schema's selector rules exist and are unused.** `Selector { param,
  variant }` and `check_selectors` (`niri-config/src/material/pipeline.rs`)
  require every variant of an `Enum` parameter to select exactly one stage
  and all selecting stages to be one optic. `STAGES` has no selector today;
  negative fixtures prove the rules.
- **Hook pins are per program.** The material program must call each
  material hook exactly once in stage order; the effect program is pinned
  as one call across `blur_down.frag` and `blur_up.frag` concatenated
  (`pipeline_other_programs_call_their_hooks_exactly_once`); the GLSL-reads
  pin maps each `mat_*` uniform an optic declares to the parameter node it
  reads.
- **Cross-material validation has a home.** Per-material rules run in
  `Material::validate`; rules over the whole table run once after every
  include has merged, where `validate_material_refs` runs
  (`niri-config/src/lib.rs`, `recursion == 0`).
- **Pixels can be tested in process.** `src/tests/ring_pair.rs` renders
  the real material program through the headless backend's surfaceless GLES
  renderer under a frozen clock (`render_at`, `diff`), and the fixture's
  test client can map layer-shell surfaces (`src/tests/layer_shell.rs`), so
  the effect buffer can be given content without a nested compositor.
- **The prism rack stays valid when the stages land.** `validateRack`
  (prism `src/rack.js`) fails only for a stage that owns a node prism writes
  and has no device. The two new stages own nothing (§6), so the vendored
  schema can move ahead of `prism-be5abe`.

## 3. Configuration

```kdl
material "terminal-glass" {
    glass {
        noise 0.3 type="fine" site="backdrop"
    }
}
```

- `site` is a string property on `noise`, decoded like `type`: `NoiseSite {
  Glass = 0, Backdrop = 1, Film = 2 }` with `NAMES = ["glass", "backdrop",
  "film"]` and a `FromStr` whose error is `unknown NoiseSite value: <s>`.
  Omitted is `glass`. `ResolvedNoise` gains `site: NoiseSite`, default
  `Glass`; `resolve` copies it. The inherit rule is untouched: `site` can
  only be written beside an amount, and an omitted node resolves exactly as
  today.
- **One `ParamSpec`:** node `"noise site="`, `ParamKind::Enum { default:
  "glass", variants: NoiseSite::NAMES }`, `write: noise 0.5 site="<v>"`,
  `read: None` (as `noise type=`). The generated parameter table in
  `material-config.md` gains its row under `MATERIAL_DOCS_UPDATE=1`.
- **Agreement at the backdrop.** After the config has merged, the materials
  whose noise sits at `backdrop` must agree on amount and type. Two that do
  not fail decoding with one error naming both:

  ```
  materials "a" and "b" both place noise at the backdrop with different
  settings (0.3 fine, 0.1 fine); the backdrop is one texture per output
  ```

  The check lives beside `validate_material_refs`, runs at `recursion == 0`,
  and compares the resolved `(amount, kind)` pairs. A material with noise
  at `glass` or `film` is not consulted: those placements are per material.
- **The derived per-output value.** `Config::backdrop_grain() ->
  Option<BackdropGrain { amount: f64, kind: NoiseType }>` returns the agreed
  pair, or `None` when no material places noise at the backdrop. Validation
  guarantees agreement, so the accessor takes the first match and has no
  conflict branch. The compositor hands it to every effect buffer in the
  same loop that hands them the blur options, on load and on reload.
- **Independence from the blur switch.** Backdrop grain is a grain, not a
  blur: it applies whether or not `blur { off }` is set and whether or not
  the window's `backdrop-blur` selects the blurred source. What differs is
  what the window sees of it (§4, §7).
- **The window-site element.** The background-effect element under a
  window whose rule asks for `background-effect { blur true }` samples the
  same texture, so a non-prism window with a blurred background effect shows
  grained backdrop once any material places noise there. That is what
  coverage `backdrop` means in the schema. Prism's terminal rule pins that
  element inert (`blur false; noise 0; saturation 1`), so prism-managed
  windows see the grain only through the material element. The reference
  documents say this once (§9).

## 4. The backdrop site: a grain pass in the effect buffer

- **Options.** `EffectBuffer` gains `grain: Option<GrainOptions { amount:
  f32, kind: NoiseType }>` and `update_grain_options(Option<GrainOptions>)`.
  A change (including `Some` to `None`) clears the grained texture, clears
  the blurred texture, invalidates both prefilter pyramids, and increments
  the buffer's `commit_counter` whenever an offscreen exists. The counter
  is how consumers learn that the buffer's content changed without their
  own state changing: the tile's material fingerprint carries `(bg.id(),
  bg.commit())` (`src/layout/tile.rs`) and the xray elements re-render on a
  changed `commit()` (`src/render_helpers/xray.rs`). `update_blur_options`
  increments it only when a blurred texture existed, because the sharp
  texture is unchanged; grain changes the sharp consumers' source too, so
  it increments unconditionally. Unchanged options do nothing.
- **Texture.** `Offscreen` gains `grained: Option<GlesTexture>`, the same
  size and format as the sharp texture (`Abgr8888`, output pixels). A
  private `source(&self) -> &GlesTexture` returns the grained texture when
  grain is set and prepared, the sharp texture otherwise. Every read of
  `offscreen.texture` as content moves to `source()`: `prepare_blur`'s
  `prepare_textures` (size), both branches of `render` (the `!blur` return
  and the `blur.render(renderer, &offscreen.texture, ..)` draw, which is
  the one that produces what frosted windows see), and `render_prefiltered`
  for the sharp pyramid's base and level-0 sample. The blurred pyramid's
  base is the blurred texture, now a blur of the grained one. The only
  remaining uses of `texture` itself are its creation, the damage render
  into it and the grain pass's input.
- **Pass.** `prepare` runs `prepare_grain` after `prepare_offscreen` when
  grain is set and `grained` is `None`: lazily compile `GrainProgram`
  (as `Blur::new` compiles its programs), create the texture if missing,
  draw one full quad from the sharp texture into it under `blur.vert`.
  A successful program is cached on its owning EGL share group and borrowed
  by the buffers; reconnections reuse that one program, and context teardown
  releases it. Failed compilations do not enter the cache. Framebuffer and
  draw errors return to the sharp-source fallback after releasing the quad's
  framebuffer and consuming its GL errors.
  Tracy spans `EffectBuffer::prepare_grain` (CPU) and `Grain::render` (GPU,
  through `with_profiled_context`) bracket it, so §7 can count and time it.
- **Invalidation.** `prepare_offscreen` already clears the blurred texture
  and invalidates both pyramids when the sharp texture is damaged; it now
  clears `grained` too, in the same branch. The three invalidation causes
  (sharp damage, grain option change, renderer context change) and what
  each clears are factored into one pure function with a table test, so
  the cascade is readable without a renderer.
- **Shader.** The grain program is an effect-program source assembled as:
  the effect header (`#version 100`, precision, `varying vec2 v_coords`,
  `uniform sampler2D tex`), the shared helper block, the noise optic's GLSL,
  and a `main` that calls `noise_source` exactly once. The helper block is
  the part of `prelude.frag` the grain formulas need (`srgbToLinear`,
  `linearToSrgb`, `hash12`, `fineGrain`, `linearToOklab`, `oklabToLinear`),
  moved to `shaders/material/common.frag` and included from both programs
  so the formulas have one home; the material source stays the same text
  in the same order, which the existing composition test keeps pinning.
- **`noise_source`.** `vec4 noise_source(vec4 texel, vec2 fragCoord)`: a
  texel with `a == 0` returns unchanged; otherwise un-premultiply, apply the
  grain core (below) to the straight sRGB-encoded color, clamp to `[0, 1]`
  (the texture is 8-bit and premultiplied, so signed excursions cannot be
  stored), re-premultiply. Seed: `fragCoord + (47, 113)`, output pixels,
  the same offset as the glass site, so under a sharp window at `ior 1` with
  no displacement the backdrop grain lands where the glass grain would.
  Static, like today's grain; the texture is cached.
- **Failure.** A grain program that fails to compile or draw logs a
  warning once, marks the buffer's grain as failed, and `source()` returns
  the sharp texture until the next invalidation, when it retries. This is
  the prefilter's failure pattern (`PrefilterStatus::Failed`,
  `prefilter_failure_retries_only_after_invalidation`). A compositor cannot
  fail early at frame time; the warning names the pass.
- **Scope and cost.** The schema records the new stage with scope `output`
  at site `source`, cost class `cached`: the pass runs once per sharp
  damage and once per option change, and its result is shared by every
  consumer on that output. A backdrop that redraws every frame pays the pass
  every frame, before the blur it already pays. Dragging the amount
  invalidates the grain, the blur and both pyramids on every change. §7
  measures all three.

## 5. The glass and film sites in the material shader

- **Uniform.** `NoiseOptic` declares `mat_noise_site` beside `mat_noise`
  and `mat_noise_type`, and `values` emits `glass.noise.site as u8 as f32`
  third. The amount and type uniforms are computed as today for every site;
  at `backdrop` they are unused by the material program (the grain is in
  the texture), and the schema says so by listing them under `reads` of the
  backdrop stage rather than giving the material a second amount.
- **`noise_behind` keeps its arithmetic.** The function gains one gate,
  `mat_noise_site` must be `glass`, beside the amount-0 early return, and
  nothing else changes: white and fine still encode, add signed grain and
  decode; lightness still returns its clamped linear Oklab result directly,
  without an encode-decode round trip. A shared "encoded in, encoded out"
  core was considered and rejected: the extra round trip on the lightness
  path can move quantized pixels by one code, and the omitted-equals-glass
  identity cannot see a regression both share. The byte-identity claim is
  therefore checked against a baseline binary (§7.2, §8), not only within
  one build.
- **`noise_post` and `noise_source` share helpers, not a body.** Both are
  written in `noise.frag` beside `noise_behind` from the same prelude
  helpers (`hash12`, `fineGrain`, the transfer and Oklab functions) and the
  same seed offset, each in its own carrier: `noise_post` takes encoded
  glass, adds white or fine grain in encoding, and for lightness decodes,
  moves Oklab lightness on the clamped color, clamps and re-encodes;
  `noise_source` takes a premultiplied texel (§4). A grain-kind switch
  shared by the three functions keeps one `if` ladder per kind rather than
  one formula per site.
- **`noise_post`.** `vec3 noise_post(vec3 encoded, vec2 fragCoord)`:
  returns `encoded` unless the site is `film` and the amount is above 0.
  `main.frag` calls it once, where the post comment is, between the encode
  and the coverage multiply:

  ```glsl
  vec3 glassColor = linearToSrgb(transmitted + within + specular + emissive);
  glassColor = noise_post(glassColor, gl_FragCoord.xy);
  glassed = vec4(glassColor, 1.0) * coverage;
  ```

  Film grain therefore covers the ring, aurora, glint and sweeps as well as
  the transmitted backdrop, is not attenuated by the slab, and lands only
  where the slab has coverage: translucent window pixels and the band outside
  the window. Opaque client pixels bypassed the shader before any of this.
- **Identities the tests hold.** (a) Omitted site equals `site="glass"`
  pixel for pixel. (b) On the flat face at `ior 1`, white attenuation, no
  signal responses and the other optics neutral, `film` at amount `a` equals
  `glass` at amount `a` within 1/255 per channel: for white and fine the
  glass path is `decode(encode(x) + g)` then `* 1` then `encode`, the film
  path is `encode(x) + g`; for lightness both move the Oklab lightness of
  the same clamped color and differ only in where the final encode
  happens. (c) Under the same conditions with `blur { off }` and
  roughness 0, `backdrop` at amount `a` equals `glass` at amount `a` within
  2/255: the grain is stored in an 8-bit texture before the tap reads it.
  Each identity is one in-process render test (§8).

## 6. The schema and its pins

Stages, in the order they enter `STAGES` (grouped by site in site order;
within `source`, the call sequence):

| Stage | Site | Scope | Owns | Reads | Optic | Selector | Animated |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `backdrop-grain` | source | output | | `noise`, `noise type=`, `noise site=` | `{noise, source, effect}` | `{noise site=, backdrop}` | no |
| `blur` | source | output | (unchanged) | | | | |
| `prefilter` | source | material | (unchanged) | | | | |
| `noise` | behind | material | `noise`, `noise type=`, `noise site=` | `noise`, `noise type=`, `noise site=`, `blur noise`, `backdrop-blur` | `{noise, behind, material}` | `{noise site=, glass}` | no |
| `film-grain` | post | material | | `noise`, `noise type=`, `noise site=` | `{noise, post, material}` | `{noise site=, film}` | no |

- `stage()` gains a selector argument (or a `selected()` sibling); the
  `Stage` shape, the JSON shape and the version stay as the prism design's
  Section 2 defines them. `every_parameter_is_owned_by_exactly_one_stage`
  holds: the new parameter is owned by `noise` alone.
  `todays_stages_pass_the_selector_rules` now exercises real selectors;
  the negative fixtures stay.
- **Interactions: none added.** Blur and roughness soften backdrop grain
  only while the window's source gate selects the blurred texture or the
  pyramid picks a level above zero, so by the design's own rule (an edge is
  unconditional) these are conditional dependencies and go in prism's
  interaction document as prose, with §7's numbers. Film grain adds to the
  ring, aurora, glint and sweeps; it removes nothing, so it is not a
  `shadows` edge. The prism design's Section 5 sentence that film-site grain
  creates `shadows` edges is wrong by its own Section 1 definition;
  `prism-be5abe` corrects it (the `shadows` kind still enters with film-site
  saturation, which greys light, a follow-up).
- **Hook pins.** `pipeline_material_hooks_are_called_exactly_once_in_stage_order`
  gains `noise_post` after the encode. `pipeline_post_site_is_empty_until_a_stage_lands`
  is deleted; the post stage is pinned by the stage-order test like every
  other. The effect-program pin changes shape: each Effect stage with an
  optic must have its hook called exactly once in exactly one of the
  effect program's files (`blur_down.frag`, `blur_up.frag`, `grain.frag`);
  `blur` and `prefilter` have no optic and are not hooks. The follow-up
  triage note on `material-a00785` anticipated this revisit.
- **GLSL reads.** The uniform-to-node map gains `mat_noise_site` to
  `"noise site="`. The grain program sets `mat_noise` and `mat_noise_type`
  from `GrainOptions`; it never reads `mat_noise_site`, and the pin scans
  the optic's GLSL, which declares all three.
- **Generated file.** `resources/materials/pipeline.json` is regenerated
  under `MATERIAL_DOCS_UPDATE=1`; the version stays 1. At merge, prism's
  `defs/rack/pipeline.json` is refreshed in a one-line commit with the rack
  unchanged, as `prism-14040e` did for the sweeps row; prism's contract test
  would otherwise fail against this checkout. The noise device keeps `stage:
  noise` until `prism-be5abe` teaches the rack about selectors.

## 7. Evidence

Three measurements, each written into one evidence document under
`docs/materials/`, dated the day the captures run, with `tools/capture-meta
show` output where a capture lane ran.

### 7.1 The look, offline, before any renderer code

`docs/materials/scripts/noise-placement-sim.sh` takes one backdrop image
(default: a synthesized backdrop with gradients, edges and a textured
region, so the script needs no host file) and produces one contact sheet
plus a table. Grain: signed white noise at a stated amount in sRGB, and
its fine form (grain minus its 3×3 mean, scaled as `fineGrain` scales).
Blur: a Kawase-like chain of `p` half-size box downsamples then `p`
bilinear upsamples, for `p` in 0, 1, 2, 3. Roughness: a pyramid level `L`
as `2^L` box downsample and bilinear upsample, for `L` in 0, 1, 2. Cells:
grain after the chain (what `glass` does) beside grain before the chain
(what `backdrop` does), for every `p × L`, both grain kinds. The table
reports the standard deviation of the grain that remains in each cell (the
cell minus the same chain over the ungrained image), absolute and relative
to the glass site at `p = 0`, `L = 0`. In this model one blur pass and one
pyramid level are the same operation, so `p = 1, L = 0` equals `p = 0,
L = 1`; the shader's Kawase offset and its level mix make them differ.

The sheet is for the owner to look at once; the table is the first row of
the dominance matrix's perceptual half. The prediction, made before the
run: backdrop grain reads as grain at `p = 0`, as softened grain at `p = 1`,
and as mottling by `p = 3`; roughness levels compound it. If the owner finds
the backdrop look useless at every `p`, this design loses §4 and the
`backdrop` variant before the plan is written.

**Run of 2026-10-05** (default synthesized backdrop, amount 0.3, seed 11,
512 px). Remaining grain relative to the glass site, which stays at 1.00 in
every cell (it is added after the chain):

| Grain | `p = 0` | `p = 1` | `p = 2` | `p = 3` | `p = 3, L = 2` |
| --- | ---: | ---: | ---: | ---: | ---: |
| white | 1.00 | 0.32 | 0.15 | 0.083 | 0.077 |
| fine | 1.00 | 0.19 | 0.072 | 0.046 | 0.044 |

Absolute `sd` at `p = 0`: 0.0856 (white), 0.0839 (fine), the uniform grain's
`0.3 / sqrt(12)` as expected. The sheets match the prediction: at `p = 0`
the two sites are the same grain; at `p = 1` backdrop grain is soft but
still grain; from `p = 2` it is colour mottling at the blur's scale (the
model's view: the real chain measured 4 % at one pass and nothing above the
8-bit floor from three passes, see the evidence document), and
fine grain, being high-pass, is erased faster than white. Nothing in the
run argues for dropping the site: under a sharp window it is the glass
grain moved into the image, and under a frosted one it is a different,
softer texture whose amount the rack can still explore. The owner's look
at the sheets is recorded in the evidence document when the plan runs
§7.1 again against the implementation's own captures.

### 7.2 Captures of the implementation

A headless Weston smoke, `docs/materials/scripts/glass-noise-site-smoke.sh`,
built on `glass-optic-smoke-lib.sh` and the statistics of
`glass-noise-type-smoke.sh` (difference against the amount-0 capture of the
same fixture; `sd`, downsampled `sd`, low-frequency ratio, determinism).
Two fixtures, because the prefilter level is `roughness * clamp(ior * 2 -
2, 0, 1)` and is zero at `ior 1` whatever roughness says:

- **Identity fixture:** a flat warm backdrop with one textured region, the
  transparent kitty over glass at `ior 1`, white attenuation, no responses,
  as the noise-type smoke isolates grain. Cells: site in `glass`,
  `backdrop`, `film`; blur in off, 1 pass, 3 passes; grain `fine` at amount
  0.3, plus `white` and `lightness` at the same amount for the blur-off
  row.
- **Roughness fixture:** the same at `ior 1.5` (the clamp is 1, so
  `roughness` is the normalized level and the pyramid level is `max_level *
  roughness`), `backdrop-blur` on at 3 passes and off, roughness in 0, 0.5,
  1, sites `glass` and `backdrop`, grain `fine` at 0.3.
- **Ring fixture:** for assertion 7 only, below.

Assertions:

1. Determinism: two captures of each cell are identical.
2. Omitted site equals `site="glass"` (absolute error 0), for all three
   grain kinds.
3. Against a baseline binary built from `b261ad1a` with the same config
   (which has no `site`), every omitted-site cell is identical (absolute
   error 0). This is the byte-identity check; assertion 2 alone cannot see
   a change both paths share.
4. `backdrop` at blur off equals `glass` within 2/255 mean absolute error
   over the glass area, in the identity fixture.
5. `backdrop`'s grain `sd` falls monotonically with passes (identity
   fixture) and with roughness (roughness fixture), while `glass`'s stays
   within 5 % across the same cells; the values are reported beside §7.1's
   prediction. "Monotonically" holds down to the 8-bit floor: once a cell's
   grain `sd` is below half a code (the in-process presence threshold),
   later cells must stay below it rather than fall further. The quiet pilot
   of 2026-10-05 measured backdrop roughness 0 at exactly 0 under three
   passes and roughness 1 at 0.037 codes grey (a one-signed, blue-only
   one-code shift; cause not established), not returning grain texture.
6. `film` at blur off equals `glass` within 1/255 on the face.
7. Film grains the light and glass grain is compressed under it, by the
   transfer curve, not by magic. Fixture: a flat backdrop at encoded 0.5
   (linear 0.214), white attenuation, grain amount 0.1 so no cell clips,
   the focus response on with `ring-rest 1` and a high `ring-glow`,
   captured after the comet's whole run has ended: the head's envelope
   reaches 0 at the end of its lap, but the tail keeps draining behind it
   for `ring::run_length` (perimeter plus tail length, capped by
   `ring-beam-decay`), so the smoke waits that distance divided by
   `ring-beam-speed` plus a margin, and only the uniform rest glow remains
   in the band. Setting `ring-beam-speed 0` before the fixture focuses is
   the alternative when a static head is acceptable; it is not here,
   because the head would make the band's level non-uniform. The band's
   grain-off encoded level
   `e_b` is measured from the capture, not assumed, and should land near
   0.9. Expected ratio of grain `sd` in the band to grain `sd` on the face:
   at `film`, 1.0, because film grain is added in encoding after the light;
   at `glass`, `decode'(0.5) * encode'(decode(e_b))`, because glass grain
   enters as an encoded perturbation of the transmitted sample, is decoded,
   has the ring's light added, and is re-encoded at the brighter level
   (with `att = 1` the two derivatives are the whole effect; at `e_b =
   0.9` the product is about 0.47). Tolerance ±0.1 on each ratio. The
   glass site's ratio is not expected near zero: additive light does not
   remove transmitted grain.

### 7.3 Cost

Tracy captures through the signals smoke's tooling, counting and timing the
`Grain::render` GPU span and the `Blur::render` and
`Prefilter::downsample` spans it triggers, in three cases on the headless
lane:

- **Static wallpaper:** after the first frame, N frames with no backdrop
  damage show zero grain, blur or downsample spans. The pass is cached.
- **Repeated damage, standing in for an animated backdrop:** the
  background layer is remapped K times with distinct images (swaybg
  restarts); per damage, the grain pass time is reported next to the blur
  and pyramid times it precedes. The per-frame cost of an animated
  backdrop is this per-damage number times the frame rate, stated as a
  derivation, not a measurement.
- **Parameter dragging:** the amount is changed through config reload at
  10 Hz for 5 s; per change, the full cascade (grain, blur, both pyramids)
  is timed, against the `glass` site where a change costs no cached work.
  Both workloads contain a blurred and a sharp roughness consumer; distinct
  preparation spans prove both pyramids were requested. Wallpaper mapping
  and sharp-damage spans prove each damage stimulus, and incomplete intervals
  are rejected rather than divided by the requested number of updates.

The numbers go in the evidence document and, as prose, into prism's
interaction document under the cost class. "Cheaper" was the hypothesis the
task names; the document says which cases it holds in.

## 8. Verification

- **Config (`niri-config`).** `site` parses for each name; omitted is
  `glass`; an unknown name fails with the `NoiseSite` error; two materials
  differing in amount, and two differing in type, at the backdrop fail with
  the §3 message naming both; two agreeing pass; a `glass`-site material
  beside a `backdrop`-site one passes. `backdrop_grain()` returns the pair
  or `None`. The parameter-table doc test regenerates the `noise site=` row.
- **Optic.** `NoiseOptic::values` emits the site third, the amount and type
  unchanged; the inherit cases keep their existing tests.
- **Effect buffer.** The invalidation table test (§4) covers each cause
  against each cache. `GrainOptions` change detection: equal options are a
  no-op.
- **Schema.** The existing pins run against the new tables:
  ownership, reads, site grouping, registry order per `(program, site)`,
  selector rules on real selectors, hook calls per program with the
  reshaped effect pin, GLSL reads with the new uniform, and the generated
  file. One new negative fixture: a selector stage whose optic differs from
  its siblings.
- **Pixels, in process (`src/tests/noise_site.rs`).** The three
  identities of §5 as frozen-clock renders with `diff`: omitted equals
  glass; film equals glass on the face within 1/255; backdrop equals glass
  within 2/255 at blur off and roughness 0, with the fixture's test client
  mapping a background layer surface so the effect buffer has content. A
  fourth render checks that backdrop grain's `sd` over the glass area falls
  when the config sets three blur passes and `backdrop-blur` is on. A
  fifth covers the damage contract of §4, and it cannot be a pixel check:
  the fixture's `render_at` renders every element into a fresh target with
  full damage, so pixels change there whether or not anyone published
  damage. The test instead reads the counters the real frame path reads.
  Two windows on one output, one with noise at `backdrop` and one at
  `glass`; before and after a config reload that changes only the backdrop
  amount, it records the output's background effect buffer's `commit()`
  and the `glass` tile's material render fingerprint (the one that carries
  `(bg.id(), bg.commit())`), and asserts both advanced. With the increment
  of §4 removed, the buffer's counter is equal before and after and the
  assertion fails; the plan demonstrates that once. Byte identity against
  the previous renderer is the smoke's baseline comparison (§7.2,
  assertion 3), since one build cannot hold both.
- **Smoke and cost.** §7.2 and §7.3 run once on the headless lane and their
  results go in the evidence document; the smoke stays in
  `docs/materials/scripts` for reruns.
- **Gates.** `just test-fast` while working, `just check` before commits,
  `just gate` before the merge. No live desktop is used: the in-process tests
  are headless and the capture lane is nested Weston.

Acceptance: every §8 test green; the evidence document records §7.1's
sheet and table, §7.2's seven assertions and §7.3's three cases; the
`material-config.md` table and `render-pipeline.md` rows match the code; the
generated schema file is fresh and prism's copy is refreshed at merge; a
config without `site` renders byte-identical to the baseline binary (§7.2
assertion 3), with identity (a) and the noise-type smoke's metrics
unchanged on rerun as the in-build checks.

## 9. Documentation and follow-ups

- `docs/materials/render-pipeline.md`: §1 step 2 gains the grain pass
  between the sharp texture and the blur; §3 row 3b notes the site gate and
  row 9 becomes "Post: film grain"; §4's "two noise sites" bullet becomes
  the three material placements plus the window-site element; §5's table
  gains `noise site=`.
- `docs/materials/material-config.md`: the generated row; the noise optic
  paragraph describes the three sites, the agreement rule and its error,
  and the window-site consequence of §3.
- `docs/materials/adding-an-optic.md`: one paragraph on an optic with
  several placements: one stage per placement, a selector each, and a hook
  per program.
- Prism, in `prism-be5abe`: `glass.noiseSite`, the rack selector, the
  panel's "one amount, per output" explanation, the Section 5 correction
  of §6, and the interaction document's conditional-dependency and cost
  prose from §7.
- Follow-ups to file at close: film-site saturation (the first `shadows`
  edge), and how `material-3fcba2`'s layers select a site (one site per
  layer, which the agreement rule then applies per output across layers).

## Out of scope

Noise layers (`material-3fcba2`); film-site saturation; the prism side
(`prism-be5abe`); order controls at any site; a per-site seed or grain
scale parameter (grain stays one pixel in its carrier's pixel space: output
pixels at the backdrop, screen pixels on the glass and the film); grain
with a time term; changes to the window-site `blur { noise }`.
