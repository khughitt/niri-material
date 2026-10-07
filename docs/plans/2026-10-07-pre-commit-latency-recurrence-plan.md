# Pre-commit latency recurrence implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring both pre-commit routes back under their targets (fast ≤ 30 s, full ≤ 35 s warm) so `hook-pre-commit` clears its 45 s limit, without omitting tests or relaxing bounds.

**Architecture:** One module-bucket partition (`module_buckets`) feeds both routes of the tooling coordinator. The fast route runs `min(4, limit)` buckets. The full route splits its non-lifecycle remainder into the same `min(4, limit)` buckets, submitted before the one-per-process lifecycle cases. An explicit `NEXTEST_TEST_THREADS` grant may now reach 16 workers; unset stays 10.

**Tech Stack:** Python 3 standard library (`unittest`, `concurrent.futures`), just, `tools/tt`, `host-budget`, `tt-latency`.

**Spec:** `docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md`

## Global Constraints

- Fast route: `min(4, limit)` children, whole modules per child.
- Full route: remainder in `min(4, limit)` module buckets, submitted first; each `DriverCleanupTests`/`VtLibTests` case in its own process.
- Pool ceiling: an explicit `NEXTEST_TEST_THREADS` grant is capped at 16; unset gives 10. Malformed values still fail before discovery.
- Budget `1` means one child at a time: the remainder in one process, then each lifecycle case in its own sequential process.
- Each step child is started before its work and closed with `tasks done` in the commit that lands it, its record staged with the code. The parent closes last, after the acceptance child and the final branch review.
- No test is omitted, no sleep shortened, no time bound relaxed, and the 45 s limit is not raised.
- Acceptance (warm, idle host): median of three `just check-full` ≤ 35 s (required ≤ 45 s); median of three `just check` ≤ 30 s; three real hook runs per code route through `.githooks/pre-commit` on staged changes (fast each < 30 s, full each < 35 s), plus Task 2's own full-route commit hook < 35 s; `tt-latency verify material-5094f2 --after <remedy commit time> --dry-run` exits 0 during acceptance and corrections, and the one recording `tt-latency verify` runs after the final review passes, immediately before parent closure, with the latest remedy timestamp. A recorded verification marks the target satisfied and later runs skip it, even with a newer `--after`.

## Review Focus

- A remainder module that shares state through module or class fixtures: its cases must stay in one process. Pinned by the full-route test's per-module PID check (Task 2).
- A lifecycle class living in a module with no remainder cases: it must not create an empty bucket or a duplicate run. Pinned by the full-route fixture, whose lifecycle module holds only `DriverCleanupTests` (Task 2).
- Budget `1` on the full route: the remainder must still run in one process and each lifecycle case in its own, never in the remainder's process. Pinned by the `('1', 1, 1)` subtest (Task 2).
- A grant between 10 and 16 (a partly loaded host): the grant itself wins. Pinned by the `('12', 12)` row of the budget test (Task 2).
- Fewer modules than buckets (a narrow fixture or a future small suite): no empty job is launched. Pinned by `module_buckets` dropping empty buckets, exercised by the existing two-module full-route test (Task 1).

---

## File structure

- `tools/tooling_tests.py`: the coordinator. Gains `MODULE_BUCKETS` and `module_buckets()`; `worker_limit()` gains the 16 ceiling; `run_tooling()` uses the shared partition on both routes.
- `tools/test_tooling_tests.py`: coordinator tests. The budget test, the fast fixture test, and one new full-route test.
- `AGENTS.md`: the Gates paragraph names the new counts.

Run focused tooling tests with:

```bash
just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one <dotted test name>
```

Commits touching `tools/tooling_tests.py` or `tools/test_tooling_tests.py` take the full pre-commit route; that is expected.

### Task 1: Shared module partition and four fast children

**Files:**
- Modify: `tools/tooling_tests.py` (`worker_limit` neighbourhood for the new constant and helper; `run_tooling` fast branch around lines 202–216)
- Modify: `tools/test_tooling_tests.py:90-147` (`test_fast_preserves_native_inventory_and_module_fixtures`)
- Modify: `AGENTS.md` (Gates, the "Fast tooling uses … with two children" sentence)

Start: `tasks start material-8a5af7`.

**Interfaces:**
- Produces: `MODULE_BUCKETS: int = 4`; `module_buckets(cases: Sequence[unittest.TestCase], count: int) -> list[list[str]]` — whole modules per bucket, largest module first into the emptiest bucket, empty buckets dropped, case IDs in discovery order within a module.

