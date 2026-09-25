---
id: material-43f88c
title: "Adopt host-budget: cargo build, clippy and nextest under host-budget run"
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-24T19:38:55Z
updated: 2026-09-25T15:26:52Z
started: 2026-09-25T15:17:09Z
completed: 2026-09-25T15:26:52Z
depends: []
tags: [testing]
source: ops-6d19bb
model: claude-opus-5-5
agent: claude-code/claude-opus-5-5
---

ops docs/specs/2026-09-24-host-budget-design.md, Consumers. CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS come from the budget.

## Notes

- 2026-09-25T15:17:09Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:a80a2e70-e4ce-4a9b-887c-fc0b56920174","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-25T15:26:52Z (material-43f88c): done
  provenance: {"harness_session":"claude-code:a80a2e70-e4ce-4a9b-887c-fc0b56920174","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-25T15:26:52Z (material-43f88c): justfile: test-fast, test, check, hook-pre-commit and hook-pre-push run their command under host-budget run inside tt, so CARGO_BUILD_JOBS, RUST_TEST_THREADS and NEXTEST_TEST_THREADS come from the host budget; ci-test and ci-test-release stay unwrapped (CI has no ops tooling). No hard-coded -j or --test-threads existed in the recipes, and there is no tracked cargo or nextest config, so nothing to remove and no CI env to add. AGENTS.md notes the wrapper.
  provenance: {"harness_session":"claude-code:a80a2e70-e4ce-4a9b-887c-fc0b56920174","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
