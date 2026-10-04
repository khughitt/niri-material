# Pre-commit tooling latency

Task: material-cd7782. Status: implemented; consecutive concurrency stability and post-remedy latency acceptance remain to verify.

## Outcome

Target a successful, warm fast pre-commit median **≤ 35 seconds**, leaving at least
10 seconds below the existing 45-second incident limit. Preserve lifecycle coverage
at commit time for changes to its subjects and tests, and in full validation and
CI for every change. Require a successful, warm **full-route median ≤ 45 seconds**
as well, aiming for the same 35-second headroom target. Both routes remain recorded
under `hook-pre-commit`, so their mix must not restore the latency incident.
Preserve production waits and the intended exit-status, failure-bound, and cleanup
guarantees; correcting a production trap defect is explicitly in scope.

## Evidence before selecting the design

[Probe receipts and reproduction inputs](../../tasks/files/material-cd7782/material-pre-commit-revision-evidence.md)
retain the sanitized timing records, installed versions, test verdicts, and
disposable scripts behind these measurements.

The incident's seven-day median is 45.305 seconds over 140 qualifying hooks. The
successful-hook history supplied in review shows a sustained regression:

| Day | Median seconds | Maximum seconds |
| --- | ---: | ---: |
| September 27–October 1 | 24–28 | 47–88 |
| October 2 | 64 | 115 |
| October 3 | 109 | 305 |
| October 4 | 247 | 299 |

A baseline through the Python `just test-fast` override and standard profiler
passed 287 discovered cases in 237.311 seconds, with two optional retained-binary
skips. The class costs below are cumulative test-body times; setup, teardown,
other modules, and runner overhead belong to the last row.

| Work | Cases | Seconds | Fast subset |
| --- | ---: | ---: | --- |
| `test_optic_settling.DriverCleanupTests` | 21 | 185.293 | Skipped |
| Other `test_optic_settling` test bodies | 35 | 1.064 | Included |
| `test_vt_lib.VtLibTests` | 11 | 23.181 | Skipped |
| Other test bodies, setup, teardown, and overhead | 220 | 27.773 | Mostly included |

The entire optic-settling module's test bodies cost 186.357 seconds; that is not
the amount removed. The remainder includes the selected classes' setup/cleanup,
which fast mode also omits (their setup bodies cost 0.221 seconds). A corrected
discovery probe skipped exactly the two classes
above: 287-case inventory, 32 selected skips plus two existing optional skips,
26.700 seconds, exit zero. Early faulty probes were discarded.

Warm Rust stage measurements used the main checkout's own hydrated target; its
Rust and tooling sources match the task worktree. Formatting took 1.234–1.260
seconds in isolated probes. The first clippy run refreshed metadata in 9.113
seconds; subsequent runs took 0.307 and 0.313 seconds. Cold/refresh costs must be
retained separately from steady-state acceptance evidence.

Three complete candidate fast-hook probes then ran through `just hook-pre-commit`
with temporary command overrides. They executed the real report staging, hygiene,
target isolation, formatting, clippy, task/report/pin checks, and the temporary
fast discovery probe, under the existing timing wrapper and host budget. A private
Git index isolated report staging. The timing log was private too: these diagnostic
samples are not post-remedy incident-verification runs.

| Stage, seconds | Probe 1 | Probe 2 | Probe 3 |
| --- | ---: | ---: | ---: |
| Stage divergence report | 0.226 | 0.186 | 0.409 |
| Hygiene | 1.753 | 1.681 | 2.296 |
| Target isolation | 0.275 | 0.308 | 0.296 |
| Format | 1.174 | 1.158 | 2.503 |
| Clippy | 0.374 | 0.700 | 0.987 |
| Fast tooling | 26.094 | 27.628 | 26.573 |
| Task consistency | 0.032 | 0.064 | 0.032 |
| Report freshness | 0.196 | 0.378 | 0.197 |
| Package pin | 0.053 | 0.127 | 0.048 |
| **Whole wrapped hook** | **30.986** | **32.972** | **34.424** |

The whole-hook median is **32.972 seconds**, supporting the 35-second target
before implementation. Wrapped totals include launcher overhead. These are warm
prototypes, not a guarantee for cold builds or a substitute for verification in
the worktree. Normal desktop activity and some concurrent package fetching were
present; only the latency verifier decides which final runs qualify.

A separate broad sleep-compression experiment reduced a screencast test from
46.755 to 13.623 seconds but changed a dead-consumer failure from “exited” to
“never became ready.” It failed and its approach is rejected.