- [ ] **Step 1: Write the failing test.** In `test_fast_preserves_native_inventory_and_module_fixtures`, add two more nonempty modules after the `test_beta.py` fixture, and update the counts:

```python
        self.fixture(body, 'test_gamma.py')
        self.fixture(body, 'test_delta.py')
```

Change `'Ran 6 tests'` to `'Ran 10 tests'`. Extend `expected` with:

```python
            'test_gamma.Cases.test_one', 'test_gamma.Cases.test_two',
            'test_delta.Cases.test_one', 'test_delta.Cases.test_two',
```

Change the budget loop to `for budget, children in (('10', 4), ('1', 1)):`, the inventory line to `f'Fast tooling: 10 cases, {children} children'`, and `self.assertEqual(len(ids), 6)` to `self.assertEqual(len(ids), 10)`. Keep every other assertion: the native-inventory comparison, `OK (skipped=1, expected failures=1)`, one PID per module for `.seen`, `.class` and `.module`, and `len(pids) == children`.

- [ ] **Step 2: Run it and see it fail.**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests.CoordinatorTests.test_fast_preserves_native_inventory_and_module_fixtures`
Expected: FAIL in subtest `budget='10'` on `'Fast tooling: 10 cases, 4 children'` (the output says 2 children).

- [ ] **Step 3: Implement.** In `tools/tooling_tests.py`, after `worker_limit()`:

```python
MODULE_BUCKETS = 4


def module_buckets(cases: Sequence[unittest.TestCase], count: int) -> list[list[str]]:
    """Whole modules per bucket, largest first into the emptiest; no empty buckets."""
    modules = {}
    for case in cases:
        modules.setdefault(type(case).__module__, []).append(case.id())
    buckets = [[] for _ in range(count)]
    for ids in sorted(modules.values(), key=len, reverse=True):
        min(buckets, key=len).extend(ids)
    return [bucket for bucket in buckets if bucket]
```

In `run_tooling`, replace

```python
    if fast:
        limit = min(2, limit)
```

with

```python
    buckets = min(MODULE_BUCKETS, limit)
    if fast:
        limit = buckets
```

and replace the fast partition block

```python
    if fast:
        modules = {}
        for case in inventory:
            modules.setdefault(type(case).__module__, []).append(case.id())
        buckets = [[] for _ in range(limit)]
        for ids in sorted(modules.values(), key=len, reverse=True):
            min(buckets, key=len).extend(ids)
        jobs = [bucket for bucket in buckets if bucket]
```

with

```python
    if fast:
        jobs = module_buckets(inventory, buckets)
```

Leave the full branch unchanged in this task.

In `AGENTS.md`, Gates, change "Fast tooling uses `python3 -m tools.tooling_tests --fast` with two children and keeps whole modules in one worker." to "Fast tooling uses `python3 -m tools.tooling_tests --fast` with four children and keeps whole modules in one worker."

- [ ] **Step 4: Run it and see it pass, with its neighbours.**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests`
Expected: OK. The existing full-route test (`test_full_overrides_fast_and_budget_one_preserves_all_ids`) still passes unchanged.

- [ ] **Step 5: Run the fast route whole.**

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=1 python3 -m tools.tooling_tests --fast' test-fast`
Expected: `Fast tooling: <n> cases, 4 children`, then `OK`. Note the `Ran … in` seconds (spec probe: about 11.4 s).

- [ ] **Step 6: Commit.**

```bash
tasks done material-8a5af7 "fast tooling runs four module buckets through module_buckets(); fixture has four modules"
tasks check
git add tools/tooling_tests.py tools/test_tooling_tests.py AGENTS.md tasks/material-8a5af7.md
git commit -m "perf(tools): run fast tooling in four module buckets (material-8a5af7)"
```

### Task 2: Bucketed full remainder and the 16-worker grant ceiling

**Files:**
- Modify: `tools/tooling_tests.py` (`worker_limit`; `run_tooling` full branch)
- Modify: `tools/test_tooling_tests.py:30-39` (`test_budget_caps_total_children_and_rejects_malformed_values`)
- Create test: `CoordinatorTests.test_full_buckets_remainder_modules_and_isolates_lifecycle_cases` in `tools/test_tooling_tests.py`, after `test_full_overrides_fast_and_budget_one_preserves_all_ids`
- Modify: `AGENTS.md` (Gates, "Full tooling defaults to ten children total")

