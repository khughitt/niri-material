# Per-route host conditions for capture evidence

**Status:** draft for owner review (2026-10-08). Revised after spec review
round 1, which covered renderer logging, launchers outside the lib, pre-hold
dedicated checks, renderer evidence on rejected launches, and the pilot's
reference interval.
**Task:** `material-18c2a1`, under `material-2834d7` (evidence instruments).
**Extends:** [capture protocol design](2026-09-11-material-capture-protocol-design.md)
(`tools/capture-meta`) and [disturber hold design](2026-10-04-capture-disturber-hold-design.md).
**Brief:** [capture lanes and lifecycle](../notes/2026-10-06-capture-lifecycle-brief.md#renderer-aware-capture).

## 1. Intent

Every capture fixture in this repository gates on the same thing: a GPU-quiet
`preflight --lane headless`, then a GPU-quiet `settle` before every nested
launch. That gate exists for timing and power evidence, but it is also applied
to fixtures whose evidence host load cannot change.

- **`glass-edge-sheet.sh`** settles 77 times to produce a contact sheet the
  owner judges by eye.
- **`glass-noise-layers-smoke.sh`** settles once per cell, and every cell it
  asserts is a pixel equality.

Both wait in the quiet queue for an idle machine their evidence does not need.

The opposite risk exists too. A nested measurement trusts that the GPU it
sampled is the GPU that rendered. Only `hidden-window-attribution.sh` checks
the nested compositor's renderer. Every other measured fixture would accept a
Tracy timing from a nested niri that fell back to a software renderer, and
label it with NVIDIA telemetry. With niri's default log filter, its logs would
not even show which renderer drew.

The idle-budget evidence records three desktop attempts that each lost a settle
to a different transient. All of them predate the disturber hold (2026-10-04),
which powers the monitors off, inhibits idle and holds user timers for the
length of a run. No nested measurement with per-case settles has been attempted
with the desktop up since. Whether the desktop can host measurements is open,
not settled against.

This design names the evidence class of every capture route. For each class
it states what the route may claim, which host condition its preflight and
settles require, and how a mismatch fails before anything is measured.
Hardware thresholds are unchanged.

Success means each of the following holds:

- Every capture fixture and every compositor launcher in this repository and
  in niri-experiments is assigned to one route.
- A pixel route runs on a desktop in use and records no GPU telemetry.
- Every measured launch records the renderer each compositor reported, at the
  moment it was checked. A measurement run whose launch was not verified does
  not release cleanly.
- A route's host-condition mismatch refuses before the hold changes anything on
  the host.
- The quiet queue's `--needs` for each route follows from this document.
- Whether nested measurements may run with the desktop idle is decided by a
  pilot whose reference runs, hashes and interval are fixed here in advance.

## 2. Terms

**Route:** a fixture's evidence class together with its topology. There are four
(§3). `capture-meta`'s `--lane` argument names the route.

**Host condition:** the state of the host session during a run. Three exist:

| Condition | Meaning | Quiet-queue `--needs` |
| --- | --- | --- |
| `in-use` | The desktop is up and someone may be working on it; other sessions may run builds. | none (not a quiet step) |
| `desktop-idle` | The desktop is up but untouched. The hold has powered the monitors off and held idle and user timers. No other session runs work. | `idle` |
| `tty` | The ordinary desktop session is stopped, and the run starts from a TTY login. | `headless` |

The capture lane `headless` (niri nested under headless Weston) and the
quiet-queue condition `headless` (desktop stopped) are unrelated meanings of one
word. This document writes `tty` for the condition and `nested` for the
topology, and leaves both existing spellings in place (§4, decision 6).

**Launcher:** a shell function that starts a compositor for a sub-run. Six
exist (§6.4).

## 3. Routes

| Route | `--lane` | Evidence it may claim | Claims it may not make | Host condition |
| --- | --- | --- | --- | --- |
| Frozen pixels | none (no capture-meta) | Pixels at a frozen instant from the in-process headless renderer | Anything about time, cost or a real GPU | any |
| Nested pixels | `pixels` (new) | Pixels from a static scene: equality and AE between shots in one run; sheets for the owner | Draw times; redraw counts or rates; fps; wall-clock motion | `in-use` or stricter |
| Nested measurements | `headless` (unchanged) | Draw-time medians; redraw and draw counts and rates; cadence; fps; pixels sampled at wall-clock instants | Board power | `tty`; `desktop-idle` after the §8.2 pilot |
| Dedicated | `dedicated` (unchanged) | DRM timing; VT resume; unlock; screencast; board power | — | `tty` only |

**Classification rule.** A value is a measurement if it could change when the
host is slower or the GPU is shared. This covers anything a fixture asserts or
records, including values written to its metrics "recorded only". A fixture
that records even one such value is a measurement fixture, whatever else it
asserts.

A static scene shot twice is a pixel value, and so is a sheet of static cells.
A frame grabbed after a `sleep` while an animation runs is a measurement:
under contention, the frame shown at that instant changes.

### 3.1 Current fixtures

| Fixture | Route | Why |
| --- | --- | --- |
| `src/tests/ring_pair.rs`, `ring_look.rs`, `glass_edge.rs`, `accent_tint.rs`, `noise_layers.rs`, `noise_site.rs` | Frozen pixels | In-process with a frozen clock; unchanged |
| `glass-edge-sheet.sh` | Nested pixels | Static cells after a fixed wait; PASS is a clean finish; the owner judges the sheet |
| `glass-noise-layers-smoke.sh` | Nested pixels | Every assertion is a pixel statistic of a static cell, each shot twice to AE 0 |
| `glass-iridescence-smoke.sh` | Nested measurements | Records GPU draw medians |
| `glass-render-order-smoke.sh` | Nested measurements | Records GPU draw medians and timed `aurora-motion` shots |
| `glass-aurora-smoke.sh` | Nested measurements | Asserts redraw rates; shots are time-bucketed |
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
drops the timing and runs on the pixel route. That split is a per-fixture
follow-up, taken when the fixture is next needed (§10).

The three clip fixtures are measurements as they stand. Their visual purpose
belongs to frozen-clock fixtures, as the
[workstreams brief](../notes/2026-10-02-workstreams-brief.md) already routes it.

## 4. Decisions

| # | Question | Decision | Rejected |
| --- | --- | --- | --- |
| 1 | How a fixture names its route | One new value for the existing `--lane`: `pixels`. `headless` and `dedicated` keep their meaning. | A separate `--evidence` flag: it changes every caller, including two fixtures in niri-experiments, for three valid combinations out of six. |
| 2 | Does a pixel run take the capture lock? | Yes, the same exclusive lock. A pixel run overlapping a measurement would land inside that measurement's settles, and pixel runs are short. | A shared reader lock for pixel runs: it adds a lock mode for runs that are quick to take in turn. |
| 3 | What a pixel run holds | User timers and the declared services, not idle and not the monitors. | The full hold: it powers off the monitors of a desktop in use. No hold: on 2026-09-24, `wali-rotate.timer`'s Prism apply stopped a nested kitty from mapping, which disturbs pixels too. |
| 4 | Do pixel runs sample the GPU? | No. Their records hold no `baseline` and no GPU fields, and sub-runs open with a new `begin` verb instead of `settle` (§6.2). | Sampling recorded but not judged: numbers no check reads, needing a sampler the route does not use on a non-NVIDIA host. |
| 5 | Nested measurements with the desktop idle | `tty` stays the queue condition until the §8.2 pilot passes. Preflight does not refuse a present desktop: every settle is judged, so a desktop that cannot stay quiet costs a refused run, never a wrong number. | Admitting `desktop-idle` now: there is no post-hold evidence. Refusing a desktop in `capture-meta`: that hard-codes the answer the pilot is meant to find. |
| 6 | Rename the `headless` lane to `nested` | Not in this change. Condition and topology get distinct words in this document and in `show`. | Renaming: a schema change in records, plus two niri-experiments fixtures, for a word. File it if the collision costs a real mistake. |
| 7 | Where the renderer check lives | In `capture-meta`, as a `renderer` verb that parses the launch's logs, judges them, and writes the result to the open sub-run at once (§6.3). Every launcher calls it, and `release` refuses a measurement run with a finished sub-run that was never verified. | Lib-only shell checks: five of six launchers live outside the lib, and a shell check records nothing. Recording through `finish`: a rejected launch is never finished, so its evidence would be lost. |
| 8 | Which GPU a renderer must name | The sampled GPU: every renderer line from niri and Weston must contain `environment.gpu.name`. | Asserting `NVIDIA`, as `hidden-window-attribution.sh` does: that names a vendor, not the GPU that was sampled. |
| 9 | When fixtures build | Before preflight, followed by a bounded load wait, so the hold window and the baseline exclude the build. | Building after preflight, as today: a build's tail refused the first settle in `hidden-window-attribution.sh`, which is why it grew `await_load`. |

## 5. Host conditions per route

### 5.1 Frozen pixels

Frozen pixels run under `cargo test` or nextest. There is no capture-meta, no
lock and no queue, and nothing changes.

### 5.2 Nested pixels

- **Allowed conditions:** `in-use`, `desktop-idle` and `tty`. A stricter host
  satisfies a looser condition.
- **Preflight:** it checks the lock, recovers a stale hold and applies the
  partial hold (decision 3). It writes `run`, `environment` and `hold`. It
  takes no samples and runs no `host-load`.
- **Per sub-run:** `begin` replaces `settle`, and there is no GPU cooldown.
  `renderer` records each compositor's renderer without requiring one (§6.3).
- **Queue:** the fixture's tasks need `nested`, not `quiet`, so they run
  whenever the session is free.

### 5.3 Nested measurements

- **Allowed conditions:** `tty`. `desktop-idle` is added after the §8.2 pilot.
- **Preflight and settle:** unchanged thresholds, the full hold, and NVIDIA
  sampling.
- **Per launch:** `renderer` must verify both compositors before any stimulus.
- **Queue:** `--needs headless` until the pilot passes, then `idle`.

### 5.4 Dedicated

- **Allowed conditions:** `tty` only.
- **Before the hold** (§7): the session environment check
  (`XDG_SESSION_TYPE=tty`, no `DISPLAY` or `WAYLAND_DISPLAY`), the live desktop
  socket check and a GPU client inventory. A TTY login with no display
  variables can still have the ordinary desktop running, which only the socket
  and client checks see.
- **Power evidence:** the idle-budget power lane's `fuser` inventory stays the
  stricter evidence, as the protocol already says.
- **Renderer:** niri on DRM already rejects software EGL devices
  (`src/backend/tty.rs`). `renderer` still verifies and records its line. There
  is no Weston.

## 6. `capture-meta` and launcher changes

### 6.1 `preflight --lane pixels`

The order is:

1. Write `run`, with `lane: "pixels"`.
2. Acquire the lock and recover a stale hold, as for every lane.
3. Plan the hold with only `timer` and `service` items. `plan_hold` gains a
   `kinds` argument; the full hold passes every kind.
4. Start the guard and apply the hold.
5. Write `hold`, then `environment`.
6. Write `preflight`: `{"verdict": "unsampled", "at", "lock", "host_condition"}`,
   with no `thresholds`, no `baseline` and no `reasons`.

`environment.gpu` is recorded when `nvidia-smi` is present (name, uuid and
driver, so the pixel record says what it rendered on) and omitted otherwise.
This lane never refuses for a missing sampler. `GpuReader` stays required for
the other lanes.

`hold_settle` defaults to 0 on this lane. Its wait lets the GPU leave the
P-state transient of a monitor power-off, and this lane powers nothing off.

### 6.2 `begin`

```
tools/capture-meta begin <run-dir> --sub-run NAME [--input PATH]...
```

Only `pixels` runs accept `begin`; any other lane exits 2. It checks the lock
as `settle` does and appends `{"name", "started", "inputs", "verdict": "begun"}`.

`finish` stamps a begun entry as it stamps a settled one. `settle` on a
`pixels` run exits 2, so a pixel run can never carry sampled numbers.

The lib's `settle_before_launch` takes its verb from the run's lane: on
`headless`, `gpu_cooldown` then `settle`; on `pixels`, `begin`. It reads the
lane through `capture-meta show --field run.lane`, a small read-only addition
to `show`.

The lib's timing helpers (`gpu_cooldown`, `capture_bg`, `trace_run`,
`gpu_median_ns`) fail at once under a `pixels` run, naming the helper and the
lane. A fixture that starts recording a timing therefore cannot stay on the
pixel route by accident.

