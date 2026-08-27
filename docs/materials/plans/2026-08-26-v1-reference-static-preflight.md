# Frozen-Reference Static Preflight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the circular v1 static-optics instrument, prove all nine
frozen-reference static rows before rebuilding native, then recapture and
record the complete parity verdict.

**Architecture:** Keep the existing POSIX-shell replay and dependency-free
Node analyzer. Both implementations capture a slab-free live-pane source and
derive the optical rectangle from niri IPC; the analyzer measures the bevel,
flat face, and inner-face warp field with separate geometry-selected oracles.
A reference-only schema-2 preflight gates the old native build, while the
final schema-2 run reuses the same static path and retains the existing
geometry and motion analysis.

**Tech Stack:** POSIX shell, Node.js built-ins and `node:test`, ImageMagick,
`jq`, niri IPC, Weston headless GL, Quickshell/Qt Quick 3D, Cargo.

**Spec:** `docs/materials/2026-08-25-v1-reference-static-preflight-design.md`

**Status:** executed and re-run 2026-08-26. The final instrument is
`niri-experiments` commit `d00f81c32cb8e6ee60881eeee4e65493e39ec6bf`;
the frozen-reference preflight passed 9/9 rows and final evidence commit
`981998bad277064f1331865939dae26731f1e4ed` passed integrity with all 28
implementation rows and all 14 combined parameters in two independent
captures. The parity gate is complete; physical DRM remains before v1
acceptance.

**Post-execution amendment:** `niri-experiments` commit
`5d3dfd26c4d4a4ef2c464c1d649dba533ccb27c5` adds an exact reference-geometry
partial schema and 3/3 preflight. Geometry now derives both implementations to
80-pixel gaps, while static optics and motion stay at 24. Replay correction
`705718d410232badb9ddd7e27d2da2e40ca3eb24` captures motion first and makes
the geometry transition one-way because the frozen native surface does not
reproduce its initial optical pixels after an 80-to-24 resize round trip.

**Final motion amendment:** `niri-experiments` evidence commit
`981998bad277064f1331865939dae26731f1e4ed` uses a stiffness-100 spring, 12
requested offsets through 1100 ms, pane-progress alignment, directional bevel
shear, translation-removed face ripple, and paired-delta noise. It also loads
native variants exactly once and updates the frozen reference's watched JSON
in place. Two independent captures pass 28/28 rows and 14/14 parameters.

## Global Constraints

- Implement evidence tooling in a fresh `niri-experiments` worktree and branch
  from exact `results/slice3` commit
  `7729dfc151eae41c946d2495ef67010c1cd20635`.
- Keep the frozen reference at
  `70f26af4324635bb827d07f0eaf1fc227acb43ea`; do not edit it.
- Require `NIRI_MATERIAL_WORK_ROOT` and `PREFLIGHT_NIRI`; do not fall back to
  `/tmp` or a repository-contained path.
- Do not build the old native surface until the real reference preflight exits
  `0` with nine passing rows.
- The preflight binary is only a feasibility host. The rebuilt old-surface
  binary hosts both phases of final evidence and supplies the final binary
  pins.
- Final schema is `2`; native and reference have identical implementation key
  sets. Delete the phase-equivalent reference source path instead of keeping a
  compatibility branch.
- Geometry/support/saturation defects exit `2`; valid wrong-direction evidence
  exits `1`; all requested rows passing exits `0`.
- Commit only fixtures, distilled evidence, hashes, and documentation. Passing
  preflight and final raw captures are deleted at their specified lifecycle
  points.
- No new dependency, test framework, reusable ROI API, resumable capture
  framework, frozen-client edit, or direct native-to-reference pixel threshold.
- Conventional commits only, with named staging paths and no AI-attribution
  trailers.
- Repository code and documentation contain no absolute host paths.

## File Structure

| File | Responsibility | Task |
| --- | --- | --- |
| `niri-experiments/fixtures/diagnostic-grid.png` | Committed 40-by-40 dense grid with deterministic asymmetric watermark | 1 |
| `niri-experiments/fixtures/diagnostic-grid.svg` | Retained reproducible source for the committed PNG | 1 |
| `niri-experiments/fixtures/v1-parity-analyze.mjs` | Schema-2 validation, ROI checks, signed bevel profiles, flat-face residual, local warp field, and final/static/geometry CLI modes | 1, amendment |
| `niri-experiments/fixtures/v1-parity-analyze.test.mjs` | Physics-shaped synthetic controls and exit-contract tests | 1 |
| `niri-experiments/fixtures/v1-parity-replay.sh` | Required work root, phase-specific scenes, exact partial manifests, preflight lifecycles, and final manifest | 2, amendment |
| `niri-experiments/docs/results/2026-08-24-v1-parity.md` | Corrected branch/pins, preflight result, full matrix, margins, and verdict | 3, 4 |
| `docs/materials/2026-08-25-v1-reference-static-preflight-design.md` | Implemented status and evidence commit | 5 |
| `docs/materials/2026-08-24-v1-parity-design.md` | Replacement evidence verdict | 5 |
| `docs/materials/2026-08-22-v1-design.md` | Current v1 gate status | 5 |
| `docs/materials/README.md` | User-facing progress and document links | 5 |
| `docs/materials/plans/2026-08-26-v1-reference-static-preflight.md` | Execution record and final pins | Plan commit, 5 |

---

### Task 1: Replace the static analyzer and synthetic controls

Create the evidence worktree, commit the final diagnostic asset, and make the
analyzer independently validate the accepted static contract.

