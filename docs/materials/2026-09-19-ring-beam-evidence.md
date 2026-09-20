# The ring beam: execution evidence

**Status:** implemented on `material-2c3984` (`ring.rs` `5610d666`, config
and compositor `98013739`, fixtures `0a12a7bd`, `0219599f`); every
redraw-count case of the signals smoke passed on 2026-09-20 under the binary
built from `0219599f`. The six review sheets of the design's §6 are **not
yet recorded**: the capture preflight refused every attempt of 2026-09-20
morning on desktop activity (below), and the release binary they will use
is built and hashed. The constants judged on the sheets (`ring-gap`, spill,
envelope, pace, bevel 0) await the owner's acceptance; this record does not
grade them. **Owner acceptance: pending** (last section).

**Design:** [2026-09-19-ring-beam-design.md](../specs/2026-09-19-ring-beam-design.md).
**Plan:** `docs/plans/2026-09-19-ring-beam.md`. **Task:** `material-912dab`,
step 6 of `material-2c3984`; `material-9306b5` (ring appearance) is closed as
subsumed when the sheets are accepted.

## Host and binary

Host `titan`, the headless lane: every nested instance runs under its own
`weston --backend=headless --renderer=gl` unit, never the desktop session.
Worktree HEAD `0219599f9f30d2d5994bb49d9a35d558ec94e365`. The smoke and
the sheets ran one after the other, never concurrently. No orphaned test
process was found before either run (the only `kitty` processes were the
desktop's own, on `wayland-1`).

| Binary | Build | SHA-256 |
| --- | --- | --- |
| `material-signals-0219599f/signals-11660-1789901391/niri` | `cargo build --release --features profile-with-tracy` at `0219599f` | `e1f73dd59fd51ce201ca4beb8cb2c84c0b43bec3e213d2800509e9260832f09d` |
| `ring-motion-clips-0219599f/ring-clips-94117-1789902947/niri` | `cargo build --release` at `0219599f` (the sheets' release binary; that run stopped before recording, see below) | `fcb57713f67519abad3be96ecc46a078da741e18bbdcafa8e28ebf3b4d6f35a9` |
| `…/niri-nospill` | the same tree with `BEAM_SPILL` 0.25 → 0.00 in `ring.rs` and `main.frag` | (built by the recording run) |
| `…/niri-splash` | the same tree with `BEAM_ENVELOPE` Plateau → Splash in `ring.rs` | (built by the recording run) |

## Spec contracts checked against the implementation

Two contracts the design settled before execution were re-read against the
code at `0219599f` for this record; both hold.

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

## Redraw counts, 2026-09-20

Command, from `.worktrees/material-2c3984`:

```bash
NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/material-signals-smoke.sh cases
```

Exit 0; log `$NIRI_MATERIAL_WORK_ROOT/ring-beam-cases.log`; rates in
`$NIRI_MATERIAL_WORK_ROOT/material-signals-0219599f/signals-11660-1789901391/rates.txt`.
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
| `toggle-control` (no material, six focus changes) | 24 | recorded | — |
| `focus-none-toggle` | 20 total, 0 in the 10.9 s after | ≤ control + 6; 0 after | yes |
| `beam-run` (one gain, two-column scene, `ring-beam-speed 1200`) | 146 during the 3 s run, 0 in the 12 s after | ≥ 60 during (3 × 20); 0 after | yes |
| `demand-pulse` | 533 | 540 ± 15 % | yes |
| `demand-pulse-focused` | 0 | 0 | yes |
| `demand-breathe` | 160 | 160 ± 15 % | yes |
| `demand-flash` | 320 | 320 ± 15 % | yes |
| `idle-pulse` (`idle-after-ms 5000`, no input) | 0 | 0 | yes |
| `idle-resume` (one `wlrctl pointer move 1 0` at 12 s) | 115 in the 5 s after input; 0 after the gate re-engaged | 533 × 5/20 = 133 ± 25 % (100–166); 0 | yes |
| `dpms-off-pulse` (`power-off-monitors`) | 0 | 0 | yes |
| `ten-breathe` / `ten-flash` | 160 / 319 | within 10 % of one window (160 / 320) | yes |
| `inactive-workspace`, `hidden-tab`, `offscreen-column` | 0 / 0 / 0 | 0 | yes |
| `motion-off`, `impulse-none` | 6 / 6 | ≤ 6, 0 after | yes |
| `reduced-flash` | 533 | pulse rate ± 15 % | yes |
| `attention-none` | 0 | 0 | yes |
| `done-pulse` | burst 93, after 0 | burst ≥ 30, after 0 | yes |
| `slowdown` | 533 | pulse rate ± 10 % | yes |

Reading the beam cases:

- `beam-run`: 146 redraws in the 3.5 s window around a run of about 2.7 s
  is the llvmpipe host's animation-loop rate (`done-pulse`'s impulse burst
  runs at the same rate: 93 in 3 s), and the 12 s after the run are zero:
  the beam is finite, the tail drain ends on the frame that sees the head
  past `P + L`, and the settled ring reports no deadline and no damage. The
  brief's figure for this case was "≥ 80"; the fixture's own bound is
  `3 × 20 = 60`, and 146 clears both.
- `focused-settled` and `beam-off` are 0: a focused window at rest costs
  nothing whether the beam ran and finished or never ran.
- The unchanged attention cases repeat the 2026-09-19 numbers to the
  frame (`demand-pulse` 533, `idle-resume` 115, `reduced-flash` 533,
  `slowdown` 533), so the beam changed nothing outside focus.

No case was skipped.

### The first attempt

The first run of the smoke (`signals-4172611-1789900736`, log
`ring-beam-cases-attempt1.log`) failed at `demand-pulse-focused` with 7
redraws in the steady window, after `beam-run` had already passed there
with 143 during and 0 after. The seven redraws are one 120 ms burst at
19.5 s of the trace, preceded by 180 `CompositorHandler::commit` zones and
followed by nothing; the host's `kitty.conf` was written at that instant
(its mtime is 0.6 s before the burst), and kitty reloads and repaints on a
config write, in the nested instance as on the desktop. A client repaint
storm, not a material redraw: the case has no beam (`ring-beam-speed 0`)
and no material zone in the trace. The rerun above is clean end to end.

## Review sheets (pending a quiet host)

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

### Attempts of 2026-09-20

No sheet is recorded yet. Runs under
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
released its lock and recorded nothing past the refusal. The recording is
owed on a quiet host with the command above; the table, hashes and settle
figures of the sheets go here when it lands.

The script defect: `scratch_build` declared `local name=$1
src=$OUT/src-$name` in one statement, and bash expands `$name` before
`local` assigns it, so `set -u` aborted the run at the first scratch build
(`name: unbound variable`). Fixed in `ring-motion-clips.sh` by declaring
`src` on its own line; the script's syntax and the non-scratch path are
otherwise as recorded by `0219599f`.

## Deviations from the spec

- **Sheets not recorded.** The design's §6 gate — the six sheets and the
  owner's judgment — is open; this record carries the smoke and the
  contract checks only until they land.
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

## Owner acceptance: pending

The sheets above are for the owner's judgment; nothing here is graded. The
decisions, each with the value the branch ships today:

| Decision | Sheets to compare | Shipped default | Accepted |
| --- | --- | --- | --- |
| Gap from the face edge | `beam-run` (8) vs `beam-gap16` (16) | `ring-gap 8` | pending |
| Edge spill onto the chamfer | `beam-run` (spill) vs `beam-nospill` (none) | `BEAM_SPILL 0.25` | pending |
| Envelope shape | `beam-run` (plateau) vs `beam-splash` (splash) | `BEAM_ENVELOPE Plateau` | pending |
| Pace | `beam-run` (300 px/s) vs `beam-fast` (900 px/s) | `ring-beam-speed 300` | pending |
| Flat slab | `beam-bevel0`: the ring carries with no chamfer and no spill | (no constant; a fixture check) | pending |
| Head, tail, glow | the corner crops of every sheet | `BEAM_HEAD_SIGMA 20`, `BEAM_TAIL_START 0.6`, `BEAM_TAIL_FRACTION 0.25` / `BEAM_TAIL_MAX 1200`, `BEAM_REST 0.2`, `BEAM_BASE 0.7`, `ring-glow 1` | pending |

Any constant the owner moves is changed in `ring.rs`, the shader,
`niri-config` defaults and Prism's defaults together, `just test` re-run, and
the affected sheet re-recorded before this section is filled in.
