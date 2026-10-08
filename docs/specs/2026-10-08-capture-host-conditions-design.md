# Per-route host conditions for capture evidence

**Status:** draft for owner review (2026-10-08).
**Task:** `material-18c2a1`, under `material-2834d7` (evidence instruments).
**Extends:** [capture protocol design](2026-09-11-material-capture-protocol-design.md)
(`tools/capture-meta`) and [disturber hold design](2026-10-04-capture-disturber-hold-design.md).
**Brief:** [capture lanes and lifecycle](../notes/2026-10-06-capture-lifecycle-brief.md#renderer-aware-capture).

## 1. Intent

Every capture fixture in this repository gates on the same thing: a GPU-quiet
`preflight --lane headless`, then a GPU-quiet `settle` before every nested
launch. That gate exists for timing and power evidence. It is applied as well to
fixtures whose evidence cannot be changed by host load. `glass-edge-sheet.sh`
settles 77 times to produce a contact sheet the owner judges by eye.
`glass-noise-layers-smoke.sh` settles once per cell, and every cell it asserts
is a pixel equality. Both therefore wait in the quiet queue for an idle machine
that their evidence does not need.

The opposite risk exists too. A nested measurement trusts that the GPU it
sampled is the GPU that rendered. Only `hidden-window-attribution.sh` checks
the nested compositor's renderer. Every other measured fixture would accept a
Tracy timing from a nested niri that fell back to a software renderer, and
would label it with NVIDIA telemetry.

The idle-budget evidence also records three desktop attempts that each lost a
settle to a different transient. Those attempts predate the disturber hold
(2026-10-04), which powers the monitors off, inhibits idle and holds user
timers for the length of a run. No nested measurement with per-case settles has
been attempted with the desktop up since. Whether the desktop can host
measurements is therefore open, not settled against.

This design names the evidence class of every capture route. For each class it
states what the route may claim, which host condition its preflight and settles
require, and how a mismatch fails before anything is measured. The hardware
thresholds stay as they are.

Success:

- Every capture fixture in this repository is assigned to one route.
- A pixel route runs on a desktop in use and records no GPU telemetry.
- A measurement route refuses a renderer that is not the sampled GPU.
- A route's host-condition mismatch refuses before the hold changes anything on
  the host.
- The quiet queue's `--needs` for each route follows from this document.
- Whether nested measurements may run with the desktop idle is decided by a
  pilot whose verdict criteria are written here in advance.

## 2. Terms

**Route:** a fixture's evidence class together with its topology. There are four
(§3). `capture-meta`'s `--lane` argument names the route.

**Host condition:** the state of the host session during a run. Three exist:

| Condition | Meaning | Quiet-queue `--needs` |
| --- | --- | --- |
| `in-use` | The desktop is up and someone may be working on it. Other sessions may run builds. | none (not a quiet step) |
| `desktop-idle` | The desktop is up but untouched. The hold has powered the monitors off and held idle and user timers; no other session runs work. | `idle` |
| `tty` | The ordinary desktop session is stopped. The run starts from a TTY login. | `headless` |

The capture lane named `headless` (niri nested under headless Weston) and the
quiet-queue condition `headless` (desktop stopped) are unrelated meanings of one
word. This document writes `tty` for the condition and `nested` for the
topology. It leaves both existing spellings in place (§4, decision 6).

## 3. Routes

| Route | `--lane` | Evidence it may claim | Claims it may not make | Host condition |
| --- | --- | --- | --- | --- |
| Frozen pixels | none (no capture-meta) | Pixels at a frozen instant from the in-process headless renderer | Anything about time, cost or a real GPU | any |
| Nested pixels | `pixels` (new) | Pixels from a static scene: equality and AE between shots in one run, and sheets for the owner | Draw times, redraw counts or rates, fps, wall-clock motion | `in-use` or stricter |
| Nested measurements | `headless` (unchanged) | Draw-time medians, redraw and draw counts and rates, cadence, fps, and pixels sampled at wall-clock instants | Board power | `tty`; `desktop-idle` after the §8.2 pilot |
| Dedicated | `dedicated` (unchanged) | DRM timing, VT resume, unlock, screencast, board power | — | `tty` only |

**Classification rule.** A value belongs to a measurement route if it could
change when the host is slower or the GPU is shared. That covers anything a
fixture asserts or records, including a value marked "recorded only". A fixture
that records even one such value is a measurement fixture, whatever else it
asserts. A static scene shot twice is a pixel value. So is a sheet of static
cells. A frame grabbed after a `sleep` while an animation runs is a measurement:
under contention, the frame shown at that instant changes.

### 3.1 Current fixtures

| Fixture | Route | Why |
| --- | --- | --- |
| `src/tests/ring_pair.rs`, `ring_look.rs`, `glass_edge.rs`, `accent_tint.rs`, `noise_layers.rs`, `noise_site.rs` | Frozen pixels | In-process, frozen clock; unchanged |
| `glass-edge-sheet.sh` | Nested pixels | Static cells after a fixed wait; PASS is a clean finish; the owner judges the sheet |
| `glass-noise-layers-smoke.sh` | Nested pixels | Every assertion is a pixel statistic of a static cell, each shot twice to AE 0 |
| `glass-iridescence-smoke.sh` | Nested measurements | Records GPU draw medians |
| `glass-render-order-smoke.sh` | Nested measurements | Records GPU draw medians and timed `aurora-motion` shots |
| `glass-aurora-smoke.sh` | Nested measurements | Redraw-rate assertions; time-bucketed shots |
| `glass-noise-site-smoke.sh` | Nested measurements | Ring cells wait a computed comet lap of wall time |
| `noise-layers-cost.sh`, `noise-placement-cost.sh` | Nested measurements | Draw-time medians |
| `hidden-window-attribution.sh` | Nested measurements | Redraw counts and fps thresholds |
| `optic-settling-smoke.sh --lane headless` | Nested measurements | Cadence against an absolute schedule |
| `focus-swap-clips.sh`, `drag-lag-clips.sh`, `ring-motion-clips.sh` | Nested measurements | Frames are taken at wall-clock instants |
| `optic-settling-smoke.sh --lane dedicated` | Dedicated | DRM timing, resume, unlock, cast |
| idle-budget `power` (niri-experiments) | Dedicated | Board power |
| idle-budget `trace`, `jelly-motion.sh` (niri-experiments) | Nested measurements | Redraw counts and timing |

Four mixed fixtures assert pixels and also record a timing: iridescence,
render-order, aurora and noise-site. Each could gain a pixels-only mode that
drops the timing and runs on the pixel route. That split is a follow-up for
each fixture, taken when the fixture is next needed (§10). The three clip
fixtures are measurements as they stand. Their visual purpose belongs to
frozen-clock fixtures, as the
[workstreams brief](../notes/2026-10-02-workstreams-brief.md) already routes it.

## 4. Decisions

| # | Question | Decision | Rejected |
| --- | --- | --- | --- |
| 1 | How a fixture names its route | One new value for the existing `--lane`: `pixels`. `headless` and `dedicated` keep their meaning. | A separate `--evidence` flag. It would change every caller, including two fixtures in niri-experiments, and only three of the six lane/evidence pairs are valid. |
| 2 | Does a pixel run take the capture lock? | Yes, the same exclusive lock. A pixel run that overlapped a measurement would land inside that measurement's settles. Pixel runs are short. | A shared reader lock for pixel runs: it adds a lock mode, and there is nothing to share, since two nested-pixel runs can each run quickly in turn. |
| 3 | What a pixel run holds | User timers and the declared services. Idle and the monitors are left alone. | The full hold: it powers off the monitors of a desktop in use. No hold: on 2026-09-24, `wali-rotate.timer`'s Prism apply stopped a nested kitty from mapping, which is a disturbance to pixels too. |
| 4 | Do pixel runs sample the GPU? | No. Their records hold no `baseline` and no GPU fields, and their sub-runs are opened with a new `begin` verb, not `settle` (§6.2). | Sampling, recorded but not judged: the record would carry numbers no check reads. On a non-NVIDIA host it would need a sampler the route does not use. |
| 5 | Nested measurements with the desktop idle | `tty` remains the queue condition until the §8.2 pilot passes. Preflight does not refuse a present desktop: every settle is judged, so a desktop that cannot stay quiet costs a refused run, never a wrong number. | Admitting `desktop-idle` now: there is no post-hold evidence. Refusing a desktop in `capture-meta`: that hard-codes the answer the pilot is meant to find. |
| 6 | Rename the `headless` lane to `nested` | Not in this change. The condition and the topology get distinct words in this document and in `show`. | A rename: a schema change in records, plus two niri-experiments fixtures, for a word. File it if the collision costs a real mistake. |
| 7 | Renderer verification | On measurement routes, every nested niri must report a GL renderer whose name contains the sampled GPU's name, or the sub-run fails before measuring (§6.3). Weston's renderer is recorded too. | Asserting `NVIDIA` literally, as `hidden-window-attribution.sh` does: that names a vendor, not the GPU that was sampled. |
| 8 | When fixtures build | Before preflight, followed by a bounded load wait. The hold window and the baseline then exclude the build. | Building after preflight, as fixtures do today: a build's tail refused the first settle in `hidden-window-attribution.sh`, which is why it grew `await_load`. |

## 5. Host conditions per route

### 5.1 Frozen pixels

Frozen-pixel tests run under `cargo test` or nextest, with no capture-meta
involvement, no lock and no queue. Nothing changes.

### 5.2 Nested pixels

- **Allowed:** `in-use`, `desktop-idle` and `tty`. A stricter host satisfies a
  looser condition.
- **Preflight:** it checks the lock, recovers a stale hold, applies the partial
  hold (decision 3) and writes `run`, `environment` and `hold`. It does no
  sampling and runs no `host-load`.
- **Per sub-run:** `begin` records the sub-run's inputs. There is no GPU
  cooldown.
- **Quiet queue:** the fixture's tasks need `nested`, not `quiet`. They run
  whenever the session is free.

### 5.3 Nested measurements

- **Allowed:** `tty`. `desktop-idle` is added after the §8.2 pilot.
- **Preflight and settle:** unchanged thresholds, the full hold, and NVIDIA
  sampling.
- **Renderer check:** both nested compositors are checked after every launch
  (§6.3).
- **Quiet queue:** `--needs headless` until the pilot passes, then `idle`.

### 5.4 Dedicated

- **Allowed:** `tty` only. The existing session checks stay: `XDG_SESSION_TYPE=tty`,
  no `DISPLAY` or `WAYLAND_DISPLAY`, and no GPU client of either kind.
- **What moves:** those checks run before the hold (§7).
- **What stays:** the idle-budget power lane's `fuser` inventory stays the
  stricter power evidence, as the protocol already says.
- **Renderer:** niri on DRM already rejects software EGL devices
  (`src/backend/tty.rs`). The renderer check of §6.3 still runs, so the record
  carries the renderer's name.

## 6. `capture-meta` and smoke-lib changes

### 6.1 `preflight --lane pixels`

Preflight runs the steps below in order. Steps 2 and 3 are the same as for the
other lanes.

1. Write `run`, including `lane: "pixels"`.
2. Acquire the lock.
3. Recover a stale hold.
4. Plan the hold with only `timer` and `service` items. `plan_hold` gains a
   `kinds` argument, and the full hold passes every kind.
5. Start the guard, and apply the hold.
6. Write `hold`, then `environment`.

Preflight then writes a `preflight` section:
`{"verdict": "unsampled", "at", "lock", "host_condition"}`. It has no
`thresholds`, no `baseline` and no `reasons`.

`environment` on this lane omits `gpu` when `nvidia-smi` is absent and records
whatever is present. It never refuses for a missing sampler. `GpuReader` is
constructed only for the other lanes.

`hold_settle` defaults to 0 on this lane. Its wait exists to let the GPU leave
the P-state of a monitor power-off, and this lane powers nothing off.

### 6.2 `begin`

```
tools/capture-meta begin <run-dir> --sub-run NAME [--input PATH]...
```

`begin` exists only for `pixels` runs and refuses (exit 2) on any other lane.
It checks the lock as `settle` does and appends one sub-run entry:
`{"name", "started", "inputs", "verdict": "begun"}`. `finish` stamps the entry
as it stamps a settled one. `settle` refuses on a `pixels` run (exit 2), so a
fixture cannot have sampled numbers attached to a pixel run.

In the smoke lib, `settle_before_launch` takes the verb from the run's lane.
It calls `gpu_cooldown` and `settle` on `headless`, and `begin` on `pixels`.
`start_nested` keeps its signature, so every launch still passes through one
place.

The lib's timing helpers (`gpu_cooldown`, `trace_run`, `gpu_median_ns`,
`capture_bg`) fail at once under a `pixels` run, naming the helper and the
lane. A fixture that starts recording a timing therefore cannot stay on the
pixel route by accident. The lib reads the lane from `capture.json` through
`capture-meta show --field run.lane`, a small read-only addition to `show`.