**Files:**
- Create worktree: `niri-experiments/.worktrees/reference-static-preflight`
- Verify: `niri-experiments/fixtures/diagnostic-grid.svg`
- Create: `niri-experiments/fixtures/diagnostic-grid.png`
- Modify: `niri-experiments/fixtures/v1-parity-analyze.mjs`
- Modify: `niri-experiments/fixtures/v1-parity-analyze.test.mjs`

**Interfaces:**
- Consumes: schema-2 `manifest.json` or
  `reference-static-manifest.json`, 1280-by-720 raw RGB duplicate pairs, and
  `{x,y,width,height}` `static_roi`.
- Produces: `staticMasks`, `validateStaticRoiResponse`, `signedBevelShift`,
  `flatFaceVariation`, and `localWarpField` exports for the self-check;
  `evaluateManifest` for 28 final rows; `evaluateReferenceStaticManifest` for
  nine reference rows; the amendment adds the exact geometry-only form
  `v1-parity-analyze.mjs --reference-geometry ARTIFACT_DIR` beside the final
  and `--reference-static` modes.

`signedBevelShift` returns
`Array<{side:"top"|"left",shift:number,score:number}>` or throws on a boundary
winner. `localWarpField` returns
`{vectors:Array<{x:number,y:number,dx:number,dy:number}>,possibleCandidates:number,acceptedCandidates:number,possibleNeighborPairs:number,neighborPairs:number}`
or throws on ties, saturation, or insufficient support. Update
`fieldFrequency` and the internal variance metric to consume that exact field
shape; do not retain the old static-field shape as a compatibility path.

- [x] **Step 1: Create the isolated evidence worktree**

```bash
git -C "$NIRI_EXPERIMENTS_ROOT" worktree add \
  "$NIRI_EXPERIMENTS_ROOT/.worktrees/reference-static-preflight" \
  -b results/reference-static-preflight 7729dfc151eae41c946d2495ef67010c1cd20635
git -C "$NIRI_EXPERIMENTS_ROOT/.worktrees/reference-static-preflight" status --short
```

Expected: an empty status on `results/reference-static-preflight`.

- [x] **Step 2: Materialize the accepted diagnostic tile**

Use a host-local temporary script with the spike's fixed xorshift seed
`0x6d2b79f5` to write a 40-by-40 grayscale PPM made of 2-by-2 cells in the
range 32–223. Rasterize the existing SVG to 40-by-40, blend watermark and grid
with ImageMagick `compose:args=15,85`, exclude wall-clock PNG chunks, and
commit the resulting reproducible PNG bytes.

```js
// $NIRI_MATERIAL_WORK_ROOT/diagnostic-watermark.mjs
import { writeFileSync } from "node:fs";
let state = 0x6d2b79f5;
const random = () => {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
};
const width = 40, height = 40, cell = 2;
const rgb = Buffer.alloc(width * height * 3);
for (let y = 0; y < height; y += cell) for (let x = 0; x < width; x += cell) {
  const value = 32 + random() % 192;
  for (let by = 0; by < cell; by++) for (let bx = 0; bx < cell; bx++)
    rgb.fill(value, ((y + by) * width + x + bx) * 3,
             ((y + by) * width + x + bx) * 3 + 3);
}
writeFileSync(process.argv[2], Buffer.concat([
  Buffer.from(`P6\n${width} ${height}\n255\n`), rgb,
]));
```

```bash
node "$NIRI_MATERIAL_WORK_ROOT/diagnostic-watermark.mjs" \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-watermark.ppm"
magick -background none MSVG:fixtures/diagnostic-grid.svg -resize 40x40! \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-base.png"
magick "$NIRI_MATERIAL_WORK_ROOT/diagnostic-base.png" \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-watermark.ppm" \
  -define compose:args=15,85 -compose blend -composite \
  -define png:exclude-chunk=date,time \
  PNG24:fixtures/diagnostic-grid.png
sleep 1
magick "$NIRI_MATERIAL_WORK_ROOT/diagnostic-base.png" \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-watermark.ppm" \
  -define compose:args=15,85 -compose blend -composite \
  -define png:exclude-chunk=date,time \
  PNG24:"$NIRI_MATERIAL_WORK_ROOT/diagnostic-grid-repeat.png"
cmp fixtures/diagnostic-grid.png \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-grid-repeat.png"
magick fixtures/diagnostic-grid.png \
  -format '%wx%h\n%[pixel:p{20,20}]\n%[fx:standard_deviation]\n' info:
sha256sum fixtures/diagnostic-grid.png \
  "$NIRI_MATERIAL_WORK_ROOT/diagnostic-grid-repeat.png"
```

Expected: `40x40`, sample `srgb(44,54,59)`, standard deviation `0.0355911`,
and SHA-256
`6fafae8c6cf3e3815346128ffdb402d5c730ae3a8749cb060013821ba0fe0316`
for both encodes.
The analyzer obtains its expected hash from the committed adjacent asset with
`createHash("sha256")`; the results document records the asset SHA as the
human audit anchor.

- [x] **Step 3: Write failing geometry and profile tests**

Add these exports to the test import, then add the exact synthetic helpers and
focused controls:

```js
const patternedFrame = (width, height) => {
  const frame = new Uint8Array(width * height * 3);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const value = (x * 17 + y * 31 + x * y * 3) & 255;
    frame.fill(value, (y * width + x) * 3, (y * width + x) * 3 + 3);
  }
  return frame;
};
const shiftBevel = (baseline, roi, width, shift) => {
  const variant = baseline.slice();
  for (let normal = 3; normal <= 11; normal++) {
    for (let x = roi.x + 28; x < roi.x + roi.width - 28; x++) {
      const to = ((roi.y + normal) * width + x) * 3;
      const from = ((roi.y + normal + shift) * width + x) * 3;
      variant.set(baseline.subarray(from, from + 3), to);
    }
    for (let y = roi.y + 28; y < roi.y + roi.height - 28; y++) {
      const to = (y * width + roi.x + normal) * 3;
      const from = (y * width + roi.x + normal + shift) * 3;
      variant.set(baseline.subarray(from, from + 3), to);
    }
  }
  return variant;
};
const bevelBandFixture = () => {
  const width = 160, height = 160;
  const roi = { x: 20, y: 20, width: 120, height: 120 };
  const baseline = patternedFrame(width, height);
  return { baseline, inward: shiftBevel(baseline, roi, width, 2),
    outward: shiftBevel(baseline, roi, width, -2), roi, width, height };
};
const linearToSrgb8 = value => Math.round(255 * (value <= 0.0031308
  ? value * 12.92 : 1.055 * value ** (1 / 2.4) - 0.055));
const flatFaceFixture = () => {
  const width = 64, height = 64;
  const baseline = new Uint8Array(width * height * 3);
  const latent = new Float64Array(baseline.length);
  for (let pixel = 0; pixel < width * height; pixel++) {
    for (let channel = 0; channel < 3; channel++) {
      const index = pixel * 3 + channel;
      const code = 32 + channel * 16 + pixel * (channel * 2 + 1) % 128;
      latent[index] = srgb8ToLinear(code + (pixel % 2 ? 0.4 : -0.4));
      baseline[index] = linearToSrgb8(latent[index]);
    }
  }
  return { baseline, latent,
    mask: rectMask(width, height, 16, 16, 47, 47) };
};
const addLinearConstant = (baseline, latent, mask, checker = 0) => {
  const output = baseline.slice();
  for (let pixel = 0; pixel < mask.length; pixel++) if (mask[pixel]) {
    for (let channel = 0; channel < 3; channel++)
      output[pixel * 3 + channel] = linearToSrgb8(
        latent[pixel * 3 + channel] + 0.010666666666666666
          + (pixel % 2 ? checker : -checker));
  }
  return output;
};
```

```js
test("static ROI derives bevel and 32-pixel face masks", () => {
  const masks = staticMasks({ x: 10, y: 10, width: 100, height: 100 }, 120, 120);
  assert.equal(masks.bevel[(15 * 120) + 50], 1);
  assert.equal(masks.face[(42 * 120) + 42], 1);
  assert.equal(masks.face[(41 * 120) + 42], 0);
  assert.throws(() => staticMasks({ x: 0, y: 0, width: 64, height: 64 }, 120, 120),
                /32-pixel inset is empty/);
});

test("ROI response cross-check uses directional shadow reaches", () => {
  const roi = { x: 24, y: 24, width: 490, height: 684 };
  const valid = rectMask(1280, 720, 0, 8, 549, 719);
  assert.doesNotThrow(() => validateStaticRoiResponse(
    roi, [valid, valid.slice()], 1280, 720));
  const swallowed = rectMask(1280, 720, 0, 0, 560, 719);
  assert.throws(() => validateStaticRoiResponse(
    roi, [swallowed, swallowed.slice()], 1280, 720), /ROI edge response/);
});

test("signed bevel shift rejects reversal and search saturation", () => {
  const { baseline, inward, outward, roi, width, height } = bevelBandFixture();
  assert.deepEqual(signedBevelShift(baseline, inward, roi, width, height, 4)
    .map(value => Math.round(value.shift)), [2, 2]);
  assert.ok(signedBevelShift(baseline, outward, roi, width, height, 4)
    .every(value => value.shift < 0));
  assert.throws(() => signedBevelShift(baseline,
    shiftBevel(baseline, roi, width, 4), roi, width, height, 4), /search boundary/);
});

test("flat face permits a constant but rejects spatial restructuring", () => {
  const { baseline, latent, mask } = flatFaceFixture();
  assert.equal(flatFaceVariation(
    baseline, addLinearConstant(baseline, latent, mask), mask), 0);
  assert.ok(flatFaceVariation(baseline,
    addLinearConstant(baseline, latent, mask, 0.002), mask) > 0);
});

test("chromatic predicate keeps red at zero and orders separations", () => {
  assert.equal(evaluateOracle("chromatic-aberration", {
    signal: 1, noiseFloor: 0, redWithinNoise: true,
    greenMinusRed: 2, blueMinusGreen: 2, outsideChangedFraction: 0,
  }).verdict, "pass");
});
```

The `bevelBandFixture` changes only a 12-pixel outer band; remove the former
whole-pane `[2,2]` `ior` shift from `writePassingArtifact`.

- [x] **Step 4: Write failing local-field and schema tests**

Build the synthetic source from the same 40-pixel asymmetric tile with these
helpers:

```js
const warpFixture = () => {
  const width = 160, height = 160;
  const tile = patternedFrame(40, 40);
  const source = new Uint8Array(width * height * 3);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const from = ((y % 40) * 40 + x % 40) * 3;
    source.set(tile.subarray(from, from + 3), (y * width + x) * 3);
  }
  return { width, height, source,
    face: rectMask(width, height, 24, 24, 135, 135) };
};
const renderWarp = (source, width, height, mask, shift) => {
  const output = source.slice();
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    if (!mask[y * width + x]) continue;
    const [dx, dy] = shift(x, y);
    const from = ((y + dy) * width + x + dx) * 3;
    output.set(source.subarray(from, from + 3), (y * width + x) * 3);
  }
  return output;
};
const slowWarp = (x, y) => [Math.floor(x / 48) % 2 ? 3 : -3,
                             Math.floor(y / 48) % 2 ? 3 : -3];
const fastWarp = (x, y) => [Math.floor(x / 16) % 2 ? 3 : -3,
                             Math.floor(y / 16) % 2 ? 3 : -3];
```

