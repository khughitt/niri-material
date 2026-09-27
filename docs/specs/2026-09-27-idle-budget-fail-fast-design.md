# Idle-budget fixture: judge each case as it lands

**Status:** draft for review.

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
that exist after `capture_meta identity` (and after `preflight` for power). The
fixture calls `idle-budget.py check-run "$OUT"` once before case 1, and `analyze`
calls the same function. A run whose identity or preflight evidence is bad stops
before its first capture, not 58 min later.

### 3. A verdict per case, and stopping on the first failure

After each case's last artifact is written, the fixture runs
`idle-budget.py observe "$OUT" <name>`:
- trace: after `csvexport --gpu`;
- power: after `stop_scene`, which follows the after-snapshots.

`observe` runs the existing `trace_observation` or `power_observation` for that
one item. It checks the output mode against the first case's `<name>.verdict.json`,
writes `<name>.verdict.json` (`{"observation", "passed", "summary" | "error"}`),
prints one line to stderr (`A-move-1 pass: 0 redraws, 0 material draws, pixels
equal`), and exits 1 on an integrity error or a failed behavioural gate.

With `fail_fast` (the default), a failing `observe` stops the run: the fixture
calls `fail "first failure: <name>"` and the stop path (§4) takes over. With
`--inventory`, the fixture records the failure and runs every case, as it does
today.

A verdict file marks a case complete. A case interrupted before its verdict is
not run: its raw files stay in `OUT` and no analysis claims them.

### 4. Every exit analyzes what completed

The fixture wraps the lib's `fail` so the message is also written to
`$OUT/stop.json` with the case in progress (`{"exit", "during", "message"}`).
The fixture's `cleanup` first tears down (native cleanup, as now) and then, in
trace and power mode, once `manifest.json` exists, runs `idle-budget.py analyze
"$OUT" > analysis.json` and writes `SHA256SUMS`, whatever the exit status. A
successful run follows the same path, so there is only one.

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
the first 8 cases (about 17 min) instead of as late as case 22 (`O-move-1`). `analyze`
compares matrices as sets, so only `trace_matrix`'s order changes. The power
lane's ABBA order within blocks is part of its measurement design and does not
change.

### 7. The trace must reach its window's end

`trace_observation` adds a gate: the latest CPU zone end (start + duration over
all zones) is at or past the window end. Today the heartbeat coverage check
alone would accept a trace that ended up to 1.5 s early.

### 8. The seat-manager exemption

`inventory` records the seat managers by pid: `{"systemd": 1, "systemd-logind":
<MainPID>}`, read once per inventory with `systemctl show -p MainPID --value
systemd-logind`. `check_inventory` exempts a device holder only if all of these
hold:
- its pid is one of those two;
- its user is root;
- it holds only `/dev/dri/card*` nodes;
- nvidia-smi does not list it;
- our compositor (the first pid in `required`) also holds a card node.

With `required` empty, as in the preflight inventory, nothing is exempt. That
matches the 2026-09-27 evidence: the preflight inventory had no holders at all,
and pid 1 and logind appear only beside our niri on `/dev/dri/card1`.
`power_observation` rechecks each stored inventory with the pids it recorded. A
MainPID that is missing or zero refuses the inventory.

## Interfaces

| Where | Change |
| --- | --- |
| `idle-budget.sh` | `trace` and `power` accept `--pilot` and `--inventory`; `fail` wrapper, `stop.json`, `observe` after each case, analysis in `cleanup` |
| `idle-budget.py` | `check_run`, `observe` and `check-run` subcommands; `manifest` takes `--pilot` / `--inventory`; `pilot_matrix`; repetition-major `trace_matrix`; partial `analyze`; trace-end gate; pinned seat exemption |
| `idle-budget.just` | `trace *flags`, `power *flags` pass the flags through |
| `test_idle_budget.py` | cases below |
| `docs/results/2026-09-11-idle-budget.md` | reproduction section names the pilot and `--inventory` |

No change to `glass-optic-smoke-lib.sh`, `capture-meta`, or niri.

## Testing

Offline, in `test_idle_budget.py`:
- `observe` on synthetic trace and power items writes a verdict and exits 1 on a
  failed gate, an integrity error, and an output-mode change.
- `analyze` on a run with verdicts for some declared cases reports those cases,
  `complete: false`, `not_run`, and `stopped` from `stop.json` and from a refused
  sub-run in `capture.json`. It computes no power comparison for an incomplete or
  pilot run, and reports the pilot's descriptive block.
- The manifest matrix check refuses a pilot subset declared as a full run, and
  the reverse.
- `trace_matrix` is repetition-major and still equals the old matrix as a set.
- The trace-end gate refuses a trace ending 1 s before the window end whose
  heartbeat coverage passes.
- The seat exemption: pinned pids with our niri present pass. They are refused
  with `required` empty, with a non-pinned root `systemd`, with a holder on a
  render node, and with a missing MainPID.
- Shell: source `idle-budget.sh` with the per-case function and the helpers
  stubbed. Check that a failing `observe` stops the loop after that case, that
  `--inventory` continues, and that a `fail` mid-run leaves `stop.json` and an
  `analysis.json` covering the completed cases.

Live: a trace pilot and a power pilot on a TTY with the desktop stopped (see the
settle-gated capture practice). Each must pass, leave per-case verdicts, and
produce an `analysis.json` with `pilot: true`. One deliberately interrupted
trace pilot (TERM after case 2) must leave an analysis of cases 1–2 with
`stopped` set. These need the machine and are parked `quiet` for the person.

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
