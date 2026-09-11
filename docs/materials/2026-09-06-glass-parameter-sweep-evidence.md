# Glass parameter sweep: evidence

**Tasks:** `material-37cec9` (the `ior` run), `material-48dc76` (the remaining
Prism slider candidates).
**Design:** [`2026-09-06-glass-parameter-sweep-design.md`](../specs/2026-09-06-glass-parameter-sweep-design.md).
**Script:** [`scripts/glass-parameter-sweep.sh`](scripts/glass-parameter-sweep.sh).
**Binary:** `niri 26.04 (5dbe182d)`, the installed `niri-material 26.04.r278.g5dbe182d-1`,
for every table in this document.

Part 1 is the first run of the sweep, against the parameter `prism-8e8a18`
asked for. Part 2 runs the rest of `prism-5758d3`'s candidates.

# Part 1: refraction (`ior`)

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

### The ring was not the cause — correction

This document originally said the first runs varied because the focus response
defaults to `RingLight`, "which drifts over time and is drawn at the window
edge", and that pinning `response "default" { focus "none" }` fixed it. That
explanation is wrong. `material-0af212` re-ran the experiment on `a0559ebf`:

| variant, two independent runs | AE | `sweep.tsv` |
| --- | --- | --- |
| as shipped (ring pinned off) | 0 | identical |
| ring left at its default | 0 | identical |
| pre-pin script `d69fd99b` (ring on, backdrop marker present) | 0 | identical |

The mechanism could not have been drift in any case: `drift_rate` returns `0.`
when animations are off (`src/render_helpers/signal.rs:233-236`), and
`animations { off; }` was already in the script during the run that varied.

The commit that added the pin, `0b6b4ce3`, also removed a backdrop marker that
was painted at the window edge, so the pin was never isolated from that change —
and neither change is needed for reproducibility on this host. **The `0.077` did
not reproduce and its cause is not established.** The pin stays because the ring
is a separate feature that does not belong in a glass measurement, not because
it fixes anything.

The ring is nonetheless real and visible: with animations off it still draws,
contributing `AE 3675` against the same frame with `focus "none"`. Comparisons
across the pin are therefore not valid, even though comparisons across runs
within either setting are.

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
| **flex and ripple** | Outside this static run; subsequently measured by the [motion companion](2026-09-11-jelly-motion-sweep.md), `material-36e968` |

# Part 2: the remaining Prism slider candidates

**Task:** `material-48dc76`, 2026-09-06. Same binary, same script (last changed in
`78d581e4`), same headless Weston host. Every run measured the window at
`704x640+40+40` and derived the same two ROIs as Part 1, `bevel 27x320+35+200`
and `face 234x213+275+254`, so the columns are comparable across all nine
tables in this document. Every `niri.log` is free of material errors,
fallbacks and panics.

`prism-5758d3` names depth, blur, noise and saturation. Against
`prism/defs/glass.yaml`: depth is `glass.thickness` `[0, 200]`; **Prism's
"Blur" slider is `glass.roughness` `[0, 1]`**, not niri's global `blur` block;
noise is `glass.noise` `[0, 1]`; saturation is `glass.saturation` `[0, 3]`.
Those four are the slider tables. The global `blur` block's four keys are
swept as well because that is the `BLOCK=blur` path of the script, which had
not been run: it sets `backdrop-blur true` and omits the material's own noise
and saturation so the global values are inherited. Prism exposes the block only
through the `backdropBlur` boolean, so those four tables inform niri config
bounds rather than a Prism slider.

Values span each range end to end and are denser at the low end, where the
`ior` run showed the change concentrates. All values are Lab RMSE; the
normalized columns derive from per-step.

## Sliders

### `glass thickness` (Prism: Depth, `[0, 200]`, default 20)