### 6.3 Renderer verification

On `headless` and `dedicated` runs, after each launch and before the first
stimulus, the lib reads the new portion of the compositor's log, as
`check_renderer` does in `hidden-window-attribution.sh` today.

The check:

- **Requires** a `GL Renderer:` line whose text contains
  `environment.gpu.name` (for example `NVIDIA GeForce RTX 3070`).
- **Fails** on `llvmpipe` and on `software rasterizer`.
- **Records** the line through `finish --renderer "<line>"`. `finish` stores it
  on the sub-run as `renderer.niri`.

Weston's renderer line is read from its own log and recorded as
`renderer.weston`, under the same requirement. The function moves into the lib
as `verify_renderer <name>`, and `hidden-window-attribution.sh` drops its copy.

A failed check fails the fixture with the sub-run left open. The record then
shows which launch rendered elsewhere. On `pixels` runs the same lines are
recorded through `finish --renderer`, but nothing is required of them.

### 6.4 Build, then wait, then preflight

The lib gains `await_load` from `hidden-window-attribution.sh` unchanged: it
waits for 1-minute load below 1.0, with a 5-minute bound and a rehearsal
bypass. Measurement fixtures run their build, then `await_load`, then
`capture_preflight`. Pixel fixtures may run the same order, but skip
`await_load`.

