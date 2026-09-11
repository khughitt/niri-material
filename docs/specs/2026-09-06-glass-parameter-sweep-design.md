# Glass parameter sweep script: design

**Status:** implemented on branch `harness/glass-parameter-sweep`; `ior`
evidence in
[`2026-09-06-glass-parameter-sweep-evidence.md`](../materials/2026-09-06-glass-parameter-sweep-evidence.md).

Revised three times under review. What shipped: two fixed ROIs, blur gating,
per-step normalization, the focus ring pinned off for reproducibility. The static tool does not establish ray bending
(`material-343f27` remains open) or measure motion. Flex/ripple now have a
[separate motion sweep](../materials/2026-09-11-jelly-motion-sweep.md)
(`material-36e968`), with measured progress alignment and repeat-noise checks.

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
| `TABLE_ONLY` | no | `1` to recompute the table from captures already in `OUT` |

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

`jelly-flex` and `jelly-ripple` are **not** supported by the static script.
`jelly_state` in `src/render_helpers/material/mod.rs` derives jelly activity
from motion residuals, and the shader gates ripple behind
`mat_jelly_activity > 0.0`, so on a settled window with
`animations { off; }` both render identically at every value across their whole
range. A sweep that reported them would report zeros and read as "no effect".
The [motion companion](../materials/2026-09-11-jelly-motion-sweep.md)
(`material-36e968`) supplies a scripted stimulus, timed bursts, and alignment
by measured pane progress.

Sweeping `bevel` changes the chamfer the bevel ROI frames. The ROI is sized to
cover the whole swept range and then held fixed — see "Two ROIs".

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

The grid is periodic. That is fine for RMSE, which does not care, and wrong for
template matching, which slides by exactly one period — see the bending note.

Glass values are pinned in the script and recorded in the evidence document.
They are not derived from a generated `prism.kdl`: that file drifts (`ior` moved
1.02 to 1.24 within a week), which would make two runs of the same sweep
incomparable.

The material also pins `response "default" { focus "none" }`. The focus response
defaults to `RingLight`, drawn at the window edge — exactly the bevel band — and
the ring is a separate feature with its own parameters, so it does not belong in
a measurement of glass optics.

An earlier version of this section said the pin was needed because the ring
"drifts over time" and had moved the bevel column between two runs by Lab RMSE
`0.077`. That is wrong, and `material-0af212` measured it: `animations { off; }`
already pins the drift rate to zero (`drift_rate` returns `0.` when animations
are off, `src/render_helpers/signal.rs:233-236`) and that line was present in the
run that varied. Two runs with the ring left on compare at `AE 0` with identical
`sweep.tsv`, and so do two runs of the pre-pin script (`d69fd99b`), which also
carried the backdrop marker the same commit removed. The `0.077` did not
reproduce and its cause is not established. The pin stays as a precaution, not
as a fix.

### Two ROIs

`slabSurface` (`material.frag:319-325`) sets `normal = vec3(0,0,1)` everywhere
inside the inner face, and `tap` (`:334`) refracts against it with
`refract(vec3(0,0,-1), vec3(0,0,1), 1.0/ior)`, whose `.xy` is identically zero.
**With `distortion = 0`, lateral refraction in the flat face is zero at every
IOR.** The qualification matters: `:392` perturbs the face normal whenever
`mat_distortion > 0`, at which point the face bends too. The pinned baseline
sets `distortion 0`, so the property holds for every sweep run against it — but
it is a property of that baseline, not of the shader.

The normal tilts on its own only in the chamfer ring, where `di >= 0`. So the
script reports two regions:

| ROI | Where |
| --- | --- |
| `bevel` | band straddling the window edge, covering the chamfer |
| `face` | centered box well inside the inner face |

Both are **fixed for the whole sweep**: derived once, from the first capture's
window geometry, and applied as the same screen coordinates to every value.
RMSE between crops taken at different positions measures the crop, and between
crops of different sizes it does not compute at all. When `bevel` is the swept
key the band is sized from `max(VALUES)` so it covers the widest chamfer in the
sweep; otherwise from the pinned `bevel`. A sweep whose band would be
degenerate — sweeping `bevel` with every value `0` — is rejected rather than
silently compared over an empty region. The window rect itself is constant
across a sweep: every supported key is an optic rendered inside the rect, not a
layout input.

Both ROIs are overridable for a parameter that wants a different frame.

### What the ROI metrics do and do not show

The two columns are **regional image change**, nothing more. Neither is evidence
of ray bending, and the spec does not claim otherwise:

- Fresnel varies on the chamfer as well as the face — `f0 = (ior-1)/(ior+1)`
  at `:454` is a function of IOR everywhere `coverage > 0`.
- The focus filament refracts through `1.0 + (mat_ior-1.0) * mat_light_ior`
  (`lightShift`, `:342`), so it moves with IOR too.

A non-zero `bevel` delta is therefore consistent with backdrop refraction being
entirely broken. **The script does not report bending, and no column of its
output should be read as bending.** Establishing why the image changes is
deferred to `material-343f27`, which owes its own validated instrument.

