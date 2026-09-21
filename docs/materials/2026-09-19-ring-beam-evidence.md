# The ring beam: execution evidence

**Status:** implemented on `material-2c3984` (`ring.rs` `5610d666`, config
and compositor `98013739`, fixtures `0a12a7bd`, `0219599f`, scheduling
`06b71bb1`); every redraw-count case of the signals smoke passed on
2026-09-21 under the binary built from `35e736b6`, and the six review
sheets of the design's §6 are recorded from the same tree (the preflight
had refused every attempt of 2026-09-20 on desktop activity; the sheets
landed once the desktop was logged out). The constants judged on the
sheets (`ring-gap`, spill, envelope, pace, bevel 0) await the owner's
acceptance; this record does not grade them. **Owner acceptance: given
2026-09-21, defaults unchanged** (last section).

**Design:** [2026-09-19-ring-beam-design.md](../specs/2026-09-19-ring-beam-design.md).
**Plan:** `docs/plans/2026-09-19-ring-beam.md`. **Task:** `material-912dab`,
step 6 of `material-2c3984`; `material-9306b5` (ring appearance) is closed as
subsumed when the sheets are accepted.

## Host and binary

Host `titan`, the headless lane: every nested instance runs under its own
`weston --backend=headless --renderer=gl` unit, never the desktop session.
Worktree HEAD `35e736b6c5fb3fd57d2b2b6d89a6c6ef04adef3f` for the sheets
and the smoke of 2026-09-21 (the tree differed from it only by the task
record). The sheets and the smoke ran one after the other, never
concurrently, with the desktop session logged out; no other process used
the GPU (the capture baseline lists no GPU client). The first smoke, of
2026-09-20 under `0219599f`, stands as the earlier record below.

