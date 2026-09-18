# Ring of light focus response: wakeup and capture evidence

**Status:** passing, 2026-09-05 on `design/ring-light`, except the DRM
acceptance runs, which are awaiting the operator on VT2. Every wakeup case and
every gated capture check passes on the nested headless host. Two properties
are recorded but **not verified by measurement** and are called out where they
belong: the filament's face confinement *under a running resize*, and its
zero-chamfer branch.

**Historical record:** these measurements use the pre-within filament
placement. Their cadence and fixture observations remain retained history, but
their placement and inset-validation conclusions are superseded by the
[render-order evidence](2026-09-12-material-render-order-evidence.md).

Measured against `166cd3e6` ("feat(material): cap the focus filament's
refracted shift"), the head of the ring-light implementation; no Rust source
has changed since, so the binaries below differ only in their embedded version
string and their profiling feature.

| Run | Binary | sha256 |
|---|---|---|
| wakeup cadence | release + `profile-with-tracy` | `237a399d4f5f155333d63e7d4ed573dfbc5ebfed74cbcbc1766f2116071d4455` |
| measured captures | release | `e06d52d80ee4239bbd3a1df463b333f07e4b79210808b11608515c2bc847a4a0` |

Both report `niri 26.04 (33b5217f-modified)`: the tree carried this document
and the two harness scripts on top of the commit. The two builds render
identically, which the captures show directly — `rest-confinement` returns the
same emissive triple to six decimals on either binary.

Defaults under test: `focus "ring-light"`, `ring-inset 5`, `ring-width 2.6`,
`ring-color "#ccccff"`, `ring-drift-hz 15`, `light-ior 6`.

## Wakeup cadence

`docs/materials/scripts/material-signals-smoke.sh cases`, exit 0, `cases: OK`.
Run directory `signals-1602095-1788624775`. Every pre-existing case now pins
`ring-drift-hz 0` in its material, so the focused window in each fixture no
longer drifts and the existing zero-redraw expectations still describe the
compositor rather than the new clock.

The seven new cases, the control they are measured against, and the unchanged
`quiet-ring` line, from `rates.txt`:

| Case | Measured redraws | Acceptance |
|---|---:|---|
| `quiet-ring` (unchanged, drift now pinned) | 0 | pass: zero |
| `focused-static` (focused, `ring-drift-hz 0`) | 0 (0.0/s) | pass: zero |
| `focused-drift` (focused, 15 Hz) | 300 (15.0/s) | pass: 300 +/-15% |
| `focused-reduced` (focused, 15 Hz, `motion "reduced"`) | 150 (7.5/s) | pass: 150 +/-15% |
| `focused-anim-off` (focused, 15 Hz, `animations { off; }`) | 0 (0.0/s) | pass: zero |
| `focus-none-drift` (focused, 15 Hz, `focus "none"`) | 0 (0.0/s) | pass: zero |
| `other-focused` (two windows, one focused, 15 Hz) | 300 (15.0/s) | pass: within 15% of `focused-drift` |
| `toggle-control` (six focus changes, no material at all) | 22 (1.1/s) | recorded, not gated |
| `focus-none-toggle` (the same six under `focus "none"`) | 21, and 0 in the 10.9 s after | pass: at most control + 6 = 28, then zero |

The drift lands exactly on its bucket rate: 15.0/s at `ring-drift-hz 15`, half
that under `motion "reduced"`, nothing at all under `animations { off; }`,
under `focus "none"`, or on an unfocused window. `other-focused` matching
`focused-drift` to the count is the direct statement that only the focused
window drifts: adding a second, unfocused window adds no wakeups.

### The focus toggle is measured against a client control

A focus change is client damage before it is anything else. In the Tracy trace
each redraw during the toggles carries 60 to 100 `CompositorHandler::commit`
zones at its own timestamp, in bursts of three to five frames per change: that
is kitty repainting its cursor as it gains and loses activation, and the
compositor has to serve it whatever the material does. An absolute bound on
this case would have measured the fixture, so the acceptance is the control.

`toggle-control` runs the same six toggles on the same two windows with **no
material on them at all** and costs 22 redraws. `focus-none-toggle`, the same
toggles with the material present and `focus "none"`, costs **21** — inside the
control, and well inside the control-plus-six bound that allows each change the
one coalescible redraw its own focus bookkeeping can request. Under
`focus "none"` the material adds nothing measurable to a focus change.

The "nothing after" window is derived rather than fixed. Each `niri msg action
focus-window` spawns the niri binary, so the six changes take about 6 s rather
than the 3 s of their sleeps, and the duration varies with host load; a fixed
window would straddle the toggles themselves. The case times its own sequence,
waits out the client's repaint burst, and starts the after-window where the
toggles ended — here 10.9 s of quiet tail, containing **0** redraws.

Every pre-existing case still meets its published acceptance with the drift
pinned: `demand-pulse` 533, `demand-pulse-focused` 0, `demand-breathe` 160,
`demand-flash` 320, `ten-breathe` 160, `ten-flash` 313, `inactive-workspace` 0,
`hidden-tab` 0, `offscreen-column` 0, `motion-off` 6, `reduced-flash` 534,
`attention-none` 0, `impulse-none` 6, `done-pulse` burst 93 with 0 after,
`slowdown` 533 — the same numbers as the 2026-09-03 signals evidence.

## Measured captures

`docs/materials/scripts/focus-ring-light.sh` with its default `CASES`, exit 0.
Capture directory `focus-ring-light-e06d52d8`. Each case renders the scene
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
| selectors `accent "ring"` + `focus "ring-light"`: face AE, red share | 0, 1.000 | pass: 0, at least 0.90 |
| selectors `accent "ring"` + `focus "none"`: face AE, red share | 0, 1.000 | pass: 0, at least 0.90 |
| selectors `accent "none"` + `focus "ring-light"`: face AE, red share | 0, 0.272 | pass: 0, 0.24 to 0.31 |
| selectors `accent "none"` + `focus "none"`: face AE, emissive luminance | 0, 0.0000 | pass: 0, at most 0.005 |

Recorded, not gated (see "resize-flex" below and "tiny" after it):

| Value | Measured |
|---|---|
| resize-flex: host layout skew at rest | 0 px |
| resize-flex: host layout skew mid-resize | -1 px |
| resize-flex: face pixels differing mid-resize | 261 of 254828 (261.494 recorded) |
| resize-flex: face max channel delta mid-resize | 104 |
| resize-flex: filament emissive luminance mid-resize vs rest | 1.182x |
| tiny (zero chamfer) | not verified by render |

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

### resize-flex: confinement under flex is not verified by measurement

The case starts two nested instances, identical but for the disabled response,
and drives `set-column-width +200` into both under `animations { slowdown 50;
}`, then samples both three seconds in.

At rest the two instances are byte-identical over the face (AE 0) and align at
exactly **0 px**, which is what makes every other cross-instance comparison in
this document valid, and which is gated.

Mid-resize they are not the same scene. One `niri msg` process spawn separates
the two `set-column-width` calls, and another the two screenshot requests, so
the hosts sit at different points in a 20 s animation: they align at **-1 px**
of layout, and each client has re-rendered its terminal text at a slightly
different width. The result is **261** differing face pixels out of 254828
(the recorded HDRI absolute-error value is 261.494), with a **104**-level
maximum, concentrated on glyph edges near the top-left of
the face — client content, not light. Any face light the filament could leak
under flex is below that floor, so this comparison cannot see it, and the
**1.182x** emissive ratio is likewise sampled across the skew rather than
across a clean jelly breath.

Face confinement under a running resize and the jelly-breath ratio are
therefore recorded and **not verified by measurement**. What *is* verified is
confinement at rest, above: zero face pixels change, and the whole-frame
difference is bounded by the slab rect. A deterministic mid-flex probe — both
renders at identical animation progress, by clock stepping or a single-host
toggle — is filed as `material-22d78f`.

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

## DRM acceptance

Both runs were performed by the operator on VT2 on 2026-09-05 against the
candidate built from `b8688ba4` (materials-26.04 after the merge and the doc
corrections; `niri 26.04 (b8688ba4-modified)`, sha256 `9855c8dc…`, snapshotted
at `$NIRI_MATERIAL_WORK_ROOT/ring-light-drm-b8688ba4/niri`). The fixture is
`niri-experiments` results/slice3 with two commits on top of 536992b: 0f33981
admits `b8688ba4` in the `material_source_commit` allowlist, and 9d8180e pins
`ring-drift-hz 0` in the tracked `fixtures/v1-drm-smoke.kdl` for the pinned
run; the drifting run used the tracked config as of 0f33981. Both runs were
analyzed with `--analyze`; the machine summaries are copied beside the binary.

### Pinned run (`ring-drift-hz 0`): the acceptance gate

Artifact `v1-drm-acceptance.1khrOY`. 17 of 19 gates pass. Every identity gate
is byte-identical: `static-repeat`, `reload-invalid-retained`,
`reload-restored`, `move-return`, `move-repeat`, `resize-return`,
`remap-return`, `overview-return`, `workspace-return`, `final-return`. The
non-identity gates `reload-changed`, `diagnostic-tile`, `move-motion`,
`resize-motion`, `overview-transition`, `invalid-reload-log` and `clean-log`
pass. `workspace-down-transition` and `workspace-up-transition` fail with "no
frame contained both exact colors"; see below.

### Drifting run (unmodified config, 15 Hz)

Artifact `v1-drm-acceptance.miY5lu`. All ten identity gates differ (the
fixture reports AE 2424 to 2964 per pair), as a drifting filament must; the
same seven non-identity gates pass and the same two workspace-transition gates
fail. The bounded-diff check per identity pair, with the probe rectangle
`x=1066 y=46 w=1320 h=1360` (read from the differing pixels' bounding box
`1332x1372+1060+40`, which is the window plus the 6 px exterior slab band
under `bevel 12` with 6 px offsets, and confirmed against the frame) and the
bound `2 * (w + h + 24) * 12 = 64896`:

| Pair | Band pixels | Outside the 12 px inflated rect | Deeper than 16 px into the window |
|---|---|---|---|
| initial-a / initial-b | 55780 | 0 | 0 |
| reload-valid / reload-invalid | 55680 | 0 | 0 |
| initial-a / reload-restored | 55568 | 0 | 0 |
| initial-a / move-settled-a | 56286 | 0 | 0 |
| move-settled-a / move-settled-b | 55992 | 0 | 0 |
| initial-a / resize-settled | 55824 | 0 | 0 |
| initial-a / remap-settled | 53929 | 0 | 0 |
| initial-a / overview-return | 55004 | 0 | 0 |
| initial-a / workspace-return | 55004 | 0 | 0 |
| initial-a / final-settled | 55831 | 0 | 0 |

The plan asked for zero differing pixels inside the window body on the premise
that the probe is opaque. The fixture's probe is translucent glass (the
acceptance design selects a translucent kitty for the `frost` material), so
the filament's halo legitimately reaches about 10 px inside the window edge,
and 26 to 28 thousand of each pair's band pixels lie in that inner strip. The
body was therefore measured inset by 16 px: zero differing pixels on every
pair. Counts are exact histogram counts of the thresholded difference image;
the numbers are kept in `ring-light-drm-b8688ba4/drift-bounded-diff.txt`.

