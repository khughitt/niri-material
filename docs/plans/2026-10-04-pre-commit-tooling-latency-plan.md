# Pre-commit tooling latency implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task by task in the existing worktree. Steps use checkbox syntax. Owner plan review precedes implementation.

**Goal:** Restore pre-commit latency without removing lifecycle coverage from affected commits or full validation.

**Architecture:** Native unittest discovery retains the fast/full class skip contract. Full validation schedules individual lifecycle cases alongside the remaining suite in independent processes. A static source/AST check protects narrow full-route patterns on every commit; CI additionally rejects unexpected skips.

**Tech stack:** Python standard library, Bash, just, Git, existing `tools/tt` and host-budget; Ubuntu 24.04 media dependencies from the spec.

**Spec:** [Accepted design](../specs/2026-10-04-pre-commit-tooling-latency-design.md).
**Evidence:** [Probe receipts, source reproduction and route recount](../../tasks/files/material-cd7782/material-pre-commit-revision-evidence.md).
**Workspace:** Reuse `.worktrees/pre-commit-latency`, branch `fix/pre-commit-latency`; setup is complete. Run commands from that worktree. No shared launcher, service or desktop pointer changes.

**Execution status:** Tasks 1–5 are committed and reviewed. All six tasks and the accepted fast-runner amendment are implemented and verified on 2026-10-05.

## Global constraints

- Fast whole-hook warm median **≤35 s**; full whole-hook warm median **≤45 s**, aim **≤35 s**. Both code routes record `hook-pre-commit`; keep slow and cold samples.
- `NIRI_TOOLING_FAST`: unset/`0` full, `1` fast; every other value, including empty, raises before decorators. Repository commands explicitly set their mode.
- Full default **10 child processes total**, including one remainder process. When set, `NEXTEST_TEST_THREADS` caps the total; accept only positive ASCII decimal integers. No new budget variable.
- Preserve production waits, `BOUND_S = 5`, expected TERM status 143, partial evidence, VT restoration and child/lock cleanup. No retry-until-green acceptance, skip of the blocker, or bound widening.
- Preserve all existing assertions. Splitting four scenarios into native methods adds three cases; new contract tests add more. Inventory comparisons use the resulting current IDs, not a hard-coded 287.
- Keep `test_cmd` Rust only. Full tooling runs once in full check/gate/non-CI push and the separate tooling CI job. Rust CI recipes get no media dependencies.
- Supported tooling versions: just **1.58.0**, Cargo **1.99.0**; base Ubuntu 24.04, Python **3.12.3**, PyGObject **3.48.2**, GStreamer **1.24.2**, Git **2.43.0**.
- Runtime packages: `python3 git jq procps dbus python3-gi gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0 gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire`. Bootstrap transport needs `ca-certificates xz-utils`.
- CI allowlist is exactly the five optional test IDs in the spec; dependency skips, unexpected dynamic skips, import errors and worker failures fail. Optional tests may execute when dependencies are available.
- Static checks inspect source only: no tracing dependency, privileges, lifecycle execution or maintained import graph.
- Use `just test-one` for focused work, then `just test-fast` with the Python override where appropriate before each code commit. Keep timing wrappers. `tasks check` before every commit; each step child closes in its deliverable commit.
- Execute inline. Start each tracked child and run `trial-arm`; if halt routing requires an override, record `tasks start <child> --force --reason 'step of the active latency halt remedy'`. Never start unrelated work.

## Review focus

These conditions have explicit checks in the tasks below:

1. TERM between locking and journal completion: assert status 143 and complete cleanup, including a second TERM during cleanup (Tasks 1, 2, 6).
2. Low/malformed worker budget: one child still runs all tests exactly once; malformed input fails before any spawn (Task 2).
3. New shell helper, changed class path, transitive Python import, renamed/deleted staged subject or undecodable Git path: static check or routing must fail closed (Tasks 3, 5).
4. Missing required GI/GStreamer binding replaced by an optional skip, or a worker exits without a result: CI must fail with named diagnostics (Tasks 2, 4, 5).
5. Interrupted parent while a stub driver has started its own process group: reap workers and recorded driver children; do not leak a bus/socket or delete another case's directory (Tasks 1, 2).

## File map and interfaces