### 6.3 `renderer`

```
tools/capture-meta renderer <run-dir> --sub-run NAME --niri-log PATH [--weston-log PATH]
```

A launcher calls `renderer` once per launch, after the compositor is up and
before any stimulus. The log paths hold only this launch's output. The lib
already tracks `LOG_OFFSET` into its appended `niri.log`. Every launcher slices
its logs from the launch's start into `$OUT/<sub-run>.niri.renderer.log` and
`$OUT/<sub-run>.weston.renderer.log`, and those slices are retained in the run.

**Enabling the line.** niri's `DEFAULT_LOG_FILTER` sets
`smithay::backend::renderer::gles=error`, so a default niri never logs its
renderer. Retained cost-run logs contain no such line. Every launcher therefore
starts niri with `RUST_LOG=niri=debug,smithay::backend::renderer::gles=info`.
That is the default filter with the GLES target raised to `info`, and it is
already used by `hidden-window-attribution.sh`. Weston logs its renderer at its
default level.

**Parsing.** The actual formats, both of which are test fixtures:

- **niri (smithay):** an `INFO` record whose message is
  `GL Renderer: "NVIDIA GeForce RTX 3070/PCIe/SSE2"`. niri writes ANSI colour
  escapes to its log, so `renderer` strips `\x1b\[[0-9;]*m` before matching.
  The pattern is `GL Renderer: "(.+)"`.
