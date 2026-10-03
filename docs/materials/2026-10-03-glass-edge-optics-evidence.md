# Glass edge optics evidence (material-be611b)

Evidence for the glass-edge optics plan (`docs/plans/2026-10-02-glass-edge-optics.md`). Tasks 6 to 9 append their steps below.

## Step 1: the bevel as a height field (Task 4)

- Commit under test: `65a99387` (feat(material): render the glass bevel as a height field)
- Renderer: `NVIDIA GeForce RTX 3070/PCIe/SSE2` (from the dumps' `.json`; the hardware driver, not llvmpipe)
- Evidence root: `/mnt/ssd3/niri-material/material-be611b/20261003T061451`
  - before: `base/` (eight cases) and `ring-look-base/`, from Task 3 on the unmodified shader
  - after: `step1/` (eight cases) and `ring-look-step1/`

### Dumps

```
GLASS_EDGE_DUMP=$EV/step1 just test-one -p niri every_case_renders_frozen   # 1 passed
RING_LOOK_DUMP=$EV/ring-look-step1 just test-one -p niri accepted_ring_look  # 1 passed
```

### Checks

Each command exited 0. `C=docs/materials/scripts/glass-edge-compare.py`, `ALL` the eight cases.

```
$ python3 glass-edge-compare.py identical /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque --region outside
stock-off outside: 0 differing pixels
stock-on outside: 0 differing pixels
stock-iridescence outside: 0 differing pixels
live-off outside: 0 differing pixels
live-on outside: 0 differing pixels
live-iridescence outside: 0 differing pixels
live-translucent outside: 0 differing pixels
live-opaque outside: 0 differing pixels
[exit 0]
$ python3 glass-edge-compare.py identical /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-opaque --region window
live-opaque window: 0 differing pixels
[exit 0]
$ python3 glass-edge-compare.py predict-face /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 stock-off --ior 1.5
stock-off face, f0 0.04000: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-face /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-off --ior 1.28
live-off face, f0 0.01508: worst 0 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-within /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 stock-off stock-on --ior 1.5
stock-on face with ring and aurora, f0 0.04000: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-within /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-off live-on --ior 1.28
live-on face with ring and aurora, f0 0.01508: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py report /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 stock-off --region bevel
stock-off bevel: rmse 1.360, max 2 code values
[exit 0]
$ python3 glass-edge-compare.py report /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-off --region bevel
live-off bevel: rmse 10.376, max 29 code values
[exit 0]
$ python3 glass-edge-compare.py report /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 stock-on --region bevel
stock-on bevel: rmse 1.046, max 2 code values
[exit 0]
$ python3 glass-edge-compare.py report /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-on --region bevel
live-on bevel: rmse 15.768, max 29 code values
[exit 0]
$ python3 glass-edge-compare.py report /mnt/ssd3/niri-material/material-be611b/20261003T061451/base /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 live-translucent --region face
live-translucent face: rmse 0.000, max 0 code values
[exit 0]
```

### Ring-look comparison

All fourteen ring-look PNGs differ from the base (13 to 67 thousand pixels each, in the 2560x1440 sheet), confined to the slab edges (bounding boxes at the two windows' outlines, nothing elsewhere). The largest per-channel change is 12 code values (large-beam-150ms), otherwise 7 to 9. By eye the large and small windows look the same at sheet scale: the same dark glass, the same rim and the same beam highlight on the top edge. The visible change is a slightly different bevel shading along the outer rim (profile-shaped normals, ray-path attenuation, Fresnel on the transmitted light), not a change of the ring's placement or beam.

### Reading

- Outside the slab nothing changed in any of the eight cases, and no opaque window pixel changed: the change is confined to the bevel.
- Face predictions hold. Stock face (f0 0.040) is within 1 code value of the rounded `(1 - F)` prediction, which shows `(1 - F)` is applied. The live face is dark (about code 10), where f0 = 1.5 % is under one code value, so the live prediction (worst 0) cannot detect a missing `(1 - F)`; only the stock case can.
- The within predictions (face with ring and aurora) hold at worst 1 code value in both stock and live, so the ring's face attenuation is unchanged.
- Bevel deltas (reported, not asserted): stock bevel rmse 1.360 (max 2) off and 1.046 (max 2) on; live bevel rmse 10.376 (max 29) off and 15.768 (max 29) on. The stock bevel, over a light backdrop at ior 1.5, moves little; the live bevel, over a dark window with a bright backdrop refracted in, moves a lot, as the profile bends the normals.
- Review Focus 4 (translucent client over the face): the live-translucent face delta is rmse 0.000, max 0 code values. The predicted darkening, (1 - win.a) * f0 * T, is about 0.4 * 0.015 * 15 = 0.09 code values at ior 1.28, below one code value, so it rounds to 0. A build that never applied (1 - F) under the translucent window would also measure 0, so this case neither confirms nor refutes Review Focus 4. "Expected" here means only that the measurement does not contradict the plan's "within a code value".

## Step 2

The `Surface` specular hook (material-d63184) is a signature refactor: renders are decoded-identical to step 1 across iridescence.

```
$ python3 glass-edge-compare.py identical /mnt/ssd3/niri-material/material-be611b/20261003T061451/step1 /mnt/ssd3/niri-material/material-be611b/20261003T061451/step2 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque
stock-off all: 0 differing pixels
stock-on all: 0 differing pixels
stock-iridescence all: 0 differing pixels
live-off all: 0 differing pixels
live-on all: 0 differing pixels
live-iridescence all: 0 differing pixels
live-translucent all: 0 differing pixels
live-opaque all: 0 differing pixels
[exit 0]
```