| File | Responsibility |
| --- | --- |
| `docs/materials/scripts/optic-settling-smoke.sh` | Split journal end-time substitutions to avoid older Bash trap/parser corruption |
| `tools/test_optic_settling.py` | Deterministic journal signal regression, preserved lifecycle cases, four independent screencast methods, actual static coverage regression outside the omitted class |
| `tools/test_vt_lib.py` | Shared fast-mode decorator; retain VT behavior |
| New `tools/tooling_tests.py` | Mode/budget parsing, static check, native full coordinator and CI skip guard; import has no discovery side effects |
| New `tools/test_tooling_tests.py` | Cheap synthetic parser, inventory, worker, skip and static-source contracts |
| `tools/fake_screencast.py`, `tools/test_screencast_consumer.py` | Portable GIO registration and consumer startup/failure coverage |
| `tools/upstream-report`, `tools/test_upstream_report.py` | Validate missing merge-tree result fields independently of Git version |
| `.githooks/pre-commit`, `justfile`, `tools/test_gates.py` | Narrow staged routing, recipes and timing/count/failure propagation |
| `.github/workflows/ci.yml`, `AGENTS.md` | Separate guarded tooling job and accurate test guidance |

`tools/tooling_tests.py` exposes these interfaces; later tasks must use the same names:

| Interface | Contract |
| --- | --- |
| `fast_mode() -> bool` | Parse the current fast-mode environment; no discovery side effects |
| `worker_limit() -> int` | Strict budget parsing; total child cap, default 10 |
| `assert_static_coverage(root: Path, patterns: Sequence[str]) -> None` | Raise a named `ValueError` for uncovered or unresolved repository dependencies |
| `flatten(suite: unittest.TestSuite) -> list[unittest.TestCase]` | Preserve all native cases in discovery order |
| `run_full(ci: bool = False) -> int` | Aggregate native validation; zero only for a complete passing permitted inventory |
| `run_job(ids: list[str]) -> dict[str, object]` | Own one subprocess worker and return its validated receipt |

CLI: `python3 -m tools.tooling_tests --full`, `--full --ci`, or `--check-paths`.
The coordinator's private worker entry point accepts a result-file path and a list
of native IDs. It is internal; do not expose a new user-facing selection system.

### Task 1: Fix the journal TERM parser window

**Files:** Modify driver `stim()` around line 545 and `tools/test_optic_settling.py`; no mode/routing enablement yet.
**Consumes:** Existing driver, real locked-session cleanup test, ten-iteration minimal evidence.
**Produces:** Deterministic journal signal regression; driver retaining 143 and cleanup with split substitutions.

- [x] Add `JournalSignalTests` outside `DriverCleanupTests`. Extract the real `stim()` function body from the source, wrap it with the existing literal TERM action and a small EXIT cleanup marker, and inject TERM from the **second** `mono()` call. Use a temporary directory shared across command substitutions to count calls, as in the evidence reproduction. Execute through `subprocess.run(['bash', '-c', script], timeout=15, ...)`; assert 143, empty stderr and the cleanup marker. Do not test a duplicated copy of the arithmetic expression.

```python
body = script_source.split('stim() {', 1)[1].split('\n}\n', 1)[0]
run = subprocess.run(['bash', '-c', prefix + 'stim() {' + body +
                     '\n}\nstim lock 2 true\n'],
                     capture_output=True, text=True, timeout=15)
self.assertEqual(run.returncode, 143, run.stderr)
self.assertEqual(run.stderr, '')
self.assertIn('cleanup', run.stdout)
```

- [x] Confirm red on Ubuntu Bash 5.2.21 through `just test-one`; host Bash 5.3.20 already passes this old-expression case. Keep the failure receipt. The earlier minimal original-body probe fails Ubuntu 10/10; it does not establish real-driver resolution.
- [x] Replace only the end calculation in `stim()`. Preserve the original order: obtain current monotonic time, convert effect duration, add; no added sleeps or changes to the start timestamp.

```bash
    end=$(mono)
    local effect_ns
    effect_ns=$(awk -v s="$effect" 'BEGIN { printf "%d", s * 1e9 }')
    end=$((end + effect_ns))
```

