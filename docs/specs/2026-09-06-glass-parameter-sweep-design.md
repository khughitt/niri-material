# Glass parameter sweep script: design

**Status:** designed, not yet implemented. Branch `harness/glass-parameter-sweep`.
Revised after review: two ROIs, blur gating, per-step normalization, flex and
ripple deferred to `material-36e968`.

**Task:** `material-37cec9`

## Context

Prism's sliders in `defs/glass.yaml` are set by eye. `prism-5758d3` reports the
consequence: ranges too wide and linear scales on multiplicative effects, so a
slider spends most of its travel in a saturated dead zone with a narrow,
near-sigmoidal transition somewhere in the middle. Refraction (`prism-8e8a18`)
is the worst case — range `[1, 3]`, useful somewhere near real glass at 1.4 to
1.7, and milky above that.

Choosing bounds needs data: where along a parameter's range does the rendered
image stop changing? Nothing measures that today.
[`glass-noise-saturation-smoke.sh`](../materials/scripts/glass-noise-saturation-smoke.sh)
renders a glass material on a headless Weston host and compares captures, but
it asserts fixed values as a pass/fail gate. The same harness run across N
values of one parameter, reporting deltas instead of assertions, answers the
question.

`prism-5758d3` depends on this task; refraction is the first parameter to run.

## Decision

A new script,
[`docs/materials/scripts/glass-parameter-sweep.sh`](../materials/scripts/glass-parameter-sweep.sh),
renders one KDL parameter at N caller-supplied values and reports the
perceptual delta between neighbouring captures.

It is a measurement tool, not a gate. A flat sweep is a finding, not a failure.

### Interface

Env vars, following the model script's style:

| Var | Required | Meaning |
| --- | --- | --- |
| `NIRI` | yes | niri binary to run |
| `OUT` | yes | artifact directory |
| `BLOCK` | yes | `glass` or `blur` — which KDL block receives the key |
| `KEY` | yes | KDL key, e.g. `ior`, `thickness`, `noise` |
| `VALUES` | yes | two or more whitespace-separated values, ascending |
| `ROI_BEVEL` | no | override the derived bevel-ring crop |
| `ROI_FACE` | no | override the derived face-interior crop |

The caller supplies the values rather than a range and a count. Ranges worth
sweeping are rarely uniform — refraction wants density near 1.4 to 1.7 and a
few coarse probes above it — and a value list keeps that choice with the person
reading the table. Fewer than two values is an error: every metric here is a
difference.

### Supported keys

| Block | Keys |
| --- | --- |
| `glass` | `ior`, `thickness`, `attenuation-distance`, `chromatic-aberration`, `distortion`, `anisotropic-blur`, `roughness`, `bevel`, `light-ior`, `noise`, `saturation` |
| `blur` | `noise`, `saturation`, `passes`, `offset` |

`jelly-flex` and `jelly-ripple` are **not** supported. `material.rs:216` derives
jelly activity from motion residuals and the shader gates ripple behind
`mat_jelly_activity > 0.0` (`material.frag:398`), so on a settled window with
`animations { off; }` both render identically at every value across their whole
range. A sweep that reported them would report zeros and read as "no effect".
Measuring them needs a motion stimulus and a capture phase pinned to the
impulse: `material-36e968`.

Sweeping `bevel` changes the geometry the bevel ROI is derived from. The script
derives both ROIs per value rather than once per sweep, so the crop tracks the
chamfer it is measuring.

### Blur-block gating

`resolve_material` (`src/layout/tile.rs:204-212`) inherits the global blur
`noise` and `saturation` only while backdrop blur is effective:

```rust
let inherited = |global: f64, neutral: f64| if backdrop_blur { global } else { neutral };
```

The model script pins `backdrop-blur false`, so a `blur { noise }` sweep against
that baseline would resolve to the neutral `0.` at every value and report a flat
curve that is an artifact of the gate, not a property of the parameter.

Therefore, when `BLOCK=blur` the emitted config sets glass `backdrop-blur true`
and omits `blur { off }`. The script asserts the gate rather than trusting it:
if the resolved config would leave backdrop blur ineffective, it fails with a
message naming the gate instead of producing zeros.

Glass now carries its own `noise` and `saturation` keys (`material-1293e8`).
Those are the route for measuring the **material** controls, and they render as
written regardless of the gate. The `blur` block is for measuring the
**compositor-global** values Prism also exposes. Both are sweepable; they are
different parameters and the table records which was swept.

### Scene

The backdrop is a generated high-frequency pattern (a grid over color patches),
not the model script's three flat color bars: a flat field shows a bent ray
landing on the same color it started from.

Glass values are pinned in the script and recorded in the evidence document.
They are not derived from a generated `prism.kdl`: that file drifts (`ior` moved
1.02 to 1.24 within a week), which would make two runs of the same sweep
incomparable.

### Two ROIs

