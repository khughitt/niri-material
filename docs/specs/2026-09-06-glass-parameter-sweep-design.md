# Glass parameter sweep script: design

**Status:** designed, not yet implemented. Branch `harness/glass-parameter-sweep`.

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
| `KEY` | yes | KDL key, e.g. `ior`, `thickness`, `jelly-flex`, `noise` |
| `VALUES` | yes | whitespace-separated values, e.g. `"1 1.25 1.5 1.75 2 2.5 3"` |
| `ROI` | no | crop geometry, default `200x400+100+160` |

Both blocks are injectable because Prism's slider list spans both: refraction,
depth, flex and ripple are `glass` keys; blur, noise and saturation live in the
top-level `blur` block.

The caller supplies the values rather than a range and a count. Ranges worth
sweeping are rarely uniform — refraction wants density near 1.4 to 1.7 and a
few coarse probes above it — and a value list keeps that choice with the
person reading the table.

### Scene

The backdrop is a generated high-frequency pattern (a grid over color patches),
not the model script's three flat color bars.

Refraction and distortion bend the backdrop. Across a flat field that bending
is invisible except where it crosses the two bar boundaries, so a color-bar
sweep would under-report exactly the parameter Prism most needs measured. A
patterned backdrop registers the bend across the whole ROI.

Glass values are pinned in the script and recorded in the evidence document.
They are not derived from a generated `prism.kdl`: that file drifts (`ior`
moved 1.02 to 1.24 within a week), which would make two runs of the same sweep
incomparable.

### Metrics

Per value, in Lab colorspace:

| Column | Meaning |
| --- | --- |
| `neighbor` | RMSE against the previous value |
| `cumulative` | RMSE against the first value |
| `normalized` | `neighbor` over the largest `neighbor` in the sweep |

`neighbor` locates where the effect stops changing. `cumulative` separates a
parameter that saturated from one that never moved — both show a small
`neighbor`, and only `cumulative` tells them apart. `normalized` makes the dead
zone readable without reading absolute RMSE values.

Lab rather than sRGB RMSE because the question is perceptual: how much visible
change does a slider step buy. If `magick compare` does not honour
`-colorspace Lab`, each capture is converted to Lab first and those are
compared — equivalent, and explicit about when the conversion happens.

### Output

A TSV table to stdout and `$OUT/sweep.tsv`, one row per value, plus the
per-value PNGs, ROI crops and configs the model script already writes.
`$OUT/SHA256SUMS` and the niri version are recorded the same way.

### Failure

Exit non-zero only for harness failure: a capture that was not written, a
non-numeric metric, a config that does not validate, or a material error,
shader fallback or panic in `niri.log`. The numeric guards come from the model
script, so a broken capture cannot be reported as a small delta.

## Structure

The script is self-contained. The harness core — `write_config`,
`start_nested`, `stop_nested`, `capture`, `compare_metric`, `is_number`,
`fail`, `cleanup` — is copied from `glass-noise-saturation-smoke.sh`, which is
how the five existing scripts in that directory already relate to each other:
each carries its own copy and there is no library to import.

Extracting a shared `lib-headless.sh` and moving both scripts onto it is the
alternative. It is not taken here because the model script is recorded
evidence for `material-1293e8`, and rewriting its harness underneath a merged
result costs more than the duplication saves at two call sites. If a third
sweep-shaped script appears, extract then.

## Verification

A real run on the headless Weston host sweeping `ior` over seven values, with
the resulting table recorded as evidence under `docs/materials/`. That run is
also the artifact `prism-8e8a18` consumes, so the acceptance evidence and the
first consumer are the same thing.

Checks that the run must satisfy:

- every value produces a capture, and the table has one row per value
- `cumulative` is non-zero somewhere — the parameter demonstrably moves the image
- `niri.log` is free of material errors, shader fallbacks and panics
- a second run of the same sweep reproduces the table

## Non-goals

- Choosing the bounds. The script reports; `prism-5758d3` decides.
- Sweeping two parameters jointly. One key per run.
- Asserting thresholds. No pass/fail on the numbers.