```
BLOCK=glass KEY=thickness VALUES="0 5 10 20 40 80 120 200"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 5 | 5 | 0.0499784 | 0.00999568 | 0.0499784 | 0.8410 | 0.000745634 | 0.000149127 | 0.000745634 | 0.5619 |
| 10 | 5 | 0.0594264 | 0.0118853 | 0.054687 | 1.0000 | 0.00127631 | 0.000255262 | 0.0018271 | 0.9618 |
| 20 | 10 | 0.0459981 | 0.00459981 | 0.0251297 | 0.3870 | 0.00265404 | 0.000265404 | 0.00411203 | 1.0000 |
| 40 | 20 | 0.0149083 | 0.000745415 | 0.0167739 | 0.0627 | 0.00429648 | 0.000214824 | 0.00818501 | 0.8094 |
| 80 | 40 | 0.0464275 | 0.00116069 | 0.0570524 | 0.0977 | 0.00802128 | 0.000200532 | 0.0161434 | 0.7556 |
| 120 | 40 | 0.0481391 | 0.00120348 | 0.027558 | 0.1013 | 0.00758704 | 0.000189676 | 0.0236373 | 0.7147 |
| 200 | 80 | 0.0159916 | 0.000199895 | 0.041674 | 0.0168 | 0.0152474 | 0.000190592 | 0.0387857 | 0.7181 |

**Front-loaded at the edge, and the cumulative column is not trustworthy
here.** Bevel per-step peaks at `5` to `10` (`0.010` to `0.012`), is a third of
that by `20` and about a tenth from `40` on. Bevel cumulative, though, is not
monotonic: `0.055` at `10`, `0.025` at `20`, `0.057` at `80`, `0.028` at `120`.
Thickness sets how far the chamfer displaces the backdrop, and the backdrop is
a grid with a 20px period, so a displacement near a multiple of the period
compares well against the origin and one near a half period compares badly.
The oscillation is the instrument, not the glass. This is a limitation of the
periodic backdrop for any parameter that translates it (see "Instrument
caveat" below); per-step between neighbours is affected less but not immune,
so read the low-end concentration as the finding and the exact numbers past
`40` as unreliable. The face barely moves: per-step `0.0002` throughout and
cumulative `0.039` at `200`, consistent with a face that refracts nothing at
`distortion 0`. For the slider: the region where a step buys visible edge
change is `0` to `40`, a fifth of the range.

### `glass roughness` (Prism: Blur, `[0, 1]`, default 0.08)

```
BLOCK=glass KEY=roughness VALUES="0 0.05 0.1 0.2 0.4 0.7 1"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.05 | 0.05 | 0.0432947 | 0.865894 | 0.0432947 | 0.9659 | 0.0406333 | 0.812666 | 0.0406333 | 0.9655 |
| 0.1 | 0.05 | 0.0448252 | 0.896504 | 0.0880852 | 1.0000 | 0.0420836 | 0.841672 | 0.0826838 | 1.0000 |
| 0.2 | 0.1 | 0.0357402 | 0.357402 | 0.106542 | 0.3987 | 0.0329482 | 0.329482 | 0.0999401 | 0.3915 |
| 0.4 | 0.2 | 0.0299371 | 0.149685 | 0.115355 | 0.1670 | 0.030313 | 0.151565 | 0.107955 | 0.1801 |
| 0.7 | 0.3 | 0.0355849 | 0.118616 | 0.122352 | 0.1323 | 0.0373875 | 0.124625 | 0.11577 | 0.1481 |
| 1 | 0.3 | 0.0548665 | 0.182888 | 0.143785 | 0.2040 | 0.04871 | 0.162367 | 0.132424 | 0.1929 |

**The steepest curve in this document.** Per-step is `0.87` to `0.90` from `0`
to `0.1`, `0.36` for `0.1` to `0.2`, `0.15` for `0.2` to `0.4`, and `0.12` to
`0.18` for the rest. Cumulative reaches `0.088` at `0.1` and `0.144` at `1`, so
61% of the total change over the slider happens in its first tenth, and the
face behaves the same way (`0.083` of `0.132`). The mechanism is in
`effect_buffer.rs`: level-of-detail is `levels * roughness * clamp(2*ior - 2)`,
which is `levels * roughness` at the baseline `ior 1.5`, and successive pyramid
levels each halve the resolution, so equal steps in roughness buy
geometrically less image change. Prism's default `0.08` sits in the steep
part. This is the clearest case for a logarithmic transform among the
candidates, and the upper half of the range is close to flat. The rise at
`0.7` to `1` (`0.18`) is the last, smallest pyramid level being reached.