Assert:

```js
const { width, height, source, face } = warpFixture();
const baseline = localWarpField(source, source, face, width, height);
const slow = localWarpField(source,
  renderWarp(source, width, height, face, slowWarp), face, width, height);
const fast = localWarpField(source,
  renderWarp(source, width, height, face, fastWarp), face, width, height);
assert.equal(baseline.vectors.every(({ dx, dy }) => dx === 0 && dy === 0), true);
assert.ok(fieldFrequency(fast) > fieldFrequency(slow));
assert.throws(() => localWarpField(source,
  renderWarp(source, width, height, face, () => [15, 0]),
  face, width, height), /search boundary/);
```

Update `manifestFixture` to schema `2`, add the same `static_roi` and `source`
to both implementations, and delete `reference.static_source`. Add a distinct
`referenceStaticManifestFixture` with only the exact partial pins/tools and
reference static keys. Test schema `1`, extra native pins, invalid ROI,
duplicate edge disagreement, insufficient field support, and insufficient
neighbor pairs as thrown integrity errors.

- [x] **Step 5: Run the tests to verify the new contract fails**

```bash
node --test fixtures/v1-parity-analyze.test.mjs
```

Expected: FAIL because the new exports, schema, and physics-shaped controls do
not exist yet.

- [x] **Step 6: Implement the minimum static helpers**

Keep the functions in `v1-parity-analyze.mjs`:

```js
export function staticMasks(roi, width, height) {
  const { x, y, width: roiWidth, height: roiHeight } = roi;
  if (![x, y, roiWidth, roiHeight].every(Number.isInteger)
      || roiWidth <= 64 || roiHeight <= 64 || x < 0 || y < 0
      || x + roiWidth > width || y + roiHeight > height)
    throw new RangeError("invalid static ROI or 32-pixel inset is empty");
  const pane = new Uint8Array(width * height);
  const bevel = new Uint8Array(width * height);
  const face = new Uint8Array(width * height);
  const insideRounded = (px, py, inset) => {
    const left = x + inset, top = y + inset;
    const right = x + roiWidth - 1 - inset;
    const bottom = y + roiHeight - 1 - inset;
    const radius = Math.max(0, 28 - inset);
    if (px < left || px > right || py < top || py > bottom) return false;
    if (px >= left + radius && px <= right - radius
        || py >= top + radius && py <= bottom - radius) return true;
    const cx = px < left + radius ? left + radius : right - radius;
    const cy = py < top + radius ? top + radius : bottom - radius;
    return (px - cx) ** 2 + (py - cy) ** 2 <= radius ** 2;
  };
  for (let py = y; py < y + roiHeight; py++) {
    for (let px = x; px < x + roiWidth; px++) {
      const index = py * width + px;
      pane[index] = Number(insideRounded(px, py, 0));
      bevel[index] = Number(pane[index] && !insideRounded(px, py, 12));
      face[index] = Number(px >= x + 32 && px < x + roiWidth - 32
        && py >= y + 32 && py < y + roiHeight - 32);
    }
  }
  return { pane, bevel, face };
}

export function flatFaceVariation(baseline, variant, mask) {
  if (baseline.length !== variant.length || baseline.length !== mask.length * 3)
    throw new RangeError("flat-face inputs have inconsistent dimensions");
  const intervals = Array.from({ length: 3 }, () => [-Infinity, Infinity]);
  let count = 0;
  for (let pixel = 0; pixel < mask.length; pixel++) if (mask[pixel]) {
    count++;
    for (let channel = 0; channel < 3; channel++) {
      const index = pixel * 3 + channel;
      const baselineLow = srgb8ToLinear(Math.max(0, baseline[index] - 0.5));
      const baselineHigh = srgb8ToLinear(Math.min(255, baseline[index] + 0.5));
      const variantLow = srgb8ToLinear(Math.max(0, variant[index] - 0.5));
      const variantHigh = srgb8ToLinear(Math.min(255, variant[index] + 0.5));
      intervals[channel][0] = Math.max(
        intervals[channel][0], variantLow - baselineHigh);
      intervals[channel][1] = Math.min(
        intervals[channel][1], variantHigh - baselineLow);
    }
  }
  if (!count) throw new RangeError("flat-face mask is empty");
  return Math.max(...intervals.map(([lower, upper]) => Math.max(0, lower - upper)));
}
```

Implement `validateStaticRoiResponse` by computing each duplicate's exact
source-to-default changed-pixel bounding box. Require the boxes to match and
require `x0`, `y0`, `x1`, and `y1` to fall within, respectively,
`[roi.x-24,roi.x]`, `[roi.y-16,roi.y]`,
`[roi.x+roi.width-1,roi.x+roi.width-1+36]`, and
`[roi.y+roi.height-1,roi.y+roi.height-1+44]`, clipped to the frame.

Implement `signedBevelShift` by averaging linear-light samples along the
straight top and left edges for every requested normal coordinate, taking the
first derivative, normalizing each candidate derivative by its Euclidean
norm, and minimizing mean squared error over every integer offset from
`-search` through `search`. Refine an interior integer winner with
`winner + 0.5*(leftScore-rightScore)/(leftScore-2*winnerScore+rightScore)`.
Reject a winner equal to either bound. Use bounds `4` for `ior` and `[1,4,5]`
for chromatic RGB.

