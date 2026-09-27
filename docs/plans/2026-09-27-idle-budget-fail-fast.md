# Idle-budget fixture: judge each case as it lands — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the idle-budget fixture judges every case right after its export, stops at
the first failure unless asked for the complete matrix, analyzes whatever completed on
any exit, and offers a pilot per lane.

**Architecture:** the manifest, written before case 1, becomes the only statement of
which cases run and in what order. One shell loop runs the declared cases and calls
`idle-budget.py observe` after each. The fixture's `cleanup` reaps every writer into
`OUT`, then analyzes and checksums, and calls the lib's `native_cleanup` last.

**Tech Stack:** Bash (`set -euo pipefail`, `set -E`), Python 3 stdlib, `unittest`,
Just, and the vendored `tools/tt` of niri-material.

**Spec:** [2026-09-27-idle-budget-fail-fast-design.md](../specs/2026-09-27-idle-budget-fail-fast-design.md)
(reviewed twice, approved for planning 2026-09-27).

## Global Constraints

- The code lands in **niri-experiments** on branch `results/idle-budget`, head
  `647f3c8`, in the worktree `~/d/niri-experiments/.worktrees/material-b15ad7`.
  The spec, this plan and the task records land in **niri-material** on branch
  `material-b15ad7` (`.worktrees/material-b15ad7`).
- Files touched in niri-experiments: `fixtures/idle-budget.sh`,
  `fixtures/idle-budget.py`, `fixtures/idle-budget.just`,
  `fixtures/test_idle_budget.py`, `docs/results/2026-09-11-idle-budget.md`.
- No change to `docs/materials/scripts/glass-optic-smoke-lib.sh`,
  `tools/capture-meta`, or niri (spec, *Interfaces*).