`capture_identity` stays after preflight, because `identity` writes into a
preflighted record. The fixture's `OUT` guard and trap come from sourcing the
lib, which still happens first. This changes the protocol design's §5 order
("preflight, then build"). That design gains a note pointing here.

## 7. Fail-early mismatch handling

Every check below runs before the hold changes the host, except where the
check needs the hold applied. A refusal writes `preflight.verdict: "refused"`
with the reason.

| Mismatch | Detected by | When | Exit |
| --- | --- | --- | --- |
| `dedicated` with a desktop session or display variables | `session_reasons` | before the hold. Today it runs after the hold and its settle sleep; it moves. | 1 |
| `dedicated` or `headless` without `nvidia-smi` | `GpuReader()` | before the lock. Today it is already raised at construction in `cmd_preflight`. | 2 |
| A measurement lane with a live desktop whose monitors will not power off | `wait_powered_off` | during the hold (existing) | 2 |
| Lock held by a live run | `acquire_lock` | first (existing) | 1 |
| `settle` on `pixels`, or `begin` on another lane | the verb | at call | 2 |
| A lib timing helper under `pixels` | the lib | at call; the fixture fails | fixture failure |
| Nested renderer not the sampled GPU | `verify_renderer` | after launch, before any stimulus; the sub-run stays open | fixture failure |
| A disturber fires, or a held monitor wakes | journal scan and guard (existing) | at release; the run is disturbed | 1 |

