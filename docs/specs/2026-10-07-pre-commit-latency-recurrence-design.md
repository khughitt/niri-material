# Pre-commit latency recurrence

Status: Implemented (material-5094f2).

## 1. Incident

`tt-latency` filed material-5094f2 at 2026-10-07T03:05:28Z:
`hook-pre-commit` median 45.425 s against the 45 s limit, 37 judged runs on
3 days. The window starts at the floor that material-cd7782 left on
2026-10-05T10:16:51Z. It is that incident again, two days later, by the same
0.4 s margin.

material-cd7782's design (2026-10-04) set two targets for this hook: a warm
fast-route median of at most 35 s, and a warm full-route median of at most
45 s, aiming for 35 s. It verified the full route at 40.752 s. Both targets
apply here; neither is measured standing, so both crept.

## 2. What the runs show

The hook has three routes (AGENTS.md, Gates). Replaying the incident's
filtering (successful, uncontended, unwidened runs in the window) over the
timing log splits the 37 judged runs by route:

| Route | Judged runs | Median |
| --- | ---: | ---: |
| fast tooling | 29 | 37.618 s |
| full tooling (lifecycle subjects, gate wiring) | 4 | 89.765 s |
| legacy `unittest discover` (before routing landed) | 4 | 242.398 s |

The 8 slow runs push the window median into the upper part of the fast
route's spread. The legacy runs age out of the window; the full and fast runs
stay, so both routes need to shrink.

### Fast route

On the idle host (one run per step, 2026-10-06 23:30 local):

| Step | Seconds |
| --- | ---: |
| fast tooling (409 cases, 2 children, 34.9 s of case time) | 24.1 |
| `cargo clippy --all --all-targets` (warm) | 6.4 |
| `tools/ops-check` | 2.2 |
| `cargo fmt --check` | 1.2 |
| upstream-report, target-dir-check, tasks check, package-pin | < 1 together |

No case is slow enough to omit: the largest module is `test_upstream_report`
at 6.9 s and the slowest case 2.7 s. The 2-child cap from material-cd7782 was
the minimal change that met 35 s then; the suite has grown since.

Worker-count probe, fast tooling alone, two runs each, idle host:

| Children | Seconds |
| ---: | --- |
| 2 | 24.1, 24.0 |
| 4 | 11.4, 11.5 |
| 6 | 15.7, 15.7 |

Six are slower than four: whole-module buckets leave the largest module as a
floor, and each extra child pays interpreter start and discovery.

### Full route

Warm `just check-full` on the worktree (2026-10-07, load average about 3 on
32 CPUs, Dropbox syncing): 53.0 s cold-clippy, then **46.3 s and 46.6 s**.
The full route is over its 45 s requirement, not only its 35 s aim.

Full tooling is 40.3 s of that. The coordinator runs every non-lifecycle case
(387 cases, 38.8 s of case time) in a single remainder process, beside the 37
`DriverCleanupTests`/`VtLibTests` cases (219.7 s of case time, longest
20.6 s) scheduled one per process. The remainder was 26.7 s in the 10-04
design; at 38.8 s it is now the critical path by itself.

The lifecycle cases mostly wait on their time bounds: a run uses about 95 s of
CPU over 32 s of wall time. Probes, full tooling alone, two runs each,
remainder split into module buckets with the fast route's partition:

| Remainder | Pool | Order | Seconds |
| --- | ---: | --- | --- |
| one process (today) | 10 | — | 40.0, 40.1 |
| 4 module buckets | 10 | buckets first | 31.6, 31.8 |
| 4 module buckets | 10 | lifecycle first | 33.4, 33.8 |
| 6 module buckets | 10 | buckets first | 31.9, 31.7 |
| 4 module buckets | 14 | buckets first | 25.4, 25.4 |
| 4 module buckets | 16 | buckets first | 22.7, 23.5 |

Every probe passed with the same 424 cases. At 16 the floor is the longest
lifecycle case (20.6 s).

## 3. Design

All changes are in `tools/tooling_tests.py`, its tests, and AGENTS.md.

**One module partition.** Factor the fast route's module-bucket partition
(largest module first into the emptiest bucket, whole modules per bucket) into
one function, used by both routes, with **`min(4, limit)` buckets**.

**Fast route: four children.** The fast pool becomes `min(4, limit)`
children, each running one bucket. Expected on an idle host: about 23 s for
the whole hook.

**Full route: bucketed remainder, larger pool.**
- The non-lifecycle remainder is split into `min(4, limit)` module buckets
  instead of one process. Lifecycle cases stay one per process.