- Test command, from the experiments worktree:
  `MATERIAL_ROOT=<absolute path of niri-material's .worktrees/material-b15ad7> just --justfile fixtures/idle-budget.just test`.
  A single test: `python3 -m unittest -v fixtures/test_idle_budget.py -k <name>`
  (only for red/green inside a step; the recipe run is the step's pass gate).
- Trace pilot: `A-move-1`, `A-resize-1`, `C-move-1`, `D-move-1`. Power pilot:
  `sham-1-1` … `sham-1-4`.
- Full trace order is repetition-major: `A-move`, `A-resize`, `B-move`, `B-resize`,
  `P-move`, `C-move`, `D-move`, `O-move` for repetition 1, then 2, then 3, then
  `B-long-move-1`.
- The analysis runs under `timeout 300`. Weston's host socket is awaited for 5 s.
- `stop.json` is `{"exit", "during", "kind", "message"}`, with `kind` one of `fail`,
  `first-failure`, `signal`, `command`, `exit`. The first writer wins. Nothing is
  written into `OUT` once `SEALED=1`.
- Exit status: a non-zero run status is kept. On zero, a failed or timed-out analysis,
  a teardown error, or a failed checksum makes it 1.
- Seat managers are pinned by pid: `{"systemd": 1, "systemd-logind": <MainPID>}`. They
  are exempt only beside an explicit `compositor` that holds a `/dev/dri/card*` node.
- Commits: conventional, with no AI attribution of any kind. Each code commit in
  niri-experiments pairs with a `tasks done <step>` commit in niri-material that names
  the experiments commit.

## Review Focus

1. **A stop before `OUT/manifest.json` exists** (identity refused, preflight refused).
   Expected: `stop.json` if `OUT` exists, no analysis and no checksum, and the run's
   status. Pinned in Task 6 (`test_stop_before_manifest_writes_no_analysis`).
2. **A case whose verdict file already exists** (an operator reusing `OUT` is refused
   upstream, but a double `observe` is a fixture bug). Expected: `observe` refuses
   rather than overwrite. Pinned in Task 3 (`test_observe_refuses_to_overwrite_a_verdict`).
3. **An `observe` asked for a name the manifest does not declare.** Expected: exit 1
   with the name in the message, and no verdict file. Pinned in Task 3.
4. **A second INT during cleanup** (an operator pressing Ctrl-C twice). Expected: the
   teardown still completes and the status is the first signal's. Pinned in Task 6
   (`test_second_interrupt_during_analysis_still_tears_down`).
5. **A manifest without the `pilot` key** (a run from before this change). Expected:
   `analyze` fails loudly with a `KeyError` about `pilot`; no silent default. Pinned in
   Task 1 (`test_manifest_without_pilot_is_refused`).

---

### Task 1: The declared plan: manifest flags, pilot matrix, repetition-major order, `plan`

**Files:**
- Modify: `fixtures/idle-budget.py` (`trace_matrix`, new `KEYS`, `TRACE_KINDS`,
  `pilot_matrix`, `declared_matrix`, `observation_name`, `write_manifest`, `plan_rows`;
  `analyze`'s matrix check; `main`'s `manifest` and new `plan` subcommands)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Produces:
  - `KEYS: dict[str, tuple[str, str, str]]`: `{"trace": ("case", "stimulus", "repetition"), "power": ("comparison", "block", "position")}`
  - `trace_matrix() -> list[tuple[str, str, int]]`: repetition-major
  - `pilot_matrix(mode: str) -> list[tuple]`
  - `declared_matrix(mode: str, pilot: bool) -> list[tuple]`
  - `observation_name(mode: str, item: dict) -> str`: `"A-move-1"`, `"sham-1-1"`
  - `write_manifest(run: Path, mode: str, pilot: bool, fail_fast: bool) -> None`
  - `plan_rows(run: Path) -> list[tuple[str, ...]]`: trace
    `(name, case, stimulus, repetition)`; power `(name, case)`
  - CLI: `idle-budget.py manifest RUN MODE [--pilot] [--inventory]`;
    `idle-budget.py plan RUN` prints one tab-separated row per line.
  - Manifest JSON: `{"mode", "pilot", "fail_fast", "observations": [...]}`.

- [ ] **Step 0: Create the experiments worktree** (once, before any task)

```bash
cd ~/d/niri-experiments
work-link --ensure .worktrees
git worktree add .worktrees/material-b15ad7 results/idle-budget   # not checked out anywhere else
git worktree lock --reason "on WORK_ROOT storage (host: $(uname -n))" .worktrees/material-b15ad7
cd .worktrees/material-b15ad7 && git log --oneline -1   # expect 647f3c8
```

Commits land directly on `results/idle-budget`. From the niri-material main checkout,
`tasks start` this task's step child.

- [ ] **Step 1: Update the synthetic trace run and write the failing tests**

In `IntegrationTests.trace_run`, build the matrix from the module and write the new
manifest keys. Replace the `matrix = [...]` line and the observation append:

```python
        matrix = idle_budget.trace_matrix()
        for case, stimulus, rep in matrix:
            name = f"{case}-{stimulus}-{rep}"
            observations.append({"case": case, "stimulus": stimulus, "repetition": rep})
```

and the manifest write:

```python
        (root / "manifest.json").write_text(json.dumps({"mode": "trace", "pilot": False, "fail_fast": True, "observations": observations}))
        (root / "capture.json").write_text(json.dumps(self.capture_record(root, "headless", [(idle_budget.observation_name("trace", o), o["case"]) for o in observations])))
```

In `test_power_cli_uses_raw_48_window_matrix`, change the manifest write to
`dict(mode='power', pilot=False, fail_fast=True, observations=items)`.

Add a test class:

```python
class PlanTests(unittest.TestCase):
    OLD_TRACE = [(c, s, r) for c in "ABPCDO" for s in (("move", "resize") if c in "AB" else ("move",))
                 for r in (1, 2, 3)] + [("B", "long-move", 1)]

    def test_trace_matrix_is_repetition_major(self):
        matrix = idle_budget.trace_matrix()
        self.assertEqual(matrix[:8], [("A", "move", 1), ("A", "resize", 1), ("B", "move", 1), ("B", "resize", 1),
                                      ("P", "move", 1), ("C", "move", 1), ("D", "move", 1), ("O", "move", 1)])
        self.assertEqual(matrix[8][2], 2)
        self.assertEqual(matrix[-1], ("B", "long-move", 1))
        self.assertEqual(sorted(matrix), sorted(self.OLD_TRACE))

    def test_pilot_matrices(self):
        self.assertEqual(idle_budget.pilot_matrix("trace"),
                         [("A", "move", 1), ("A", "resize", 1), ("C", "move", 1), ("D", "move", 1)])
        self.assertEqual(idle_budget.pilot_matrix("power"), [("sham", 1, p) for p in (1, 2, 3, 4)])

    def test_manifest_records_plan_and_flags(self):
        cli = IntegrationTests().cli
        for mode, flags, pilot, fail_fast, count in (("trace", [], False, True, 25), ("trace", ["--pilot"], True, True, 4),
                                                     ("power", ["--inventory"], False, False, 48),
                                                     ("power", ["--pilot", "--inventory"], True, False, 4)):
            with self.subTest(mode=mode, flags=flags), tempfile.TemporaryDirectory() as directory:
                result = cli("manifest", directory, mode, *flags)
                self.assertEqual(result.returncode, 0, result.stderr)
                manifest = json.loads((Path(directory) / "manifest.json").read_text())
                self.assertEqual((manifest["mode"], manifest["pilot"], manifest["fail_fast"]), (mode, pilot, fail_fast))
                self.assertEqual(len(manifest["observations"]), count)
                self.assertNotEqual(cli("manifest", directory, mode).returncode, 0)   # never overwritten

    def test_plan_prints_manifest_order(self):
        cli = IntegrationTests().cli
        for mode, flags in (("trace", []), ("trace", ["--pilot"]), ("power", []), ("power", ["--pilot"])):
            with self.subTest(mode=mode, flags=flags), tempfile.TemporaryDirectory() as directory:
                cli("manifest", directory, mode, *flags)
                result = cli("plan", directory)
                self.assertEqual(result.returncode, 0, result.stderr)
                rows = [line.split("\t") for line in result.stdout.splitlines()]
                matrix = idle_budget.declared_matrix(mode, bool(flags))
                if mode == "trace":
                    self.assertEqual(rows, [[f"{c}-{s}-{r}", c, s, str(r)] for c, s, r in matrix])
                else:
                    self.assertEqual(rows, [[f"{c}-{b}-{p}", idle_budget.power_case(c, b, p)] for c, b, p in matrix])

    def test_analyze_refuses_a_matrix_that_disagrees_with_its_pilot_flag(self):
        for pilot, keep in ((True, None), (False, 4)):
            with self.subTest(pilot=pilot), tempfile.TemporaryDirectory() as directory:
                root = Path(directory); IntegrationTests().trace_run(root)
                manifest = json.loads((root / "manifest.json").read_text())
                manifest["pilot"] = pilot
                if keep: manifest["observations"] = [dict(zip(idle_budget.KEYS["trace"], row)) for row in idle_budget.pilot_matrix("trace")]
                (root / "manifest.json").write_text(json.dumps(manifest))
                result = IntegrationTests().cli("analyze", directory)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("declared observation matrix", result.stdout + result.stderr)

    def test_manifest_without_pilot_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().trace_run(root)
            manifest = json.loads((root / "manifest.json").read_text()); del manifest["pilot"]
            (root / "manifest.json").write_text(json.dumps(manifest))
            result = IntegrationTests().cli("analyze", directory)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("pilot", result.stdout + result.stderr)
```

- [ ] **Step 2: Run the new tests to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k PlanTests`
Expected: FAIL/ERROR (`pilot_matrix`, `declared_matrix`, `observation_name` missing;
the `manifest` CLI rejects `--pilot`).

- [ ] **Step 3: Implement**

Replace `trace_matrix` and add the helpers beside `power_matrix`:

```python
KEYS = {"trace": ("case", "stimulus", "repetition"), "power": ("comparison", "block", "position")}
# Every kind once per repetition: a failure specific to one kind shows within the first 8 cases.
TRACE_KINDS = (("A", "move"), ("A", "resize"), ("B", "move"), ("B", "resize"),
               ("P", "move"), ("C", "move"), ("D", "move"), ("O", "move"))


def trace_matrix():
    return [(c, s, r) for r in (1, 2, 3) for c, s in TRACE_KINDS] + [("B", "long-move", 1)]


def pilot_matrix(mode):
    """One case per gate: quiet window and pixels (A-move), the resize journal, and each cadence."""
    if mode == "trace":
        return [("A", "move", 1), ("A", "resize", 1), ("C", "move", 1), ("D", "move", 1)]
    return [("sham", 1, position) for position in (1, 2, 3, 4)]


def declared_matrix(mode, pilot):
    if pilot:
        return pilot_matrix(mode)
    return trace_matrix() if mode == "trace" else power_matrix()


def observation_name(mode, item):
    return "-".join(str(item[key]) for key in KEYS[mode])


def write_manifest(run, mode, pilot, fail_fast):
    observations = [dict(zip(KEYS[mode], row)) for row in declared_matrix(mode, pilot)]
    with (Path(run) / "manifest.json").open("x") as stream:
        json.dump({"mode": mode, "pilot": pilot, "fail_fast": fail_fast, "observations": observations}, stream, indent=2)


def plan_rows(run):
    manifest = read_json(Path(run) / "manifest.json")
    mode = manifest["mode"]
    rows = []
    for item in manifest["observations"]:
        name = observation_name(mode, item)
        if mode == "trace":
            rows.append((name, item["case"], item["stimulus"], str(item["repetition"])))
        else:
            rows.append((name, power_case(item["comparison"], item["block"], item["position"])))
    return rows
```

In `analyze`, replace the two lines computing `keys` and `planned` with:

```python
    keys = KEYS[mode]
    planned = declared_matrix(mode, manifest["pilot"])
```

In `main`, replace the loop that builds `analyze`/`manifest`/`identity` parsers and
the `manifest` branch:

```python
    for name in ("analyze", "identity", "plan"):
        sub = commands.add_parser(name); sub.add_argument("run")
        if name == "identity": sub.add_argument("mode", choices=("trace", "power"))
    sub = commands.add_parser("manifest")
    sub.add_argument("run"); sub.add_argument("mode", choices=("trace", "power"))
    sub.add_argument("--pilot", action="store_true")
    sub.add_argument("--inventory", action="store_true")
```

```python
    elif args.command == "manifest":
        write_manifest(Path(args.run), args.mode, args.pilot, not args.inventory)
    elif args.command == "plan":
        for row in plan_rows(Path(args.run)):
            print("\t".join(row))
```

The shell's existing `python3 "$fixture/idle-budget.py" manifest "$OUT" "$mode"`
call still works unchanged (full plan, fail-fast). Task 5 moves it.

- [ ] **Step 4: Run the suite**

Run: `MATERIAL_ROOT=… just --justfile fixtures/idle-budget.just test`
Expected: all tests pass, including every pre-existing one.

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.py fixtures/test_idle_budget.py
git commit -m "feat(idle-budget): declare the plan in the manifest: pilot matrix, repetition-major trace order, plan rows"
```

Then, in niri-material's `.worktrees/material-b15ad7`: `tasks done <Task 1 child>
"<experiments sha>: …"`, `tasks check`, commit `chore(tasks): close <id>`.

---

### Task 2: Seat managers pinned by pid beside an explicit compositor

**Files:**
- Modify: `fixtures/idle-budget.py` (`SEAT_MANAGERS` removed; new
  `seat_manager_pids`, `check_seat_managers`; `check_inventory`, `inventory`,
  `collect`, `power_observation`, `analyze`'s preflight check, `main`'s `collect`
  and `preflight`)
- Modify: `fixtures/idle-budget.sh` (`power_all`'s `collect` call passes `$NIRI_PID`)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Consumes: Task 1's manifest keys (the synthetic power run writes them).
- Produces:
  - `seat_manager_pids() -> dict[str, int]`: `{"systemd": 1, "systemd-logind": N}`
  - `check_inventory(xml, device_users, device_report, owned: set, required: set, compositor: int | None, seat_managers: dict) -> list[int]`
  - `inventory(owned, required, compositor) -> dict`, which gains `"seat_managers"`
  - `collect(run, name, owned, required, niri, compositor)`; `<name>.interval.json`
    gains `"compositor"`
  - CLI: `idle-budget.py collect RUN NAME NIRI OWNED REQUIRED COMPOSITOR`
  - `IntegrationTests.power_run(root, pilot=False, watts=None)` test helper;
    `watts(name, case) -> float`

- [ ] **Step 1: Extract the synthetic power run and write the failing tests**

Move the body of `test_power_cli_uses_raw_48_window_matrix` that builds the run into
`IntegrationTests.power_run`, with the new fields:

```python
    SEAT = {"systemd": 1, "systemd-logind": 977}

    def power_run(self, root, pilot=False, watts=None):
        """A synthetic power run over the declared plan; `watts(name, case)` sets each window."""
        self.trace_run(root)
        identity = json.loads((root / 'identity.json').read_text()); identity.update(features=[], patch_sha256=None)
        (root / 'identity.json').write_text(json.dumps(identity))
        integrity = PowerIntegrityTests()
        output = json.loads((root / 'A-move-1.before.outputs.json').read_text())
        (root / 'preflight.json').write_text(json.dumps(dict(xml=integrity.xml([]), device_users='', device_report='',
            devices=['/dev/nvidia0', '/dev/dri/renderD128'], seat_managers=self.SEAT)))
        watts = watts or (lambda name, case: 22 if case == "C" else 21 if case == "D" else 20)
        items = []
        for comparison, block, position in idle_budget.declared_matrix("power", pilot):
            name = f'{comparison}-{block}-{position}'
            case = idle_budget.power_case(comparison, block, position)
            (root / f'{name}.identity.json').write_text((root / 'identity.json').read_text())
            (root / f'{name}.renderer.log').write_text('GL Renderer: NVIDIA test')
            (root / f'{name}.pids').write_text('1\n4\n2\n3\n')
            for when in ('before', 'after'):
                (root / f'{name}.{when}.windows.json').write_text((root / 'A-move-1.before.windows.json').read_text())
                (root / f'{name}.{when}.outputs.json').write_text((root / 'A-move-1.before.outputs.json').read_text())
            items.append(dict(comparison=comparison, block=block, position=position))
            (root / f'{name}.interval.json').write_text(json.dumps(dict(start=160, end=190, owned=[1, 2, 3, 4], required=[1, 2, 3], compositor=1)))
            value = watts(name, case)
            users, report = integrity.fuser(integrity.owned_rows([1, 2, 3]))
            row = dict(power_w=value, telemetry=[str(value), "P8", "210", "405", "0", "40"], missed_deadlines=0, outputs=output,
                       inventory=dict(xml=integrity.xml([1, 2, 3]), device_users=users, device_report=report,
                                      devices=['/dev/nvidia0', '/dev/dri/renderD128'], seat_managers=self.SEAT))
            rows = [{**row, 'deadline': 100 + i, 'request_time': 100 + i, 'return_time': 100.1 + i} for i in range(90)]
            (root / f'{name}.samples.jsonl').write_text(''.join(json.dumps(r) + '\n' for r in rows))
        (root / 'manifest.json').write_text(json.dumps(dict(mode='power', pilot=pilot, fail_fast=True, observations=items)))
        cases = [(idle_budget.observation_name("power", i), idle_budget.power_case(i['comparison'], i['block'], i['position'])) for i in items]
        (root / 'capture.json').write_text(json.dumps(self.capture_record(root, 'dedicated', cases)))
        return items
```

`test_power_cli_uses_raw_48_window_matrix` keeps its assertions and corruption loop,
opening with `IntegrationTests().power_run(root)`. Add one corruption case to the loop's
tuple, `'compositor'`, handled before the samples are touched:

```python
                if corruption == 'compositor':
                    interval = root / 'A-B-1-1.interval.json'; saved = interval.read_text()
                    interval.write_text(json.dumps({**json.loads(saved), 'compositor': 4}))
                    with self.subTest(corruption=corruption):
                        self.assertNotEqual(IntegrationTests().cli('analyze', directory).returncode, 0)
                    interval.write_text(saved); continue
```

In `PowerIntegrityTests`, pass the new arguments everywhere:
- `test_requires_owned_clients_and_rejects_contamination`: calls become
  `idle_budget.check_inventory(xml, users, report, {1,2,3,4}, {1,2,3}, 1, IntegrationTests.SEAT)`,
  and the empty-XML call `check_inventory('<nvidia_smi_log><gpu/></nvidia_smi_log>', '', '', set(), set(), None, IntegrationTests.SEAT)`.
- `test_parses_a_captured_fuser_report`: `…, {3561296}, {3561296}, 3561296, IntegrationTests.SEAT)`.

Replace `test_seat_managers_holding_only_the_kms_node_are_not_gpu_clients` with:

```python
    def test_seat_managers_are_pinned_by_pid_beside_our_compositor(self):
        # A DRM session makes logind open the primary node for the compositor, and
        # systemd keeps the fd: both hold /dev/dri/card1 without rendering.
        seat = [("/dev/dri/card1", "root", 1, "systemd"), ("/dev/dri/card1", "root", 977, "systemd-logind")]
        owned = [("/dev/dri/card1", "keith", 10, "niri")] + self.owned_rows([10, 12])
        pinned = IntegrationTests.SEAT
        def check(rows, xml_pids=(10, 12), compositor=10, seat_managers=pinned):
            return idle_budget.check_inventory(self.xml(list(xml_pids)), *self.fuser(rows), {10, 11, 12}, {10, 12},
                                               compositor, seat_managers)
        self.assertEqual(check(seat + owned), [10, 12])
        rejected = {
            "logind also on a render node": dict(rows=seat + [("/dev/dri/renderD128", "root", 977, "systemd-logind")] + owned),
            "logind also on an nvidia node": dict(rows=seat + [("/dev/nvidia0", "root", 977, "systemd-logind")] + owned),
            "a user-owned systemd": dict(rows=seat + [("/dev/dri/card1", "keith", 978, "systemd")] + owned),
            "a root systemd that is not pid 1": dict(rows=seat + [("/dev/dri/card1", "root", 980, "systemd")] + owned),
            "another root process": dict(rows=seat + [("/dev/dri/card1", "root", 979, "Xorg")] + owned),
            "no compositor (preflight)": dict(rows=seat + owned, compositor=None),
            "compositor not in required": dict(rows=seat + owned, compositor=11),
            "a kitty on the card node, the compositor off it": dict(
                rows=seat + [("/dev/dri/card1", "keith", 12, "kitty")] + self.owned_rows([10, 12])),
            "a GPU process named like a seat manager": dict(rows=seat + owned, xml_pids=(977, 10, 12)),
            "logind MainPID missing": dict(rows=seat + owned, seat_managers={"systemd": 1}),
            "logind MainPID zero": dict(rows=seat + owned, seat_managers={"systemd": 1, "systemd-logind": 0}),
        }
        for label, arguments in rejected.items():
            with self.subTest(label), self.assertRaises(ValueError):
                check(**arguments)
        users, report = self.fuser(seat + owned)
        with self.assertRaises(ValueError):   # every table row needs its pid
            idle_budget.check_inventory(self.xml([10, 12]), users.rsplit(None, 1)[0], report, {10, 11, 12}, {10, 12}, 10, pinned)

    def test_seat_manager_pids_reads_logind_main_pid(self):
        from unittest.mock import patch
        with patch.object(idle_budget, "query", return_value="977\n") as query:
            self.assertEqual(idle_budget.seat_manager_pids(), {"systemd": 1, "systemd-logind": 977})
        self.assertEqual(query.call_args[0][0], ["systemctl", "show", "-p", "MainPID", "--value", "systemd-logind"])
        for raw in ("0\n", "\n", "abc\n"):
            with self.subTest(raw=raw), patch.object(idle_budget, "query", return_value=raw), self.assertRaises(ValueError):
                idle_budget.seat_manager_pids()