Start: `tasks start material-4c455e`.

**Interfaces:**
- Consumes: `MODULE_BUCKETS`, `module_buckets(cases, count)` from Task 1; the local `buckets = min(MODULE_BUCKETS, limit)` in `run_tooling`.
- Produces: `worker_limit()` returns 10 when `NEXTEST_TEST_THREADS` is unset, else `min(16, grant)`.

- [ ] **Step 1: Write the failing tests.** Budget rows become:

```python
        for value, expected in ((None, 10), ('1', 1), ('10', 10), ('12', 12), ('16', 16), ('999', 16)):
```

New test in `CoordinatorTests`:

```python
    def test_full_buckets_remainder_modules_and_isolates_lifecycle_cases(self):
        cases = '''    def record(self):
        with open(self.id() + '.pid', 'a') as out:
            out.write(os.environ['NIRI_TOOLING_FAST'] + ':' + str(os.getpid()) + '\\n')
    def test_one(self): self.record()
    def test_two(self): self.record()
'''
        modules = ('alpha', 'beta', 'gamma', 'delta')
        for name in modules:
            self.fixture('class Cases(unittest.TestCase):\n' + cases, f'test_{name}.py')
        self.fixture('class DriverCleanupTests(unittest.TestCase):\n' + cases, 'test_life.py')
        lifecycle = ['test_life.DriverCleanupTests.test_one', 'test_life.DriverCleanupTests.test_two']
        expected = {f'test_{name}.Cases.test_{case}' for name in modules for case in ('one', 'two')}
        expected.update(lifecycle)
        for budget, children, remainder_processes in (('10', 10, 4), ('1', 1, 1)):
            with self.subTest(budget=budget):
                for path in self.root.glob('*.pid'):
                    path.unlink()
                run = self.run_suite(NEXTEST_TEST_THREADS=budget)
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn(f'Full tooling: 10 cases, {children} children', run.stdout)
                self.assertIn('Ran 10 tests', run.stdout)
                runs = {path.name.removesuffix('.pid'): path.read_text().splitlines()
                        for path in self.root.glob('*.pid')}
                self.assertEqual(set(runs), expected)
                self.assertTrue(all(len(lines) == 1 for lines in runs.values()), runs)
                pid = {case_id: lines[0].split(':')[1] for case_id, lines in runs.items()}
                self.assertEqual({lines[0].split(':')[0] for lines in runs.values()}, {'0'})
                for name in modules:
                    self.assertEqual(pid[f'test_{name}.Cases.test_one'], pid[f'test_{name}.Cases.test_two'])
                remainder = {pid[case_id] for case_id in expected if '.Cases.' in case_id}
                self.assertEqual(len(remainder), remainder_processes)
                self.assertEqual(len({pid[case_id] for case_id in lifecycle}), 2)
                self.assertFalse(remainder & {pid[case_id] for case_id in lifecycle})
```

(`self.command` in `CoordinatorTests` is already `--full --ci`; `fixture()` already prefixes `import unittest, os, subprocess, time`.)

- [ ] **Step 2: Run them and see them fail.**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests.ModeTests.test_budget_caps_total_children_and_rejects_malformed_values tools.test_tooling_tests.CoordinatorTests.test_full_buckets_remainder_modules_and_isolates_lifecycle_cases`
Expected: the budget test fails at `('12', 12)` (gets 10); the new test fails in subtest `budget='10'` with `1 != 4` on the remainder process count.

- [ ] **Step 3: Implement.** In `worker_limit()`, keep the unset default and raise the grant ceiling:

```python
def worker_limit() -> int:
    # Unset (CI, raw runs) keeps 10; a host-budget grant may use up to 16.
    raw = os.environ.get('NEXTEST_TEST_THREADS')
    if raw is None:
        return 10
    if re.fullmatch(r'[0-9]+', raw) is None or int(raw) < 1:
        raise ValueError('NEXTEST_TEST_THREADS must be a positive decimal integer')
    return min(16, int(raw))
```

In `run_tooling`, replace the full branch

```python
    else:
        lifecycle = [case.id() for case in inventory if type(case).__name__ in LIFECYCLE_CLASSES]
        remainder = [case.id() for case in inventory if type(case).__name__ not in LIFECYCLE_CLASSES]
        jobs = ([remainder] if remainder else []) + [[case_id] for case_id in lifecycle]
