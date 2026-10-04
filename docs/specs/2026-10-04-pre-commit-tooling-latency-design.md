# Pre-commit tooling latency

Task: material-cd7782. Status: revised proposal; owner review required before planning.

## Outcome

Target a successful, warm fast pre-commit median **≤ 35 seconds**, leaving at least
10 seconds below the existing 45-second incident limit. Preserve lifecycle coverage
at commit time for changes to its subjects and tests, and in full validation and
CI for every change. Keep production waits, failure bounds, and cleanup intact.

Commits changing capture/tooling subsystems deliberately retain the expensive
suite. Their timing remains visible under `hook-pre-commit`; the design does not
raise the limit, rename away slow samples, or claim every commit will take 35 seconds.

## Evidence before selecting the design

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

1. **Full tooling:** any staged path matches `tools/*`, `docs/materials/scripts/*`,
   `.githooks/*`, `justfile`, or `.github/workflows/*`. These deliberately broad
   prefixes cover the driver, VT library, analyzer, consumer, tests, imported
   helpers, and gate/CI wiring without a maintained dependency graph.
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
Full subsystem hooks will still cost minutes; sustained subsystem churn can breach
the rolling median again. If that happens, address that evidence in a later remedy
rather than suppressing the measurements. Parallel execution remains a viable
alternative if the reviewed target cannot hold.

## Exact command and recipe composition

Keep `test_cmd` **Rust only** (`cargo test --all --exclude niri-visual-tests`).
Define `tooling_fast_cmd` as standard discovery with explicit fast mode `1`, and
`tooling_full_cmd` as standard discovery with explicit full mode `0`. `check_cmd`
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
| `hook-pre-commit-docs` | Stage report, then unchanged `docs_check_cmd` | None |
| `hook-pre-push-fast` (CI-covered origin refs) | `check_cmd`, then config/IPC nextest | Fast once |
| `hook-pre-push` (other or unclassified pushes) | `full_check_cmd`, then Rust-only `test_cmd -- --quiet` | Full once |
| `ci-test` | Rust-only `test_cmd -- --quiet` | None |
| `ci-test-release` | Rust-only `test_cmd --release -- --quiet` | None |
| New `ci-tooling-test` | Explicit full mode, guarded native discovery | Full once |
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
The tooling job installs the dependency list established by the clean-container
experiment below, plus the validated just and Rust toolchain versions. It does not
need a real compositor, GPU, live desktop, retained niri binary, or ops installation.

A small CI adapter composes native unittest discovery and `TextTestRunner` with a
skip guard. Before running, reject any class/method dependency skip outside the
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

The clean Ubuntu 24.04 pilot exposed two version requirements before the full
run: the distribution's just 1.21 cannot parse `set quiet`, and Cargo 1.75 misses
the parent `build-dir` isolation assertion. Use the existing setup-just action
with the validated 1.58.0 version and the stable Rust toolchain action to supply
Cargo 1.99.0. The clean probe uses official standalone Cargo and no rustc: metadata
needs no compilation, and the pilot proves the isolation assertions still execute.