- [x] Register every driver returned by `DriverCleanupTests.start()` for cleanup at launch: a cleanup callback stops its recorded process group if still running and always waits for the driver. Retain `kill_leftovers` for stub PIDs and the existing production cleanup assertions. VT tests already own their process cleanup; inspect and retain it. This gives Task 2's interrupted worker a cleanup callback for the driver as well as its children.
- [x] Run the focused journal test, locked-session test, second-TERM test and VT restoration tests through the front door. Then run **10 consecutive isolated locked-session cases per environment**, stopping at the first failure and retaining elapsed time, status, stderr and PID cleanup. Use the evidence's existing Ubuntu dependency/archive bootstrap and a disposable Git copy for Ubuntu; pilot only the journal and locked-session cases here, because consumer portability is fixed later in Task 4. No GI-dependent pilot or complete Ubuntu suite is required in this step. Provide the disposable container's host-budget forwarding stub shown in Task 6 before calling host front doors; it is not a shared host installation. A real-case failure blocks later enablement, even if the minimal regression is green.

```bash
just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_optic_settling.JournalSignalTests
just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest discover -s tools 2>&1' test-fast
```

- [x] `tasks check`; close the child with root-cause and real-case receipts; commit `fix: preserve TERM during stimulus journaling`. Full existing commit hooks are expected here; keep their timing.

### Task 2: Add explicit mode and parallel full execution

**Files:** Create `tools/tooling_tests.py`, `tools/test_tooling_tests.py`; modify the two omitted classes' decorators and split the screencast method in `tools/test_optic_settling.py`.
**Consumes:** Task 1's cleanup callbacks and regression, native discovery, spec's five optional IDs.
**Produces:** `fast_mode`, `worker_limit`, `flatten`, `run_full` and full/CI CLI; sequential discovery remains usable.

- [x] Pin parser contracts with table-driven unittest cases and subprocess import checks. Valid mode values are unset/`0`/`1`; valid budget values are unset/`1`/`10`/`999`. Reject empty, `0`, `-1`, `1.5`, whitespace, Unicode digits and arbitrary strings for the budget. Verify no worker was created on parser error.

```python
def fast_mode():
    raw = os.environ.get('NIRI_TOOLING_FAST')
    if raw not in (None, '0', '1'):
        raise ValueError('NIRI_TOOLING_FAST must be unset, 0 or 1')
    return raw == '1'

def worker_limit():
    raw = os.environ.get('NEXTEST_TEST_THREADS')
    if raw is None:
        return 10
    if re.fullmatch(r'[0-9]+', raw) is None or int(raw) < 1:
        raise ValueError('NEXTEST_TEST_THREADS must be a positive decimal integer')
    return min(10, int(raw))
```

- [x] Decorate the two classes with the shared parser; parse at import before decorators, not lazily per method. Use reason `NIRI_TOOLING_FAST=1; use full validation or NIRI_TOOLING_FAST=0`. Repository full CLI sets `0` before discovery and in each child's environment; fast and invalid raw-discovery imports retain their contract.
- [x] Replace the four-scenario method with these native methods, sharing a small assertion helper: `test_screencast_refuses_wrong_probe_size`, `test_screencast_refuses_small_probe`, `test_screencast_refuses_wrong_sample_size`, `test_screencast_refuses_dead_consumer`. Copy each existing environment/message pair exactly; fresh setUp/cleanup per method. Retain case/output cleanup assertions. Measure each separately through `just test-one` before enabling the coordinator and append their timings to evidence. This resolves the remaining longest-piece uncertainty; do not assume all four cost ≤19 s.
- [x] Implement discovery via `unittest.defaultTestLoader.discover('tools')` after explicit full mode. Flatten recursively and reject duplicate IDs. Classify exactly the two lifecycle class names; all other cases form one remainder job. Compare the union and count of jobs against discovery before spawning. In budget 1, execute jobs sequentially without dropping the remainder.
- [x] Use `ThreadPoolExecutor(max_workers=worker_limit())` only to supervise independent `subprocess.Popen` Python workers; each child runs native unittest, with no shared environment mutation. Submit the remainder first, then one lifecycle ID per job. This avoids private multiprocessing APIs for cancellation. Give each worker a private result file inside the coordinator's `TemporaryDirectory`; stdout/stderr remain private until collection. Run module mode so `tools.*` imports can resolve; in the worker add the repository's `tools` directory to `sys.path` before `loadTestsFromNames(ids)`, retaining discovery's native `test_*` module IDs.

