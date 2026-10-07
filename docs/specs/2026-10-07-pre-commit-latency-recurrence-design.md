# Pre-commit latency recurrence

Status: draft for owner review (material-5094f2).

## 1. Incident

`tt-latency` filed material-5094f2 at 2026-10-07T03:05:28Z:
`hook-pre-commit` median 45.425 s against the 45 s limit, 37 judged runs on
3 days. The window starts at the floor that material-cd7782 left on
2026-10-05T10:16:51Z. It is that incident again, two days later, by the same
0.4 s margin.

## 2. What the runs show

The hook has three routes (AGENTS.md, Gates). Successful wrapper runs after
the floor, by route:

| Route | Runs | Median | Range |
| --- | ---: | ---: | --- |
| fast tooling | 40 | 37.5 s | 33.8–102.2 s |
| full tooling (lifecycle subjects, gate wiring) | 7 | 106.6 s | 46.3–298.7 s |
| legacy `unittest discover` (before routing landed) | 4 | 242.4 s | 241.7–258.6 s |

The median of the judged set sits on the fast route's upper edge. A handful of
full-route commits in the window decides the verdict: the live check at
03:21Z read `ok` (median 42.256 s, 39 runs) only because the evening's
commits were all fast-route.

The fast route on the idle host (one run per step, 2026-10-06 23:30 local):

| Step | Seconds |
| --- | ---: |
| fast tooling (409 cases, 2 children, 34.9 s of case time) | 24.1 |
| `cargo clippy --all --all-targets` (warm) | 6.4 |
| `tools/ops-check` | 2.2 |
| `cargo fmt --check` | 1.2 |
| upstream-report, target-dir-check, tasks check, package-pin | < 1 together |

The fast route has no cases slow enough to omit: the largest module is
`test_upstream_report` at 6.9 s and the slowest case 2.7 s. The 2-child cap
from material-cd7782 (spec 2026-10-04, §"two native unittest child
processes") was the minimal change that met its 35 s fast target then. The
suite has grown since, and nothing measures the fast target, so it crept back.

Worker-count probe, fast tooling alone, two runs each, idle host:

| Children | Seconds | Failures |
| ---: | ---: | --- |
| 2 | 24.1, 24.0 | none |
| 4 | 11.4, 11.5 | only the test pinning "2 children" |
| 6 | 15.7, 15.7 | only the test pinning "2 children" |

Six children are slower than four: whole-module buckets leave the largest
module as a floor, and each extra child pays interpreter start and discovery.

## 3. Design

**Fast route: four children.** In `tools/tooling_tests.py`, the fast limit
becomes `min(4, limit)`, still capped by `NEXTEST_TEST_THREADS`, and the
module-bucket partition is unchanged. The coordinator test asserts four,
and AGENTS.md's Gates section, which says fast tooling runs "with two
children", says four.
The hook runs under `host-budget run`, which granted `NEXTEST_TEST_THREADS=16`
on the idle host, so the cap of four is what applies; a loaded host's smaller
grant still wins.
Expected fast route on an idle host: about 23 s, leaving about 20 s of
headroom under the 45 s limit.

**Full route: unchanged here.** Its 7 runs are the lifecycle and gate-wiring
commits material-cd7782 sent to the full suite on purpose. With the fast route
near 23 s, the judged median sits on the fast route unless more than half the
window's commits take the full route.

**Rejected alternatives.**
- Raising the 45 s limit: it hides the growth that caused both incidents.
- Omitting more modules from the fast tier: no module is slow, and every
  omission moves coverage to pre-push.
- Six or more children: measured slower than four.

## 4. Verification

1. The coordinator test is updated, and the fast route is checked at the
   one-child budget too: `NEXTEST_TEST_THREADS=1` still runs the same inventory
   sequentially.
2. Three staged commits through the real hook on the idle host, fast route,
   each under 30 s.
3. `tt-latency verify material-5094f2 --after <remedy commit time>` on the host the breach names
   exits 0 once three qualifying runs exist (`[verify] min_runs = 3`). Its
   output goes in the `tasks done` message.

## 5. Out of scope

A standing check that the fast route stays under its target is ops tooling
(`tt-latency` judges one limit per target). It would have caught this creep.
It is filed as feedback to ops, not built here.