- **Weston:** `[21:51:39.332] GL renderer: NVIDIA GeForce RTX 3070/PCIe/SSE2`,
  with a lowercase `renderer` and no quotes. The pattern is
  `\] GL renderer: (.+)$`.

**Judgement.** Each compositor's verdict is one of three values:

- `verified`: at least one renderer line, every line contains
  `environment.gpu.name`, and none matches `llvmpipe` or `software rasterizer`
  (case-insensitive).
- `missing`: no renderer line was found.
- `mismatch`: any line fails the other conditions.

`--weston-log` is required on `headless` and `pixels`, and refused on
`dedicated`, where no Weston runs.

**Recording.** `renderer` writes to the latest open sub-run of that name, under
the record lock, before it exits:

```json
"renderer": {"expected": "NVIDIA GeForce RTX 3070", "at": "…",
             "niri": {"verdict": "verified", "lines": ["NVIDIA GeForce RTX 3070/PCIe/SSE2"]},
             "weston": {"verdict": "verified", "lines": ["NVIDIA GeForce RTX 3070/PCIe/SSE2"]},
             "verdict": "verified"}
```

The overall `verdict` is `verified` only when every compositor is. A rejected
launch keeps its observed lines and verdicts even though the fixture then fails
and the sub-run is never finished.

