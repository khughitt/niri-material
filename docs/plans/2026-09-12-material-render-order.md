# Material Render Order Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan task-by-task. Steps use checkboxes for tracking.

**Status:** draft for review. Neither implementation step has started.

**Goal:** Move backdrop colour operations behind attenuation and interior
light ahead of surface light, preserving the material composition contract.

**Architecture:** Two sequential changes to the existing material shader:
`behind` transforms averaged linear samples; `within` adds refracted interior
light with its remaining-path attenuation. Keep the optic registry, uniform
plumbing and capture library; introduce no rendering abstraction.

**Tech Stack:** Rust, GLSL ES 1.00, Bash, Python standard library,
ImageMagick 7, nested Weston GLES, existing Tracy 0.13.1 tools, `just`.

**Spec:** [Material render order by depth](../specs/2026-09-12-material-render-order-design.md),
reviewed at `522a09fe`, including transmitted chamfer grain without a mask.

## Global constraints

- Work in `.worktrees/material-5b3107`, branch `material-5b3107`. Commands
  below run after `cd .worktrees/material-5b3107`; file inventories include
  that prefix so they resolve from the main checkout.
- Read the spec and `.worktrees/material-5b3107/docs/materials/render-pipeline.md`
  before shader edits. Inspect actual commits and code before trusting status
  headers or checkboxes.
- Execute Task 1 before Task 2: both touch the shader, registries and docs.
  Use the existing children, not new step tasks. `tasks start` precedes edits;
  `tasks done` belongs in the implementation commit after its evidence passes.
- Keep parameter names, bounds, KDL, Prism, `postprocess.frag`, texture
  ownership, damage and animation cadence unchanged. No bevel noise mask,
  ring cutoff, new oscillator, depth parameter or compatibility layer.
- Grain remains screen-seeded after averaged taps. The chamfer may transmit
  grain. Surface and interior additive light must not be postprocessed by it.
- `d = 0.8` is measured from the slab's back: both interior lookups use
  `0.2 * thickness`, and their radiance uses `pow(att, vec3(0.2))` once.
- Preserve the literal `0.2` in GLSL. Computing `1.0 - 0.8` introduces a
  different floating-point value and defeats the preserved-factor claim.
- Neutral hooks return before colour conversion. No masking signed grain
  with a new output clamp. Oklab lightness retains its existing gamut clamp.
- Run tests through `just`; required gates are `just test` and `just check`.
  Runtime GLSL requires actual GLES capture proof, not only Rust compilation.
- Captures run serially under the existing capture preflight, identity,
  settling, hashing and release protocol. Preserve refusal records; never
  bypass readiness thresholds to obtain a passing capture.
- Commit conventionally, without attribution trailers. Update the relevant
  spec status and user docs with each delivered child; regenerate the staged
  upstream report before committing. No installation or desktop restart is
  included in this plan.

## Files and responsibilities

All entries below begin at `.worktrees/material-5b3107/`.

