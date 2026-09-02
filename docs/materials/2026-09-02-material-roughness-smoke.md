# Material roughness: nested GPU smoke evidence

**Result:** the focused nested GPU smoke passed 2026-09-02 from source
`b220152d` and binary SHA-256
`c6635d5865a204a80cde839eb3d4bd7bf587c7e819ad3b4af4e907271885becd`.
This is not final acceptance: the 60-second idle/allocation trace, matched GPU
timing, damage-regeneration count, and numeric overview transition-width gate
remain open in `material-ef9eec`.

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

## Observations

- The actual GLES material program compiled and rendered with five samplers.
- Roughness 0.08 changed the settled capture relative to roughness 0 with RMSE
  `72.5368` (`0.00110684` normalized).
- Roughness 0.5 increased that difference to RMSE `611.318` (`0.00932812`
  normalized), visibly removing checkerboard detail through the glass.
- The overview capture retained the softened treatment.
- The worst-case optics fixture rendered with roughness, anisotropic blur, and
  chromatic aberration all active.
- The observed nested-compositor stream contained no material-prefilter or
  plain-window-fallback warning. Full log retention is deferred to the final
  trace run.
- The owned Weston, nested niri, swaybg, and Kitty processes were stopped; no
  evidence-run socket remained listening.

The overview image is characterization only. It does not satisfy the accepted
transition-width criterion because this smoke did not retain a calibrated zoom
measurement or compute the 10%-to-90% edge width.