```

- [ ] **Step 2: Run to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k PowerIntegrityTests`
Expected: FAIL/ERROR (`check_inventory` takes 5 positional arguments; no
`seat_manager_pids`).

- [ ] **Step 3: Implement**

Replace `SEAT_MANAGERS` and the seat block of `check_inventory`:

```python
# A DRM session makes systemd-logind open the KMS primary node for the compositor, and
# systemd (pid 1) keeps the fd. They are pinned by pid, never by command name.
def seat_manager_pids():
    raw = query(["systemctl", "show", "-p", "MainPID", "--value", "systemd-logind"]).strip()
    if not raw.isdigit() or int(raw) <= 1:
        raise ValueError(f"systemd-logind MainPID unavailable: {raw!r}")
    return {"systemd": 1, "systemd-logind": int(raw)}


def check_seat_managers(seat_managers):
    if set(seat_managers) != {"systemd", "systemd-logind"} or seat_managers["systemd"] != 1 \
            or not isinstance(seat_managers["systemd-logind"], int) or seat_managers["systemd-logind"] <= 1:
        raise ValueError(f"seat managers must be pid 1 and systemd-logind's MainPID: {seat_managers!r}")


def check_inventory(xml, device_users, device_report, owned, required, compositor, seat_managers):
    check_seat_managers(seat_managers)
    tree = ET.fromstring(xml)
    # … unchanged through `holders = device_holders(device_users, device_report)` …
    seat = set()
    # Exempt only while our compositor holds a card node: that is when logind opened it for us.
    if compositor is not None and compositor in required and any(
            pid == compositor and device.startswith("/dev/dri/card") for device, _, pid, _ in holders):
        pinned = set(seat_managers.values())
        seat = {pid for _, user, pid, _ in holders if pid in pinned and user == "root"}
        seat -= {pid for device, _, pid, _ in holders if pid in seat and not device.startswith("/dev/dri/card")}
    devices -= seat - pids
    # … unchanged from `if not required <= pids …` …
```

`inventory` takes and passes the compositor, and records the pids:

```python
def inventory(owned, required, compositor):
    # … unchanged through the fuser call …
    seat_managers = seat_manager_pids()
    clients = check_inventory(xml, result.stdout, result.stderr, set(owned), set(required), compositor, seat_managers)
    return {"xml": xml, "device_users": result.stdout, "device_report": result.stderr,
            "devices": devices, "seat_managers": seat_managers, "clients": clients}
```

`collect(run, name, owned, required, niri, compositor)` calls
`inventory(owned, required, compositor)`, and its `interval.json` write adds
`"compositor": compositor`. `power_observation`, after reading `interval`:

```python
    if interval["compositor"] != pids[0]:
        raise ValueError("recorded compositor differs from the scene's niri")
```

and its inventory recheck becomes
`check_inventory(inv["xml"], inv["device_users"], inv["device_report"], set(interval["owned"]), set(interval["required"]), interval["compositor"], inv["seat_managers"])`.
`analyze`'s power preflight check becomes
`check_inventory(preflight["xml"], preflight["device_users"], preflight["device_report"], set(), set(), None, preflight["seat_managers"])`.

In `main`: the `collect` parser gets a sixth positional `compositor`; the branches become
`inventory([], [], None)` for `preflight`, and for `collect`:
`collect(Path(args.run), args.name, list(map(int, args.owned.split(","))), list(map(int, args.required.split(","))), args.niri, int(args.compositor))`.

In `idle-budget.sh`'s `power_all`, the collector line becomes:

```bash
                setsid python3 "$fixture/idle-budget.py" collect "$OUT" "$name" "$NIRI" "$owned" "$required" "$NIRI_PID" & SAMPLE_PID=$!
```