| Binary | Build | SHA-256 |
| --- | --- | --- |
| `material-signals-35e736b6/signals-3039250-1789981256/niri` | `cargo build --release --features profile-with-tracy` at `35e736b6` | `9116b3dc0145e3d76061abef46cf447a1b585fe5c2640bd16dd1a417e800769d` |
| `ring-motion-clips-35e736b6/ring-clips-2977602-1789979677/niri` | `cargo build --release` at `35e736b6` (the sheets' release binary) | `f8ef14fe552161e224d0ff396e3bb47c8d35b680c4d7aa32e6cdab8778108b4e` |
| `…/niri-nospill` | the same tree with `BEAM_SPILL` 0.25 → 0.00 in `ring.rs` and `main.frag` | `f5726b1fcfc4e69529a03e9d7bfca94bda22e3f604fda10f8b8d1f27b1ebcd2b` |
| `…/niri-splash` | the same tree with `BEAM_ENVELOPE` Plateau → Splash in `ring.rs` | `a67ed73253331411634a15b2241adaeda9390c9fb05f1da77b5fae1b4163dbb5` |
| `material-signals-0219599f/signals-11660-1789901391/niri` | `cargo build --release --features profile-with-tracy` at `0219599f` (the 2026-09-20 smoke) | `e1f73dd59fd51ce201ca4beb8cb2c84c0b43bec3e213d2800509e9260832f09d` |

## Spec contracts checked against the implementation

Two contracts the design settled before execution were re-read against the
code at `0219599f` for this record, and again at `35e736b6` (`06b71bb1`
touched only which layout query reports the beam's pending frame, not the
face, the perimeter or the completion rule); both hold.

- **§1.2 rendered face.** `Tile::material_dynamics` (`src/layout/tile.rs`)
  builds the face from the rendered `MaterialFrame` — the slab size is
  `slab_rect × area_size`, the chamfer is `frame.chamfer`, the radii are the
  fitted `CornerRadius` the element receives — and the jelly resize of the
  same frame, through `ring::face(slab_size, chamfer, corner_radius,
  jelly_resize)` (`src/render_helpers/material/ring.rs`). `face` scales the
  inner half-extents by `1 + resize / max(2·half_ext, 1)` and refits the
  radii against the binding box, which is `slabSurface`'s `inner_half` and
  `inner_r` (`prelude.frag`, lines 334–354) term for term. `beam_perimeter`
  and the shader's `arcPosition` then read the same face; the only textual
  difference from the spec is the Rust signature, `beam_perimeter(&Face,
  gap)` on a `Face { half, radii }` struct rather than three loose
  arguments. Completion is decided in `material_dynamics` on that frame's
  perimeter (`head_px >= perimeter + tail_length(perimeter)` sets
  `done`); `advance_animations` only drops a beam already marked done.
- **§2 never-rendered timeout.** `FocusBeam::is_done` expires a beam at
  `BEAM_MAX_RUN` (120 s) only while `rendered` is false; `material_dynamics`
  sets `rendered` on the first geometry evaluation and nothing clears it,
  so hiding a rendered tile neither resets the flag nor re-arms the
  timeout. The tile tests
  `the_beam_ends_only_when_a_rendered_frame_sees_the_tail_clear`,
  `a_rendered_slow_beam_outlives_the_unrendered_backstop` (`P = 6000`,
  `speed = 50`, active at 121 s, ends at 144 s) and
  `hiding_a_rendered_beam_does_not_rearm_the_backstop` cover the three
  clauses.

## Redraw counts, 2026-09-21 (and 2026-09-20)

Command, from `.worktrees/material-2c3984`:

```bash
NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/material-signals-smoke.sh cases
```

Two full runs. The first, on 2026-09-20 under `0219599f`: exit 0, log
`$NIRI_MATERIAL_WORK_ROOT/ring-beam-cases-0219599f.log`, rates in
`$NIRI_MATERIAL_WORK_ROOT/material-signals-0219599f/signals-11660-1789901391/rates.txt`.
The second, on 2026-09-21 under `35e736b6` — after `06b71bb1` moved the
beam's frame scheduling from `are_transitions_ongoing` to
`are_animations_ongoing`, so that the evidence names the binary the branch
ships — exit 0, log `$NIRI_MATERIAL_WORK_ROOT/ring-beam-cases.log`, rates
in
`$NIRI_MATERIAL_WORK_ROOT/material-signals-35e736b6/signals-3039250-1789981256/rates.txt`.
The table gives both as `2026-09-21 / 2026-09-20` where they differ.
The nested `niri.log` holds no error, warning or panic. Counts are `Niri::redraw` zones in the final
20 s of a 30 s Tracy capture unless a window is named. Every fixture carries
`signal { idle-after-ms 0 }` except the two idle cases.

Two cases were renamed by the fixture commit `0a12a7bd`, so earlier
`rates.txt` files line up as follows: `focused-static` (`ring-sweep-ms 0`)
is now `beam-off` (the base fixture's `ring-beam-speed 0`, the same window
at rest), and `focus-none-sweep` is now `focus-none-beam` (`focus "none"`
with `ring-beam-speed 1200`). The sweep cases `sweep-toggle` (six gains, one
lap each) became `beam-run`: one focus gain on the two-column scene at
`ring-beam-speed 1200; ring-gap 8`. The focused face there is about
616 × 688 (`P ≈ 2600`, `L ≈ 650`), so the run is about 2.7 s, passed to the
case as 3 s; the run window is `[12, 15.5)` s of the capture and the after
window the final 12 s, which no run at this speed can reach.

| Case | Result | Expectation | Met |
| --- | --- | --- | --- |
| `quiet-ring` | 0 | 0 | yes |
| `beam-off` (`ring-beam-speed 0`; was `focused-static`) | 0 | 0 | yes |
| `focused-settled` (`ring-beam-speed 1200`, focused before the window) | 0 | 0 — settled focus costs nothing (design §2) | yes |
| `focused-reduced` (`motion "reduced"`) | 0 | 0 | yes |
| `focused-anim-off` (`animations { off }`) | 0 | 0 | yes |
| `focus-none-beam` (`focus "none"`; was `focus-none-sweep`) | 0 | 0 | yes |
| `other-focused` (the beam material on an unfocused window) | 0 | 0 | yes |
| `toggle-control` (no material, six focus changes) | 23 / 24 | recorded | — |
| `focus-none-toggle` | 19 / 20 total, 0 in the 10.9 s after | ≤ control + 6; 0 after | yes |
| `beam-run` (one gain, two-column scene, `ring-beam-speed 1200`) | 152 / 146 during the 3 s run, 0 in the 12 s after | ≥ 60 during (3 × 20); 0 after | yes |
| `demand-pulse` | 533 | 540 ± 15 % | yes |
| `demand-pulse-focused` | 0 | 0 | yes |
| `demand-breathe` | 160 | 160 ± 15 % | yes |
| `demand-flash` | 320 | 320 ± 15 % | yes |
| `idle-pulse` (`idle-after-ms 5000`, no input) | 0 | 0 | yes |
| `idle-resume` (one `wlrctl pointer move 1 0` at 12 s) | 108 / 115 in the 5 s after input; 0 after the gate re-engaged | 533 × 5/20 = 133 ± 25 % (100–166); 0 | yes |
| `dpms-off-pulse` (`power-off-monitors`) | 0 | 0 | yes |
| `ten-breathe` / `ten-flash` | 160 / 320 (160 / 319) | within 10 % of one window (160 / 320) | yes |
| `inactive-workspace`, `hidden-tab`, `offscreen-column` | 0 / 0 / 0 | 0 | yes |
| `motion-off`, `impulse-none` | 6 / 6 | ≤ 6, 0 after | yes |
| `reduced-flash` | 533 | pulse rate ± 15 % | yes |
| `attention-none` | 0 | 0 | yes |
| `done-pulse` | burst 93, after 0 | burst ≥ 30, after 0 | yes |
| `slowdown` | 534 / 533 | pulse rate ± 10 % | yes |

Reading the beam cases:

- `beam-run`: 152 (146) redraws in the 3.5 s window around a run of about 2.7 s
  is the llvmpipe host's animation-loop rate (`done-pulse`'s impulse burst
  runs at the same rate: 93 in 3 s, both days), and the 12 s after the run are zero:
  the beam is finite, the tail drain ends on the frame that sees the head
  past `P + L`, and the settled ring reports no deadline and no damage. The
  brief's figure for this case was "≥ 80"; the fixture's own bound is
  `3 × 20 = 60`, and both runs clear both.