The consumer fixture also calls `register_object_with_closures2`, introduced in
GLib 2.84; Ubuntu's GLib 2.80 does not provide it. Change the single fixture call
to the established binding `register_object` API, with no runtime fallback or
version branch. [GIO documents its binding and deprecation](https://docs.gtk.org/gio/method.DBusConnection.register_object_with_closures.html):
it remains available on the newer incident host but is deprecated since 2.84.
This is confined to the fake service, and requires the consumer startup/failure
suite to pass on both environments before shipping. Do not skip those cases to
accommodate the older runner.

The consumer checks for the `pipewiresrc` factory before applying its test pipeline
override. Therefore the tooling job also needs `gstreamer1.0-pipewire`, even though
it does not run a PipeWire server or consume real compositor frames. Assert
`gst-inspect-1.0 pipewiresrc` succeeds before discovery, alongside the skip guard.

The clean experiment uses Ubuntu 24.04 image digest
`sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55`.
APT's confirmed runtime list is:

```text
python3 git jq procps dbus python3-gi
gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0
gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-pipewire
```

The standalone archive/bootstrap experiment also installs `ca-certificates` and
`xz-utils`. CI reuses its existing setup actions to supply just and Cargo; those
setup actions' transport requirements must be satisfied independently of Python
suite dependencies. The validated runtime versions are Python 3.12.3, PyGObject
3.48.2, GStreamer 1.24.2, PipeWire plugin 1.0.5, Git 2.43.0, jq 1.7.1, and procps
4.0.4. just is 1.58.0 and Cargo is 1.99.0; the Cargo release archive's SHA-256 was
checked against the official stable manifest.

Prototypes modify only disposable copies: the fake-service registration call,
gate's host-budget stub, and report validation described below. The CI guard is a scratch adapter, invoked through
an existing CI recipe's command override, not a shipped recipe. A six-case pilot
passed in **24.006 seconds with no skips**, covering driver crops, bounded VT
failure, consumer startup/summary, gate arguments/timing, and both Cargo-isolation
positive checks. All six consumer startup/failure cases also passed on the incident
host in **2.539 seconds** with the portable registration call.

Before media installation, the guard rejected eight required consumer/PyGObject
skips with exit 2. This negative control checked the reported skip identities;
failures before unittest startup do not supply dependency-guard evidence. Two
additional single-case controls confirmed nonzero verdicts for a deliberately
failing lifecycle test and an unexpected dynamic skip (the latter otherwise
reports unittest success).

The first complete clean run discovered **287 cases in 193.244 seconds**, with
exactly the five optional skips, zero errors, and one failure: the upstream-report
invalid-ref test. All lifecycle and consumer cases passed. Git 2.43 permits an
invalid-ref `merge-tree` invocation to exit 1 with no result tree; the current
report treats all exit-1 outputs as conflicts. Newer Git on the incident host
returns exit 128 for that input (Git 2.56), so the existing test had not exposed
this there. The Ubuntu diagnostic observed exit 1, empty stdout, and
`merge-tree: no-such-ref - not something we can merge` on stderr.

CI enablement must add an explicit protocol check in `tools/upstream-report`:
a return status of 0 or 1 still requires a nonempty result-tree field. Reject
missing trees with `ReportError` and retain stderr in the diagnostic. Preserve
status-1 directory conflicts with a valid tree and no conflicted-file entries.
This corrects error classification without a Git version branch or upgrade-only
workaround. Add deterministic empty-output regression coverage independent of
Git version, and retain the existing directory/conflict tests.

The report correction passed all **66 report tests in 6.010 seconds**, including
real directory conflicts and the older Git's invalid-ref result. A subsequent
full run again discovered 287 cases and exactly five optional skips, but failed
one locked-session TERM cleanup case in 193.597 seconds: the shell reported
`trap: unexpected EOF while looking for matching ')'` and exited 2 instead of 143.
That same case passed in the previous run. This is an unresolved intermittent
cleanup failure, not a missing dependency; do not skip it, widen its bound, or
retry until green and call it resolved. The guarded front door correctly failed
on this real lifecycle failure.

Implementation must diagnose this intermittent TERM/trap failure on Ubuntu before
enabling the new CI job and fast omission. Preserve the lifecycle assertion, exit-status and
cleanup guarantees, and production waits. A passing rerun alone does not discharge
this prerequisite; record the reproduction and disposition in the implementation
plan and evidence. Three focused repetitions passed in 12.078 seconds, followed
by another complete run of 287 cases in 193.802 seconds: exactly five optional
skips, zero errors, and the same one TERM/trap failure. The suite has **not** passed
in this environment. Its runtime dependencies and full coverage are established;
the remaining failure is an explicit shipping blocker, owned by the agent after
spec review, rather than evidence that CI is ready.

## Alternatives and documentation

Path-triggered full coverage is selected over unconditional fast hooks because
capture and tooling changes need their lifecycle tests before committing. Parallel
processes could reduce waiting time while retaining every case; that is a scheduling
alternative, not test selection. It is deferred because the measured fast hook
already meets the target and staged paths preserve coverage for affected work.

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
   and record both code-hook routes as `hook-pre-commit`.
2. Run full Python validation through the existing `just test-fast` override with
   explicit `NIRI_TOOLING_FAST=0`. Preserve all 287 existing cases; only the two
   retained-binary skips are expected on the incident host. Added contract tests
   increase the count. Run the required affected Rust `just test-fast` gate.
3. Reproduce the CI recipe in clean Ubuntu without host-budget. Run a representative
   pilot before full discovery, inspect skip IDs, and show both a missing mandatory
   dependency and a deliberately failing lifecycle case produce nonzero verdicts.
   Guard tests must also reject unexpected dynamic skips. No GitHub write is needed.
4. Warm the task worktree's **own** Rust artifacts through the normal check front
   door, retain cold/refresh timings, and record the remedy timestamp only after the
   implementation is in place. Run actual hook recipes without diagnostic overrides.
   Validate both staged routes; expected full-route cost is documented, not hidden.
5. `tt-latency verify material-cd7782 --after <remedy timestamp>` needs at least
   **three successful, uncontended, unwidened runs starting strictly after that
   timestamp**, with a median **≤ 45 seconds**, on every host in the breach notes.
   Confirm the fast route additionally meets the ≤ 35-second design target. Schedule
   sequential runs when competing load will not invalidate them; busy or widened
   samples do not count. Read the verifier's actual count and exclusions before
   retrying. Today's post-now dry run reports `runs: 0, outstanding here`, as expected.
   Close only after verification exits zero everywhere, and include its output in
   `tasks done`. If the target or incident check fails, keep the task open and revise
   from the measured stage costs.

After spec acceptance, the agent diagnoses the remaining Ubuntu TERM failure and
writes the implementation plan from that evidence. Implementation begins after
the plan's separate owner review, as required by the repository process.