| File | Responsibility |
| --- | --- |
| `.worktrees/material-5b3107/src/render_helpers/shaders/material/main.frag` | Hook call order; inline ring; separate within/surface sums. |
| `.worktrees/material-5b3107/src/render_helpers/shaders/material/prelude.frag` | Defined signed colour conversions; correct light-path commentary. |
| `.worktrees/material-5b3107/src/render_helpers/shaders/material/saturation.frag` | Linear-boundary saturation hook. |
| `.worktrees/material-5b3107/src/render_helpers/shaders/material/noise.frag` | Linear-boundary noise hook, unchanged grain formulas. |
| `.worktrees/material-5b3107/src/render_helpers/shaders/material/aurora.frag` | Refracted within hook. |
| `.worktrees/material-5b3107/src/render_helpers/material/optics/mod.rs` | Renderer registry order. |
| `.worktrees/material-5b3107/niri-config/src/material/optics/mod.rs` | Matching `ORDER` and `params()` aggregation order. |
| `.worktrees/material-5b3107/src/render_helpers/shaders/mod.rs` | Existing source-assembly regression test. |
| `.worktrees/material-5b3107/src/render_helpers/material/optics/{noise,saturation,aurora}.rs` | Stage comments only; preserve values and cadence tests. |
| `.worktrees/material-5b3107/docs/materials/scripts/glass-render-order-smoke.sh` | New focused smoke using the existing optic capture library. |
| `.worktrees/material-5b3107/docs/materials/scripts/glass-render-order-metrics.py` | New small metric program: signed grain, quantization intervals, ring reach. No capture orchestration. |
| `.worktrees/material-5b3107/tools/test_glass_render_order_metrics.py` | One compact synthetic-image regression test for the metric program. |
| `.worktrees/material-5b3107/tools/test_glass_optic_smoke.py` | Include the new smoke in existing protocol-adoption checks. |
| `.worktrees/material-5b3107/docs/materials/scripts/glass-noise-type-smoke.sh` | Preserve formula checks using an untinted fixture. |
| `.worktrees/material-5b3107/docs/materials/scripts/glass-noise-saturation-smoke.sh` | Desaturation probe uses neutral tint and additive light. |
| `.worktrees/material-5b3107/docs/materials/scripts/focus-ring-light.sh` | Replace rest face-zero assertions with the shared reach metric. |
| `.worktrees/material-5b3107/docs/materials/scripts/glass-aurora-smoke.sh` | Existing neutral, cadence and cost regression run. |
| `.worktrees/material-5b3107/docs/materials/{render-pipeline,adding-an-optic,material-config}.md` | Current behavior and generated parameter-table order. |
| `.worktrees/material-5b3107/docs/specs/2026-09-10-material-optics-design.md` | Landed hook contract/status. |
| `.worktrees/material-5b3107/docs/specs/2026-09-12-material-render-order-design.md` | Partial/completed implementation status and evidence links. |
| `.worktrees/material-5b3107/docs/materials/2026-09-12-material-render-order-evidence.md` | Create in Task 1; append Task 2 results, identities and capture limitations. |

Older affected specs are listed in each documentation step. Do not rewrite
historical evidence as if its original captures used the new order.

## Shared capture contract

Reuse `glass-optic-smoke-lib.sh`: `capture_preflight`, `build_binaries`,
`capture_identity`, `calibrate_probe_rect`, `start_nested`, `spawn_probe`,
`probe_rect`, `shot`, `roi`, `trace_run`, `gpu_median_ns`, `median3`, `finish`.
Do not copy Weston lifecycle code from the older noise or ring scripts.

The new smoke accepts `PHASE=behind|within`, requires `BASE_NIRI` and
`BASE_NIRI_TRACY` immutable baseline binaries, and uses the library's `OUT`,
`NIRI_MATERIAL_WORK_ROOT` and `CAPTURE_TASK`. Baseline source is `522a09fe`
for Task 1 and the completed Task 1 commit for Task 2. Snapshot baseline
binaries before rebuilding; record their source commit, feature sets and
hashes. Use an isolated baseline checkout if no matching retained binaries
exist; run setup if its justfile defines it. Do not change the implementation
worktree's HEAD to build a baseline.

The entry sequence is explicit:

```bash
set -eu
: "${PHASE:?behind or within}" "${BASE_NIRI:?baseline release binary}"
: "${BASE_NIRI_TRACY:?baseline Tracy binary}" "${CAPTURE_TASK:?task id}"
case "$PHASE" in behind|within) ;; *) exit 2 ;; esac
HERE=$(dirname "$(readlink -f "$0")")
. "$HERE/glass-optic-smoke-lib.sh"
capture_preflight headless
build_binaries
capture_identity --binary "$BASE_NIRI" --binary "$BASE_NIRI_TRACY" \
    --input "$HERE/glass-render-order-metrics.py" \
    --config phase="$PHASE" --config output=1280x720 --config scale=1
calibrate_probe_rect "$NIRI" 0
```