A recount of non-merge commits reachable from `materials-26.04`, from September 28
00:00 UTC through `ca634f24` on October 4, reproduces the review's 125 code commits.
The existing docs-only allowlist excludes another 128 commits from this table;
daily counts use local committer dates. Commit counts approximate hook runs, since
amends and failed attempts are absent.

| Full-route patterns | Full | Fast | Full share | October 2 full / fast |
| --- | ---: | ---: | ---: | ---: |
| Previous broad prefixes | 56 | 69 | 44.8% | 25 / 14 |
| Review's lifecycle subjects | 34 | 91 | 27.2% | 20 / 19 |
| Subjects plus sourced `glass-optic-smoke-lib.sh` | 36 | 89 | 28.8% | 20 / 19 |

The driver sources that shared helper before running any case; excluding it would
leave real lifecycle behavior uncovered. Its inclusion adds two full-route commits
(`146d56b3`, `264c4155`). The 51.3% full share on October 2 means narrowing alone
cannot protect the rolling median while full hooks take about four minutes.
Parallel full validation is therefore required. Its speed is an implementation
acceptance check, not an already measured result; this revision adds only the
route recount to the existing timing evidence.

## Selection and fast-mode contract

Use unittest's class-level skip on `DriverCleanupTests` and `VtLibTests`, conditional
on `NIRI_TOOLING_FAST=1`. Parse the variable before applying either decorator:
**unset or `0` means full; `1` means fast; every other value, including an empty
string, raises with the variable name and accepted values.** A small shared parser
keeps the two classes' contract identical. Native unittest discovery stays intact.

Raw `unittest discover` inherits the caller's environment: an exported
`NIRI_TOOLING_FAST=1` intentionally skips these classes. Document this caveat and
identify `NIRI_TOOLING_FAST=0` / full validation in the skip reason. Repository fast
and full commands always set `1` and `0` explicitly; a shell export cannot weaken
the full routes. Focused Python commands should explicitly use `0` when lifecycle
coverage is intended.

Extend `.githooks/pre-commit`'s existing staged-path classification. Choose the
first applicable route:

1. **Full tooling:** any staged path matches the following narrow list:
   `tools/optic_settling.py`, `tools/screencast_consumer.py`,
   `tools/fake_screencast.py`, `tools/test_optic_settling.py`,
   `tools/test_screencast_consumer.py`, `tools/test_vt_lib.py`,
   `docs/materials/scripts/vt-lib.sh`,
   `docs/materials/scripts/optic-settling-smoke.sh`,
   `docs/materials/scripts/glass-optic-smoke-lib.sh`,
   `docs/materials/scripts/*-client.c`, `.githooks/*`, `justfile`,
   or `.github/workflows/*`. Include the new shared coordinator/mode helper
   `tools/tooling_tests.py` and its new `tools/test_tooling_tests.py` contract tests
   too; neither has commits in the historical recount.
   Unrelated tools and capture scripts use the fast route.
2. **Docs only:** every staged path matches the existing `docs_paths` allowlist.
3. **Fast tooling:** remaining successfully classified code/documentation changes,
   including Rust-only commits.

Use the existing NUL-delimited, strict UTF-8, `--no-renames` staged-path read so
renames consider both endpoints and deletions count. Empty indexes, invalid names,
failed Git reads, or failed pattern evaluation must never select fast or docs-only:
fall back to full validation, or fail before validation if the justfile itself
cannot load. Mixed docs and subsystem changes select full. Full patterns take
precedence over docs-only patterns.

Add `hook-pre-commit-full` as a recipe if needed for dispatch, but have both code
recipes record the existing timing target **`hook-pre-commit`**. Preserve test counts
and failure propagation. Do not invent an exemption or selection-widening marker.
Narrowing reduces how often lifecycle validation runs; parallel scheduling below
must reduce its cost. Neither change permits excluding slow successful samples.

Add a cheap static gate regression against this same path list, running on the
fast route as well as full validation. The docs-only recipe invokes this check
directly too, so every commit checks the list. Read source, without executing
lifecycle cases or tracing file access:

- Inspect the driver's `. "$HERE/…"` source statements and literal
  `$ROOT/tools/…` references; recursively inspect any further source statements
  in those shell helpers.
- Reject command substitution inside arithmetic expansion in the driver and
  sourced helpers, naming the file and line. Helper tool fallbacks replaced by
  lifecycle stubs (including `CAPTURE_META`) stay outside the contract.
- Inspect repository path literals used by `DriverCleanupTests` and `VtLibTests`,
  including their module-level path definitions (`LIB` in `test_vt_lib.py`).
