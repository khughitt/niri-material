# Glass parameter sweep: refraction evidence

**Task:** `material-37cec9`.
**Design:** [`2026-09-06-glass-parameter-sweep-design.md`](../specs/2026-09-06-glass-parameter-sweep-design.md).
**Script:** [`scripts/glass-parameter-sweep.sh`](scripts/glass-parameter-sweep.sh).
**Binary:** `niri 26.04 (5dbe182d)`, the installed `niri-material 26.04.r278.g5dbe182d-1`.

First run of the sweep, against the parameter `prism-8e8a18` asked for.

```
NIRI=/usr/bin/niri OUT=<dir> BLOCK=glass KEY=ior \
  VALUES="1 1.25 1.4 1.55 1.7 2.2 3" \
  docs/materials/scripts/glass-parameter-sweep.sh
```

## What this table is

**It reports where the rendered image changes as `ior` varies. It does not
report why, and nothing in it is evidence that the ray bends.** See "Bending is
not measured here".

## Scene

Headless Weston, 1280x720, kiosk shell. One blank transparent kitty over a
generated backdrop: color patches under a 20px grid. Window rect measured from
an opaque material-free probe at `704x640+40+40`.

| ROI | Crop | Region |
| --- | --- | --- |
| `bevel` | `27x320+35+200` | band straddling the window's left edge, covering the 12px chamfer |
| `face` | `234x213+275+254` | centered box inside the flat inner face |

Glass baseline, pinned in the script rather than taken from a generated
`prism.kdl`: `thickness 20`, `attenuation-color "#dfe8ff"`,
`attenuation-distance 60`, `chromatic-aberration 0`, `distortion 0 scale=0.5`,
`anisotropic-blur 0`, `roughness 0`, `backdrop-blur false`, `jelly-flex 0`,
`jelly-ripple 0`, `bevel 12`, `offset-x 0`, `offset-y 0`, and
`response "default" { focus "none" }`.

## Result

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | - | - | - | 0 | - | - | - | 0 | - |
| 1.25 | 0.25 | 0.0552609 | 0.221044 | 0.0552609 | 0.4376 | 0.00364698 | 0.0145879 | 0.00364698 | 0.3868 |
| 1.4 | 0.15 | 0.0757632 | 0.505088 | 0.0674038 | 1.0000 | 0.00459197 | 0.0306131 | 0.00803898 | 0.8118 |
| 1.55 | 0.15 | 0.0532471 | 0.354981 | 0.0268651 | 0.7028 | 0.00565668 | 0.0377112 | 0.0135105 | 1.0000 |
| 1.7 | 0.15 | 0.00765512 | 0.0510341 | 0.0338847 | 0.1010 | 0.00454362 | 0.0302908 | 0.0178559 | 0.8032 |
| 2.2 | 0.5 | 0.0250198 | 0.0500396 | 0.058613 | 0.0991 | 0.0161886 | 0.0323772 | 0.0339719 | 0.8586 |
| 3 | 0.8 | 0.0288793 | 0.0360991 | 0.0871472 | 0.0715 | 0.0199162 | 0.0248952 | 0.0537994 | 0.6602 |

All values are Lab RMSE, normalized column derived from per-step.

## Reading

The two regions behave differently, which is the point of separating them.

**The bevel band front-loads.** Per-step change peaks at `1.4` and has fallen to
a tenth of that by `1.7`. From `1.7` to `3` — most of the slider's travel —
per-step sits between `0.036` and `0.051`, against `0.505` at the peak. The
region where a step of `ior` buys visible change at the window edge is roughly
`1.0` to `1.7`, and the top half of the `[1, 3]` range is close to flat.

**The face keeps changing.** Per-step is far smaller throughout (peak `0.0377`
against the bevel's `0.505`) but does not collapse: it is still `0.0249` at `3`,
two thirds of its peak, and `face_cumulative` rises monotonically across the
whole range. Whatever the face is doing, it does not saturate where the bevel
does.

For `prism-5758d3`, that is the substantive finding: a slider bounded at `3` and
scaled linearly spends its upper half delivering very little edge change, which
matches the reported symptom of a narrow transition and a long dead zone.

For `prism-8e8a18` specifically, the question was whether the milkiness is a
range problem or a rendering one. This table narrows it — the face continues to
change as `ior` rises while the edge stops — but does not settle it, because
settling it requires knowing what the face change *is*, which is the next
section.

## Bending is not measured here

Fresnel `f0 = (ior-1)/(ior+1)` (`material.frag:454`) varies with `ior` wherever
`coverage > 0`, and the focus filament refracts through
`1.0 + (mat_ior-1.0) * mat_light_ior` (`:342`). Either alone would produce a
rising curve in both columns with backdrop refraction completely broken. The
table above cannot distinguish them.

Two instruments were built and rejected in this task. Both are recorded in the
design doc; `material-343f27` owns the follow-up.

**Template displacement**, `magick compare -subimage-search` with NCC on a
unique asymmetric marker in a constrained window. It passed its own synthetic
self-checks on the real run — `+5,+0` translation reported `+5,+0`, brightness
`x1.35` at zero translation reported `+0,+0` — and reported `0,0` displacement
at every value from `1` to `3` while `bevel_cumulative` rose to `0.077`. The
template spanned x 35-65 against a chamfer at x 40-52, so 18 of 30 columns could
not bend at `distortion 0` and pinned the match. Repositioning does not fix the
deeper problem: chamfer refraction is a spatially varying warp, not a rigid
translation, and the flat face — the one region where displacement would be
rigid — does not bend at `distortion 0`.

**Flat-versus-textured control**, exploratory only, retained for whoever picks
up `material-343f27`:

| backdrop | bevel delta, `ior 1` to `ior 3` |
| --- | --- |
| flat `rgb(96,96,96)` | 0.0512 |
| textured (the sweep's) | 0.0772 |

**No conclusion about bending is drawn from these two numbers.** Subtracting
RMSE values does not isolate the effect present in one and absent from the
other: RMSE combines effects nonlinearly. The two backdrops also differ in mean
color, which moves the Fresnel and attenuation response independently of any
texture. The pair is suggestive and nothing more.

## Reproducibility

Two independent runs of `VALUES="1 1.4 1.7 3"` produced byte-identical
`sweep.tsv` files, and every ROI crop compared at `AE 0`.

This required a fix the first runs exposed. With the focus response left at its
default, two runs of the same sweep differed in the bevel band by Lab RMSE
`0.077` — the same magnitude as the entire `ior 1` to `ior 3` signal — while the
face band was byte-identical. The default response is `RingLight`, which drifts
over time and is drawn at the window edge, exactly the bevel band. The script
now pins `response "default" { focus "none" }`.

Anyone comparing figures against an earlier capture set should check for that
pin: numbers taken without it are not comparable, and the bevel column in
particular was measuring the ring's phase as much as the glass.

## Checks

| Check | Result |
| --- | --- |
| one row per value, seven captures | pass |
| both ROIs identical screen coordinates across the run | pass, `27x320+35+200` and `234x213+275+254` |
| `bevel_cumulative` non-zero | pass, `0.0871` at `ior 3` |
| `niri.log` free of material errors, fallbacks, panics | pass |
| second run reproduces the table | pass, byte-identical |
| table stage over duplicate captures: sentinels then zeros | pass |
| table stage positive control: unequal steps, equal image change | pass, `per_step` flattens what `neighbor` exaggerates |
| **bending demonstrated** | **not met**, deferred to `material-343f27` |
| **flex and ripple** | **not met**, out of scope, `material-36e968` |
