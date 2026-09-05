# Ring of light focus response: wakeup and capture evidence

**Status:** partial, 2026-09-05 on `design/ring-light`. Seven of the eight
wakeup cases and three of the four measurable capture cases pass on the nested
headless host; the fifth capture case (`tiny`) is recorded as not verified by
render. The wakeup case `focus-none-toggle`, the two mid-resize bounds of
`resize-flex`, and both DRM acceptance runs do not close here. Each is recorded
below with its measured number and the reason, and none of them is a defect in
the rendering the plan added.

Measured against `166cd3e6` ("feat(material): cap the focus filament's
refracted shift"), the head of the ring-light implementation. The working tree
at capture time differed from that commit only in the two harness scripts and
this document, so the binary reports `niri 26.04 (166cd3e6-modified)`.

| Binary | sha256 |
|---|---|
| `cargo build --release` | `9900c0467290f733e5586fea8cd0317d611c19762d7c861064c02ef607217574` |
| `cargo build --release --features profile-with-tracy` (wakeup runs) | `68d1f1abd95b8bd0bfd695c15ca1f37712a8793cd6f950794f89ecc0d332dec6` |

Defaults under test: `focus "ring-light"`, `ring-inset 5`, `ring-width 2.6`,
`ring-color "#ccccff"`, `ring-drift-hz 15`, `light-ior 6`.

## Wakeup cadence

`docs/materials/scripts/material-signals-smoke.sh cases`. Every pre-existing
case now pins `ring-drift-hz 0` in its material, so the focused window in each
fixture no longer drifts and the existing zero-redraw expectations still
describe the compositor rather than the new clock.

The seven new cases and the unchanged `quiet-ring` line, from the gated run's
`rates.txt` (`signals-1322718-1788620308`):

| Case | Measured redraws | Acceptance |
|---|---:|---|
| `quiet-ring` (unchanged, drift now pinned) | 0 | pass: zero |
| `focused-static` (focused, `ring-drift-hz 0`) | 0 (0.0/s) | pass: zero |
| `focused-drift` (focused, 15 Hz) | 300 (15.0/s) | pass: 300 +/-15% |
| `focused-reduced` (focused, 15 Hz, `motion "reduced"`) | 150 (7.5/s) | pass: 150 +/-15% |
| `focused-anim-off` (focused, 15 Hz, `animations { off; }`) | 0 (0.0/s) | pass: zero |
| `focus-none-drift` (focused, 15 Hz, `focus "none"`) | 0 (0.0/s) | pass: zero |
| `other-focused` (two windows, one focused, 15 Hz) | 300 (15.0/s) | pass: within 15% of `focused-drift` |
| `focus-none-toggle` (six focus changes under `focus "none"`) | 23, then 12 in the final 14 s | **not met**: wanted at most 6, then zero |

The drift lands exactly on its bucket rate: 15.0/s at `ring-drift-hz 15`, half
that under `motion "reduced"`, nothing at all under `animations { off; }`,
under `focus "none"`, or on an unfocused window. `other-focused` matching
`focused-drift` to the count is the direct statement that only the focused
window drifts: adding a second, unfocused window adds no wakeups.

### focus-none-toggle: what the number means

The case fails, and it is not the material. Repeating the same six focus
toggles on the same fixture with **no material on the windows at all** costs
20 redraws in the same window; with the material and `focus "none"` it costs
21 to 23 over two runs. The redraws are the fixture's own client damage:
in the Tracy trace each redraw carries 60 to 100 `CompositorHandler::commit`
zones at its own timestamp, in bursts of three to five frames per focus
change, which is kitty repainting its cursor as it gains and loses activation.
No material configuration can make a focus change on this fixture cost one
redraw.

The `after` bound misses for a second, independent reason: `during_focus_toggles`
was costed at 3 s (six changes, 0.5 s apart), but each `niri msg action
focus-window` spawns the niri binary and takes about 0.5 s of its own, so the
sequence takes about 6 s and runs from `end-18 s` to `end-12 s`, straddling the
`[end-14 s, end)` window the "nothing after" check reads. Both bounds are
recorded as measured; neither was relaxed.

Every pre-existing case still meets its published acceptance with the drift
pinned, confirmed on a full pass of the suite (`signals-1476691-1788622538`,
run with the `focus-none-toggle` gate downgraded to a warning so the cases
after it could run): `demand-pulse` 533, `demand-pulse-focused` 0,
`demand-breathe` 160, `demand-flash` 320, `ten-breathe` 159, `ten-flash` 319,
`inactive-workspace` 0, `hidden-tab` 0, `offscreen-column` 0, `motion-off` 6,
`reduced-flash` 534, `attention-none` 0, `impulse-none` 6, `done-pulse` burst
91 with 0 after, `slowdown` 533 — the same numbers as the
2026-09-03 signals evidence.

## Measured captures

`docs/materials/scripts/focus-ring-light.sh`. Each case renders the scene
twice, once with the filament and once with it disabled (`response "default" {
focus "none"; accent "none"; }`), so refraction, attenuation, Fresnel and jelly
are identical in both and only the filament differs. Every case pins
`ring-drift-hz 0`.

The capture material is the focus-glass spike's dark split glass with three
kinds of change:

- pinned so the sampling coordinates stay valid as the generated
  `prism.kdl` drifts: `ior 1.02`, `thickness 41.7`, `bevel 11` (with the
  block's own `offset-x 1`, `offset-y 1`); the harness refuses to run if the
  written config does not carry all three on both materials;
- measurable: `attenuation-color "#888888"` so the attenuation exponent scales
  all channels equally and hue survives, and `chromatic-aberration 0` so the
  three per-channel bands are identical;
- unchanged otherwise, including the base `ring-color "#ccccff"` (linear
  0.604, 0.604, 1.0). The accent used throughout is `#ff0000` (linear 1, 0, 0).

Geometry on the 1280x720 headless host with gaps 54 and two half-width
columns: the focused right window is 559x612 at (667, 54); with `bevel 11` and
offset 1 the slab is the window inflated by 10 and slid by 1, so its top edge
is y=45. The filament is sampled at (946, 50), `ring-inset` px inside that
edge; the face region is the window inset by 40 px on every side
(479x532+707+94).

| Check | Measured | Acceptance |
|---|---|---|
| rest-confinement: face pixels changed by the filament | 0 | pass: 0 |
| rest-confinement: filament red share | 0.272 | pass: 0.24 to 0.31 |
| rest-confinement: filament emissive luminance | 0.0719 | pass: above 0.01 |
| accent-midfade: red share at half fade | 0.498 | pass: 0.46 to 0.54 |
| accent-midfade: red share settled | 1.000 | pass: at least 0.90 |
| resize-flex: face pixels differing at rest | 0 | pass: 0 |
| resize-flex: face max channel delta under flex | 104 | **not met**: wanted at most 2 |
| resize-flex: filament emissive luminance vs rest | 1.186x | **not met**: wanted at least 1.20x |
| selectors `accent "ring"` + `focus "ring-light"`: face AE, red share | 0, 1.000 | pass: 0, at least 0.90 |
| selectors `accent "ring"` + `focus "none"`: face AE, red share | 0, 1.000 | pass: 0, at least 0.90 |
| selectors `accent "none"` + `focus "ring-light"`: face AE, red share | 0, 0.272 | pass: 0, 0.24 to 0.31 |
| selectors `accent "none"` + `focus "none"`: face AE, emissive luminance | 0, 0.0000 | pass: 0, at most 0.005 |
| tiny (zero chamfer) | not verified by render | recorded, see below |

Sampled emissive triples at (946, 50), linear light, filament minus disabled:

| Scene | r | g | b |
|---|---:|---:|---:|
| rest, no accent | 0.068547 | 0.068547 | 0.115011 |
| `accent "ring"` + `focus "ring-light"`, settled red | 0.160641 | 0.000000 | 0.000000 |
| `accent "ring"` + `focus "none"`, settled red | 0.043461 | 0.000000 | 0.000000 |
| `accent "none"` + `focus "ring-light"`, settled red | 0.068547 | 0.068547 | 0.115011 |
| `accent "none"` + `focus "none"`, settled red | 0.000000 | 0.000000 | 0.000000 |

The rest triple's channel ratio (0.0685 : 0.0685 : 0.1150) reproduces
`ring-color` in linear light (0.604 : 0.604 : 1.0) to three digits, and the
`accent "none"` selector reproduces it exactly: the filament is never tinted
where the accent is not selected. The half-fade red share of 0.498 is the
straight mix at presence 0.5 (`mix(#ccccff, #ff0000, 0.5)` gives 0.500); the
doubly faded regression the check was written against would give at most 0.435.

### Confinement

Beyond the face-region count, the whole-frame difference between the
filament-enabled and filament-disabled rest frames has bounding box
`579x632+658+45` — exactly the slab rect (the window inflated by 10 and slid by
1). Not one pixel outside the slab changes. Within the slab the difference
runs from y=45 (the slab edge, 37/255) through a peak at y=47 (139/255) and
decays inward, and is zero at y=44 and above.

### resize-flex: what the two numbers mean

The case starts two nested instances, identical but for the disabled response,
and drives `set-column-width +200` into both under `animations { slowdown 50;
}`, then samples both three seconds in. At rest the two instances are
byte-identical over the face (AE 0), which is what makes every other
cross-instance comparison here valid.

Under the moving resize they are not. Scanning row y=360 of the two flex
frames puts the right window's slab edge at x=557..561 in the
filament-enabled frame and x=555..556 in the disabled one: the two instances
are 2 to 5 px apart in the resize. The animation travels 200 px in about 20 s,
so that is 0.2 to 0.5 s of clock skew — one `niri msg` process spawn, which is
what separates the two `set-column-width` calls and the two screenshot
requests. A geometric offset of several pixels moves window content and the
refracted backdrop, which is what the 104-level face delta measures; it is not
light on the face. The emissive ratio of 1.186x is sampled across that same
offset, so it does not measure the jelly breath either.

Both numbers are recorded as measured. Neither was tuned, and no acceptance
bound was relaxed to admit them. Making this case measurable needs the two
resizes and the two screenshots driven from one trigger with sub-frame skew,
which the current one-process-per-IPC-call harness cannot provide.

### tiny (zero chamfer): not verified by render

The shader gate is

```glsl
if ((showAccent || showFocus) && slabChamfer > 0.0) {
```

with `slabChamfer` from `material_frame`'s

```rust
let max_chamfer = (slab.size.w.min(slab.size.h) / 2. - 1.).max(0.);
glass.bevel.min(max_chamfer) as f32
```

so the gate is only reached with a zero chamfer when the slab is at most 2 px
on one axis. A chamfer of 0 is otherwise unconfigurable with a filament, since
`ring-inset + ring-width <= bevel` with `ring-width > 0`. No client on this
host produces such a window: kitty and foot have a one-cell minimum and
`weston-simple-egl` is fixed at 250 px. `niri-visual-tests` renders with
`xray: None`, so materials do not draw there. The branch is covered by the
Rust-side `material_frame` tests only. Whether a tiny-client fixture is worth a
follow-up task is a closure decision.

## DRM acceptance: awaiting operator run

The retained DRM gate lives in the `niri-experiments` evidence worktree
(`fixtures/v1-drm-smoke.sh`, procedure in
`docs/materials/plans/2026-08-27-v1-drm-acceptance.md`). Its identity gates
require paired settled frames to be byte-identical, which a drifting filament
cannot satisfy, so the acceptance is two runs: a pinned one that must pass
every gate, and a drifting one bounded to the bevel band.

Neither run happened here, and neither handoff could be prepared. `--run`
requires an active VT2 session on seat0 with the RTX 3070 and DP-1, which no
agent session can provide; but `--prepare`, which does work outside VT2,
refused both configurations for reasons that need a decision in the
`niri-experiments` repository:

**Pinned run.** `fixtures/v1-drm-smoke-pinned.kdl` was written next to the
original — the retained config with

```kdl
material "frost" {
    response "default" {
        ring-drift-hz 0
    }
    glass { ... }
}
```

added — and validates against this build. `--prepare` refuses it:

```
fixture worktree must be clean
```

(exit 2). The fixture requires `git status --short` to be empty, so an
untracked config makes every `--prepare` in that worktree fail, and
`set_paths` hard-codes `config_path=$repository_root/fixtures/v1-drm-smoke.kdl`
in any case, so the fixture cannot be pointed at another file. A pinned run
needs the drift pinned in the tracked `v1-drm-smoke.kdl` and committed in
`niri-experiments`.

**Drifting run.** With a clean worktree, `--prepare` gets as far as writing the
handoff and then refuses at `validate_handoff` (exit 2) because the candidate's
source commit is not admitted:

```sh
case "$(jq -r '.pins.material_source_commit' "$handoff")" in
    138697be4cbb779c80425fe2a366ceca3610f38e) ;;  # accepted v1
    52f74f1059fd9651ed378db60a077129d53dc0d9) ;;  # v1 + backdrop-blur
    *) return 2 ;;
esac
```

The fixture's own comment says admitting a source is "a reviewable edit rather
than a caller argument", so `166cd3e6cd7b28563d8a095ba35ad02855c0db48` has to
be added to that list and committed in `niri-experiments` before this build can
be prepared at all.

Once both edits land in the fixture repository, the operator runs, from the
evidence worktree, for each of the two configs:

```sh
export NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material
artifact_dir=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/v1-drm-acceptance.XXXXXX")
handoff=$NIRI_MATERIAL_WORK_ROOT/v1-drm-acceptance-handoff.json
sh fixtures/v1-drm-smoke.sh --prepare \
    "$NIRI_MATERIAL_WORK_ROOT/ring-light-drm-166cd3e6/niri" \
    "$artifact_dir" "$handoff" \
    'niri 26.04 (166cd3e6-modified)' \
    166cd3e6cd7b28563d8a095ba35ad02855c0db48
sh fixtures/v1-drm-smoke.sh --run "$handoff"       # active VT2, seat0
sh fixtures/v1-drm-smoke.sh --analyze "$handoff"
```

The candidate binary is snapshotted at
`$NIRI_MATERIAL_WORK_ROOT/ring-light-drm-166cd3e6/niri`
(sha256 `9900c046...`, recorded beside it in `binary.sha256`); rebuild and
re-snapshot if the source moves, and pass the new `--version` string and
commit.

**Pass criteria, pinned run.** Every gate, including `static-repeat`,
`move-repeat`, `resize-return`, `remap-return` and `final-return`, passes. A
failure blocks closure.

**Pass criteria, drifting run.** The run completes with no compositor error in
its log and every non-identity gate passes. Each identity gate that fails must
fail only inside the probe's bevel band. Take the probe window rectangle
`x,y,w,h` from the gate's artifact JSON and, for each failing pair:

```sh
magick a.png b.png -compose difference -composite -threshold 0 diff.png
# Outside the band (everything but the window inflated by the 12 px bevel): must be empty.
magick diff.png -fill black -draw "rectangle $((x-12)),$((y-12)) $((x+w+12)),$((y+h+12))" -format '%[fx:int(mean*w*h)]' info:
# Inside the window body (opaque probe pixels pass through the shader untouched): must be empty.
magick diff.png -crop "${w}x${h}+${x}+${y}" +repage -format '%[fx:int(mean*w*h)]' info:
# The band itself: at most its area.
magick diff.png -format '%[fx:int(mean*w*h)]' info:
```

The first two counts must be 0 and the third at most `2 * (w + h + 24) * 12`.
Any other outcome, or any non-identity failure, blocks closure.

## Retained artifacts

All under `$NIRI_MATERIAL_WORK_ROOT`:

- `focus-ring-light-9900c046/` — every capture, the per-case configs, the
  check log (`checks.txt`) and the frame hashes (`SHA256SUMS`).
- `material-signals-166cd3e6/` — the wakeup runs: Tracy traces, CSV exports and
  `rates.txt` per run directory.
- `ring-light-drm-166cd3e6/` — the snapshotted candidate binary and its hash,
  for the operator's DRM runs.