```

with

```python
    else:
        # Remainder buckets go first so they never queue behind the slow lifecycle cases.
        lifecycle = [case.id() for case in inventory if type(case).__name__ in LIFECYCLE_CLASSES]
        remainder = [case for case in inventory if type(case).__name__ not in LIFECYCLE_CLASSES]
        jobs = module_buckets(remainder, buckets) + [[case_id] for case_id in lifecycle]
```

In `AGENTS.md`, Gates, change "Full tooling defaults to ten children total;" to "Full tooling splits the non-lifecycle remainder into four module buckets beside one process per lifecycle case, with ten children total by default and up to sixteen under a granted budget;". Keep the rest of that sentence ("both routes are capped by `NEXTEST_TEST_THREADS` when set; …").

- [ ] **Step 4: Run the coordinator tests.**

Run: `just --set one_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest' test-one tools.test_tooling_tests`
Expected: OK, including both subtests of the new test and the unchanged `test_full_overrides_fast_and_budget_one_preserves_all_ids`.

- [ ] **Step 5: Run full tooling, then the sequential reference.**

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full' test-fast`
Expected: `Full tooling: <n> cases, 16 children` on an idle host, `OK (skipped=2)`, `Ran <n> tests in` about 23 s.

Run: `just --set fast_cmd 'env NIRI_TOOLING_FAST=0 python3 -m unittest discover -s tools 2>&1' test-fast`
Expected: `Ran <n> tests` with the same `<n>` and `OK (skipped=2)`. The coordinator already refuses a partition that loses or duplicates discovered IDs, so equal counts and verdicts complete the comparison.

- [ ] **Step 6: Commit.**

```bash
tasks done material-4c455e "full remainder runs in four module buckets ahead of lifecycle cases; granted budgets reach 16 workers"
tasks check
git add tools/tooling_tests.py tools/test_tooling_tests.py AGENTS.md tasks/material-4c455e.md
git commit -m "perf(tools): bucket the full tooling remainder and allow a 16-worker grant (material-4c455e)"
```

This commit takes the full pre-commit route; its `hook-pre-commit` run is the first real full-route sample. Its UTC commit time is the remedy timestamp for Task 3 (step 4 there shows the command).

### Task 3: Acceptance measurements and latency verification

Start: `tasks start material-bea903`.

**Files:**
- Modify: `tasks/material-bea903.md` and `tasks/material-5094f2.md` through `tasks note` / `tasks done` only.
- Modify: `docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md` (status line, measured results).

**Interfaces:**
- Consumes: Tasks 1 and 2 committed on the branch; the remedy timestamp from Task 2's commit.

- [ ] **Step 1: Check the host is idle.** Run `uptime` and `host-load`. If the load average exceeds 4, or another session runs a build or capture, park with `tasks park material-bea903 "rerun Task 3 measurements, then verify" --reason quiet --waiting-on user --minutes 15`.

- [ ] **Step 2: Measure both routes, warm.** One untimed `just check-full` to warm clippy, then:

```bash
for route in check-full check; do for i in 1 2 3; do
  s=$(date +%s.%N); just $route >/dev/null 2>&1; rc=$?
  echo "$route $i rc=$rc $(python3 -c "print(round($(date +%s.%N)-$s,1))") s"
done; done
```

Expected: every `rc=0`; `check-full` median ≤ 35 s; `check` median ≤ 30 s. A miss stops here: note the numbers and return to the spec's probes before changing anything.

- [ ] **Step 3: Run the real hook on each code route, three times.** As material-cd7782 did, stage through a private index so the working index stays untouched, and run `.githooks/pre-commit` itself with no command overrides; it picks the route from the staged paths and records `hook-pre-commit` through `tools/tt`. A fast-route subject is `tools/upstream-report` (code, not in `tooling_full_paths` or `docs_paths`); a full-route subject is `tools/tooling_tests.py`.

The hook's first step, `python3 tools/upstream-report --stage`, regenerates `docs/materials/upstream-divergence.md` in the working tree from the staged paths, so a private-index run leaves that file changed and the next run refuses the unstaged edit. Each run therefore saves the report first and restores it on every exit, including failure and interruption, and removes its temporary index. Each run happens in a subshell so its trap cannot leak into the session.