### `glass noise` (Prism: Noise, `[0, 1]`, default 0)

```
BLOCK=glass KEY=noise VALUES="0 0.05 0.1 0.2 0.4 0.7 1"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.05 | 0.05 | 0.00886339 | 0.177268 | 0.00886339 | 1.0000 | 0.00888584 | 0.177717 | 0.00888584 | 0.9979 |
| 0.1 | 0.05 | 0.00768284 | 0.153657 | 0.0176317 | 0.8668 | 0.00890475 | 0.178095 | 0.0176865 | 1.0000 |
| 0.2 | 0.1 | 0.0153085 | 0.153085 | 0.0352549 | 0.8636 | 0.0153486 | 0.153486 | 0.0353566 | 0.8618 |
| 0.4 | 0.2 | 0.0291176 | 0.145588 | 0.0686447 | 0.8213 | 0.0289214 | 0.144607 | 0.0685185 | 0.8120 |
| 0.7 | 0.3 | 0.0382471 | 0.12749 | 0.110237 | 0.7192 | 0.0373881 | 0.124627 | 0.108694 | 0.6998 |
| 1 | 0.3 | 0.0350932 | 0.116977 | 0.146944 | 0.6599 | 0.0345507 | 0.115169 | 0.144633 | 0.6467 |

**Close to linear, no dead zone.** Per-step declines gently and monotonically
from `0.177` to `0.117`, cumulative rises monotonically to `0.147`, and bevel
and face are equal to three digits at every row because noise is applied after
the optics and does not care about geometry. The whole range is useful and a
linear slider is the right shape; the response is mildly sublinear at the top,
not saturated.

### `glass saturation` (Prism: Saturation, `[0, 3]`, default 1)

```
BLOCK=glass KEY=saturation VALUES="0 0.5 1 1.5 2 2.5 3"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.5 | 0.5 | 0.0304407 | 0.0608814 | 0.0304407 | 0.8570 | 0.0319561 | 0.0639122 | 0.0319561 | 0.8516 |
| 1 | 0.5 | 0.0335861 | 0.0671722 | 0.0636274 | 0.9455 | 0.0354448 | 0.0708896 | 0.0669544 | 0.9446 |
| 1.5 | 0.5 | 0.0355211 | 0.0710422 | 0.0980576 | 1.0000 | 0.0375232 | 0.0750464 | 0.103265 | 1.0000 |
| 2 | 0.5 | 0.0345583 | 0.0691166 | 0.131225 | 0.9729 | 0.0362526 | 0.0725052 | 0.138008 | 0.9661 |
| 2.5 | 0.5 | 0.0334022 | 0.0668044 | 0.162989 | 0.9403 | 0.0346916 | 0.0693832 | 0.170962 | 0.9245 |
| 3 | 0.5 | 0.0121589 | 0.0243178 | 0.172096 | 0.3423 | 0.0114215 | 0.022843 | 0.179007 | 0.3044 |

**Linear from `0` to `2.5`, then it clips.** Per-step is `0.061` to `0.071`
across the first five steps and symmetric around `1`, so desaturating and
oversaturating by the same amount cost the same. The last step, `2.5` to `3`,
delivers a third of the others (`0.024`): channels are clipping. A linear
slider is right; the top of the range past about `2.5` is where it stops
paying, and on this backdrop that is the only saturated region.

## The global `blur` block (`BLOCK=blur`)

These four runs set `backdrop-blur true` on the material and inherit the
block's noise and saturation. The script's structural gate held on every
config it wrote.

### `blur noise` (`[0, 1]`, niri default 0.02)