- `focused-settled` and `beam-off` are 0: a focused window at rest costs
  nothing whether the beam ran and finished or never ran.
- The unchanged attention cases repeat the 2026-09-19 numbers to the
  frame on both days (`demand-pulse` 533, `reduced-flash` 533, `slowdown`
  533–534, `done-pulse` 93), so the beam changed nothing outside focus.
  `06b71bb1`'s scheduling move changed no count either: the beam cases
  and the focus toggles match the `0219599f` run within the host's
  frame-to-frame variance.

No case was skipped in either run.

### The aborted runs

Each day's first run failed one case that the rerun passed; neither
failure is the beam's.

2026-09-21, `signals-3013733-1789980505` (log
`ring-beam-cases-35e736b6-attempt1.log`): `idle-resume` counted 99 against
the 100–166 bound, after `beam-run` had passed with 159 during and 0
after. The case counts `Niri::redraw` in `[end − 18 s, end − 13 s)` of the
trace while the pointer move fires 12 s from the trace's start, and this
capture ended at 35.44 s where the 2026-09-20 one ended at 36.43 s, so the
window slid a second earlier and cut the burst's last 1.4 s. Histogrammed
per second, the two traces hold the same burst: 136 redraws at 27/s over
five seconds (2026-09-21 seconds 18–23: 7, 27, 26, 27, 27, 22; 2026-09-20
seconds 19–24: 23, 27, 27, 26, 27, 6). The fixture's window is anchored to
the wrong end; filed as `material-85f0c0`. The rerun above counted 108.

2026-09-20, `signals-4172611-1789900736` (log
`ring-beam-cases-attempt1.log`):
the run failed at `demand-pulse-focused` with 7 redraws in the steady
window, after `beam-run` had already passed there with 143 during and 0
after. The seven redraws are one 120 ms burst at
19.5 s of the trace, preceded by 180 `CompositorHandler::commit` zones and
followed by nothing; the host's `kitty.conf` was written at that instant
(its mtime is 0.6 s before the burst), and kitty reloads and repaints on a
config write, in the nested instance as on the desktop. A client repaint
storm, not a material redraw: the case has no beam (`ring-beam-speed 0`)
and no material zone in the trace. That day's rerun was clean end to end.