```python
jobs = [remainder_ids] + [[case_id] for case_id in lifecycle_ids]
expected = {case.id() for case in inventory}
assigned = [case_id for job in jobs for case_id in job]
if len(assigned) != len(expected) or set(assigned) != expected:
    raise RuntimeError('full discovery partition lost or duplicated cases')
child_env = {**os.environ, 'NIRI_TOOLING_FAST': '0'}
with ThreadPoolExecutor(max_workers=worker_limit()) as pool:
    try:
        results = list(pool.map(run_job, jobs))
    finally:
        cancel_pending_and_reap_workers()
```

`run_job(ids)` owns one worker handle and returns its validated receipt. Protect
its active-handle registry with a lock. `cancel_pending_and_reap_workers() -> None`
sets a cancellation Event and stops/waits the registered children before executor
shutdown; `run_job` checks that Event while holding the same lock before spawning,
so cancellation cannot miss a late worker. Give jobs a 120-second outer deadline,
then signal workers for cleanup before killing tracked survivors; this supervisor
bound does not replace any existing test assertion. Receipt fields are `ids`, `ran`, `skipped`
(ID/reason pairs), `failures`, `errors`, and per-ID elapsed times. Print aggregate
`Ran N tests` and one unittest-style verdict for `tools/tt`; preserve named failure
tracebacks and skip reasons. Missing/malformed receipt or nonzero worker exit fails,
even if other receipts pass. No worker records a separate top-level timing run.

- [x] Apply the CI optional skip allowlist before scheduling (class/method decorators) and after collecting native results (dynamic skips). Copy all five IDs from the spec exactly. Optional tests may pass; missing required classes cannot replace optional skips. Test a missing mandatory class, an unexpected dynamic skip, an allowed optional skip, a failure, an import error and `os._exit(7)` using disposable synthetic suites. Test exact inventory union, count, mode override and budget 1 with cheap cases; do not rerun lifecycle tests inside these unit controls.
- [x] Handle parent INT/TERM by cancelling queued jobs, signalling tracked workers, waiting for cleanup, then killing/waiting only tracked survivors. Workers convert interruption into a nonzero outcome. A small result subclass retains every started TestCase; the worker's `finally` calls `doCleanups()` on them before exit (already completed cleanups are empty). Retain the references even after native `stopTest`, which runs during unwinding. Native unittest does not guarantee all cleanup on KeyboardInterrupt. Do not rely on killing only the worker group: real stub drivers create their own session. Task 1's registered driver cleanup closes that gap. A synthetic fixture must record a child PID before interruption and prove it is reaped; validate actual drivers again in Task 6. Preserve EXIT cleanup too.
- [x] Compare sequential full native discovery with parallel IDs, assertions and optional skips; run focused contracts, then the full Python gate via the parallel override. Keep any full-route implementation hook data after this point as early parallel evidence, though it precedes remedy verification.

```bash
just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests
just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast
```

- [x] `tasks check`; close child; commit `feat: parallelize full tooling validation`.

### Task 3: Check narrow lifecycle paths statically

**Files:** `tools/tooling_tests.py`, `tools/test_tooling_tests.py`, `tools/test_optic_settling.py`, `justfile` (add only `tooling_full_paths` here).
**Consumes:** `assert_static_coverage` interface and helper module from Task 2.
**Produces:** Cheap source checker, direct `--check-paths` CLI and a native regression outside skipped classes.

- [x] Define `tooling_full_paths` in the justfile from the exact spec list, including the sourced shared helper and the two new helper/contract files. Keep one route list, consumed by both hook classification and static tests through `just --evaluate tooling_full_paths`.
- [x] Add temporary-root synthetic fixtures: entry driver sources a helper that sources a second helper; a selected test class names a root-relative script; covered Python imports `tools.other`, including `from tools import other`. Start with all paths allowed, then remove each found path in turn and assert failure naming its source location. Include missing source, source cycle, unsupported dynamic source operand and unresolved module cases. No fixture imports or executes its source.