```
BLOCK=blur KEY=noise VALUES="0 0.05 0.1 0.2 0.4 0.7 1"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.05 | 0.05 | 0.00770048 | 0.15401 | 0.00770048 | 1.0000 | 0.00886873 | 0.177375 | 0.00886873 | 1.0000 |
| 0.1 | 0.05 | 0.00770047 | 0.154009 | 0.0152968 | 1.0000 | 0.007678 | 0.15356 | 0.0176114 | 0.8657 |
| 0.2 | 0.1 | 0.0153146 | 0.153146 | 0.0305566 | 0.9944 | 0.01529 | 0.1529 | 0.0352018 | 0.8620 |
| 0.4 | 0.2 | 0.0306076 | 0.153038 | 0.0611232 | 0.9937 | 0.0305043 | 0.152522 | 0.0703751 | 0.8599 |
| 0.7 | 0.3 | 0.0418876 | 0.139625 | 0.10211 | 0.9066 | 0.0412333 | 0.137444 | 0.116757 | 0.7749 |
| 1 | 0.3 | 0.0364524 | 0.121508 | 0.134938 | 0.7890 | 0.0359618 | 0.119873 | 0.153882 | 0.6758 |

### `blur saturation` (`[0, 3]`, niri default 1)

```
BLOCK=blur KEY=saturation VALUES="0 0.5 1 1.5 2 2.5 3"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.5 | 0.5 | 0.0283356 | 0.0566712 | 0.0283356 | 1.0000 | 0.0287719 | 0.0575438 | 0.0287719 | 0.8674 |
| 1 | 0.5 | 0.0267881 | 0.0535762 | 0.0590065 | 0.9454 | 0.0313797 | 0.0627594 | 0.0598769 | 0.9461 |
| 1.5 | 0.5 | 0.027912 | 0.055824 | 0.0905529 | 0.9851 | 0.0326947 | 0.0653894 | 0.0918394 | 0.9857 |
| 2 | 0.5 | 0.0283222 | 0.0566444 | 0.122218 | 0.9995 | 0.0331685 | 0.066337 | 0.123911 | 1.0000 |
| 2.5 | 0.5 | 0.0279634 | 0.0559268 | 0.153171 | 0.9869 | 0.0325977 | 0.0651954 | 0.155127 | 0.9828 |
| 3 | 0.5 | 0.0133081 | 0.0266162 | 0.166099 | 0.4697 | 0.0158748 | 0.0317496 | 0.168122 | 0.4786 |

**The inherited pair reproduces the written pair.** Blur noise per-step runs
`0.154` to `0.122` against glass noise's `0.177` to `0.117`; blur saturation
runs `0.054` to `0.057` then `0.027` against glass saturation's `0.061` to
`0.071` then `0.024`. Same shape, same clip at `3`, small offsets from the
blurred backdrop the inherited values act on. That is the result the
`BLOCK=blur` path exists to produce: the inheritance gate is open when it
should be and the inherited values render. A flat table here would have been
the gate artifact the script's header warns about.

### `blur passes` (`1` to `31`, niri default 3)