Implement `localWarpField` with constants `radius=8`, `stride=8`, and
`search=15`. At every face-mask lattice point, compare the rendered 17-by-17
linear-luminance patch with every source patch in the 31-by-31 search area by
zero-mean normalized correlation. Reject tied minima and boundary winners;
store integer `{x,y,dx,dy}` vectors keyed by lattice position. Count all
geometric candidates and all possible right/down pairs before filtering, then
require accepted candidates and retained neighbor pairs each to reach half
their possible counts.

- [x] **Step 7: Implement exact schemas and shared static evaluation**

Change `STATIC_STATES` to include `source`. Validate final implementation keys
as exactly:

```js
["static", "static_roi", "geometry", "motion", "calibration",
 "first_offset_budget_ms"]
```

Add `validateReferenceStaticManifest` with only the partial keys from the spec.
Extract the existing static-row loop into one `evaluateStaticRows` function
used by both manifest evaluators. Precompute ROI masks, edge checks, bevel
profiles, and fields before calling `addRow`, so malformed/saturated evidence
throws to CLI exit `2` rather than becoming a semantic row.

For the rows:

- `ior`: positive top and left shifts plus flat-face residual at duplicate
  noise;
- `thickness`: its explicitly magnitude-only bevel RMS clears noise and its
  inset attenuation increases in the required direction;
- chromatic aberration: red within noise, `green - red > noise`, and
  `blue - green > noise` on both edges;
- distortion: recovered field variance increases;
- distortion scale: `fieldFrequency`, defined as mean neighboring-vector
  delta divided by mean displacement magnitude, increases;
- attenuation, anisotropic blur, and samples: existing signed predicates on
  the new masks.

Make `runCli` accept `--reference-static`; it reads
`reference-static-manifest.json`, writes `reference-static-analysis.json`, and
emits nine rows. Keep final mode's `manifest.json`/`analysis.json` names.

- [x] **Step 8: Run analyzer verification and commit**

```bash
node --test fixtures/v1-parity-analyze.test.mjs
node fixtures/v1-parity-analyze.mjs 2>/dev/null; test "$?" -eq 2
git diff --check
git add fixtures/diagnostic-grid.png fixtures/v1-parity-analyze.mjs \
  fixtures/v1-parity-analyze.test.mjs
git commit -m "feat(parity): analyze geometry-selected static optics"
```

Expected: all tests pass; missing CLI input exits `2`.

---

### Task 2: Add live-pane preflight capture and schema-2 replay

Make the replay produce the analyzer's exact inputs, enforce host-local
storage, and delete passing preflight captures.

**Files:**
- Verify: `niri-experiments/fixtures/diagnostic-grid.svg`
- Modify: `niri-experiments/fixtures/v1-parity-replay.sh`

**Interfaces:**
- Consumes: committed `diagnostic-grid.png`, required
  `NIRI_MATERIAL_WORK_ROOT`, required executable `PREFLIGHT_NIRI`, frozen
  niri-glass root, and an already-running Weston socket.
- Produces: `--preflight-reference NIRI_GLASS_ROOT WESTON_SOCKET`; unchanged
  final capture modes with schema-2 manifests and `static_roi` for both sides.

- [x] **Step 1: Add failing replay self-checks**

Extend the existing trace-based `self_test` to assert:

```sh
unset NIRI_MATERIAL_WORK_ROOT
if prepare_work_root; then exit 1; fi
NIRI_MATERIAL_WORK_ROOT=$self_test_dir/work
mkdir -p "$NIRI_MATERIAL_WORK_ROOT"
requested_artifact_dir=$self_test_dir/in-repository
if prepare_artifact_dir "$requested_artifact_dir"; then exit 1; fi
```

Add a reference-static trace whose required order is:

```text
capture reference static source
copy reference static source to geometry source
probe
capture reference static default
capture reference static ior, thickness, attenuation-color,
  attenuation-distance, chromatic-aberration, distortion,
  distortion-scale, anisotropic-blur, and samples, each with its specified
  return gate
analyze-reference-static
```

Assert the partial manifest exact keys, final schema `2`, identical native and
reference implementation keys, accepted cross-section reuse of the static
source pair as geometry source, per-edge response bounds, and deletion of a
passing preflight directory. Stub analyzer exits `1` and `2` separately and
assert the failing directory remains and no native trace entry appears.

- [x] **Step 2: Run the replay tests to verify they fail**

```bash
fixtures/v1-parity-replay.sh --self-test "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
dash fixtures/v1-parity-replay.sh --self-test "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
```

Expected: FAIL on missing preflight mode, schema, and trace behavior.

- [x] **Step 3: Replace runtime rasterization with the committed tile**

Make `build_live_backdrop` and its mirrored self-check validate
`diagnostic-grid.png` as exactly 40-by-40, sample `p{20,20}` as exactly
`srgb(44,54,59)`, and require standard deviation at least `0.02`. Copy it to
`grid-tile.png`, tile it to 1280-by-720, and hash the committed PNG. Delete
only the replay's SVG/MSVG runtime rasterization; retain
`diagnostic-grid.svg` as the committed PNG's derivation source.

- [x] **Step 4: Enforce the work root and preflight binary**

Add `prepare_work_root` before any compositor process:

```sh
prepare_work_root() {
    [ -n "${NIRI_MATERIAL_WORK_ROOT:-}" ] || return 2
    work_root=$(realpath "$NIRI_MATERIAL_WORK_ROOT") || return 2
    [ -d "$work_root" ] && [ -w "$work_root" ] || return 2
    repository_root=$(realpath "$script_dir/..") || return 2
    case "$work_root/" in "$repository_root/"*) return 2 ;; esac
}
```

For preflight, require `PREFLIGHT_NIRI` to be absolute, regular, executable,
and non-symlinked; hash it and record its `--version`. Require every requested
artifact directory and old Cargo target to be a child of `work_root`.

- [x] **Step 5: Derive and cross-check `static_roi`**

After the probe stabilizes, save `windows`, `workspaces`, and `outputs` IPC
JSON. Select exactly one `v1-parity-probe`. Compute its frame position from
`layout.tile_pos_in_workspace_view + layout.window_offset_in_tile`, then apply
the pinned pane formula:

```text
x = window_x - paneLip + paneShiftX
y = window_y - paneLip + paneShiftY
width  = window_width  + 2*paneLip
height = window_height + 2*paneLip
```

Write the same exact `static_roi` object into the implementation manifest.
The analyzer independently computes the source/default pixel bounding boxes
and applies the 24/16/36/44 directional cross-check; the replay must not
replace its IPC-derived ROI with that shadow-inclusive box.

- [x] **Step 6: Capture reference static optics through the live pane**

Replace the preview static sequence with:

```sh
capture_pair reference static source || return
copy_reference_source_to_geometry || return
spawn_probe || return
capture_pair reference static default || return
```

Capture the same nine variants and return gates as today. Native already has a
slab-free static source; route both modes through the same manifest/static
state shape. Historically, after the final static return to default, this step
captured reference geometry `default` immediately with the existing probe.
The post-execution amendment captures motion first in the 24-pixel scene,
then transitions that same probe once to the geometry-only 80-pixel scene. Do
not capture another geometry source, spawn a second probe, or round-trip the
scene back to 24 pixels. Remove
`show_reference_preview`, the static-anchor helpers, their PID cleanup state
and cleanup test, the separate reference geometry-source capture,
`static_source`, and the phase-equivalent registration metadata completely.

- [x] **Step 7: Write the exact partial and final manifests**

`reference-static-manifest.json` uses schema `2`, one reference implementation,
only `static` and `static_roi`, and the exact partial pins/tools from the spec.
The final manifest uses schema `2`, includes `source` for reference, and gives
both implementations identical key sets. Update shell-side `jq` validators and
phase-state diagnostic hashes at the same time.

- [x] **Step 8: Implement preflight lifecycle**

`--preflight-reference NIRI_GLASS_ROOT WESTON_SOCKET` creates
`$work_root/v1-reference-static.XXXXXX`, captures the partial manifest, and
runs:

```sh
node "$script_dir/v1-parity-analyze.mjs" --reference-static "$artifact_dir"
```

On exit `0`, print the nine-row analysis and delete the entire raw preflight
directory immediately. On exit `1` or `2`, print the retained directory and
return the analyzer status. Existing process cleanup still runs on every path.
No native capture or build function is reachable from this mode.

- [x] **Step 9: Run replay and analyzer verification, then commit**

```bash
fixtures/v1-parity-replay.sh --self-test "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
dash fixtures/v1-parity-replay.sh --self-test "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
node --test fixtures/v1-parity-analyze.test.mjs
git diff --check
git add fixtures/v1-parity-replay.sh
git commit -m "feat(parity): gate native capture on reference static preflight"
```

Expected: both replay suites and the analyzer suite pass. The committed SVG
and PNG both remain: the SVG is derivation input, while the PNG is the sole
runtime diagnostic asset.

---

### Task 3: Run and record the frozen-reference preflight

Prove the nine static reference rows with the committed instrument before any
old-surface build.

**Files:**
- Modify: `niri-experiments/docs/results/2026-08-24-v1-parity.md`

**Interfaces:**
- Consumes: committed Task 1–2 fixture hashes and `PREFLIGHT_NIRI`.
- Produces: one recorded 9/9 preflight with feasibility-host version/hash and
  numeric signal, noise, and margin for every row.

- [x] **Step 1: Verify pins and environment before starting Weston**

```bash
test -d "$NIRI_MATERIAL_WORK_ROOT" && test -w "$NIRI_MATERIAL_WORK_ROOT"
test -x "$PREFLIGHT_NIRI"
test "$(git -C "$NIRI_GLASS_ROOT" rev-parse HEAD)" = \
  70f26af4324635bb827d07f0eaf1fc227acb43ea
git status --short
sha256sum "$PREFLIGHT_NIRI" fixtures/diagnostic-grid.png \
  fixtures/v1-parity-replay.sh fixtures/v1-parity-analyze.mjs \
  fixtures/v1-parity-analyze.test.mjs
```

Expected: clean evidence branch and exact frozen reference commit.

- [x] **Step 2: Start the dedicated headless host and run preflight**

```bash
weston_unit=niri-material-v1-reference-static-weston
systemd-run --user --unit="$weston_unit" --collect \
  --setenv=XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
  weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
  --width=1280 --height=720 --socket=v1-reference-static
trap 'systemctl --user stop niri-material-v1-reference-static-weston.service >/dev/null 2>&1 || true' EXIT HUP INT TERM
attempt=0
while test "$attempt" -lt 100 \
  && test ! -S "$XDG_RUNTIME_DIR/v1-reference-static"; do
  sleep 0.1
  attempt=$((attempt + 1))
done
test -S "$XDG_RUNTIME_DIR/v1-reference-static"
fixtures/v1-parity-replay.sh --preflight-reference \
  "$NIRI_GLASS_ROOT" v1-reference-static
preflight_status=$?
systemctl --user stop "$weston_unit.service"
journalctl --user -u "$weston_unit.service" \
  >"$NIRI_MATERIAL_WORK_ROOT/v1-reference-static-weston.log"
trap - EXIT HUP INT TERM
test "$preflight_status" -eq 0
```