Use the library's 1280×720, scale-1, warm `rgb(140,115,90)` wallpaper fixture.
Run one transparent idle probe, no focus thief, fixed creation order (the
material seed is an allocation counter), and a settled, pinned phase.
Keep blur/roughness/smear/distortion/ripple off unless a case names them.
Use `offset-x 0; offset-y 0` so the slab is the calibrated window inflated
by the bevel on every side; the outer radius is the bevel when window
corner radii are zero. Evaluate distance at pixel centres with that rounded
box, including corners. Check repeated decoded captures for equality before
computing metrics. Fail if geometry changes or a crop is empty.

For response variants, write the library config then replace its existing
`focus "none"` or `accent "none"` line in that case's file. Do not append
duplicate KDL nodes through `RESPONSE_EXTRA`. Keep `ring-drift-hz 0` for all
rest comparisons. Validate every generated config before launching.

The metric program decodes via checked `magick IMAGE -alpha off -depth 8
rgb:-`, checks image dimensions and exact byte count, then uses Python
`math`, `statistics`, `json`, and `subprocess`. Reject failed decoding,
non-finite results and empty selections. Provide three explicit subcommands:

```text
grain ON OFF --rect X Y W H
additive NOISE_LIGHT NOISE_DARK ZERO_LIGHT ZERO_DARK --rect X Y W H
reach ON OFF --window X Y W H --bevel B --thickness T --inset I --width W
```

`grain` reports the population standard deviation of signed sRGB luma
residuals using weights `(0.2126, 0.7152, 0.0722)`, not absolute residuals.
`additive` reports maximum residual, interval failures and sample count;
the four images must be aligned and fully covered, with transparent client
background. `reach` reports the bound, reference contour, maximum channel
delta and changed-pixel count outside both, plus maximum visible interior
reach. A visible pixel differs by **more than one** 8-bit channel code;
one-code changes are accepted quantization-scale differences, not byte
identity. Do not label the resulting count "all changed pixels".

For cost, use the library's existing 30-second Tracy run, 20–28-second GPU
sample window and three rotated rounds of baseline/neutral/active with the
same repainting probe. Record medians and percentage differences; do not
invent a performance pass threshold. Preserve the existing aurora cadence
checks separately. Builds are not measurements of shader frame cost.

Run the new smoke through the existing timed `just` front door. Export
`BASE_NIRI`, `BASE_NIRI_TRACY` and `NIRI_MATERIAL_WORK_ROOT` to the verified
artifact locations, then use a fresh output directory for each attempt:

```bash
PHASE=behind CAPTURE_TASK=material-f8b6e9 \
OUT=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/render-order-behind.XXXXXX") \
just --set fast_cmd 'bash docs/materials/scripts/glass-render-order-smoke.sh' test-fast

PHASE=within CAPTURE_TASK=material-92edaf \
OUT=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/render-order-within.XXXXXX") \
just --set fast_cmd 'bash docs/materials/scripts/glass-render-order-smoke.sh' test-fast
```

The variable override uses the existing recipe, so no additional justfile
recipe is needed. Run the existing regression smokes by replacing its shell
command and supplying each script's required environment. Never use the
same `OUT` for a second run.

### Quantization-aware additive comparison

Let `D` decode one sRGB channel. A captured integer channel `c` represents
the interval `[D(max(0,c-0.5)/255), D(min(255,c+0.5)/255)]`. For ordered
images `(noise/light, noise/dark, zero/light, zero/dark)`, compute:

```python
def decode(x):
    return x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4

def additive_interval(codes):
    bounds = [(decode(max(0, c - 0.5) / 255),
               decode(min(255, c + 0.5) / 255)) for c in codes]
    lo = bounds[0][0] - bounds[1][1] - bounds[2][1] + bounds[3][0]
    hi = bounds[0][1] - bounds[1][0] - bounds[2][0] + bounds[3][1]
    return lo, hi
```