## Review sheets, 2026-09-21

`docs/materials/scripts/ring-motion-clips.sh` records the six sequences of
the design's §6 under the capture protocol (`tools/capture-meta` preflight,
identity, settle before every launch, release), from a release build. Every
sequence is the same scene — two `kitty` panes at `background_opacity 0.6`
over a checkerboard, `gaps 24`, `focus-ring { off }`, the startup hotkey
overlay skipped, the idle gate off — and the same drive: the right pane
focused and settled, a rest shot, then a 15 s burst of full-frame
screenshots as fast as the async screenshot path allows, with focus moved to
the left pane just after the burst starts. Each sequence leaves a
`<name>/` frame directory with every frame's request instant in
`frames.txt`, a `<name>.gif`, a `<name>-sheet.png` (8 across), and
`<name>-corner/` with 4× crops of the focused pane's top-left corner at rest,
mid-pass (the frame nearest 2 s) and as the tail clears (nearest 13 s).

| Sequence | Fixture | Binary |
| --- | --- | --- |
| `beam-run` | `beam.kdl`: `bevel 10; ring-gap 8; ring-beam-speed 300; ring-glow 1` | release |
| `beam-gap16` | `beam-gap16.kdl`: the same at `ring-gap 16` | release |
| `beam-nospill` | `beam.kdl` | scratch build, `BEAM_SPILL 0` |
| `beam-splash` | `beam.kdl` | scratch build, `BEAM_ENVELOPE Splash` |
| `beam-fast` | `beam-fast.kdl`: the same at `ring-beam-speed 900` | release |
| `beam-bevel0` | `beam-bevel0.kdl`: `bevel 0; offset-x 0; offset-y 0` (no chamfer, so no spill) | release |

Command, from `.worktrees/material-2c3984`:

```bash
CAPTURE_TASK=material-912dab NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/ring-motion-clips.sh
```

Exit 0 (`clips: OK`); run `ring-clips-2977602-1789979677` under
`$NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-35e736b6/`, log
`$NIRI_MATERIAL_WORK_ROOT/ring-beam-clips.log`, started 04:34:37 with the
desktop session logged out (`session.type tty`). Preflight `quiet` on a
20 s baseline: CPU 1.4 % busy, load 0.29, GPU 0 % at P8 throughout,
10.6 W with an IQR of 0.29 W, no GPU client. The release binary was
rebuilt by the run from `35e736b6` (the tree differed from that commit
only by the task record `tasks/material-912dab.md`, which `tasks start`
had rewritten); the two scratch builds followed from copies of the tree.
The capture record (`capture.json`) carries the baseline, the per-launch
settles and the three binary hashes.

| Sequence | Binary | Frames | Span | `<name>.gif` | `<name>-sheet.png` |
| --- | --- | --- | --- | --- | --- |
| `beam-run` | `niri` | 149 | 14.985 s | `1b61078ad8be…` | `87043b97bfcb…` |
| `beam-gap16` | `niri` | 148 | 14.919 s | `6a70aece0cb2…` | `a113388c792f…` |
| `beam-nospill` | `niri-nospill` | 149 | 14.981 s | `4ff6ce7a9488…` | `910fa32d2d1b…` |
| `beam-splash` | `niri-splash` | 148 | 14.983 s | `dabfa1c9aeb2…` | `04ed53076c62…` |
| `beam-fast` | `niri` | 149 | 14.929 s | `2f3d47424cca…` | `898696489eb0…` |
| `beam-bevel0` | `niri` | 147 | 14.992 s | `b1cf8252904c…` | `5838d46fb21a…` |

(SHA-256 prefixes; the full hashes are `sha256sum` over the run
directory.) About ten frames a second is what the async screenshot path
delivers on this host; at 300 px/s that is one frame every 30 px of
travel, so the head is sampled, not streaked.

### What the frames show

The head was traced by differencing every frame against the frame nearest
13 s (the ring at rest), masking the cursor cell, and taking the centroid
of pixels that differ by more than 6/255. The focused pane is the left
one; its slab spans about x 24–628, y 24–696. Centroid `(x, y)` and peak
difference per frame:

| Sequence | f003 (0.24 s) | f010 (1.0 s) | f020 (2.0 s) | f030 (3.0 s) | f050 (5.0 s) | f070 (7.0 s) | f080 (8.0 s) | f090 (9.0 s) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `beam-run` | (119, 49) / 12 | (329, 49) / 10 | (624, 62) / 41 | (623, 305) / 41 | (400, 690) / 41 | (43, 432) / 41 | (44, 165) / 21 | — |
| `beam-gap16` | (114, 53) / 37 | (273, 52) / 33 | (581, 67) / 41 | (618, 333) / 33 | (346, 684) / 41 | (51, 383) / 33 | (52, 139) / 16 | — |
| `beam-nospill` | (113, 49) / 11 | (319, 49) / 10 | (617, 64) / 41 | (619, 299) / 41 | (405, 687) / 41 | (44, 429) / 41 | (44, 165) / 19 | — |
| `beam-splash` | (114, 49) / 12 | (324, 48) / 8 | (624, 61) / 30 | (622, 336) / 24 | (333, 689) / 13 | — | — | — |
| `beam-fast` | (256, 49) / 9 | (622, 279) / 41 | (109, 690) / 41 | — | — | — | — | — |
| `beam-bevel0` | — | — | (618, 58) / 32 | (619, 278) / 41 | (415, 687) / 41 | (33, 449) / 41 | (32, 158) / 22 | — |

- **One lap, then rest.** `beam-run` starts at the top-left corner on the
  first frame after the focus change, runs the top edge at about 30 px per
  0.1 s frame (300 px/s), descends the right edge from 2 s, crosses the
  bottom at 5 s, climbs the left edge at 7 s, and has no difference from
  the rest frame from 9 s on: the face's perimeter is about 2400 px, so the
  lap is 8 s, plus the 300 ms fade. No frame after that differs from rest.
- **Pace.** `beam-fast` covers the same path in a third of the time — the
  right edge at 1 s, the bottom at 2 s, the left edge at 2.5 s — and is at
  rest by 3 s.
- **Gap.** `beam-gap16`'s ring sits 8 px further in on every edge (left
  edge x ≈ 51 against 43; bottom y ≈ 684 against 690).
- **Envelope.** `beam-splash` peaks lower (30 against 41 on the right edge)
  and decays along the lap (24 at 3 s, 13 at 5 s, nothing measurable
  after 6 s); `beam-run`'s plateau holds 41 from the right edge to the
  left.
- **Spill.** `beam-nospill` follows `beam-run`'s path frame for frame; the
  difference is confined to the chamfer band and does not move the
  centroid. On `beam-run`'s bottom-left corner crop the chamfer takes a
  blue cast as the head rounds it; on `beam-nospill` and `beam-bevel0` it
  does not.
- **Flat slab.** `beam-bevel0`'s ring rides 8 px inside the slab edge
  (x ≈ 32; the bevel-10 sequences sit at 43), and the top edge shows no
  measurable difference because it lies under kitty's tab bar; the other
  three edges carry the head at the same peak.
- **The 2 s corner crop.** The plan's mid-pass crop is the focused pane's
  top-left corner at the frame nearest 2 s; by then the head has reached
  the top-right corner and the crop holds the tail's end over the resting
  ring. The head crops below are the supplement for judging the comet.

The peak difference of 41/255 is over kitty's `background_opacity 0.6`
terminal; along the top edge under the tab bar the head measures 10–14.
Whether that is enough presence at `ring-glow 1` is the owner's call.

### Owner review composites

Under `$NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-35e736b6/review/`, made
from the run's frames with ImageMagick (160 × 160 crops at 4×, point
filter; the full-frame pair at half size):

| File | Shows |
| --- | --- |
| `gap-8-vs-16-head.png` | the head on the right edge at 3 s, `beam-run` beside `beam-gap16` |
| `spill-vs-nospill-vs-bevel0-corner.png` | the bottom-left corner at 6 s as the head rounds it, `beam-run`, `beam-nospill`, `beam-bevel0` |
| `plateau-vs-splash-head.png` | the head at 3 s and 5 s, `beam-run` beside `beam-splash` |
| `pace-300-vs-900.png` | full frames at 1 s and 2.5 s, `beam-run` over `beam-fast` |

