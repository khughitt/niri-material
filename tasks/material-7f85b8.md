---
id: material-7f85b8
title: Pre-commit regenerates and stages the upstream divergence report
status: done
priority: 2
size: s
complexity: low
process: direct
owner: material-7f85b8
created: 2026-10-01T23:11:35Z
updated: 2026-10-02T00:59:18Z
started: 2026-10-02T00:57:24Z
completed: 2026-10-02T00:59:18Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

Re-copying tools/tt from ops changes its +N/-0 row in docs/materials/upstream-divergence.md (class C files are listed with line counts), so the pre-commit upstream-report --check fails until `just upstream-report` is rerun and staged. Class A was already made count-only for the same reason. Options: list class C paths without line counts, or have the re-copy step regenerate the report.

## Notes

- 2026-10-02T00:57:24Z (materials-26.04): Rescoped 2026-10-02 (owner: fix the pre-commit check): class B +/- counts change on nearly every src commit, not only on vendored re-copies, so --check failed on both material-519eeb and material-1d70db commits. Fix: upstream-report --stage regenerates from the index and stages the report, refusing when the report has unstaged edits; the pre-commit recipes run it before their checks. --check and CI unchanged, so a commit still cannot land without its inventory.
- 2026-10-02T00:57:24Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:57:28Z (material-7f85b8): resumed
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:59:18Z (material-7f85b8): done
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T00:59:18Z (material-7f85b8): upstream-report --stage regenerates the report from the index, splices into the staged document and stages it, refusing when the report has unstaged edits; hook-pre-commit and hook-pre-commit-docs run it before their checks. --check, just check and CI unchanged. 5 new tests; design spec updated. Verified live: this commit's own report refresh was staged by the hook.
  provenance: {"harness_session":"claude-code:cebfaf5f-51dd-49c1-ae0b-f56f976f9f14","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