- [ ] **Step 4: Run the suite** — Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.py fixtures/idle-budget.sh fixtures/test_idle_budget.py
git commit -m "fix(idle-budget): pin the seat-manager exemption to pid 1 and logind's MainPID beside our compositor"
```

Then close the Task 2 child in niri-material as in Task 1.

---

### Task 3: A verdict per case: `check_run`, `observe`, and the trace-end gate

**Files:**
- Modify: `fixtures/idle-budget.py` (`check_run` extracted from `analyze`; new
  `judge`, `verdict_path`, `output_baseline`, `verdict_line`, `observe`; trace-end
  gate in `trace_observation`; `check-run` and `observe` subcommands)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Consumes: `KEYS`, `declared_matrix`, `observation_name` (Task 1); the new
  `check_inventory` signature (Task 2).
- Produces:
  - `check_run(run: Path) -> tuple[dict, dict, dict]`: `(manifest, provenance, hardware)`;
    raises `ValueError`/`KeyError`/`OSError`/`ET.ParseError`
  - `judge(run: Path, item: dict, mode: str) -> tuple[dict | None, str | None]`: `(summary, error)`
  - `verdict_path(run: Path, name: str) -> Path`: `run / f"{name}.verdict.json"`
  - `observe(run: Path, name: str) -> bool`: writes the verdict and returns passed
  - Verdict JSON: `{"observation": item, "passed": bool, "summary"?: dict, "error"?: str}`
  - CLI: `idle-budget.py check-run RUN` (exit 0/1); `idle-budget.py observe RUN NAME` (exit 0 pass, 1 otherwise)

- [ ] **Step 1: Extend the synthetic run so its traces reach the window end, and write the failing tests**

In `trace_run`, the heartbeat rows become `range(length + 2)` (the window ends at
28.001 s for a 20 s case; the last heartbeat must be past it):

```python
            rows += [f"Niri::refresh_idle_inhibit,{int((8.1+i)*1e9)},1000" for i in range(length + 2)]
```

Add a test class:

```python
class ObserveTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(); self.root = Path(self.directory.name)
        IntegrationTests().trace_run(self.root)

    def tearDown(self):
        self.directory.cleanup()

    def observe(self, name):
        return IntegrationTests().cli("observe", str(self.root), name)

    def verdict(self, name):
        return json.loads((self.root / f"{name}.verdict.json").read_text())

    def set_output(self, name, width):
        for when in ("before", "after"):
            path = self.root / f"{name}.{when}.outputs.json"
            outputs = json.loads(path.read_text()); outputs["test"]["modes"][0]["width"] = width
            path.write_text(json.dumps(outputs))

    def add_redraw(self, name):
        path = self.root / f"{name}.csv"
        path.write_text(path.read_text() + "Niri::redraw,15000000000,1000\n")

    def test_a_passing_case_writes_its_verdict_and_one_line(self):
        result = self.observe("A-move-1")
        self.assertEqual(result.returncode, 0, result.stderr)
        verdict = self.verdict("A-move-1")
        self.assertTrue(verdict["passed"]); self.assertEqual(verdict["summary"]["redraws"], 0)
        self.assertEqual(verdict["observation"], {"case": "A", "stimulus": "move", "repetition": 1})
        self.assertEqual(result.stderr.strip(), "A-move-1 pass: 0 redraws, 0 material draws, pixels equal")

    def test_a_failed_gate_keeps_its_summary(self):
        self.add_redraw("A-move-1")
        result = self.observe("A-move-1")
        self.assertEqual(result.returncode, 1)
        verdict = self.verdict("A-move-1")
        self.assertFalse(verdict["passed"]); self.assertEqual(verdict["summary"]["redraws"], 1)
        self.assertEqual(verdict["error"], "behavioral gate failed")
        self.assertTrue(result.stderr.startswith("A-move-1 FAIL: 1 redraws"))

    def test_an_integrity_error_has_no_summary(self):
        (self.root / "A-move-1.gpu.csv").unlink()
        self.assertEqual(self.observe("A-move-1").returncode, 1)
        verdict = self.verdict("A-move-1")
        self.assertNotIn("summary", verdict); self.assertIn("gpu.csv", verdict["error"])

    def test_the_output_baseline_is_the_first_reconstructed_case(self):
        (self.root / "A-move-1.gpu.csv").unlink()           # case 1: error only, no output
        self.set_output("A-resize-1", 1920)                  # case 2: reconstructed, becomes the baseline
        self.assertEqual(self.observe("A-move-1").returncode, 1)
        self.assertEqual(self.observe("A-resize-1").returncode, 0)
        self.set_output("B-move-1", 1920)
        self.assertEqual(self.observe("B-move-1").returncode, 0)   # same output as the baseline
        self.assertEqual(self.observe("B-resize-1").returncode, 1)  # 1280 differs from 1920
        self.assertIn("output mode differs", self.verdict("B-resize-1")["error"])

    def test_a_failed_gate_summary_is_a_baseline_too(self):
        self.add_redraw("A-move-1")
        self.observe("A-move-1")
        self.set_output("A-resize-1", 1920)
        self.assertEqual(self.observe("A-resize-1").returncode, 1)
        self.assertIn("output mode differs", self.verdict("A-resize-1")["error"])

    def test_a_trace_ending_before_the_window_fails_although_heartbeats_cover_it(self):
        path = self.root / "A-move-1.csv"
        rows = [r for r in path.read_text().splitlines()
                if not (r.startswith("Niri::refresh_idle_inhibit") and float(r.split(",")[1]) / 1e9 > 27.2)]
        path.write_text("\n".join(rows) + "\n")   # last heartbeat 27.1 s; the window ends at 28.001 s
        self.assertEqual(self.observe("A-move-1").returncode, 1)
        self.assertIn("trace ends before", self.verdict("A-move-1")["error"])

    def test_observe_refuses_an_undeclared_name(self):
        result = self.observe("Z-move-1")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Z-move-1", result.stderr)
        self.assertFalse((self.root / "Z-move-1.verdict.json").exists())

    def test_observe_refuses_to_overwrite_a_verdict(self):
        self.assertEqual(self.observe("A-move-1").returncode, 0)
        self.assertEqual(self.observe("A-move-1").returncode, 1)

    def test_check_run_gates_run_level_evidence(self):
        self.assertEqual(IntegrationTests().cli("check-run", str(self.root)).returncode, 0)
        (self.root / "source.tar").write_bytes(b"changed")
        result = IntegrationTests().cli("check-run", str(self.root))
        self.assertEqual(result.returncode, 1)
        self.assertIn("source_sha256", result.stderr)
```

- [ ] **Step 2: Run to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k ObserveTests`
Expected: FAIL/ERROR (no `observe` or `check-run` subcommand).

- [ ] **Step 3: Implement**

In `trace_observation`, directly after `end = …`:

```python
    if max(t + d for zone in cpu.values() for t, d in zone) < end:
        raise ValueError("trace ends before the observation window does")
```

Split `analyze`: everything from reading the manifest through the matrix check moves
into `check_run`, and `analyze` starts by calling it:

```python
def check_run(run):
    """Run-level evidence, needing only what exists before case 1: identity, lane,
    preflight, hardware, capture-time binary, and the declared matrix."""
    run = Path(run)
    manifest = read_json(run / "manifest.json")
    mode = manifest["mode"]
    if mode not in KEYS:
        raise ValueError("unknown lane")
    provenance = identity(run, mode)
    record = capture_record(run)
    # … the existing lane, preflight, power tty/preflight-inventory, hardware and
    # binary-hash checks, unchanged …
    actual = [tuple(item[k] for k in KEYS[mode]) for item in manifest["observations"]]
    if sorted(actual) != sorted(declared_matrix(mode, manifest["pilot"])):
        raise ValueError("declared observation matrix missing or duplicated")
    return manifest, provenance, hardware


def verdict_path(run, name):
    return Path(run) / f"{name}.verdict.json"


def judge(run, item, mode):
    try:
        summary = (trace_observation if mode == "trace" else power_observation)(run, item)
    except (ValueError, KeyError, OSError, ET.ParseError) as error:
        return None, str(error)
    return summary, None if summary.get("passed", True) else "behavioral gate failed"


def output_baseline(run, manifest):
    """The output of the first case, in manifest order, whose observation was reconstructed."""
    for item in manifest["observations"]:
        path = verdict_path(run, observation_name(manifest["mode"], item))
        if path.exists():
            verdict = read_json(path)
            if "summary" in verdict:
                return verdict["summary"]["output"]
    return None


def verdict_line(name, verdict):
    summary = verdict.get("summary")
    if summary is None:
        detail = verdict["error"]
    elif "redraws" in summary:
        detail = (f"{summary['redraws']} redraws, {summary['material_draws']} material draws, "
                  f"pixels {'equal' if summary['pixels_equal'] else 'differ'}")
    else:
        detail = f"median {summary['median_w']:.3f} W"
    if summary is not None and not verdict["passed"]:
        detail += f" ({verdict['error']})"
    return f"{name} {'pass' if verdict['passed'] else 'FAIL'}: {detail}"


def observe(run, name):
    run = Path(run)
    manifest = read_json(run / "manifest.json")
    mode = manifest["mode"]
    items = [item for item in manifest["observations"] if observation_name(mode, item) == name]
    if len(items) != 1:
        raise ValueError(f"{name} is not a declared observation")
    baseline = output_baseline(run, manifest)
    summary, error = judge(run, items[0], mode)
    errors = [error] if error else []
    if summary is not None and baseline is not None and summary["output"] != baseline:
        errors.append("output mode differs from the run's first reconstructed case")
    verdict = {"observation": items[0], "passed": not errors}
    if summary is not None:
        verdict["summary"] = summary
    if errors:
        verdict["error"] = "; ".join(errors)
    with verdict_path(run, name).open("x") as stream:
        json.dump(verdict, stream, indent=2)
    print(verdict_line(name, verdict), file=sys.stderr)
    return verdict["passed"]
```

