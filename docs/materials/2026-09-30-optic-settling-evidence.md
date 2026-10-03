# Sustained optic settling: acceptance evidence

**Status:** headless lane passed (pilot and matrix, 2026-10-02); owner accepted
the idle/resume clip and authorized local integration on 2026-10-02;
five lifecycle cases remain unverified.
**Task:** `material-2ee11e` under `material-f86183`.
**Design:** [accepted spec, round 4](../specs/2026-09-29-sustained-optic-settling-design.md);
[plan, Task 5](../plans/2026-09-30-sustained-optic-settling.md).
**Review page:** <https://claude.ai/artifact/4b8iRwZA54qojFr3T3EEYU>.

## Identity

| Item | Value |
| --- | --- |
| Source | `2f3b5b6a` (whole-change review, fix wave and scoped re-review applied) |
| Binary | `--release --features profile-with-tracy`, sha256 `9c2cc1c18fd0055f054560ac9fd15e1921d8f18c46f073feb5146389cad6a8e3` |
| Host | nested niri under headless Weston 1280×720 (`headless-1`), from a TTY with the desktop stopped |
| Pilot | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/pilot-20261002-1`, `analysis.json` sha256 `461a79e0…`, `manifest.json` `9a201043…` |
| Matrix | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/matrix-20261002-1`, gated on the pilot; `analysis.json` `cf44d639…`, `manifest.json` `d324a1cb…` |

Each run directory carries `SHA256SUMS` over its traces, CSV exports,
screenshots and manifest, plus per-case `case.kdl`.

## Verdict

| Run | Duration | Passed | Unverified | Failed | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| Pilot | 25 min | 23 | 5 | 0 | `lane-passed` |
| Matrix | 39 min | 27 | 5 | 0 | `lane-passed` |

Neither run logged a panic or left a process behind. The largest gap between
a trace's end and its declared capture end was 0.88 s (pilot `vis-workspace`)
and 0.85 s (matrix `startup-no-input`), within the 1.5 s bound.

| Family | Cases | Reading |
| --- | --- | --- |
| Active, idle, resume | aurora-full ×3, aurora-reduced ×3 | Full: 18 redraws per 4.5 s active (4 Hz). Reduced: 9 (2 Hz). One edge-flush redraw, then zero redraws and material draws until resume; logical time equal at pause and resume. aurora-full held 601.9 s. |
| Combined motion | combined | Aurora with breathe: 35, then 52 redraws per 4.5 s active, the union of both schedules; one flush. |
| Gate policy | gate-off, amount-0, drift-0, motion-off, animations-off, startup-no-input, default-threshold | gate-off keeps 4 Hz (41 in 10 s). Amount 0, drift 0, motion off and animations off have no cadence. The 30 s default pauses on time (118 redraws in its 29.5 s active window). |
| Client damage | client-damage | Client commits keep drawing with Aurora held; no optic cadence resumes. |
| Finite and attention | impulse-idle, attention-cycle | Finite impulses play out while idle, then stop. Attention settles and resumes on its absolute clock. |
| Visibility | vis-workspace, vis-tab, vis-offscreen, vis-overview | Held-interval redraws are the journaled stimuli only. |
| Outputs and DPMS | dpms-ipc, dpms-input | IPC power-on while idle draws once and stays held; input resumes. |
| Session activation | screencopy | Screencopy draws held material with no redraw and no edge. |
| Reload and clock | reload-held, reload-lower, reload-disable | Lowering the threshold pauses at the reload; raising it or 0 resumes there. |

### Unverified

`tty-resume` (needs a real TTY session; the dedicated lane exists but has
not been run), `unlock` (wiring covered by Task 3's real-handler test),
`idle-inhibitor`, `output-removal` (needs a second output) and `screencast`
(needs a real screencast consumer; the `screencopy` case is not a substitute).
These stay open.

The dedicated real-TTY lane is implemented
([design](../specs/2026-10-02-real-tty-settling-lane-design.md)): `drm-aurora`,
`tty-resume`, `unlock` and `screencast` run on DP-1 from a TTY through
`optic-settling-smoke.sh --lane dedicated`. They stay unverified until its
development check, pilot and matrix pass.

`idle-inhibitor` now has a headless case (`material-80caf4`). A real
`zwp_idle_inhibit_manager_v1` client (`scripts/idle-inhibit-client.c`, built
per run and identified with its source) maps a window and inhibits while the
probe is held. Then a client line draws, and the inhibitor is released.
The case passes only with a single pause edge and no resume. Each of the
`inhibit` and `release` stimuli must carry niri's `IdleInhibit inhibited=1`
or `inhibited=0` trace message inside its own window. That proves the
inhibition took hold, and the absent resume edge proves it did not count as
input activity. It is unverified until a pilot runs it.

| Remaining acceptance | Task |
| --- | --- |
| Real TTY resume and unlock | `material-f7eb0b` |
| Real screencast consumer | `material-3acc86` |
| Real idle inhibitor | `material-80caf4` |
| Removal of one of two outputs | `material-1af3c6` |

All four follow-ups remain children of `material-f86183`, alongside the
deferred review minors in `material-285f81`; the goal is not complete.

On resume, the offline reducer reproduced both recorded verdicts, and all
277 pilot and 333 matrix artifact hashes matched `SHA256SUMS`.

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