Require the interval to contain zero (allow `1e-7` linear arithmetic error).
Use only channels with all four codes strictly between 0 and 255; require
at least 1,000 valid channels and at least 100 with a light-toggle difference
above two codes. Otherwise fail as an uninformative fixture. Report excluded
clipped channels. Use `noise 0.5` for this test so the old post placement has
a measurable failure; use `0.02` only for the default-grain comparison.

### Derived rest ring bound

Pin `ior 1.5`, `light-ior 6`, `thickness 20`, `bevel 12`, `ring-inset 5`,
`ring-width 2.6`, white ring colour, focus ring enabled, pinned drift,
zero jelly activity, aurora/noise disabled and saturation neutral. Run
aberration `0` and `0.6`. Also run dense `thickness 80` with attenuation
colour `#222436`, distance `30`; and a wide-core case `ring-width 20`,
`bevel 32`, thickness `20`. These satisfy inset + width <= bevel.

For a conservative bound allow both focus and maximal accent: colour
channels <= 1, rest gain `G <= 0.7 + 1.0 = 1.7`, attenuation <= 1.
The sRGB encoding slope is at most `12.92`; set linear tail budget
`epsilon = 0.5 / (255 * 12.92)`. Allocating half to core and half to halo:

```python
from math import ceil, log, sqrt

def ring_bound(thickness, inset, width):
    epsilon = 0.5 / (255 * 12.92)
    gain = 1.7
    core = inset + width * sqrt(0.5 * log(2 * gain / epsilon))
    halo = inset + 2 + 9 * sqrt(0.5 * log(0.6 * gain / epsilon))
    shift = 0.5 * inset + 2 * (0.2 * thickness)
    return ceil(max(core, halo) + shift)
```

Each refracted XY vector has length <= 1, so the difference of two
`lightShift` vectors is <= twice the remaining path. This bounds the
uncapped chromatic offsets without pretending the shared cap bounds them.
The rounded-box distance changes by no more than the displacement. Beyond
this radius, the continuous encoded contribution is below half a code;
rounding two captures permits at most one code of difference.

Calculated before capture: **37 px** for the pinned fixture, **61 px** for
dense glass, **61 px** for the wide core. The requested reference contour is
**27.5 px** in all three. For the older ring smoke's pinned thickness 41.7
and default width, the conservative bound is **46 px**. Use its actual
validated width and colours, and reject a gain/configuration outside the
derivation. Do not gate these bounds during motion, where jelly gain and
capture alignment differ.

---

### Task 1: Behind hook: noise and saturation

**Task:** `material-f8b6e9`.

**Files:** Main/prelude/noise/saturation shaders, both optic registries,
shader assembly test, noise/saturation stage comments, new smoke and metrics
files, metric regression test, existing noise smokes and documentation from
the inventory above. Do not change aurora or ring behavior in this step.

**Interfaces:** Consumes the existing averaged `sampled` and sRGB transfer
helpers. Produces `vec3 saturation_behind(vec3 color, vec2 fragCoord)` and
`vec3 noise_behind(vec3 color, vec2 fragCoord)`, both linear in/out. Interim
registry order is `saturation, noise, iridescence, aurora`. Captures expose
the three metric commands specified above; implement `reach` in Task 2.

- [ ] **1. Claim and pin the baseline.** Run `tasks start material-f8b6e9`.
  Verify `522a09fe` is an ancestor and source files still match the reviewed
  order. Snapshot the two baseline binaries with provenance. Re-read the
  transfer functions and every caller before changing their shared math:

  ```bash
  rg -n 'srgbToLinear|linearToSrgb|noise_post|saturation_post' src/render_helpers
  ```