`analyze` begins `manifest, provenance, hardware = check_run(run)` and keeps
`mode = manifest["mode"]`; its per-item loop calls `judge` (Task 4 rewrites the rest).
With a pre-existing verdict file, `open("x")` raises `FileExistsError`, an `OSError`
that `main`'s handler turns into exit 1.

In `main`: add parsers `check-run` (`run`) and `observe` (`run`, `name`), and branches:

```python
    if args.command == "observe":
        return not observe(Path(args.run), args.name)
    if args.command == "check-run":
        check_run(Path(args.run))
```

(`main`'s return value goes to `sys.exit`; `True` exits 1.)

- [ ] **Step 4: Run the suite** — Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.py fixtures/test_idle_budget.py
git commit -m "feat(idle-budget): judge one case at a time and gate traces that end before their window"
```

Then close the Task 3 child in niri-material.

---

### Task 4: Analyze what completed

**Files:**
- Modify: `fixtures/idle-budget.py` (`analyze`, new `stop_record`, `optional_json`;
  `main`'s exit rule)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Consumes: `check_run`, `judge`, `verdict_path`, `observation_name` (Task 3);
  `IntegrationTests.power_run` (Task 2).
- Produces the analysis JSON fields: `mode`, `pilot`, `identity`, `hardware`,
  `observations`, `integrity_failures`, `not_run: list[str]`, `complete: bool`,
  `stopped: dict | None` (with `refusal: {"sub_run", "reason"}` when a sub-run was
  refused), `teardown: list[str]`, and for power either `power`/`budget_passed`
  (complete full run) or `pilot_block` (complete pilot).
  `analyze` exits 1 when `integrity_failures` is non-empty, when `complete` is false,
  or when `budget_passed` is false.

- [ ] **Step 1: Mark synthetic cases complete and write the failing tests**

At the end of each case in `trace_run`, and of each window in `power_run`, write its
verdict (only its presence matters to `analyze`):

```python
            (root / f"{name}.verdict.json").write_text(json.dumps({"observation": observations[-1], "passed": True}))
```

(in `power_run`, with `items[-1]`). Add:

```python
class PartialAnalysisTests(unittest.TestCase):
    def analyze(self, root):
        result = IntegrationTests().cli("analyze", str(root))
        return result.returncode, json.loads(result.stdout)

    def test_complete_trace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().trace_run(root)
            code, result = self.analyze(root)
            self.assertEqual(code, 0, result["integrity_failures"])
            self.assertEqual((result["complete"], result["not_run"], result["stopped"], result["pilot"]), (True, [], None, False))

    def test_a_stopped_run_reports_its_completed_cases_and_why(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().trace_run(root)
            names = [f"{c}-{s}-{r}" for c, s, r in idle_budget.trace_matrix()]
            for name in names[3:]:
                (root / f"{name}.verdict.json").unlink()
            (root / "stop.json").write_text(json.dumps({"exit": 1, "during": names[3], "kind": "fail", "message": f"settle refused before {names[3]}"}))
            record = json.loads((root / "capture.json").read_text())
            entry = next(e for e in record["sub_runs"] if e["name"] == names[3]); entry.update(verdict="refused", reason="gpu busy")
            (root / "capture.json").write_text(json.dumps(record))
            (root / "teardown.json").write_text(json.dumps(["Weston socket h still present"]))
            code, result = self.analyze(root)
            self.assertEqual(code, 1)
            self.assertEqual([idle_budget.observation_name("trace", o) for o in result["observations"]], names[:3])
            self.assertEqual(result["not_run"], names[3:])
            self.assertFalse(result["complete"])
            self.assertEqual(result["stopped"]["during"], names[3])
            self.assertEqual(result["stopped"]["refusal"], {"sub_run": names[3], "reason": "gpu busy"})
            self.assertEqual(result["teardown"], ["Weston socket h still present"])
            self.assertEqual(result["integrity_failures"], [])

    def test_a_run_level_failure_is_reported_not_raised(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().trace_run(root)
            (root / "source.tar").write_bytes(b"changed")
            code, result = self.analyze(root)
            self.assertEqual(code, 1)
            self.assertIn("run-level check", result["integrity_failures"][0]["error"])
            self.assertEqual(len(result["not_run"]), 25)
            self.assertEqual(result["observations"], [])

    def test_power_pilot_reports_a_descriptive_block(self):
        medians = {"sham-1-1": 20.0, "sham-1-2": 20.4, "sham-1-3": 20.2, "sham-1-4": 20.1}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().power_run(root, pilot=True, watts=lambda name, case: medians[name])
            code, result = self.analyze(root)
            self.assertEqual(code, 0, result["integrity_failures"])
            self.assertNotIn("power", result); self.assertNotIn("budget_passed", result)
            block = result["pilot_block"]
            self.assertEqual(block["medians_w"], [20.0, 20.4, 20.2, 20.1])
            self.assertAlmostEqual(block["repeat_spreads_w"][0], 0.1); self.assertAlmostEqual(block["repeat_spreads_w"][1], 0.2)
            self.assertAlmostEqual(block["block_delta_w"], 0.25)
            self.assertEqual((block["target_w"], block["descriptive"]), (1.0, True))

    def test_an_incomplete_power_run_computes_no_comparison(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); IntegrationTests().power_run(root)
            (root / "B-D-3-4.verdict.json").unlink()
            code, result = self.analyze(root)
            self.assertEqual(code, 1)
            self.assertNotIn("power", result); self.assertNotIn("budget_passed", result)
            self.assertEqual(result["not_run"], ["B-D-3-4"])
```

- [ ] **Step 2: Run to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k PartialAnalysisTests`
Expected: FAIL (`KeyError: 'complete'`, and the run-level case exits with no JSON).

- [ ] **Step 3: Implement**

```python
def optional_json(path, default):
    return read_json(path) if path.exists() else default


def stop_record(run):
    """stop.json, plus the refused sub-run from capture.json when the stop was a refusal."""
    stop = optional_json(run / "stop.json", None)
    if stop is None or not (run / "capture.json").exists():
        return stop
    refused = [entry for entry in read_json(run / "capture.json").get("sub_runs", []) if entry.get("verdict") != "settled"]
    if refused:
        stop["refusal"] = {"sub_run": refused[-1]["name"], "reason": refused[-1].get("reason")}
    return stop


def analyze(run):
    run = Path(run)
    manifest = read_json(run / "manifest.json")
    mode, pilot = manifest["mode"], manifest["pilot"]
    names = [observation_name(mode, item) for item in manifest["observations"]]
    result = {"mode": mode, "pilot": pilot, "identity": None, "hardware": None,
              "observations": [], "integrity_failures": [], "not_run": [],
              "stopped": stop_record(run), "teardown": optional_json(run / "teardown.json", [])}
    try:
        _, result["identity"], result["hardware"] = check_run(run)
    except (ValueError, KeyError, OSError, ET.ParseError) as error:
        result["integrity_failures"].append({"error": f"run-level check: {error}"})
        result.update(not_run=names, complete=False)
        return result
    summaries = []
    for item, name in zip(manifest["observations"], names):
        if not verdict_path(run, name).exists():   # a verdict marks a case complete
            result["not_run"].append(name)
            continue
        summary, error = judge(run, item, mode)
        if summary is not None:
            summaries.append(summary)
        if error is not None:
            result["integrity_failures"].append({"observation": item, "error": error})
    if summaries and any(s["output"] != summaries[0]["output"] for s in summaries):
        result["integrity_failures"].append({"error": "output mode changed across run"})
    result["observations"] = summaries
    result["complete"] = not result["not_run"]
    if mode == "power" and result["complete"] and not result["integrity_failures"]:
        if pilot:
            medians = [s["median_w"] for s in sorted(summaries, key=lambda s: s["position"])]
            result["pilot_block"] = {"medians_w": medians,
                                     "repeat_spreads_w": [abs(medians[0] - medians[3]), abs(medians[1] - medians[2])],
                                     "block_delta_w": block_delta(medians), "target_w": 1.0, "descriptive": True}
        else:
            # … the existing blocks/`power_comparison`/`budget_passed` code, unchanged …
    return result
```

`main`'s analyze branch:

```python
        return bool(result["integrity_failures"]) or not result["complete"] or not result.get("budget_passed", True)
```

- [ ] **Step 4: Run the suite** — Expected: all pass. The existing power CLI test
now needs its windows' verdicts, which `power_run` writes.

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.py fixtures/test_idle_budget.py
git commit -m "feat(idle-budget): analyze the completed cases of a stopped run and report why it stopped"
```

Then close the Task 4 child in niri-material.

---

### Task 5: One loop over the declared plan, with fail-fast and `--inventory`

**Files:**
- Modify: `fixtures/idle-budget.sh` (flag parsing, `write_manifest`, `check_run_gate`,
  `observe_case`, `analyze_run`, `seal_checksums`, `run_cases`, `trace_one` and new
  `power_one` signatures, `trace_all`/`power_all`, the `runtime` tail, the dispatcher)
- Modify: `fixtures/idle-budget.just` (`trace` and `power` take flags)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Consumes: CLIs `manifest … [--pilot] [--inventory]`, `plan`, `check-run`,
  `observe`, `analyze` (Tasks 1, 3, 4).
- Produces (top-level shell functions, which Task 6 and the tests override):
  - `write_manifest`: uses globals `OUT`, `mode`, `PILOT`, `FAIL_FAST`
  - `check_run_gate`, `observe_case NAME`, `analyze_run` (stdout: the analysis JSON),
    `seal_checksums`
  - `run_cases FUNCTION`: calls `FUNCTION name case stimulus repetition` (trace) or
    `FUNCTION name case` (power) per plan row; keeps `CURRENT_CASE`; sets
    `STOP_KIND=first-failure` before `fail` on a failed case when fail-fast
  - `trace_one NAME CASE STIMULUS REPETITION`, `power_one NAME CASE`
  - `bash idle-budget.sh trace|power [--pilot] [--inventory]`; unknown flag, or any flag
    on `prepare`, exits 2
  - just: `trace *flags`, `power *flags`
  - Test helper `ShellTests.fixture_env(root, mode) -> dict` (stub tools, identity,
    env), which Task 6 reuses

- [ ] **Step 1: Write the failing tests**

Add the shared helper to `ShellTests`, factored from
`test_runtime_validates_identity_before_reaching_launch_boundary`'s setup:

```python
    @staticmethod
    def fixture_env(root, mode, capture_meta="true"):
        import os
        bins = root / "bin"; bins.mkdir()
        scripts = {
            "niri": 'case "$1" in --version) echo "niri '+"a"*40+'" ;; validate) : ;; *) exit 99 ;; esac',
            "magick": 'if [ "$1" = --version ]; then echo synthetic; else for last; do :; done; : > "$last"; fi',
            "kitty": 'test "$1" = --version', "swaybg": 'test "$1" = --version',
        }
        for tool, body in scripts.items():
            path = bins / tool; path.write_text("#!/bin/sh\n" + body + "\n"); path.chmod(0o755)
        binary = bins / "niri"
        (bins / "niri.source.tar").write_bytes(b"src"); (bins / "niri.marker.patch").write_bytes(b"patch")
        (bins / "niri.identity.json").write_text(json.dumps(dict(
            binary_sha256=idle_budget.digest(binary), source_sha256=idle_budget.digest(bins / "niri.source.tar"),
            source_commit="a"*40, features=["profile-with-tracy"] if mode == "trace" else [],
            patch_sha256=idle_budget.digest(bins / "niri.marker.patch") if mode == "trace" else None)))
        runtime = root / "runtime"; runtime.mkdir()
        env = {**os.environ, "PATH": str(bins) + ":" + os.environ["PATH"], "CAPTURE_META": capture_meta,
               "FIXTURE": str(Path(__file__).with_name("idle-budget.sh")), "OUT": str(root / "material-265eb0/run"),
               "NIRI_BIN": str(binary), "NIRI_MATERIAL_WORK_ROOT": str(root), "XDG_RUNTIME_DIR": str(runtime),
               "XDG_SESSION_TYPE": "tty", "POWER_OUTPUT": "DP-1", "POWER_MODE": "1920x1080@60", "POWER_SCALE": "1"}
        for key in ("DISPLAY", "WAYLAND_DISPLAY"): env.pop(key, None)
        return env

    # Stubs for a run without a compositor: the case function logs its arguments, observe
    # writes a verdict and fails the case named in FAIL_AT.
    LOOP_STUBS = r"""
source "$FIXTURE"
check_run_gate() { :; }
observe_case() { echo '{}' > "$OUT/$1.verdict.json"; [ "$1" != "${FAIL_AT:-}" ]; }
analyze_run() { echo '{}'; }
fake_one() { echo "$*" >> "$OUT/calls"; }
trace_all() { run_cases fake_one; }
power_all() { run_cases fake_one; }
"""

    def run_loop(self, mode, flags=(), extra="", env_extra=None, timeout=20):
        import subprocess
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        env = {**self.fixture_env(root, mode), **(env_extra or {})}
        script = self.LOOP_STUBS + extra + f'\nruntime {mode} {" ".join(flags)}\n'
        result = subprocess.run(["bash", "-c", script], env=env, capture_output=True, text=True, timeout=timeout)
        return result, Path(env["OUT"])
```

Then the tests:

```python
    def test_cases_run_in_manifest_order_for_every_plan(self):
        for mode, flags in (("trace", ()), ("trace", ("--pilot",)), ("power", ()), ("power", ("--pilot",))):
            with self.subTest(mode=mode, flags=flags):
                result, out = self.run_loop(mode, flags)
                self.assertEqual(result.returncode, 0, result.stderr)
                matrix = idle_budget.declared_matrix(mode, bool(flags))
                if mode == "trace":
                    expected = [f"{c}-{s}-{r} {c} {s} {r}" for c, s, r in matrix]
                else:
                    expected = [f"{c}-{b}-{p} {idle_budget.power_case(c, b, p)}" for c, b, p in matrix]
                self.assertEqual((out / "calls").read_text().splitlines(), expected)

    def test_a_failed_case_stops_the_run_unless_inventory(self):
        result, out = self.run_loop("trace", env_extra={"FAIL_AT": "A-resize-1"})
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len((out / "calls").read_text().splitlines()), 2)
        self.assertIn("first failure: A-resize-1", result.stderr)
        result, out = self.run_loop("trace", ("--inventory",), env_extra={"FAIL_AT": "A-resize-1"})
        self.assertEqual(len((out / "calls").read_text().splitlines()), 25)
        self.assertFalse(json.loads((out / "manifest.json").read_text())["fail_fast"])

    def test_unknown_flags_and_flags_on_prepare_are_refused(self):
        for mode, flags in (("trace", ("--pilto",)), ("prepare", ("--pilot",))):
            with self.subTest(mode=mode, flags=flags):
                result, _ = self.run_loop(mode, flags)
                self.assertEqual(result.returncode, 2)
```

Update two existing tests to the new signatures:
- `test_trace_orchestration_journals_helpers_and_waits_before_after_snapshot`: the
  script's last line becomes `trace_one A-move-1 A move 1`.
- `test_runtime_calls_capture_meta_in_order_per_mode` and
  `test_runtime_validates_identity_before_reaching_launch_boundary`: after
  `source "$FIXTURE"`, add `analyze_run() { echo '{}'; }` (their `trace_all` stubs exit
  after the manifest is written; from Task 6 on, cleanup analyzes it).

- [ ] **Step 2: Run to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k ShellTests`
Expected: FAIL (`run_cases: command not found`; flags are not parsed).

- [ ] **Step 3: Implement**

Top-level helpers, after `stop_clients`:

```bash
write_manifest() {
    local flags=()
    [ "$PILOT" = 0 ] || flags+=(--pilot)
    [ "$FAIL_FAST" = 1 ] || flags+=(--inventory)
    python3 "$fixture/idle-budget.py" manifest "$OUT" "$mode" "${flags[@]}"
}

check_run_gate() { python3 "$fixture/idle-budget.py" check-run "$OUT" || fail 'run-level check refused; see analysis.json'; }
observe_case() { python3 "$fixture/idle-budget.py" observe "$OUT" "$1"; }
analyze_run() { timeout 300 python3 "$fixture/idle-budget.py" analyze "$OUT"; }
seal_checksums() { (cd "$OUT" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS); }

# The manifest is the only statement of which cases run and in what order. Rows go
# through an array, so no case function can read the loop's input.
run_cases() {
    local one=$1 plan rows row name case stimulus repetition
    check_run_gate
    plan=$(python3 "$fixture/idle-budget.py" plan "$OUT")
    [ -n "$plan" ] || fail 'empty plan'
    mapfile -t rows <<< "$plan"
    for row in "${rows[@]}"; do
        IFS=$'\t' read -r name case stimulus repetition <<< "$row"
        CURRENT_CASE=$name
        if [ -n "$stimulus" ]; then "$one" "$name" "$case" "$stimulus" "$repetition"; else "$one" "$name" "$case"; fi
        if ! observe_case "$name"; then
            [ "$FAIL_FAST" = 0 ] || { STOP_KIND=first-failure; fail "first failure: $name"; }
        fi
        CURRENT_CASE=
    done
}
```

`trace_one` and `power_one`:

```bash
trace_one() {
    local name=$1 case=$2 stimulus=$3 repetition=$4 length=20
    # … body unchanged from `[ "$stimulus" != long-move ] || length=600` on …
}

trace_all() {
    tools_ready; reserve_tracy_port
    cp "$TOOLS/tracy-capture" "$TOOLS/tracy-csvexport" "$OUT/"
    run_cases trace_one
}

power_one() {
    local name=$1 case=$2 owned required rc
    JOURNAL=$OUT/$name.actions.tsv; : > "$JOURNAL"
    start_scene "$case" "$name"
    snapshot "$name" before
    stimulate move
    owned="$NIRI_PID,$WALL_PID,${CLIENT_PIDS[1]},${CLIENT_PIDS[2]}"
    required="$NIRI_PID,${CLIENT_PIDS[1]},${CLIENT_PIDS[2]}"
    setsid python3 "$fixture/idle-budget.py" collect "$OUT" "$name" "$NIRI" "$owned" "$required" "$NIRI_PID" & SAMPLE_PID=$!
    rc=0; wait "$SAMPLE_PID" || rc=$?; SAMPLE_PID=
    [ "$rc" = 0 ] || fail "power collector failed: $name"
    snapshot "$name" after
    check_renderer "$name"
    stop_scene
}

power_all() {
    python3 "$fixture/idle-budget.py" preflight "$OUT/preflight.json"
    run_cases power_one
}
```

In `runtime`, parse flags first:

```bash
runtime() {
    mode=$1; shift
    PILOT=0; FAIL_FAST=1; CURRENT_CASE=; STOP_KIND=
    local flag
    for flag in "$@"; do
        case $flag in
            --pilot) PILOT=1 ;;
            --inventory) FAIL_FAST=0 ;;
            *) echo "FAIL: unknown flag: $flag" >&2; exit 2 ;;
        esac
    done
    if [ "$mode" = prepare ] && [ $# -gt 0 ]; then echo 'FAIL: prepare takes no flags' >&2; exit 2; fi
    # … unchanged from `: "${OUT:?…}"` …
```

Replace the tail after `capture_meta identity … || fail 'identity refused'`:

```bash
    [ "$mode" = prepare ] && return
    write_manifest
    if [ "$mode" = trace ]; then trace_all; else power_all; fi
    local result=0
    analyze_run > "$OUT/analysis.json" || result=$?
    seal_checksums
    return "$result"
}
```

Dispatcher: `prepare|trace|power) runtime "$@" ;;`, and the usage line
`usage: $0 build-tracy|build-power|prepare|trace|power [--pilot] [--inventory]`.

`idle-budget.just`:

```
trace *flags:
    python3 {{root}}/tools/tt material-ec6229-trace -- bash {{fixture}}/idle-budget.sh trace {{flags}}

power *flags:
    python3 {{root}}/tools/tt material-ec6229-power -- bash {{fixture}}/idle-budget.sh power {{flags}}
```

- [ ] **Step 4: Run the suite** — Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.sh fixtures/idle-budget.just fixtures/test_idle_budget.py
git commit -m "feat(idle-budget): run the manifest's cases in one loop, stop at the first failure, --pilot and --inventory"
```

Then close the Task 5 child in niri-material.

---

### Task 6: Every exit records why, reaps OUT's writers, analyzes, and seals

**Files:**
- Modify: `fixtures/idle-budget.sh` (top-level `record_stop`, `teardown_error`,
  `reap_weston`; inside `runtime`: `fail` redefinition, stop variables, signal and
  `ERR` traps, the new `cleanup`; the tail loses its analysis and checksum)
- Test: `fixtures/test_idle_budget.py`

**Interfaces:**
- Consumes: `analyze_run`, `seal_checksums`, `run_cases`' `CURRENT_CASE` and
  `STOP_KIND` (Task 5); `analyze`'s `teardown` (Task 4).
- Produces: `$OUT/stop.json`, `$OUT/teardown.json` (a JSON list of strings),
  `analysis.json` and `SHA256SUMS` written on every exit once the manifest exists.

- [ ] **Step 1: Write the failing tests**

```python
    # A fake Weston: a host socket file, and a shutdown line in OUT's log when killed.
    WESTON_STUB = r"""
start_fake_weston() {
    touch "$XDG_RUNTIME_DIR/$HOST"
    sh -c 'trap "echo caught signal 15 >> \"\$0\"; [ -n \"\$LINGER\" ] || rm -f \"\$1\"; exit 0" TERM
           while :; do sleep 0.05; done' "$OUT/weston.log" "$XDG_RUNTIME_DIR/$HOST" &
    WESTON_PID=$!
}
fake_one() {
    echo "$*" >> "$OUT/calls"
    [ "$1" != "${WESTON_AT:-}" ] || start_fake_weston
    if [ "$1" = "${STOP_AT:-}" ]; then
        case $STOP_KIND_TEST in
            fail) fail boom ;;
            TERM|INT) kill -"$STOP_KIND_TEST" $$; sleep 0.3 ;;
            command) false ;;
        esac
    fi
}
analyze_run() { (cd "$OUT" && ls *.verdict.json 2>/dev/null) | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read().split()))'; }
"""

    def test_every_stop_records_its_kind_and_analyzes_completed_cases(self):
        for kind, status, message in (("fail", 1, "boom"), ("TERM", 143, "TERM"), ("INT", 130, "INT"), ("command", 1, "false")):
            with self.subTest(kind=kind):
                result, out = self.run_loop("trace", extra=self.WESTON_STUB,
                                            env_extra={"STOP_AT": "A-resize-1", "STOP_KIND_TEST": kind})
                self.assertEqual(result.returncode, status, result.stderr)
                stop = json.loads((out / "stop.json").read_text())
                self.assertEqual(stop["kind"], {"TERM": "signal", "INT": "signal"}.get(kind, kind))
                self.assertEqual((stop["exit"], stop["during"]), (status, "A-resize-1"))
                self.assertIn(message, stop["message"])
                self.assertEqual(json.loads((out / "analysis.json").read_text()), ["A-move-1.verdict.json"])

    def test_first_failure_is_recorded_as_such(self):
        result, out = self.run_loop("trace", env_extra={"FAIL_AT": "A-resize-1"})
        stop = json.loads((out / "stop.json").read_text())
        self.assertEqual((stop["kind"], stop["during"], stop["message"]), ("first-failure", "A-resize-1", "first failure: A-resize-1"))

    def test_checksums_hold_after_exit_with_weston_shutdown_logged(self):
        import subprocess
        for label, env in (("clean", {"WESTON_AT": "B-long-move-1"}),
                           ("term mid-case", {"WESTON_AT": "A-resize-1", "STOP_AT": "A-resize-1", "STOP_KIND_TEST": "TERM"})):
            with self.subTest(label):
                result, out = self.run_loop("trace", extra=self.WESTON_STUB, env_extra=env)
                self.assertEqual(result.returncode, 0 if label == "clean" else 143, result.stderr)
                self.assertIn("caught signal 15", (out / "weston.log").read_text())
                check = subprocess.run(["sha256sum", "-c", "--quiet", "SHA256SUMS"], cwd=out, capture_output=True, text=True)
                self.assertEqual(check.returncode, 0, check.stdout + check.stderr)
                self.assertIn("./weston.log", (out / "SHA256SUMS").read_text())

    def test_a_lingering_weston_socket_is_a_teardown_error_and_nothing_changes_after_the_seal(self):
        import subprocess
        result, out = self.run_loop("trace", extra=self.WESTON_STUB,
                                    env_extra={"WESTON_AT": "B-long-move-1", "LINGER": "1"}, timeout=40)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(len(json.loads((out / "teardown.json").read_text())), 1)
        self.assertFalse((out / "stop.json").exists())   # the lib's late fail is sealed out
        check = subprocess.run(["sha256sum", "-c", "--quiet", "SHA256SUMS"], cwd=out, capture_output=True, text=True)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)

    def test_a_failed_or_timed_out_analysis_fails_a_clean_run(self):
        for code in (1, 124):
            with self.subTest(code=code):
                result, _ = self.run_loop("trace", extra=f"analyze_run() {{ return {code}; }}\n")
                self.assertEqual(result.returncode, 1, result.stderr)

    def test_a_failing_teardown_command_keeps_the_runs_stop_reason(self):
        result, out = self.run_loop("trace", extra=self.WESTON_STUB + "stop_clients() { false; }\n",
                                    env_extra={"STOP_AT": "A-move-1", "STOP_KIND_TEST": "fail"})
        self.assertEqual(result.returncode, 1)
        stop = json.loads((out / "stop.json").read_text())
        self.assertEqual((stop["kind"], stop["message"]), ("fail", "boom"))

    def test_second_interrupt_during_analysis_still_tears_down(self):
        extra = self.WESTON_STUB + r"""
analyze_run() { kill -INT $$; sleep 0.3; echo '"analyzed"'; }
"""
        result, out = self.run_loop("trace", extra=extra, env_extra={"STOP_AT": "A-move-1", "STOP_KIND_TEST": "TERM"})
        self.assertEqual(result.returncode, 143, result.stderr)
        self.assertEqual(json.loads((out / "analysis.json").read_text()), "analyzed")
        self.assertTrue((out / "SHA256SUMS").exists())

    def test_stop_before_manifest_writes_no_analysis(self):
        result, out = self.run_loop("trace", extra='write_manifest() { fail "refused before the plan"; }\n')
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads((out / "stop.json").read_text())["message"], "refused before the plan")
        self.assertFalse((out / "analysis.json").exists()); self.assertFalse((out / "SHA256SUMS").exists())

    def test_release_runs_after_the_seal(self):
        import subprocess
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        stub = root / "capture-meta-stub"
        # The lib creates OUT before its first capture-meta call (the existing stub test relies on it).
        stub.write_text('#!/bin/sh\n{ printf "%s" "$1"; [ "$1" != release ] || { [ -e "$OUT/SHA256SUMS" ] && printf " sealed"; }; echo; } >> "$OUT/capture-calls"\n')
        stub.chmod(0o755)
        env = self.fixture_env(root, "trace", capture_meta=str(stub))
        result = subprocess.run(["bash", "-c", self.LOOP_STUBS + "runtime trace --pilot\n"], env=env, capture_output=True, text=True, timeout=20)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((Path(env["OUT"]) / "capture-calls").read_text().splitlines()[-1], "release sealed")
