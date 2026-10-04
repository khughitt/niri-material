# Pre-commit tooling latency

Task: material-cd7782. Status: proposed; owner review required before planning.

## Outcome

Bring the successful, warm pre-commit hook median on the incident host below
the existing 45-second limit while retaining all tooling tests in CI and full
validation. Keep the production capture delays, failure bounds, and cleanup
behavior intact. This design changes when expensive tests run.

## Evidence

The incident records a seven-day median of 45.305 seconds over 140 qualifying
hooks. That rolling median understates the current slowdown: recent successful
hooks take 245–299 seconds, and successful full tooling runs take about 237
seconds.

A baseline run through `just test-fast`, with Python's standard profiler, passed
287 discovered cases in 237.311 seconds, with two existing optional skips.

| Work | Baseline seconds |
| --- | ---: |
| Optic-settling tests | 186.357 |
| VT tests | 23.181 |
| Remaining test bodies, setup, and cleanup | 27.773 |

Hygiene, target isolation, task consistency, report freshness, and package-pin
checks together took 2.223 seconds. Rust checks were not timed in this initial
investigation; the complete hook remains the acceptance measurement.

The expensive tests launch stub processes but preserve real startup, settling,
timeout, and restoration waits. A temporary experiment accelerated shell sleeps
in the slowest screencast test. It reduced the run from 46.755 to 13.623 seconds,
but changed the dead-consumer failure from “exited” to “never became ready.”
That experiment failed and its approach is rejected.

A separate temporary probe retained the complete 287-case inventory but skipped
the 21 `DriverCleanupTests` cases and 11 `VtLibTests` cases. It passed in 26.700
seconds, with 34 skips: those 32 cases plus the two existing optional skips.
Early faulty versions of this probe were discarded; their timings are not evidence.

## Selected design

Use unittest's existing class-level skip mechanism on those two process-test
classes, conditional on `NIRI_TOOLING_FAST=1`. No custom discovery runner or test
dependency is needed. Ordinary unittest discovery, focused runs, and the existing
Python `test-fast` override continue to include all cases by default.

The justfile defines a fast tooling command for `check_cmd` and a full tooling
command that explicitly sets `NIRI_TOOLING_FAST=0`. The fast command retains
analyzer, metadata, metrics, consumer, gate, package-pin, target-isolation, and
upstream-report checks. Its skip messages identify the full-validation route.

| Entry point | Tooling coverage |
| --- | --- |
| `just check`, code pre-commit, CI-covered pre-push | Fast subset |
| Existing Python `test-one` and `test-fast` overrides | Full discovery by default |
| `just test`, non-CI pre-push | Full discovery, once per invocation |
| New `just ci-tooling-test` recipe | Full discovery without host tooling |
| Docs-only pre-commit | Existing consistency checks |

Add a tooling job to the existing CI workflow, using `ci-tooling-test` on the
workflow's existing triggers. Today the CI test recipes run Rust tests only;
the slow tooling tests must have this destination before the commit gate omits
them. Install the ordinary Linux dependencies required for the stub driver and
consumer tests: Python, jq, D-Bus, PyGObject, and GStreamer base tools/bindings.
Existing optional image and retained-binary test conditions remain explicit.
No compositor, GPU, live desktop, host services, or ops installation is needed.

The gate fixture must stub `host-budget` when it invokes a host recipe to inspect
argument forwarding. That test verifies the recipe's arguments and timing record,
so it should not require the real host-budget executable on a CI runner.

Update the current test guidance and justfile comments to describe the split.
Do not change the latency limit, production capture scripts, or the timing wrapper.

## Alternatives

- Compress stub sleeps: rejected because the probe changed the observed failure,
  and indiscriminate compression weakens timing and readiness coverage.
- Raise the budget: rejected because current hooks take several minutes, far beyond
  the incident's small rolling-median breach.
- Add an affected-test selector or parallel runner: deferred; an explicit split
  using unittest is enough to test first and avoids new selection infrastructure.

The tradeoff is deliberate: commits no longer run the 32 process lifecycle cases.
CI and full validation retain those cases with their existing waits and assertions.

## Verification and completion

1. Add focused gate checks proving the fast command selects the intended classes,
   the full command clears fast mode, command failures propagate, and CI calls the
   full tooling recipe. Keep these checks runnable through `just test-one`.
2. Run full tooling validation through the existing Python `just test-fast`
   override. All 287 current cases remain discoverable; only the two existing
   optional skips are expected on the incident host.
3. Exercise the CI tooling recipe without a real host-budget dependency and verify
   that a failing slow case makes it fail. No GitHub write is required.
4. Warm the task worktree's own Rust artifacts through the normal check front
   door, then measure at least three sequential, successful full pre-commit
   recipe runs after the recorded remedy timestamp. Retain cold-build records;
   distinguish setup cost from steady-state hook latency.
5. Run `tt-latency verify material-cd7782 --after <remedy timestamp>` on every host
   named in the incident. Close only after it exits zero, with its output in the
   completion note. If the complete hook still exceeds 45 seconds, keep the task
   open and profile the remaining stages before proposing another change.

Spec acceptance permits writing the implementation plan. Implementation begins
after that plan's separate owner review, as required by the repository process.