```bash
REPORT=docs/materials/upstream-divergence.md
stage_run() (  # $1: path whose staged blob gains a trailing newline
  set -u
  saved=$(mktemp); idx=$(mktemp)
  cp -p "$REPORT" "$saved"
  cleanup() { cp -p "$saved" "$REPORT"; rm -f "$saved" "$idx"; }
  trap cleanup EXIT
  trap 'exit 130' INT TERM
  cp "$(git rev-parse --git-path index)" "$idx"
  mode=$(git ls-files -s -- "$1" | cut -d' ' -f1)
  blob=$( { cat "$1"; echo; } | git hash-object -w --stdin)
  GIT_INDEX_FILE=$idx git update-index --cacheinfo "$mode,$blob,$1"
  s=$(date +%s.%N); GIT_INDEX_FILE=$idx .githooks/pre-commit >/dev/null 2>&1; rc=$?
  echo "$1 rc=$rc $(python3 -c "print(round($(date +%s.%N)-$s,1))") s"
)
before=$(git status --porcelain)
for i in 1 2 3; do stage_run tools/upstream-report; done
for i in 1 2 3; do stage_run tools/tooling_tests.py; done
[ "$(git status --porcelain)" = "$before" ] && echo "working tree unchanged" || git status --short
```

The last line must print `working tree unchanged`; anything else is a cleanup fault to fix before going on.

Confirm the recorded runs and their routes:

```bash
jq -r --arg since "<remedy timestamp>" 'select(.project=="material" and .target=="hook-pre-commit" and .at>=$since) | "\(.at) \(.seconds) exit=\(.exit) \(if (.command|test("--full")) then "full" else "fast" end)"' ~/.local/share/ops/runs.jsonl
```

Expected: every run exits 0; three fast runs each under 30 s; the full runs (three here plus Task 2's commit) each under 35 s. A miss stops here: note the numbers before changing anything.

- [ ] **Step 4: Verify without recording.** The remedy timestamp is Task 2's commit time in UTC:

```bash
TZ=UTC git log -1 --date=format-local:%Y-%m-%dT%H:%M:%SZ --format=%cd <task-2-commit>
tt-latency verify material-5094f2 --after <remedy timestamp> --dry-run
```

Expected: exit 0 once three qualifying runs exist (`[verify] min_runs = 3`). Do not run it without `--dry-run` here: a recorded verification marks the target satisfied, and a later corrective rerun with a newer `--after` would then pass with no new runs.

- [ ] **Step 5: Record and close the acceptance step.** Update the spec's status line to "Implemented (material-5094f2)" and add a "Results" paragraph under §4 with the step 2 medians, the step 3 hook seconds, and the dry-run verify verdict. Then:

```bash
tasks note material-5094f2 "accept: check-full median <x> s, check median <y> s, full hooks <z1>/<z2>/<z3> s (Task 2 commit <z0> s), fast hooks <a>/<b>/<c> s"
tasks done material-bea903 "both routes under target; tt-latency verify --dry-run exit 0"
tasks check
git add docs/specs/2026-10-07-pre-commit-latency-recurrence-design.md tasks/material-bea903.md tasks/material-5094f2.md
git commit -m "docs(specs): record the latency recurrence results (material-bea903)"
```

- [ ] **Step 6: Final branch review and corrections.** Dispatch one fresh reviewer on the most capable model over the whole branch (`git diff materials-26.04...HEAD`), with the spec and this plan. Note the round on the parent: `tasks note material-5094f2 "review: impl round <n> — verdict: <revise|accept>; findings: <label> <count>, … | none; reviewer: <harness/model>"`. While a re-review reproduces Critical or Important findings, run up to five corrective rounds (one fix commit plus one scoped re-review each). A fix that touches `tools/` invalidates step 2 and step 3 timings: rerun both, and rerun step 4 (still `--dry-run`) with the fix's commit time as the new remedy timestamp. The latest remedy timestamp, Task 2's or the last `tools/` fix's, is the one step 7 uses.

- [ ] **Step 7: Record verification and close the parent.** Only after step 6's final review passes, with every child done. This is the first and only recording verification:

```bash
tt-latency verify material-5094f2 --after <latest remedy timestamp>
```

It must exit 0; keep its full output. Then, immediately:

```bash
tasks done material-5094f2 "<full tt-latency verify output>"
tasks check
git add tasks/material-5094f2.md
git commit -m "chore(tasks): close the pre-commit latency recurrence (material-5094f2)"
```

If it does not exit 0, do not close: note the output on the parent and return to step 2.