```python
with self.assertRaisesRegex(ValueError, 'nested-lib.sh'):
    assert_static_coverage(root, patterns_without_nested_lib)
with self.assertRaisesRegex(ValueError, 'tools/other.py'):
    assert_static_coverage(root, patterns_without_other)
```

- [x] Implement a bounded source scanner, not a shell interpreter: inspect lines starting with `.` or `source`, use `shlex` for the operand, resolve literal `$HERE/…`, `${HERE}/…`, `$ROOT/…`, `${ROOT}/…` or relative helper paths. Reject unsupported repository source operands instead of guessing. Recurse through source statements with a visited set and fail if an input is missing. Inspect literal `$ROOT/tools/…` references in the entry driver, resolving analyzer and consumer paths; generated `$OUT` fixtures and external tools are outside this source contract. Read helper contents for further source statements; do not build a general shell call graph or execute heredocs.
  Check `$ROOT/tools/…` references only in the entry driver because references that lifecycle tests replace with stubs are outside the contract, including the helper's `CAPTURE_META` fallback to `capture-meta`.
- [x] While scanning the driver and recursively sourced helpers, reject command substitution inside arithmetic expansion (`$(( … $(…) … ))`), naming the file and line; add a synthetic fixture so the whole TERM parser defect class is guarded deterministically on the fast route.
- [x] Parse Python with `ast`. Resolve `tools.*` imports in the covered modules and their transitive tools imports to module `.py` or package `__init__.py`, allowing the existing namespace package root. Resolve imports rather than imported attributes: `from tools.optic_settling import FAMILIES` adds that module once. Handle relative imports within covered `tools` modules, and reject missing/ambiguous modules. Test `from tools import other` separately.
- [x] For the two selected classes, inspect repository-root `Path` division expressions and referenced module-level path definitions (including `LIB`). Recognize the existing `Path(__file__).resolve().parents[1]` and `self.root` anchors; accept only literal path segments beneath the repository root, and fail on dynamic root-path construction. Do not treat temporary-directory paths or docstrings as dependencies. Check each found repository path with `fnmatch.fnmatchcase` against `tooling_full_paths`, reporting the referring file/line. No maintained module-edge list.
- [x] Add `LifecycleCoverageTests.test_subject_paths_are_full_routed` outside `DriverCleanupTests`; it reads the patterns and invokes the source checker. Verify it still executes with fast mode `1`. `--check-paths` performs the same check without unittest discovery, for docs-only commits.

```python
patterns = subprocess.check_output(
    ['just', '--evaluate', 'tooling_full_paths'], text=True).split()
assert_static_coverage(ROOT, patterns)
```

- [x] Focused static tests through `just test-one`, then Python `just test-fast` via parallel full override. Confirm checking real sources succeeds without media startup or lifecycle execution. `tasks check`; close child; commit `test: guard lifecycle routing with static source checks`.

### Task 4: Make full CI dependencies portable and explicit

**Files:** `tools/fake_screencast.py`, `tools/test_screencast_consumer.py`, `tools/upstream-report`, `tools/test_upstream_report.py`, gate test host-budget stub if not already added.
**Consumes:** Spec dependency evidence; Task 2's guarded full runner.
**Produces:** Portable fake-service registration, deterministic invalid merge-tree regression and clean-runner gate stubs.

- [x] Add a mocked `subprocess.run` merge-tree result for each status 0 and 1 with empty stdout and meaningful stderr; assert `ReportError` retains stderr. Keep directory-conflict-with-valid-tree tests. This fails independently of the installed Git version.

```python
with unittest.mock.patch.object(report.subprocess, 'run', return_value=subprocess.CompletedProcess(
        [], 1, stdout='', stderr='invalid ref diagnostic')):
    with self.assertRaisesRegex(report.ReportError, 'invalid ref diagnostic'):
        report.merge_tree(self.root, 'base', 'upstream', 'fork')
```

- [x] Add the protocol guard immediately after splitting stdout:

```python
fields = result.stdout.split('\0')
if not fields[0]:
    raise ReportError(f'git merge-tree returned no result tree: {result.stderr.strip()}')
```