- Jobs are submitted buckets first, then lifecycle cases, so the remainder
  never queues behind the slow cases.
- The pool ceiling for an explicit `NEXTEST_TEST_THREADS` grant rises from 10
  to **16**. `host-budget run` granted 16 on the idle host; a loaded host's
  smaller grant still wins. With the variable unset (CI's `ci-tooling-test`,
  a raw run), the default stays **10**, so CI runners keep today's concurrency
  against the lifecycle cases' time bounds.
- Expected on an idle host: full tooling about 23 s, the whole full route
  about 30 s, under the 35 s aim.

**Tests** (`tools/test_tooling_tests.py`):
- `test_budget_caps_total_children_and_rejects_malformed_values`: unset gives
  10, `16` gives 16, `999` gives 16; the malformed values still fail.
- `test_fast_preserves_native_inventory_and_module_fixtures`: the fixture
  grows from two modules to **four nonempty modules**, so module-preserving
  partitioning can launch four workers. It keeps its native-inventory
  comparison, the expected-failure and fast-omitted lifecycle cases, and the
  per-module `setUpModule`/`setUpClass` single-process checks. Budgets `10`
  and `1` expect 4 and 1 children.
- A new full-route coordinator test: a fixture with four remainder modules and
  two lifecycle cases, budget `10`. Every case ID runs exactly once in full
  mode; each remainder module's cases and fixtures share one process; each
  lifecycle case runs in a process of its own; the remainder uses four
  processes. At budget `1` the same IDs run one child at a time: the
  remainder in one process, and each lifecycle case still in a separate
  process of its own, sequentially.

**AGENTS.md, Gates:** fast tooling runs "with two children" becomes four, and
"Full tooling defaults to ten children total" names the 16 ceiling for a
granted budget and the module-bucketed remainder.

**Rejected alternatives.**
- Raising the 45 s limit: it hides the growth that caused both incidents.
- Omitting more modules from the fast tier: no module is slow, and every
  omission moves coverage to pre-push.
- Six or more fast children, or six remainder buckets: measured no faster than
  four.
- Lifecycle cases first: measured slower, since the remainder then queues.
- Raising the unset default to 16: it changes CI's concurrency for no hook
  gain, and the lifecycle cases are time-bounded.

## 4. Verification

Acceptance, on the idle host, warm:
1. `just test-one` on the changed coordinator tests, and the tooling suite in
   both modes, pass. Full tooling's case IDs and verdicts match sequential
   native discovery (the AGENTS.md reference command).
2. **Full route:** median of three `just check-full` runs **≤ 35 s**
   (required ≤ 45 s). At least one real full-route hook commit (the
   implementation commit touches `tools/tooling_tests.py`, which routes full)
   records under 35 s.
3. **Fast route:** median of three `just check` runs ≤ 30 s, and three staged
   fast-route commits through the real hook, each under 30 s.
4. `tt-latency verify material-5094f2 --after <remedy commit time>` on the
   host the breach names, exits 0 once three qualifying runs exist
   (`[verify] min_runs = 3`). Its output goes in the `tasks done` message.
   Items 2 and 3 stand beside it: verify judges the mixed median, which
   fast-only runs can pass while the full route still regresses.

**Results** (2026-10-08, the breach host, idle, from a TTY, warm after one
untimed `check-full`): `just check-full` 27.8/27.6/27.8 s, median 27.8 s;
`just check` 18.3/18.3/18.3 s, median 18.3 s. Staged hook runs through
`.githooks/pre-commit`: fast route 18.3/18.2/18.3 s, full route
27.9/28.0/27.7 s, all exit 0. Task 2's own full-route commit hook took
35.884 s at 11:41Z under the load (5.13, competing browser tests) that
refused the acceptance preflight a minute later. Ruling: item 2's commit
criterion is not met by that commit itself; it is satisfied by the three
idle staged full-route hook runs (27.7–28.0 s) and the miss is waived as
contention, not the change.
`tt-latency verify material-5094f2 --after 2026-10-07T11:41:28Z --dry-run`
exits 0: met, median 27.8 s over 8 runs, limit 45 s. The 8 include one
contended 100.3 s fast-route run from another checkout at pre-remedy
`e93bbf61`.

## 5. Out of scope

A standing check per route (fast ≤ 35 s, full ≤ 45 s) is ops tooling:
`tt-latency` judges one limit per target. It would have caught both creeps.
Filed as ops-6cff48, not built here.
