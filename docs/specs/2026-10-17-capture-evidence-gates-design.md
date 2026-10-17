# Capture gates judge measured contention, not circumstantial host events

**Status:** implemented on `capture-gates-evidence` (plan
`docs/plans/2026-10-17-capture-evidence-gates.md`); retained-record replay in
§6.2. Approved by the owner for planning at spec review round 2 (2026-10-17),
after round 1 made `judge` keep refused preflights as failures and stop
reading an unscanned or unreleased record as a clean scan.
**Task:** `material-2e4ea5`, under `material-2834d7` (evidence instruments).
**Changes:** [capture protocol design](2026-09-11-material-capture-protocol-design.md) §2.4
(settle tolerance), [disturber hold design](2026-10-04-capture-disturber-hold-design.md) §4.3
and §5 (scan verdict and kinds), and
[host conditions design](2026-10-08-capture-host-conditions-design.md) §8.2 (pilot criteria).
**Related:** `material-04818d` (what selects the 10.4 W or 17 W idle floor),
`material-d2e552` (clock-state selection of draws).

## 1. Intent

On 2026-10-16, four quiet-host capture attempts were stopped by a gate whose
trigger did not touch the measurement:

| Run | Gate | Trigger |
| --- | --- | --- |
| `material-6901e0/cost-pilot-1792145107` | settle, `tilt10-a2` | `gpu_power_w` 10.685 against an 8.945 W baseline taken while the fb0-blanked display slept; it woke during the first sub-run |
| `noise-idle-ab-full-1792148041` | settle, `none-b3-1` | the idle floor stepped 10.425 → 17.115 W between sub-runs |
| `noise-idle-ab-full-1792148490` | settle, `none-b1-1` | the idle floor stepped back, 17.11 → 10.39 W |
| `material-6901e0/cost-pilot-1792145940` | release scan | `restart` of `getty@tty2.service`: a console login on another VT, with no CPU or GPU load |

In every one of the refused settle windows, the GPU was at P8 at 210 MHz,
utilization was 0, no GPU client was listed, and the power IQR was at most
0.29 W. Nothing on the host was using the GPU. The floor itself does not move
the measurement. `none-a-1` drew in a median of 0.287744 ms with the floor at
17.11 W (`…1792148490`) and at 10.465 W (`…1792148041`), and in 0.286720 ms in
the passing run (`noise-idle-ab-full-1792149757`), one 1024 ns GPU-timer tick
away.

The gates exist for real reasons, and their reasons must keep refusing:

- **2026-09-24, desktop up:** the idle lock raised the floor 9.9 → 11.585 W
  (`material-265eb0/trace-20260924T163317`, `B-resize-3`).
- **2026-09-24:** a desktop GPU blip put utilization at 5.0 against a 0.0
  baseline (`trace-20260924T165644`, `B-move-2`).
- **2026-09-24:** build-load tails refused preflights.
- **Timers that fired in a run:** the hold's positive control
  (`hold-pilot/pc-1`) and `systemd-tmpfiles-clean` in
  `noise-layers-pilot-1791337568`.

The 2026-09-24 idle-lock window and the 2026-10-16 floor-step windows look
the same in isolation. Both show P8, utilization 0 and a low IQR. What
separates them is a measured fact recorded in each window: on 2026-09-24 the
desktop's niri, noctalia and qs were GPU clients, and on 2026-10-16 no client
existed.

This design makes every gate decision one of two severities:

- **fail:** evidence of contention, or a change in a quantity the run claims.
- **annotate:** a recorded observation that is neither.

It records the severity and the rule beside each observation, so a reviewer
can re-judge a run from its `capture.json` alone.

Success means each of the following holds:

- Replayed through the new rules, each of the four 2026-10-16 refusals
  becomes an annotation.
- Every refusal listed above still fails.
- No settle window that passed in a retained record would fail.
- `capture-meta judge` gives the replay verdict for any retained run without
  touching the host or the record.

## 2. Principle

A gate fails a run on evidence that something competed with the measurement,
or that a quantity the run claims changed. It annotates anything else it
observes.

Evidence of contention is one of:

- a CPU, memory or GPU-utilization median outside its tolerance of the
  baseline;