**Exit codes:**

| Exit | When |
| --- | --- |
| 2 | No open sub-run of that name; or a second `renderer` call for one sub-run (one launch per sub-run, as the lib already does); or an unreadable log |
| 1 | On a measurement lane, an overall verdict other than `verified`. The launcher fails the fixture. |
| 0 | `verified`. On `pixels`, also any verdict: nothing is required, there is no `expected` when `environment.gpu` is absent, and the per-compositor verdict there is `recorded` or `missing`. |

**Coverage at release.** On `headless` and `dedicated`, `release` exits 1 when
any finished sub-run lacks `renderer.verdict == "verified"`, naming those
sub-runs, after the hold is restored. This works the same way as a disturbed
run. A launcher that never calls `renderer`, in this repository or in
niri-experiments, therefore cannot produce a clean measurement run. `show`
prints each sub-run's renderer verdict.

### 6.4 Launchers

Every compositor launcher adopts §6.2 and §6.3. These six exist, plus one fixture that already uses the lib's:

| Launcher | Where | Change |
| --- | --- | --- |
| `start_nested` | `glass-optic-smoke-lib.sh` | `RUST_LOG`, log slices, `renderer` before returning, verb by lane. This covers the ten lib fixtures and idle-budget `trace`. |
| `start_nested` | `focus-swap-clips.sh` | `RUST_LOG`; Weston output to a file (it runs under `systemd-run`, which logs to the journal today); `renderer` |
| `start_nested` | `drag-lag-clips.sh` | Same as `focus-swap-clips.sh` |
| `start_nested` | `ring-motion-clips.sh` | Same as `focus-swap-clips.sh` |
| `start_drm` | `optic-settling-smoke.sh` | `RUST_LOG`, niri log slice, `renderer` with no Weston log |
| `start_drm` | niri-experiments `idle-budget.sh` (power) | Same as `optic-settling-smoke.sh`; a change in that repository |
| — | niri-experiments `jelly-motion.sh` | Already launches through the lib's `start_nested`. Its own `GL renderer: NVIDIA GeForce RTX 3070` grep of `weston.log` becomes redundant and goes. |