The gifs and the 8-across sheets remain the primary artifacts.

### Attempts of 2026-09-20

The morning's runs recorded no sheet. Runs under
`$NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-0219599f/`, logs
`$NIRI_MATERIAL_WORK_ROOT/ring-beam-clips-attempt*.log`:

| Run | Started | Outcome |
| --- | --- | --- |
| `ring-clips-87477-1789902750` | 07:12:30 | preflight refused: `gpu_util_pct 9.5 exceeds 5.0` |
| `ring-clips-94117-1789902947` | 07:15:47 | preflight `quiet`; release build and hash; stopped at the first scratch build by a script defect (below) |
| `ring-clips-134336-1789904049` | 07:34:09 | preflight refused: `gpu_pstate ['P5', 'P8'] not always P8`, `gpu_power_iqr_w 3.34 exceeds 1.0` |

Between and after those, seven launch windows over about twenty minutes
never saw the GPU idle for twenty consecutive seconds (utilization 8–43 %,
P5 samples, 13–19 W): the desktop session was in use. Each refused run
released its lock and recorded nothing past the refusal. The recording
above landed the next morning with the desktop logged out.

The script defect: `scratch_build` declared `local name=$1
src=$OUT/src-$name` in one statement, and bash expands `$name` before
`local` assigns it, so `set -u` aborted the run at the first scratch build
(`name: unbound variable`). Fixed in `ring-motion-clips.sh` by declaring
`src` on its own line; the script's syntax and the non-scratch path are
otherwise as recorded by `0219599f`.

## Deviations from the spec

- **`beam-run` bound.** The plan's Task 6 named "≥ 80 redraws during the
  run" for a single 1280×720 pane at 1200 px/s (run ≈ 4 s); the fixture
  commit `0219599f` moved the case to the two-column scene (run ≈ 2.7 s,
  passed as 3 s) with the bound `3 × 20 = 60`. The measured 146 clears the
  plan's figure as well.
- **Prism not merged.** `prism-1514d3` (`fe7b59f`) carries `beamSpeed`,
  `gap` and `glow`; it merges at the rollout (design §4), after the new
  niri is installed.
- **Rust signature.** `beam_perimeter(&Face, gap)` takes the face struct
  `ring::face` returns rather than the spec's `(face_half, face_radii,
  gap)`; the arithmetic is the spec's.
- No constant, key, default or shader line differs from the design.

## Owner acceptance: 2026-09-21

The owner read the `beam-run` gif and the four composites on 2026-09-21
and accepted every shipped default ("Looks good"). No constant moved, so
no sheet was re-recorded. The decisions, each with the value the branch
ships:

| Decision | Sheets to compare | Shipped default | Accepted |
| --- | --- | --- | --- |
| Gap from the face edge | `beam-run` (8) vs `beam-gap16` (16) | `ring-gap 8` | accepted |
| Edge spill onto the chamfer | `beam-run` (spill) vs `beam-nospill` (none) | `BEAM_SPILL 0.25` | accepted |
| Envelope shape | `beam-run` (plateau) vs `beam-splash` (splash) | `BEAM_ENVELOPE Plateau` | accepted |
| Pace | `beam-run` (300 px/s) vs `beam-fast` (900 px/s) | `ring-beam-speed 300` | accepted |
| Flat slab | `beam-bevel0`: the ring carries with no chamfer and no spill | (no constant; a fixture check) | accepted |
| Head, tail, glow | the corner crops of every sheet | `BEAM_HEAD_SIGMA 20`, `BEAM_TAIL_START 0.6`, `BEAM_TAIL_FRACTION 0.25` / `BEAM_TAIL_MAX 1200`, `BEAM_REST 0.2`, `BEAM_BASE 0.7`, `ring-glow 1` | accepted |

A constant moved later is changed in `ring.rs`, the shader, `niri-config`
defaults and Prism's defaults together, `just test` re-run, and the
affected sheet re-recorded.