- [x] Change only `register_object_with_closures2` to `register_object` in the fake service. No runtime version branch or fallback layer. Run all six consumer startup/failure cases on both environments, and report tests against host Git and Ubuntu Git 2.43.
- [x] Make gate fixtures provide `host-budget run --` with argv forwarding; assert the remainder of the command is preserved. The test must not resolve shared ops tooling inside CI. Keep the timing and nonzero forwarding assertions.
- [x] Through the existing CI command override, pilot one consumer, journal, gate-argv and Cargo-isolation case in clean Ubuntu; assert `gst-inspect-1.0 pipewiresrc` first. Run the guarded full suite once and retain skip IDs. Negative controls remove a required binding in a disposable environment or inject a required dependency skip, and inject a dynamic unexpected skip and a lifecycle failure; each must fail for its intended reason, not a startup failure. Use the previously successful missing-package control as baseline evidence, not as proof of new-runner behavior.
- [x] Focused suites and Python full gate through just; `tasks check`; close child; commit `fix: make tooling validation portable on Ubuntu`.

### Task 5: Route commits and wire guarded full CI

**Files:** `.githooks/pre-commit`, `justfile`, `tools/test_gates.py`, `.github/workflows/ci.yml`, `AGENTS.md`; update the spec status only when changes land.
**Consumes:** Tasks 1–4's green prerequisites, parallel runner and static path list.
**Produces:** Spec's exact recipe table, fail-closed full routing and separate tooling CI job.

- [x] Extend gate fixtures first. Assert subject/test/shared-helper/client/full-wiring paths select full; unrelated tools/capture scripts and Rust paths select fast; existing guide/task/spec paths select docs-only. Test mixed paths, deletions, both rename endpoints, empty index, failed Git/pattern reads and undecodable UTF-8. A partial classification may never select fast. Keep LFS behavior intact.
- [x] Hook evaluates both lists successfully and reads staged paths via checked NUL-delimited `git diff --cached --name-only --no-renames -z`, strict UTF-8. Full wins over docs-only; any classification error falls back to full (or fails before recipes when justfile cannot load). Both code recipes record `hook-pre-commit`.
- [x] Share check fragments; put only these tooling commands in their respective checks:

```just
# Explicit flags override a caller's exported fast mode.
tooling_fast_cmd := "env NIRI_TOOLING_FAST=1 python3 -m unittest discover -s tools 2>&1"
tooling_full_cmd := "env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full 2>&1"
tooling_paths_cmd := "env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --check-paths"
```

`check_cmd` uses fast tooling; `full_check_cmd` substitutes full tooling once.
`docs_check_cmd` adds `tooling_paths_cmd` to existing hygiene/task/report/pin checks.
Stage the report as today. Add host `check-full` and `hook-pre-commit-full`, retaining
tt/host-budget wrappers. `gate` becomes full check then Rust `test`; other-remote
push uses full check then Rust `test_cmd`; CI-covered origin push remains fast check
plus config/IPC nextest. `test`, `ci-test`, `ci-test-release` remain Rust-only. Add
`ci-tooling-test` using explicit mode 0 and `--full --ci`, through tt without host-budget.

- [x] Recipe tests override tooling/Rust commands with marker fixtures: assert exact once-only execution, failures forwarded, exported mode `1` cannot weaken full routes, and both full/fast code-hook records use the same target/count. Docs-only static check must run without lifecycle discovery.
- [x] Add separate Ubuntu job using `actions/checkout@v6`, the existing Rust toolchain action pinned to supported **1.99.0**, setup-just **1.58.0**, the exact runtime APT list, factory preflight and `just ci-tooling-test`. Do not alter Rust jobs' dependency sets. The job's guarded runner names the five optional skips from the spec and rejects every other one.
- [x] Update the justfile `check_cmd` comment, hook “Seconds only” header/classification comments and AGENTS gate/Python guidance. Document raw discovery's exported fast-variable caveat, mode 0 reference discovery, static docs checks, lifecycle subject routing, default/capped child count and distinct full CI recipe.
- [x] Focused gate tests and affected Rust `just test-fast`; Python full gate via the coordinator. Keep implementation full-hook timings: serial approximately 240 s before coordinator availability, parallel records afterward. None qualifies for incident verification until the final remedy timestamp. `tasks check`; close child; commit `feat: route lifecycle commits through parallel validation`.

### Task 6: Verify concurrency stability and close latency evidence

