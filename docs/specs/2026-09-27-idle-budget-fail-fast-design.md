# Idle-budget fixture: judge each case as it lands

**Status:** reviewed (two rounds, 2026-09-27); approved for planning. Cleanup order revised from the plan review (2026-09-27).

**Task:** `material-b15ad7`, within `material-53f873`.
**Changes:** the fixture from [the idle-budget design](2026-09-11-material-idle-budget-design.md),
on niri-experiments `results/idle-budget` (head `647f3c8`).

## Intent

The idle-budget fixture will be rerun whenever the dynamics change (the sprint's
micro-movement and settle work), and each full run takes the machine for about an
hour. The goal is that a run which is going to fail says so within minutes, that a
run which stops early still reports what it saw, and that a short pilot can be run
before anyone gives up the machine for an hour.

From the task and its notes:
- judge each case right after its export and stop at the first failure, unless
  asked for the complete matrix;
- on a settle refusal or abort, analyze the completed cases before exiting;
- a pilot mode for each lane;
- a gate that the trace reaches the end of its window (2026-09-25 Task 2 review);
- pin the seat-manager exemption to pid 1 and systemd-logind's MainPID, and grant
  it only while our niri runs (2026-09-27 Task 3 review).

Assumed, not stated: the per-case gate should be exactly the analyzer's existing
gate, not a second, weaker check. The full run should also put the most
informative cases first (see *Case order*).

## Evidence

- The first complete TTY trace (`trace-20260925T043128`) ran 58 min to report a
  failure that its first case already showed about 2 min in: one redraw 3 s after
  the stimulus, from kitty's `mouse_hide_wait`.
- The same redraw was already in the completed cases of two refused runs on
  2026-09-24 (`trace-20260924T163317`, `trace-20260924T165644`). The fixture
  analyzes only after all 25 cases, so nobody read those cases.
- The pilots that found the cause took 19 min in all. They were run by hand, as
  were the three power pilots (`power-pilot-20260927T02*`, sham block 1 with
  per-window `power_observation`). Those pilots found three power-lane defects
  within their first windows.

Today `runtime` runs every case, then writes `manifest.json`, then runs
`idle-budget.py analyze` once over the whole declared matrix. On `fail` (a settle
refusal, an IPC failure, a missing client), the `EXIT` trap tears the scene down
and leaves no analysis at all.

## Design

### 1. The run declares its plan first

`idle-budget.py manifest` runs before the first case rather than after the last.
The manifest records `mode`, `pilot` (bool), `fail_fast` (bool), and the ordered
`observations` list for that plan. `analyze` checks the observations against the
matrix that the flags imply, as it now does against the full matrix, so a pilot
cannot pass as a full run.

### 2. Run-level checks before the first case