- a P-state other than P8;
- a GPU client the lane forbids;
- a held disturber that ran;
- a held monitor that woke;
- on the nested lane, any change in GPU power while a GPU client exists that
  could have caused it.

Board power is claimed only by the dedicated lane (host conditions §3). On the
nested measurement lane (`headless`), GPU power is not a claim, and it is
evidence only where a contender exists.

## 3. Decisions

| # | Question | Decision | Rejected |
| --- | --- | --- | --- |
| 1 | How settle judges GPU power on `headless` | Against the baseline, as today, when any GPU client appears in the window. With no client in the window, a departure beyond `settle_gpu_power_w` is annotated (§4.1). | Judging every window by its own values only, with absolute limits: utilization ≤ 5 would pass the 2026-09-24 `B-move-2` blip (5.0 against 0.0). Comparing each window with the previous one: the 2026-09-24 idle-lock step is 1.59 W against its predecessor, 0.09 W from passing, and a slow drift on a desktop would never fail. Accepting the two known floors (10.4 and 17 W): host-specific numbers, and the fb0 case (8.9 W) is a third. |
| 2 | How the client test is made | From the window's own `gpu_clients` (graphics or compute, any sample), not from `host_condition`. | `host_condition`: records from before 2026-10-08 lack it, and the client list is the measurement itself. A desktop is always a graphics client in its windows (the desktop's niri, noctalia and qs on 2026-09-24). |
| 3 | GPU power on `dedicated` | Unchanged: it fails against the baseline. Board power is that lane's claim. | Relaxing it with `headless`: the idle-budget power lane measures exactly this floor. |
| 4 | A power-IQR check in settle windows | None is added. Each window's IQR is already recorded. A client-free window whose IQR exceeds the preflight's `gpu_power_iqr_w` is annotated. | Failing on IQR: two client-free windows that passed in retained records (`sheet-3`/`focused-k2-r0.6-h0-g1` 1.328 W, `render-order-within-pixels`/`within-aurora-light-ior-6` 1.318 W) would newly fail on the same physics as a floor step. |
| 5 | CPU, memory, utilization, P-state and clients in settle | Unchanged, against the baseline. | Absolute per-window limits: see 1. |
| 6 | VT events in the release scan | A `restart` of `getty@tty*.service` or `autovt@tty*.service` is annotated on `headless` and `pixels`, and fails on `dedicated`, where niri owns a VT and a login elsewhere means a VT switch. | Annotating it everywhere: a VT switch pauses a DRM session. Classifying by "no load": the scan has no load samples for the event's moment. |
| 7 | Other scan kinds | Unchanged: they fail on every lane. | — |
| 8 | Severities | Two are recorded: `fail` and `annotate`. Journal entries the scan does not flag stay unrecorded, as today. | A third, `ignore`, recorded but never shown: an annotation already does not fail, and an unrecorded event cannot be re-judged. |
| 9 | Re-judging | A read-only `capture-meta judge <run-dir>` re-applies the current rules to the record's windows and disturbances (§5). | Rewriting the record: `SHA256SUMS` covers `capture.json`, and the recorded severities are the history. |
| 10 | Thresholds and preflight | Unchanged. No threshold is added, and preflight's quietness judgement (which refused the 2026-09-24 build-load tails) is untouched. | — |

## 4. Record changes

The schema stays 1. Every addition is optional, and old records still validate
and render.

### 4.1 Settle

`judge_settled` returns two lists: reasons (`fail`) and annotations
(`annotate`). It takes the window's samples, the baseline, the thresholds and
the lane, as today.

On `headless`, when no sample in the window lists a graphics or compute client:

- **`unclaimed-power-step`:** `gpu_power_w` differs from the baseline by more
  than `settle_gpu_power_w`. Annotated, never a reason.
- **`unclaimed-power-spread`:** `gpu_power_iqr_w` exceeds `gpu_power_iqr_w`.
  Annotated.

Otherwise, the power rule is today's and fails. Every other rule is today's
and fails.

Each entry gains `gpu_power_step_w`: its `gpu_power_w` minus that of the
previous `settled` entry in the run, or minus the baseline's for the first.
This is the per-sub-run floor record the task asks for, and it is recorded
whatever the verdict.