Two instruments were built and rejected during this task. Both are recorded
because the reasons generalize, and a later attempt that repeats either will
waste the same effort.

**Template displacement.** `magick compare -subimage-search`, NCC on a unique
asymmetric marker in a window constrained to its known position. The matcher
passed its own synthetic self-checks on the real run — a known `+5,+0`
translation reported `+5,+0`, and a `-evaluate multiply 1.35` brightness change
at zero translation reported `+0,+0` — and still reported `0,0` displacement at
every value from `ior 1` to `ior 3`, while `bevel_cumulative` rose to `0.0772`.

The immediate cause was placement: the template spanned x 35–65 while the
chamfer was only x 40–52, so 18 of its 30 columns were raw backdrop or flat
face, neither of which bends at `distortion 0`, and the non-bending majority
pinned the match. The deeper cause is not fixable by moving the template.
Refraction through the chamfer is a spatially varying **warp**, not a rigid
translation, so a matcher that reports one offset is answering the wrong
question; and the flat face, the only region where displacement would be rigid,
does not bend at all at `distortion 0`. Widening the bevel or shrinking the
template addresses the placement and leaves the warp.

**Flat-versus-textured control.** Capture `ior 1` and `ior 3` over the textured
backdrop and again over a flat one, on the reasoning that bending can only
manifest where there is texture to displace, so the difference isolates it. The
run gave a bevel delta of `0.0512` flat against `0.0772` textured.

That subtraction does not isolate bending. RMSE combines effects nonlinearly, so
the difference of two RMSE values is not the magnitude of the effect present in
one and absent from the other. The two backdrops also differ in mean color,
which moves the Fresnel and attenuation response independently of any texture.
The numbers are kept in the evidence document as exploratory only, and no
conclusion about how much of the bevel delta is bending is drawn from them.

Separating face from bevel still serves `prism-8e8a18`: whether refraction's
milkiness tracks the face or the bevel is a useful question, and it is answerable
from where the image changes. Whether the milkiness is a range problem or a
rendering one is not answerable from this script.

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
change does a slider step buy.

Measured on IM 7.1.2-31, on one pair of flat colors:

| Route | RMSE |
| --- | --- |
| `compare -colorspace Lab a b` | `0.1198` |
| convert each to Lab as MIFF, compare those | `0.119849` |
| convert each to Lab as TIFF, compare those | **`0`** |
| convert each to Lab as PNG, compare those | `0.245946` (the sRGB value) |

`compare` honours `-colorspace Lab`, so the metric is taken that way directly
and no intermediate file is written. The earlier draft's fallback — convert each
capture to Lab, then compare the converted files — is **wrong** and is recorded
here so it is not reintroduced: PNG cannot store Lab and magick converts back on
write, silently yielding the sRGB answer, and the TIFF route silently reports
`0` for a pair that plainly differs. A zero from a broken conversion is
indistinguishable from the flat sweep this script is built to report, which
makes it the worst available failure. If an intermediate is ever needed, MIFF is
the only container of the three that survives the round-trip.

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

The table is computed in a pass that reads the captures back from disk, separate
from the pass that produces them. That separation is what makes the
zero-normalization case checkable against prepared images, and it lets a table
be recomputed with different metrics without re-running the harness.

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
- both ROIs are the same screen coordinates in every row of the run
- `bevel_cumulative` is non-zero — the region responds to IOR at all
- `niri.log` is free of material errors, shader fallbacks and panics
- a second run of the same sweep reproduces the table, byte for byte

### Explicitly unmet

Two criteria an earlier draft of this spec asserted are **not** met, and are
recorded here rather than quietly dropped.

**Bending.** No check here establishes that the ray bends. `bevel_cumulative`
being non-zero says the region responds to IOR, which Fresnel and the filament
are sufficient to explain. The instrument that was supposed to close this —
marker displacement — failed validation on the real run and is deferred to
`material-343f27` along with the reasons. Anyone reading this table for evidence
that refraction works is reading it wrong.

**Flex and ripple.** Out of scope for the static harness; `material-36e968`.

A second run sweeping glass `noise` exercises the noise path in both regions.
It does **not** exercise the flat-sweep path: `:535-539` adds noise to
`glassColor` before multiplying by `coverage`, so noise lands on the chamfer as
well as the face and distinct values always differ somewhere.

The zero-normalization path is checked directly instead. The table stage reads
the captures back from disk in a pass separate from capturing them, so it can be
pointed at a prepared directory. Run over two copies of a single capture, it
must produce the first row's sentinels unchanged — `-` for `step`, `neighbor`,
`per_step` and `normalized`, `0` for `cumulative` — and `0` for every one of
those columns on the second and subsequent rows, with no division error. The
sentinel contract and the zero contract apply to different rows and neither
overrides the other. Keeping the two stages separable is a requirement of the
design, not an implementation detail.

## Non-goals

- Choosing the bounds. The script reports; `prism-5758d3` decides.
- Sweeping two parameters jointly. One key per run.
- Asserting thresholds. No pass/fail on the numbers.
- Flex and ripple. Measured separately by the motion companion (`material-36e968`).