```
BLOCK=blur KEY=passes VALUES="1 2 3 4 6 8"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | - | - | - | 0 | - | - | - | 0 | - |
| 2 | 1 | 0.0151462 | 0.0151462 | 0.0151462 | 0.7629 | 0.018457 | 0.018457 | 0.018457 | 0.8448 |
| 3 | 1 | 0.0168423 | 0.0168423 | 0.0263049 | 0.8484 | 0.0140623 | 0.0140623 | 0.0273956 | 0.6436 |
| 4 | 1 | 0.0127055 | 0.0127055 | 0.0341514 | 0.6400 | 0.0179253 | 0.0179253 | 0.0399093 | 0.8204 |
| 6 | 2 | 0.0261593 | 0.0130796 | 0.0581764 | 0.6588 | 0.0436978 | 0.0218489 | 0.0688932 | 1.0000 |
| 8 | 2 | 0.0397051 | 0.0198526 | 0.083672 | 1.0000 | 0.0239231 | 0.0119615 | 0.0863658 | 0.5475 |

**Still changing at `8`.** Per-step sits between `0.013` and `0.020` in the
bevel and cumulative climbs steadily to `0.084`; the face is non-monotonic per
step (`0.022` for `4` to `6`, `0.012` for `6` to `8`). Each pass halves the
working resolution (`1280x720` is `5x2` after eight), so the curve cannot go
on much longer, but no plateau is reached in the range a config would plausibly
use. Not a Prism slider.

### `blur offset` (niri default 3, baseline 8)

```
BLOCK=blur KEY=offset VALUES="1 2 4 8 16 32"
```

| value | step | bevel neighbor | bevel per-step | bevel cumulative | bevel norm | face neighbor | face per-step | face cumulative | face norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | - | - | - | 0 | - | - | - | 0 | - |
| 2 | 1 | 0.0198295 | 0.0198295 | 0.0198295 | 1.0000 | 0.0221526 | 0.0221526 | 0.0221526 | 1.0000 |
| 4 | 2 | 0.020421 | 0.0102105 | 0.0337269 | 0.5149 | 0.0246718 | 0.0123359 | 0.0393857 | 0.5569 |
| 8 | 4 | 0.0108515 | 0.00271288 | 0.0354464 | 0.1368 | 0.0135499 | 0.00338748 | 0.0421091 | 0.1529 |
| 16 | 8 | 0.0130663 | 0.00163329 | 0.0355597 | 0.0824 | 0.0152348 | 0.00190435 | 0.0434893 | 0.0860 |
| 32 | 16 | 0.0358368 | 0.0022398 | 0.0489234 | 0.1130 | 0.0164069 | 0.00102543 | 0.0486823 | 0.0463 |

**Front-loaded, plateau from `4` to `16`.** Per-step is `0.020` for `1` to `2`,
`0.010` for `2` to `4`, then `0.0016` to `0.0027` for everything after.
Cumulative sits at `0.034` to `0.036` from `4` through `16` and moves only at
`32`. The useful region is `1` to about `8`; beyond it the kernel is already
wider than the features it blurs. Not a Prism slider.

## Instrument caveat: a periodic backdrop and displacing parameters

The backdrop is a 20px grid so that any change registers across the ROI. The
`ior` run notes that this is fine for RMSE and wrong for template matching. The
thickness run adds a qualification: RMSE against the first capture is also
unreliable for a parameter whose effect is to translate the backdrop, because a
translation by a whole period is nearly invisible to it. Neighbouring deltas
are less exposed, since the displacement between adjacent values is usually a
fraction of a period, but a large step (`120` to `200`) can cross one. None of
the other seven parameters translate the backdrop, so their tables are not
affected. The fix, an aperiodic backdrop or a displacement instrument, belongs
with `material-343f27`.

## Summary for `prism-5758d3`

| Slider | Range | Shape | Useful region | Suggestion |
| --- | --- | --- | --- | --- |
| Depth (`thickness`) | `[0, 200]` | front-loaded at the edge, face flat | `0` to `40` | shrink the range, or a log scale |
| Blur (`roughness`) | `[0, 1]` | steep: 61% of the change in the first tenth | `0` to `0.2` | log scale; the top half is nearly flat |
| Noise | `[0, 1]` | near linear, mildly sublinear | whole range | keep |
| Saturation | `[0, 3]` | linear and symmetric about `1`, clips past `2.5` | `0` to `2.5` | keep the shape; consider capping near `2.5` |

`jelly-flex` and `jelly-ripple` remain outside the static script; the
[motion companion](2026-09-11-jelly-motion-sweep.md) covers them (`material-36e968`). None of these
tables is evidence of ray bending: `material-343f27`.

## Checks

| Check | Result |
| --- | --- |
| eight sweeps, one row per value, exit 0 | pass |
| window rect and both ROIs identical across all eight runs and Part 1 | pass, `704x640+40+40`, `27x320+35+200`, `234x213+275+254` |
| `BLOCK=blur` configs carry `backdrop-blur true` and no `blur { off }` | pass, asserted by the script on every write |
| inherited noise and saturation reproduce the written pair's shape | pass |
| `niri.log` free of material errors, fallbacks, panics | pass, all eight |
| second run reproduces a table | pass, `roughness` re-run: `sweep.tsv` byte-identical, all fourteen ROI crops `AE 0` |