Expected: analyzer exit `0`, exactly nine passing reference rows, no remaining
`v1-reference-static.*` capture directory, and the Weston socket/process gone.
If the status is nonzero, stop here: retain and print the failing artifact,
diagnose it, and do not run Task 4.

- [x] **Step 3: Record the preflight without replacing the old full verdict**

Add a “Schema-2 reference static preflight” section to the result document.
Record the evidence commit, preflight niri version/hash, fixture hashes, ROI,
support/neighbor counts, boundary count, all nine signals/noise/margins, and
the diagnostic asset SHA and deleted raw-artifact lifecycle. Correct the stale
evidence branch from `results/v1-parity` to `results/slice3` for the old run
and name the new branch for the new run.

- [x] **Step 4: Verify and commit the preflight record**

```bash
rg -n '9/9|boundary|PREFLIGHT_NIRI|results/slice3|7729dfc' \
  docs/results/2026-08-24-v1-parity.md
git diff --check
git add docs/results/2026-08-24-v1-parity.md
git commit -m "docs(results): record reference static preflight"
```

---

### Task 4: Recapture full parity and clean raw build artifacts

Only after Task 3 passes, rebuild the pinned old native surface, run both
final phases under that binary, and record the complete result whether it
passes or fails semantically.

**Files:**
- Modify: `niri-experiments/docs/results/2026-08-24-v1-parity.md`

**Interfaces:**
- Consumes: exact native commit `5411ac05de730e79d4bbacf57b8bbd2b43cec310`,
  committed analyzer/replay, frozen reference, and required work root.
- Produces: schema-2 `analysis.json`, a complete 28-row implementation matrix,
  14 combined verdicts, a committed distilled result, and no retained raw
  capture or old Cargo target.

- [x] **Step 1: Create the detached old-surface worktree and build host binary**

```bash
git -C "$NIRI_MATERIAL_ROOT" worktree add --detach \
  "$NIRI_MATERIAL_ROOT/.worktrees/v1-parity-old-surface" \
  5411ac05de730e79d4bbacf57b8bbd2b43cec310
old_target="$NIRI_MATERIAL_WORK_ROOT/targets/v1-parity-old-surface"
CARGO_TARGET_DIR="$old_target" cargo build --release \
  --manifest-path "$NIRI_MATERIAL_ROOT/.worktrees/v1-parity-old-surface/Cargo.toml"
old_niri="$old_target/release/niri"
"$old_niri" --version
sha256sum "$old_niri"
```

- [x] **Step 2: Re-run all fixture checks with the pinned host**

```bash
fixtures/v1-parity-replay.sh --self-test "$old_niri" "$NIRI_GLASS_ROOT"
dash fixtures/v1-parity-replay.sh --self-test "$old_niri" "$NIRI_GLASS_ROOT"
node --test fixtures/v1-parity-analyze.test.mjs
```

Expected: all checks pass before capture.

- [x] **Step 3: Capture native and reference on distinct sequential hosts**

```bash
artifact_dir=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/v1-parity-final.XXXXXX")
weston_unit=niri-material-v1-parity-weston
start_weston_host() {
  systemd-run --user --unit="$weston_unit" --collect \
    --setenv=XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
    --width=1280 --height=720 --socket=v1-parity-headless || return
  attempt=0
  while test "$attempt" -lt 100 \
    && test ! -S "$XDG_RUNTIME_DIR/v1-parity-headless"; do
    sleep 0.1
    attempt=$((attempt + 1))
  done
  test -S "$XDG_RUNTIME_DIR/v1-parity-headless"
}
stop_weston_host() {
  host_label=$1
  systemctl --user stop "$weston_unit.service" || return
  journalctl --user -u "$weston_unit.service" \
    >"$artifact_dir/weston-$host_label.log" || return
  test ! -S "$XDG_RUNTIME_DIR/v1-parity-headless"
}
trap 'systemctl --user stop niri-material-v1-parity-weston.service >/dev/null 2>&1 || true' EXIT HUP INT TERM

start_weston_host
fixtures/v1-parity-replay.sh --capture-native \
  "$old_niri" "$NIRI_GLASS_ROOT" "$artifact_dir" v1-parity-headless
native_capture_status=$?
stop_weston_host native
test "$native_capture_status" -eq 0

start_weston_host
fixtures/v1-parity-replay.sh --capture-reference \
  "$old_niri" "$NIRI_GLASS_ROOT" "$artifact_dir" v1-parity-headless
reference_capture_status=$?
stop_weston_host reference
trap - EXIT HUP INT TERM
test "$reference_capture_status" -eq 0
```

Expected: both hosts are reaped sequentially, final phase is `complete`, and
all manifest-referenced files verify.

- [x] **Step 4: Analyze the full evidence and inspect the complete matrix**

```bash
node fixtures/v1-parity-analyze.mjs "$artifact_dir"
analysis_status=$?
jq '{integrity, verdict, row_count:(.rows|length), rows}' \
  "$artifact_dir/analysis.json"
test "$(jq -r .integrity "$artifact_dir/analysis.json")" = pass
test "$(jq '.rows | length' "$artifact_dir/analysis.json")" -eq 28
sha256sum "$artifact_dir/captures.sha256" "$artifact_dir/analysis.json"
```