An entry with annotations and no reasons is `settled`, as today, so
`release`'s renderer coverage and idle-budget's `scene_evidence` read it
unchanged:

```json
{"name": "none-b3-1", "verdict": "settled", "gpu_power_w": 17.115, "gpu_power_step_w": 6.69,
 "annotations": [{"rule": "unclaimed-power-step",
                  "detail": "gpu_power_w 17.115 is 6.725 above baseline 10.39; no GPU client in the window"}]}
```

`settle` prints each annotation to stderr and exits 0.

### 4.2 Release scan

`scan` is unchanged in what it flags. `classify(disturbances, lane)` then
gives each entry `severity` and `rule`:

| Kind | Unit | `headless`, `pixels` | `dedicated` |
| --- | --- | --- | --- |
| `restart` | `getty@tty*.service`, `autovt@tty*.service` | annotate (`vt-login`) | fail (`vt-login`) |
| any other | any | fail (`<kind>`) | fail (`<kind>`) |

`scan.verdict` becomes:

- `clean`: no entries;
- `annotated`: every entry is `annotate`;
- `disturbed`: any entry is `fail`.

`unscanned` and `not-run` are unchanged. `exit_for` treats `annotated` as
`clean`, exiting 0 after printing the annotations to stderr. `SCAN_VERDICTS`
gains `annotated`.

The lane comes from `capture.json`'s `run.lane`. Every path that writes
`scan` has the record: release, and the guard's or a person's `restore`
(hold design §4.4). A scan written with no readable `run.lane` classifies
every entry as `fail`, which is today's behaviour.

### 4.3 Rules identity

`preflight` gains `rules: "2026-10-17"`, naming this design. A record without
it was judged by the earlier rules, where every flagged scan entry and every
out-of-tolerance window failed. `judge` prints both identities.

### 4.4 `show`

`show` prints each sub-run's `gpu_power_step_w` and its annotations, and each
scan entry with its severity. The hold line reads `annotated` with a count.

## 5. `capture-meta judge`

```
tools/capture-meta judge <run-dir>
```

`judge` reads `capture.json` and changes nothing: it takes no lock, holds
nothing and writes no file. It judges three parts of the record, each with
one of three outcomes: **pass**, **fail**, or **no evidence**.

1. **Preflight.** Its rules are unchanged, so `judge` keeps the recorded
   verdict:
   - `quiet` or `unsampled`: pass;
   - `refused`: fail (rule `preflight`), printing the recorded reasons;
   - a missing `preflight` section, or any other verdict: no evidence.

   A refused preflight usually has no sub-runs and no scan. On 2026-10-17, 29
   retained records are refused preflights: 27 from before the hold, and 2
   rolled back (`not-run`).
2. **Settle windows.** For every sub-run entry with samples (`settled` or
   `refused`), `judge` rebuilds the window summary from the recorded medians,
   IQR, P-states and clients. It applies the current settle rules against the
   recorded baseline and thresholds. Thresholds missing from older records
   take today's defaults, and the output says so. A recorded threshold that
   settle could never apply makes this part no evidence, naming the
   threshold: the windows are not re-judged. Examples are a `gpu_pstate`
   other than `P<n>`, such as the `any (user-authorized pixel-only run)` and
   `P5|P8 (user-authorized pixel-only run)` of 12 render-order pixel runs, or
   a non-numeric limit. Re-judging those windows against today's literal
   P-state rule would newly refuse 125 windows that settled under the policy
   the run recorded.

   Sample-level rules need the per-sample memory floor, which the record does
   not keep. For those, `judge` uses the recorded verdict, and says so when a
   recorded reason names the floor.

   A record with no sub-runs passes this part.
3. **Scan.** The scan's state decides how it is judged, before any
   disturbance is classified, because an empty disturbance list means "none
   found" only in a scan that ran:

   | Record state | Outcome |
   | --- | --- |
   | No `hold` section (a record from before the disturber hold, 2026-10-04; 97 quiet and 27 refused on 2026-10-17) | Pass. The output says "no hold: predates the disturber hold", so the absence is never read as a clean scan. |
   | `hold`, but no `hold_end.scan` (never released) | No evidence |
   | `hold_end.restore.state: failed` | No evidence, as `exit_for` does |
   | `scan.verdict: unscanned` | No evidence |
   | `scan.verdict: not-run` (a preflight rollback) | Fail when `preflight_exit` is 1; no evidence when it is 2 |
   | `scan.verdict` `clean`, `annotated` or `disturbed` | Each entry in `disturbances` is classified by the current `classify`: fail when any entry is `fail`, otherwise pass |