### The two workspace-transition gates

Both runs fail `workspace-down-transition` and `workspace-up-transition`
identically, with the drift on and off, so the drift is not the cause. The
gate needs one of six frames (nominally 0, 50, 100, 200, 400 and 800 ms after
the action) to show both the probe's orange marker and the purple backdrop
mid-slide. In these runs the first capture request landed 35 ms (drifting) and
41 ms (pinned) after the action, against 18 ms on the accepted Sep 1 run, and
every later frame arrived 1.4 to 1.6 s apart against 0.3 s, because each
3440x1440 screenshot took about twice as long on this machine during these
runs. With the fixture's critically damped spring (stiffness 400) the windows
had slid 317 px (drifting) and 473 px (pinned) by the first frame, against
120 px on the accepted run, carrying the 240 px marker off the top of the
screen; the backdrop band is visible mid-screen in both first frames (y 1123
and 967), so the transition itself ran as before. Recorded as a capture-timing
failure of the gate, not a rendering difference; the owner decides whether it
blocks closure or a rerun is wanted.

### Fixture notes

- `--prepare` refuses when `$NIRI_MATERIAL_WORK_ROOT/run-v1-drm-acceptance.sh`
  is left from an earlier run. The Sep 1 wrapper was renamed to
  `run-v1-drm-acceptance.sh.regression-52f74f10` and the drifting run's to
  `run-v1-drm-acceptance.sh.ring-light-drift`.
- `--analyze` exited 2 on its first invocation for both runs and left a
  `.gates.*` temporary file in the artifact directory; removing that file and
  invoking it again produced the machine summary. Not investigated further.

## Retained artifacts

All under `$NIRI_MATERIAL_WORK_ROOT`:

- `focus-ring-light-e06d52d8/` — every capture, the per-case configs, the check
  log (`checks.txt`, where `ok:` lines are gates and `info:` lines are the
  recorded-not-gated values) and the frame hashes (`SHA256SUMS`).
- `material-signals-33b5217f/signals-1602095-1788624775/` — the wakeup run:
  Tracy traces, CSV exports and `rates.txt`.
- `ring-light-drm-b8688ba4/` — the DRM candidate binary and its hash, the
  pinned config, both machine summaries and the bounded-diff numbers;
  `v1-drm-acceptance.1khrOY/` (pinned) and `v1-drm-acceptance.miY5lu/`
  (drifting) hold the runs' captures and logs. `ring-light-drm-166cd3e6/` is
  the earlier snapshot that `--prepare` refused.
