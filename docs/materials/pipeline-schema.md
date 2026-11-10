# Pipeline schema

Type: data model; document types 1. This describes format version 1 of
`resources/materials/pipeline.json`, the only version written: the glass
pipeline as data, its sites, stages and structural interactions, with every
field, its allowed values, the invariants the tests pin and how the file is
regenerated. The file is generated from `niri-config/src/material/pipeline.rs`
and is what prism vendors to validate its device rack. The pass order it
records is explained in [render-pipeline.md](render-pipeline.md), what each
optic computes in [glass-optics.md](glass-optics.md), the parameters in
[material-config.md](material-config.md), and how to add a stage in
[adding-an-optic.md](adding-an-optic.md).

## The file

One JSON object, pretty-printed with a trailing newline, never edited by
hand:

| Field | Value |
| --- | --- |
| `version` | `1`. |
| `sites` | The sites in pipeline order (section [Sites](#sites)). |
| `stages` | The stages, grouped by site in site order, and in each site in the order the renderer runs them (section [Stages](#stages)). |
| `interactions` | Structural edges between stages (section [Interactions](#interactions)). |

Two vocabularies recur. A **parameter** is a `node` string from
`all_params()` (`"ior"`, `"noise type="`, `"aurora color"`, as the
generated table in material-config.md spells them) or one of the four global
`blur` fields in `BLUR_FIELDS` (`"blur passes"`, `"blur offset"`,
`"blur noise"`, `"blur saturation"`). A **response field** is a KDL name from
`response_fields()` (`"ring-gap"`, `"accent-tint"`, `"focus"`, ...); retired
response fields are not listed.

## Sites

A site is a point in the per-frame computation where stages act.

| Field | Values | Meaning |
| --- | --- | --- |
| `id` | the eleven below | Stable name. |
| `carrier` | `texture`, `normal`, `linear`, `light`, `encoded` | The value that leaves the site: a per-output offscreen texture; the slab's surface normal; linear-light RGB of the transmitted backdrop; additive linear light; encoded sRGB. A site that changes carrier is named by what it produces. |
| `law` | `sequence`, `sum`, `product`, `coupled` | Whether the order of two stages at the site changes the result. `sequence`: composition, order matters. `sum`, `product`: commutative. `coupled`: the stages are one computation (the tap loop, the encode), with no order among them. |
| `orderable` | `true`, `false` | Whether a configuration may choose the order of stages at the site. `false` at every site: the shader calls hooks in `OPTICS` order and fixed steps where they are written. |
| `coverage` | `backdrop`, `glass`, `window` | Where the result lands: the shared texture every glass window samples; the slab's translucent pixels and the band outside the window; the window's area under the client, drawn by the background-effect element. |
| `cost` | `cached`, `fragment` | Recomputed when the backdrop is damaged and reused across frames and windows, or evaluated per glass fragment per frame. A class, not a measurement. |

| `id` | `carrier` | `law` | `coverage` | `cost` |
| --- | --- | --- | --- | --- |
| `source` | texture | sequence | backdrop | cached |
| `normal` | normal | sequence | glass | fragment |
| `taps` | linear | coupled | glass | fragment |
| `behind` | linear | sequence | glass | fragment |
| `attenuation` | linear | product | glass | fragment |
| `within` | light | sum | glass | fragment |
| `specular` | light | sequence | glass | fragment |
| `emissive` | light | sum | glass | fragment |
| `encode` | encoded | coupled | glass | fragment |
| `post` | encoded | sequence | glass | fragment |
| `background-effect` | encoded | sequence | window | fragment |

`specular` is a sequence because its hooks transform the light they receive
(see [the specular hook](glass-optics.md#specular-hook)); `within` and
`emissive` hooks return light that is added.

## Stages

A stage is one named step at a site.

| Field | Values | Meaning |
| --- | --- | --- |
| `id` | unique string | Stable name; prism's rack maps devices to these. |
| `site` | a site `id` | Where the stage acts. |
| `scope` | `output`, `material`, `window` | Where its parameters can vary: one value per output, shared by every window; per material definition; per window rule. |
| `owns` | parameters | Those whose control belongs here. Every parameter has exactly one owner. |
| `reads` | parameters | Every parameter its code consumes, as a uniform, a CPU formula feeding one, or a texture it selects. A superset of `owns`. |
| `responses` | response fields | The response-block fields it reads. |
| `optic` | object, or absent | Present when an `OPTICS` entry implements the stage: `name` (the optic's `NAME`), `hook` (`source`, `normal`, `behind`, `within`, `specular`, `emissive` or `post`), `program` (`material` for the material program, `effect` for the effect-buffer passes, `postprocess` for `postprocess.frag`). |
| `selector` | object, or absent | Present when the stage is active only while an enum parameter selects it: `param` (the parameter) and `variant` (one of its values). |
| `animated` | `true`, `false` | Whether the stage's uniforms change without a configuration change: it reads signal, jelly or tilt state, or its optic has a `next_change`. |

The stages today, without `reads` and `responses`, which the file lists in
full:

| `id` | `site` | `scope` | `owns` | `optic` (hook, program) | `selector` | animated |
| --- | --- | --- | --- | --- | --- | --- |
| `backdrop-grain` | source | output | — | noise (source, effect) | `noise site=` backdrop | no |
| `blur` | source | output | `blur passes`, `blur offset` | — | — | no |
| `prefilter` | source | material | `backdrop-blur`, `roughness` | — | — | no |
| `slab` | normal | material | `bevel`, `bevel-profile`, `offset-x`, `offset-y`, `jelly-flex` | — | — | yes |
| `distortion` | normal | material | `distortion`, `distortion scale=` | — | — | yes |
| `ripple` | normal | material | `jelly-ripple` | — | — | yes |
| `refraction` | taps | material | `ior`, `thickness` | — | — | no |
| `fringing` | taps | material | `chromatic-aberration` | — | — | yes |
| `directional-blur` | taps | material | `anisotropic-blur` | — | — | yes |
| `saturation` | behind | material | `saturation` | saturation (behind, material) | — | no |
| `noise` | behind | material | `noise`, `noise type=`, `noise site=`, `noise scale=` | noise (behind, material) | `noise site=` glass | no |
| `tint` | attenuation | material | `attenuation-color`, `attenuation-distance` | — | — | yes |
| `ring` | within | material | `light-ior` | — | — | yes |
| `aurora` | within | material | `aurora`, `aurora drift-hz`, `aurora color` | aurora (within, material) | — | yes |
| `glint` | specular | material | — | — | — | yes |
| `reflection` | specular | material | `reflection` | reflection (specular, material) | — | no |
| `edge-highlight` | specular | material | `edge-highlight` | edge-highlight (specular, material) | — | yes |
| `iridescence` | specular | material | `iridescence` | iridescence (specular, material) | — | no |
| `sweeps` | emissive | material | — | — | — | yes |
| `encode` | encode | material | — | — | — | no |
| `film-grain` | post | material | — | noise (post, material) | `noise site=` film | no |
| `effect-saturation` | background-effect | window | `blur saturation` | — | — | no |
| `effect-noise` | background-effect | window | `blur noise` | — | — | no |

The three `noise` stages are one optic at three placements: `noise site=`
selects which one a layer runs in, and the `noise` stage owns the node's
parameters for all three.

## Interactions

An interaction is an unconditional structural edge between two stages. A
dependency that holds only under a condition is not an edge.

| Field | Values | Meaning |
| --- | --- | --- |
| `kind` | `requires`, `attenuates`, `shadows` | `requires`: `from` has no visible effect while `on` is inert. `attenuates`: `from`'s setting scales `on`'s visible output without zeroing it. `shadows`: `from` runs downstream of `on` on the same carrier and can remove what `on` produced. |
| `from`, `on` | stage `id`s | The two stages, distinct. |
| `why` | string | The mechanism, ending with the rack's decision: `[expose]` (keep both and show the interaction), `[alternative]` (keep the dominated stage and prefer a cooperating placement or parameter), or `[drop]` (the dominated stage leaves the rack). |

There is one edge today: `refraction` attenuates `prefilter`, because the
prefilter level is `roughness * clamp(ior * 2 - 2, 0, 1)`: at `ior` 1 the
roughness blur flattens while a frosted backdrop still selects the blurred
source. Its decision is `[expose]`.

## Invariants

Material produces this file and never parses it. Instead its tests refuse a
stage table, or a shader, that breaks these rules (config tests in
`pipeline.rs`, renderer tests in `src/render_helpers/shaders/mod.rs` and
`src/render_helpers/material/mod.rs`):

- Every parameter is owned by exactly one stage, and `owns` names only
  parameters (`every_parameter_is_owned_by_exactly_one_stage`).
- `reads` contains `owns` and names only parameters
  (`reads_cover_owns_and_name_only_parameters`); every taps stage reads the
  prefilter's controls (`taps_stages_read_the_prefilter_controls`).
- `responses` names only response fields, and `response_fields()` lists the
  live fields of the response block in struct order
  (`responses_name_response_fields`, `response_fields_match_the_struct`).
- Site and stage ids are unique, and stages are grouped by site in site order
  (`stages_are_grouped_by_site_in_site_order`).
- Every optic in `ORDER` has a stage; an `optic` names an `ORDER` entry; a
  (name, hook) pair appears once; within one program and site, optic stages
  keep `ORDER` (`optic_stages_follow_the_registry_within_each_program_and_site`).
- A `selector` names an enum parameter the stage owns or reads and one of its
  variants, on an optic stage; across the stages selecting on one parameter,
  each variant selects exactly one stage and all belong to one optic
  (`check_selectors`, proven on fixtures by
  `selector_rules_refuse_each_defect`).
- An interaction names two distinct existing stages, and its `why` ends with a
  decision (`interactions_name_two_distinct_stages`).
- Each `material` hook is called exactly once in the material program, in
  stage order, in both the rest and the view variant
  (`pipeline_material_hooks_are_called_exactly_once_in_stage_order`); each
  `effect` or `postprocess` hook is called exactly once, in exactly one file
  of its program (`pipeline_other_programs_call_their_hooks_exactly_once`).
- Every parameter uniform an optic's GLSL reads is in its stage's `reads`
  (`pipeline_optic_glsl_reads_are_declared`).
- A stage that signals move frame to frame is `animated` and lists the
  responses that drive it (`pipeline_signal_driven_stages_are_declared`).
- Rendering is stable, ends with a newline, and omits `optic` and `selector`
  rather than writing `null` (`schema_renders_stably_with_a_trailing_newline`).
  `todays_shape` pins the current site list, the selected stages and the one
  edge, so any change to them is deliberate.
- The committed file equals the rendering
  (`material_pipeline_schema_matches_the_file`); a stale file fails with
  `resources/materials/pipeline.json is stale; rerun with MATERIAL_DOCS_UPDATE=1`.

## Regenerating

After changing a table in `pipeline.rs`, or a parameter or response field it
names, write the file and commit it with the change:

```sh
MATERIAL_DOCS_UPDATE=1 just test-one -p niri-config material_pipeline_schema_matches_the_file
```

`MATERIAL_DOCS_UPDATE=1 just test` regenerates it together with the
parameter table in material-config.md. Prism's vendored copy is refreshed
from this file in prism's own checkout.