```

- [ ] **Step 2: Run to see them fail**

Run: `python3 -m unittest -v fixtures/test_idle_budget.py -k ShellTests`
Expected: FAIL (no `stop.json`; a TERM leaves no `analysis.json`; `weston.log`
changes after `SHA256SUMS`).

- [ ] **Step 3: Implement**

Top level, after `seal_checksums`:

```bash
# Whichever records first wins, so a specific reason is never replaced by a generic one;
# nothing is written into OUT once it is sealed.
record_stop() {   # kind, exit status, message
    if [ "${SEALED:-0}" != 0 ] || [ ! -d "$OUT" ] || [ -e "$OUT/stop.json" ]; then return 0; fi
    python3 - "$OUT/stop.json" "$1" "$2" "${CURRENT_CASE:-}" "$3" <<'STOP'
import json, sys
path, kind, status, during, message = sys.argv[1:]
with open(path, "x") as stream:
    json.dump({"exit": int(status), "during": during or None, "kind": kind, "message": message}, stream, indent=2)
STOP
}

teardown_error() {
    python3 - "$OUT/teardown.json" "$1" <<'TEARDOWN'
import json, sys
from pathlib import Path
path = Path(sys.argv[1])
errors = json.loads(path.read_text()) if path.exists() else []
path.write_text(json.dumps(errors + [sys.argv[2]], indent=2) + "\n")
TEARDOWN
}