`judge` prints one line for each item whose outcome differs from the recorded
one. It then prints a summary: each part's outcome, the recorded verdicts, the
current verdicts and both rule identities.

| Exit | When |
| --- | --- |
| 2 | The record is unreadable, or any part has no evidence |
| 1 | Otherwise, when any part fails |
| 0 | Every part passes |

No evidence outranks a failure. A run that cannot be judged is never reported
as merely failed, and a failure never hides a missing scan.

A refused settle stopped its fixture. `judge` exiting 0 on such a run means
the refusal would now be an annotation. It does not mean the run is complete
evidence.

## 6. Validation

### 6.1 Offline tests

In `tools/test_capture_meta.py`, each case is built from the recorded values
of the run it is named after. The values are copied into the test, so the
suite does not read the work root.

| Case | Window or event | Expected |
| --- | --- | --- |
| `noise-idle-ab-full-1792148041` `none-b3-1` | 17.115 W against 10.39, IQR 0.027, P8, util 0, no clients, `headless` | `settled` with `unclaimed-power-step` |
| `noise-idle-ab-full-1792148490` `none-b1-1` | 10.39 W against 17.125, IQR 0.285, no clients | `settled` with `unclaimed-power-step` |
| `cost-pilot-1792145107` `tilt10-a2` | 10.685 W against 8.945, no clients | `settled` with `unclaimed-power-step` |
| `cost-pilot-1792145940` scan | `restart getty@tty2.service`, system, `headless` | `annotated`, `release` exits 0 |
| The same event on `dedicated` | — | `disturbed`, exit 1 |
| `trace-20260924T163317` `B-resize-3` | 11.585 W against 9.9, clients niri, noctalia, qs, kitty, Xwayland | `refused` on `gpu_power_w` |
| `trace-20260924T165644` `B-move-2` | util 5.0 against 0.0 | `refused` on `gpu_util_pct` |
| `power-full-20260927T030030` `sham-1-3` | P5 and P8, `dedicated` | `refused` on `gpu_pstate` |
| `dedicated` window | 17 W against 10.4, no clients | `refused` on `gpu_power_w` |
| `hold-pilot/pc-1` scan | `timer-added` and `timer-fired` | `disturbed` |
| `noise-layers-pilot-1791337568` scan | `timer-fired systemd-tmpfiles-clean.service` | `disturbed` |
| Client-free window, IQR 1.328 | `headless` | `settled` with `unclaimed-power-spread` |
| Scan with no readable lane | `restart getty@tty2.service` | `disturbed` |
| `autovt@tty3.service` restart, `pixels` | — | `annotated` |
| `getty-static.service` restart, `headless` | not a VT unit | `disturbed` |

`judge` is tested on these records:

| Record | Exit |
| --- | --- |
| A pre-2026-10-17 record with a refused power window (`noise-idle-ab-full-1792148041`); it reports the change | 0 |
| The 2026-09-24 record (`trace-20260924T163317`) | 1 |
| A refused preflight with no sub-runs and no hold, from `render-order-readiness.pCbsOC` (CPU 16.6 %, load 5.53, utilization 26 %, P3/P5/P8, a compute client) | 1 |
| A `not-run` rollback with `preflight_exit` 1 | 1 |
| A `not-run` rollback with `preflight_exit` 2 | 2 |
| A quiet preflight and settled windows with `scan.verdict: unscanned` and an empty `disturbances` list | 2 |
| A `hold` section with no `hold_end` | 2 |
| `hold_end.restore.state: failed` with a clean scan | 2 |
| A quiet record from before the hold, with settled windows and no `hold`; the output names the missing hold | 0 |
| An unscanned record that also has a refused settle window (no evidence outranks a failure) | 2 |
| No `preflight` section | 2 |

In every case, the record's bytes and mtime are unchanged afterwards.