- Parse covered Python modules with `ast` and follow their `tools.*` imports.

Every discovered repository path must match the full-route list. Report the
source location and missing path, fail on unsupported repository-path construction
or an unresolved `tools.*` import, and test a new helper, a transitive source,
a changed class path and a Python import outside the list. Do not execute shell
source or arbitrary Python expressions to resolve paths, and do not add a separately
maintained import graph. Put the regression in `tools/test_optic_settling.py` outside
the skipped class; the coordinator and shared mode parser live in the covered
`tools/tooling_tests.py`.

## Parallel full validation

Use native unittest discovery and a small standard-library process coordinator;
add no test framework. Partition the discovered suite exactly once: run the
remainder in one process concurrently with individually scheduled cases from
`DriverCleanupTests` and `VtLibTests` in a bounded process pool. Start with a total
limit of ten children, including the remainder. Cap that limit with
`NEXTEST_TEST_THREADS` when set; accept only positive decimal integers and fail
before spawning children on empty, zero, negative or malformed values.
Two whole-class processes are insufficient: the driver class alone costs 185 seconds. Do not append the 26.7-second remainder after slow workers.

Split the four independent scenarios in
`test_screencast_refuses_unchecked_crops_and_a_dead_consumer` into separately named
unittest methods with fresh fixtures. Its 46.755-second serial body exceeds the
hook budget even with other cases running concurrently. Preserve each scenario's
assertions, waits, failure messages, and cleanup; splitting increases the discovered
case count by three. Keep class/module fixtures within their worker lifecycle, and
isolate environment changes, temporary directories, D-Bus and sockets per process.

Force full mode in every child. Collect results and output per worker, then print
one aggregate summary with real test/skip counts and named failures. The CI skip
guard applies before scheduling and to every returned skip; a worker crash or
missing result fails the parent. Verify the partition against discovered test IDs:
no omissions or duplicates. The timing wrapper records one whole full-tooling run.
The coordinator owns cancellation and reaping on EXIT/TERM/INT; test fixtures retain
responsibility for their subprocess cleanup, including failure paths. Validate this
before using the coordinator in hooks and CI.

Keep sequential native discovery available through the explicit full Python
`test-fast` override as a reference. Compare its case IDs and verdicts with parallel
validation. Tune scheduling from implementation timings if the full hook misses
45 seconds; do not shorten sleeps or relax bounds to reach the target.

## Exact command and recipe composition

Keep `test_cmd` **Rust only** (`cargo test --all --exclude niri-visual-tests`).
Define `tooling_fast_cmd` as standard discovery with explicit fast mode `1`, and
`tooling_full_cmd` as parallel native discovery with explicit full mode `0`. `check_cmd`
runs the existing hygiene/target/format/clippy checks, fast tooling, then the
existing task/report/pin checks. `full_check_cmd` substitutes full tooling at that
same position. Share the surrounding command fragments to avoid divergent checks.

| Recipe / entry point | Composition | Tooling runs |
| --- | --- | --- |
| `check` | `check_cmd` | Fast once |
| `test` | Rust-only `test_cmd -- --quiet` | None |
| `gate` | Full check followed by `test` | Full once |
| `hook-pre-commit` | Stage report, then `check_cmd` | Fast once |
| `hook-pre-commit-full` | Stage report, then `full_check_cmd` | Full once |
| `hook-pre-commit-docs` | Stage report, static path check, then existing docs checks | None |
| `hook-pre-push-fast` (CI-covered origin refs) | `check_cmd`, then config/IPC nextest | Fast once |
| `hook-pre-push` (other or unclassified pushes) | `full_check_cmd`, then Rust-only `test_cmd -- --quiet` | Full once |
| `ci-test` | Rust-only `test_cmd -- --quiet` | None |
| `ci-test-release` | Rust-only `test_cmd --release -- --quiet` | None |
| New `ci-tooling-test` | `tooling_full_cmd` with CI skip guard | Full once |
| Python `test-one` override | Explicit full mode and requested unittest names | Focused |
| Python `test-fast` override | Explicit full mode and standard discovery | Full once |

`gate` can depend on a new host `check-full` recipe followed by `test`, replacing
its current `check test` prerequisites. `check-full` records its actual work through
`tools/tt` and `host-budget`; all existing host front doors keep those wrappers.
CI's tooling recipe records through `tools/tt` without requiring host-budget.
Adding tooling to `test_cmd` is rejected: it would duplicate the CI tooling job,
add media dependencies to Rust jobs, and change the standalone Rust test budget.

