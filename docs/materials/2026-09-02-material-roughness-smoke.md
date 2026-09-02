# Material roughness: nested GPU smoke evidence

**Result:** final focused acceptance passed 2026-09-02. The initial nested GPU
smoke used source `b220152d` and binary SHA-256
`c6635d5865a204a80cde839eb3d4bd7bf587c7e819ad3b4af4e907271885becd`;
the final Tracy run used the same renderer implementation from a
`profile-with-tracy` binary identifying as `v26.04-160-g919a60db`, and the
calibrated overview capture used `v26.04-161-geadba69a`.

The first trace attempt found a tool-protocol mismatch: the tree's
`tracy-client-sys 0.28.0` embeds Tracy 0.13.1 (protocol 76), while the installed
capture/export tools are 0.14.0. Matching 0.13.1 capture and CSV tools were
then built directly from Tracy's server sources with its pinned Capstone 6
headers and the installed zstd library. No incompatible trace was accepted.

## Retained artifacts

Raw inputs, captures, the Weston log, and apply-ready Prism patches are outside
the product tree under:

```text
$NIRI_MATERIAL_WORK_ROOT/material-roughness-b220152d
```

The run used a 1280 x 720 headless Weston GL host on Mesa llvmpipe, OpenGL ES
3.2. The nested compositor identified the tested binary as
`v26.04-159-gb220152d`.

| Artifact | SHA-256 |
| --- | --- |
| roughness 0 capture | `ad60ebde291d39bf6a31d689d2cb9e621b9404712bc6266740c51ebb890ffa67` |
| roughness 0.08 capture | `38f3596aaaf98bab269526e4767ed31ef6a4ce3a78aa374ca75fe9e88e85e109` |
| roughness 0.5 capture | `ade56abc18fd8792a5b8f27b0f07d0839e4a9af0da9174b536225367d1ea59e6` |
| roughness 0.5 overview | `b791697cd363d5db2ea101f3a95a7be08e5cfa2a09873cde05413c9e2118d1ca` |
| roughness 0.08, anisotropic blur 1, chromatic aberration 1 | `a3a708ecfe0664a4c6683f28cffafe47a4d8f02283da19112baa7d4a3369f3b1` |
| roughness 0 KDL | `b33bcdc79c286135a4a15e0e061e4a414ef93a2b6f8503d87444f6b91acdaf43` |
| roughness 0.08 KDL | `ba743dca7dd6c12bcedbac37b51bf4ea85285fc8b175790494e9817fcd7c270e` |
| roughness 0.5 KDL | `728b31e792110d79386888a99997dcab10d8a3ab86ad27243ac9a95fa56893c8` |
| worst-case KDL | `6032d0b58ade5c4aa85d23a7d930debadb5b6f200949993a5d8c4e917c505269` |
| Prism plan patch | `cdb20824dce06046085458cd8d4ec0e5480a02693cfd9486c4c58548486cf784` |
| Prism implementation patch | `a522878caad80b37307f19bb7ca30f43be3920cce94ef551367ce82c9523ef32` |
| roughness 0 GPU trace | `2559901a945a65cdcd0d37dbf5af5f41b07d37c9ee3743633e2d8823026405a9` |
| roughness 0.08 idle trace | `a1ab04cfcf687365841b73333440f2d7986c8f405d05506a374208a0c95806ec` |
| roughness 0.08 damage trace | `22677c2c07c66999be67f5f83b439885f9d159dee1c1cd56a64e087a05f2a94d` |
| worst-case GPU trace | `bc07ef33e562013845cf70ce2809b9704d5be219bd08fbef9dde425ee7e2a89c` |
| calibrated roughness 0.5 settled capture | `c139319841a98e935c99481cfbf5e881835f3db9aeba33900f5bc34429843197` |
| calibrated roughness 0.5 overview capture | `e66c9f59e1e76dce668094cef81e72a1895b73afb760d793a7668e6d3b4d80c9` |
| Tracy 0.13.1 capture tool | `7b95c9c388b6b689cd87da490b564dd086fa8c9dfd6489424b2401d6ef0c5b0c` |
| Tracy 0.13.1 CSV tool | `472e08726cc62ab66ed38f75bd4cbb351f2160c1f742a709a9d5be57c68fb463` |

## Observations

- The actual GLES material program compiled and rendered with five samplers.
- Roughness 0.08 changed the settled capture relative to roughness 0 with RMSE
  `72.5368` (`0.00110684` normalized).
- Roughness 0.5 increased that difference to RMSE `611.318` (`0.00932812`
  normalized), visibly removing checkerboard detail through the glass.
- The overview capture retained the softened treatment and passed the numeric
  transition-width gate described below.
- The worst-case optics fixture rendered with roughness, anisotropic blur, and
  chromatic aberration all active.
- The observed nested-compositor stream contained no material-prefilter or
  plain-window-fallback warning.
- The owned Weston, nested niri, swaybg, and Kitty processes were stopped; no
  evidence-run socket remained listening.

## Tracy acceptance

The 65-second idle capture retained the client's earlier history, so the gate
uses trace timestamps rather than total file contents. Its last
`EffectBuffer::prepare_prefilter`/`Prefilter::downsample` pair ended at
`144.566` seconds and the trace ended at `224.8` seconds: more than 80 seconds
with zero pyramid preparation, allocation opportunity, or downsampling.

A controlled wallpaper replacement produced two post-warm-up prepare/downsample
pairs, one for each selected source, at `30.047` and `30.626` seconds. No later
pair appeared before the trace ended at `40.88` seconds, proving reuse after
the damage regeneration.

Matched steady-state GPU `draw shader` samples use the first 14 events between
20 and 28 seconds in each trace:

| fixture | samples | median | delta from roughness 0 |
| --- | ---: | ---: | ---: |
| roughness 0 | 14 | 4.694 ms | baseline |
| roughness 0.08 | 14 | 5.103 ms | +8.7% |
| roughness 0.08, anisotropic blur 1, chromatic aberration 1 | 14 | 28.529 ms | +507.8% |

The maximal-optics fixture intentionally exercises the documented 96-fetch
case on llvmpipe; it is characterization, not a shipping threshold. The native
roughness-zero path shows no unexplained default-path regression.

## Calibrated overview acceptance

The final fixture used 80-pixel checker squares. The overview workspace measured
640 x 360 pixels for a 1280 x 720 source, recording an exact zoom of `0.5` and
a tolerance of `1 / zoom = 2` source pixels. Across 49 matched interior edges:

| measurement | result |
| --- | ---: |
| settled 10%-90% median | 30.404 source px |
| overview 10%-90% median | 15.325 screen px = 30.650 source px |
| median paired difference | 0.116 source px |
| worst paired difference | 0.491 source px |

Both differences are below the 2-source-pixel tolerance, and the on-screen
transition width scales with the recorded overview zoom.