`gpu_power_step_w` is checked against the previous settled entry, skipping a
refused one, and against the baseline for the first entry. `show` renders
annotations and severities, and old records still render.

### 6.2 Retained-record replay

After implementation, run `capture-meta judge` over every `capture.json`
under `$NIRI_MATERIAL_WORK_ROOT`. This is offline and needs no host. The
replay records:

- each of the four 2026-10-16 runs exits 0;
- every refusal listed in §1 still fails;
- every refused preflight exits 1, or 2 where its rollback could not run;
- every record judged as no evidence is listed with its reason;
- no entry recorded `settled` is `fail` under the current rules. The rules
  only relax, so this holds by construction, and the replay checks it.

The result goes in a `tasks note` on `material-2e4ea5` and is appended to
this section. On 2026-10-17, the retained set holds 1330 settled windows.

**Replay, 2026-10-17:** `capture-meta judge` over the 162 retained records
exits 0 for 104, 1 for 47 and 2 for 11.

- The four 2026-10-16 runs exit 0: `material-6901e0/cost-pilot-1792145107`,
  `material-6901e0/cost-pilot-1792145940` (its scan is now `annotated`),
  `noise-idle-ab-full-1792148041` and `noise-idle-ab-full-1792148490`.
- The refusals of §1 still fail (exit 1):
  `material-265eb0/trace-20260924T163317` and `trace-20260924T165644`
  (settle), `material-265eb0/power-full-20260927T030030` (settle, P-state),
  `hold-pilot/pc-1` and `noise-layers-pilot-1791337568` (scan), and
  `render-order-readiness.pCbsOC` (preflight).
- Of the 47 exit-1 records, 29 are refused preflights (every refused
  preflight in the set; 2 of them, `noise-layers-ab-stream-1791430290` and
  `noise-layers-preflight-1791285645`, are `not-run` rollbacks with
  `preflight_exit` 1), 16 hold a settle refusal that still refuses, and 2
  are the scan failures above.
- The 11 exit-2 records are the render-order pixel runs whose recorded
  `gpu_pstate` is a policy settle never applies, so their windows are not
  re-judged: `render-order-aurora-depth200.MZzHGt/run`,
  `render-order-aurora-depth.WzKHWH/run`, `render-order-behind-pixels.tyHcpH`,
  `render-order-behind-pixels.Z6RecW`, `render-order-behind-red.064YZ3`,
  `render-order-behind-red.qucX7K`, `render-order-old-ring.FH88OR/evidence/run`,
  `render-order-old-ring.OoNe3c/evidence/run`,
  `render-order-within-pixels.eS4KzB/run`, `render-order-within-pixels.V12xVh/run`
  and `render-order-within-resume.C2DA1f/run`. No record is unreadable, never
  released or unscanned. `render-order-within-pixels.v8Dp2y/run` records the
  same policy but no sampled window; it is a refused preflight and exits 1.
- 124 records predate the disturber hold, and their output says so.
- No recorded `settled` window fails.

## 7. Documents this changes

- **Protocol design §2.4:** the settle power tolerance applies on `dedicated`,
  and on `headless` when a GPU client is in the window. A pointer to here.
- **Hold design §4.3 and §5:** the `annotated` verdict, `severity` and `rule`
  per entry, and the VT rule. A pointer to here.
- **Host conditions §8.2:** "no disturbance is recorded" becomes "no
  disturbance of severity `fail`". A floor step annotated in a pilot run is
  reported in its `run:` note.
- **`docs/materials/capture-host-setup.md`:** where it describes the scan's
  verdicts or the settle check.
- **`material-04818d`:** its done-criterion allows "the gate change is
  reviewed". On landing, a note there says this design is that change. The
  floor's cause stays its question.

## 8. Out of scope

- **What selects the floor** (`material-04818d`) and which clock state a draw
  ran in (`material-d2e552`).
- **Sampling within a sub-run.** It is still unsampled (host conditions §9).
  The A/B alternation and the repeat check remain what measures noise inside
  a measurement.
- **Fixture-side analysis by floor.** `gpu_power_step_w` makes it possible to
  stratify medians by floor. No fixture is changed to do so here.
- **Threshold changes:** none.
