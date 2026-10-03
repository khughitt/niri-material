# Sustained optic settling: acceptance evidence

**Status:** headless lane passed (pilot and matrix, 2026-10-02); owner accepted
the idle/resume clip and authorized local integration on 2026-10-02; rerun with
a real idle inhibitor on 2026-10-03, which passed; four lifecycle cases remain
unverified.
**Task:** `material-2ee11e` under `material-f86183`; the idle-inhibitor rerun
is `material-80caf4`.
**Design:** [accepted spec, round 4](../specs/2026-09-29-sustained-optic-settling-design.md);
[plan, Task 5](../plans/2026-09-30-sustained-optic-settling.md).
**Review page:** <https://claude.ai/artifact/4b8iRwZA54qojFr3T3EEYU>.

## Identity

| Item | Value |
| --- | --- |
| Source | `e6d3b3f8`: `2f3b5b6a` plus niri's `IdleInhibit` trace message, the idle-inhibitor case and the mainline merged since |
| Binary | `--release --features profile-with-tracy`, sha256 `8947064b45610b2ae2280363fe646ce3bfc807e082830498549d4cf2441c7a7f` |
| Inhibitor client | `scripts/idle-inhibit-client.c` sha256 `ba47a8f34195…`, built per run; binary `b5d33f05bf47…` in both runs |
| Host | nested niri under headless Weston 1280×720 (`headless-1`), from a TTY with the desktop stopped |
| Pilot | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/pilot-20261003-1`, `analysis.json` sha256 `d737421e…`, `manifest.json` `0bd7e806…` |
| Matrix | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/matrix-20261003-1`, gated on the pilot; `analysis.json` `7c7bde80…`, `manifest.json` `9bee2017…` |
| Earlier runs | `pilot-20261002-1` (`analysis.json` `461a79e0…`) and `matrix-20261002-1` (`cf44d639…`) at `2f3b5b6a`, binary `9c2cc1c1…`, without the idle-inhibitor case |

Each run directory carries `SHA256SUMS` over its traces, CSV exports,
screenshots and manifest, plus per-case `case.kdl`.

## Verdict

| Run | Duration | Passed | Unverified | Failed | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| Pilot | 27 min | 24 | 4 | 0 | `lane-passed` |
| Matrix | 41 min | 28 | 4 | 0 | `lane-passed` |

Neither run logged a panic or left a process behind. The largest gap between
a trace's end and its declared capture end was 0.88 s (pilot) and 0.83 s
(matrix), both `startup-no-input`, within the 1.5 s bound. The 2026-10-02
runs at `2f3b5b6a` (25 and 39 min, 23 and 27 passed, five unverified) gave
the same readings for every other case.

| Family | Cases | Reading |
| --- | --- | --- |
| Active, idle, resume | aurora-full ×3, aurora-reduced ×3 | Full: 18 redraws per 4.5 s active (4 Hz). Reduced: 9 (2 Hz). One edge-flush redraw, then zero redraws and material draws until resume; logical time equal at pause and resume. aurora-full held 601.9 s. |
| Combined motion | combined | Aurora with breathe: 35, then 52 redraws per 4.5 s active, the union of both schedules; one flush. |
| Gate policy | gate-off, amount-0, drift-0, motion-off, animations-off, startup-no-input, default-threshold | gate-off keeps 4 Hz (41 in 10 s). Amount 0, drift 0, motion off and animations off have no cadence. The 30 s default pauses on time (118 redraws in its 29.5 s active window). |
| Client damage | client-damage | Client commits keep drawing with Aurora held; no optic cadence resumes. |
| Finite and attention | impulse-idle, attention-cycle | Finite impulses play out while idle, then stop. Attention settles and resumes on its absolute clock. |
| Visibility | vis-workspace, vis-tab, vis-offscreen, vis-overview | Held-interval redraws are the journaled stimuli only. |
| Outputs and DPMS | dpms-ipc, dpms-input | IPC power-on while idle draws once and stays held; input resumes. |
| Session activation | screencopy, idle-inhibitor | Screencopy draws held material with no redraw and no edge. A real inhibitor takes hold and releases while held with no resume edge, and a client line still draws. |
| Reload and clock | reload-held, reload-lower, reload-disable | Lowering the threshold pauses at the reload; raising it or 0 resumes there. |

### Unverified

`tty-resume` (needs a real TTY session; the dedicated lane is not
implemented), `unlock` (wiring covered by Task 3's real-handler test),
`output-removal` (needs a second output) and `screencast`
(needs a real screencast consumer; the `screencopy` case is not a substitute).
These stay open.

| Remaining acceptance | Task |
| --- | --- |
| Real TTY resume and unlock | `material-f7eb0b` |
| Real screencast consumer | `material-3acc86` |
| Removal of one of two outputs | `material-1af3c6` |

All three follow-ups remain children of `material-f86183`, alongside the
deferred review minors in `material-285f81`; the goal is not complete.

On resume, the offline reducer reproduced both recorded verdicts of the
2026-10-02 runs, and all 277 pilot and 333 matrix artifact hashes matched
`SHA256SUMS`.

### Idle inhibitor

A real `zwp_idle_inhibit_manager_v1` client (`scripts/idle-inhibit-client.c`,
built per run and identified with its source) maps a window and inhibits
while the probe is held. Then a client line draws, and the inhibitor is
released. The case passes only with a single pause edge and no resume. Each
of the `inhibit` and `release` stimuli must carry niri's `IdleInhibit
inhibited=1` or `inhibited=0` trace message inside its own window. That
proves the inhibition took hold, and the absent resume edge proves it did not
count as input activity.

In the pilot, on the trace clock, the pause came at 26.09 s. `inhibited=1` was traced 5 ms into
the inhibit window (33.79 s) and `inhibited=0` 4 ms into the release window
(41.79 s). There was no resume edge, and 6.0 s of the held interval was
stimulus-free. The matrix repeated the verdict. The first development run
(`dev-20261003-inhibit`) traced both messages but was invalid: its stimuli
began 3.7 s after the pause and left no 5 s stimulus-free span, so the
schedule moved to the lane's 25 s start (`e6d3b3f8`).

## Idle and resume clip

A probe kitty under the `aurora-full` glass with a 4 s threshold. Pointer
motion through 5.8 s, none until 19.7 s, screenshots about 0.55 s apart.
Changed pixels per step: about 45,000 per half second while active (each a
single 8-bit level: the 600 s drift period is far below what the eye sees),
exactly 0 from 10.3 s to 18.6 s, and 44,203 at the resume step, an
ordinary step with no catch-up. The clip, the step chart and the
changed-pixel masks are on the review page. The clip used the same binary
outside the capture protocol; it is visual evidence, not a measurement.
The owner accepted the held field and phase-continuous resume on 2026-10-02.

## Review before capture

The whole-change review of `5cf07746..645b9ef3` found three Important issues
in the capture tooling: a truncated trace could pass, TERM waited behind a
600 s foreground sleep, and the "screencast" case ran screencopy. These were fixed
in `2f3b5b6a`, together with a `lane-passed` verdict for a headless run, a
`niri.log` panic scan and artifact hashes, and re-reviewed clear. Its Rust
minors are deferred to `material-285f81`.

## Power

No watt saving is claimed. Zero redraws and material draws while held means
no compositor work for the field; the power effect is not measured here, and
for a playing video, the expected saving is negligible.