The gate fixture stubs `host-budget run --` and forwards the remaining argv when
it exercises host recipes. It checks argument forwarding and timing records; it
must not require a host-specific ops installation on CI.

## CI dependency and skip contract

Add a separate tooling job on the existing workflow triggers, using the workflow's
Ubuntu 24.04 runner. Rust test jobs retain their existing commands and dependencies.
The tooling job installs the dependency list established by the linked clean-container
evidence, plus the validated just and Rust toolchain versions. It does not
need a real compositor, GPU, live desktop, retained niri binary, or ops installation.

The full coordinator composes native unittest discovery and `TextTestRunner` with
a CI skip guard. Before running, reject any class/method dependency skip outside the
explicit optional allowlist; after running, reject unexpected dynamic skips too.
Print test IDs and reasons, and preserve failures/errors as nonzero exits. This
keeps missing D-Bus, GStreamer, or PyGObject from silently passing. An import error
also fails. CI forces full mode, so fast-tier skips are never allowed.

The optional allowlist contains these five existing cases (three require `magick`,
two require `MATERIAL_RETAINED_NIRI`):

- `test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_old_ring_replaces_inherited_default_response_with_retained_candidate`
- `test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_remaining_within_configs_validate_with_retained_candidate`
- `test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_positive_face_gate_requires_more_than_one_code`
- `test_glass_optic_smoke.RenderOrderBehindMatrixTest.test_signed_transfer_helpers_reject_black_clipped_and_wrong_solid_output`
- `test_glass_render_order_metrics.RenderOrderMetricsTest.test_cli_distinguishes_failed_gate_from_invalid_capture`

Use identities rather than a blanket count: a missing required class cannot replace
one optional skip and still pass. Optional cases may execute if their dependencies
are present. A clean runner without either optional dependency should report five
skips. Newly introduced optional skips require an explicit allowlist decision.

Use just **1.58.0** through the existing setup-just action and Cargo **1.99.0**
through the existing Rust toolchain action as the supported version floors for
this job. These are the validated versions, not a claim to have found the earliest
working releases. Ubuntu's packaged just 1.21 and Cargo 1.75 do not satisfy the
justfile and Cargo-isolation checks. Gate fixtures stub and forward
`host-budget run --`, so CI requires no ops installation.

The confirmed Ubuntu 24.04 runtime packages are:

```text
python3 git jq procps dbus python3-gi
gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0
gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire
```

`ca-certificates` and `xz-utils` support standalone archive bootstrapping; setup
actions must satisfy their own transport requirements. The validated base versions
are Python 3.12.3, PyGObject 3.48.2, GStreamer 1.24.2 and Git 2.43.0.
The consumer checks for the `pipewiresrc` factory even with its test pipeline
override: assert `gst-inspect-1.0 pipewiresrc` succeeds before discovery.