# Weston logs its shutdown into OUT, so it is reaped before the checksum. The lib's
# stop_weston calls fail, which exits; a lingering socket is recorded instead.
reap_weston() {
    if [ -n "$WESTON_PID" ]; then kill "$WESTON_PID" 2>/dev/null; wait "$WESTON_PID" 2>/dev/null; WESTON_PID=; fi
    for _ in $(seq 50); do [ ! -e "$XDG_RUNTIME_DIR/$HOST" ] && return 0; sleep 0.1; done
    teardown_error "Weston socket $HOST still present"
}
```

Inside `runtime`, replace the block from `CLIENT_PIDS=(); SAMPLE_PID=` through
`trap 'exit 143' TERM`:

```bash
    CLIENT_PIDS=(); SAMPLE_PID=; STOP_SIGNAL=; ERR_RECORD=; SEALED=0
    # Lib functions call fail by name, so they record through this one too.
    fail() {
        record_stop "${STOP_KIND:-fail}" 1 "$*"
        echo "FAIL: $*" >&2
        exit 1
    }
    cleanup() {
        local rc=$? signal=$STOP_SIGNAL error=$ERR_RECORD status analysis=0
        set +e
        trap - ERR
        # A handler, unlike an ignored signal, resets in children: Ctrl-C still reaches the analysis.
        trap 'SECOND_SIGNAL=1' INT TERM
        stop_clients
        if [ -n "$SAMPLE_PID" ]; then kill -- "-$SAMPLE_PID" 2>/dev/null; wait "$SAMPLE_PID" 2>/dev/null; SAMPLE_PID=; fi
        if [ -n "$CAP_PID" ]; then kill "$CAP_PID" 2>/dev/null; wait "$CAP_PID" 2>/dev/null; CAP_PID=; fi
        if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null; wait "$NIRI_PID" 2>/dev/null; NIRI_PID=; fi
        reap_weston
        if [ "$rc" != 0 ]; then
            if [ -n "$signal" ]; then record_stop signal "$rc" "$signal"
            elif [ -n "$error" ]; then record_stop command "$rc" "$error"
            else record_stop exit "$rc" "exit $rc"; fi
        fi
        if [ -e "$OUT/manifest.json" ]; then
            analyze_run > "$OUT/analysis.json" || analysis=1
            seal_checksums || analysis=1
        fi
        SEALED=1
        status=$rc
        if [ "$status" = 0 ] && { [ "$analysis" != 0 ] || [ -e "$OUT/teardown.json" ]; }; then status=1; fi
        # native_cleanup reads the status from $? and ends in exit.
        (exit "$status")
        native_cleanup
    }
    trap cleanup EXIT
    trap 'STOP_SIGNAL=INT; exit 130' INT
    trap 'STOP_SIGNAL=TERM; exit 143' TERM
    set -E
    trap 'ERR_RECORD="$BASH_COMMAND (line $LINENO) exited $?"' ERR
