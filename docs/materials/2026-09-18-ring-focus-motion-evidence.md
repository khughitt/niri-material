# Bounded focus and attention ring motion: execution evidence

**Status:** implemented on `material-82323e` through `4aad1587` (Task 1
`c0b2e151`, Task 2 `6504197d`, Task 3 `6f5f4f27`, fixtures `f2ffe8b3`);
every redraw-count case of the signals smoke passed on 2026-09-19 under the
binary built from `4aad1587`. The four review clips are not yet recorded:
both clip runs were refused by the capture preflight on host load (below).
The sweep duration (`ring-sweep-ms 1500`) awaits those clips and the
owner's review; this record does not grade them.

**Design:** [2026-09-18-ring-focus-motion-design.md](../specs/2026-09-18-ring-focus-motion-design.md).
**Plan:** `docs/plans/2026-09-18-ring-focus-motion.md`. **Task:**
`material-ee451d`, step 4 of `material-0e130e`.

## Host and binary

Host `titan`, the headless lane: every nested instance runs under its own
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
`MaterialRenderElement::draw` zones in the final 20 s of a 30 s Tracy
capture unless a window is named.

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

## Review clips: preflight refused, 2026-09-19

`docs/materials/scripts/ring-motion-clips.sh` records the four sequences of
the design's §7 under the capture protocol (`tools/capture-meta` preflight,
identity, settle per launch, release), from a release build on `sweep.kdl`
(`ring-sweep-ms 1500`, `focus-ring { off }`, gaps 24) and `idle-5s.kdl`
(`idle-after-ms 5000`): `gain-from-rest`, `alt-tab-three`, `loss-mid-lap`,
`idle-freeze-resume`. Each clip is a burst of full-frame screenshots as fast
as the async screenshot path allows, with every frame's request instant in
`frames.txt` and a `.gif` and `-sheet.png` for review.

Two launches, unchanged thresholds, both **refused** at preflight before any
capture; the lock was released each time and nothing was recorded:

| Artifact under `$NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-4aad1587` | Start, local time | CPU | Load | P-states | Power IQR |
| --- | --- | --- | --- | --- | --- |
| `ring-clips-1242643-1789816273` | 2026-09-19 07:11:13 | 11.3% | 5.43 | P0/P5/P8 | 5.11 W |
| `ring-clips-1284851-1789816649` | 2026-09-19 07:17:29 | — | 6.65 | P5/P8 | 8.265 W |

SHA-256 of each retained `capture.json`, in table order:

```text
16ac8118c4192d05c0060e17dd9fe43dbf2cf209d02b661fec6172ac76b4fd99
ab1843e134417dffa9d7a637be5f4bca23925f51c756904f446e4b3ea53addf8
```

The first refusal followed the 25-minute smoke directly; the second came
after a five-minute wait during which the load average rose to 7.87 with the
GPU at P0, so the load is the desktop in use (graphics clients: Xwayland,
firefox, kitty, niri, noctalia, qs), not the smoke's tail. The redraw
counts above are not affected: the signals smoke does not preflight and its
counts are zone counts, not timings. The clip step is parked as quiet work;
its rerun is

```bash
CAPTURE_TASK=material-ee451d NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    docs/materials/scripts/ring-motion-clips.sh
```

and lands its clip directories and `capture.json` here when it runs. The
clips are for the owner's judgment of the 1500 ms starting value and the
ease-out; `material-9306b5` tunes the ring's appearance against the settled
pattern the lap lands on.