**Files:** `tools/tooling_tests.py`, `tools/test_tooling_tests.py`, `justfile`, `tools/test_gates.py`, `AGENTS.md`, evidence via `tasks attach/detach`, task notes and document status. The fast-route amendment was accepted on 2026-10-05. No new runtime knobs or sleep changes.
**Consumes:** Complete implementation and real hook/CI front doors from Task 5.
**Produces:** Consecutive-green stability records on both environments, actual full/fast timings, verifier output and incident disposition.

**Fast-route amendment — accepted 2026-10-05:**

- [x] Add focused synthetic contracts before changing the coordinator: fast overrides exported mode 0, full still overrides 1; two fast workers preserve every discovered ID, skip and native outcome exactly once; whole modules/class fixtures stay within one worker; a budget of one is sequential and malformed budgets fail before discovery; `--fast --ci` and public misuse of private worker mode fail before discovery. Compare against explicit mode-1 native discovery, including the two class skips and expected/unexpected successes. Exercise cancellation under both worker modes without changing the existing cleanup protections.
- [x] Extend the existing standard-library coordinator with explicit `--fast`, mode 1 before discovery/worker loading, and private worker mode arguments. Group modules by native inventory; greedily assign largest case-count module groups into at most two buckets, preserving internal native order. Use `min(2, worker_limit())` for fast; keep full scheduling, ten-child default, guarded CI, exact partition checks and native receipt/verdict handling intact. Add no module timing map, tracing, framework or environment variable.
- [x] Change only `tooling_fast_cmd` to `env NIRI_TOOLING_FAST=1 python3 -m tools.tooling_tests --fast`. Update the recipe contract markers to distinguish the two public modes and retain all once-only, order, count, CI and failure assertions. Keep routing patterns, Rust commands and CI full command unchanged. Document two fast children, the shared budget cap and raw discovery as the reference.
- [x] Through `just test-one`, compare the final fast coordinator with native mode-1 discovery by IDs, skip identities and verdict on the incident host and clean Ubuntu; expected current counts are 317 with 37 host or 40 Ubuntu fast skips. Required consumer/media tests remain included. Run the required Python full `just test-fast` override, affected Rust selector if relevant, task checks and the hooks; commit `perf: parallelize fast tooling by module`. Request a scoped review of this amendment's implementation and resolve Important findings before acceptance.
- [x] Then repeat the original Task 6 gates below on that final code revision: both environments' isolated/consecutive-full loops and actual-driver cancellation, warm stage timings, a new UTC remedy timestamp, and at least three qualifying actual staged hooks per code route with no command overrides. Retain the completed revision's stability and failed headroom attempts as earlier evidence. Close neither this child nor the incident before all thresholds and the actual verifier pass.

- [x] Reproduce clean Ubuntu using the earlier evidence Dockerfile's pinned Ubuntu digest and validated just/Cargo archives. Source mounts are read-only; copy into a disposable `/repo`, initialize Git there, and keep result output separate. For focused host recipes, create a private `probe-bin/host-budget` inside the disposable container, chmod it executable and prepend that directory to the container PATH. Its contents are the following forwarding script; `ci-tooling-test` itself needs no host-budget. The same bootstrap permits Task 1's early focused Ubuntu tests. Inspect a pilot covering journal, lock cleanup, consumer, gate forwarding and Cargo isolation before full runs. All commands go through just; Docker stays foreground/`--rm`, tracked by the harness with bounded timeout.
```sh
#!/bin/sh
[ "$1" = run ] || exit 2
shift
[ "$1" = -- ] || exit 2
shift
exec "$@"
```

- [x] Run the **10 isolated locked-session repetitions** from Task 1 on the final implementation on both the incident host and clean Ubuntu, unless Task 1's ten-run receipts are at this exact final implementation revision. Each is a fresh front-door run; stop on a failure and analyze it. Do not combine ten method invocations into one shared-fixture test or relax timing. Then run **five consecutive green parallel full runs per environment** using the default total 10-child cap (retain the actual host-budget cap). Every full run includes the locked-session case under concurrency. Record the locked case's time, all IDs, skip identities, errors and cleanup for every attempt. This is the concurrency stress loop and the full-suite stability gate. A failure blocks acceptance, resets the consecutive streak after a diagnosed fix, and stays in evidence.