`slabSurface` (`material.frag:319-325`) sets `normal = vec3(0,0,1)` everywhere
inside the inner face, and `tap` (`:334`) refracts against it with
`refract(vec3(0,0,-1), vec3(0,0,1), 1.0/ior)`, whose `.xy` is identically zero.
**Lateral refraction in the flat face is zero at every IOR.** The normal tilts
only in the chamfer ring, where `di >= 0`.

Fresnel does vary across the face: `f0 = (ior-1)/(ior+1)` at `:454` changes
brightness with IOR everywhere. A single face crop would therefore produce a
non-zero curve while measuring no bending at all — and a `cumulative > 0` check
would pass, hiding the fact.

So the script derives two crops per value from the window geometry reported by
`niri msg -j windows`, and reports both:

| ROI | Where | Measures |
| --- | --- | --- |
| `bevel` | band straddling the window edge, chamfer-width wide | ray bending |
| `face` | centered box well inside the inner face | Fresnel brightness, tint, noise |

Deriving from reported geometry rather than a hardcoded crop keeps the ROIs
correct when the layout, the window size or the swept `bevel` moves them. Both
are overridable for a parameter that wants a different frame.

Separating them answers `prism-8e8a18` directly: whether refraction's milkiness
is a range problem or a rendering one is a question about which of these two
columns moves.

### Metrics

Per value and per ROI, in Lab colorspace:

| Column | Meaning |
| --- | --- |
| `neighbor` | RMSE against the previous value |
| `per_step` | `neighbor` divided by the parameter step |
| `cumulative` | RMSE against the first value |
| `normalized` | `per_step` over the largest `per_step` in the sweep |

`neighbor` is the raw measurement and is retained. It is **not** on its own
evidence of sensitivity: a caller sampling densely near 1.5 and coarsely above 2
gets smaller neighbor deltas in the dense region purely from the smaller step.
`per_step` divides that out, and the step is recorded in its own column so the
division is auditable. `normalized` is computed from `per_step`, not from
`neighbor`, for the same reason.

`cumulative` separates a parameter that saturated from one that never moved —
both show a small `neighbor`, and only `cumulative` tells them apart.

Lab rather than sRGB RMSE because the question is perceptual: how much visible
change does a slider step buy. If `magick compare` does not honour
`-colorspace Lab`, each capture is converted to Lab first and those are
compared — equivalent, and explicit about when the conversion happens.

**Edge cases.** The first row has no predecessor: its `step`, `neighbor`,
`per_step` and `normalized` are written as `-`, and its `cumulative` is `0` by
definition. When every `per_step` in a sweep is zero, `normalized` is `0` for
every row rather than a division by zero — a flat sweep is a supported outcome
and must produce a readable table. A zero step between two equal values is
rejected at argument-parse time along with unsorted input.

### Output

A TSV to stdout and `$OUT/sweep.tsv`, one row per value:

```
key  value  step  bevel_neighbor  bevel_per_step  bevel_cumulative  bevel_normalized  face_neighbor  face_per_step  face_cumulative  face_normalized
```

Plus the per-value PNGs, both ROI crops, and the configs. `$OUT/SHA256SUMS`, the
niri version and the pinned baseline are recorded as the model script does.

### Failure

Exit non-zero only for harness failure: a capture that was not written, a
non-numeric metric, a config that does not validate, an ineffective blur gate
under `BLOCK=blur`, fewer than two values, a non-ascending or duplicated value
list, or a material error, shader fallback or panic in `niri.log`. The numeric
guards come from the model script, so a broken capture cannot be reported as a
small delta.

A flat curve is not a failure.

## Structure

The script is self-contained. The harness core — `write_config`,
`start_nested`, `stop_nested`, `capture`, `compare_metric`, `is_number`, `fail`,
`cleanup` — is copied from `glass-noise-saturation-smoke.sh`, which is how the
five existing scripts in that directory already relate to each other: each
carries its own copy and there is no library to import.

Extracting a shared `lib-headless.sh` and moving both scripts onto it is the
alternative. It is not taken here because the model script is recorded evidence
for `material-1293e8`, and rewriting its harness underneath a merged result
costs more than the duplication saves at two call sites. If a third
sweep-shaped script appears, extract then.

## Verification

A real run on the headless Weston host sweeping `ior` over seven values, with
the resulting table recorded as evidence under `docs/materials/`. That run is
also the artifact `prism-8e8a18` consumes, so the acceptance evidence and the
first consumer are the same thing.

Checks that the run must satisfy:

- every value produces a capture, and the table has one row per value
- `bevel_cumulative` is non-zero — IOR demonstrably bends the ray, measured
  where bending can occur rather than where it cannot
- the two ROI columns are reported independently, so a face-only Fresnel change
  cannot be read as bending
- `niri.log` is free of material errors, shader fallbacks and panics
- a second run of the same sweep reproduces the table

A second run sweeping glass `noise` confirms the face ROI and the flat-sweep
path on a parameter whose effect is confined to the face.

## Non-goals

- Choosing the bounds. The script reports; `prism-5758d3` decides.
- Sweeping two parameters jointly. One key per run.
- Asserting thresholds. No pass/fail on the numbers.
- Flex and ripple. They need motion: `material-36e968`.
