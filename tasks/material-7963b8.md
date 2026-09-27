---
id: material-7963b8
title: "Drift canary --check compares tag commit identity, contradicting its tree contract"
status: doing
priority: 1
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-19T13:07:35Z
updated: 2026-09-27T10:21:19Z
started: 2026-09-27T10:21:19Z
depends: []
tags: [upstream, tooling, bug]
agent: "claude-code/claude-opus-5[1m]"
---

The weekly upstream-drift workflow has never passed: its only scheduled run (2026-09-14, run 34845581393) failed at `python3 tools/upstream-report --check` with `tag v26.04 resolves to 8ed0da44d974..., recorded tag_commit is aece2b0c4e1f...`, before the fetch and drift steps ran.

Cause: `resolve_baseline` in `tools/upstream-report` first compares `v26.04^{commit}` against `tag_commit` by identity. Locally the tag is the rewritten copy `aece2b0c`; on GitHub the fork's `v26.04` is upstream's tag object (`6a0a862b` -> `8ed0da44`), so a CI checkout resolves it differently. Both commits carry tree `7b010d1b`, and both `upstream-baseline.toml`'s header and the function's own docstring say validation compares trees, never commit identity — the first check does the opposite.

Fix: validate `<tag>^{tree}` against `tree` instead of comparing commits (keep `tag_commit` as the recorded local reference, or drop it from the required keys), add a tooling test that resolves the tag to a different commit with the same tree, then trigger the workflow with `gh workflow run upstream-drift.yml -R khughitt/niri-material` and confirm it reaches the drift step. Until this lands, the drift section of docs/materials/upstream-divergence.md only refreshes from local runs (last: 2026-09-06).

## Notes

- 2026-09-27T10:21:19Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:3f4a9869-e47a-4bc9-a90f-ec437ad135a0","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