`hidden-window-attribution.sh` drops its own `check_renderer` and its `RUST_LOG`
export in favour of the lib's. Its stricter failure patterns (material shader
compile errors, fallbacks, panics) stay in the fixture as its own log check,
since they are about the code under test, not the renderer.

**Static coverage check.** `tools/test_glass_optic_smoke.py` gains a source
scan over `docs/materials/scripts/*.sh`. Every script that calls
`capture_meta settle`, `capture_meta begin` or `settle_before_launch` must also
call `capture_meta renderer` or use the lib's `start_nested`, and must set the
GLES log filter. A new launcher that forgets fails the tooling suite before
`release` ever sees a run.

### 6.5 Build, then wait, then preflight

The lib gains `await_load`, moved unchanged from `hidden-window-attribution.sh`:
1-minute load below 1.0, bounded at 5 minutes, with a rehearsal bypass.
Measurement fixtures run their build, then `await_load`, then
`capture_preflight`. Pixel fixtures may build first too, but skip `await_load`.

`capture_identity` stays after preflight, because `identity` writes into a
preflighted record. Sourcing the lib (the `OUT` guard and the trap) still comes
first. This replaces the protocol design's §5 order ("preflight, then build"),
and that design gains a note pointing here.

## 7. Fail-early mismatch handling

Preflight runs every check it can before planning the hold. Only the quietness
judgement and the monitor power-off need the hold applied.

**Pre-hold order**, after `run` and the lock:

1. The lane's session checks: `session_reasons`, unchanged, for `dedicated`.
2. `desktop_socket(host)`, computed once and passed to `plan_hold` (which
   computes it today). On `dedicated`, a live socket refuses: "a desktop niri
   is running at `<socket>`".
3. One GPU client inventory (`GpuReader.clients()`). On `dedicated`, any client
   refuses. On `headless`, a compute client refuses. `pixels` skips it.

The sampling-time client checks stay, for clients that appear after this point.

| Mismatch | Detected by | When | Exit |
| --- | --- | --- | --- |
| Lock held by a live run | `acquire_lock` | first (existing) | 1 |
| `dedicated` or `headless` without `nvidia-smi` | `GpuReader()` | before the lock (existing, at construction) | 2 |
| `dedicated` with display variables | `session_reasons` | pre-hold (moved from after the hold) | 1 |
| `dedicated` with the ordinary desktop running | live desktop socket | pre-hold (new) | 1 |
| `dedicated` with any GPU client, or `headless` with a compute client | client inventory | pre-hold (new), and again while sampling (existing) | 1 |
| A measurement lane whose desktop monitors do not power off | `wait_powered_off` | during the hold (existing) | 2 |
| `settle` on `pixels`, or `begin` on another lane | the verb | at call | 2 |
| A lib timing helper under `pixels` | the lib | at call; the fixture fails | fixture failure |
| Nested or DRM renderer is not the sampled GPU, or not logged | `renderer` | after launch, before any stimulus; recorded on the open sub-run | 1, fixture failure |
| A finished measurement sub-run with no verified renderer | `release` | after the hold is restored | 1 |
| A disturber fires, or a held monitor wakes | journal scan and guard | at release; the run is disturbed (existing) | 1 |

A pre-hold refusal writes `preflight.verdict: "refused"` with its reasons and
releases the lock. The fake host's command log then shows no `systemctl stop`,
no `caffeine-enable` and no `power-off-monitors`.

`host_condition` is observed, never declared. Preflight records `tty` when
`XDG_SESSION_TYPE=tty` and no desktop socket is live, and `desktop` otherwise.
On a measurement lane, `desktop` means the hold powered the monitors off. The
observation cannot tell `in-use` from `desktop-idle`, because whether someone is
working is not visible to it. A nested measurement run records `desktop` as
fact, and the queue condition (§5.3) is what keeps such runs off a desktop in
use.

## 8. Verification

### 8.1 Offline checks (no host)

**`tools/test_capture_meta.py`**, with `FakeHost`, `ProcReader` and
`GpuReader` doubles:

- **Pixel preflight, no `nvidia-smi`:** exit 0. The record has no `baseline` and
  no `environment.gpu`, and `preflight.verdict` is `unsampled`. The hold items
  are timers and services only, and the fake host shows no `caffeine-enable`
  and no `power-off-monitors`.