`host_condition` is observed, never declared. Preflight records `tty` when
`XDG_SESSION_TYPE=tty` and no desktop socket is live. It records `desktop`
otherwise, and on a measurement lane that means the hold powered the monitors
off. The observation cannot tell `in-use` from `desktop-idle`: whether someone
is working is not visible to it. Nested measurement runs record `desktop` as
fact, and the queue condition (§5.3) is what keeps them off a desktop in use.

## 8. Verification

### 8.1 Offline checks (no host)

**`tools/test_capture_meta.py`**, with `FakeHost`, `ProcReader` and
`GpuReader` doubles:

- **Pixel preflight with no `nvidia-smi`:** exit 0. The record has no
  `baseline`, no `environment.gpu`, `preflight.verdict` is `unsampled`, and the
  hold items are timers and services only.
- **The other lanes with no `nvidia-smi`:** exit 2 naming the sampler. The host
  is untouched, with no hold file and no lock.
- **`begin`:** on `pixels`, it appends `begun` with the input hashes. On
  `headless` it exits 2. `settle` on `pixels` exits 2.
- **`finish --renderer`:** stores the line on the sub-run. A second `finish`
  still exits 2.
- **`dedicated` under a Wayland session:** refuses, and the fake host records
  no `systemctl stop` and no `power-off-monitors`.
