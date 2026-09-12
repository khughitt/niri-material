# Material performance-capture protocol: design

**Status:** implemented on `material-bae9c9` (this plan); `tools/capture-meta`
and the optic smoke adoption are in this repository, the idle-budget adoption
on `niri-experiments` `results/capture-protocol`. Thresholds are host-derived
defaults pending the first dedicated-lane run (`material-265eb0`). The manual
busy-host refusal check passed after reboot with NVIDIA 615.71.09, recorded in
`material-7f7aa3`.

**Task:** `material-bae9c9`, within `material-5d6b2c` (resource-aware rendering).
First consumers: the idle-budget fixture (`material-265eb0`, experiment branch
`results/idle-budget`) and `docs/materials/scripts/glass-optic-smoke-lib.sh`.
Downstream: `material-31074f` (per-element estimation), `material-2ebf2c`
(perceptual metric), `material-f6e284` (cost entry per optic), `prism-ed6be0`
(slow draws), and the run index in `ops-71120e`.

## 1. Problem

Every measurement so far was collected by a fixture that recorded its own idea of
provenance and environment. The optic smoke lib writes `source.commit`,
`binaries.sha256`, `kernel.txt`, `lscpu.txt`, `nvidia-smi.txt`, and
`installed.version` as loose files. The idle-budget fixture writes
`identity.json`, `hardware.json`, `preflight.json`, and `manifest.json` with a
different vocabulary. Neither checks that the machine is quiet before a run or
between sub-runs, so the
[hardware evidence](../materials/2026-09-11-material-hardware-evidence.md) could
report per-optic draw medians but had to declare its power deltas unattributable:
individual run medians spanned 13.7–29.0 W while the desktop shared the GPU and
clocks varied. At the time of writing this design, the host GPU shows 35 %
utilization from an unrelated audio workstation and a load average of 4.5. A
capture started now would inherit that noise silently.

Three things are missing, and they are the whole scope:

1. **One record shape.** A `capture.json` every run writes, so two runs from
   different fixtures, weeks apart, can be compared field by field.
2. **A preflight that refuses.** The machine must be verified quiet before the
   first sub-run, and re-verified between sub-runs; a fixture that cannot get a
   quiet machine must stop, not record.
3. **A run-directory convention.** Where runs live and what the top level of a
   run holds, so `ops-71120e` can index them without reading fixture code.

Out of scope: a generic capture runner (each fixture keeps its own orchestration),
retrofitting retained runs or the single-purpose smoke scripts, thresholds that
claim to be universal (they are recorded, not asserted as truth), and the index
itself.

## 2. The helper: `tools/capture-meta`

A standard-library Python tool beside `tools/package-pin`, tested by
`python3 -m unittest discover -s tools` like its neighbours. Fixtures call it as a
subprocess at fixed points of a run; it owns `capture.json` and the lock, and
nothing else in the run directory.

```
tools/capture-meta preflight <run-dir> --lane headless|dedicated --task ID --fixture NAME [--seconds N] [--owner-pid PID] [--threshold KEY=VALUE]...
tools/capture-meta identity  <run-dir> --source <checkout> --binary PATH... --input PATH... [--config KEY=VALUE]...
tools/capture-meta settle    <run-dir> --sub-run NAME [--input PATH]... [--seconds N]
tools/capture-meta release   <run-dir>
tools/capture-meta show      <run-dir>
```

Exit codes follow `package-pin`: 0 passed, 1 refused (the machine is not quiet, a
named `--binary` or `--input` path does not exist, or the lock is held), 2 cannot run (no `nvidia-smi`, unwritable
directory, malformed existing `capture.json`). A fixture treats 1 and 2 alike:
stop before the sub-run.

### 2.1 `preflight`

Runs once, before any compositor starts. It samples the machine for `--seconds`
(default 20) at 1 Hz and writes the `environment`, `baseline`, and `preflight`
sections of `capture.json`.

The **lane** names the isolation the fixture claims, and preflight checks the claim:

- `headless`: the fixture nests niri under a headless Weston on the same GPU the
  desktop uses. The desktop compositor and its graphics clients are expected —
  this host shows about two dozen at any time, and GeForce drivers report no
  per-process utilization for them (`nvidia-smi pmon` prints `-`), so they cannot
  be judged individually. Preflight therefore *records* the graphics client list
  (`nvidia-smi -q -x` process entries) and *refuses* on any compute client, on a
  held `capture-meta` lock, or on a baseline that is not quiet (§2.4). Aggregate
  quietness is what catches a busy graphics client: the audio workstation that
  holds the GPU at 35 % today fails the utilization threshold, and the record
  names it.
- `dedicated`: the fixture owns the GPU on a bare TTY. Refuses unless
  `XDG_SESSION_TYPE=tty` and `DISPLAY`/`WAYLAND_DISPLAY` are unset, refuses on
  *any* GPU client of either type, and applies the quietness checks. This is the
  idle-budget fixture's existing `IDLE_BUDGET_DEDICATED_SESSION` gate, moved to
  where every power capture can use it.

The lock is `$XDG_RUNTIME_DIR/capture-meta.lock`, a small JSON file naming the
owning PID (`--owner-pid`, default the calling shell's parent) and the run id.
Every lock operation holds `flock` on the sibling guard
`capture-meta.lock.d`. Acquisition publishes a complete temporary lock with an
atomic hard link; if the lock exists and names a live process, preflight refuses
(exit 1) **without writing anything to its own run directory beyond the
refusal**, so a refused run has a `capture.json` that says who held the lock. If
the named process is dead, preflight re-checks liveness under the guard, replaces
the stale lock, and records `reclaimed: true` with the dead owner's PID.
`settle` refuses when the file no longer names this run. `release` removes the
file **only when it names this run's id and owner PID**; otherwise it exits 1 and
leaves it — so a fixture whose preflight was refused can still call `release`
from its cleanup trap without evicting the capture that refused it. The lock
exists because several agent sessions share this machine and two captures in
parallel would each pass preflight against the other's warm-up.

### 2.2 `identity`

Runs once, after binaries are built or copied and configs are written, before the
first sub-run. It writes the `provenance` section:

- `source`: the checkout's `HEAD` commit, its branch, whether the tree is dirty,
  and if dirty a SHA-256 fingerprint covering `git diff HEAD` plus each
  untracked path and its content (or a symlink's target text). The untracked
  paths are listed in the record; the diff and files themselves are the
  fixture's to retain. A dirty tree is recorded, not refused; refusing belongs
  to the fixture (idle-budget builds from `git archive` precisely so it never
  happens).
- `binaries`: path basename and SHA-256 of every `--binary`. The Tracy and
  release builds are separate entries, as the hardware evidence already treats
  them.
- `inputs`: basename and SHA-256 of every `--input` that exists before the first
  sub-run: the fixture script itself, the shared helper it sourced, patches,
  wallpapers, build sidecars, and any config already written. Configs a fixture
  generates per case do not exist yet at this point; they are recorded by
  `settle --input` (§2.3). Together the two are `inputs.sha256` and
  `binaries.sha256` as structured data.
- `config`: free `KEY=VALUE` facts the fixture asserts about the scene — pinned
  glass values, preset name, output mode, scale, VRR. Glass derived from the
  live `prism.kdl` moved from IOR 1.02 to 1.24 within one week of the ring light
  work, so a capture pins its glass and records the pinned values here.

### 2.3 `settle`

Runs before every sub-run (case, round, matrix cell), after the previous
compositor has exited, after the sub-run's own inputs are written, and before its
compositor starts. It samples for `--seconds` (default 10), compares against the
preflight `baseline`, and appends one entry to `sub_runs[]` with the sample
summary, a verdict, and the basename and SHA-256 of every `--input` — the case
config, and anything else generated for this sub-run alone. The first sub-run
calls `settle` too, so its inputs are recorded and the record has one entry per
sub-run, not one fewer. Refuses (exit 1) when the
machine has not returned to baseline within tolerance; the fixture then stops,
and the run directory shows exactly which sub-run was reached and why the rest
are absent. It never waits and retries on its own: how long to wait is the
fixture's decision, and a silent retry would hide the disturbance from the record.

### 2.4 Quietness

A sample is the tuple (CPU busy fraction from `/proc/stat` deltas, 1-minute load
average, available memory from `/proc/meminfo`, GPU utilization %, GPU power W,
GPU graphics clock MHz, GPU P-state, GPU client list) taken once per second.

The baseline is the per-field median over the preflight window. **Quiet** means:
CPU busy median ≤ 10 %, load average ≤ 2.0, available memory ≥ 20 % of
`MemTotal` in every sample, GPU utilization median ≤ 5 %, P-state P8 in every
sample, no compute client in any sample (no client of any kind on the dedicated
lane), and GPU power inter-quartile range ≤ 1.0 W. A `settle` passes when its
medians are within tolerance of the baseline: CPU ± 5 points, available memory
within 5 % of `MemTotal` of the baseline and still above the floor, GPU
utilization ± 3 points, GPU power ± 1.5 W, and the same P-state and client
constraints. The memory floor is a fraction rather than a byte count because
the same rule must hold on a 16 GiB laptop and this 125 GiB host.

These numbers are defaults, overridable with `--threshold KEY=VALUE`, and every
run records the thresholds it was judged against. They come from what an idle
RTX 3070 on this host does (P8, ~18–21 W, 0–2 % utilization), not from a claim
about GPUs in general; the first dedicated-lane run should revisit them and the
design doc records the revision.

Non-NVIDIA hosts: preflight exits 2 with a message naming the missing sampler.
Supporting another vendor means adding a sampler, not weakening the rule.

### 2.5 `show`

Prints the record in a fixed human-readable layout for evidence documents: the
environment table the hardware evidence doc already renders by hand, the
provenance list, and the sub-run verdicts. Evidence docs may paste it; they must
not paraphrase it.

## 3. `capture.json`

One file at the top of a run directory, JSON, `schema: 1`. Sections are written
by the sub-command that owns them; a sub-command refuses to overwrite a section
that exists (exit 2), so a fixture cannot half-rerun into a directory.

```json
{
  "schema": 1,
  "run": {"id": "trace-20260911T043433", "task": "material-300b87",
          "fixture": "aurora-iridescence-hardware.sh", "lane": "headless",
          "started": "2026-09-11T04:34:33-04:00", "host": "<hostname>"},
  "environment": {
    "kernel": "7.2.2-arch1-1", "cpu": "AMD Ryzen Threadripper 1950X",
    "cpu_threads": 32, "memory_total_kib": 67108864,
    "gpu": {"name": "NVIDIA GeForce RTX 3070", "uuid": "GPU-…",
            "driver": "610.57.04", "device": "/dev/dri/renderD128"},
    "session": {"type": "wayland", "display": "wayland-1"},
    "tools": {"weston": "15.0.1", "kitty": "…", "tracy": "0.13.1", "nvidia-smi": "…"}
  },
  "baseline": {"seconds": 20, "samples": 20,
               "cpu_busy_pct": 1.4, "load1": 0.6, "mem_available_kib": 51234567, "mem_available_pct": 78.1,
               "gpu_util_pct": 0, "gpu_power_w": 18.4, "gpu_power_iqr_w": 0.3,
               "gpu_clock_mhz": 360, "gpu_pstate": "P8",
               "gpu_clients": {"compute": [], "graphics": ["niri", "kitty", "noctalia-shell"]}},
  "preflight": {"verdict": "quiet", "thresholds": {"cpu_busy_pct": 10, "...": "..."},
                "lock": {"path": "…/capture-meta.lock", "owner_pid": 12345, "reclaimed": false}},
  "provenance": {
    "source": {"commit": "2ad7fc3c…", "branch": "materials-26.04", "dirty": false},
    "binaries": [{"name": "niri", "sha256": "38b10fbb…"},
                 {"name": "niri-tracy", "sha256": "ed2116ce…"}],
    "inputs": [{"name": "gpu-aurora-1.kdl", "sha256": "…"}, {"name": "glass-optic-smoke-lib.sh", "sha256": "…"}],
    "config": {"preset": "aurora", "glass.ior": "1.24", "output": "1280x720@60", "scale": "1", "vrr": "off"}
  },
  "sub_runs": [
    {"name": "gpu-plain-1", "settled_at": "…", "seconds": 10,
     "cpu_busy_pct": 1.9, "mem_available_pct": 77.8, "gpu_util_pct": 0, "gpu_power_w": 18.6, "gpu_pstate": "P8",
     "inputs": [{"name": "gpu-plain-1.kdl", "sha256": "…"}],
     "verdict": "settled"},
    {"name": "gpu-aurora-1", "settled_at": "…", "verdict": "refused",
     "reason": "gpu_power_w 24.1 exceeds baseline 18.4 by more than 1.5"}
  ]
}
```

Field rules: every measured value carries its unit in the key; hashes are full
SHA-256 hex; timestamps are RFC 3339 with offset; `host` is the hostname, never
a path. Fixtures add nothing to this file — fixture-specific results stay in
their own `manifest.json`/`analysis.json`, which may cite `capture.json` by run
id. The schema changes only by incrementing `schema`, and `show` must read
every version ever written.

## 4. Run directories

The existing convention becomes a rule:

```
$NIRI_MATERIAL_WORK_ROOT/<task-id>/<lane-or-kind>-<YYYYMMDDTHHMMSS>/
    capture.json          written by capture-meta (§3)
    SHA256SUMS            written last by the fixture; covers every other file in the tree
    <fixture files…>      configs, traces, exports, images, manifest.json, analysis.json
```

`NIRI_MATERIAL_WORK_ROOT` is per-machine (here the parent of the shared Cargo
target directory) and already required by the smoke lib. `<task-id>` is the
tasks id that authorized the run; the timestamp is the run's `started` value.
A run directory is written once: the smoke lib already refuses a non-empty `OUT`,
and `capture-meta` refuses to overwrite a section. Retained runs predating this
design keep their layout; an index treats a directory without `capture.json` as
legacy and lists it by name only.

## 5. Adoption

Two fixtures adopt the helper in this task; each adoption is a small change in
the fixture's own repository and a rerun is **not** part of the adoption.

**`docs/materials/scripts/glass-optic-smoke-lib.sh`** (this repository). Order
matters, because the lib's own guard refuses a non-empty `OUT` and `preflight`
writes into it: each smoke sources the lib first (freshness check, `mkdir -p
"$OUT"`, cleanup trap) and then calls `preflight --lane headless` itself, as its
first statement after sourcing. The lib's top level never calls preflight: the
idle-budget fixture sources the same lib in every mode, and a preflight there
would break offline `prepare` and double-preflight `trace` and `power` against
their own lock and record. The lib provides the call as a function
(`capture_preflight <lane>`) so entry scripts spell it the same way, and its
`cleanup` calls `release`, ignoring the status, since a refused run has nothing
of its own to release. The smoke then calls `build_binaries` and `identity` with
both binaries, the lib, and the calling smoke script — no case config exists
yet.

`settle` is not called from the smokes at all: it goes into the lib's
`start_nested`, which every nested launch passes through with its config path in
hand — `calibrate_probe_rect`'s geometry launch, the pixel `capture` functions,
aurora's `session` (`pinned` and `moving` never touch `capture`), and `trace_run`.
The signature becomes `start_nested <niri> <config> [<sub-run name>]`; the name
defaults to the config's basename, which is what the smokes mean (one config per
launch), and callers whose observation is not its config pass the name
explicitly. `start_nested` runs `settle --sub-run "$name" --input "$cfg"` before
launching, so every compositor launch has its preceding check and its config
hash, and a smoke that adds a launch path later cannot forget it. `finish` is
expanded to hash **every**
file under `OUT` except `SHA256SUMS` (the idle-budget fixture's `find … |
sha256sum` form); today it hashes only `*.png` and `*.kdl`, which omits the
traces, exports, binaries, and `capture.json` the contract requires. The loose
files (`kernel.txt`, `lscpu.txt`, `nvidia-smi.txt`, `source.commit`,
`installed.version`, `binaries.sha256`) are removed from the lib, since
`capture.json` carries every fact they held; the `tracy.version` and
`metrics.txt` outputs stay because they are results, not environment.

**`fixtures/idle-budget.sh`** (`niri-experiments`, `results/idle-budget`):
`prepare`, `trace`, and `power` are independent invocations, each into its own
fresh `OUT`, and measurements do not read the prepared directory — so every
invocation records its own identity. In `runtime()` the order is: the
fixture's freshness and location guards → source the smoke lib (which creates
`OUT` and installs the trap the fixture then wraps) → an explicit
`capture_preflight` in the fixture, because preflight writes `capture.json` and
either guard would abort a run that had already written it. `trace` passes
`headless`, `power` `dedicated`; `prepare` makes no preflight call, since it
launches nothing and must keep working offline on a busy machine. Preflight replaces the fixture's
`IDLE_BUDGET_DEDICATED_SESSION` gate, its `hardware.csv`/`hardware.json`, and
its tool-version files.

All three modes call `identity` after the six configs are written and validated
(today's `inputs.sha256` point), but with mode-specific arguments, because
`prepare` deliberately accepts a plain executable and copies no build sidecars,
and the applied marker patch exists only for trace builds:

| Mode | `--binary` | `--input` |
| --- | --- | --- |
| `prepare` | snapshotted `niri` | six configs, fixture files, lib, wallpaper |
| `power` | snapshotted `niri` | the above + `identity.json`, `source.tar` |
| `trace` | snapshotted `niri` | the above + `marker.patch` |

`identity` refuses a missing path (exit 1), so the fixture's per-mode argument
list is what enforces "measurement modes require build evidence; trace
additionally requires the applied patch", and the offline `prepare` validation
test keeps passing without sidecars. The analyzer checks that the build-time
identity in the sidecar and the capture-time binary hash agree in the two
measurement modes.

Observation names and config names differ in this fixture: a trace observation
is `<case>-<stimulus>-<repetition>` and a power window is
`<comparison>-<block>-<position>`, while the config is one of six `<case>.kdl`
files reused across observations. The trace lane therefore launches with
`start_nested "$NIRI" "$OUT/$case.kdl" "$name"`, keeping stimulus and repetition
in the record rather than collapsing 25 observations into repeated `A`/`B`
entries. The power lane launches niri directly on DRM and calls
`settle --sub-run "$name" --input "$OUT/$case.kdl"` itself before each of its
48 windows. The fixture's per-observation `<name>.config.sha256` becomes
redundant with the sub-run's `inputs` entry and is dropped — together with its
reader: `idle-budget.py::scene_evidence(run, name, case)` compares that file
against the retained `<case>.kdl` for every observation, and moves to
`capture.json`. It must find exactly one `sub_runs[]` entry named `name`
(none or several is an integrity failure), require its verdict to be `settled`,
and compare the entry's recorded hash for `<case>.kdl` against the retained
file. Both synthetic matrices in `fixtures/test_idle_budget.py` write
`sub_runs[]` entries instead of `.config.sha256` files, and the suite gains
cases for a missing observation, a duplicated one, a `refused` verdict, and a
hash mismatch — each rejected by `analyze`. The fixture's
`preflight.json` (GPU process reporting and device-user inventory via `fuser`)
is stricter than §2.1's client check and stays — it is the power lane's
evidence, and `capture.json` records that it ran.

Removing `hardware.json` is a migration, not a deletion: `idle-budget.py
analyze` reads it for both lanes and rejects a software renderer from it. The
analyzer's hardware and identity reads move to `capture.json`
(`environment.gpu`, `provenance.binaries`), and every synthetic evidence
directory in `fixtures/test_idle_budget.py` gains a `capture.json` in place of
`hardware.json`; the software-renderer rejection and the missing-evidence cases
are re-pointed, not dropped. The suite also gains a case that runs each mode
against a stub `capture-meta` and asserts the call order and arguments.

The fixture points at the helper through the native checkout it already knows
(`MATERIAL_ROOT/tools/capture-meta`).

## 6. Testing

`tools/test_capture_meta.py`, standard library, same GIT_* scrub as its
neighbours. Sampling is behind two injectable readers (`ProcReader` for
`/proc/stat`, `/proc/loadavg`, `/proc/meminfo`; `GpuReader` for the `nvidia-smi`
queries) so the tests feed recorded sample streams and never touch hardware.

Cases: quiet baseline passes; each threshold refuses when exceeded, naming the
field; a compute client refuses even with quiet numbers; a graphics client is
recorded, not refused, on the headless lane and refused on the dedicated lane;
an available-memory sample under the floor refuses; `dedicated` refuses under a
Wayland session; a held lock refuses, a stale lock from a dead PID is reclaimed
and recorded, two processes racing for the lock admit exactly one, `release`
by the owner removes it, `release` by a refused run leaves it and exits 1; `identity`
hashes match `sha256sum` and a dirty tree records the diff hash; `settle` within
tolerance appends a `settled` entry with its inputs' hashes, outside tolerance
(including a memory drop past 5 % of `MemTotal`) appends `refused` with the
reason and exits 1; rewriting an existing section exits 2 and leaves the file
unchanged; `show` renders a schema-1 record and refuses an unknown schema.

One integration test runs the real binary against a fake `nvidia-smi` on `PATH`
and asserts the exit code and `capture.json` end to end.

The two adoptions are verified by their own repositories' offline suites: the
smoke lib by `tools/test_glass_optic_smoke.py` (already exists; gains the call
sites and a `finish` case asserting `SHA256SUMS` names every file but itself),
the fixture by `fixtures/test_idle_budget.py` (call order per mode including
preflight after lib initialization, the per-mode identity argument lists, the
existing offline `prepare` validation unchanged, and the analyzer against
`capture.json`-shaped synthetic evidence, plus a case that `prepare` sources
the lib and makes no preflight or settle call). The smoke lib test also asserts
that `start_nested` calls `settle` with the config it is about to launch and
the explicit name when one is given, since that is now the only place the check
lives, and that sourcing the lib alone runs no preflight.

## 7. Acceptance

- `tools/capture-meta` exists with the five sub-commands, exits as specified, and
  its suite passes under `python3 -m unittest discover -s tools`.
- A `capture.json` produced against the fake `nvidia-smi` validates against the
  §3 shape; `show` renders it.
- Running `preflight --lane headless` on this host while naturally GPU-busy must
  exit 1, name the failing threshold, and list the graphics clients. Verified after
  reboot on 2026-09-12 with NVIDIA 615.71.09: exit 1 for load1 5.95, GPU utilization
  8.0%, P0/P5 states, and power IQR 26.21 W; graphics clients included niri,
  noctalia, and kitty. `show` and `release` exited 0; the disposable run was removed
  (`material-7f7aa3`).
- Both adoptions land with their suites green, including the idle-budget
  analyzer reading `capture.json`; no measurement run is claimed.
- A smoke run directory's `SHA256SUMS` lists every file in the tree except
  itself — checked by the test above, not by a hardware run.
- The evidence-doc convention points here, the hardware evidence doc's
  environment table is noted as the hand-written predecessor of `show`, and the
  future performance guide task (`material-233295`) records that it must point
  here.

## 8. Open questions and follow-ups

- Between-sub-run settle time is the fixture's choice; the first real run under
  the protocol should record how long P-state recovery takes after a Tracy
  capture so fixtures can pick a default (`material-265eb0`).
- Whether `settle` should also bound *drift* across a whole run (first vs last
  sub-run baseline) is left to the estimation task, which is the first to run
  long matrices (`material-31074f`).
- AMD/Intel samplers when a second host appears.