- **The other lanes, no `nvidia-smi`:** exit 2 naming the sampler. No hold file,
  no lock, and no host commands.
- **Pre-hold refusals on `dedicated`:** a live desktop socket with no display
  variables refuses, and so does a graphics client in the pre-hold inventory.
  On `headless`, a compute client refuses. In every case the fake host's command
  log holds no hold action and the lock is released.
- **`begin`:** appends `begun` with input hashes on `pixels`, and exits 2 on
  `headless`. `settle` exits 2 on `pixels`.
- **`renderer`** is fed the actual formats, taken from retained logs:
  - a niri `INFO` line with ANSI escapes and the quoted renderer;
  - a Weston `GL renderer:` line.

  Cases:
  - both lines verified;
  - a niri log at the default filter, with no line: `missing`;
  - llvmpipe: `mismatch`;
  - a different GPU name: `mismatch`;
  - a Weston line spelled `GL Renderer` (wrong case): `missing` for Weston;
  - a missing `--weston-log` on `headless`: exit 2;
  - a `--weston-log` on `dedicated`: exit 2;
  - a second call for the same sub-run: exit 2.

  A rejected verdict is written to the open sub-run before exit 1, and it
  survives a `finish` that never comes.
- **`release`:** exits 1 on a `headless` run with one finished sub-run that
  lacks a verified renderer, naming it, and exits 0 on a `pixels` run with
  `recorded` renderers.
- **`show`:** renders a pixels record ("GPU not sampled: pixels lane") and
  per-sub-run renderer verdicts. `show --field run.lane` prints the lane.
- **`host_condition`:** recorded as `tty` and as `desktop` from fake session
  states.
- **Schema:** stays 1. Every addition is optional, and old records still
  validate and render.

**`tools/test_glass_optic_smoke.py`**, with a stub `CAPTURE_META`:

- **Lib `start_nested`:** calls `begin` under `pixels`, and `gpu_cooldown` then
  `settle` under `headless`. On both, it calls `renderer` with the sub-run name
  and both slices before returning, and starts niri with the GLES log filter.
- **Timing helpers:** each fails under `pixels`.
- **Lane by fixture:** `glass-edge-sheet.sh` and `glass-noise-layers-smoke.sh`
  preflight with `pixels`.
- **Ordering:** measurement fixtures build before `capture_preflight`.
- **Static coverage:** the source scan of §6.4 passes on the tree and fails on
  a synthetic script that settles without `renderer`.
- **The clip fixtures** (already scanned by `test_glass_optic_smoke.py`) **and
  `optic-settling-smoke.sh::start_drm`** (`tools/test_optic_settling.py`):
  their existing source tests gain assertions for the `renderer` call and the
  log filter.

niri-experiments: `fixtures/test_idle_budget.py` asserts `renderer` in the
power lane's `start_drm`, in that repository's change.

### 8.2 Pilots (host steps, each its own task)

Every live step is a separate quiet task, with a pilot first and a `run:` note
for each attempt. None is part of the implementation.

1. **Pixel route on a desktop in use.** Run `glass-noise-layers-smoke.sh` with
   `--lane pixels` on the desktop while it is in use. It needs `nested` and the
   owner's go-ahead for host use.
   - **Passes when:** every AE-0 assertion holds; the record shows no GPU
     sampling, verified-or-recorded renderers, and timers held and restored.
   - **If a retained run has identical `provenance.binaries` hashes:** its
     `metrics.txt` must also be equal.
   - **If none has:** the in-run assertions alone are the verdict, and the run
     note says so.

