# Captures

Type: protocol, data model; document types 1. The `capture.json` record is
schema version 1, the only version `scripts/capture/capture-meta` writes or reads. The
gate rules are version `2026-10-17` (`RULES` in `scripts/capture/capture-meta`), which
every preflight records as `rules`; a record without it was judged by the
earlier rules, which failed every flagged scan entry and every
out-of-tolerance window. This covers how a capture harness records a
performance or pixel run: the lanes and what each may claim, the ordered
exchange with `capture-meta`, the settle and evidence gates, the disturber
hold, the record's fields, the harnesses under `scripts/capture/`, and how a
run fails. What a host needs first is in
[capture-host-setup.md](capture-host-setup.md); what the measurements found
is in [performance.md](performance.md).

## Participants

- **Fixture:** a harness under `scripts/capture/` (see [Harnesses](#harnesses)).
  It builds, launches compositors, drives stimuli, writes its own results
  (`manifest.json`, `analysis.json`, traces, images) and, last,
  `SHA256SUMS` over the run directory.
- **`capture-meta`** (`scripts/capture/capture-meta`): owns `capture.json`, the lock and
  the gates. Exit status 0 passed, 1 refused (the run must not proceed),
  2 cannot run (the tool could not do its job). A fixture stops on 1 and 2 alike.
- **The hold** (`scripts/capture/capture_hold.py`): stops and restores the host's
  disturbers, keeps the host hold file and scans the journal.
- **The guard:** a transient user unit, `capture-meta-guard-<key>.service`,
  running `capture-meta guard <run-dir>`. It watches the fixture's pid and the
  held monitors once a second and restores the hold if the fixture dies.
- **The host:** the systemd user and system managers and their journals,
  `nvidia-smi`, ops's `host-load`, a desktop niri and Noctalia when one runs,
  and the DRM connectors in `/sys/class/drm`.

## Lanes

`--lane` names the route a run claims. Frozen-clock renders under
`cargo test` (`src/tests/`) use no lane and no `capture-meta`.

| Lane | Topology | May claim | May not claim | Preflight |
| --- | --- | --- | --- | --- |
| `pixels` | niri nested under headless Weston | Pixels of a static scene: equality and AE between shots in one run; review sheets | Draw times, redraw counts or rates, fps, wall-clock motion | Lock and partial hold (user timers, declared services); no sampling |
| `headless` | niri nested under headless Weston | Draw-time medians, redraw and draw counts and rates, cadence, fps, pixels sampled at wall-clock instants | Board power | Full hold, 1 Hz sampling, quiet judgement |
| `dedicated` | niri on DRM from a TTY, desktop stopped | DRM timing, VT resume, unlock, screencast, board power | — | As `headless`, plus a TTY session and no GPU client of any kind |

A value is a measurement if it could change when the host is slower or the
GPU is shared; a fixture that records one runs on a measured lane
(`headless` or `dedicated`). On the dedicated lane preflight refuses unless
`XDG_SESSION_TYPE` is `tty`, `DISPLAY` and `WAYLAND_DISPLAY` are unset and no
desktop niri answers. `optic-settling-smoke.sh --lane dedicated` also needs
`DRM_OUTPUT` and `DRM_MODE`, exactly one connected output, the `chvt` sudo
rule and `pipewiresrc`; it switches VTs through `vt-lib.sh` and restores the
starting VT on every exit before releasing (up to three attempts, each
verified within 2 s, recorded in `vt-restore.json` as `not-needed`,
`restored` or `failed`).

`host_condition` records what preflight observed: `tty` (a TTY session and no
live desktop socket) or `desktop`. Preflight does not refuse a desktop on the
`headless` lane; the quiet queue runs measured lanes from a TTY.

## Exchange

1. **Build, then wait.** The fixture builds its binaries before preflight and
   waits out the build's load (`await_load`), so the hold and the baseline
   exclude it. A run directory (`OUT`) must be fresh or empty.
2. **`preflight <run-dir> --lane L --task ID --fixture NAME`** (`--seconds`,
   default 20; `--owner-pid`, default the invoking process, capture-meta's
   parent pid; `--threshold KEY=VALUE`; `--tool NAME[=VERSION]`;
   `--hold-settle`, default 10 s on a measured lane, 0 on `pixels`). In order: write `run`; take the lock; restore
   any stale hold whose owner died; refuse on the pre-hold checks (session,
   live desktop on `dedicated`, forbidden GPU clients); write the host hold
   file, start the guard and confirm it is active; apply the hold and write
   `hold`; wait the hold settle; write `environment`. On `pixels`, write
   `preflight` with verdict `unsampled` and stop. Otherwise sample at 1 Hz,
   write `baseline`, judge quietness, and write `preflight` (`quiet` or
   `refused`). Any failure after the hold rolls it back.
3. **`identity <run-dir> --source <checkout>`** with `--binary`, `--input` and
   `--config KEY=VALUE`: writes `provenance` once, after binaries and static
   inputs exist.
4. **Per sub-run** (case, round or cell), after the previous compositor has
   exited and the sub-run's inputs are written: on a measured lane the shared
   lib waits for 3 s of consecutive P8 readings (at most 60 s), then
   **`settle <run-dir> --sub-run NAME --input <cfg>`** (`--seconds`, default
   10) samples and appends a `settled` or `refused` entry. On `pixels`,
   **`begin`** appends a `begun` entry with its inputs and no samples.
5. **`renderer <run-dir> --sub-run NAME --niri-log <log>`** (and
   `--weston-log` on `headless` and `pixels`; refused on `dedicated`) judges
   that launch's renderer lines before any stimulus. One check per sub-run.
6. The fixture drives its stimuli and captures.
7. **`finish <run-dir> --sub-run NAME`** once the compositor has stopped:
   stamps `finished` and `duration_s`. A sub-run cut short is never finished.
8. **`release <run-dir>`** from the fixture's exit trap, every exit: restores
   the hold, scans the journal once, writes `hold_end`, refuses a measured run
   with a settled sub-run whose renderer is unverified, releases the lock and
   stamps the run's `finished`.
9. The fixture writes `SHA256SUMS`. A later stamp by recovery refreshes its
   `capture.json` line.

Out of band: **`show <run-dir>`** prints the record in a fixed layout;
**`judge <run-dir>`** re-judges a record under the current rules and writes
nothing; **`restore`** restores a hold whose owner is dead, by hand.

## Settling

Preflight samples once a second for `--seconds`: CPU busy %, 1-minute load,
available memory, and from `nvidia-smi` (exactly one GPU) utilization, power,
graphics clock, P-state and the GPU clients. The baseline is each value's
median, the power IQR, and the sets of P-states and clients seen. Quiet means,
with the default thresholds:

| Threshold | Default | Rule |
| --- | --- | --- |
| `cpu_busy_pct` | 10 | Median at most this |
| `load1` | 2.0 | Median at most this |
| `mem_available_pct_min` | 20 | Every sample at least this % of `MemTotal` |
| `gpu_util_pct` | 5 | Median at most this |
| `gpu_pstate` | P8 | Every sample in this P-state |
| `gpu_power_iqr_w` | 1.0 | Power inter-quartile range at most this |

No compute client may appear in any sample; on `dedicated` no graphics
client either. A refusal on CPU or load attaches `host-load`'s top five
processes as `host_load` and names them in the message.

A settle window passes when its medians stay within tolerance of the
baseline: `settle_cpu_busy_pct` 5 points, `settle_mem_pct` 5 points,
`settle_gpu_util_pct` 3 points, `settle_gpu_power_w` 1.5 W, with the P-state,
client and memory-floor rules as above. Settle never waits or retries; how
long to wait is the fixture's choice, and a refused settle stops the run with
the reached sub-run recorded. Thresholds can be overridden per run with
`--threshold` and are recorded with the verdict.

## Evidence gates

The rules (`2026-10-17`) fail a run on evidence that something competed with
the measurement or that a claimed quantity changed, and annotate anything else
they observe. Contention is a CPU, memory or utilization median outside
tolerance, a P-state other than the threshold's, a forbidden GPU client, a
held disturber that ran, a held monitor that woke, or, on `headless`, a power
change while a GPU client exists that could cause it. Board power is claimed
only on `dedicated`.

- **Settle windows.** On `headless`, when no sample in the window lists a
  graphics or compute client, a power departure beyond `settle_gpu_power_w` is
  annotated `unclaimed-power-step`, and a power IQR above `gpu_power_iqr_w` is
  annotated `unclaimed-power-spread`. Otherwise a power IQR is not judged in a
  settle window on any lane (the preflight baseline does judge it on measured
  lanes), and a power departure with a client present follows the rule above.
  Every other kind of departure fails. An entry with annotations and no
  reasons is `settled`; settle prints each annotation and exits 0.
- **Release scan.** A `restart` of `getty@ttyN.service` or
  `autovt@ttyN.service` (a console login on another VT) is annotated
  `vt-login` on `headless` and `pixels` and fails on `dedicated`. Every other
  flagged kind fails on every lane, as does everything when the lane is
  unreadable.
- **Renderer.** On a measured lane every niri and Weston renderer line must
  contain `environment.gpu.name` and must not name a software rasterizer
  (`verified`); otherwise the check refuses at once. On `pixels` the lines
  are only `recorded`.
- **Re-judging.** `judge` keeps a recorded preflight verdict, re-applies the
  settle rules to every sampled window (missing thresholds at today's
  defaults) and re-classifies the scan. Each part is pass, fail or no
  evidence; it exits 1 on a fail and 2 on no evidence.

## Disturber hold

What is held depends on the lane and on whether a desktop niri answers
(`NIRI_SOCKET`, else the one live `niri.*.sock` in `$XDG_RUNTIME_DIR`; more
than one refuses):

| Item `kind` | Held on | Action | Restore |
| --- | --- | --- | --- |
| `timer` | every lane | Each active, non-transient user timer is stopped | `systemctl --user start` |
| `service` | every lane | Each running unit named in `${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold` is stopped | `systemctl --user start` |
| `idle` | measured lanes, desktop up | `noctalia msg caffeine-enable` (prior state `unknown`) | `noctalia msg caffeine-disable` |
| `monitors` | measured lanes, desktop up | `power-off-monitors`; each lit connector must read `dpms` `Off` within 5 s | `power-on-monitors` |

System timers need root and are not held; their names are recorded and a
firing is flagged. Transient timers, declared services that are not running
and an absent Noctalia are recorded under `not_held`.

The host hold file, `$XDG_RUNTIME_DIR/capture-meta.hold.json`, is written
before each action (write-ahead), so a kill leaves a file naming everything
that may have changed. A restore marks it `restoring`, fixes the scan's end,
undoes still-held items in reverse order, writes `hold_end` and only then
removes the file. Whoever restores a hold the fixture never released (the
guard, the next preflight or `restore`) also stamps the run's end and names
itself in `finished_by`; the guard retries a failed restore 20 times, 30 s
apart.

The scan reads both journals over the held interval and flags:
`held-started` (a held unit started), `timer-added` (a new timer started),
`timer-fired` (a timer's service activated), `restart` (a unit restarted),
`monitor-woke` (a held connector read `On`) and `monitor-unwatched` (a
connector's `dpms` could not be read).

## Capture JSON

One `capture.json` at the top of the run directory, `"schema": 1`. The
`environment`, `baseline`, `preflight`, `provenance` and `hold` sections are
written once by their verb, and a second write is refused (exit 2), so a run
directory cannot be half rerun. The rest change after their first write:
`sub_runs` entries are appended, the restore attempts of `hold_end` accumulate across
retries, and the `finished` time of `run` is stamped later by release or a recoverer. Times are RFC 3339 with offset (the
`_us` fields are epoch microseconds), hashes full SHA-256, and measured values
carry their unit in the key. Fixtures add nothing to it.

| Section | Writer | Fields |
| --- | --- | --- |
| `run` | preflight; release or a recoverer | `id` (the run directory's name), `task`, `fixture`, `lane` (`pixels`, `headless`, `dedicated`), `started`, `host` (hostname); then `finished`, `duration_s` (whole seconds), `finished_by` (`release`, `guard`, `next-preflight`, `hand`; a recoverer's time is an upper bound) |
| `environment` | preflight | `kernel`, `cpu`, `cpu_threads`, `memory_total_kib`, `session` {`type`, `display`}, `tools` {name: version string}, `gpu` {`name`, `uuid`, `driver`, `device`} when a GPU sampler exists |
| `baseline` | preflight, measured lanes | `seconds`, `samples`, `cpu_busy_pct`, `load1`, `mem_available_kib`, `mem_total_kib`, `mem_available_pct`, `gpu_util_pct`, `gpu_power_w`, `gpu_power_iqr_w`, `gpu_clock_mhz`, `gpu_pstates` (sorted list), `gpu_clients` {`compute`, `graphics`: process names} |
| `preflight` | preflight | `verdict` (`quiet`, `refused`, `unsampled`), `at`, `reasons` (list), `thresholds` (every key above), `lock` {`path`, `owner_pid`, `run_id`, `run_dir`, `reclaimed`, `previous_owner_pid` when reclaimed}, `host_condition` (`tty`, `desktop`), `rules`, `host_load` (the `host-load` report, carrying `load.top`) |
| `provenance` | identity | `source` {`commit`, `branch`, `dirty`; when dirty `diff_sha256` over `git diff HEAD` and each untracked file, and `untracked`}, `binaries` and `inputs` (lists of {`name`, `sha256`}), `config` {free `KEY`: `VALUE` strings} |
| `hold` | preflight | `started`, `started_us`, `items` (as in [Disturber hold](#disturber-hold), each with `kind`, `action`, `restore`, and `unit`, `socket`, `connectors`, `prior`, `locked` where they apply), `not_held` ({`kind`, `unit`, `reason`}), `socket`, `config` (path or `absent`), `desktop` (`present`, `absent`), `timer_map`, `active_at_hold`, `system_timers`, `connectors`, `guard`, `owner_pid`, `recovered` (null, or the stale run restored first) |
| `hold_end` | release or a recoverer | `restore` {`state` (`complete`, `failed`), `attempts` [{`at`, `by`, `undone`, `failures`, `notes`}]}, `scan` {`verdict` (`clean`, `annotated`, `disturbed`, `unscanned`, `not-run`), `until_us`, `disturbances`, `error` when unscanned, `preflight_exit` when not run}, `cleanup` {`guard`, `owner_pid`} |
| `sub_runs` | settle, begin, renderer, finish | List of entries, below |

A disturbance has `at`, `at_us`, `manager` (`user`, `system`, `sysfs`),
`unit`, `kind`, `severity` (`fail`, `annotate`) and `rule` (`vt-login` or the
kind). A sub-run entry has `name`, `started`, `inputs` and `verdict`
(`settled`, `refused`, `begun`). A sampled entry adds `settled_at`, `seconds`,
the baseline's summary fields, `gpu_power_step_w` (its power minus the previous
settled entry's, or the baseline's), `reason` when refused and `annotations`
({`rule`, `detail`}). `renderer` holds `at`, `expected` (measured lanes), a
`niri` and optional `weston` result {`verdict` (`verified`, `mismatch`,
`missing`, `recorded`), `lines`} and an overall `verdict` (`verified`,
`refused`, `recorded`). `finish` adds `finished` and `duration_s`.

`capture-meta` rejects a record whose `schema` is not 1, whose root is not an
object, or whose sections have the wrong shape (a section that is not an
object, `provenance` lists without string `name` and `sha256`, a `hold_end`
restore state outside `complete`/`failed`, a scan verdict outside the five,
sub-run entries without string `name` and `verdict`).

## Harnesses

Every harness that writes `capture.json` takes `CAPTURE_TASK` (the task
authorizing the run, recorded as `task`) and `NIRI_MATERIAL_WORK_ROOT` (the per-machine evidence root, which
also holds the retained Tracy 0.13.1 tools). Those sharing
`glass-optic-smoke-lib.sh` also take `OUT`, a fresh directory; the lib owns
Weston, niri, the settle-before-launch and renderer calls, and the exit trap.
The three clip fixtures make their own run directory under the evidence root.

| Lane | Harnesses in `scripts/capture/` |
| --- | --- |
| `pixels` | `glass-edge-sheet.sh`, `glass-noise-layers-smoke.sh` |
| `headless` | `glass-aurora-smoke.sh`, `glass-iridescence-smoke.sh`, `glass-noise-site-smoke.sh`, `glass-render-order-smoke.sh`, `noise-layers-cost.sh`, `noise-placement-cost.sh`, `hidden-window-attribution.sh`, `view-tilt-cost.sh`, `focus-swap-clips.sh`, `drag-lag-clips.sh`, `ring-motion-clips.sh` |
| `headless` or `dedicated` | `optic-settling-smoke.sh prepare\|pilot\|matrix [--lane headless\|dedicated]`; a `matrix` needs `PILOT_DIR`, and `CASES` limits a development run, which never passes as a pilot |
| no `capture.json` | `material-signals-smoke.sh`, `focus-ring-light.sh`, `glass-parameter-sweep.sh`, `noctalia-surfaces.sh` |

Support files: `vt-lib.sh` (verified `chvt` switching), `idle-inhibit-client.c`
and `session-lock-client.c` (built per run), `vdrag.py` (a scripted drag),
`focus-glass-content.sh`, and offline analysis scripts (`*.py`). On the
dedicated lane, `scripts/capture/screencast_consumer.py` is the screencast client: a
ScreenCast session on the case's private bus recording one monitor with the
cursor hidden, consumed as DMA-BUFs through GStreamer GL; it prints
`ready <node>`, journals each frame's `CLOCK_MONOTONIC` arrival, saves a
frame on `SIGUSR1` and writes a summary on `SIGTERM`.

## Failures

| Exit | Where | Meaning and remedy |
| --- | --- | --- |
| 1 | preflight | Lock held by a live run, a pre-hold check, or a host that is not quiet; `reasons` names each. Wait or remove the load; any hold is already rolled back. |
| 2 | preflight | No `nvidia-smi` on a measured lane, no `host-load`, `XDG_RUNTIME_DIR` unset, more than one live desktop socket, a guard that did not start, a monitor that did not power off, an unknown declared service, a malformed lock or hold file, or a section that already exists. |
| 1 | settle | Window outside tolerance, or the lock no longer names this run. The fixture stops; the entry records why. |
| 1 | renderer | A measured launch whose renderer is missing, another GPU or software. |
| 1 | release | Scan `disturbed`, a preflight refusal rolled back, launches without a verified renderer, or a lock that is not this run's (left in place). |
| 2 | release | Hold not fully restored (each item's `restore` command is named), or the journal scan failed (`unscanned`; not evidence until rescanned). |
| 1 / 2 | judge | Fails under the current rules / no evidence (unreleased hold, unscanned, malformed record). |

A run with a failed restore keeps its hold file, guard and lock; `release`
retries, and `capture-meta restore` restores by hand once the owner is dead.

Known gaps: GPU sampling supports one NVIDIA GPU through `nvidia-smi`, so
measured lanes cannot run elsewhere. A restored timer re-anchors its
`OnActiveSec` trigger, so a held run can cause off-schedule timer runs after
release (see [capture-host-setup.md](capture-host-setup.md#disturber-hold)).
Per-sample memory readings are not kept, so `judge` lets a refusal on the
memory floor stand. Nested measurements with the desktop up and idle are not
admitted; measured lanes run from a TTY.