Ubuntu's GLib 2.80 lacks `register_object_with_closures2` (introduced in 2.84).
Change the fake service's single registration call to the established binding
`register_object` API, without a runtime fallback or version branch.
[GIO documents the binding's availability and deprecation](https://docs.gtk.org/gio/method.DBusConnection.register_object_with_closures.html).
Verify consumer startup/failure cases on Ubuntu and the newer incident host.

In `tools/upstream-report`, `merge-tree` exit 0 or 1 must also contain a nonempty
result-tree field. Otherwise raise `ReportError`, retaining stderr. Git 2.43 can
return exit 1 with empty stdout for an invalid ref; accepting it as a conflict is
incorrect. Preserve status-1 directory conflicts with a valid tree and no
conflicted-file entries. Add a deterministic empty-output regression independent
of Git version, and retain the existing conflict tests.

**Shipping blocker:** the clean full suite still fails intermittently in
`test_optic_settling.DriverCleanupTests.test_term_with_the_session_locked_reaps_the_lock_client`:
Bash reports `trap: unexpected EOF while looking for matching ')'`, and the driver
exits 2 instead of the required 143. Diagnose this before enabling the CI job or
fast omission. A real defect in the production driver's traps or cleanup is in
scope; the fix must retain the exit-status, failure-bound and cleanup guarantees.
A minimal reproduction using the actual `stim()` body now isolates TERM during
its arithmetic expression with two command substitutions: Ubuntu Bash 5.2.21
reproduces the EOF error, while splitting the substitutions exits 143 cleanly on
both Bash versions. Ten repetitions confirmed this mechanism; the real lifecycle
case and concurrent suite still require validation. The evidence retains receipts. Preserve waits and assertions; do not skip the case, widen its bound, or
retry until green and call it resolved. A passing rerun alone does not discharge
the blocker. Record reproduction and disposition in the plan and evidence.

The linked evidence retains pilot/full-run outcomes, negative controls, image and
archive digests, package versions and API/Git version investigation. Those probes
establish the runtime contract; they do not establish a passing CI suite or the
parallel route's performance.

## Alternatives and documentation

Path-triggered full coverage is selected over unconditional fast hooks because
lifecycle subjects need their tests before committing. Narrow subjects with a
read-coverage gate replace broad prefixes; measured churn makes broad routing too
frequent. Parallel case processes are selected over serial full validation and
whole-class workers because the full route threatens the aggregate rolling median.
Fast-route timing alone cannot justify deferring full-route scheduling.

Do not compress production/stub waits or raise the latency budget. Update these
specific files during implementation: the `check_cmd` comment in `justfile`, the
“Seconds only” header and classification comments in `.githooks/pre-commit`, and
this repository's `AGENTS.md` gate and Python-testing guidance. Mention the full
routes and raw-discovery environment caveat.

## Verification and completion

1. Through `just test-one`, verify the portable fake service on the incident host
   and Ubuntu, and report error classification on both Git versions, then verify
   staged selection for subjects, tests, helpers, workflow/hook/justfile changes, mixed paths, deletions, both rename endpoints,
   empty indexes, invalid UTF-8, and partial/failed pattern or Git reads. Prove fast
   selects exactly the two classes; unset/`0` selects neither; invalid flags fail.
   Prove full routes override exported `1`, run tooling once, propagate failure,
   and record both code-hook routes as `hook-pre-commit`. Check unrelated tools and
   capture scripts take fast; the sourced shared helper takes full. Verify the
   static coverage gate rejects unlisted direct/transitive shell sources, class
   paths and `tools.*` imports during fast validation, without running slow cases.
2. Run full Python validation through the existing `just test-fast` override with
   explicit `NIRI_TOOLING_FAST=0`. Preserve the assertions of all 287 existing
   cases; only the two retained-binary skips are expected on the incident host.
   Added contract tests
   and the three additional scenario methods increase the count. Compare sequential
   and parallel discovery IDs/results, including CI skip handling, worker failures
   and cancellation cleanup. Run the required affected Rust `just test-fast` gate.
3. Reproduce the CI recipe in clean Ubuntu without host-budget. Run a representative
   pilot before full discovery, inspect skip IDs, and show both a missing mandatory
   dependency and a deliberately failing lifecycle case produce nonzero verdicts.
   Guard tests must also reject unexpected dynamic skips. After the TERM blocker
   is fixed, require consecutive green parallel full runs on both the incident host
   and clean Ubuntu; the plan defines the count and isolated/concurrent stress loops.
   No GitHub write is needed.
4. Warm the task worktree's **own** Rust artifacts through the normal check front
   door, retain cold/refresh timings, and record the remedy timestamp only after the
   implementation is in place. Run actual hook recipes without diagnostic overrides.
   Implementation commits touching `justfile` or `.githooks/` take the full route.
   Before the coordinator is available they cost approximately 240 seconds; after
   it is available, retain them as early full-route performance data. Both precede
   the remedy timestamp and do not qualify for post-remedy verification.
   Validate both staged routes, including complete stage timings; fast must have a
   median ≤ 35 seconds, full ≤ 45 seconds (aim ≤ 35). Parallel speed must be measured
   here before accepting the remedy, rather than inferred from the serial profile.
5. `tt-latency verify material-cd7782 --after <remedy timestamp>` needs at least
   **three successful, uncontended, unwidened runs starting strictly after that
   timestamp**, with a median **≤ 45 seconds**, on every host in the breach notes.
   Obtain at least three qualifying runs of each code route so a fast-only sample
   cannot conceal slow full validation; retain the aggregate verifier output too.
   Confirm the fast route additionally meets the ≤ 35-second design target. Schedule
   sequential runs when competing load will not invalidate them; busy or widened
   samples do not count. Read the verifier's actual count and exclusions before
   retrying. Today's post-now dry run reports `runs: 0, outstanding here`, as expected.
   Close only after verification exits zero everywhere, and include its output in
   `tasks done`. If the target or incident check fails, keep the task open and revise
   from the measured stage costs.

The static-check substitution satisfies the owner's conditional spec acceptance.
The implementation plan uses the minimal TERM reproduction, and retains real-case
isolated/concurrent stress as a shipping gate. Implementation begins after the
plan's separate owner review, as required by the repository process.