```

The tail loses its analysis and checksum:

```bash
    [ "$mode" = prepare ] && return
    write_manifest
    if [ "$mode" = trace ]; then trace_all; else power_all; fi
}
```

- [ ] **Step 4: Run the suite** — Expected: all pass, including
`test_runtime_signals_reap_capture_sampler_clients_and_compositor` (prepare mode:
`stop.json` is written, with no analysis).

- [ ] **Step 5: Commit**

```bash
git add fixtures/idle-budget.sh fixtures/test_idle_budget.py
git commit -m "feat(idle-budget): every exit records why, reaps OUT's writers, then analyzes and checksums"
```

Then close the Task 6 child in niri-material.

---

### Task 7: Documentation, full gate, and the results branch

**Files:**
- Modify (experiments): `docs/results/2026-09-11-idle-budget.md`
- Modify (niri-material): `docs/specs/2026-09-27-idle-budget-fail-fast-design.md`
  (status line), `docs/materials/2026-09-11-idle-budget-evidence.md` (one
  sentence naming the pilot)

- [ ] **Step 1: Results doc.** In the reproduction section, after the trace and power
  commands, add:

```markdown
Each lane runs a pilot first: `… idle-budget.just trace --pilot` (A-move-1,
A-resize-1, C-move-1, D-move-1; about 9 min) or `… power --pilot` (sham block 1;
about 10 min). A run stops at its first failing case and writes `analysis.json` for
the cases it completed, with the reason in `stopped`; `--inventory` runs every case
regardless. Each case's verdict is in `<case>.verdict.json` as it lands.
```

Replace the *Caveats* bullet "The analyzer does not require the trace to reach the
window end…" with: "Since `material-b15ad7` the analyzer requires each trace to reach
its window end. This run predates that gate, and every trace here runs about 58 s past
its window."

- [ ] **Step 2: Full suite.** Run the test recipe. Expected: all pass. Then
  `bash -n fixtures/idle-budget.sh`.

- [ ] **Step 3: Commit**

```bash
git add docs/results/2026-09-11-idle-budget.md
git commit -m "docs(idle-budget): the pilot, fail-fast, and per-case verdicts"
```

Pushing `results/idle-budget` is an action outside the repository: ask the person
first.

- [ ] **Step 4: niri-material.** Set the spec status to "implemented (experiments
  `<sha>`); live pilots pending", add the sentence to the evidence doc, `tasks done`
  the Task 7 child, `tasks check`, commit `docs(specs): idle-budget fail-fast
  implemented`.

---

### Task 8: Live pilots (quiet host; the person runs them)

Not an agent step: `tasks park <Task 8 child> … --reason quiet --waiting-on user
--needs headless --minutes 35`. From a TTY with the desktop session stopped (see the
settle-gated capture practice), in the niri-material checkout:

```bash
X=~/d/niri-experiments/.worktrees/material-b15ad7/fixtures
W=$NIRI_MATERIAL_WORK_ROOT/material-265eb0; T=$(date +%Y%m%dT%H%M%S)
# 1. trace pilot, ~12 min
OUT=$W/pilot-trace-$T NIRI_BIN=$W/trace-target/release/niri MATERIAL_ROOT=$PWD just --justfile $X/idle-budget.just trace --pilot
# 2. interrupted trace pilot, ~6 min: press Ctrl-C after the second verdict line
OUT=$W/pilot-trace-int-$T NIRI_BIN=$W/trace-target/release/niri MATERIAL_ROOT=$PWD just --justfile $X/idle-budget.just trace --pilot
# 3. power pilot, ~12 min, dedicated DRM session
OUT=$W/pilot-power-$T NIRI_BIN=$W/power-target/release/niri POWER_OUTPUT=<output> POWER_MODE=<mode> POWER_SCALE=<scale> \
  MATERIAL_ROOT=$PWD just --justfile $X/idle-budget.just power --pilot
```

Both binaries and their `niri.identity.json` were present on 2026-09-27. Recheck
before parking; if either is gone, the next step for the person begins with
`build-tracy` / `build-power`, plus about 10 min each. For run 2, wait until
`analysis.json` exists before judging it: Ctrl-C also reaches `just` and `tt`, which
may return to the prompt before the fixture finishes its cleanup. Pass criteria, which
the agent reads afterwards from `OUT` (not from the wrapper's exit status):
1. Runs 1 and 3 hold four verdicts each, with `analysis.json` `pilot: true`,
   `complete: true`, and for power a `pilot_block`. Neither has a `stop.json`.
2. Run 2's `stop.json` is `kind: "signal"`, message `INT`, `exit: 130`. Its
   `analysis.json` covers the cases whose verdicts landed, and `sha256sum -c
   SHA256SUMS` passes.
3. No settle refusal was caused by `observe` running just before a settle.
4. The power pilot's preflight passes under the pinned seat exemption.

Each attempt gets its `run:` note in the tasks format. Then close the task and the
parent `material-b15ad7`.
