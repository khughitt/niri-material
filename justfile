# Front door: `just test-one <nextest args>` while editing; `just test-fast`
# (affected-only) before committing. Docs-only commits take the docs checks.
# CI-covered pushes take check + nextest; other pushes take check + the full suite.
# The git hooks call `hook-pre-commit` and `hook-pre-push`, which run the very same
# commands under their own target names so the report can price the hooks.
# Every recipe runs through the vendored timing wrapper tools/tt (ops bin/tt).
# The host recipes run their command under `host-budget run`, which sets
# CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS from the host's CPU budget; the ci-
# recipes do not, since CI has no ops tooling.
# Design: ops docs/specs/2026-09-04-test-ci-audit-design.md and
# docs/specs/2026-09-24-host-budget-design.md.

set quiet
set positional-arguments

tt := "python3 tools/tt"

# Commands shared by the recipes and hooks. Avoid single quotes inside them.
# fast_cmd: nextest on the packages changed against HEAD and their dependents
# (tools/test-affected). test_cmd: what CI's test job runs. check_cmd: what CI's
# rustfmt and clippy jobs run, the selector's own tests, and the task tracker;
# tools/target-dir-check first, so a worktree sharing a target dir fails before cargo.
nextest_cmd := "cargo nextest run --status-level fail --final-status-level fail"
fast_cmd := "python3 tools/test-affected --status-level fail --final-status-level fail"
one_cmd := nextest_cmd
test_cmd := "cargo test --all --exclude niri-visual-tests"
check_cmd := "python3 tools/ops-check && python3 tools/target-dir-check && cargo fmt --all -- --check && cargo clippy --all --all-targets && python3 -m unittest discover -s tools 2>&1 && tasks check && python3 tools/upstream-report --check && python3 tools/package-pin --check"

# Prose-only trees: material docs and wiki contain Rust test inputs.
# Keep the repository-specific consistency checks in both commit paths.
docs_paths := "README.md AGENTS.md CONTRIBUTING.md docs/specs/*.md docs/plans/*.md docs/notes/*.md tasks/*.md"
docs_check_cmd := "python3 tools/ops-check && tasks check && python3 tools/upstream-report --check && python3 tools/package-pin --check"
# The hooks regenerate and stage the divergence report before checking it: its
# seam line counts move with nearly every source commit. `check` and CI only check.
stage_report_cmd := "python3 tools/upstream-report --stage"

# ci.yml has an unfiltered push trigger; its test job runs the full suite on origin.
ci_suite_refs := "refs/heads/* refs/tags/*"
ci_remote := "origin"
# Fixed fast set: config (including material parameters/wiki examples) and IPC.
# Compositor tests run in the affected inner loop and full CI suite. A HEAD-based
# selector would miss committed pushes; rerunning all compositor tests costs 30s.
push_fast_cmd := nextest_cmd + " --package niri-config --package niri-ipc"

# A fresh checkout, right after `git worktree add`: the hooks, .worktrees off the
# synced tree (work-link, when this machine has dotfiles; its own exit status when
# present), and a cargo target dir of its own (tools/target-dir-check).
setup_cmd := "git config core.hooksPath .githooks && if command -v work-link >/dev/null; then work-link --ensure .worktrees; fi && python3 tools/target-dir-check"

setup:
    {{tt}} setup -- sh -c '{{setup_cmd}}'

# Focused nextest arguments, preserved verbatim; override one_cmd for tooling tests.
test-one +args:
    {{tt}} test-one -- host-budget run -- sh -c '{{one_cmd}} "$@" 2>&1' test-one "$@"

# Affected-only: the inner loop.
test-fast:
    {{tt}} test-fast -- host-budget run -- sh -c '{{fast_cmd}}'

# The full suite.
test:
    {{tt}} test -- host-budget run -- sh -c '{{test_cmd}} -- --quiet'

# Format, clippy, tooling tests, tasks check.
check:
    {{tt}} check -- host-budget run -- sh -c '{{check_cmd}}'

gate: check test

# What the pre-commit hook runs: the report staged, then `check`'s command,
# under its own hook target.
hook-pre-commit:
    {{tt}} hook-pre-commit -- host-budget run -- sh -c '{{stage_report_cmd}} && {{check_cmd}}'

# Docs/task-only commits retain consistency checks without compiling Rust.
hook-pre-commit-docs:
    {{tt}} hook-pre-commit-docs -- host-budget run -- sh -c '{{stage_report_cmd}} && {{docs_check_cmd}}'

# CI-covered pushes: checks plus the fixed nextest set.
hook-pre-push-fast:
    {{tt}} hook-pre-push-fast -- host-budget run -- sh -c '{{check_cmd}} && {{push_fast_cmd}} 2>&1'

# Other pushes: the same commands as `gate`, under one hook target.
hook-pre-push:
    {{tt}} hook-pre-push -- host-budget run -- sh -c '{{check_cmd}} && {{test_cmd}} -- --quiet'

# CI runs the full suite with captured success output and quiet status lines.
ci-test:
    {{tt}} ci-test -- sh -c '{{test_cmd}} -- --quiet'

# CI's randomized-and-slow job; it sets RUN_SLOW_TESTS and the proptest limits.
ci-test-release:
    {{tt}} ci-test-release -- sh -c '{{test_cmd}} --release -- --quiet'

# Pin the Arch package on a commit (default HEAD) and derive pkgver, the source
# commit, and NIRI_BUILD_COMMIT from it. `check` verifies the three still agree.
package-pin commit="HEAD":
    python3 tools/package-pin {{commit}}

# Regenerate the upstream divergence report. Pass --drift for the drift block
# (needs `git fetch upstream --tags` first), --check to verify staged freshness.
upstream-report *args:
    python3 tools/upstream-report {{args}}
