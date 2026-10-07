# Sustained optic settling: acceptance evidence

**Status:** headless lane passed (pilot and matrix, 2026-10-02); owner accepted
the idle/resume clip and authorized local integration on 2026-10-02; rerun with
a real idle inhibitor on 2026-10-03, which passed; the dedicated real-TTY lane
(TTY resume, unlock, a real screencast consumer) passed on 2026-10-03; output
removal, which no capture host could drive, is covered by a deterministic
in-process test (2026-10-07).
**Task:** `material-2ee11e` under `material-f86183`; the idle-inhibitor rerun
is `material-80caf4`, the dedicated lane `material-f7eb0b` and
`material-3acc86`.
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

`output-removal` needs a second output, and the nested and dedicated lanes
each drive one. `tty-resume`, `unlock` and `screencast` passed in the
[dedicated lane](#dedicated-lane) on 2026-10-03.

The two-output capture (`material-1af3c6`) was dropped on 2026-10-07: no host
here has a second output, so it stays unverified on hardware. Its deterministic
coverage is `optic_settling_removing_one_of_two_outputs_keeps_the_shared_timeline`
in `src/tests/attention_idle.rs` (`material-f5b371`). It uses two headless outputs
with real calloop timers and an Aurora tile on the output that stays lit:

| Case | Assertion |
| --- | --- |
| Remove while active | The timeline keeps running on its anchor; the lit output re-arms its optic timer, which fires and redraws it. |
| Re-add while active | Still running on the same anchor; the lit output keeps its cadence. |
| Remove while idle | The timeline stays held; no output state remains for the removed output and no redraw is queued; the lit output's next pass arms no deadline and samples the held instant. |
| Re-add while idle | Still held at the same instant; neither output arms an optic deadline. |
| Resume | The timeline resumes from the held instant. |

Two mutations made it fail: dropping the optic timer's redraw, and pausing
the timeline in `Niri::remove_output`.

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

In the pilot, on the trace clock, the pause came at 26.09 s. `inhibited=1`
was traced 5 ms into the inhibit window (33.79 s) and `inhibited=0` 4 ms into
the release window (41.79 s). There was no resume edge, and 6.0 s of the held interval was
stimulus-free. The matrix repeated the verdict. The first development run
(`dev-20261003-inhibit`) traced both messages but was invalid: its stimuli
began 3.7 s after the pause and left no 5 s stimulus-free span, so the
schedule moved to the lane's 25 s start (`e6d3b3f8`).

## Dedicated lane

niri on DRM from a TTY with the desktop stopped
([design](../specs/2026-10-02-real-tty-settling-lane-design.md),
[plan](../plans/2026-10-02-real-tty-settling-lane.md)), through
`optic-settling-smoke.sh --lane dedicated`. The display was not dimmed
(`display-dim` is not installed).

| Item | Value |
| --- | --- |
| Source | `3125163e` |
| Binary | `--release --features profile-with-tracy`, sha256 `91b9a0c02b50b3961f925689b0cdf300b3f38d85ea396bdb89f1b5461ba09287` |
| Output | `DRM_OUTPUT=DP-1`, `DRM_MODE=3440x1440@59.999`, scale 1 |
| Lock client | `session-lock-client` sha256 `cd54292e…`, source `session-lock-client.c` `6dcde22e…` |
| Consumer | `screencast_consumer.py` sha256 `2c292520…`: GStreamer 1.28.7 `pipewiresrc ! glupload ! gldownload ! videoconvert ! appsink`, PipeWire 1.6.9 |
| Development | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-dev-20261003-2`, `CASES='drm-aurora tty-resume screencast'` |
| Pilot | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-pilot-20261003-1`, `analysis.json` sha256 `f1c4d177…`, `manifest.json` `8659ae13…` |
| Matrix | `$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-matrix-20261003-1`, gated on the pilot; `analysis.json` `db8053fc…`, `manifest.json` `c92cdd9b…` |

| Run | Duration | Passed | Unverified | Failed | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| Development | 4 min | 3 | 25 | 0 | `development-passed` |
| Pilot | 6 min | 4 | 25 | 0 | `lane-passed` |
| Matrix | 10 min | 8 | 25 | 0 | `lane-passed` |

The unverified cases are the headless lane's; the development subset does not run `unlock`. No run
logged a panic, and every `vt-restore.json` reads `not-needed`: niri
returned the VT itself.

| Case | Reading |
| --- | --- |
| drm-aurora | 18 redraws per 4.5 s active, one edge flush, 7.0 s quiet held, 10 redraws after resume. |
| tty-resume ×3 | Edges `[0, 1, 0]`, the resume inside `vt-return`. `vt-out` lasted 3.1 s, under the 6 s cap. The held interval drew once (the flush), and the field paused again 5 s after resume. |
| unlock ×3 | Edges `[0, 1, 0]`, the resume inside `unlock`. Of the held interval's three redraws, one is the edge flush and two fall inside the `lock` stimulus. |
| screencast | A real PipeWire consumer took three samples while held. The client region changed between samples 1 and 2 and held between 2 and 3, the Aurora crops stayed equal across all three, and no resume edge was raised. The 8 held redraws are the edge flush, three at cast start, one per sample and the collect frame. |

The first development run (`tty-dev-20261003-1`, at `54e30870`) was invalid.
`tracy-csvexport` leaves zone names unquoted, and the DRM renderer's
`MultiRenderer<'_, '_, '_>` zones put text into the time column. The
analyzer now folds those spilled fields back into the name (`3125163e`). The
headless runs had shifted only single-comma rows, filed under names the
analyzer never reads, so their verdicts stand. A copy of the first run's
artifacts re-analyzed as `development-passed` before the second run.

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