- [ ] **2. Add the measurement and assembly regressions first.** Extend the
  existing assembly test to expect the interim registry and the two behind
  calls between averaged sampling and `vec3 transmitted`. Retain the
  existing uniform/registry checks. Add the `grain` and `additive` metric
  commands and new smoke entry sequence above. In the metric test import the
  script with `importlib.util.spec_from_file_location` and test the actual
  functions, including a deliberately perturbed light response:

  ```python
  lo, hi = metrics.additive_interval([160, 128, 160, 128])
  self.assertLessEqual(lo, 0)
  self.assertGreaterEqual(hi, 0)
  lo, hi = metrics.additive_interval([190, 128, 160, 128])
  self.assertGreater(lo, 0)
  self.assertEqual(metrics.grain_sd([(100, 100, 100)] * 2,
                                   [(100, 100, 100)] * 2), 0)
  ```

  Search call positions only inside `main`, excluding hook definitions:

  ```rust
  let main = source.split_once("void main()").unwrap().1;
  let saturation = main.find("sampled = saturation_behind(").unwrap();
  let noise = main.find("sampled = noise_behind(").unwrap();
  let attenuation = main.find("vec3 transmitted = sampled * att;").unwrap();
  assert!(saturation < noise && noise < attenuation);
  assert!(!main.contains("saturation_post("));
  assert!(!main.contains("noise_post("));
  ```

  Define `grain_sd(on, off)` over equally sized nonempty sequences of RGB
  byte triples, returning `statistics.pstdev` of signed weighted differences
  divided by 255. Check empty/mismatched data and failed decode raise errors.
  Run `just test` and `just check`: the source-assembly expectations must
  fail against the old shader. Run the new additive capture against the old
  binary and retain its nonzero exit and measured failure; a geometry or
  protocol refusal is not the expected regression failure.

- [ ] **3. Make the transfer helpers defined for signed channels.** Change
  only the unused power branch bases in the material prelude:

  ```glsl
  // srgbToLinear: preserve its existing low branch and selection.
  vec3 high = pow(max((c + 0.055) / 1.055, vec3(0.0)), vec3(2.4));
  // linearToSrgb: preserve its existing low branch and selection.
  vec3 high = 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055;
  ```

  Do not modify the independent postprocess shader. Negative inputs select
  their existing signed linear branches; above-one inputs remain above one.

- [ ] **4. Move saturation and noise.** Replace the saturation function:

  ```glsl
  vec3 saturation_behind(vec3 color, vec2 fragCoord) {
      if (mat_saturation == 1.0)
          return color;
      vec3 encoded = linearToSrgb(color);
      const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
      return srgbToLinear(mix(vec3(dot(encoded, luma)), encoded, mat_saturation));
  }
  ```

  Rename noise, return the linear input at zero, then encode its active
  input. Keep its seed and grain computation verbatim. White and fine return
  `srgbToLinear(encoded + grain)` using their current scalar/vector formulas.
  For lightness, return `clamp(oklabToLinear(lab), 0.0, 1.0)` directly after
  the existing lab L adjustment; this is the linear result of its old
  encode/decode round trip, not a new grain algorithm.

  Insert before Beer-Lambert attenuation, outside the tap loop:

  ```glsl
  sampled = saturation_behind(sampled, gl_FragCoord.xy);
  sampled = noise_behind(sampled, gl_FragCoord.xy);
  ```

  Remove the two post calls; leave a comment reserving `post` after encoding.
  Reorder `OPTICS`, `ORDER` **and `params()`** to the interim order. Update
  the existing assembly markers and stage comments accordingly.