Exit `0` means all rows passed; exit `1` is valid semantic failure and still
continues to documentation. Exit `2` is invalid evidence: retain the artifact,
diagnose, and repeat this task before documenting a verdict.

- [x] **Step 5: Replace the result with the new evidence**

Update every pin, host/tool version, ROI/support/boundary count, duplicate
floor, parameter signal, noise, direction, margin, combined verdict, caveat,
and lifecycle statement from the new manifest and analysis. Explicitly print
the small `samples` margin. Do not carry forward an old row or old branch name.

- [x] **Step 6: Verify and commit the distilled result**

```bash
sha256sum fixtures/diagnostic-grid.png fixtures/v1-parity-replay.sh \
  fixtures/v1-parity-analyze.mjs fixtures/v1-parity-analyze.test.mjs
if rg -n 'results/v1-parity|/tmp/niri-material|static_source|schema: 1|ac7c503e|Diagnostic SVG|200 px|200-pixel|38,50,56' \
  docs/results/2026-08-24-v1-parity.md; then exit 1; fi
git diff --check
git add docs/results/2026-08-24-v1-parity.md
git commit -m "docs(results): record corrected v1 parity evidence"
```

- [x] **Step 7: Delete raw capture and old build data**

After the result commit exists, delete the exact `artifact_dir`, run Cargo
clean against the dedicated target, and remove the detached worktree:

```bash
find "$artifact_dir" -depth -delete
CARGO_TARGET_DIR="$old_target" cargo clean \
  --manifest-path "$NIRI_MATERIAL_ROOT/.worktrees/v1-parity-old-surface/Cargo.toml"
find "$old_target" -depth -delete
git -C "$NIRI_MATERIAL_ROOT" worktree remove \
  "$NIRI_MATERIAL_ROOT/.worktrees/v1-parity-old-surface"
test ! -e "$artifact_dir" && test ! -e "$old_target"
```

---

### Task 5: Propagate the evidence status through niri-material

Update every current status claim after the evidence commit exists. The final
wording follows the measured verdict; it does not assume acceptance.

**Files:**
- Modify: `docs/materials/2026-08-25-v1-reference-static-preflight-design.md`
- Modify: `docs/materials/2026-08-24-v1-parity-design.md`
- Modify: `docs/materials/2026-08-22-v1-design.md`
- Modify: `docs/materials/README.md`
- Modify: `docs/materials/plans/2026-08-26-v1-reference-static-preflight.md`

**Interfaces:**
- Consumes: committed niri-experiments evidence revision and its exact final
  verdict.
- Produces: one internally consistent status story in design, plan, and
  user-facing progress docs.

- [x] **Step 1: Verify the evidence commit and result before editing claims**

```bash
evidence_commit=$(git -C "$NIRI_EXPERIMENTS_WORKTREE" rev-parse HEAD)
git -C "$NIRI_EXPERIMENTS_WORKTREE" merge-base --is-ancestor \
  7729dfc151eae41c946d2495ef67010c1cd20635 "$evidence_commit"
test -f "$NIRI_EXPERIMENTS_WORKTREE/docs/results/2026-08-24-v1-parity.md"
```

- [x] **Step 2: Update design and plan status from measured evidence**

Mark the preflight design implemented at the exact evidence commit. Replace
the old parity outcome in the parent design and v1 design. In the parent
design, also replace the superseded SVG hash, 200-pixel period, crop-fill
rationale, and sample-pixel gate with the committed PNG contract and new
measurement. Mark this plan executed with the preflight and final evidence
commits, final integrity and semantic verdict, and whether physical DRM
remains blocked.

- [x] **Step 3: Update the user-facing README and grep for propagated drift**

Add the preflight design and plan to the documentation list, replace the old
`7729dfc` outcome paragraph with the new evidence, then run:

```bash
rg -n '7729dfc|results/v1-parity|only `lip`|optics port is unverified|/tmp/niri-material|ac7c503e|200 px|200-pixel|diagnostic SVG|38,50,56' \
  docs/materials docs/wiki README.md
```

Every surviving match must be explicitly historical and correctly qualified;
otherwise update it in this same change.

- [x] **Step 4: Verify and commit niri-material status**

```bash
git diff --check
git status --short
git add docs/materials/2026-08-25-v1-reference-static-preflight-design.md \
  docs/materials/2026-08-24-v1-parity-design.md \
  docs/materials/2026-08-22-v1-design.md docs/materials/README.md \
  docs/materials/plans/2026-08-26-v1-reference-static-preflight.md
git commit -m "docs(material): record corrected v1 parity outcome"
```

- [x] **Step 5: Final verification**

```bash
git status --short
git -C "$NIRI_EXPERIMENTS_WORKTREE" status --short
node --test "$NIRI_EXPERIMENTS_WORKTREE/fixtures/v1-parity-analyze.test.mjs"
"$NIRI_EXPERIMENTS_WORKTREE/fixtures/v1-parity-replay.sh" --self-test \
  "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
dash "$NIRI_EXPERIMENTS_WORKTREE/fixtures/v1-parity-replay.sh" --self-test \
  "$PREFLIGHT_NIRI" "$NIRI_GLASS_ROOT"
```

Expected: both worktrees are clean and every fixture check passes. Report the
actual final evidence verdict and cleanup state; do not claim v1 acceptance
unless all combined rows passed.