The run-level half of `analyze` becomes `check_run(run)`: identity, capture lane,
quiet preflight, hardware identity, and the capture-time binary hash, plus the
power lane's tty-session and preflight-inventory checks. It needs only artifacts
that exist after `capture_meta identity` (and after `preflight` for power).
The order is: `manifest`, then power's `preflight`, then `idle-budget.py
check-run "$OUT"`, then case 1. A run whose identity or preflight evidence is
bad stops before its first capture, not 58 min later. The manifest already
exists at that point, so the stop still gets an analysis (§4). `analyze` calls
the same `check_run` and reports a failure there as a run-level integrity
failure, with every case under `not_run`, rather than raising.

### 3. A verdict per case, and stopping on the first failure

After each case's last artifact is written, the fixture runs
`idle-budget.py observe "$OUT" <name>`:
- trace: after `csvexport --gpu`;
- power: after `stop_scene`, which follows the after-snapshots.

`observe` runs the existing `trace_observation` or `power_observation` for that
one item. It writes `<name>.verdict.json` (`{"observation", "passed", "summary" |
"error"}`), prints one line to stderr (`A-move-1 pass: 0 redraws, 0 material
draws, pixels equal`), and exits 1 on an integrity error or a failed behavioural
gate.

It also checks the output mode against a baseline. The baseline is the output of
the first case, in manifest order, whose verdict carries a `summary`: its
observation was reconstructed, whether or not its gate passed. A verdict holding
only an `error` has no output and is skipped. Until a case yields a summary there
is no baseline, and the first one to yield a summary becomes it. `analyze`'s
cross-run check already compares against the first summary, so the two agree.

With `fail_fast` (the default), a failing `observe` stops the run: the fixture
calls `fail "first failure: <name>"` and the stop path (§4) takes over. With
`--inventory`, the fixture records the failure and runs every case, as it does
today.

A verdict file marks a case complete. A case interrupted before its verdict is
not run: its raw files stay in `OUT` and no analysis claims them.

### 4. Every exit analyzes what completed

**Stop reason.** The fixture keeps `CURRENT_CASE`, set when a case starts and
cleared once its verdict is written. It writes `$OUT/stop.json` (`{"exit",
"during", "kind", "message"}`) from three sources. Whichever writes first wins,
so a specific message is never overwritten by a generic one:
- `fail`: the fixture redefines the lib's `fail` after sourcing it. Lib functions
  call `fail` by name, so they get the new one too. It writes `kind: "fail"`
  with the message, or `kind: "first-failure"` when a failing `observe` stopped
  the run.
- Signals: the INT and TERM traps record `kind: "signal"` and the signal's name
  in a variable before exiting with 130 or 143, as now.
- Command failures: the fixture sets `set -E` and an `ERR` trap that records the
  failing command, its line and its status. A `set -e` exit then has a reason.
  Commands in conditional contexts (`||`, `if`) do not fire `ERR`, which matches
  `set -e`.

On a non-zero exit with no `stop.json` yet, `cleanup` writes one from the signal
or `ERR` record. If neither exists it writes `kind: "exit"` with the status alone.

**Cleanup order.** The fixture's `cleanup` no longer calls the lib's `cleanup`.
That function ends in `exit`, and its `stop_weston` repeats a fatal socket check
through `fail`, which would exit before the runtime directory is removed and the
capture lock released. The teardown cannot move into a subshell either: niri, the
capture and weston are the parent shell's children, and a subshell cannot `wait`
on them. The fixture's `cleanup` does the whole teardown itself, from the lib's
building blocks (`remove_runtime_dir`, `capture_meta release`), in this order:
1. Record `rc=$?` and the pending signal or `ERR` record, then `set +e` and
   `trap - ERR`. `ERR` fires even under `set +e`, and without this a failing
   teardown command would be taken as the run's stop reason. Replace the INT and
   TERM traps with one that only notes a second signal. A trap with a handler,
   unlike an ignored signal, resets to the default in children, so Ctrl-C still
   reaches a running analysis.
2. Stop every process that writes into `OUT`, and reap each: the clients, the
   power sampler, the capture, niri, and then weston. A signal caught by a trap
   interrupts `wait` (status > 128) while the child still runs, so `reap` waits
   again until the process is gone; a zombie still answers `kill -0` until it is
   reaped. The clients and the sampler run in their own `setsid` groups, whose
   other members are not our children, so `reap_group` also polls, for up to 5 s,
   until the group is empty. Weston logs its shutdown (`caught signal 15`) to
   `$OUT/weston.log`, so it must be gone before the checksum. `reap_weston` kills
   and reaps `WESTON_PID`, clears it, and waits up to 5 s for the host socket to
   go. None of these steps calls `fail`: a group or socket that outlives its
   bound is recorded as a teardown error.
3. Remove the runtime directory (`remove_runtime_dir`); a failure is a teardown
   error. Each teardown error goes to `$OUT/teardown.json`, which `analyze`
   reports under `teardown`, and marks the run's teardown failed.
4. Write the fallback `stop.json` (above).
5. In trace and power mode, once `manifest.json` exists, run `timeout 300
   idle-budget.py analyze "$OUT" > analysis.json`, then write `SHA256SUMS`.
6. Seal the directory: set `SEALED=1`. From then on, the `fail` redefinition,
   the stop recorders and the teardown recorder write nothing into `OUT`.
7. `capture_meta release`, which reads `capture.json` and unlinks the global
   lock outside `OUT`. It stays last so that no other capture starts while this
   one is still analyzing. Then `exit` with the status below.

**Exit status.**
- A non-zero run status (`fail` 1, 130, 143, or a failed command's status) is
  kept.
- On a zero run status, a non-zero or timed-out analysis makes it 1, and so do
  a teardown error from steps 2–3 and a `SHA256SUMS` failure.

The success path is the same one: `runtime` no longer runs the analysis or
writes `SHA256SUMS` itself.

`analyze` recomputes every completed case from raw, as now. The verdict files
decide only which cases are complete. It adds these fields:
- `pilot`, `complete` (every declared case has a verdict), `not_run` (declared
  cases without one);
- `stopped`: `null` on a complete run. Otherwise it holds `stop.json`, plus the
  settle refusal's reason from `capture.json` when the stop was a refusal.

The power comparisons and `budget_passed` are computed only for a complete,
non-pilot power run. `analyze` exits non-zero for an incomplete run, as for any
integrity failure.

### 5. Pilot mode

`--pilot` runs a declared subset that exercises every gate once through its
verdict:
- **trace**: `A-move-1` (the quiet window, pixel equality, and the move journal),
  `A-resize-1` (the resize journal), `C-move-1` (the 79–81 cadence), and
  `D-move-1` (the 39–41 cadence). About 9 min after the build and preflight.
- **power**: sham block 1 (`sham-1-1` to `sham-1-4`), as in the hand-run pilots.
  It covers the settle, P8 rest, the inventory and seat exemption, the 1 Hz
  schedule, and the window median. About 10 min. Its analysis also reports the
  block's repeat spreads (|w1−w4|, |w2−w3|) and block delta against the 1.0 W
  target, marked descriptive: one block is not a precision result.

A pilot fails fast like a full run, and `--pilot --inventory` runs the whole
subset. Pilot output goes to its own `OUT` like any run; the name is the
operator's choice, as today.

### 6. Case order

The full trace runs repetition-major: repetition 1 of every kind (`A-move`,
`A-resize`, `B-move`, `B-resize`, `P`, `C`, `D`, `O`), then repetition 2, then
3, then the 600 s hold last. A failure specific to one kind then shows within
the first 8 cases (about 17 min) instead of as late as case 22 (`O-move-1`). The
power lane's ABBA order within blocks is part of its measurement design and does
not change.

Execution follows the manifest, not loops of its own. Today `trace_all` and
`power_all` nest shell loops that repeat the matrix, and `power_all` calls
`power_case` through inline Python. Both are replaced by one loop over
`idle-budget.py plan "$OUT"`, which reads `manifest.json` and prints one TSV row
per declared observation in order:
- trace: `name case stimulus repetition`;
- power: `name case`, with the case resolved by `power_case`.

The shell captures the rows with a command substitution, `plan=$(…)`, so a failure
stops the run under `set -e`. It reads them into an array with `mapfile`,
because the case functions must not read from the loop's stdin. The manifest is
then the only statement of both the set and the order, and a pilot needs no
separate loop.

### 7. The trace must reach its window's end

`trace_observation` adds a gate: the latest CPU zone end (start + duration over
all zones) is at or past the window end. Today the heartbeat coverage check
alone would accept a trace that ended up to 1.5 s early.

### 8. The seat-manager exemption

`inventory` records the seat managers by pid: `{"systemd": 1, "systemd-logind":
<MainPID>}`, read once per inventory with `systemctl show -p MainPID --value
systemd-logind`.

The compositor is passed explicitly, never inferred from `required`: both
`collect` and `power_observation` turn `required` into a set, so it has no
first element. `inventory` and `check_inventory` take a `compositor` argument
(a pid, or `None`). `collect` takes it as its own argument, which `power_one`
passes as `$NIRI_PID`, and writes it to `<name>.interval.json`. `power_observation`
refuses the window unless the recorded `compositor` equals the scene's recorded
niri pid (`pids[0]` from `<name>.pids`), and rechecks each stored inventory with
it.

`check_inventory` exempts a device holder only if all of these hold:
- its pid is one of the two recorded seat-manager pids;
- its user is root;
- it holds only `/dev/dri/card*` nodes;
- nvidia-smi does not list it;
- `compositor` is not `None`, is in `required`, and itself holds a
  `/dev/dri/card*` node in the same report.

With `compositor=None`, as in the preflight inventory, nothing is exempt. That
matches the 2026-09-27 evidence: the preflight inventory had no holders at all,
and pid 1 and logind appear only beside our niri on `/dev/dri/card1`. A MainPID
that is missing or zero refuses the inventory.

## Interfaces

| Where | Change |
| --- | --- |
| `idle-budget.sh` | `trace` and `power` accept `--pilot` and `--inventory`; one loop over `plan` replaces `trace_all`/`power_all`'s nested loops (`trace_one`, a new `power_one`); `fail` redefinition, signal and `ERR` records, `stop.json`; `observe` after each case; `cleanup` that reaps with retried waits, removes the runtime directory, analyzes, checksums, seals, and releases, without the lib's `cleanup`; `idle-budget-supervise.sh` for live runs |
| `idle-budget.py` | `check_run`; `check-run`, `observe` and `plan` subcommands; `manifest` takes `--pilot` / `--inventory`; `pilot_matrix`; repetition-major `trace_matrix`; partial `analyze`; trace-end gate; seat exemption pinned by pid with an explicit `compositor` through `inventory`, `check_inventory`, `collect` and `interval.json` |
| `idle-budget.just` | `trace *flags`, `power *flags` pass the flags through |
| `test_idle_budget.py` | cases below |
| `docs/results/2026-09-11-idle-budget.md` | reproduction section names the pilot and `--inventory` |

No change to `glass-optic-smoke-lib.sh`, `capture-meta`, or niri.

## Testing

Offline, in `test_idle_budget.py`:
- `observe` on synthetic trace and power items writes a verdict and exits 1 on a
  failed gate, an integrity error, and an output-mode change.
- The output baseline: case 1 fails integrity (an `error`-only verdict), case 2
  is reconstructed and becomes the baseline, and case 3 passes with case 2's
  output but fails with a different one. A failed-gate verdict that carries a
  `summary` also serves as the baseline.
- `plan` prints the manifest's rows in manifest order for the full trace, the
  trace pilot, the full power run and the power pilot, with power cases resolved
  by `power_case`.
- `analyze` on a run with verdicts for some declared cases reports those cases,
  `complete: false`, `not_run`, and `stopped` from `stop.json` and from a refused
  sub-run in `capture.json`. It computes no power comparison for an incomplete or
  pilot run, and reports the pilot's descriptive block.
- The manifest matrix check refuses a pilot subset declared as a full run, and
  the reverse.
- `trace_matrix` is repetition-major and still equals the old matrix as a set.
- The trace-end gate refuses a trace ending 1 s before the window end whose
  heartbeat coverage passes.
- The seat exemption: pinned pids beside our compositor on a card node pass.
  They are refused with `compositor=None`, with a non-pinned root `systemd`,
  with a holder on a render node, and with a missing MainPID. They are also
  refused when a kitty in `required` holds a card node but the compositor does
  not, so a client cannot stand in for the compositor. `power_observation`
  refuses an `interval.json` whose `compositor` differs from `pids[0]`.
- Shell: `idle-budget.sh` sourced, with the case functions (`trace_one`,
  `power_one`), the analysis and `capture-meta` stubbed, and the stubs logging
  their calls:
  - The case functions are called in manifest order for each of the four plans.
  - A failing `observe` stops the loop after that case, and `--inventory`
    continues past it.
  - `fail` mid-run, TERM mid-case, INT mid-case, and a failing plain command
    (`false` under `set -e`) each leave a `stop.json` of the matching `kind`
    naming the case in progress. `fail`'s message survives the fallback. Each
    leaves an `analysis.json` covering exactly the completed cases, the exit
    statuses 1, 143, 130 and the command's own, and `capture_meta release`
    called last, after the seal.
  - A clean run whose analysis fails exits 1, and so does a run whose analysis
    times out.
  - The weston stub appends a shutdown line (`caught signal 15`) to
    `$OUT/weston.log` when it is killed. After the fixture has exited, `sha256sum
    -c SHA256SUMS` in `OUT` passes for a clean run and for a TERM mid-case. The
    stub's shutdown line is present and covered by the sums.
  - A second signal while cleanup reaps (a weston stub, and separately a client
    stub, that on TERM signals the fixture and then writes its shutdown line
    0.3 s later): the interrupted `wait` is retried, the line lands before the
    checksum, and `sha256sum -c` passes after exit.
  - A weston stub whose socket lingers: the run exits 1, `analysis.json` lists
    the teardown error, the runtime directory is removed and `capture_meta
    release` runs, and no file in `OUT` changes after `SHA256SUMS`
    (`sha256sum -c` still passes after exit, and there is no `stop.json`).
  - A teardown command failing inside `cleanup` does not replace the stop reason
    recorded from the run.

Live: a trace pilot and a power pilot on a TTY with the desktop stopped (see the
settle-gated capture practice). Each must pass, leave per-case verdicts, and
produce an `analysis.json` with `pilot: true`. One deliberately interrupted
trace pilot (Ctrl-C after the second verdict) must leave an analysis of the
completed cases with `stopped` set. Each run goes through
`idle-budget-supervise.sh`: it runs the command in its own process group,
with INT at its default, and returns only when every process in the group has
exited. It delivers INT and TERM to the fixture's shell alone: `tt` runs its
command with `subprocess.run`, which SIGKILLs the child when `tt` itself is
interrupted (ops-8fe6c9), so a Ctrl-C reaching `tt` would cut the fixture's
cleanup short. The wrappers (`just`, `tt`) can return before the fixture's
cleanup has finished, and `analysis.json` exists from the moment its redirection
opens, so neither the prompt nor the file says the run is done. These need the
machine and are parked `quiet` for the person.

## Rejected alternatives

- **A background watcher judging cases as they land**, like the one that
  watched the passing run. It races the next case's settle and capture, adds CPU
  load inside capture windows, and needs a signal to stop the run. Judging
  between cases is deterministic, and its cost falls where `csvexport` already
  runs.
- **A per-case verdict ledger in `capture-meta` for every fixture.** It has one
  consumer today. If a second fixture wants it, file that then.
- **The task's two-case trace pilot (A-move-1, C-move-1)** misses the resize
  journal and D's cadence range. **One case of every kind** (8 cases, about
  17 min) repeats what the repetition-major full run's first 17 min shows anyway.

## Risks

- `observe` adds seconds of CPU work right before the next case's settle. The
  existing `csvexport` step already runs there and the settle gate passed after
  it in both valid runs; the live pilot confirms that `observe` does not trip it.
- The stricter seat exemption could refuse a DRM session that the current rule
  accepts. The power pilot is the check, and the evidence above says it should
  pass.