- [ ] **5. Run the behind acceptance matrix.** Add these cases to the new
  smoke's `behind` phase, using the shared geometry and metric contract:

  | Case | Required observation |
  | --- | --- |
  | Explicit `noise 0; saturation 1`, ring/aurora disabled | Baseline vs candidate decoded pixel identity; repeat captures identical. |
  | All three noise types at `0.02`, `ior 1`, white attenuation, no additive lights | Face signed residual SD within 10% of baseline; report both SDs. `ior 1` removes face Fresnel. |
  | Three types at `0.5`, warm midtone, white attenuation | Preserve fine/white grain distribution checks and lightness chroma check from the existing noise-type smoke. |
  | `saturation 0`, white tint, `ior 1`, no additive lights | Grayscale within one code; coloured-tint variant may remain coloured. |
  | `noise 0/0.5` crossed with pinned `aurora 0/0.5`, then ring focus none/ring-light, then iridescence `0/0.8` | Additive interval contains zero on valid fully covered channels; count informative samples. Iridescence isolates a change to the glint without changing IOR/transmission. |
  | Dense tint `#222436`, thickness 80, distance 30 | Report face and bevel grain at `0.02`; no zero-bevel or baseline-amplitude gate. |
  | Black and near-black backdrop, `noise 1` for each type; saturated backdrop at saturation 3 | Real GLES compiles; finite expected results after output clamping, no fallback; catch broken signed transfer on dark grain. |
  | Opaque glyphs and an opaque-client probe | Decoded identity under noise/saturation toggles, at opacity 1 and 0.7 separately. |

  Test static light cases separately to avoid saturation. Pin phase and
  creation order; reject saturated or uninformative additive probes. The
  glint test covers iridescence's additive delta; check the unmodified
  Fresnel base analytically on the uniform straight chamfer and through the
  source-order assertion, since there is no independent Fresnel-off knob.

  Keep sweeps in the final unattenuated sum and exercise the existing signal
  smoke's done/error sweeps. Dynamic captures are evidence with timestamps,
  not a four-frame equality gate unless their signal states match. Do not
  add a production debug uniform just to freeze them.

  Change the older noise-type and saturation smoke fixtures to white tint,
  no additive lights and `ior 1` where their old grayscale/chroma assertions
  require it; retain their explicit/omitted/blur-off checks. For cost, compare
  baseline/neutral/active noise+saturation using the shared three-round
  protocol. Record the new smoke command and all exit statuses in evidence.

- [ ] **6. Verify and document the delivered stage.** Run:

  ```bash
  MATERIAL_DOCS_UPDATE=1 just test
  just check
  ```

  Update current pipeline, optic recipe and config prose for behind in this
  commit. Add only the implemented hook to their current-stage table;
  within remains planned. Mark behind implemented/within pending in the
  render-order spec and plan. Add partial-supersession status links to the
  2026-09-02 noise/saturation, 2026-09-05 glass-noise/saturation-params and
  2026-09-06 glass-noise-type specs. Update the optics spec's hook count and
  noise/saturation positions. Search user docs for propagated old claims:

  ```bash
  rg -n 'after the glass optics|finished glass|encoded glass|four hook|Stage (9|10)' docs
  ```

  Correct current prose; label old evidence historical. Publish measured
  cost, signed grain, additive comparison, opaque identity and protocol
  limits in the new evidence file. No unmeasured result is marked passed.

- [ ] **7. Close the child in its commit.** Only after acceptance passes:

  ```bash
  tasks done material-f8b6e9 'Behind hook implemented; colour-space, additive-light and capture checks recorded'
  tasks check
  git add src niri-config tools docs tasks/material-f8b6e9.md
  just upstream-report
  git add docs/materials/upstream-divergence.md
  git diff --cached --check
  git commit -m 'feat(material): apply backdrop colour optics behind attenuation'
  ```

  Inspect the staged diff before committing; include only this child's work.
  The hook repeats the required check gate. If capture readiness blocks the
  evidence, record and park the child; do not close it as implemented-and-proven.

### Task 2: Within hook: ring and aurora

**Task:** `material-92edaf`, depends on `material-f8b6e9`.

**Files:** Main and aurora shaders, prelude light-path comments, aurora stage
comment, both registries and shader assembly test; extend the new smoke,
metric program/test and evidence; update the older ring smoke's rest gates
and current docs. Run the existing aurora smoke without changing its cadence.

**Interfaces:** Consumes Task 1's linear `sampled`, behind hooks, `att` and
metric/capture program. Produces `vec3 aurora_within(vec2 p, vec3 n, vec3 att,
float innerDist)` returning attenuated linear light, with inline ring added
to `within`. Final registry: `saturation, noise, aurora, iridescence`.