```python
# Run inside the target environment, with the repository as cwd.
# The harness tracks this foreground controller and its inherited children.
for iteration in range(10):
    subprocess.run(['just', '--set', 'one_cmd',
        'env NIRI_TOOLING_FAST=0 python3 -m unittest', 'test-one',
        'tools.test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client'],
        check=True, timeout=90)
for iteration in range(5):
    subprocess.run(['just', 'ci-tooling-test'], check=True, timeout=180)
```

On the host use the host `test-fast` parallel full override rather than CI when
checking host-budget behavior. Clean Ubuntu exercises the default ten-child
configuration; host receipts retain their actual cap. Do not override a host
budget to make performance pass. Record cap-induced slow runs and resolve their cause.
Each environment needs a complete consecutive-green set, not a single pass.

- [x] Interrupt a representative actual driver worker under the coordinator and verify every recorded worker/driver/stub PID is gone; retain no leaked bus/socket or runtime directory. The cancellation negative control fails by design and sits outside the five green stability runs.
- [x] Warm this worktree's own Rust artifacts through `just check`; preserve cold/refresh records. Run actual staged hook routes with a private index if needed to avoid modifying the working index, **without command overrides**, under the standard host budget/timing log. Keep actual 10-child/capped full data separate from theoretical reviewer estimates. Before declaring remedy active, all fixes and routing must be in place and stability must be green; record a UTC remedy timestamp in a task note.
- [x] Obtain at least **three successful, uncontended, unwidened actual full hooks and three fast hooks strictly after the remedy timestamp** on the incident host. Measure all hook stages; require fast median ≤35 s and full median ≤45 s, aim ≤35 s. Keep earlier parallel implementation records as useful performance data; they cannot count toward this post-remedy check. A busy host or widened run does not qualify. Run sequentially and inspect exclusions before retrying.

```bash
tt-latency verify material-cd7782 --after '<recorded UTC remedy timestamp>'
```

- [x] Repeat for every host named in breach notes (currently one host). Require exit 0 and keep its actual count/median/exclusions in evidence. If full speed, stability or the verifier fails, retain the halt, diagnose the stage/worker evidence and revise the affected work; do not close on exhausted effort or fast-only success.
- [x] Harvest timings with `tt-report`, inspect `host-load --section session`, restore any host pointer before parking (none is needed here), and remove only owned scratch containers/images. Preserve the existing worktree for review/integration; no push or PR is authorized.
- [x] Self-review spec coverage, run the required whole-branch review workflow and correct reproduced Important findings before closing. Update docs to verified behavior, close this child, then `tasks done material-cd7782` with verifier output in the same final evidence commit. `tasks check`; commit `docs: verify pre-commit latency remedy`. Personal-profile local integration follows the applicable finishing workflow; external writes remain gated.

## Plan acceptance and execution handoff

The owner conditionally accepted the design once runtime read observation became
a static source check; that substitution is complete. The implemented TERM fix and production/concurrency stability passed on both
environments. Fast-route headroom is verified; the execution amendment was accepted before implementation on 2026-10-05. Historical probe and acceptance
receipts remain in the linked task evidence.

The owner accepted this plan with two Task 3 edits incorporated above; no further
review round is required. The agent executes inline in this worktree, taking its tracked
children in order, retains failed attempts, and closes the halt only with successful
latency verification. This review gate does not authorize a GitHub write or desktop use.

[Task 6 stability, actual-hook miss and fast-prototype evidence](../../tasks/files/material-cd7782/material-final-latency-evidence.md) records the proposed amendment's measured basis. The owner accepted the amendment; all Task 6 runtime and latency gates passed.

## Verified execution

Merged revision94d3585a includes current main's capture-hold additions. All398
cases retain native/fast parity and both environments' required stability sets.
Qualifying actual hook medians: fast24.818s/full40.752s.
Actual verifier: exit0, six qualifying runs, median32.447s, limit45s; zero widened
or contended qualifying runs. The initial same-second sample remains excluded.
[Verified evidence](../../tasks/files/material-9f9ca2/material-verified-latency-evidence.md)
contains the complete completion receipts and registry workaround. No shared host
pointers changed, and no push, PR or live desktop use was performed.