2. **Nested measurement with the desktop idle.**

   **Reference** (pinned). Three retained TTY runs, all `preflight.verdict:
   quiet`, `environment.session.type: tty`, `hold.desktop: absent`, source
   `1d9e8ddb`:

   | Run | b-side `none` median (ms) | b-side `one-fine-1` median (ms) |
   | --- | --- | --- |
   | `noise-layers-ab-stream-1791430389` | 0.229376, 0.229376 | 0.301568, 0.301056 |
   | `noise-layers-ab-hermite-rebuild-1791431057` | 0.229376, 0.229376 | 0.302080, 0.302080 |
   | `noise-layers-ab-loop-1791432546` | 0.229376, 0.229376 | 0.302080, 0.302080 |

   Pinned identities:
   - **b binary** (`niri-tracy-b` in each run):
     `cb21ad039beaeae2e0fa9ce3e2e9249a81d65c45a9728b9d0db9bb91fcb801e4`.
   - **b-side case configs:**
     - `none-b*.kdl`:
       `aa09f751706396f2d83a6d8eb07f6e8762d1dca2829d852013fd54812aff2bb9`
     - `one-fine-1-b*.kdl`:
       `bc488733c6c015d5546e8c503b15487098e471d1394f9255f84c95433c344cc3`

   **How the fixture runs the retained binary.** `noise-layers-cost.sh`'s
   existing A/B mode: `NOISE_LAYERS_COST_PILOT=0`,
   `NOISE_LAYERS_COST_AB=$NIRI_MATERIAL_WORK_ROOT/noise-layers-ab-hermite-rebuild-1791431057/niri-tracy-b`.
   This runs `none` and `one-fine-1`, two rounds per side. Side a is built from
   the checkout as usual and is not part of the comparison. Only the b-side
   medians are compared.

   **Interval.** Let:
   - `R_c` be the six reference b-side medians of case `c`;
   - `w_c = max(R_c) − min(R_c)`;
   - `q = 0.001024 ms`, one tick of this host's GPU timestamp. Every reference
     median is a multiple of `q/2`, the half-ticks coming from even-count
     medians.

   The accepted interval is
   `I_c = [min(R_c) − w_c − q, max(R_c) + w_c + q]`:
   - `none`: `[0.228352, 0.230400]`;
   - `one-fine-1`: `[0.299008, 0.304128]`.

   **Procedure.** Two desktop-idle runs: the owner away, the full hold applied,
   no other session working, `--needs idle`. The first run is the pilot, and
   the second runs only if the first passes.

   **Before taking the host,** an offline rehearsal (`CAPTURE_META` stubbed)
   generates the b-side configs on the pilot's commit. If their hashes differ
   from the pinned ones, the fixture's configs have changed since `1d9e8ddb`,
   so this reference does not apply. The pilot then starts with three fresh
   TTY runs of the same command in one quiet session, which become the
   reference by the same formula, and this document records their run IDs.

   **Passes when,** for both runs:
   - every settle is `settled`;
   - no disturbance is recorded;
   - every renderer is `verified`;
   - `provenance.binaries` includes the pinned b hash;
   - the b-side config hashes equal the reference's;
   - all four b-side medians of each case lie in `I_c`.

   **Outcome.** A pass admits `desktop-idle` in §5.3, maps nested measurements
   to `idle` in the queue, and records the run IDs here. A refused settle or an
   out-of-interval median is recorded with its reason, and the route stays
   `tty`. A retry needs a named cause first.

## 9. Out of scope

- **A software-rendered lane.** It waits for a consumer, and none of the current
  fixtures is one. Such a consumer would have to prove which renderer actually
  drew, for every participating compositor and client: nested niri, Weston,
  kitty and any stimulus client. That proof is each process's own renderer
  report, the kind `renderer` collects for the two compositors, never a Mesa
  environment variable. Only then is "software-rendered timing" a claim about
  the run. This stays an open question; it is not designed here.
- **Client renderers on measurement routes.** Kitty and the stimulus clients are
  not verified. They render through the same driver, and the hidden-window
  attribution work (`material-7afc31`) owns per-client GPU attribution.
- **Sampling within a sub-run.** Neither condition samples the GPU between a
  settle and the end of its sub-run. A transient inside a sub-run is caught
  only if a held disturber fired. The §8.2 pilot's interval check is what
  tests whether this residual risk moves results with the desktop up.
- **Threshold changes:** none.
- **Input prechecks** (`material-5dbf17`) and **fixture outcome finalization**
  (`material-232940`) are other tasks' work.

## 10. Follow-ups this design names

- **Pixels-only modes:** one for each mixed fixture (iridescence, render-order,
  aurora, noise-site). One idea task each, filed by the plan, to be taken when
  the fixture is next needed.
- **niri-experiments:** `idle-budget.sh` power's `start_drm` and
  `jelly-motion.sh` adopt `renderer` (§6.4), and both build before preflight
  (§6.5). This is a change in that repository, with its own task. Until it
  lands, `release` refuses their measurement runs (§6.3), and this is
  intended.
- **On completion:** a `tasks note` on `material-6bd4a3` with these decisions,
  and an update to the capture lifecycle brief.