- [ ] **1. Claim, snapshot and add the failing checks.** Run `tasks start
  material-92edaf`; snapshot Task 1's release/Tracy binaries. Update the
  existing shader-source assertions to expect within accumulation before
  specular, `aurora_within`, a `0.2` ring lookup path and no ring mask.
  Add `reach` to the metric program using the bound above and rounded-box
  distance at pixel centres. Extend the same metric regression test:

  ```python
  self.assertEqual(metrics.ring_bound(20, 5, 2.6), 37)
  self.assertEqual(metrics.ring_bound(80, 5, 2.6), 61)
  self.assertEqual(metrics.ring_bound(20, 5, 20), 61)
  ```

  Exercise the actual reach selection with synthetic aligned images: a
  two-code pixel outside the bound must fail; one-code noise passes; a
  two-code pixel inside contributes to measured reach but does not fail.
  Include a rounded corner and a changed outside-slab pixel to test geometry
  independently. Run `just test` and `just check` and retain the intended
  old-source assertion failure. An old masked ring can pass an upper bound;
  that alone is not a regression test for removing the mask.

- [ ] **2. Move the ring to within.** Immediately after `transmitted`, create
  `vec3 within = vec3(0.0)` and move the existing ring block there. Preserve
  its selectors and `slabChamfer > 0.0` guard. Remove the `mask` declaration,
  mask branch and multiplication. Change only the lookup depth and sum:

  ```glsl
  float depth = mat_thickness * 0.2;
  // Existing capped base, per-channel bands and signal glow remain here.
  within += color * glow * band * pow(att, vec3(0.2));
  ```

  Reuse the current ring code for every expression between those lines;
  do not reconstruct or retune it. Preserve cap application before channel
  offsets, `light-ior`, Gaussian halo/core, drift, colour and jelly gain.

- [ ] **3. Refract aurora and finish the sums.** Rename its function, preserve
  the neutral branch, and change the field's coordinate construction:

  ```glsl
  vec3 aurora_within(vec2 p, vec3 n, vec3 att, float innerDist) {
      if (mat_aurora <= 0.0)
          return vec3(0.0);
      float ior = 1.0 + (mat_ior - 1.0) * mat_light_ior;
      vec2 landing = p + lightShift(n, ior, mat_thickness * 0.2);
      vec3 q = vec3(landing * AURORA_SCALE, 0.0) + mat_jelly_seed
             + vec3(cos(mat_aurora_phase), 0.0, sin(mat_aurora_phase)) * AURORA_LOOP_RADIUS;
      float field = 0.5 + 0.5 * snoiseJelly(q);
      float coarse = 0.5 + 0.5 * snoise(q * 0.5);
      vec3 color = mix(mat_aurora_color_a, mat_aurora_color_b, field);
      return mat_aurora * 0.35 * color * (0.5 + 0.5 * coarse) * pow(att, vec3(0.2));
  }
  ```

  Follow the inline ring with `within += aurora_within(p, n, att, innerDist)`.
  Leave Fresnel, iridescence and accent ordered as before, then initialize
  `emissive` for the unchanged sweep loop. Encode:

  ```glsl
  vec3 glassColor = linearToSrgb(transmitted + within + specular + emissive);
  ```

  Reorder both registries and `params()` to the final order; update the
  assembly test and stage comments. Reserve `post` without active calls.