- **`show`:** renders a pixels record ("GPU not sampled: pixels lane") and a
  record with renderers. `show --field run.lane` prints the lane.
- **`host_condition`:** recorded as `tty` and as `desktop` from fake session
  states.
- **Schema:** stays 1. Every addition is optional, and old records still
  validate and render.

**`tools/test_glass_optic_smoke.py`**, with a stub `CAPTURE_META`:

- **`start_nested`:** calls `begin` under `pixels`, and `gpu_cooldown` then
  `settle` under `headless`.
- **Timing helpers:** each one fails under `pixels`.
- **`verify_renderer`:** passes a log naming the sampled GPU, and fails on
  llvmpipe, on a missing line, and on a different GPU name.
- **Adopted fixtures:** `glass-edge-sheet.sh` and `glass-noise-layers-smoke.sh`
  preflight with `pixels`.
- **Ordering:** the measurement fixtures build before `capture_preflight`.

### 8.2 Pilots (host steps, each its own task)

Every live step is a separate quiet task with a pilot first and a `run:` note
for each attempt. None is part of the implementation.

1. **Pixel route on a desktop in use.** Run `glass-noise-layers-smoke.sh` with
   the `pixels` lane, on the desktop while it is in use (needs `nested` and the
   owner's go-ahead for host use).
   - **Passes when:** every AE-0 assertion holds, and the record shows no GPU
     fields and timers held and restored. If a retained run has identical
     `provenance.binaries` hashes (for example under
     `noise-layers-pilot-1791337568`), `metrics.txt` must equal its own as
     well. If none has, the in-run assertions alone are the verdict, and the
     run note says so.
   - **What it shows:** that pixel evidence does not depend on the host
     condition.
2. **Nested measurement with the desktop idle.** Run `noise-layers-cost.sh`
   twice on one retained binary, from the desktop with the owner away, the full
   hold applied and no other session working (`--needs idle`). Use a binary
   that already has a TTY run: `noise-idle-variants`' B-spline build, or the
   kept `558bfa02`.
   - **The run's pilot first,** then the full run.
   - **Passes when:** both runs pass every settle; no disturbance is recorded;
     and each case's median lies within the spread of that case across the
     binary's TTY runs, widened by the run-to-run spread those runs show.
   - **What follows a pass:** §5.3 admits `desktop-idle`, the quiet queue maps
     nested measurements to `idle`, and this document records the evidence.
   - **What follows a refused settle:** the refusal and its reason are recorded,
     and the route stays `tty`. A refused settle is safe, so a retry needs a
     named cause first.

## 9. Out of scope

- **A software-rendered lane.** It waits for a consumer, which none of the
  current fixtures is. Such a consumer would have to prove, for every
  participating compositor and client (nested niri, Weston, kitty and any
  stimulus client), which renderer actually drew. That proof needs each
  process's own renderer report, not a Mesa environment variable. Only then is
  "software-rendered timing" a claim about the run. Recorded as an open
  question; not designed here.
- **Sampling within a sub-run.** Neither condition samples the GPU between a
  settle and the sub-run's end. A transient inside a sub-run is caught only if
  a held disturber fired. The §8.2 pilot's comparison of medians is the check
  that this residual risk does not move results.
- **Hidden-window GPU attribution.** It stays under `material-7afc31`.
- **Threshold changes.** The thresholds are unchanged.
- **Input prechecks** belong to `material-5dbf17`. **Fixture outcome
  finalization** belongs to `material-232940`.

## 10. Follow-ups this design names

- A pixels-only mode for each mixed fixture (iridescence, render-order, aurora,
  noise-site). One idea task each, filed by the plan, taken when the fixture is
  next needed.
- The niri-experiments fixtures (`idle-budget.sh`, `jelly-motion.sh`) build
  before preflight (§6.4). This is a change in that repository, filed there.
- On completion: a `tasks note` on `material-6bd4a3` with these decisions, and
  an update to the capture lifecycle brief.
