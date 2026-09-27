# Front door for tests. Inner loop: `just test-fast` (affected-only). Full suite:
# `just test`. Gates: `just check` at pre-commit, `just gate` at pre-push; CI runs
# `ci-test` and `ci-test-release`, the same suite under the targets it always ran.
# The git hooks call `hook-pre-commit` and `hook-pre-push`, which run the very same
# commands under their own target names so the report can price the hooks.
# Every recipe runs through the vendored timing wrapper tools/tt (ops bin/tt).
# The host recipes run their command under `host-budget run`, which sets
# CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS from the host's CPU budget; the ci-
# recipes do not, since CI has no ops tooling.
# Design: ops docs/specs/2026-09-04-test-ci-audit-design.md and
# docs/specs/2026-09-24-host-budget-design.md.

set quiet

tt := "python3 tools/tt"

# The three commands, each written once. Avoid single quotes inside them.
# fast_cmd: nextest on the packages changed against HEAD and their dependents
# (tools/test-affected). test_cmd: what CI's test job runs. check_cmd: what CI's
# rustfmt and clippy jobs run, the selector's own tests, and the task tracker.
fast_cmd := "python3 tools/test-affected"
test_cmd := "cargo test --all --exclude niri-visual-tests"
check_cmd := "python3 tools/ops-check && cargo fmt --all -- --check && cargo clippy --all --all-targets && python3 -m unittest discover -s tools 2>&1 && tasks check && python3 tools/upstream-report --check && python3 tools/package-pin --check"

# Affected-only: the inner loop.
test-fast:
    {{tt}} test-fast -- host-budget run -- sh -c '{{fast_cmd}}'

# The full suite.
test:
    {{tt}} test -- host-budget run -- sh -c '{{test_cmd}}'

# Format, clippy, tooling tests, tasks check.
check:
    {{tt}} check -- host-budget run -- sh -c '{{check_cmd}}'

gate: check test

# What the pre-commit hook runs: `check`'s command under its own hook target.
hook-pre-commit:
    {{tt}} hook-pre-commit -- host-budget run -- sh -c '{{check_cmd}}'

# What the pre-push hook runs: the same commands as `gate`, under one hook target.
hook-pre-push:
    {{tt}} hook-pre-push -- host-budget run -- sh -c '{{check_cmd}} && {{test_cmd}}'

# CI's test job, exactly as before it went through the front door.
ci-test:
    {{tt}} ci-test -- sh -c '{{test_cmd}} -- --nocapture'

# CI's randomized-and-slow job; it sets RUN_SLOW_TESTS and the proptest limits.
ci-test-release:
    {{tt}} ci-test-release -- sh -c '{{test_cmd}} --release'

# Pin the Arch package on a commit (default HEAD) and derive pkgver, the source
# commit, and NIRI_BUILD_COMMIT from it. `check` verifies the three still agree.
package-pin commit="HEAD":
    python3 tools/package-pin {{commit}}

# Regenerate the upstream divergence report. Pass --drift for the drift block
# (needs `git fetch upstream --tags` first), --check to verify staged freshness.
upstream-report *args:
    python3 tools/upstream-report {{args}}