- [ ] **4. Run the within acceptance matrix.** Implement these cases in the
  new smoke's `within` phase, using the derived bounds already written above:

  | Case | Required observation |
  | --- | --- |
  | Ring/aurora disabled, noise/saturation neutral | Baseline decoded identity and deterministic repeat. |
  | Pinned ring on/off, aberration 0 and 0.6 | No delta >1 code deeper than 37 px; report deltas beyond 27.5 px. |
  | Dense and wide-core ring fixtures | Their 61 px gates pass; ring changes visible pixels, including a face strip just inside the structural chamfer. This positive gate fails against the old bevel mask. |
  | Aurora pinned, `light-ior 1` vs `6`, distortion 0, otherwise identical | Face unchanged (flat normal); chamfer changes. This fails when aurora still ignores its supplied normal. |
  | Aurora with distortion 0.4 and jelly motion | Capture refracted motion and interior appearance; preserve field phase/cadence. Record alignment limits instead of inventing a motion equality gate. |
  | Ring focus/accent selector combinations | Keep the existing selector and colour-share checks; replace only face-zero assertions. |
  | All active optics, opaque glyphs/client | Byte identity on opaque source pixels under matched rest captures. |
  | Noise/light four-way pairs from Task 1 | Still pass after the ring mask and aurora landing point change. |

  Run ring motion via the existing move/resize commands in the older ring
  smoke, with pinned ring drift first, then its existing nonzero drift.
  Record reach, maximum interior delta, timing and geometry for each frame.
  Judge whether the existing drift/refraction satisfies subtle movement;
  record that judgment and any follow-up rather than adding an oscillator.

  For dense attenuation proof, keep thickness, geometry, normal and signal
  state fixed and change only attenuation colour. Subtract ring-off from
  ring-on in linear light at the same pixels. The expected per-channel
  radiance ratio is `(att_dense / att_white)^0.2`, where `att` uses the
  structural cosine and configured optical distance. Compare using decoded
  intervals on unclipped informative channels. Do the same for aurora.
  Do not compare total brightness across the old/new landing points.

  Replace every rest-state `FACE_CROP` zero assertion in
  `focus-ring-light.sh` (rest, selector, resize-at-rest) with `reach` using
  its validated window and slab geometry. Its offset 1 needs the actual
  shifted/inflated slab rectangle; extend `reach` with `--offset-x` and
  `--offset-y` defaulting to 0 and account for them using `material_frame`'s
  inflation rule. Preserve other selector/layout assertions and historical
  evidence. Keep moving-face deltas recorded, not gated.

  Run `glass-aurora-smoke.sh` for neutral, pinned phase, full/reduced/off
  cadence and cost. Use the new smoke's rotated baseline/neutral/active
  timings to record the cost of moving ring and aurora. No new cadence or
  damage-tracking implementation belongs in this task.

- [ ] **5. Verify, update statuses and close the child.** Run
  `MATERIAL_DOCS_UPDATE=1 just test` and `just check`. Update pipeline,
  optic recipe (six hooks), config prose, optics spec and this spec/plan
  with actual implementation status and evidence. Mark the old
  `.worktrees/material-5b3107/docs/specs/2026-09-05-ring-light-focus-response-design.md`
  confinement/depth contract partially superseded. Search the user docs:

  ```bash
  rg -n 'aurora_emissive|four hook|five hook|zero on the face|confined to the bevel|0\.6.*thickness|60 %' docs
  ```

  Update current statements, link historical captures to the replacement,
  and retain all measurements that are merely recorded. Then:

  ```bash
  tasks done material-92edaf 'Within ring and aurora implemented; rest bounds, attenuation and motion evidence recorded'
  tasks check
  git add src niri-config tools docs tasks/material-92edaf.md
  just upstream-report
  git add docs/materials/upstream-divergence.md
  git diff --cached --check
  git commit -m 'feat(material): refract interior light at its remaining slab depth'
  ```

  Inspect the staged diff first. After both children are done, use
  `tasks prime` closeout to verify the goal's delivered behavior and evidence,
  then `tasks done material-5b3107` with the verdict in the landing/closeout
  commit. Correct final design status in the same integration change; do
  not call work "merged" while it only exists on this branch.

## Plan review and handoff

Review this file before implementation, as the repository instructions
require. Recommended execution is inline and sequential with
`superpowers:executing-plans`; a parallel split would contend on shared
shader and documentation files. No execution-mode question is needed.

Self-review covers every spec section: colour-space preservation and
transmitted grain (Task 1), depth/attenuation and motion (Task 2), neutral
and opaque identity, matching registries, unchanged parameter policy,
capture validity, frame cost and per-child documentation (both). The
37/61/61 px bounds are calculated values, not claimed capture results.
