# Bounded focus and attention ring motion: execution evidence

**Status:** implemented on `material-82323e` (Task 1 `c0b2e151`, Task 2
`6504197d`, Task 3 `6f5f4f27`, fixtures `f2ffe8b3`); every redraw-count case
of the signals smoke passed on 2026-09-19 under the binary built from
`4aad1587`, and the four review clips were recorded the same day under the
release binary built from `079ed613` (below). The sweep duration
(`ring-sweep-ms 1500`) was read by the owner live (`57cd81e5`); the sweep
itself was replaced by the ring beam on 2026-09-21
([2026-09-19-ring-beam-evidence.md](2026-09-19-ring-beam-evidence.md)), so
this record is historical.

**Design:** [2026-09-18-ring-focus-motion-design.md](../specs/2026-09-18-ring-focus-motion-design.md).
**Plan:** `docs/plans/2026-09-18-ring-focus-motion.md`. **Task:**
`material-ee451d`, step 4 of `material-0e130e`.

## Host and binary

The verification host, headless lane: every nested instance runs under its own
`weston --backend=headless --renderer=gl` unit, never the desktop session.
Load average at launch 1.31. `wlrctl` (AUR) was installed for the
idle-resume input and is now a `cases` requirement of the smoke.

| Binary | Build | SHA-256 |
| --- | --- | --- |
| `material-signals-4aad1587/signals-977607-1789814990/niri` | `cargo build --release --features profile-with-tracy` at `4aad1587` | `0cb9c925654b306da116575f818ad5d783a4f0dc3cdba2f17d2bd644f3a345c7` |

## Redraw counts, 2026-09-19

Command, from `.worktrees/material-82323e`:

```bash
NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/material-signals-smoke.sh cases
```

Exit 0; log `$NIRI_MATERIAL_WORK_ROOT/ring-motion-cases.log`; rates in
`$NIRI_MATERIAL_WORK_ROOT/material-signals-4aad1587/signals-977607-1789814990/rates.txt`.
The nested `niri.log` holds no error or panic. Every fixture carries
`signal { idle-after-ms 0 }` except the two idle cases, so each case
measures what it measured before the gate existed. Counts are
`Niri::redraw` zones (exact name) in the final 20 s of a 30 s Tracy
capture unless a window is named. An earlier revision called them
`MaterialRenderElement::draw` zones; the smoke has always counted redraws
([correction](2026-09-30-hidden-window-attribution-evidence.md#correction-to-the-2026-09-18-evidence)).

| Case | Result | Expectation | Met |
| --- | --- | --- | --- |
| `quiet-ring` | 0 | 0 | yes |
| `focused-static` (`ring-sweep-ms 0`) | 0 | 0 | yes |
| `focused-settled` (`ring-sweep-ms 1500`, focused before the window) | 0 | 0 — settled focus costs nothing (design §2) | yes |
| `focused-reduced` (`motion "reduced"`) | 0 | 0 | yes |
| `focused-anim-off` (`animations { off }`) | 0 | 0 | yes |
| `focus-none-sweep` (`focus "none"`) | 0 | 0 | yes |
| `other-focused` (the sweep material on an unfocused window) | 0 | 0 | yes |
| `toggle-control` (no material, six focus changes) | 24 | recorded | — |
| `focus-none-toggle` | 26 total, 0 in the 10.8 s after | ≤ control + 6; 0 after | yes |
| `sweep-toggle` (six gains, each one lap) | 338 total, 0 in the 9.4 s after | total recorded; 0 after the last lap | yes |
| `demand-pulse` | 533 | 540 ± 15 % | yes |
| `demand-pulse-focused` | 0 | 0 | yes |
| `demand-breathe` | 160 | 160 ± 15 % | yes |
| `demand-flash` | 320 | 320 ± 15 % | yes |
| `idle-pulse` (`idle-after-ms 5000`, no input) | 0 | 0 — idle before the window (design §3) | yes |
| `idle-resume` (one `wlrctl pointer move 1 0` inside the instance at 12 s) | 115 in the 5 s after input; 0 after the gate re-engaged | 533 × 5/20 = 133 ± 25 % (100–166); 0 | yes |
| `dpms-off-pulse` (`power-off-monitors`) | 0 | 0 | yes |
| `ten-breathe` / `ten-flash` | 160 / 320 | within 10 % of one window | yes |
| `inactive-workspace`, `hidden-tab`, `offscreen-column` | 0 / 0 / 0 | 0 | yes |
| `motion-off`, `impulse-none` | 6 / 6 | ≤ 6, 0 after | yes |
| `reduced-flash` | 534 | pulse rate ± 15 % | yes |
| `attention-none` | 0 | 0 | yes |
| `done-pulse` | burst 87, after 0 | burst ≥ 30, after 0 | yes |
| `slowdown` | 533 | pulse rate ± 10 % | yes |

Reading the two new gated cases:

- `sweep-toggle`: 338 redraws over six 1.5 s laps is about 56 per lap, the
  llvmpipe host's animation-loop rate for a refresh-rate transition (the
  `done-pulse` impulse burst runs at the same rate: 87 in 3 s, of which
  1.5 s is the envelope). The quiet tail after the last lap is zero, so the
  sweep is finite and the settled ring reports no deadline and no damage.
- `idle-resume`: 115 redraws in the five seconds after the virtual-pointer
  motion is the pulse rate (26.6/s) resumed from the absolute clock, then
  zero once five further input-free seconds re-engaged the gate. The
  input entered `process_input_event` through the wlr virtual-pointer
  protocol, as any device would.

No case was skipped.

## Review clips, 2026-09-19

`docs/materials/scripts/ring-motion-clips.sh` records the four sequences of
the design's §7 under the capture protocol (`tools/capture-meta` preflight,
identity, settle before every launch, release), from a release build on
`sweep.kdl` (`ring-sweep-ms 1500`, `focus-ring { off }`, gaps 24, startup
hotkey overlay skipped) and `idle-5s.kdl` (the same with
`idle-after-ms 5000`). Each clip is a burst of full-frame screenshots as
fast as the async screenshot path allows (about 10/s here), every frame's
request instant in `frames.txt`, with a `.gif` and a `-sheet.png` for
review. Command, from `.worktrees/material-82323e`:

```bash
CAPTURE_TASK=material-ee451d NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/ring-motion-clips.sh
```

The recorded run is `$NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-079ed613/ring-clips-3278563-1789856906`
(the verification host, headless lane, started 2026-09-19 18:28:26 local; exit 0).
Release binary SHA-256
`0ef259b34eac9eecc7a08013429d95a36718c26d38b835838caeeacd19e34c02`.
Preflight `quiet`: load1 0.64, CPU 1.1 %, GPU P8 throughout, power IQR
0.12 W. Every settle `settled` with the GPU at P8 for all ten samples. The
nested `niri.log` holds only the per-frame "error showing screenshot
notification" warning (no notification service in the nested session).

| Clip | Sequence (design §7) | Frames | Span | Settle CPU / power IQR |
| --- | --- | --- | --- | --- |
| `gain-from-rest` | focus moves to a window at rest: one lap, then rest | 25 | 2.5 s | 1.0 % / 0.305 W |
| `alt-tab-three` | three focus changes 0.4 s apart: each gain starts from rest or is skipped mid-lap | 39 | 4 s | 1.3 % / 0.357 W |
| `loss-mid-lap` | focus leaves 0.5 s into a lap: the lap runs out on the unfocused window | 24 | 2.5 s | 1.1 % / 0.09 W |
| `idle-freeze` | `idle-after-ms 5000`, a pulsing window, no input: attention frozen | 30 | 3 s | 1.1 % / 0.418 W |
| `idle-resume` | one `wlrctl pointer move 1 0` inside the instance: attention resumes | 30 | 3 s | (same settle) |

SHA-256 of `capture.json` and the review gifs:

```text
ebdeb4a66f95b962508f7902af0741502ecd6795c2699ce65928a4915a767f57  capture.json
f1a6283ba5eeddcf23f6fa8ed5013612e540e09ad5a6106b968456b4177ad7cc  gain-from-rest.gif
9e208c28315ee3c653566b1e166e4042698df332cfa4522f63ce0a7b9f71c4c3  alt-tab-three.gif
1c51f746e3a12f53c00d86c7d0735cac5007461251e26306c1b5d384e071387c  loss-mid-lap.gif
1227c98a10a39cc52afbe5a5aa9630d9ba80e00f4179441c3a519cb02fb671db  idle-freeze.gif
ef196d53495beb671d89ac2a1efdaea4442275976af399ba105e34307e2255c2  idle-resume.gif
```

The clips are for the owner's judgment of the 1500 ms starting value and
the ease-out; `material-9306b5` tunes the ring's appearance against the
settled pattern the lap lands on.

### Runs before the recorded one

Preflight refused twice on 2026-09-19 morning on desktop load
(`ring-motion-clips-4aad1587/ring-clips-1242643-1789816273`, load1 5.43,
GPU P0/P5/P8, power IQR 5.11 W; `…/ring-clips-1284851-1789816649`, load1
6.65, IQR 8.265 W) and once in the evening at load1 2.12 against the 2.0
limit (`ring-motion-clips-079ed613/ring-clips-3254819-1789855169`), where
the whole load was one orphaned test process from another project; it was
stopped by hand and the sweep for such leftovers is filed as `ops-38be00`.

Two fixture defects surfaced on the first recording runs and are fixed in
the script that produced the recorded run:

- the nested niri showed its startup hotkey overlay over the middle of every
  frame (`ring-clips-3258158-1789855913`, `…-3267144-1789856387`,
  `…-3270108-1789856468` hold complete but obscured clips); the fixture now
  sets `hotkey-overlay { skip-at-startup; }`;
- Weston's GL renderer holds the GPU at P0, and after its unit stops the
  GPU steps down through P5 to P8 over about two seconds, so the next
  sequence's settle saw a P5 sample and refused (`…-3273560-1789856628`,
  and the third settle of the runs above); `stop_nested` now waits for the
  step-down before returning.

Each refused run released its lock and recorded nothing past the refusal.
