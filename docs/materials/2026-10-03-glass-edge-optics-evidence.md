# Glass edge optics evidence (material-be611b)

Evidence for the glass-edge optics plan (`docs/plans/2026-10-02-glass-edge-optics.md`). Tasks 6 to 9 append their steps below.

## Step 1: the bevel as a height field (Task 4)

- Commit under test: `65a99387` (feat(material): render the glass bevel as a height field)
- Renderer: `NVIDIA GeForce RTX 3070/PCIe/SSE2` (from the dumps' `.json`; the hardware driver, not llvmpipe)
- Evidence root: `/mnt/ssd3/niri-material/material-be611b/20261003T061451` (`$EV` below)
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
$ python3 glass-edge-compare.py identical $EV/base $EV/step1 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque --region outside
stock-off outside: 0 differing pixels
stock-on outside: 0 differing pixels
stock-iridescence outside: 0 differing pixels
live-off outside: 0 differing pixels
live-on outside: 0 differing pixels
live-iridescence outside: 0 differing pixels
live-translucent outside: 0 differing pixels
live-opaque outside: 0 differing pixels
[exit 0]
$ python3 glass-edge-compare.py identical $EV/base $EV/step1 live-opaque --region window
live-opaque window: 0 differing pixels
[exit 0]
$ python3 glass-edge-compare.py predict-face $EV/base $EV/step1 stock-off --ior 1.5
stock-off face, f0 0.04000: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-face $EV/base $EV/step1 live-off --ior 1.28
live-off face, f0 0.01508: worst 0 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-within $EV/base $EV/step1 stock-off stock-on --ior 1.5
stock-on face with ring and aurora, f0 0.04000: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py predict-within $EV/base $EV/step1 live-off live-on --ior 1.28
live-on face with ring and aurora, f0 0.01508: worst 1 code values from the rounded prediction
[exit 0]
$ python3 glass-edge-compare.py report $EV/base $EV/step1 stock-off --region bevel
stock-off bevel: rmse 1.360, max 2 code values
[exit 0]
$ python3 glass-edge-compare.py report $EV/base $EV/step1 live-off --region bevel
live-off bevel: rmse 10.376, max 29 code values
[exit 0]
$ python3 glass-edge-compare.py report $EV/base $EV/step1 stock-on --region bevel
stock-on bevel: rmse 1.046, max 2 code values
[exit 0]
$ python3 glass-edge-compare.py report $EV/base $EV/step1 live-on --region bevel
live-on bevel: rmse 15.768, max 29 code values
[exit 0]
$ python3 glass-edge-compare.py report $EV/base $EV/step1 live-translucent --region face
live-translucent face: rmse 0.000, max 0 code values
[exit 0]
```

### Ring-look comparison

All fourteen ring-look PNGs differ from the base (13 to 67 thousand pixels each, in the 2560x1440 sheet), confined to the slab edges (bounding boxes at the two windows' outlines, nothing elsewhere). The largest per-channel change is 12 code values (large-beam-150ms), otherwise 7 to 9. By eye the large and small windows look the same at sheet scale: the same dark glass, the same rim and the same beam highlight on the top edge. The visible change is a slightly different bevel shading along the outer rim, not a change of the ring's placement or beam. These cases run at bevel-profile 1, where the straight-side normal is unchanged exactly (`bevel::tests` proves it), so the expected mechanism is the ray-path attenuation (the bevel now thins toward the silhouette) and the `(1 - F)` scaling of the transmitted light, not profile-shaped normals. No measurement here separates the two.

### The one_core relaxation (material-d37c1a)

After the height-field bevel, `ring_cap_keeps_one_core` failed on the binding case, right and bottom edges: the outward walk from the core met 59, 58, 59 on the shoulder and reported a second maximum. The cause is expected to be the intended spill brightening where the glass thins (spec 3.2 and 3.3: spill and aurora brighten where the glass thins), with quantization possibly contributing; it was not isolated by measurement. The walk starts at low = core, so a core+1 sample just off the face is walked through, which is the core spreading onto the chamfer. `one_core` therefore ends the walk only on a rise of more than one code value over the lowest so far. It still rejects a real second maximum: a rise of 2 past the minimum ends the walk and is reported when it is at least half the core (`one_core_stops_at_a_two_count_rise`), and the unlit, misplaced and plateau cases are unchanged.

### Reading

- Outside the slab nothing changed in any of the eight cases, and no opaque window pixel changed: the change is confined to the bevel.
- Face predictions hold. Stock face (f0 0.040) is within 1 code value of the rounded `(1 - F)` prediction, which shows `(1 - F)` is applied. The live face is dark (about code 10), where f0 = 1.5 % is under one code value, so the live prediction (worst 0) cannot detect a missing `(1 - F)`; only the stock case can. The predicted stock face shift is the centre pixel [139,115,92] to [136,113,91], so the 1-code tolerance is tight enough to see it.
- The within predictions (face with ring and aurora) hold at worst 1 code value in both stock and live, so the ring's face attenuation is unchanged. The live within prediction is as insensitive as the live face check (same f0, same dark face); only the stock-on result carries weight.
- Bevel deltas (reported, not asserted): stock bevel rmse 1.360 (max 2) off and 1.046 (max 2) on; live bevel rmse 10.376 (max 29) off and 15.768 (max 29) on. The stock bevel, over a light backdrop at ior 1.5, moves little; the live bevel, over the fully clear client with a bright backdrop refracted in, moves a lot. At bevel-profile 1 the straight-side normal is unchanged, so the expected mechanism is the ray-path attenuation and `(1 - F)` acting on the bright refracted backdrop, not the profile bending the normals; this is the expected mechanism, not a measured attribution.
- Review Focus 4 (translucent client over the face): the live-translucent face delta is rmse 0.000, max 0 code values. The predicted darkening, (1 - win.a) * f0 * T, is about 0.4 * 0.015 * 15 = 0.09 code values at ior 1.28, below one code value, so it rounds to 0. A build that never applied (1 - F) under the translucent window would also measure 0, so this case neither confirms nor refutes Review Focus 4. "Expected" here means only that the measurement does not contradict the plan's "within a code value".

## Step 2

The `Surface` specular hook (material-d63184) is a signature refactor: renders are decoded-identical to step 1 across iridescence.

```
$ python3 glass-edge-compare.py identical $EV/step1 $EV/step2 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque
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

## Step 3

The `reflection` optic (material-02b42a). Dumps are in `$EV/step3` (before: `step3-before`, taken at the Task 6 head).

### Motion test and its negative control

`the_reflection_follows_the_perturbed_direction_in_motion` renders one frozen mid-resize instant four times (reflection 0.6 and 0, perturbed and flat) over a patterned checker backdrop and counts channels whose reflection isolate differs by more than 0.02 in linear light between the two perturbation states.

- Real shader: 364 channels changed, 0 clipped.
- Mutated shader (`vec2 bent = s.acrossDir;`, the perturbation removed; substitution verified, restored afterwards): 0 channels changed, 0 clipped; the test fails with "the reflection ignores the perturbation: 0 channels changed (0 clipped)".
- Calibration: the real count (364) is below the 2500 that keeps 500, so the second rule applies: 364 is at least 25 times max(0, 4) = 100, and the threshold is the geometric mean of the real count and max(mutated, 4), sqrt(364 * 4) = 38. The mutated count is 0, whose literal geometric mean would be 0 (a test that cannot fail), so the same floor of 4 that the rule's condition uses is applied to the mean. `REFLECTION_MOTION_MIN = 38`. With 38 the real test passes (364) and the mutated one fails (0).

### Neutrality and intended change

```
$ python3 glass-edge-compare.py identical step3-before step3 stock-off stock-on stock-iridescence live-off live-on live-iridescence live-translucent live-opaque
stock-off all: 0 differing pixels
stock-on all: 0 differing pixels
stock-iridescence all: 0 differing pixels
live-off all: 0 differing pixels
live-on all: 0 differing pixels
live-iridescence all: 0 differing pixels
live-translucent all: 0 differing pixels
live-opaque all: 0 differing pixels
[exit 0]
$ pair step3 stock-off stock-reflection-0 --region all --expect same
stock-off vs stock-reflection-0 all: 0 differing pixels (expect same)
[exit 0]
$ pair step3 stock-off stock-reflection --region face --expect same
stock-off vs stock-reflection face: 0 differing pixels (expect same)
[exit 0]
$ pair step3 stock-off stock-reflection --region outside --expect same
stock-off vs stock-reflection outside: 0 differing pixels (expect same)
[exit 0]
$ pair step3 stock-off stock-reflection --region bevel --expect differ
stock-off vs stock-reflection bevel: 23864 differing pixels (expect differ)
[exit 0]
$ pair step3 live-off live-reflection-0 --region all --expect same
live-off vs live-reflection-0 all: 0 differing pixels (expect same)
[exit 0]
$ pair step3 live-off live-reflection --region face --expect same
live-off vs live-reflection face: 0 differing pixels (expect same)
[exit 0]
$ pair step3 live-off live-reflection --region outside --expect same
live-off vs live-reflection outside: 0 differing pixels (expect same)
[exit 0]
$ pair step3 live-off live-reflection --region bevel --expect differ
live-off vs live-reflection bevel: 24279 differing pixels (expect differ)
[exit 0]
```

Every existing case is decoded-identical to step 2; `reflection 0` renders exactly as no node; `reflection 0.6` leaves the face and everything outside the slab untouched and changes the bevel (stock 23864 pixels, live 24279).

By eye (`live-reflection.png`): the dark live bevel takes a faint backdrop-coloured lift that is largest at the silhouette. Row 200, left edge, live-off to live-reflection, (r,g,b): x=306 (47,43,39) to (49,44,40); x=312 (37,35,34) to (40,38,35); x=318 (33,32,32) to (36,34,33). The lift is small because Fresnel at ior 1.28 is small; it is warm, the backdrop's hue.

## Step 4

The `edge-highlight` optic (material-5e64ef). Dumps are in `$EV/step4` (before: `step4-before`, taken at the Task 7 head `a39d0088`). Gate commands exit 0 as shown.

```
$ identical step4-before step4 ...
stock-off all: 0 differing pixels
stock-on all: 0 differing pixels
stock-iridescence all: 0 differing pixels
live-off all: 0 differing pixels
live-on all: 0 differing pixels
live-iridescence all: 0 differing pixels
live-translucent all: 0 differing pixels
live-opaque all: 0 differing pixels
stock-reflection-0 all: 0 differing pixels
stock-reflection all: 0 differing pixels
live-reflection-0 all: 0 differing pixels
live-reflection all: 0 differing pixels
[exit 0]
$ pair step4 stock-off stock-highlight-0 --region all --expect same
stock-off vs stock-highlight-0 all: 0 differing pixels (expect same)
[exit 0]
$ pair step4 stock-k2 stock-highlight --region face --expect same
stock-k2 vs stock-highlight face: 0 differing pixels (expect same)
[exit 0]
$ pair step4 stock-k2 stock-highlight --region outside --expect same
stock-k2 vs stock-highlight outside: 0 differing pixels (expect same)
[exit 0]
$ pair step4 stock-k2 stock-highlight --region bevel --expect differ
stock-k2 vs stock-highlight bevel: 990 differing pixels (expect differ)
[exit 0]
$ pair step4 live-off live-highlight-0 --region all --expect same
live-off vs live-highlight-0 all: 0 differing pixels (expect same)
[exit 0]
$ pair step4 live-k2 live-highlight --region face --expect same
live-k2 vs live-highlight face: 0 differing pixels (expect same)
[exit 0]
$ pair step4 live-k2 live-highlight --region outside --expect same
live-k2 vs live-highlight outside: 0 differing pixels (expect same)
[exit 0]
$ pair step4 live-k2 live-highlight --region bevel --expect differ
live-k2 vs live-highlight bevel: 24997 differing pixels (expect differ)
[exit 0]
```

Every existing case is decoded-identical to step 3 at the default edge-highlight 0; `edge-highlight 0` renders exactly as no node; with `bevel-profile 2`, `edge-highlight 0.5` leaves the face and everything outside the slab untouched and lights the bevel (stock 990 pixels, live 24997; the stock bevel is thin).

By eye (`live-highlight.png` against `live-k2.png`, (r,g,b)): the lobe sits on the top-left bevel. Row 200, left edge: x=312 (31,30,30) to (45,44,44). Top edge, x=640: y=34 (31,30,30) to (50,50,49). The bottom-right bevel barely moves (x=950, y=200: (14,12,11) to (15,13,12)). Summed channel difference by quadrant of the changed region: top-left 172904, top-right 116894, bottom-left 78787, bottom-right 29853.

## Step 5: contact sheet

The owner's visual judgement (spec §6), from `docs/materials/scripts/glass-edge-sheet.sh` on the headless Weston lane. It was run from a TTY with the desktop stopped, on 2026-10-03 between 22:22 and 22:43 EDT:

```
OUT=$EV/sheet-4 CAPTURE_TASK=material-124f1f \
  SHEET_WALL=<wali current: backgrounds/3440/PXL_20220530_182540069.jpg> \
  NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material docs/materials/scripts/glass-edge-sheet.sh
...
sheet: $EV/sheet-4/glass-edge-sheet.png
PASS: artifacts in $EV/sheet-4
```

- Artifacts: `$EV/sheet-4` (`glass-edge-sheet.png`, the 76 cells and their full captures, per-cell KDL, `capture.json`, `cooldown.txt`).
- Identity (`capture.json`): source `ab46ca55` on `glass-edges`. The dirty diff touches only docs and this script's cool-down, so the binaries are that commit's. `niri` sha256 `0c519635…`, `niri-tracy` `51a9cf77…`, wallpaper `PXL_20220530_182540069.jpg` sha256 `4353cddf…` (wali's current wallpaper at run time), output 1280x720 at scale 1, VRR off. The renderer was the NVIDIA RTX 3070 on driver 615.71.09. This is the shaders' first compile on the real driver, and it compiled.
- Quietness: the preflight was quiet (cpu 1.4 %, load1 0.83, GPU 0 %, P8), and all 77 settles were settled.
- Attempts before it, each with a `run:` note on material-124f1f:
  - Pilot `sheet-pilot-2`: passed. The crop holds the corner, the bevel and the wallpaper.
  - `sheet`: preflight refused on load1, the tail of the pilot's build.
  - `sheet-2` and `sheet-3`: a settle was refused on a P5 sample after 7 and 16 cells. The previous cell's GPU tail landed in the next 10 s window; an idle 60 s probe held P8 throughout.
  - The script then waited for 3 s of P8 before each cell (bounded at 60 s); since the materials-26.04 merge the shared `glass-optic-smoke-lib.sh` runs that wait before every launch. The waits were 3 to 5 s, and the settle gate is unchanged. `dropbox.service`, which crash-loops in a TTY, and `wali-rotate.timer` were stopped for the window and restored afterwards.

What the sheet shows, for the owner to judge:
- `edge-highlight 0.5` at roughness 1 lights an even grey band around the whole bevel on every look. At roughness 0 the lobe is a point at the corner, barely visible at this crop.
- `reflection 0.6` against 0 is hard to tell apart at this size on the dark focused and inactive looks.
- The thin, weak look at roughness 0 is nearly invisible over this wallpaper.
- `bevel-profile` mainly changes the highlight band's shape: flat at k 1, rounded at k 2 and 4.
